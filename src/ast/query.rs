use crate::{
    SelectStatement, common::{expr::Expr, group::Group, limit::Limit, order::Order, pratt_parser::{PrattOutput, PrattParserTrait, PrecedenceTrait}, utils::maybe_kind}, keyword::Keyword, token::{self, TokenKind, TokenTable}
};
use crate::error::ParserError;

#[derive(Debug, PartialEq)]
pub enum SetOperator {
    Union,
    Intersect,
    Except,
}

#[derive(Debug, PartialEq)]
pub enum QueryInner {
    Select(SelectStatement),

    SetOperation {
        op: SetOperator,
        left: Box<Query>,
        right: Box<Query>,
    }
}


#[derive(Debug, PartialEq)]
pub struct Query {
    pub inner: QueryInner,
    pub group_by: Option<Group>,
    pub having: Option<Expr>,
    pub order_by: Option<Order>,
    pub limit: Option<Limit>,
}

impl Query {
    pub fn build(token_table: &mut TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        let inner = QueryInner::build(token_table, cursor)?;

        let group_by = if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Group)) {
            Some(Group::build(token_table, cursor)?)
        } else {
            None
        };

        let having =
        if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Having)) {
            *cursor += 1;
            Some(Expr::build(token_table, cursor)?)
        } else {
            None
        };

        let order_by = if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Order)) {
            Some(Order::build(token_table, cursor)?)
        } else {
            None
        };

        let limit = if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Limit)) {
            Some(Limit::new(token_table, cursor)?)
        } else {
            None
        };

        Ok(Self { inner, group_by, having, order_by, limit })
    }
}