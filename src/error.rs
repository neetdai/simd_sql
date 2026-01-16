use simdutf8::basic::Utf8Error;
use thiserror::Error;

use crate::token::TokenKind;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParserError {
    #[error("invalid utf-8")]
    InvalidUtf8(#[from] Utf8Error),

    #[error("ahocorasick error `{0}`")]
    AhoCorasickBuild(String),

    #[error("invalid token from `{0}` to `{1}`")]
    InvalidToken(usize, usize),

    #[error("syntax error from `{0}` to `{1}`")]
    SyntaxError(usize, usize),

    #[error("unexpected token: expected `{expected}`, found `{found}`")]
    UnexpectedToken { expected: TokenKind, found: TokenKind },
}
