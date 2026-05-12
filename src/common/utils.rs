use crate::{
    ParserError,
    token::{TokenKind, TokenTable},
};

pub(crate) fn expect_kind(
    token_table: &TokenTable,
    cursor: &usize,
    token_kind: &TokenKind,
) -> Result<(), ParserError> {
    match token_table.get_kind(*cursor) {
        Some(kind) if kind != token_kind => Err(ParserError::UnexpectedToken {
            expected: token_kind.clone(),
            found: kind.clone(),
        }),
        None => Err(ParserError::UnexpectedToken {
            expected: token_kind.clone(),
            found: TokenKind::Unknown,
        }),
        _ => Ok(()),
    }
}

pub(crate) fn maybe_kind(token_table: &TokenTable, cursor: &usize, token_kind: &TokenKind) -> bool {
    if let Some(kind) = token_table.get_kind(*cursor) {
        kind == token_kind
    } else {
        false
    }
}
