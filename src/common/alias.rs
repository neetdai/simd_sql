use crate::{
    ParserError,
    keyword::Keyword,
    token::{TokenKind, TokenTable},
};

pub(crate) trait Aliasable: Sized {
    fn aliasable(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError>;
}

#[derive(Debug, PartialEq)]
pub struct Alias<T> {
    pub name: Option<usize>,
    pub value: T,
}

impl<T> Alias<T>
where
    T: Aliasable,
{
    pub(crate) fn new(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        let value = T::aliasable(token_table, cursor)?;
        match token_table.get_kind(*cursor) {
            Some(TokenKind::Keyword(Keyword::As)) => {
                *cursor += 1;
                if let Some(TokenKind::Identifier) = token_table.get_kind(*cursor) {
                    let name = *cursor;
                    *cursor += 1;
                    Ok(Alias {
                        name: Some(name),
                        value,
                    })
                } else {
                    Err(ParserError::SyntaxError(*cursor, *cursor))
                }
            }
            Some(TokenKind::Identifier) => {
                if let Some(TokenKind::Identifier) = token_table.get_kind(*cursor) {
                    let name = *cursor;
                    *cursor += 1;
                    Ok(Alias {
                        name: Some(name),
                        value,
                    })
                } else {
                    Ok(Alias { name: None, value })
                }
            }
            _ => Ok(Alias { name: None, value }),
        }
    }
}
