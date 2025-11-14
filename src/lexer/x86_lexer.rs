use std::{
    arch::x86_64::*,
    path::is_separator,
    simd::{Simd, cmp::{SimdPartialEq, SimdPartialOrd}},
    str::FromStr,
};

use minivec::MiniVec;

use crate::{
    error::ParserError,
    keyword::{Keyword, KeywordMap},
    token::{TokenKind, TokenTable},
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

    #[inline]
    fn skip_whitespace(&mut self) {
        let length = self.inner.len();
        let mut pos = self.position;
        if is_x86_feature_detected!("avx2") {
            while pos + 32 < length {
                let slice = Simd::<u8, 32>::from_slice(&self.inner[pos..pos+32]);

                let space = Simd::<u8, 32>::splat(b' ');
                let tab = Simd::<u8, 32>::splat(b'\t');
                let newline = Simd::<u8, 32>::splat(b'\n');
                let cr = Simd::<u8, 32>::splat(b'\r');

                let mask = slice.simd_eq(space) | slice.simd_eq(tab) | slice.simd_eq(newline) | slice.simd_eq(cr);
                if mask.all() {
                    pos += 32;
                } else {
                    let result = (!mask).to_bitmask();
                    let index = result.trailing_zeros() as usize;
                    pos += index;

                    break;
                }
            }

            if is_x86_feature_detected!("sse4.2") {
                while pos + 16 < length {
                    let slice = Simd::<u8, 16>::from_slice(&self.inner[pos..pos+16]);

                    let space = Simd::<u8, 16>::splat(b' ');
                    let tab = Simd::<u8, 16>::splat(b'\t');
                    let newline = Simd::<u8, 16>::splat(b'\n');
                    let cr = Simd::<u8, 16>::splat(b'\r');

                    let mask = slice.simd_eq(space) | slice.simd_eq(tab) | slice.simd_eq(newline) | slice.simd_eq(cr);
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

            if is_x86_feature_detected!("avx2") {
                while pos + 32 < length {
                    let slice = Simd::<u8, 32>::from_slice(&self.inner[pos..pos+32]);

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

                    let slice = Simd::<u8, 16>::from_slice(&self.inner[pos..pos+16]);

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
                if !c.is_ascii_digit() {
                    break;
                }
                pos += 1;
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

            if is_x86_feature_detected!("avx2") {
                while pos + 32 < length {
                    let slice = Simd::<u8, 32>::from_slice(&self.inner[pos..pos+32]);

                    let digit_mask = slice.simd_ge(Simd::splat(b'0' - 1)) & slice.simd_le(Simd::splat(b'9' + 1));
                    let lower_mask = slice.simd_ge(Simd::splat(b'a' - 1)) & slice.simd_le(Simd::splat(b'z' + 1));
                    let upper_mask = slice.simd_ge(Simd::splat(b'A' - 1)) & slice.simd_le(Simd::splat(b'Z' + 1));
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
                    let slice = Simd::<u8, 16>::from_slice(&self.inner[pos..pos+16]);

                    let digit_mask = slice.simd_ge(Simd::splat(b'0' - 1)) & slice.simd_le(Simd::splat(b'9' + 1));
                    let lower_mask = slice.simd_ge(Simd::splat(b'a' - 1)) & slice.simd_le(Simd::splat(b'z' + 1));
                    let upper_mask = slice.simd_ge(Simd::splat(b'A' - 1)) & slice.simd_le(Simd::splat(b'Z' + 1));
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
                if c.is_ascii_alphanumeric() || c == &b'_' {
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
            let lower_mask = source_chunk.simd_gt(Simd::<u8, 16>::splat(b'a' - 1)) & source_chunk.simd_lt(Simd::<u8, 16>::splat(b'z' + 1));

            let source_upper = source_chunk - (lower_mask.select(Simd::<u8, 16>::splat(32), Simd::<u8, 16>::splat(0)));

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
    #[inline]
    fn scan_string(&mut self, terminator: u8) -> Result<(TokenKind, usize, usize), ParserError> {
        let start = self.position;
        let mut pos = self.position + 1;
        let length = self.inner.len();

            if is_x86_feature_detected!("avx2") {
                while pos + 32 < length {
                    let slice = Simd::<u8, 32>::from_slice(&self.inner[pos..pos+32]);

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

    pub(crate) fn tokenize(&mut self) -> Result<TokenTable, ParserError> {
        let mut table = TokenTable::new();

        loop {
            self.skip_whitespace();

            let c = match self.inner.get(self.position) {
                Some(c) => *c,
                None => break,
            };

            match c {
                b'(' => {
                    let (kind, start, end) = (TokenKind::LeftParen, self.position, self.position);
                    table.push(kind, start, end);
                    self.position += 1;
                }
                b')' => {
                    let (kind, start, end) = (TokenKind::RightParen, self.position, self.position);
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
                b'<' => match self.inner.get(self.position + 1) {
                    Some(b'=') => {
                        table.push(TokenKind::LessEqual, self.position, self.position + 1);
                        self.position += 2;
                    }
                    Some(b'>') => {
                        table.push(TokenKind::NotEqual, self.position, self.position + 1);
                        self.position += 2;
                    }
                    _ => {
                        table.push(TokenKind::Less, self.position, self.position);
                        self.position += 1;
                    }
                },
                b'>' => match self.inner.get(self.position + 1) {
                    Some(b'=') => {
                        table.push(TokenKind::GreaterEqual, self.position, self.position + 1);
                        self.position += 2;
                    }
                    _ => {
                        table.push(TokenKind::Greater, self.position, self.position);
                        self.position += 1;
                    }
                },
                b'=' => {
                    table.push(TokenKind::Equal, self.position, self.position);
                    self.position += 1;
                }
                b',' => {
                    table.push(TokenKind::Comma, self.position, self.position);
                    self.position += 1;
                }
                b'+' => {
                    table.push(TokenKind::Plus, self.position, self.position);
                    self.position += 1;
                }
                b'-' => match self.inner.get(self.position + 1) {
                    Some(b'0'..=b'9') => {
                        let start = self.position;
                        self.position += 1;
                        let (kind, _, end) = self.scan_number()?;
                        table.push(kind, start, end);
                        self.position += 1;
                    }
                    _ => {
                        table.push(TokenKind::Subtract, self.position, self.position);
                        self.position += 1;
                    }
                },
                b'*' => {
                    table.push(TokenKind::Multiply, self.position, self.position);
                    self.position += 1;
                }
                b'/' => {
                    table.push(TokenKind::Divide, self.position, self.position);
                    self.position += 1;
                }
                b'%' => {
                    table.push(TokenKind::Mod, self.position, self.position);
                    self.position += 1;
                }
                _ => {
                    table.push(TokenKind::Unknown, self.position, self.position);
                    self.position += 1;
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
