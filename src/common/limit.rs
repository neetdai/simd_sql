use crate::{
    ParserError,
    common::expr::Expr,
    token::{TokenKind, TokenTable},
};

#[derive(Debug, PartialEq)]
pub struct Limit {
    page: Option<Expr>,
    limit: Expr,
}

impl Limit {
    pub fn new(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        let first = token_table
            .get_kind(*cursor)
            .map(|kind| kind == &TokenKind::Identifier)
            .unwrap_or(false);
        let comma = token_table
            .get_kind(*cursor + 1)
            .map(|kind| kind == &TokenKind::Comma)
            .unwrap_or(false);
        let second = token_table
            .get_kind(*cursor + 2)
            .map(|kind| kind == &TokenKind::Identifier)
            .unwrap_or(false);

        match (first, comma, second) {
            (true, true, true) => {
                let page = Expr::build(token_table, cursor)?;
                *cursor += 1; // skip comma
                let limit = Expr::build(token_table, cursor)?;
                Ok(Limit {
                    page: Some(page),
                    limit,
                })
            }
            (true, false, _) => {
                let limit = Expr::build(token_table, cursor)?;
                Ok(Limit { page: None, limit })
            }
            _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        }
    }
}
