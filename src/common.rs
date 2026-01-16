use std::{alloc::Allocator, borrow::Cow};

use minivec::MiniVec;

use crate::{ParserError, keyword::Keyword, token::{TokenKind, TokenTable}};

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

    pub(crate) fn build(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        match token_table.get_kind(*cursor) {
            Some(TokenKind::Number) => {
                todo!()
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
    name: Option<(usize, usize)>,
    value: T,
}

impl<T> FromToken for Alias<T> where T: FromToken {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> where Self: Sized {
        let value = T::from_token(token_table, cursor)?;
        match token_table.get_kind(*cursor) {
            Some(TokenKind::Keyword(Keyword::As)) => {
                *cursor += 1;
                if let Some((TokenKind::Identifier, (start, end))) = token_table.get_entry(*cursor) {
                    *cursor += 1;
                    Ok(Alias { name: Some((*start, *end)), value })
                } else {
                    Err(ParserError::SyntaxError(*cursor, *cursor))
                }
            }
            Some(TokenKind::Identifier) => {
                if let Some((TokenKind::Identifier, (start, end))) = token_table.get_entry(*cursor) {
                    *cursor += 1;
                    Ok(Alias { name: Some((*start, *end)), value })
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
    prefix: Option<(usize, usize)>,
    value: (usize, usize),
}

impl FromToken for Field {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> where Self: Sized {
        match token_table.get_entry(*cursor) {
            Some((TokenKind::Identifier, (current_start, current_end))) => {
                *cursor += 1;
                if let Some([TokenKind::Dot, TokenKind::Identifier]) = token_table.get_kind(*cursor..=*cursor + 1) {
                    *cursor += 1;
                    let (start, end) = token_table.get_position(*cursor).unwrap();
                    *cursor += 1;
                    Ok(Self { prefix: Some((*current_start, *current_end)), value: (*start, *end) })
                } else {
                    Ok(Self { prefix: None, value: (*current_start, *current_end) })
                }
            }
            _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Column {
    prefix: Option<(usize, usize)>,
    value: (usize, usize),
}

impl FromToken for Column {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> where Self: Sized {
        match token_table.get_entry(*cursor) {
            Some((TokenKind::Identifier, (current_start, current_end))) => {
                *cursor += 1;
                if let Some([TokenKind::Dot, TokenKind::Identifier]) = token_table.get_kind(*cursor..=*cursor + 1) {
                    *cursor += 1;
                    let (start, end) = token_table.get_position(*cursor).unwrap();
                    *cursor += 1;
                    Ok(Self { prefix: Some((*current_start, *current_end)), value: (*start, *end) })
                } else {
                    Ok(Self { prefix: None, value: (*current_start, *current_end) })
                }
            }
            _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionCall {
    name: (usize, usize),
    args: MiniVec<Expr>,
}

impl FromToken for FunctionCall {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> where Self: Sized {
        if let Some((TokenKind::Identifier, (start, end))) = token_table.get_entry(*cursor) {
            *cursor += 1;
            let name = (*start, *end);

            expect_kind(token_table, cursor, &TokenKind::LeftParen)?;
            *cursor += 1;

            let mut args = MiniVec::with_capacity(8);
            loop {
                let expr = Expr::build(token_table, cursor)?;
                args.push(expr);

                if let Some(TokenKind::Comma) = token_table.get_kind(*cursor) {
                    *cursor += 1;
                    continue;
                } else if let Some(TokenKind::RightParen) = token_table.get_kind(*cursor) {
                    *cursor += 1;
                    break;
                } else {
                    return Err(ParserError::SyntaxError(*cursor, *cursor));
                }
            }

            Ok(Self {
                name: name,
                args,
            })
        } else {
            return Err(ParserError::SyntaxError(*cursor, *cursor));
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct StringLiteral {
    value: (usize, usize),
}

impl FromToken for StringLiteral {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> where Self: Sized {
        if let Some((TokenKind::StringLiteral, (start, end))) = token_table.get_entry(*cursor) {
            *cursor += 1;
            Ok(Self { value: (*start, *end) })
        } else {
            Err(ParserError::SyntaxError(*cursor, *cursor))
        }
    }
}

#[cfg(test)]
mod test {
    use minivec::mini_vec;

    use crate::{Expr, common::{Alias, Column, FunctionCall, StringLiteral}, keyword::Keyword, token::{TokenKind, TokenTable}};


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
                prefix: Some((0, 1)),
                value: (3, 4),
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
            name: Some((7, 8)),
            value:  Column {
                prefix: Some((0, 1)),
                value: (3, 4),
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
            name: Some((5, 6)),
            value:  Column {
                prefix: Some((0, 1)),
                value: (3, 4),
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
            name: (0, 1),
            args: mini_vec![
                Expr::StringLiteral(StringLiteral {
                    value: (3, 5)
                })
            ]
        } }))
    }
}