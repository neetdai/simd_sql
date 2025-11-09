use crate::{error::ParserError, keyword::KeywordMap, lexer::{Lexer, SimdLexer}};
use bumpalo::Bump;
use simdutf8::basic::from_utf8;

#[derive(Debug)]
pub struct Parser {
    arena: Bump,
    keyword_map: KeywordMap,
}

impl Parser {
    pub fn new() -> Result<Self, ParserError> {
        
        Ok(Self {
            arena: Bump::new(),
            keyword_map: KeywordMap::new(),
        })
    }

    pub fn parse(&self, text: &str) -> Result<(), ParserError> {
        let text = from_utf8(text.as_bytes())?;
        let tokentable = {
            if is_x86_feature_detected!("avx2") {
                let mut lexer = SimdLexer::new(&text, &self.keyword_map)?;
                lexer.tokenize()?
            } else {
                let mut lexer = Lexer::new(&text, &self.keyword_map)?;
                lexer.tokenize()?
            }
        };
        Ok(())
    }
}
