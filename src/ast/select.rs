use std::{alloc::Allocator};

use crate::{Expr, ParserError, common::{Alias, expect_kind}, keyword::Keyword, token::{TokenKind, TokenTable}};


#[derive(Debug)]
pub struct SelectStatement<A> where A:Allocator {
    columns: Vec<Alias, A>,
    // table: Expr,
}

impl<A> SelectStatement<A> where A: Allocator {
    pub fn new(token_table: &TokenTable, cursor: &mut usize, allocator: A) -> Result<Self, ParserError> {

        Ok(Self {
            columns: Vec::new_in(allocator),
        })
    }

    fn build_ast(token_table: &TokenTable, cursor: &mut usize) -> Result<(), ParserError> {
        expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Select))?;

        let mut columns = Vec::new();
        loop {
            match token_table.get_kind(*cursor) {
                Some(TokenKind::Comma) => continue,
                Some(TokenKind::Keyword(_)) => break,
                Some(_) => {
                    let expr = Alias::new(token_table, cursor)?;
                    columns.push(expr);
                }
                None => break,
            }
        }

        expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::From))?;

        Ok(())
    }
}