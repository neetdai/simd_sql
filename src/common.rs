use std::{alloc::Allocator, borrow::Cow};

use crate::{ParserError, keyword::Keyword, token::{TokenKind, TokenTable}};


#[derive(Debug)]
pub enum Expr {
    Column(Alias<Column>),
    Field(Field),
}

impl Expr {
    pub(crate) fn class_column(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        Alias::<Column>::from_token(token_table, cursor).map(Expr::Column)
    }

    pub(crate) fn class_field(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        Field::from_token(token_table, cursor).map(Expr::Field)
    }
}

trait FromToken {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> where Self: Sized;
}

#[derive(Debug)]
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
                    Ok(Alias { name: None, value })
                }
            }
            Some(TokenKind::Identifier) => {
                *cursor += 1;
                if let Some((TokenKind::Identifier, (start, end))) = token_table.get_entry(*cursor) {
                    *cursor += 1;
                    Ok(Alias { name: Some((*start, *end)), value })
                } else {
                    Ok(Alias { name: None, value })
                }
            },
            _ => {
                Err(ParserError::SyntaxError(*cursor, *cursor))
            }
        }
    }
}

#[derive(Debug)]
pub struct Field {
    prefix: Option<(usize, usize)>,
    value: (usize, usize),
}

impl FromToken for Field {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> where Self: Sized {
        match token_table.get_entry(*cursor) {
            Some((TokenKind::Identifier, (start, end))) => {
                *cursor += 1;
                if let Some((TokenKind::Identifier, (start, end))) = token_table.get_entry(*cursor) {
                    *cursor += 1;
                    Ok(Field { prefix: Some((*start, *end)), value: (*start, *end) })
                } else {
                    Ok(Field { prefix: None, value: (*start, *end) })
                }
            }
            _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        }
    }
}

#[derive(Debug)]
pub struct Column {
    prefix: Option<(usize, usize)>,
    value: (usize, usize),
}

impl FromToken for Column {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> where Self: Sized {
        match token_table.get_entry(*cursor) {
            Some((TokenKind::Identifier, (start, end))) => {
                *cursor += 1;
                if let Some((TokenKind::Identifier, (start, end))) = token_table.get_entry(*cursor) {
                    *cursor += 1;
                    Ok(Self { prefix: Some((*start, *end)), value: (*start, *end) })
                } else {
                    Ok(Self { prefix: None, value: (*start, *end) })
                }
            }
            _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        }
    }
}