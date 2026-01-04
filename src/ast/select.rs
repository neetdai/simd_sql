use std::string::ParseError;

use crate::{expr::Expr, token::TokenTable};


#[derive(Debug)]
pub struct SelectStatement {
    columns: Vec<Expr>,
    // table: Expr,
}

impl SelectStatement {
    pub fn new(tokenTable: &TokenTable) -> Result<Self, ParseError> {
        let mut cursor = 0usize;

        Ok(Self {
            columns: Vec::new(),
        })
    }

    fn parse_columns(tokenTable: &TokenTable, cursor: &mut usize) -> Result<(), ParseError> {
        
        Ok(())
    }
}