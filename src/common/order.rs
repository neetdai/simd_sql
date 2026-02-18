use minivec::MiniVec;

use crate::{ParserError, common::{expr::Expr, utils::{expect_kind, maybe_kind}}, keyword::Keyword, token::{TokenKind, TokenTable}};

#[derive(Debug, PartialEq)]
pub enum OrderDirection {
    ASC,
    DESC,
}

#[derive(Debug, PartialEq)]
pub struct OrderItem {
    expr: Expr,
    direction: OrderDirection,
}

#[derive(Debug, PartialEq)]
pub struct Order {
    columns: MiniVec<OrderItem>,
}

impl Order {
    pub(crate) fn build(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Order))?;
        *cursor += 1;
        expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::By))?;
        *cursor += 1;

        let mut columns = MiniVec::new();
        loop {
            match token_table.get_kind(*cursor) {
                Some(TokenKind::Identifier) => {
                    let expr = Expr::build(token_table, cursor)?;
                    let direction = if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Asc)) {
                        *cursor += 1;
                        OrderDirection::ASC
                    } else if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Desc)) {
                        *cursor += 1;
                        OrderDirection::DESC
                    } else {
                        OrderDirection::ASC
                    };
                    columns.push(OrderItem { expr, direction });
                }
                Some(TokenKind::Comma) => {
                    *cursor += 1;
                }
                _ => {
                    break;
                }
            }
        }

        Ok(Order { columns })
    }
}
