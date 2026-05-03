use crate::{Statement, error::ParserError, keyword::KeywordMap, lexer::Lexer, token::TokenTable};
use simdutf8::basic::from_utf8;

#[derive(Debug)]
pub struct Parser {
    keyword_map: KeywordMap,
}

impl Parser {
    pub fn new() -> Result<Self, ParserError> {
        Ok(Self {
            keyword_map: KeywordMap::new()
                .map_err(|err| ParserError::AhoCorasickBuild(err.to_string()))?,
        })
    }

    pub fn parse<'a>(&'a self, text: &'a str) -> Result<Statement<'a>, ParserError> {
        let text = from_utf8(text.as_bytes())?;
        let mut tokentable = TokenTable::with_source(text);
        {
            let mut lexer = Lexer::new(text, &self.keyword_map)?;
            lexer.tokenize(&mut tokentable)?;
        }
        let mut cursor = 0;
        let statement = Statement::new(&tokentable, &mut cursor)?;
        Ok(statement)
    }
}
