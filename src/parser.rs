use crate::error::ParserError;
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
        Ok(())
    }
}
