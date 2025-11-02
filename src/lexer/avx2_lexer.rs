use std::arch::x86_64::*;

use crate::{
    error::ParserError,
    token::{TokenKind, TokenTable},
};

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
                let chunk_ptr = self.inner.as_ptr().add(self.position).cast::<i8>();
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
                if mask != -1 {
                    // 不是所有字符都是空白
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
                let chunk_ptr = self.inner.as_ptr().add(pos).cast::<i8>();
                let slice = _mm256_loadu_epi8(chunk_ptr);

                let min = _mm256_set1_epi8((b'0' - 1).cast_signed());
                let max = _mm256_set1_epi8((b'9' + 1).cast_signed());

                let min_mask = _mm256_cmpgt_epi8(slice, min);
                let max_mask = _mm256_cmpgt_epi8(max, slice);

                let cmp = _mm256_and_si256(min_mask, max_mask);
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

    #[inline]
    fn scan_identify(&mut self) -> Result<(TokenKind, usize, usize), ParserError> {
        let start = self.position;
        let mut pos = self.position;
        let length = self.inner.len();

        unsafe {
            while pos + 32 < length {
                let chunk_ptr = self.inner.as_ptr().add(pos).cast::<i8>();
                let slice = _mm256_loadu_epi8(chunk_ptr);

                let is_digit = _mm256_and_si256(
                    _mm256_cmpgt_epi8(slice, _mm256_set1_epi8((b'0' - 1).cast_signed())),
                    _mm256_cmpgt_epi8(_mm256_set1_epi8((b'9' + 1).cast_signed()), slice)
                );

                let is_lower = _mm256_and_si256(
                    _mm256_cmpgt_epi16(slice,_mm256_set1_epi8((b'a' - 1).cast_signed())),
                    _mm256_cmpgt_epi16(_mm256_set1_epi8((b'z' + 1).cast_signed()), slice),
                );

                let is_upper = _mm256_and_si256(
                    _mm256_cmpgt_epi8(slice,_mm256_set1_epi8((b'A' - 1).cast_signed())),
                    _mm256_cmpgt_epi8(_mm256_set1_epi8((b'Z' + 1).cast_signed()), slice),
                );

                let is_underline = _mm256_cmpeq_epi8(slice, _mm256_set1_epi8(b'_'.cast_signed()));

                let mask = _mm256_movemask_epi8(
                    _mm256_or_si256(
                        _mm256_or_si256(is_digit, is_underline),
                        _mm256_or_si256(is_lower, is_upper)
                    )
                );

                if mask != -1 {
                    let trailling_zeros = mask.trailing_zeros();
                    pos += trailling_zeros as usize;
                    break;
                }
                pos += 32;
            }

            dbg!(&pos);
            let tmp_pos = pos;
            for index in tmp_pos..length {
                let c = self.inner.get_unchecked(index);
                if (*c >= b'0' && *c <= b'9') || (*c >= b'a' && *c <= b'z') || (*c >= b'A' && *c <= b'Z') || *c == b'_' {
                    pos += 1;
                } else {
                    break;
                }
            }
        }

        let end = pos;
        self.position = pos;

        Ok((TokenKind::Identifier, start, end))
    }

    // 匹配字符串
    #[inline]
    fn scan_string(&mut self, terminator: u8) -> Result<(TokenKind, usize, usize), ParserError> {
        let start = self.position;
        let mut pos = self.position + 1;
        let length = self.inner.len();

        unsafe {
            while pos + 32 < length {
                let chunk_ptr = self.inner.as_ptr().add(pos).cast::<i8>();
                let slice = _mm256_loadu_epi8(chunk_ptr);

                let target = _mm256_set1_epi8(terminator.cast_signed());
                let is_target = _mm256_cmpeq_epi8(slice, target);
                let mask = _mm256_movemask_epi8(is_target);

                if mask != -1 {
                    let trailling_one = mask.trailing_zeros();
                    let tmp_pos = trailling_one as usize;
                    let prev_value = self.inner.get_unchecked(pos + tmp_pos - 1);

                    if *prev_value != b'\\' {
                        pos += tmp_pos;
                        break;
                    } else {
                        pos += tmp_pos + 1;
                    }
                } else {
                    pos += 32;
                }
            }

            let tmp_pos = pos;
            for index in tmp_pos..length {
                let c = self.inner.get_unchecked(index);
                let prev_c = self.inner.get_unchecked(index - 1);
                if *c == terminator && *prev_c != b'\\' {
                    break;
                }
                pos += 1;
            }

            let end = pos;
            self.position = pos;
            Ok((TokenKind::StringLiteral, start, end))
        }
    }

    pub(crate) fn tokenize(&mut self) -> Result<TokenTable, ParserError> {
        let mut table = TokenTable::new();

        loop {
            self.skip_whitespace();

            match self.inner.get(self.position) {
                Some(c) => match *c {
                    b'(' => {
                        let (kind, start, end) =
                            (TokenKind::LeftParen, self.position, self.position);
                        table.push(kind, start, end);
                        self.position += 1;
                    }
                    b')' => {
                        let (kind, start, end) =
                            (TokenKind::RightParen, self.position, self.position);
                        table.push(kind, start, end);
                        self.position += 1;
                    }
                    b'\'' => {
                        let (kind, start, end) = self.scan_string(b'\'')?;
                        table.push(kind, start, end);
                        self.position += 1;
                    }
                    b'"' => {
                        let (kind, start, end) = self.scan_string(b'"')?;
                        table.push(kind, start, end);
                        self.position += 1;
                    }
                    b'a'..=b'z' | b'A'..=b'Z' => {
                        let (kind, start, end) = self.scan_identify()?;
                        table.push(kind, start, end);
                        self.position += 1;
                    }
                    b'0'..=b'9' => {
                        let (kind, start, end) = self.scan_number()?;
                        table.push(kind, start, end);
                        self.position += 1;
                    }
                    _ => {
                        todo!()
                    }
                },
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
        let mut lexer = SimdLexer::new(
            r#"                 
                "#,
        );
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
        let mut lexer = SimdLexer::new("1234567890");
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::Number,],
                positions: vec![(0, 10)],
            }
        );

        let mut lexer = SimdLexer::new(
            "123451111111111111111111111111111111111111 2222222222222222222222222222",
        );
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::Number, TokenKind::Number],
                positions: vec![(0, 42), (43, 71)],
            }
        );
    }

    #[test]
    fn test_match_string() {
        let mut lexer = SimdLexer::new("'helloWorld'");
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::StringLiteral],
                positions: vec![(0, 11)],
            }
        );

        let mut lexer = SimdLexer::new(r#"'hello\\'World'"#);
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::StringLiteral],
                positions: vec![(0, 14)],
            }
        );

        let mut lexer =
            SimdLexer::new("'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'");
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::StringLiteral],
                positions: vec![(0, 60)],
            }
        );

        let mut lexer =
            SimdLexer::new("\'aaaaaaaaaaaaa\\'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\'");
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::StringLiteral],
                positions: vec![(0, 64)],
            }
        );
    }

    #[test]
    fn test_match_indentify() {
        let mut lexer = SimdLexer::new("asdfghjk");
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::Identifier],
                positions: vec![(0, 8)],
            }
        );

        let mut lexer = SimdLexer::new("qwertyuiopASDFGHJKL1234567890_zxcvbnm 1234567890");
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::Identifier, TokenKind::Number],
                positions: vec![(0, 37), (38, 48)],
            }
        );
    }
}
