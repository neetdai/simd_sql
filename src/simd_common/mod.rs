mod common;
mod func;
mod sse;

pub(crate) use common::SimdTrait;
pub(crate) use func::{find_consecutive_in_range, longest_consecutive_matching, mixed_match};
pub(crate) use sse::Sse;
