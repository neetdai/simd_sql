use std::{alloc::Allocator};

use crate::{Expr, ParserError, common::{Alias, expect_kind, maybe_kind}, keyword::Keyword, token::{TokenKind, TokenTable}};


#[derive(Debug)]
pub struct SelectStatement {
    columns: Vec<Alias>,
    // table: Expr,
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
                Some(TokenKind::Comma) => continue,
                Some(TokenKind::Keyword(_)) => break,
                Some(_) => {
                    let expr = Alias::new(token_table, cursor)?;
                    columns.push(expr);
                }
                None => break,
            }
        }

        if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::From)) {
            *cursor += 1;
            
        }

        Ok(Self {
            columns,
        })
    }
}