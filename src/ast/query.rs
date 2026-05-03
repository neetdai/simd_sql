use minivec::MiniVec;

use crate::error::ParserError;
use crate::{
    SelectStatement,
    ast::cte::CteBinding,
    common::{
        limit::Limit,
        order::Order,
        pratt_parser::{Flow, PrattOutput, PrattParser, PrattParserTrait, PrecedenceTrait},
        utils::{expect_kind, maybe_kind},
    },
    keyword::Keyword,
    token::{TokenKind, TokenTable},
};

#[derive(Debug, PartialEq)]
pub enum SetOperator {
    Union,
    UnionAll,
    Intersect,
    Except,
}

impl PrecedenceTrait for SetOperator {
    fn precedence(&self) -> usize {
        match self {
            SetOperator::Union | SetOperator::UnionAll => 1,
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
pub enum Query<'a> {
    Select(SelectStatement<'a>),
    Cte {
        ctes: MiniVec<CteBinding<'a>>,
        query: Box<Query<'a>>,
    },
    SetOperation {
        op: SetOperator,
        left: Box<Query<'a>>,
        right: Box<Query<'a>>,
        order_by: Option<Order<'a>>,
        limit: Option<Limit<'a>>,
    },
}

impl<'a> Query<'a> {
    pub(crate) fn build(
        token_table: &TokenTable<'a>,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        let mut query = PrattParser::parse_expression::<Self>(token_table, cursor)?;

        if let Self::SetOperation {
            op: _,
            left: _,
            right: _,
            order_by,
            limit,
        } = &mut query
        {
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

impl<'a> PrattParserTrait<'a> for Query<'a> {
    type Item = SetOperator;
    type Output = Self;

    fn match_item(token_table: &TokenTable, cursor: &mut usize) -> Option<Self::Item> {
        match token_table.get_kind(*cursor) {
            Some(TokenKind::Keyword(Keyword::Union)) => {
                *cursor += 1;
                if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::All)) {
                    *cursor += 1;
                    Some(SetOperator::UnionAll)
                } else {
                    Some(SetOperator::Union)
                }
            }
            Some(TokenKind::Keyword(Keyword::Intersect)) => {
                *cursor += 1;
                Some(SetOperator::Intersect)
            }
            Some(TokenKind::Keyword(Keyword::Except)) => {
                *cursor += 1;
                Some(SetOperator::Except)
            }
            _ => None,
        }
    }

    fn parse_primary(
        token_table: &TokenTable<'a>,
        cursor: &mut usize,
    ) -> Result<Self::Output, ParserError> {
        match token_table.get_kind(*cursor) {
            Some(TokenKind::Keyword(Keyword::Select)) => {
                SelectStatement::new(token_table, cursor).map(Query::Select)
            }
            Some(TokenKind::LeftParen) => {
                *cursor += 1;
                let query = Self::build(token_table, cursor)?;
                expect_kind(token_table, cursor, &TokenKind::RightParen)?;
                *cursor += 1;
                Ok(query)
            }
            _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        }
    }

    fn parse_postfix(
        left: Self::Output,
        _token_table: &TokenTable<'a>,
        _cursor: &mut usize,
    ) -> Result<(Self::Output, Flow), ParserError> {
        Ok((left, Flow::Run))
    }
}

impl<'a> PrattOutput<SetOperator> for Query<'a> {
    fn apply(op: SetOperator, left: Self, right: Self) -> Self {
        Self::SetOperation {
            op,
            left: Box::new(left),
            right: Box::new(right),
            order_by: None,
            limit: None,
        }
    }
}
