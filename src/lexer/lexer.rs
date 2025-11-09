use std::{iter::Peekable, process::Termination, str::{CharIndices, FromStr}};

use crate::{
    error::ParserError, keyword::{Keyword, KeywordMap}, token::{TokenKind, TokenTable}
};

#[derive(Debug)]
pub(crate) struct Lexer<'a> {
    inner: Peekable<CharIndices<'a>>,
    text: &'a str,
    position: usize,
    keyword_map: KeywordMap,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(text: &'a str) -> Result<Self, ParserError> {
        Ok(Self {
            inner: text.char_indices().peekable(),
            text,
            position: 0,
            keyword_map: KeywordMap::new(),
        })
    }

    // 跳过空白符
    #[inline]
    fn skip_whitespace(&mut self) {
        while let Some((index, _)) = self.inner.next_if(|(_, c)| c.is_whitespace()) {
            self.position = index;
        }
    }

    // 匹配数字
    #[inline]
    fn match_number(&mut self) -> Result<(TokenKind, usize, usize), ParserError> {
        let start_position = self.position;
        while let Some((index, _)) = self.inner.next_if(|(_, c)| c.is_ascii_digit()) {
            self.position = index;
        }
        let end_position = self.position;

        Ok((TokenKind::Number, start_position, end_position))
    }

    // 匹配字面量
    #[inline]
    fn match_identify(&mut self) -> Result<(TokenKind, usize, usize), ParserError> {
        let start_position = self.position;
        while let Some((index, _)) = self.inner.next_if(|(_, c)| c.is_alphabetic() || *c == 'c') {
            self.position = index;
        }
        let end_position = self.position;

        let source = match self.text.get(start_position..=end_position) {
            Some(s) => s,
            None => return Err(ParserError::InvalidToken(start_position, end_position)),
        };

        if let Some(keyword) = self.maybe_keyword(source) {
            Ok((TokenKind::Keyword(keyword), start_position, end_position))            
        } else {
            Ok((TokenKind::Identifier, start_position, end_position))
        }
    }

    // 可能是关键词
    fn maybe_keyword(&self, source: &str) -> Option<Keyword> {
        let len = source.len();
        let tmp = source.chars().map(|c| c.to_ascii_uppercase()).collect::<String>();
        self.keyword_map.get(len)?
            .iter().find(|keyword| {
                keyword.as_str() == &tmp
            })
            .copied()
    }

    // 匹配字符串
    #[inline]
    fn scan_string(&mut self, terminator: char) -> Result<(TokenKind, usize, usize), ParserError> {
        let start_position = self.position;
        loop {
            match self.inner.peek() {
                Some((index_now, c)) if *c == terminator => {
                    self.position = *index_now;
                    self.inner.next();
                    break;
                }
                Some((index_now, c)) if *c == '\\' => {
                    self.position = *index_now;
                    self.inner.next();
                    // 跳过转义字符后的下一个字符
                    self.inner.next();
                }
                Some((index_now, _)) => {
                    self.position = *index_now;
                    self.inner.next();
                }
                None => return Err(ParserError::InvalidToken(start_position, self.position)),
            }
        }

        let end_position = self.position;

        Ok((TokenKind::StringLiteral, start_position, end_position))
    }

    // 词法分析主函数
    #[inline]
    pub(crate) fn tokenize(&mut self) -> Result<TokenTable, ParserError> {
        let mut table = TokenTable::new();

        loop {
            self.skip_whitespace();

            match self.inner.peek() {
                Some((index, c)) => {
                    self.position = *index;
                    match c {
                        '0'..='9' => {
                            let (kind, start, end) = self.match_number()?;
                            table.push(kind, start, end);
                        }
                        c if c.is_alphabetic() => {
                            let (kind, start, end) = self.match_identify()?;
                            table.push(kind, start, end);
                        }
                        '(' => {
                            let (kind, start, end) = (TokenKind::LeftParen, *index, *index);
                            self.position = *index;
                            self.inner.next();
                            table.push(kind, start, end);
                        }
                        ')' => {
                            let (kind, start, end) = (TokenKind::RightParen, *index, *index);
                            self.position = *index;
                            self.inner.next();
                            table.push(kind, start, end);
                        }
                        ',' => {
                            let (kind, start, end) = (TokenKind::Comma, *index, *index);
                            self.position = *index;
                            self.inner.next();
                            table.push(kind, start, end);
                        }
                        '\'' => {
                            self.inner.next();
                            let (kind, start, end) = self.scan_string('\'')?;
                            table.push(kind, start, end);
                        }
                        '"' => {
                            self.inner.next();
                            let (kind, start, end) = self.scan_string('"')?;
                            table.push(kind, start, end);
                        }
                        '\\' => {
                            let (kind, start, end) = (TokenKind::BackSlash, *index, *index);
                            self.position = *index;
                            self.inner.next();
                            table.push(kind, start, end);
                        }
                        ';' => {
                            let (kind, start, end) = (TokenKind::Eof, *index, *index);
                            self.position = *index;
                            self.inner.next();
                            table.push(kind, start, end);
                        }
                        '>' => {
                            let index_now = *index;
                            self.inner.next();
                            if let Some((index_next, _)) = self.inner.next_if(|(_, c)| *c == '=') {
                                let (kind, start, end) =
                                    (TokenKind::GreaterEqual, index_now, index_next);
                                self.position = index_next;
                                table.push(kind, start, end);
                            } else {
                                let (kind, start, end) = (TokenKind::Greater, index_now, index_now);
                                self.position = index_now;
                                table.push(kind, start, end);
                            }
                        }
                        '<' => {
                            let index_now = *index;
                            self.inner.next();

                            match self.inner.peek() {
                                Some((index_next, '>')) => {
                                    let (kind, start, end) =
                                        (TokenKind::NotEqual, index_now, *index_next);
                                    self.position = index_next + 1;
                                    self.inner.next();
                                    table.push(kind, start, end);
                                }
                                Some((index_next, '=')) => {
                                    let (kind, start, end) =
                                        (TokenKind::LessEqual, index_now, *index_next);
                                    self.position = index_next + 1;
                                    self.inner.next();
                                    table.push(kind, start, end);
                                }
                                _ => {
                                    let (kind, start, end) =
                                        (TokenKind::Less, index_now, index_now);
                                    self.position = index_now;
                                    table.push(kind, start, end);
                                }
                            }
                        }
                        '=' => {
                            let (kind, start, end) = (TokenKind::Equal, *index, *index);
                            self.position = *index;
                            self.inner.next();
                            table.push(kind, start, end);
                        }
                        '+' => {
                            let (kind, start, end) = (TokenKind::Plus, *index, *index);
                            self.position = *index;
                            self.inner.next();
                            table.push(kind, start, end);
                        }
                        '-' => {
                            let index_now = *index;
                            self.inner.next();
                            if let Some((index_next, _)) = self.inner.next_if(|(_, c)| c.is_alphanumeric()) {
                                let (kind, _, end) = self.match_number()?;
                                self.position = index_next;
                                table.push(kind, index_now, end);
                            } else {
                                let (kind, start, end) = (TokenKind::Subtract, index_now, index_now);
                                self.position = index_now;
                                self.inner.next();
                                table.push(kind, start, end);
                            }
                        }
                        '*' => {
                            let (kind, start, end) = (TokenKind::Multiply, *index, *index);
                            self.position = *index;
                            self.inner.next();
                            table.push(kind, start, end);
                        }
                        '/' => {
                            let (kind, start, end) = (TokenKind::Divide, *index, *index);
                            self.position = *index;
                            self.inner.next();
                            table.push(kind, start, end);
                        }
                        '%' => {
                            let (kind, start, end) = (TokenKind::Mod, *index, *index);
                            self.position = *index;
                            self.inner.next();
                            table.push(kind, start, end);
                        }
                        _ => {
                            let (kind, start, end) = (TokenKind::Unknown, *index, *index);
                            self.position = *index;
                            self.inner.next();
                            table.push(kind, start, end);
                        }
                    }
                }
                None => break,
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
        let mut lexer = Lexer::new("   \t\n   hello").unwrap();
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            TokenTable {
                tokens: vec![TokenKind::Identifier],
                positions: vec![(8, 12)],
            }
        );
    }

    #[test]
    fn test_match_number() {
        let mut lexer = Lexer::new("12345").unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::Number,],
                positions: vec![(0, 4)],
            }
        );

        let mut lexer = Lexer::new("-12345").unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::Number,],
                positions: vec![(0, 5)],
            }
        );
    }

    #[test]
    fn test_match_string() {
        let mut lexer = Lexer::new("helloWorld").unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::Identifier],
                positions: vec![(0, 9)],
            }
        );
    }

    #[test]
    fn test_tokenize_numbers() {
        let mut lexer = Lexer::new("123 456").unwrap();
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            TokenTable {
                tokens: vec![TokenKind::Number, TokenKind::Number],
                positions: vec![(0, 2), (4, 6)],
            }
        );
    }

    #[test]
    fn test_tokenize_strings() {
        let mut lexer = Lexer::new("hello world").unwrap();
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            TokenTable {
                tokens: vec![TokenKind::Identifier, TokenKind::Identifier],
                positions: vec![(0, 4), (6, 10)],
            }
        )
    }

    #[test]
    fn test_tokenize_punctuation() {
        let mut lexer = Lexer::new("(),'\"\\;'").unwrap();
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            TokenTable {
                tokens: vec![
                    TokenKind::LeftParen,
                    TokenKind::RightParen,
                    TokenKind::Comma,
                    TokenKind::StringLiteral
                ],
                positions: vec![(0, 0), (1, 1), (2, 2), (3, 7)],
            }
        );
    }

    #[test]
    fn test_tokenize_mixed() {
        let mut lexer = Lexer::new("func(123, 'abc');").unwrap();
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            TokenTable {
                tokens: vec![
                    TokenKind::Identifier,
                    TokenKind::LeftParen,
                    TokenKind::Number,
                    TokenKind::Comma,
                    TokenKind::StringLiteral,
                    TokenKind::RightParen,
                    TokenKind::Eof
                ],
                positions: vec![(0, 3), (4, 4), (5, 7), (8, 8), (10, 14), (15, 15), (16, 16)],
            }
        )
    }

    #[test]
    fn test_tokenize_cmp() {
        let mut lexer = Lexer::new("a > b >= c < d <= e <> f = g").unwrap();
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
    fn test_tokenize_unknown() {
        let mut lexer = Lexer::new("@#$").unwrap();
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            TokenTable {
                tokens: vec![TokenKind::Unknown, TokenKind::Unknown, TokenKind::Unknown],
                positions: vec![(0, 0), (1, 1), (2, 2)],
            }
        )
    }

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("").unwrap();
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
    fn test_whitespace_handling() {
        let mut lexer = Lexer::new("  hello   world  ").unwrap();
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            TokenTable {
                tokens: vec![TokenKind::Identifier, TokenKind::Identifier],
                positions: vec![(2, 6), (10, 14)],
            }
        );
    }

    #[test]
    fn test_tokenize_line_break() {
        let mut lexer = Lexer::new("\n").unwrap();
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
    fn test_keyword() {
        let mut lexer = Lexer::new("select from").unwrap();
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            TokenTable {
                tokens: vec![TokenKind::Keyword(Keyword::Select), TokenKind::Keyword(Keyword::From)],
                positions: vec![(0, 5), (7, 10)],
            }
        );
    }

    #[test]
    fn test_sql() {
        let mut lexer = Lexer::new("select * from a").unwrap();
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
