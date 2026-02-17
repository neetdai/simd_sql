use std::{alloc::Allocator};

use crate::{ParserError, common::{alias::Alias, expr::Expr, from::From, utils::{expect_kind, maybe_kind}}, keyword::Keyword, token::{TokenKind, TokenTable}};


#[derive(Debug, PartialEq)]
pub struct SelectStatement {
    columns: Vec<Alias<Expr>>,
    from: Option<From>,
    where_statement: Option<Expr>,
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
                },
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
            Some(From::parse(token_table, cursor)?)
        } else {
            None
        };

        let where_statement = if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Where)) {
            *cursor += 1;
            Some(Expr::build(token_table, cursor)?)
        } else {
            None
        };

        Ok(Self {
            columns,
            from,
            where_statement,
        })
    }
}