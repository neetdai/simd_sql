#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TokenKind {
    LineBreak,
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
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    Plus,
    Subtract,
    Multiply,
    Divide,
    Mod,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) start_position: usize,
    pub(crate) end_position: usize,
}
