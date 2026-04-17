mod avx512;
mod common;
mod avx2;
mod sse;
mod func;

pub(crate) use avx512::Avx512;
pub(crate) use common::SimdTrait;
pub(crate) use avx2::Avx2;
pub(crate) use sse::Sse;
pub(crate) use func::{find_consecutive_in_range, longest_consecutive_matching};