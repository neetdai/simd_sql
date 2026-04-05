use minivec::MiniVec;

use crate::{
    ParserError,
    common::{
        expr::Expr,
        utils::{expect_kind, maybe_kind},
    },
    keyword::Keyword,
    token::{TokenKind, TokenTable},
};

#[derive(Debug, PartialEq)]
pub struct InsertStatement {
    pub table: Expr,
    pub columns: MiniVec<Expr>,
    pub values: MiniVec<Expr>,
}

impl InsertStatement {
    pub fn new(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        Self::build_ast(token_table, cursor)
    }

    fn build_ast(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Insert))?;
        *cursor += 1;

        expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Into))?;
        *cursor += 1;

        let table = Expr::build(token_table, cursor)?;

        expect_kind(token_table, cursor, &TokenKind::LeftParen)?;
        *cursor += 1;

        let mut columns = MiniVec::new();

        loop {
            match token_table.get_kind(*cursor) {
                Some(TokenKind::Comma) => {
                    *cursor += 1;
                }
                Some(TokenKind::RightParen) => {
                    *cursor += 1;
                    break;
                }
                Some(_) => {
                    let column = Expr::build(token_table, cursor)?;
                    columns.push(column);
                }
                None => break,
            }
        }

        expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Values))?;
        *cursor += 1;

        expect_kind(token_table, cursor, &TokenKind::LeftParen)?;
        *cursor += 1;

        let mut values = MiniVec::new();
        loop {
            match token_table.get_kind(*cursor) {
                Some(TokenKind::Comma) => {
                    *cursor += 1;
                }
                Some(TokenKind::RightParen) => {
                    *cursor += 1;
                    break;
                }
                Some(_) => {
                    let value = Expr::build(token_table, cursor)?;
                    values.push(value);
                }
                None => break,
            }
        }

        if maybe_kind(token_table, cursor, &TokenKind::Eof) {
            *cursor += 1;
        }

        Ok(InsertStatement {
            table,
            columns,
            values,
        })
    }
}
