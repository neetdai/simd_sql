use crate::{
    SelectStatement, common::{expr::Expr, group::Group, limit::Limit, order::Order, pratt_parser::{Flow, PrattOutput, PrattParser, PrattParserTrait, PrecedenceTrait}, utils::maybe_kind}, keyword::Keyword, token::{self, TokenKind, TokenTable}
};
use crate::error::ParserError;

#[derive(Debug, PartialEq)]
pub enum SetOperator {
    Union,
    Intersect,
    Except,
}

impl PrecedenceTrait for SetOperator {
    fn precedence(&self) -> usize {
        match self {
            SetOperator::Union => 1,
            SetOperator::Intersect => 2,
            SetOperator::Except => 3,
        }
    }

    fn is_left_associative(&self) -> bool {
        true
    }

    fn min_precedence() -> usize {
        1
    }
}

#[derive(Debug, PartialEq)]
pub enum Query {
    Select(SelectStatement),

    SetOperation {
        op: SetOperator,
        left: Box<Query>,
        right: Box<Query>,
        order_by: Option<Order>,
        limit: Option<Limit>,
    }
}

impl Query {
    pub(crate) fn build(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        let mut query = PrattParser::parse_expression::<Self>(token_table, cursor)?;

        if let Self::SetOperation { op, left, right, order_by, limit } = &mut query {
            *order_by = if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Order)) {
                Some(Order::build(token_table, cursor)?)
            } else {
                None
            };

            *limit = if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Limit)) {
                Some(Limit::new(token_table, cursor)?)
            } else {
                None
            };
        }

        Ok(query)
    }
}

impl PrattParserTrait for Query {
    type Item = SetOperator;
    type Output = Self;

    fn match_item(token_kind: &TokenKind) -> Option<Self::Item> {
        match token_kind {
            TokenKind::Keyword(Keyword::Union) => Some(SetOperator::Union),
            TokenKind::Keyword(Keyword::Intersect) => Some(SetOperator::Intersect),
            TokenKind::Keyword(Keyword::Except) => Some(SetOperator::Except),
            _ => None,
        }
    }

    fn parse_primary(
            token_table: &TokenTable,
            cursor: &mut usize,
        ) -> Result<Self::Output, ParserError> {
        match token_table.get_kind(*cursor) {
            Some(TokenKind::Keyword(Keyword::Select)) => {
                SelectStatement::new(token_table, cursor).map(Query::Select)
            },
            _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        }
    }

    fn parse_postfix(
        left: Self::Output,
        token_table: &TokenTable,
        cursor: &mut usize,
    ) -> Result<(Self::Output, Flow), ParserError> {
        Ok((left, Flow::Run))
    }
}

impl PrattOutput<SetOperator> for Query {
    fn apply(op: SetOperator, left: Self, right: Self) -> Self {
        Self::SetOperation { op, left: Box::new(left), right: Box::new(right), order_by: None, limit: None }
    }
}
