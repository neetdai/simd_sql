use std::{alloc::Allocator, borrow::Cow};

use crate::{ParserError, keyword::Keyword, token::{TokenKind, TokenTable}};


#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod test {
    use crate::{Expr, common::{Alias, Column}, keyword::Keyword, token::{TokenKind, TokenTable}};


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
}