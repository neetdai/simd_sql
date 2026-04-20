#![feature(portable_simd)]
#![feature(allocator_api)]

pub mod ast;
pub mod common;
mod error;
mod keyword;
mod lexer;
pub mod parser;
mod token;
mod simd_common;
// mod common_tmp;

pub use {error::ParserError, parser::Parser};
// pub use common_tmp::Expr;
pub use ast::{
    select::SelectStatement,
    insert::InsertStatement,
    statement::Statement,
    query::Query,
};

pub(crate) use simd_common::{find_consecutive_in_range, longest_consecutive_matching};