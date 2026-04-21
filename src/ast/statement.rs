use minivec::MiniVec;

use super::{insert::InsertStatement, select::SelectStatement, update::UpdateStatement};
use crate::{
    ast::{delete::DeleteStatement, query::Query},
    common::utils::maybe_kind,
    error::ParserError,
    keyword::Keyword,
    token::{TokenKind, TokenTable},
};

#[derive(Debug, PartialEq)]
pub enum Statement {
    Query(Query),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
}

impl Statement {
    pub fn new(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        Self::match_statement(token_table, cursor)
    }

    fn match_statement(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        match token_table.get_kind(*cursor) {
            Some(TokenKind::Keyword(Keyword::Select)) => {
                Query::build(token_table, cursor).map(Self::Query)
            }
            Some(TokenKind::Keyword(Keyword::Insert)) => {
                Ok(Self::Insert(InsertStatement::new(token_table, cursor)?))
            }
            Some(TokenKind::Keyword(Keyword::Update)) => {
                Ok(Self::Update(UpdateStatement::new(token_table, cursor)?))
            }
            Some(TokenKind::Keyword(Keyword::Delete)) => {
                Ok(Self::Delete(DeleteStatement::new(token_table, cursor)?))
            }
            _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        }
    }
}
