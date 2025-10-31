#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TokenKind {
    Number,
    StringLiteral,
    Identifier,
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

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TokenTable {
    pub(crate) tokens: Vec<TokenKind>,
    pub(crate) positions: Vec<(usize, usize)>,
}

impl TokenTable {
    pub(crate) fn new() -> Self {
        Self {
            tokens: Vec::new(),
            positions: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, kind: TokenKind, start: usize, end: usize) {
        self.tokens.push(kind);
        self.positions.push((start, end));
    }
}