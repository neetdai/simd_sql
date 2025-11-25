#![feature(portable_simd)]
#![feature(allocator_api)]

mod error;
mod keyword;
mod lexer;
pub mod parser;
mod token;

pub use {error::ParserError, parser::Parser};
