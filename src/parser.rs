use crate::{error::ParserError, lexer::{Lexer, SimdLexer}};
use bumpalo::Bump;
use simdutf8::basic::from_utf8;

#[derive(Debug)]
pub struct Parser<'a> {
    text: &'a str,
    arena: Bump,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Result<Self, ParserError> {
        let text = from_utf8(text.as_bytes())?;
        Ok(Self {
            text,
            arena: Bump::new(),
        })
    }

    pub fn parse(&self) -> Result<(), ParserError> {
        let tokentable = {
            if is_x86_feature_detected!("avx2") {
                let mut lexer = SimdLexer::new(&self.text)?;
                lexer.tokenize()?
            } else {
                let mut lexer = Lexer::new(&self.text)?;
                lexer.tokenize()?
            }
        };
        Ok(())
    }
}
