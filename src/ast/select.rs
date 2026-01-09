use std::{alloc::Allocator};

use crate::{Expr, keyword::Keyword, token::{TokenKind, TokenTable}, ParserError};


#[derive(Debug)]
pub struct SelectStatement<A> where A:Allocator {
    columns: Vec<Expr, A>,
    // table: Expr,
}

impl<A> SelectStatement<A> where A: Allocator {
    pub fn new(token_table: &TokenTable, cursor: &mut usize, allocator: A) -> Result<Self, ParserError> {

        Ok(Self {
            columns: Vec::new_in(allocator),
        })
    }
}