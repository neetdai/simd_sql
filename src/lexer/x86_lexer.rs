use std::{
    arch::x86_64::*,
    path::is_separator,
    simd::{
        Simd,
        cmp::{SimdPartialEq, SimdPartialOrd},
    },
    str::FromStr,
};

use minivec::MiniVec;

use crate::{
    error::ParserError,
    keyword::{Keyword, KeywordMap},
    token::{TokenKind, TokenTable},
};

// ============================================================================
// 1. 静态查找表 (Lookup Table) - 消除分支预测失败
// ============================================================================

// 字符分类标志位
const C_UNK: u8 = 0; // 未知/非法
const C_WSP: u8 = 1 << 0; // 空白 (Whitespace)
const C_DIG: u8 = 1 << 1; // 数字 (0-9)
const C_ALP: u8 = 1 << 2; // 字母 (a-z, A-Z, _)
const C_QUO: u8 = 1 << 3; // 引号 (' " `)
const C_SYM: u8 = 1 << 4; // 符号 (+ - * / etc)

// 256字节的映射表，将 byte 映射到分类
static CHAR_TABLE: [u8; 256] = {
    let mut t = [C_UNK; 256];

    // 设置空白
    t[b' ' as usize] = C_WSP;
    t[b'\t' as usize] = C_WSP;
    t[b'\n' as usize] = C_WSP;
    t[b'\r' as usize] = C_WSP;

    // 设置数字
    let mut i = b'0';
    while i <= b'9' {
        t[i as usize] |= C_DIG | C_ALP;
        i += 1;
    }

    // 设置字母 (包含下划线)
    let mut i = b'a';
    while i <= b'z' {
        t[i as usize] |= C_ALP;
        i += 1;
    }
    let mut i = b'A';
    while i <= b'Z' {
        t[i as usize] |= C_ALP;
        i += 1;
    }
    t[b'_' as usize] |= C_ALP;

    // 设置引号
    t[b'\'' as usize] = C_QUO;
    t[b'"' as usize] = C_QUO;

    // 设置符号
    let syms = b"+-*/%()<>=,;.\\";
    let mut j = 0;
    while j < syms.len() {
        t[syms[j] as usize] = C_SYM;
        j += 1;
    }
    t
};

#[derive(Debug)]
pub(crate) struct SimdLexer<'a> {
    inner: &'a [u8],
    position: usize,
    keyword_map: &'a KeywordMap,
}

impl<'a> SimdLexer<'a> {
    pub(crate) fn new(text: &'a str, keyword_map: &'a KeywordMap) -> Result<Self, ParserError> {
        Ok(Self {
            inner: text.as_bytes(),
            position: 0,
            keyword_map,
        })
    }

    // #[inline]
    fn skip_whitespace(&mut self) {
        let length = self.inner.len();
        let mut pos = self.position;
        if is_x86_feature_detected!("avx2") {
            while pos + 32 < length {
                let slice = Simd::<u8, 32>::from_slice(&self.inner[pos..pos + 32]);

                let space = Simd::<u8, 32>::splat(b' ');
                let tab = Simd::<u8, 32>::splat(b'\t');
                let newline = Simd::<u8, 32>::splat(b'\n');
                let cr = Simd::<u8, 32>::splat(b'\r');

                let mask = slice.simd_eq(space)
                    | slice.simd_eq(tab)
                    | slice.simd_eq(newline)
                    | slice.simd_eq(cr);
                if mask.all() {
                    pos += 32;
                } else {
                    let result = (!mask).to_bitmask();
                    let index = result.trailing_zeros() as usize;
                    pos += index;

                    break;
                }
            }
        }

        if is_x86_feature_detected!("sse4.2") {
            while pos + 16 < length {
                let slice = Simd::<u8, 16>::from_slice(&self.inner[pos..pos + 16]);

                let space = Simd::<u8, 16>::splat(b' ');
                let tab = Simd::<u8, 16>::splat(b'\t');
                let newline = Simd::<u8, 16>::splat(b'\n');
                let cr = Simd::<u8, 16>::splat(b'\r');

                let mask = slice.simd_eq(space)
                    | slice.simd_eq(tab)
                    | slice.simd_eq(newline)
                    | slice.simd_eq(cr);
                if mask.all() {
                    pos += 16;
                } else {
                    let result = (!mask).to_bitmask();
                    let index = result.trailing_zeros() as usize;
                    pos += index;

                    break;
                }
            }
        }

        let tmp_pos = pos;
        for index in tmp_pos..length {
            let c = &self.inner[index];
            if (CHAR_TABLE[*c as usize] & C_WSP) == 0 {
                break;
            }
            pos += 1;
        }
        self.position = pos;
    }

    // #[inline]
    fn scan_number(&mut self) -> Result<(TokenKind, usize, usize), ParserError> {
        let start = self.position;
        let mut pos = start;
        let length = self.inner.len();

        if is_x86_feature_detected!("avx2") {
            while pos + 32 < length {
                let slice = Simd::<u8, 32>::from_slice(&self.inner[pos..pos + 32]);

                let min = Simd::<u8, 32>::splat(b'0' - 1);
                let max = Simd::<u8, 32>::splat(b'9' + 1);
                let mask = slice.simd_gt(min) & slice.simd_lt(max);

                if mask.all() {
                    pos += 32;
                } else {
                    let result = mask.to_bitmask();
                    let index = result.trailing_zeros() as usize;
                    pos += index;
                    break;
                }
            }
        }

        if is_x86_feature_detected!("sse4.2") {
            while pos + 16 < length {
                let slice = Simd::<u8, 16>::from_slice(&self.inner[pos..pos + 16]);

                let min = Simd::<u8, 16>::splat(b'0' - 1);
                let max = Simd::<u8, 16>::splat(b'9' + 1);

                let mask = slice.simd_ge(min) & slice.simd_le(max);

                if mask.all() {
                    pos += 16;
                } else {
                    let result = mask.to_bitmask();
                    let trailling_zeros = result.trailing_zeros();
                    pos += trailling_zeros as usize;
                    break;
                }
            }
        }

        let tmp_pos = pos;
        for index in tmp_pos..length {
            let c = &self.inner[index];
            if CHAR_TABLE[*c as usize] & C_DIG == 0 {
                break;
            }
            pos += 1;
        }

        self.position = pos;
        let end = pos;

        Ok((TokenKind::Number, start, end))
    }

    // #[inline]
    fn scan_identify(&mut self) -> Result<(TokenKind, usize, usize), ParserError> {
        let start = self.position;
        let mut pos = self.position;
        let length = self.inner.len();

        if is_x86_feature_detected!("avx2") {
            while pos + 32 < length {
                let slice = Simd::<u8, 32>::from_slice(&self.inner[pos..pos + 32]);

                let digit_mask =
                    slice.simd_ge(Simd::splat(b'0' - 1)) & slice.simd_le(Simd::splat(b'9' + 1));
                let lower_mask =
                    slice.simd_ge(Simd::splat(b'a' - 1)) & slice.simd_le(Simd::splat(b'z' + 1));
                let upper_mask =
                    slice.simd_ge(Simd::splat(b'A' - 1)) & slice.simd_le(Simd::splat(b'Z' + 1));
                let underline_mask = slice.simd_eq(Simd::splat(b'_'));

                let mask = digit_mask | lower_mask | upper_mask | underline_mask;

                if mask.all() {
                    pos += 32;
                } else {
                    let result = mask.to_bitmask();
                    let trailling_zeros = result.trailing_zeros();
                    pos += trailling_zeros as usize;
                    break;
                }
            }
        }

        if is_x86_feature_detected!("sse4.2") {
            while pos + 16 < length {
                let slice = Simd::<u8, 16>::from_slice(&self.inner[pos..pos + 16]);

                let digit_mask =
                    slice.simd_ge(Simd::splat(b'0' - 1)) & slice.simd_le(Simd::splat(b'9' + 1));
                let lower_mask =
                    slice.simd_ge(Simd::splat(b'a' - 1)) & slice.simd_le(Simd::splat(b'z' + 1));
                let upper_mask =
                    slice.simd_ge(Simd::splat(b'A' - 1)) & slice.simd_le(Simd::splat(b'Z' + 1));
                let underline_mask = slice.simd_eq(Simd::splat(b'_'));

                let mask = digit_mask | lower_mask | upper_mask | underline_mask;

                if mask.all() {
                    pos += 16;
                } else {
                    let result = mask.to_bitmask();
                    let trailling_zeros = result.trailing_zeros();
                    pos += trailling_zeros as usize;
                    break;
                }
            }
        }

        let tmp_pos = pos;
        for index in tmp_pos..length {
            let c = &self.inner[index];
            if (CHAR_TABLE[*c as usize] & C_ALP) != 0 {
                pos = index;
            } else {
                break;
            }
        }

        let end = pos;
        self.position = pos;

        let source = match self.inner.get(start..=end) {
            Some(s) => s,
            None => return Err(ParserError::InvalidToken(start, end)),
        };

        if let Some(keyword) = self.maybe_keyword(source) {
            Ok((TokenKind::Keyword(keyword), start, end))
        } else {
            Ok((TokenKind::Identifier, start, end))
        }
    }

    // 可能是关键词
    fn maybe_keyword(&self, source: &[u8]) -> Option<Keyword> {
        let len = source.len();
        let list = self.keyword_map.get(len)?;

        if is_x86_feature_detected!("sse4.2") {
            let mut source_array = [0u8; 16];
            source_array[0..len].copy_from_slice(source);

            let source_chunk = Simd::from_array(source_array);
            let lower_mask = source_chunk.simd_gt(Simd::<u8, 16>::splat(b'a' - 1))
                & source_chunk.simd_lt(Simd::<u8, 16>::splat(b'z' + 1));

            let source_upper = source_chunk
                - (lower_mask.select(Simd::<u8, 16>::splat(32), Simd::<u8, 16>::splat(0)));

            for keyword in list.iter() {
                let key_len = keyword.as_str().as_bytes().len();
                let mut keyword_array = [0u8; 16];
                keyword_array[0..key_len].copy_from_slice(keyword.as_str().as_bytes());
                let keyword_chunk = Simd::from_array(keyword_array);
                if keyword_chunk.simd_eq(source_upper).all() {
                    return Some(*keyword);
                }
            }
            None
        } else {
            let tmp = source
                .iter()
                .copied()
                .map(|c| if c.is_ascii_lowercase() { c - 32 } else { c })
                .collect::<MiniVec<u8>>();
            list.iter()
                .copied()
                .find(|keyword| keyword.as_str().as_bytes() == tmp.as_slice())
        }
    }

    // 匹配字符串
    // #[inline]
    fn scan_string(&mut self, terminator: u8) -> Result<(TokenKind, usize, usize), ParserError> {
        let start = self.position;
        let mut pos = self.position + 1;
        let length = self.inner.len();

        if is_x86_feature_detected!("avx2") {
            while pos + 32 < length {
                let slice = Simd::<u8, 32>::from_slice(&self.inner[pos..pos + 32]);

                let target = Simd::from_array([terminator; 32]);
                let mask = slice.simd_eq(target);

                if mask.any() {
                    let result = mask.to_bitmask();
                    let index = result.trailing_zeros() as usize;
                    let prev_value = self.inner[pos + index - 1];
                    if prev_value != b'\\' {
                        pos += index;
                        break;
                    } else {
                        pos += index + 1;
                    }
                } else {
                    pos += 32;
                }
            }
        }

        if is_x86_feature_detected!("sse4.2") {
            while pos + 16 < length {
                let slice = Simd::<u8, 16>::from_slice(&self.inner[pos..pos + 16]);
                let target = Simd::from_array([terminator; 16]);
                let mask = slice.simd_eq(target);

                if mask.any() {
                    let result = mask.to_bitmask();
                    let index = result.trailing_zeros() as usize;
                    let prev_value = self.inner[pos + index - 1];
                    if prev_value != b'\\' {
                        pos += index;
                        break;
                    } else {
                        pos += index + 1;
                    }
                } else {
                    pos += 16;
                }
            }
        }

        let tmp_pos = pos;
        for index in tmp_pos..length {
            let c = &self.inner[index];
            let prev_c = &self.inner[index - 1];
            if *c == terminator && *prev_c != b'\\' {
                break;
            }
            pos += 1;
        }

        let end = pos;
        self.position = pos;
        Ok((TokenKind::StringLiteral, start, end))
    }

    fn scan_symbol(&mut self, table: &mut TokenTable) -> Result<(), ParserError> {
        let start = self.position;
        let end = self.position;
        match self.inner.get(self.position) {
            Some(b'(') => {
                table.push(TokenKind::LeftParen, start, end);
                self.position += 1;
            }
            Some(b')') => {
                table.push(TokenKind::RightParen, start, end);
                self.position += 1;
            }
            Some(b'<') => match self.inner.get(self.position + 1) {
                Some(b'=') => {
                    table.push(TokenKind::LessEqual, self.position, self.position + 1);
                    self.position += 2;
                }
                Some(b'>') => {
                    table.push(TokenKind::NotEqual, self.position, self.position + 1);
                    self.position += 2;
                }
                _ => {
                    table.push(TokenKind::Less, start, end);
                    self.position += 1;
                }
            },
            Some(b'>') => match self.inner.get(self.position + 1) {
                Some(b'=') => {
                    table.push(TokenKind::GreaterEqual, self.position, self.position + 1);
                    self.position += 2;
                }
                _ => {
                    table.push(TokenKind::Greater, start, end);
                    self.position += 1;
                }
            },
            Some(b'=') => {
                table.push(TokenKind::Equal, start, end);
                self.position += 1;
            }
            Some(b'.') => {
                table.push(TokenKind::Dot, start, end);
                self.position += 1;
            }
            Some(b',') => {
                table.push(TokenKind::Comma, start, end);
                self.position += 1;
            }
            Some(b'+') => {
                table.push(TokenKind::Plus, start, end);
                self.position += 1;
            }
            Some(b'-') => match self.inner.get(self.position + 1) {
                Some(b'0'..=b'9') => {
                    let start = self.position;
                    self.position += 1;
                    let (kind, _, end) = self.scan_number()?;
                    table.push(kind, start, end);
                    self.position += 1;
                }
                _ => {
                    table.push(TokenKind::Subtract, start, end);
                    self.position += 1;
                }
            },
            Some(b'*') => {
                table.push(TokenKind::Multiply, start, end);
                self.position += 1;
            }
            Some(b'/') => {
                table.push(TokenKind::Divide, start, end);
                self.position += 1;
            }
            Some(b'%') => {
                table.push(TokenKind::Mod, start, end);
                self.position += 1;
            }
            Some(b';') => {
                table.push(TokenKind::Eof, start, end);
                self.position += 1;
            }
            _ => return Err(ParserError::InvalidToken(start, end)),
        };
        Ok(())
    }

    pub(crate) fn tokenize(&mut self) -> Result<TokenTable, ParserError> {
        let mut table = TokenTable::with_capacity(self.inner.len() / 4);

        loop {
            self.skip_whitespace();

            let c = match self.inner.get(self.position) {
                Some(c) => *c,
                None => break,
            };

            let char_class = CHAR_TABLE[c as usize];

            if (char_class & C_ALP) != 0 {
                if (char_class & C_DIG) != 0 {
                    let (kind, start, end) = self.scan_number()?;
                    table.push(kind, start, end);
                    self.position += 1;
                } else {
                    let (kind, start, end) = self.scan_identify()?;
                    table.push(kind, start, end);
                    self.position += 1;
                }
            } else if (char_class & C_SYM) != 0 {
                self.scan_symbol(&mut table)?;
            } else if (char_class & C_QUO) != 0 {
                let (kind, start, end) = self.scan_string(c)?;
                table.push(kind, start, end);
                self.position += 1;
            } else {
                table.push(TokenKind::Unknown, self.position, self.position);
                self.position += 1;
            }

            // match c {
            //     b'(' => {
            //         let (kind, start, end) = (TokenKind::LeftParen, self.position, self.position);
            //         table.push(kind, start, end);
            //         self.position += 1;
            //     }
            //     b')' => {
            //         let (kind, start, end) = (TokenKind::RightParen, self.position, self.position);
            //         table.push(kind, start, end);
            //         self.position += 1;
            //     }
            //     b'\'' => {
            //         let (kind, start, end) = self.scan_string(b'\'')?;
            //         table.push(kind, start, end);
            //         self.position += 1;
            //     }
            //     b'"' => {
            //         let (kind, start, end) = self.scan_string(b'"')?;
            //         table.push(kind, start, end);
            //         self.position += 1;
            //     }
            //     b'a'..=b'z' | b'A'..=b'Z' => {
            //         let (kind, start, end) = self.scan_identify()?;
            //         table.push(kind, start, end);
            //         self.position += 1;
            //     }
            //     b'0'..=b'9' => {
            //         let (kind, start, end) = self.scan_number()?;
            //         table.push(kind, start, end);
            //         self.position += 1;
            //     }
            //     b'<' => match self.inner.get(self.position + 1) {
            //         Some(b'=') => {
            //             table.push(TokenKind::LessEqual, self.position, self.position + 1);
            //             self.position += 2;
            //         }
            //         Some(b'>') => {
            //             table.push(TokenKind::NotEqual, self.position, self.position + 1);
            //             self.position += 2;
            //         }
            //         _ => {
            //             table.push(TokenKind::Less, self.position, self.position);
            //             self.position += 1;
            //         }
            //     },
            //     b'>' => match self.inner.get(self.position + 1) {
            //         Some(b'=') => {
            //             table.push(TokenKind::GreaterEqual, self.position, self.position + 1);
            //             self.position += 2;
            //         }
            //         _ => {
            //             table.push(TokenKind::Greater, self.position, self.position);
            //             self.position += 1;
            //         }
            //     },
            //     b'=' => {
            //         table.push(TokenKind::Equal, self.position, self.position);
            //         self.position += 1;
            //     }
            //     b',' => {
            //         table.push(TokenKind::Comma, self.position, self.position);
            //         self.position += 1;
            //     }
            //     b'+' => {
            //         table.push(TokenKind::Plus, self.position, self.position);
            //         self.position += 1;
            //     }
            //     b'-' => match self.inner.get(self.position + 1) {
            //         Some(b'0'..=b'9') => {
            //             let start = self.position;
            //             self.position += 1;
            //             let (kind, _, end) = self.scan_number()?;
            //             table.push(kind, start, end);
            //             self.position += 1;
            //         }
            //         _ => {
            //             table.push(TokenKind::Subtract, self.position, self.position);
            //             self.position += 1;
            //         }
            //     },
            //     b'*' => {
            //         table.push(TokenKind::Multiply, self.position, self.position);
            //         self.position += 1;
            //     }
            //     b'/' => {
            //         table.push(TokenKind::Divide, self.position, self.position);
            //         self.position += 1;
            //     }
            //     b'%' => {
            //         table.push(TokenKind::Mod, self.position, self.position);
            //         self.position += 1;
            //     }
            //     _ => {
            //         table.push(TokenKind::Unknown, self.position, self.position);
            //         self.position += 1;
            //     }
            // }
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
        let keyword_map = KeywordMap::new();
        let mut lexer = SimdLexer::new(
            r#"                 
                "#,
            &keyword_map,
        )
        .unwrap();
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
        let keyword_map = KeywordMap::new();
        let mut lexer = SimdLexer::new("1234567890", &keyword_map).unwrap();
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
            &keyword_map,
        )
        .unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::Number, TokenKind::Number],
                positions: vec![(0, 42), (43, 71)],
            }
        );

        let mut lexer = SimdLexer::new("-123", &keyword_map).unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::Number],
                positions: vec![(0, 4)],
            }
        );
    }

    #[test]
    fn test_tokenize_cmp() {
        let keyword_map = KeywordMap::new();
        let mut lexer = SimdLexer::new("a > b >= c < d <= e <> f = g", &keyword_map).unwrap();
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            TokenTable {
                tokens: vec![
                    TokenKind::Identifier,
                    TokenKind::Greater,
                    TokenKind::Identifier,
                    TokenKind::GreaterEqual,
                    TokenKind::Identifier,
                    TokenKind::Less,
                    TokenKind::Identifier,
                    TokenKind::LessEqual,
                    TokenKind::Identifier,
                    TokenKind::NotEqual,
                    TokenKind::Identifier,
                    TokenKind::Equal,
                    TokenKind::Identifier
                ],
                positions: vec![
                    (0, 0),
                    (2, 2),
                    (4, 4),
                    (6, 7),
                    (9, 9),
                    (11, 11),
                    (13, 13),
                    (15, 16),
                    (18, 18),
                    (20, 21),
                    (23, 23),
                    (25, 25),
                    (27, 27)
                ],
            }
        )
    }

    #[test]
    fn test_match_string() {
        let keyword_map = KeywordMap::new();
        let mut lexer = SimdLexer::new("'helloWorld'", &keyword_map).unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::StringLiteral],
                positions: vec![(0, 11)],
            }
        );

        let mut lexer = SimdLexer::new(r#"'hello\\'World'"#, &keyword_map).unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::StringLiteral],
                positions: vec![(0, 14)],
            }
        );

        let mut lexer = SimdLexer::new(
            "'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'",
            &keyword_map,
        )
        .unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::StringLiteral],
                positions: vec![(0, 60)],
            }
        );

        let mut lexer = SimdLexer::new(
            "\'aaaaaaaaaaaaa\\'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\'",
            &keyword_map,
        )
        .unwrap();
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
        let keyword_map = KeywordMap::new();
        let mut lexer = SimdLexer::new("asdfghjk", &keyword_map).unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::Identifier],
                positions: vec![(0, 7)],
            }
        );

        let mut lexer = SimdLexer::new(
            "qwertyuiopASDFGHJKL1234567890_zxcvbnm 1234567890",
            &keyword_map,
        )
        .unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::Identifier, TokenKind::Number],
                positions: vec![(0, 36), (38, 48)],
            }
        );
    }

    #[test]
    fn test_keyword() {
        let keyword_map = KeywordMap::new();
        let mut lexer = SimdLexer::new("select from", &keyword_map).unwrap();
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            TokenTable {
                tokens: vec![
                    TokenKind::Keyword(Keyword::Select),
                    TokenKind::Keyword(Keyword::From)
                ],
                positions: vec![(0, 5), (7, 10)],
            }
        );
    }

    #[test]
    fn test_sql() {
        let keyword_map = KeywordMap::new();
        let mut lexer = SimdLexer::new("select * from a", &keyword_map).unwrap();
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            TokenTable {
                tokens: vec![
                    TokenKind::Keyword(Keyword::Select),
                    TokenKind::Multiply,
                    TokenKind::Keyword(Keyword::From),
                    TokenKind::Identifier,
                ],
                positions: vec![(0, 5), (7, 7), (9, 12), (14, 14)],
            }
        );
    }
}
