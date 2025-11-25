use std::alloc::{Allocator, Global};

use crate::{
    error::ParserError,
    keyword::KeywordMatcher,
    lexer::{Lexer, SimdLexer},
};
use bumpalo::Bump;
use simdutf8::basic::from_utf8;

#[derive(Debug)]
pub struct Parser<A: Allocator> {
    allocator: A,
    keyword_matcher: KeywordMatcher,
}

impl<A: Allocator> Parser<A> {
    pub fn new(allocator: A) -> Result<Self, ParserError> {
        Ok(Self {
            allocator,
            keyword_matcher: KeywordMatcher::new(),
        })
    }

    pub fn parse(&self, text: &str) -> Result<(), ParserError> {
        let text = from_utf8(text.as_bytes())?;
        let tokentable = {
            if is_x86_feature_detected!("avx2") {
                let mut lexer = SimdLexer::new(&text, &self.keyword_matcher, &self.allocator)?;
                lexer.tokenize()?
            } else {
                let mut lexer = Lexer::new(&text, &self.keyword_matcher)?;
                lexer.tokenize()?
            }
        };
        Ok(())
    }
}
