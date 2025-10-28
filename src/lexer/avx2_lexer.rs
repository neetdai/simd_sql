use std::arch::x86_64::*;

use crate::{error::ParserError, token::Token};

const UNKNOWN: u8 = 0;
const WHITESPACE: u8 = 1;
const LINE_BREAK: u8 = 2;
const NUMBER: u8 = 3;
const STRING_LITERAL: u8 = 4;
const EOF: u8 = 5;
const LEFT_PAREN: u8 = 6;
const RIGHT_PAREN: u8 = 7;
const SINGLE_QUOTATION: u8 = 8;
const DOUBLE_QUOTATION: u8 = 9;
const BACKSLASH: u8 = 10;
const COMMA: u8 = 11;

#[derive(Debug)]
pub(crate) struct SimdLexer<'a> {
    inner: &'a [u8],
    position: usize,
}

impl<'a> SimdLexer<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self {
            inner: text.as_bytes(),
            position: 0,
        }
    }

    fn character_masks(bytes: &[u8]) -> (__m256i, __m256i, __m256i, __m256i) {
        todo!()
    }

    #[inline]
    fn find_byte_position(bytes: &[u8], target: u8) -> u32 {
        unsafe {
            let target_vec = _mm256_set1_epi8(target as i8);
            let bytes_ptr = bytes.as_ptr() as *const __m256i;
            let bytes_data = _mm256_loadu_si256(bytes_ptr); // 加载256位数据

            let cmp_result = _mm256_cmpeq_epi8(bytes_data, target_vec);
            let mask = _mm256_movemask_epi8(cmp_result).cast_unsigned(); // 获取掩码
            mask
        }
    }

    #[inline]
    fn find_range_position(bytes: &[u8], min: u8, max: u8) -> u32 {
        unsafe {
            let min_vec = _mm256_set1_epi8(min as i8);
            let max_vec = _mm256_set1_epi8(max as i8);
            let bytes_ptr = bytes.as_ptr() as *const __m256i;
            let bytes_data = _mm256_loadu_si256(bytes_ptr); // 加载256位数据

            let cmp_min = _mm256_cmpgt_epi8(bytes_data, min_vec);
            let cmp_max = _mm256_cmpgt_epi8(max_vec, bytes_data);
            let cmp_result = _mm256_and_si256(cmp_min, cmp_max);
            let mask = _mm256_movemask_epi8(cmp_result).cast_unsigned(); // 获取掩码

            mask
        }
    }

    #[inline]
    fn find_mixed_positions(bytes: &[u8], targets: &[u8]) -> u32 {
        unsafe {
            let mut combined_mask = 0u32;

            for &target in targets {
                let target_vec = _mm256_set1_epi8(target as i8);
                let bytes_ptr = bytes.as_ptr() as *const __m256i;
                let bytes_data = _mm256_loadu_si256(bytes_ptr); // 加载256位数据

                let cmp_result = _mm256_cmpeq_epi8(bytes_data, target_vec);
                let mask = _mm256_movemask_epi8(cmp_result).cast_unsigned(); // 获取掩码

                combined_mask |= mask;
            }

            combined_mask
        }
    }

    pub(crate) fn tokenize(&mut self) -> Result<Vec<Token>, ParserError> {
        let mut tokens = Vec::new();

        Ok(tokens)
    }
}
