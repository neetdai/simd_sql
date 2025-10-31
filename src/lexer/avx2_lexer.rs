use std::arch::x86_64::*;

use crate::{error::ParserError, token::{TokenKind, TokenTable}};

const UNKNOWN: u8 = 0;
const WHITESPACE: u8 = 1;
const LINE_BREAK: u8 = 2;
const NUMBER: u8 = 3;
const STRING_LITERAL: u8 = 4;
const EOF: u8 = 5;
const LEFT_PAREN: u8 = 6;
const RIGHT_PAREN: u8 = 7;
const SINGLE_QUOTATION: u8 = 8;
const DOUBLE_QUOTATION: u8 = 9;
const BACKSLASH: u8 = 10;
const COMMA: u8 = 11;

#[derive(Debug)]
pub(crate) struct SimdLexer<'a> {
    inner: &'a [u8],
    position: usize,
}

impl<'a> SimdLexer<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self {
            inner: text.as_bytes(),
            position: 0,
        }
    }

    #[inline]
    fn skip_whitespace(&mut self) {
        let length = self.inner.len();
        let mut pos = self.position;
        unsafe {
            while pos + 32 < length {
                let chunk_ptr = self.inner.as_ptr().add(self.position) as *const i8;
                let slice = _mm256_loadu_epi8(chunk_ptr);

                let space = _mm256_set1_epi8(b' '.cast_signed());
                let tab = _mm256_set1_epi8(b'\t'.cast_signed());
                let cr = _mm256_set1_epi8(b'\r'.cast_signed());
                let change_line = _mm256_set1_epi8(b'\n'.cast_signed());

                let is_space = _mm256_cmpeq_epi8(slice, space);
                let is_tab = _mm256_cmpeq_epi8(slice, tab);
                let is_cr = _mm256_cmpeq_epi8(slice, cr);
                let is_change_line = _mm256_cmpeq_epi8(slice, change_line);

                let whitespace_mask = _mm256_or_si256(
                    _mm256_or_si256(is_space, is_tab),
                    _mm256_or_si256(is_cr, is_change_line),
                );

                // 检查是否所有字符都是空白
                let mask = _mm256_movemask_epi8(whitespace_mask);
                if mask != -1 { // 不是所有字符都是空白
                    let trailing_zeros = (!mask).trailing_zeros() as usize;
                    pos += trailing_zeros;
                    break;
                }
            
                pos += 32;
            }

            let tmp_pos = pos;
            for index in tmp_pos..length {
                let c = self.inner.get_unchecked(index);
                if *c != b' ' && *c != b'\t' && *c != b'\r' && *c != b'\n' {
                    break;
                }
                pos += 1;
            }
            self.position = pos;
        }
    }
    
    #[inline]
    fn scan_number(&mut self) -> Result<(TokenKind, usize, usize), ParserError> {
        let start = self.position;
        let mut pos = start;
        let length = self.inner.len();
        
        unsafe {
            while pos + 32 <= length {
                let chunk_ptr = self.inner.as_ptr().add(pos) as *const i8;
                let slice = _mm256_loadu_epi8(chunk_ptr);

                let min = _mm256_set1_epi8(b'0'.cast_signed());
                let max = _mm256_set1_epi8(b'9'.cast_signed());

                let min_mask = _mm256_cmpeq_epi8(slice, min);
                let max_mask = _mm256_cmpeq_epi8(slice, max);

                let cmp = _mm256_or_si256(min_mask, max_mask);
                let mask = _mm256_movemask_epi8(cmp);

                if mask != -1 {
                    let trailling_zeros = mask.trailing_zeros();
                    pos += trailling_zeros as usize;
                    break;
                }
                pos += 32;
            }

            let tmp_pos = pos;
            for index in tmp_pos..length {
                let c = self.inner.get_unchecked(index);
                if *c < b'0' || *c > b'9' {
                    break;
                }
                pos += 1;
            }
        }
        self.position = pos;
        let end = pos;

        Ok((TokenKind::Number, start, end))
    }

    pub(crate) fn tokenize(&mut self) -> Result<TokenTable, ParserError> {
        let mut table = TokenTable::new();

        loop {
            self.skip_whitespace();

            match self.inner.get(self.position) {
                Some(c) => {
                    match *c {
                        b'0'..=b'9' => {
                            let (kind, start, end) = self.scan_number()?;
                            table.push(kind, start, end);
                        }
                        _ => {
                            todo!()
                        }
                    }
                }
                None => {
                    break;
                }
            }
        }

        Ok(table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{TokenKind, TokenTable};

    #[test]
    fn test_skip_whitespace() {
        let mut lexer = SimdLexer::new(r#"                 
                "#);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            TokenTable {
                tokens: vec![],
                positions: vec![],
            }
        );
    }

    #[test]
    fn test_match_number() {
        let mut lexer = SimdLexer::new("123451111111111111111111111111111111111111");
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::Number,],
                positions: vec![(0, 42)],
            }
        );
    }
}