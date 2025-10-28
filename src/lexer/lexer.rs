use std::{iter::Peekable, str::CharIndices};

use crate::{
    error::ParserError,
    token::{Token, TokenKind, TokenTable},
};

#[derive(Debug)]
pub(crate) struct Lexer<'a> {
    inner: Peekable<CharIndices<'a>>,
    text: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self {
            inner: text.char_indices().peekable(),
            text,
            position: 0,
        }
    }

    // 跳过空白符
    #[inline]
    fn skip_whitespace(&mut self) {
        while let Some((index, _)) = self
            .inner
            .next_if(|(_, c)| *c == ' ' || *c == '\t' || *c == '\r')
        {
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

    // 匹配字符串
    #[inline]
    fn match_identify(&mut self) -> Result<(TokenKind, usize, usize), ParserError> {
        let start_position = self.position;
        while let Some((index, _)) = self.inner.next_if(|(_, c)| c.is_alphabetic() || *c == 'c') {
            self.position = index;
        }
        let end_position = self.position;
        
        Ok((TokenKind::Identifier, start_position, end_position))
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
                            let (kind, start, end)  = self.match_identify()?;
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
                            let (kind, start, end) = (TokenKind::SingleQuotation, *index, *index);
                            self.position = *index;
                            self.inner.next();
                            table.push(kind, start, end);
                        }
                        '"' => {
                            let (kind, start, end) = (TokenKind::DoubleQuotation, *index, *index);
                            self.position = *index;
                            self.inner.next();
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
                        '\n' => {
                            let (kind, start, end) = (TokenKind::LineBreak, *index, *index);
                            self.position = *index;
                            self.inner.next();
                            table.push(kind, start, end);
                        }
                        '>' => {
                            let index_now = *index;
                            self.inner.next();
                            if let Some((index_next, _)) = self.inner.next_if(|(_, c)| *c == '=') {
                                let (kind, start, end) = (TokenKind::GreaterEqual, index_now, index_next);
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
                                    let (kind, start, end) = (TokenKind::NotEqual, index_now, *index_next);
                                    self.position = index_next + 1;
                                    self.inner.next();
                                    table.push(kind, start, end);
                                }
                                Some((index_next, '=')) => {
                                    let (kind, start, end) = (TokenKind::LessEqual, index_now, *index_next);
                                    self.position = index_next + 1;
                                    self.inner.next();
                                    table.push(kind, start, end);
                                }
                                _ => {
                                let (kind, start, end) = (TokenKind::Less, index_now, index_now);
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
    use crate::token::TokenKind;

    #[test]
    fn test_skip_whitespace() {
        let mut lexer = Lexer::new("   \t\n   hello");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![Token{
                kind: TokenKind::LineBreak,
                start_position: 4,
                end_position: 4, 
            }, Token {
                kind: TokenKind::StringLiteral,
                start_position: 8,
                end_position: 12,
            }]
        );
    }

    #[test]
    fn test_match_number() {
        let mut lexer = Lexer::new("12345");
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            vec![Token {
                kind: TokenKind::Number,
                start_position: 0,
                end_position: 4,
            }]
        );
    }

    #[test]
    fn test_match_string() {
        let mut lexer = Lexer::new("helloWorld");
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            vec![Token {
                kind: TokenKind::StringLiteral,
                start_position: 0,
                end_position: 9,
            }]
        );
    }

    #[test]
    fn test_tokenize_numbers() {
        let mut lexer = Lexer::new("123 456");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenKind::Number,
                    start_position: 0,
                    end_position: 2,
                },
                Token {
                    kind: TokenKind::Number,
                    start_position: 4,
                    end_position: 6,
                },
            ]
        );
    }

    #[test]
    fn test_tokenize_strings() {
        let mut lexer = Lexer::new("hello world");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenKind::StringLiteral,
                    start_position: 0,
                    end_position: 4,
                },
                Token {
                    kind: TokenKind::StringLiteral,
                    start_position: 6,
                    end_position: 10,
                }
            ]
        )
    }

    #[test]
    fn test_tokenize_punctuation() {
        let mut lexer = Lexer::new("(),'\"\\;");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 7);
        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenKind::LeftParen,
                    start_position: 0,
                    end_position: 0,
                },
                Token {
                    kind: TokenKind::RightParen,
                    start_position: 1,
                    end_position: 1,
                },
                Token {
                    kind: TokenKind::Comma,
                    start_position: 2,
                    end_position: 2,
                },
                Token {
                    kind: TokenKind::SingleQuotation,
                    start_position: 3,
                    end_position: 3,
                },
                Token {
                    kind: TokenKind::DoubleQuotation,
                    start_position: 4,
                    end_position: 4,
                },
                Token {
                    kind: TokenKind::BackSlash,
                    start_position: 5,
                    end_position: 5,
                },
                Token {
                    kind: TokenKind::Eof,
                    start_position: 6,
                    end_position: 6,
                }
            ]
        );
    }

    #[test]
    fn test_tokenize_mixed() {
        let mut lexer = Lexer::new("func(123, 'abc');");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 9);
        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenKind::StringLiteral,
                    start_position: 0,
                    end_position: 3,
                },
                Token {
                    kind: TokenKind::LeftParen,
                    start_position: 4,
                    end_position: 4,
                },
                Token {
                    kind: TokenKind::Number,
                    start_position: 5,
                    end_position: 7,
                },
                Token {
                    kind: TokenKind::Comma,
                    start_position: 8,
                    end_position: 8,
                },
                Token {
                    kind: TokenKind::SingleQuotation,
                    start_position: 10,
                    end_position: 10,
                },
                Token {
                    kind: TokenKind::StringLiteral,
                    start_position: 11,
                    end_position: 13,
                },
                Token {
                    kind: TokenKind::SingleQuotation,
                    start_position: 14,
                    end_position: 14,
                },
                Token {
                    kind: TokenKind::RightParen,
                    start_position: 15,
                    end_position: 15,
                },
                Token {
                    kind: TokenKind::Eof,
                    start_position: 16,
                    end_position: 16,
                }
            ]
        )
    }

    #[test]
    fn test_tokenize_cmp() {
        let mut lexer = Lexer::new("a > b >= c < d <= e <> f = g");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 13);
        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenKind::StringLiteral,
                    start_position: 0,
                    end_position: 0,
                },
                Token {
                    kind: TokenKind::Greater,
                    start_position: 2,
                    end_position: 2,
                },
                Token {
                    kind: TokenKind::StringLiteral,
                    start_position: 4,
                    end_position: 4,
                },
                Token {
                    kind: TokenKind::GreaterEqual,
                    start_position: 6,
                    end_position: 7,
                },
                Token {
                    kind: TokenKind::StringLiteral,
                    start_position: 9,
                    end_position: 9,
                },
                Token {
                    kind: TokenKind::Less,
                    start_position: 11,
                    end_position: 11,
                },
                Token {
                    kind: TokenKind::StringLiteral,
                    start_position: 13,
                    end_position: 13,
                },
                Token {
                    kind: TokenKind::LessEqual,
                    start_position: 15,
                    end_position: 16,
                },
                Token {
                    kind: TokenKind::StringLiteral,
                    start_position: 18,
                    end_position: 18,
                },
                Token {
                    kind: TokenKind::NotEqual,
                    start_position: 20,
                    end_position: 21,
                },
                Token {
                    kind: TokenKind::StringLiteral,
                    start_position: 23,
                    end_position: 23,
                },
                Token {
                    kind: TokenKind::Equal,
                    start_position: 25,
                    end_position: 25,
                },
                Token {
                    kind: TokenKind::StringLiteral,
                    start_position: 27,
                    end_position: 27,
                },
            ]
        )
    }

    #[test]
    fn test_tokenize_unknown() {
        let mut lexer = Lexer::new("@#$");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenKind::Unknown,
                    start_position: 0,
                    end_position: 0,
                },
                Token {
                    kind: TokenKind::Unknown,
                    start_position: 1,
                    end_position: 1,
                },
                Token {
                    kind: TokenKind::Unknown,
                    start_position: 2,
                    end_position: 2,
                },
            ]
        )
    }

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_whitespace_handling() {
        let mut lexer = Lexer::new("  hello   world  ");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenKind::StringLiteral,
                    start_position: 2,
                    end_position: 6,
                },
                Token {
                    kind: TokenKind::StringLiteral,
                    start_position: 10,
                    end_position: 14,
                }
            ]
        );
    }

    #[test]
    fn test_chinese() {
        let mut lexer = Lexer::new("你好世界");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::StringLiteral,
                start_position: 0,
                end_position: 9,
            }]
        );
    }

    #[test]
    fn test_tokenize_line_break() {
        let mut lexer = Lexer::new("\n");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::LineBreak,
                start_position: 0,
                end_position: 0,
            }]
        );
    }
}
