use std::{fmt::Display, slice::SliceIndex};

use crate::keyword::Keyword;

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u16)]
pub(crate) enum TokenKind {
    Number,
    StringLiteral,
    Identifier,
    Eof,
    Dot,
    LeftParen,
    RightParen,
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
    Keyword(Keyword),
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Number => write!(f, "Number"),
            TokenKind::StringLiteral => write!(f, "StringLiteral"),
            TokenKind::Identifier => write!(f, "Identifier"),
            TokenKind::Eof => write!(f, "Eof"),
            TokenKind::Dot => write!(f, "Dot"),
            TokenKind::LeftParen => write!(f, "LeftParen"),
            TokenKind::RightParen => write!(f, "RightParen"),
            TokenKind::BackSlash => write!(f, "BackSlash"),
            TokenKind::Comma => write!(f, "Comma"),
            TokenKind::Unknown => write!(f, "Unknown"),
            TokenKind::Less => write!(f, "Less"),
            TokenKind::LessEqual => write!(f, "LessEqual"),
            TokenKind::Greater => write!(f, "Greater"),
            TokenKind::GreaterEqual => write!(f, "GreaterEqual"),
            TokenKind::Equal => write!(f, "Equal"),
            TokenKind::NotEqual => write!(f, "NotEqual"),
            TokenKind::Plus => write!(f, "Plus"),
            TokenKind::Subtract => write!(f, "Subtract"),
            TokenKind::Multiply => write!(f, "Multiply"),
            TokenKind::Divide => write!(f, "Divide"),
            TokenKind::Mod => write!(f, "Mod"),
            TokenKind::Keyword(kw) => kw.fmt(f),
        }
    }
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

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            tokens: Vec::with_capacity(capacity),
            positions: Vec::with_capacity(capacity),
        }
    }

    pub(crate) fn push(&mut self, kind: TokenKind, start: usize, end: usize) {
        self.tokens.push(kind);
        self.positions.push((start, end));
    }

    pub(crate) fn get_kind<I>(&self, index: I) -> Option<&I::Output> where I: SliceIndex<[TokenKind]> {
        self.tokens.get(index)
    }

    pub(crate) fn get_position<I>(&self, index: I) -> Option<&I::Output> where I: SliceIndex<[(usize, usize)]> {
        self.positions.get(index)
    }

    pub(crate) fn get_entry(&self, index: usize) -> Option<(&TokenKind, &(usize, usize))> {
        self.tokens.get(index)
            .zip(self.positions.get(index))
    }
}
