use std::simd::{Mask, Simd, cmp::SimdPartialEq, u8x64};

#[derive(Debug)]
pub(crate) struct TwoPassLexer<'a> {
    inner: &'a [u8],
    position: usize,
}

impl<'a> TwoPassLexer<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self {
            inner: text.as_bytes(),
            position: 0,
        }
    }
}
