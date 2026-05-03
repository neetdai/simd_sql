use minivec::MiniVec;

use crate::{
    ParserError,
    common::{expr::Expr, utils::expect_kind},
    keyword::Keyword,
    token::{TokenKind, TokenTable},
};

#[derive(Debug, PartialEq)]
pub struct Group<'a> {
    pub columns: MiniVec<Expr<'a>>,
}

impl<'a> Group<'a> {
    pub(crate) fn build(
        token_table: &TokenTable<'a>,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Group))?;
        *cursor += 1;
        expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::By))?;
        *cursor += 1;

        let mut columns = MiniVec::new();
        loop {
            match token_table.get_kind(*cursor) {
                Some(TokenKind::Identifier) => {
                    let expr = Expr::build(token_table, cursor)?;
                    columns.push(expr);
                }
                Some(TokenKind::Comma) => {
                    *cursor += 1;
                }
                _ => {
                    break;
                }
            }
        }

        Ok(Self { columns })
    }
}
