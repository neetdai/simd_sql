#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TokenKind {
    Symbol,
    Number,
    StringLiteral,
    Eof,
    LeftParen,
    RightParen,
    SingleQuotation,
    DoubleQuotation,
    BackSlash,
    Comma,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) start_position: usize,
    pub(crate) end_position: usize,
}
