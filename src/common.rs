use std::{alloc::Allocator, borrow::Cow};

use minivec::MiniVec;

use crate::{ParserError, keyword::Keyword, token::{self, TokenKind, TokenTable}};

pub(crate) fn expect_kind(token_table: &TokenTable, cursor: &usize, token_kind: &TokenKind) -> Result<(), ParserError> {
    if let Some(kind) = token_table.get_kind(*cursor) {
        if kind != token_kind {
            return Err(ParserError::UnexpectedToken { expected: token_kind.clone(), found: kind.clone() });
        }
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Column(Alias<Column>),
    Field(Field),
    FunctionCall(Alias<FunctionCall>),
    StringLiteral(StringLiteral),
    NumbericLiteral(NumbericLiteral),
    FourBasic(),
}

impl Expr {
    pub(crate) fn class_column(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        Alias::<Column>::from_token(token_table, cursor).map(Expr::Column)
    }

    pub(crate) fn class_field(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        Field::from_token(token_table, cursor).map(Expr::Field)
    }

    pub(crate) fn class_function_call(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        Alias::<FunctionCall>::from_token(token_table, cursor).map(Expr::FunctionCall)
    }

    pub(crate) fn class_string_literal(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        StringLiteral::from_token(token_table, cursor).map(Expr::StringLiteral)
    }

    pub(crate) fn class_number_literal(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        NumbericLiteral::from_token(token_table, cursor).map(Expr::NumbericLiteral)
    }

    pub(crate) fn build(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        match token_table.get_kind(*cursor) {
            Some(TokenKind::Number) => {
                let number = Self::class_number_literal(token_table, cursor)?;

                match token_table.get_kind(*cursor) {
                    Some(TokenKind::Plus | TokenKind::Subtract | TokenKind::Divide | TokenKind::Multiply | TokenKind::Mod) => {

                    }
                    _ => {
                        todo!()
                    }
                }

                Ok(number)
            }
            Some(TokenKind::StringLiteral) => {
                Self::class_string_literal(token_table, cursor)
            }
            Some(TokenKind::Identifier) => {
                match token_table.get_kind(*cursor) {
                    Some(TokenKind::LeftParen) => {
                        Self::class_function_call(token_table, cursor)
                    }
                    _ => {
                        Self::class_field(token_table, cursor)
                    }
                }
            }
            _ => {
                Err(ParserError::SyntaxError(*cursor, *cursor))
            }
        }
    }
}

trait FromToken {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> where Self: Sized;
}

#[derive(Debug, PartialEq)]
pub struct Alias<T> {
    name: Option<usize>,
    value: T,
}

impl<T> FromToken for Alias<T> where T: FromToken {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> where Self: Sized {
        let value = T::from_token(token_table, cursor)?;
        match token_table.get_kind(*cursor) {
            Some(TokenKind::Keyword(Keyword::As)) => {
                *cursor += 1;
                if let Some(TokenKind::Identifier) = token_table.get_kind(*cursor) {
                    let name = *cursor;
                    *cursor += 1;
                    Ok(Alias { name: Some(name), value })
                } else {
                    Err(ParserError::SyntaxError(*cursor, *cursor))
                }
            }
            Some(TokenKind::Identifier) => {
                if let Some(TokenKind::Identifier) = token_table.get_kind(*cursor) {
                    let name = *cursor;
                    *cursor += 1;
                    Ok(Alias { name: Some(name), value })
                } else {
                    Ok(Alias { name: None, value })
                }
            },
            _ => {
                Ok(Alias { name: None, value })
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Field {
    prefix: Option<usize>,
    value: usize,
}

impl FromToken for Field {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> where Self: Sized {
        let first = token_table.get_kind(*cursor).map(|kind| kind == &TokenKind::Identifier).unwrap_or(false);
        let dot = token_table.get_kind(*cursor + 1).map(|kind| kind == &TokenKind::Dot).unwrap_or(false);
        let second = token_table.get_kind(*cursor + 2).map(|kind| kind == &TokenKind::Identifier).unwrap_or(false);

        let sum = (first as usize) + (dot as usize) + (second as usize);

        let (prefix, value) = match (first, sum) {
            (true, 1) => (None, *cursor),
            (true, 3) => (Some(*cursor), *cursor + 2),
            _ => return Err(ParserError::SyntaxError(*cursor, *cursor)),
        };

        *cursor += sum;

        Ok(Self {
            prefix,
            value,
        })


        // match token_table.get_entry(*cursor) {
        //     Some((TokenKind::Identifier, (current_start, current_end))) => {
        //         *cursor += 1;
        //         if let Some([TokenKind::Dot, TokenKind::Identifier]) = token_table.get_kind(*cursor..=*cursor + 1) {
        //             *cursor += 1;
        //             let (start, end) = token_table.get_position(*cursor).unwrap();
        //             *cursor += 1;
        //             Ok(Self { prefix: Some((*current_start, *current_end)), value: (*start, *end) })
        //         } else {
        //             Ok(Self { prefix: None, value: (*current_start, *current_end) })
        //         }
        //     }
        //     _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        // }
    }
}

#[derive(Debug, PartialEq)]
pub struct Column {
    prefix: Option<usize>,
    value: usize,
}

impl FromToken for Column {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> where Self: Sized {
        let first = token_table.get_kind(*cursor).map(|kind| kind == &TokenKind::Identifier).unwrap_or(false);
        let dot = token_table.get_kind(*cursor + 1).map(|kind| kind == &TokenKind::Dot).unwrap_or(false);
        let second = token_table.get_kind(*cursor + 2).map(|kind| kind == &TokenKind::Identifier).unwrap_or(false);

        let sum = (first as usize) + (dot as usize) + (second as usize);

        let (prefix, value) = match (first, sum) {
            (true, 1) => (None, *cursor),
            (true, 3) => (Some(*cursor), *cursor + 2),
            _ => return Err(ParserError::SyntaxError(*cursor, *cursor)),
        };

        *cursor += sum;

        Ok(Self {
            prefix,
            value,
        })
        
        // match token_table.get_entry(*cursor) {
        //     Some((TokenKind::Identifier, (current_start, current_end))) => {
        //         *cursor += 1;
        //         if let Some([TokenKind::Dot, TokenKind::Identifier]) = token_table.get_kind(*cursor..=*cursor + 1) {
        //             *cursor += 1;
        //             let (start, end) = token_table.get_position(*cursor).unwrap();
        //             *cursor += 1;
        //             Ok(Self { prefix: Some((*current_start, *current_end)), value: (*start, *end) })
        //         } else {
        //             Ok(Self { prefix: None, value: (*current_start, *current_end) })
        //         }
        //     }
        //     _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        // }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionCall {
    name: usize,
    args: MiniVec<Expr>,
}

impl FromToken for FunctionCall {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> where Self: Sized {
        let first = token_table.get_kind(*cursor).map(|kind| kind == &TokenKind::Identifier).unwrap_or(false);
        let second = token_table.get_kind(*cursor + 1).map(|kind| kind == &TokenKind::LeftParen).unwrap_or(false);

        if !(first && second) {
            return Err(ParserError::SyntaxError(*cursor, *cursor));
        }

        let name_pos = *cursor;
        *cursor += 2;

        let mut args = MiniVec::with_capacity(8);
        loop {
            let expr = Expr::build(token_table, cursor)?;
            args.push(expr);

            match token_table.get_kind(*cursor) {
                Some(TokenKind::Comma) => {
                    *cursor += 1;
                    continue;
                }
                Some(TokenKind::RightParen) => {
                    *cursor += 1;
                    break;
                }
                _ => {
                    return Err(ParserError::SyntaxError(*cursor, *cursor));
                }
            }
        }

        Ok(Self {
            name: name_pos,
            args,
        })

        // if let Some((TokenKind::Identifier, (start, end))) = token_table.get_entry(*cursor) {
        //     *cursor += 1;
        //     let name = (*start, *end);

        //     expect_kind(token_table, cursor, &TokenKind::LeftParen)?;
        //     *cursor += 1;

        //     let mut args = MiniVec::with_capacity(8);
        //     loop {
        //         let expr = Expr::build(token_table, cursor)?;
        //         args.push(expr);

        //         if let Some(TokenKind::Comma) = token_table.get_kind(*cursor) {
        //             *cursor += 1;
        //             continue;
        //         } else if let Some(TokenKind::RightParen) = token_table.get_kind(*cursor) {
        //             *cursor += 1;
        //             break;
        //         } else {
        //             return Err(ParserError::SyntaxError(*cursor, *cursor));
        //         }
        //     }

        //     Ok(Self {
        //         name: name,
        //         args,
        //     })
        // } else {
        //     return Err(ParserError::SyntaxError(*cursor, *cursor));
        // }
    }
}

#[derive(Debug, PartialEq)]
pub struct StringLiteral {
    value: usize,
}

impl FromToken for StringLiteral {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> where Self: Sized {
        if let Some(TokenKind::StringLiteral) = token_table.get_kind(*cursor) {
            let value = *cursor;
            *cursor += 1;
            Ok(Self { value  })
        } else {
            Err(ParserError::SyntaxError(*cursor, *cursor))
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct NumbericLiteral {
    value: usize,
}

impl FromToken for NumbericLiteral {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> where Self: Sized {
        if let Some(TokenKind::Number) = token_table.get_kind(*cursor) {
            let value = *cursor;
            *cursor += 1;
            Ok(Self { value  })
        } else {
            Err(ParserError::SyntaxError(*cursor, *cursor))
        }
    }
}

// 四则运算
#[derive(Debug, PartialEq)]
pub enum FourBasic {
    Add {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Subtract {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Multiply {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Divide {
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

impl FourBasic {
    
}


#[cfg(test)]
mod test {
    use minivec::mini_vec;

    use crate::{Expr, ParserError, common::{Alias, Column, FunctionCall, StringLiteral}, keyword::Keyword, token::{TokenKind, TokenTable}};


    #[test]
    fn test_column() {
        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1); // prefix
        token_table.push(TokenKind::Dot, 2, 2);
        token_table.push(TokenKind::Identifier, 3, 4); // value
        
        let mut cursor = 0;
        let expr = Expr::class_column(&token_table, &mut cursor).unwrap();
        assert_eq!(expr, Expr::Column(Alias {
            name: None,
            value:  Column {
                prefix: Some(0),
                value: 2,
            }
        }));
        assert_eq!(cursor, 3);

        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1); // prefix
        token_table.push(TokenKind::Dot, 2, 2);
        token_table.push(TokenKind::Identifier, 3, 4); // value
        token_table.push(TokenKind::Keyword(Keyword::As), 5, 6); // As
        token_table.push(TokenKind::Identifier, 7, 8); // alias
        
        let mut cursor = 0;
        let expr = Expr::class_column(&token_table, &mut cursor).unwrap();
        assert_eq!(expr, Expr::Column(Alias {
            name: Some(4),
            value:  Column {
                prefix: Some(0),
                value: 2,
            }
        }));
        assert_eq!(cursor, 5);

        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1); // prefix
        token_table.push(TokenKind::Dot, 2, 2);
        token_table.push(TokenKind::Identifier, 3, 4); // value
        token_table.push(TokenKind::Identifier, 5, 6); // alias
        
        let mut cursor = 0;
        let expr = Expr::class_column(&token_table, &mut cursor).unwrap();
        assert_eq!(expr, Expr::Column(Alias {
            name: Some(3),
            value:  Column {
                prefix: Some(0),
                value: 2,
            }
        }));
        assert_eq!(cursor, 4);
    }

    #[test]
    fn test_function() {
        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1); // name
        token_table.push(TokenKind::LeftParen, 2, 2);
        token_table.push(TokenKind::StringLiteral, 3,5);
        token_table.push(TokenKind::RightParen, 6, 6); // args

        let mut cursor = 0;
        let expr = Expr::class_function_call(&token_table, &mut cursor).unwrap();
        assert_eq!(expr, Expr::FunctionCall(Alias { name: None, value: FunctionCall {
            name: 0,
            args: mini_vec![
                Expr::StringLiteral(StringLiteral {
                    value: 2
                })
            ]
        } }));
        assert_eq!(cursor, 4);
    }

    #[test]
    fn test_function_1() {
        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1); // name
        token_table.push(TokenKind::LeftParen, 2, 2);
        token_table.push(TokenKind::StringLiteral, 3,5);
        token_table.push(TokenKind::Comma, 6, 6);
        token_table.push(TokenKind::StringLiteral, 7,8);
        token_table.push(TokenKind::RightParen, 9, 9); // args

        let mut cursor = 0;
        let expr = Expr::class_function_call(&token_table, &mut cursor).unwrap();
        assert_eq!(expr, Expr::FunctionCall(Alias { name: None, value: FunctionCall {
            name: 0,
            args: mini_vec![
                Expr::StringLiteral(StringLiteral {
                    value: 2
                }),
                Expr::StringLiteral(StringLiteral {
                    value: 4
                })
            ]
        } }));
        assert_eq!(cursor, 6);
    }

    #[test]
    fn test_function_should_panic_1()
    {
        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1); // name
        token_table.push(TokenKind::LeftParen, 2, 2);
        token_table.push(TokenKind::StringLiteral, 3,5);
        token_table.push(TokenKind::Comma, 6, 6);
        token_table.push(TokenKind::RightParen, 7, 7); // args

        let mut cursor = 0;
        let expr = Expr::class_function_call(&token_table, &mut cursor);
        assert_eq!(expr, Err(ParserError::SyntaxError(4, 4)));
    }
}