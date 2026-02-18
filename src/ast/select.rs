use std::alloc::Allocator;

use minivec::MiniVec;

use crate::{
    ParserError,
    common::{
        alias::Alias,
        expr::Expr,
        from::From,
        group::Group,
        limit::Limit,
        order::Order,
        utils::{expect_kind, maybe_kind},
    },
    keyword::Keyword,
    token::{TokenKind, TokenTable},
};

#[derive(Debug, PartialEq)]
pub struct SelectStatement {
    columns: Vec<Alias<Expr>>,
    from: Option<MiniVec<From>>,
    where_statement: Option<Expr>,
    group_by: Option<Group>,
    order_by: Option<Order>,
    limit: Option<Limit>,
}

impl SelectStatement {
    pub fn new(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        // Ok(Self {
        //     columns: Vec::new(),
        // })
        Self::build_ast(token_table, cursor)
    }

    fn build_ast(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Select))?;
        *cursor += 1;

        let mut columns = Vec::new();
        loop {
            match token_table.get_kind(*cursor) {
                Some(TokenKind::Comma) => {
                    *cursor += 1;
                }
                Some(TokenKind::Keyword(_)) => break,
                Some(_) => {
                    let expr = Alias::new(token_table, cursor)?;
                    columns.push(expr);
                }
                None => break,
            }
        }

        let from = if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::From)) {
            *cursor += 1;

            let mut list = MiniVec::new();
            loop {
                match token_table.get_kind(*cursor) {
                    Some(TokenKind::Comma) => {
                        *cursor += 1;
                    }
                    Some(TokenKind::Keyword(_)) => break,
                    Some(_) => {
                        list.push(From::class_table(token_table, cursor)?);
                    }
                    None => break,
                }
            }
            Some(list)
        } else {
            None
        };

        let where_statement =
            if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Where)) {
                *cursor += 1;
                Some(Expr::build(token_table, cursor)?)
            } else {
                None
            };

        let group_by = if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Group)) {
            Some(Group::build(token_table, cursor)?)
        } else {
            None
        };

        let order_by = if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Order)) {
            Some(Order::build(token_table, cursor)?)
        } else {
            None
        };

        let limit = if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Limit)) {
            *cursor += 1;
            Some(Limit::new(token_table, cursor)?)
        } else {
            None
        };

        Ok(Self {
            columns,
            from,
            where_statement,
            group_by,
            order_by,
            limit,
        })
    }
}
