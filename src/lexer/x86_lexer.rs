use std::{
    alloc::Allocator, arch::x86_64::*, cmp::Reverse, collections::{BTreeMap, BinaryHeap}, ops::Add, path::is_separator, simd::{LaneCount, Mask, Simd, SupportedLaneCount, cmp::{SimdPartialEq, SimdPartialOrd}}, str::FromStr
};

use minivec::MiniVec;
use phf::phf_map;

use crate::{
    error::ParserError,
    keyword::{Keyword, KeywordMatcher},
    token::{self, TokenKind, TokenTable},
};

const EVEN_BITS: u64 = 0x5555_5555_5555_5555;
const ODD_BITS: u64 = 0xaaaa_aaaa_aaaa_aaaa;

const symbol_map: phf::Map<&'static [u8], TokenKind> = phf_map! {
    b"+" => TokenKind::Plus,
    b"-" => TokenKind::Subtract,
    b"*" => TokenKind::Multiply,
    b"/" => TokenKind::Divide,
    b"%" => TokenKind::Mod,
    b"(" => TokenKind::LeftParen,
    b")" => TokenKind::RightParen,
    b"<" => TokenKind::Less,
    b"<>" => TokenKind::NotEqual,
    b"<=" => TokenKind::LessEqual,
    b">" => TokenKind::Greater,
    b">=" => TokenKind::GreaterEqual,
    b"=" => TokenKind::Equal,
    b"," => TokenKind::Comma,
};

#[derive(Debug, PartialEq, Eq)]
struct TokenItem {
    start: usize,
    end: usize,
    kind: TokenKind,
}

impl Ord for TokenItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for TokenItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub(crate) struct SimdLexer<'a, A: Allocator> {
    inner: &'a [u8],
    position: usize,
    keyword_matcher: &'a KeywordMatcher,
    allocator: &'a A,
}

impl<'a, A: Allocator> SimdLexer<'a, A> {
    pub(crate) fn new(text: &'a str, keyword_matcher: &'a KeywordMatcher, allocator: &'a A) -> Result<Self, ParserError> {
        Ok(Self {
            inner: text.as_bytes(),
            position: 0,
            keyword_matcher,
            allocator,
        })
    }

    #[inline]
    fn skip_whitespace(&mut self) {
        let length = self.inner.len();
        let mut pos = self.position;
        if is_x86_feature_detected!("avx2") {
            while pos + 32 < length {
                let slice = Simd::<u8, 32>::from_slice(&self.inner[pos..pos+32]);

                let space = Simd::<u8, 32>::splat(b' ');
                let tab = Simd::<u8, 32>::splat(b'\t');
                let newline = Simd::<u8, 32>::splat(b'\n');
                let cr = Simd::<u8, 32>::splat(b'\r');

                let mask = slice.simd_eq(space) | slice.simd_eq(tab) | slice.simd_eq(newline) | slice.simd_eq(cr);
                if mask.all() {
                    pos += 32;
                } else {
                    let result = (!mask).to_bitmask();
                    let index = result.trailing_zeros() as usize;
                    pos += index;

                    break;
                }
            }

            if is_x86_feature_detected!("sse4.2") {
                while pos + 16 < length {
                    let slice = Simd::<u8, 16>::from_slice(&self.inner[pos..pos+16]);

                    let space = Simd::<u8, 16>::splat(b' ');
                    let tab = Simd::<u8, 16>::splat(b'\t');
                    let newline = Simd::<u8, 16>::splat(b'\n');
                    let cr = Simd::<u8, 16>::splat(b'\r');

                    let mask = slice.simd_eq(space) | slice.simd_eq(tab) | slice.simd_eq(newline) | slice.simd_eq(cr);
                    if mask.all() {
                        pos += 16;
                    } else {
                        let result = (!mask).to_bitmask();
                        let index = result.trailing_zeros() as usize;
                        pos += index;

                        break;
                    }
                }
            }

            let tmp_pos = pos;
            for index in tmp_pos..length {
                let c = &self.inner[index];
                if *c != b' ' && *c != b'\t' && *c != b'\r' && *c != b'\n' {
                    break;
                }
                pos += 1;
            }
            self.position = pos;
        }
    }

    #[inline]
    fn scan_number(&mut self) -> Result<(TokenKind, usize, usize), ParserError> {
        let start = self.position;
        let mut pos = start;
        let length = self.inner.len();

            if is_x86_feature_detected!("avx2") {
                while pos + 32 < length {
                    let slice = Simd::<u8, 32>::from_slice(&self.inner[pos..pos+32]);

                    let min = Simd::<u8, 32>::splat(b'0' - 1);
                    let max = Simd::<u8, 32>::splat(b'9' + 1);
                    let mask = slice.simd_gt(min) & slice.simd_lt(max);

                    if mask.all() {
                        pos += 32;
                    } else {
                        let result = mask.to_bitmask();
                        let index = result.trailing_zeros() as usize;
                        pos += index;
                        break;
                    }
                }
            }

            if is_x86_feature_detected!("sse4.2") {
                while pos + 16 < length {

                    let slice = Simd::<u8, 16>::from_slice(&self.inner[pos..pos+16]);

                    let min = Simd::<u8, 16>::splat(b'0' - 1);
                    let max = Simd::<u8, 16>::splat(b'9' + 1);

                    let mask = slice.simd_ge(min) & slice.simd_le(max);

                    if mask.all() {
                        pos += 16;                        
                    } else {
                        let result = mask.to_bitmask();
                        let trailling_zeros = result.trailing_zeros();
                        pos += trailling_zeros as usize;
                        break;
                    }
                }
            }

            let tmp_pos = pos;
            for index in tmp_pos..length {
                let c = &self.inner[index];
                if !c.is_ascii_digit() {
                    break;
                }
                pos += 1;
            }
        
        self.position = pos;
        let end = pos;

        Ok((TokenKind::Number, start, end))
    }

    #[inline]
    fn scan_identify(&mut self) -> Result<(TokenKind, usize, usize), ParserError> {
        let start = self.position;
        let mut pos = self.position;
        let length = self.inner.len();

            if is_x86_feature_detected!("avx2") {
                while pos + 32 < length {
                    let slice = Simd::<u8, 32>::from_slice(&self.inner[pos..pos+32]);

                    let digit_mask = slice.simd_ge(Simd::splat(b'0' - 1)) & slice.simd_le(Simd::splat(b'9' + 1));
                    let lower_mask = slice.simd_ge(Simd::splat(b'a' - 1)) & slice.simd_le(Simd::splat(b'z' + 1));
                    let upper_mask = slice.simd_ge(Simd::splat(b'A' - 1)) & slice.simd_le(Simd::splat(b'Z' + 1));
                    let underline_mask = slice.simd_eq(Simd::splat(b'_'));

                    let mask = digit_mask | lower_mask | upper_mask | underline_mask;

                    if mask.all() {
                        pos += 32;
                    } else {
                        let result = mask.to_bitmask();
                        let trailling_zeros = result.trailing_zeros();
                        pos += trailling_zeros as usize;
                        break;
                    }
                }
            }

            if is_x86_feature_detected!("sse4.2") {
                while pos + 16 < length {
                    let slice = Simd::<u8, 16>::from_slice(&self.inner[pos..pos+16]);

                    let digit_mask = slice.simd_ge(Simd::splat(b'0' - 1)) & slice.simd_le(Simd::splat(b'9' + 1));
                    let lower_mask = slice.simd_ge(Simd::splat(b'a' - 1)) & slice.simd_le(Simd::splat(b'z' + 1));
                    let upper_mask = slice.simd_ge(Simd::splat(b'A' - 1)) & slice.simd_le(Simd::splat(b'Z' + 1));
                    let underline_mask = slice.simd_eq(Simd::splat(b'_'));

                    let mask = digit_mask | lower_mask | upper_mask | underline_mask;

                    if mask.all() {
                        pos += 16;
                    } else {
                        let result = mask.to_bitmask();
                        let trailling_zeros = result.trailing_zeros();
                        pos += trailling_zeros as usize;
                        break;
                    }
                }
            }

            let tmp_pos = pos;
            for index in tmp_pos..length {
                let c = &self.inner[index];
                if c.is_ascii_alphanumeric() || c == &b'_' {
                    pos = index;
                } else {
                    break;
                }
            }
        

        let end = pos;
        self.position = pos;

        let source = match self.inner.get(start..=end) {
            Some(s) => s,
            None => return Err(ParserError::InvalidToken(start, end)),
        };

        if let Some(keyword) = self.maybe_keyword(source) {
            Ok((TokenKind::Keyword(keyword), start, end))
        } else {
            Ok((TokenKind::Identifier, start, end))
        }
    }

    // 可能是关键词
    fn maybe_keyword(&self, source: &[u8]) -> Option<Keyword> {
        self.keyword_matcher.match_keyword(source)
        // let len = source.len();
        // let list = self.keyword_matcher.get(len)?;

        // if is_x86_feature_detected!("sse4.2") {
        //     let mut source_array = [0u8; 16];
        //     source_array[0..len].copy_from_slice(source);

        //     let source_chunk = Simd::from_array(source_array);
        //     let lower_mask = source_chunk.simd_gt(Simd::<u8, 16>::splat(b'a' - 1)) & source_chunk.simd_lt(Simd::<u8, 16>::splat(b'z' + 1));

        //     let source_upper = source_chunk - (lower_mask.select(Simd::<u8, 16>::splat(32), Simd::<u8, 16>::splat(0)));

        //     for keyword in list.iter() {
        //         let key_len = keyword.as_str().as_bytes().len();
        //         let mut keyword_array = [0u8; 16];
        //         keyword_array[0..key_len].copy_from_slice(keyword.as_str().as_bytes());
        //         let keyword_chunk = Simd::from_array(keyword_array);
        //         if keyword_chunk.simd_eq(source_upper).all() {
        //             return Some(*keyword);
        //         }
        //     }
        //     None
        // } else {
        //     let tmp = source
        //         .iter()
        //         .copied()
        //         .map(|c| if c.is_ascii_lowercase() { c - 32 } else { c })
        //         .collect::<MiniVec<u8>>();
        //     list.iter()
        //         .copied()
        //         .find(|keyword| keyword.as_str().as_bytes() == tmp.as_slice())
        // }
    }

    // 匹配字符串
    #[inline]
    fn scan_string(&mut self, terminator: u8) -> Result<(TokenKind, usize, usize), ParserError> {
        let start = self.position;
        let mut pos = self.position + 1;
        let length = self.inner.len();

            if is_x86_feature_detected!("avx2") {
                while pos + 32 < length {
                    let slice = Simd::<u8, 32>::from_slice(&self.inner[pos..pos+32]);

                    let target = Simd::from_array([terminator; 32]);
                    let mask = slice.simd_eq(target);

                    if mask.any() {
                        let result = mask.to_bitmask();
                        let index = result.trailing_zeros() as usize;
                        let prev_value = self.inner[pos + index - 1];
                        if prev_value != b'\\' {
                            pos += index;
                            break;
                        } else {
                            pos += index + 1;
                        }
                    } else {
                        pos += 32;
                    }
                }
            }

            if is_x86_feature_detected!("sse4.2") {
                while pos + 16 < length {
                    let slice = Simd::<u8, 16>::from_slice(&self.inner[pos..pos + 16]);
                    let target = Simd::from_array([terminator; 16]);
                    let mask = slice.simd_eq(target);

                    if mask.any() {
                        let result = mask.to_bitmask();
                        let index = result.trailing_zeros() as usize;
                        let prev_value = self.inner[pos + index - 1];
                        if prev_value != b'\\' {
                            pos += index;
                            break;
                        } else {
                            pos += index + 1;
                        }
                    } else {
                        pos += 16;
                    }
                }
            }

            let tmp_pos = pos;
            for index in tmp_pos..length {
                let c = &self.inner[index];
                let prev_c = &self.inner[index - 1];
                if *c == terminator && *prev_c != b'\\' {
                    break;
                }
                pos += 1;
            }

            let end = pos;
            self.position = pos;
            Ok((TokenKind::StringLiteral, start, end))
        
    }

    // 匹配符号
    fn match_symbol(&self, arr: &[(usize, usize)]) -> Vec<TokenItem, &A> {
        
        let len = arr.len();
        let mut list = Vec::with_capacity_in(len, self.allocator);

        for (start, end) in arr {
            let bytes = &self.inner[*start..=*end];
            if let Some(kind) = symbol_map.get(bytes) {
                // list.push(((*start, *end), kind.clone()));
                list.push(TokenItem {
                    start: *start,
                    end: *end,
                    kind: kind.clone(),
                });
            }
        }

        list
    }

    fn match_ident(&self, arr: &[(usize, usize)]) -> Vec<TokenItem, &A> {
        let mut list = Vec::with_capacity_in(arr.len(), self.allocator);
        for (start, end) in arr {
            let bytes = &self.inner[*start..=*end];
            if bytes.iter().all(|b| b.is_ascii_digit()) {
                list.push(TokenItem {
                    start: *start,
                    end: *end,
                    kind: TokenKind::Number,
                });
            } else {
                list.push(TokenItem {
                    start: *start,
                    end: *end,
                    kind: TokenKind::Identifier,
                });
            }
        }

        list
    }

    fn find_escape_branchless(backslash: u64, quote: u64) -> u64 {
        let escape_prefix = (((backslash + (backslash & !(backslash << 1) & EVEN_BITS)) & !backslash) & !EVEN_BITS) | (((backslash + ((backslash & !(backslash << 1)) & ODD_BITS)) & !backslash) & EVEN_BITS);
        escape_prefix & quote
    }

    fn prepare_scan_backslash<const N: usize>(bytes: &[u8]) -> u64 where LaneCount<N>: SupportedLaneCount {
        let source = Simd::<u8, N>::from_slice(bytes);
        let backslash = source.simd_eq(Simd::splat(b'\\'));
        let backslash = backslash.to_bitmask();

        backslash
    }

    fn prepare_scan_quote<const N: usize>(bytes: &[u8], quote: u8) -> u64 where LaneCount<N>: SupportedLaneCount {
        let source = Simd::<u8, N>::from_slice(bytes);
        let quote = source.simd_eq(Simd::splat(quote));
        let quote = quote.to_bitmask();

        quote
    }

    fn prepare_scan_quote_range(&self) -> Vec<usize, &A> {
        let len = self.inner.len();
        let mut position_collect = Vec::<usize, _>::with_capacity_in(len, self.allocator);
        let mut pos = 0;

        while pos + 32 < len {
            let backslash = Self::prepare_scan_backslash::<32>(&self.inner[pos..pos + 32]);
            let single_quote = Self::prepare_scan_quote::<32>(&self.inner[pos..pos + 32], b'\'');
            let double_quote = Self::prepare_scan_quote::<32>(&self.inner[pos..pos + 32], b'"');
            let single_result = Self::find_escape_branchless(backslash, single_quote);
            let double_result = Self::find_escape_branchless(backslash, double_quote);
            
            let mut mask = (single_quote ^ single_result) | (double_quote ^ double_result);

            let mut tmp_pos = pos;
            while mask != 0 {
                let p = mask.trailing_zeros() as usize;
                tmp_pos += p + 1;
                position_collect.push(tmp_pos - 1);
                // mask &= mask - 1;
                mask >>= p + 1;
            }
            
            pos += 32;
        }

        while pos + 16 < len {
            let backslash = Self::prepare_scan_backslash::<16>(&self.inner[pos..pos + 16]);
            let single_quote = Self::prepare_scan_quote::<16>(&self.inner[pos..pos + 16], b'\'');
            let double_quote = Self::prepare_scan_quote::<16>(&self.inner[pos..pos + 16], b'"');
            let single_result = Self::find_escape_branchless(backslash, single_quote);
            let double_result = Self::find_escape_branchless(backslash, double_quote);
            let mut mask = (single_quote ^ single_result) | (double_quote ^ double_result);

            let mut tmp_pos = pos;
            while mask != 0 {
                let p = mask.trailing_zeros() as usize;
                tmp_pos += p + 1;
                position_collect.push(tmp_pos - 1);
                mask >>= p + 1;
            }
            pos += 16;
        }

        let mut single_mask = 0u64;
        let mut double_mask = 0u64;
        let mut backblash_mask = 0u64;
        let mut tmp_pos = pos;
        for (index, c) in self.inner[pos..].iter().enumerate() {
            single_mask |= ((*c == b'\'') as u64) << index;
            double_mask |= ((*c == b'"') as u64) << index;
            backblash_mask |= ((*c == b'\\') as u64) << index;
        }

        let single_result = Self::find_escape_branchless(backblash_mask, single_mask);
        let double_result = Self::find_escape_branchless(backblash_mask, double_mask);
        let mut result = (single_mask ^ single_result) | (double_mask ^ double_result);

        while result != 0 {
            let p = result.trailing_zeros() as usize;
            tmp_pos += p + 1;
            position_collect.push(tmp_pos - 1);
            result >>= p + 1;
        }

        position_collect
    }

    // 单引号或双引号的范围
    fn quote_range(&self, quote: &Vec<usize, &A>) -> Vec<u8, &A> {
        let mut block_range = Vec::with_capacity_in(self.inner.len(), self.allocator);
        block_range.resize(self.inner.len(), 0);

        for chunk in quote.chunks(2) {
            let [start_position, end_position] = chunk.try_into().unwrap();
            for c in &mut block_range[start_position..=end_position] {
                *c = u8::MAX;
            }
        }

        block_range
    }

    fn prepare_scan_symbol<const N: usize>(bytes: &[u8]) -> Mask<i8, N> where LaneCount<N>: SupportedLaneCount {
        let source = Simd::<u8, N>::from_slice(bytes);
        
        let left_bracket = source.simd_eq(Simd::splat(b'('));
        let right_bracket = source.simd_eq(Simd::splat(b')'));
        let comma = source.simd_eq(Simd::splat(b','));
        let less = source.simd_eq(Simd::splat(b'<'));
        let greater = source.simd_eq(Simd::splat(b'>'));
        let equal = source.simd_eq(Simd::splat(b'='));
        let plus = source.simd_eq(Simd::splat(b'+'));
        let sub = source.simd_eq(Simd::splat(b'-'));
        let mul = source.simd_eq(Simd::splat(b'*'));
        let div = source.simd_eq(Simd::splat(b'/'));
        let mod_ = source.simd_eq(Simd::splat(b'%'));
        let eof = source.simd_eq(Simd::splat(b';'));
        // let backslash = source.simd_eq(Simd::splat(b'\\'));

        left_bracket | right_bracket
            | comma | less | greater | equal | plus | sub | mul | div | mod_ | eof
    }

    fn prepare_scan_symbol_range(&self, block_range: &Vec<u8, &A>) -> Vec<usize, &A> {
        let len = self.inner.len();
        let mut position_collect = Vec::<usize, _>::with_capacity_in(len, self.allocator);
        let mut pos = 0;

        while pos + 32 < len {
            let block_range_mask = Simd::from_slice(&block_range[pos..pos + 32]).simd_eq(Simd::splat(u8::MAX));
            let mut mask = Self::prepare_scan_symbol::<32>(&self.inner[pos..pos + 32]) & (!block_range_mask);
            let backslash = Self::prepare_scan_backslash::<32>(&self.inner[pos..pos + 32]);
            let symbol_mask = mask.to_bitmask();
            let result = Self::find_escape_branchless(backslash, symbol_mask);
            let mut mask = symbol_mask ^ result;
            
            let mut tmp_pos = pos;
            while mask != 0 {
                let p = mask.trailing_zeros() as usize;
                tmp_pos += p + 1;
                position_collect.push(tmp_pos - 1);
                mask >>= p + 1;
            }
            pos += 32;
        }

        while pos + 16 < len {
            let block_range_mask = Simd::from_slice(&block_range[pos..pos + 16]).simd_eq(Simd::splat(u8::MAX));
            let mut mask = Self::prepare_scan_symbol::<16>(self.inner[pos..pos + 16].try_into().unwrap()) & (!block_range_mask);
            let backslash = Self::prepare_scan_backslash::<16>(&self.inner[pos..pos + 16]);
            let symbol_mask = mask.to_bitmask();
            let result = Self::find_escape_branchless(backslash, symbol_mask);
            let mut mask = symbol_mask ^ result;

            let mut tmp_pos = pos;
            while mask != 0 {
                let p = mask.trailing_zeros() as usize;
                tmp_pos += p + 1;
                position_collect.push(tmp_pos - 1);
                mask >>= p + 1;
            }

            pos += 16;
        }

        let mut mask = 0;
        let mut backslash_mask = 0;
        let tmp_pos = pos;
        // dbg!(&tmp_pos);
        // dbg!(self.inner[tmp_pos] == b'=');
        for (index, c) in self.inner[tmp_pos..len].iter().enumerate() {
            let t = matches!(c, b'(' | b')' | b'+' | b'-' | b'*' | b'/' | b'=' | b'<' | b'>' | b',' | b'%' | b';');
            let backslash = matches!(c, b'\\');
            let b = block_range[index];

            mask |= ((t as u64) & (!b as u64)) << index;
            backslash_mask = (backslash as u64) << index;
        }

        dbg!(&mask);
        let result = Self::find_escape_branchless(backslash_mask, mask);
        let mut mask = mask ^ result;

        // dbg!(&mask);

        let mut tmp_pos  = pos;
        while mask != 0 {
            let index = mask.trailing_zeros() as usize;
            tmp_pos += index;
            dbg!(&tmp_pos);
            position_collect.push(tmp_pos);
            mask >>= index + 1;
            tmp_pos += 1;
        }

        position_collect
    }

    fn perpare_scan_whitespace_mask<const N: usize>(bytes: &[u8]) -> Mask<i8, N> where LaneCount<N>: SupportedLaneCount {
        let source = Simd::<u8, N>::from_slice(bytes);
        
        let space = source.simd_eq(Simd::splat(b' '));
        let tab = source.simd_eq(Simd::splat(b'\t'));
        let cr = source.simd_eq(Simd::splat(b'\r'));
        let newline = source.simd_eq(Simd::splat(b'\n'));

        space | tab | cr | newline
    }

    fn perpare_scan_no_symbol_and_whitespace(&self, block_range: &Vec<u8, &A>) -> Vec<usize, &A> {
        let len = self.inner.len();
        let mut position_collect = Vec::with_capacity_in(len, self.allocator);
        let mut pos = 0;

        while pos + 32 < len {
            let whitespace_mask = Self::perpare_scan_whitespace_mask::<32>(&self.inner[pos..pos + 32]);
            let symbol_mask = Self::prepare_scan_symbol::<32>(&self.inner[pos..pos + 32]);

            let block_range_mask = Simd::from_slice(&block_range[pos..pos + 32]).simd_eq(Simd::splat(u8::MAX));

            let mut mask = (!whitespace_mask) & (!symbol_mask) & (!block_range_mask);
            let mut mask = mask.to_bitmask() as usize;
            
            let mut tmp_pos = pos;
            while mask != 0 {
                let p = mask.trailing_zeros() as usize;
                tmp_pos += p + 1;
                position_collect.push(tmp_pos - 1);
                // mask &= mask - 1;
                mask >>= p + 1;
            }

            pos += 32;
        }

        while pos + 16 < len {
            let whitespace_mask = Self::perpare_scan_whitespace_mask::<16>(&self.inner[pos..pos + 16]);
            let symbol_mask = Self::prepare_scan_symbol::<16>(&self.inner[pos..pos + 16]);
            let block_range_mask = Simd::from_slice(&block_range[pos..pos + 16]).simd_eq(Simd::splat(u8::MAX));
            let mut mask = (!whitespace_mask) & (!symbol_mask) & (!block_range_mask);
            let mut mask = mask.to_bitmask() as usize;

            let mut tmp_pos = pos;
            while mask != 0 {
                let p = mask.trailing_zeros() as usize;
                tmp_pos += p + 1;
                position_collect.push(tmp_pos - 1);
                mask >>= p + 1;
            }

            pos += 16;
        }

        let mut mask = 0u16;
        let tmp_pos = pos;
        for (index, c) in self.inner[tmp_pos..len].iter().enumerate() {
            let t = matches!(c, b'(' | b')' | b'\'' | b'"' | b' ' | b'\t' | b'\n' | b'\r' | b'+' | b'-' | b'*' | b'/' | b'=' | b'<' | b'>' | b',' | b'%' | b';');
            let b = block_range[index];
            mask |= ((!t as u16) & (!b as u16)) << index;
        }
        // dbg!(&mask);

        let mut tmp_pos = pos;
        while mask != 0 {
            let p = mask.trailing_zeros() as usize;
            tmp_pos += p + 1;
            position_collect.push(tmp_pos -1);

            mask >>= p + 1;
        }

        position_collect
    }

    fn find_continuous_ranges(&self, arr: &[usize]) -> Vec<(usize, usize), &A> {
        // dbg!(&arr);
        let len = arr.len();
        let mut ranges = Vec::with_capacity_in(len, self.allocator);

        if len > 0 {
            let mut start = arr[0];
        let mut prev = arr[0];
            for current in &arr[1..] {
                if *current != prev + 1 {
                    ranges.push((start, prev));
                    start = *current;
                }
                prev = *current;
            }

            ranges.push((start, prev));
        }

        ranges
    }

    pub(crate) fn tokenize(&mut self) -> Result<TokenTable, ParserError> {
        let quote = self.prepare_scan_quote_range();
        let quote_block_range = self.quote_range(&quote);
        let no_whitespace_and_symbol_position_collect = self.perpare_scan_no_symbol_and_whitespace(&quote_block_range);
        let position_collect = self.prepare_scan_symbol_range(&quote_block_range);
        dbg!(&position_collect);
        let no_whitespace_and_symbol_ranges = self.find_continuous_ranges(&no_whitespace_and_symbol_position_collect);

        let symbol_ranges = self.find_continuous_ranges(&position_collect);
        let symbol_token = self.match_symbol(&symbol_ranges);
        let no_whitespace_and_symbol_token = self.match_ident(&no_whitespace_and_symbol_ranges);
        let quote_token = quote.chunks(2).map(|positions| {
            TokenItem {
                start: positions[0],
                end: positions[1],
                kind: TokenKind::StringLiteral
            }
        }).collect::<Vec<TokenItem>>();

        // dbg!(&quote_block_range);
        // dbg!(&quote);
        // dbg!(&no_whitespace_and_symbol_ranges);
        // dbg!(&symbol_token);

        // let mut heap = BinaryHeap::with_capacity_in(symbol_token.len() + no_whitespace_and_symbol_ranges.len() + quote.len(), self.allocator);
        // let mut heap = Vec::with_capacity_in(symbol_token.len() + no_whitespace_and_symbol_ranges.len() + quote.len(), self.allocator);
        // for symbol in symbol_token {
        //     heap.push(Reverse(symbol));
        //     // heap.push(symbol);
        // }
        // for no_whitespace_and_symbol in no_whitespace_and_symbol_token {
        //     heap.push(Reverse(no_whitespace_and_symbol));
        //     // heap.push(no_whitespace_and_symbol);
        // }

        let mut table = TokenTable::with_capacity(symbol_token.len() + no_whitespace_and_symbol_ranges.len() + quote_token.len());
        let mut symbol_offset = 0;
        let mut no_whitespace_and_symbol_offset = 0;
        let mut quote_offset = 0;

        let symbol_len = symbol_token.len();
        let no_whitespace_and_symbol_len = no_whitespace_and_symbol_token.len();
        let quote_len = quote_token.len();

        let default_max_token = TokenItem {
            start: usize::MAX,
            end: usize::MAX,
            kind: TokenKind::Unknown,
        };

        while (symbol_offset + no_whitespace_and_symbol_offset + quote_offset) < (symbol_len + no_whitespace_and_symbol_len + quote_len) {
            let symbol = symbol_token.get(symbol_offset).unwrap_or(&default_max_token);
            let no_whitespace_and_symbol = no_whitespace_and_symbol_token.get(no_whitespace_and_symbol_offset).unwrap_or(&default_max_token);
            let quote = quote_token.get(quote_offset).unwrap_or(&default_max_token);

            let min = symbol.min(no_whitespace_and_symbol).min(quote);

            table.push(min.kind.clone(), min.start, min.end);

            if min == symbol {
                symbol_offset += 1;
            } else if min == no_whitespace_and_symbol {
                no_whitespace_and_symbol_offset += 1;
            } else {
                quote_offset += 1;
            }
            // dbg!(&symbol_offset, &no_whitespace_and_symbol_offset, &quote_offset);
        }
        // for reverse in heap.into_iter_sorted() {
        //     let token_item = reverse.0;
        //     table.push(token_item.kind, token_item.start, token_item.end);
        // }

        // dbg!(&table);
        Ok(table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{TokenKind, TokenTable};
    use bumpalo::Bump;

    #[test]
    fn test_skip_whitespace() {
        let alloc = Bump::new();
        let binding = &alloc;
        let keyword_matcher = KeywordMatcher::new();
        let mut lexer = SimdLexer::new(
            r#"                 
                "#,
            &keyword_matcher,
            &binding
        )
        .unwrap();
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            TokenTable {
                tokens: vec![],
                positions: vec![],
            }
        );
    }

    #[test]
    fn test_match_number() {
        let alloc = Bump::new();
        let binding = &alloc;
        let keyword_matcher = KeywordMatcher::new();
        let mut lexer = SimdLexer::new("1234567890", &keyword_matcher, &binding).unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::Number,],
                positions: vec![(0, 9)],
            }
        );

        let mut lexer = SimdLexer::new(
            "123451111111111111111111111111111111111111 2222222222222222222222222222",
            &keyword_matcher,
            &binding
        )
        .unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::Number, TokenKind::Number],
                positions: vec![(0, 41), (43, 70)],
            }
        );

        
    }

    #[test]
    fn test_tokenize_cmp() {
        let alloc = Bump::new();
        let binding = &alloc;
        let keyword_matcher = KeywordMatcher::new();
        let mut lexer = SimdLexer::new("a > b >= c < d <= e <> f = g", &keyword_matcher, &binding).unwrap();
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            TokenTable {
                tokens: vec![
                    TokenKind::Identifier,
                    TokenKind::Greater,
                    TokenKind::Identifier,
                    TokenKind::GreaterEqual,
                    TokenKind::Identifier,
                    TokenKind::Less,
                    TokenKind::Identifier,
                    TokenKind::LessEqual,
                    TokenKind::Identifier,
                    TokenKind::NotEqual,
                    TokenKind::Identifier,
                    TokenKind::Equal,
                    TokenKind::Identifier
                ],
                positions: vec![
                    (0, 0),
                    (2, 2),
                    (4, 4),
                    (6, 7),
                    (9, 9),
                    (11, 11),
                    (13, 13),
                    (15, 16),
                    (18, 18),
                    (20, 21),
                    (23, 23),
                    (25, 25),
                    (27, 27)
                ],
            }
        )
    }

    #[test]
    fn test_match_string() {
        let alloc = Bump::<64>::with_min_align();
        let binding = &alloc;
        let keyword_matcher = KeywordMatcher::new();
        let mut lexer = SimdLexer::new("'helloWorld'", &keyword_matcher, &binding).unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::StringLiteral],
                positions: vec![(0, 11)],
            }
        );

        let mut lexer = SimdLexer::new(r#"'hello\\'World'"#, &keyword_matcher, &binding).unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::StringLiteral],
                positions: vec![(0, 14)],
            }
        );

        let mut lexer = SimdLexer::new(
            "'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'",
            &keyword_matcher,
            &binding
        )
        .unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::StringLiteral],
                positions: vec![(0, 60)],
            }
        );

        let mut lexer = SimdLexer::new(
            "\'aaaaaaaaaaaaa\\'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\'",
            &keyword_matcher,
            &binding
        )
        .unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::StringLiteral],
                positions: vec![(0, 64)],
            }
        );
    }

    #[test]
    fn test_match_indentify() {
        let alloc = Bump::<64>::with_min_align();
        let binding = &alloc;
        let keyword_matcher = KeywordMatcher::new();
        let mut lexer = SimdLexer::new("asdfghjk", &keyword_matcher, &binding).unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::Identifier],
                positions: vec![(0, 7)],
            }
        );

        let mut lexer = SimdLexer::new(
            "qwertyuiopASDFGHJKL1234567890_zxcvbnm 1234567890",
            &keyword_matcher,
            &binding
        )
        .unwrap();
        let token = lexer.tokenize().unwrap();
        assert_eq!(
            token,
            TokenTable {
                tokens: vec![TokenKind::Identifier, TokenKind::Number],
                positions: vec![(0, 36), (38, 48)],
            }
        );
    }

    #[test]
    fn test_keyword() {
        let alloc = Bump::<64>::with_min_align();
        let binding = &alloc;
        let keyword_matcher = KeywordMatcher::new();
        let mut lexer = SimdLexer::new("select from", &keyword_matcher, &binding).unwrap();
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            TokenTable {
                tokens: vec![
                    TokenKind::Keyword(Keyword::Select),
                    TokenKind::Keyword(Keyword::From)
                ],
                positions: vec![(0, 5), (7, 10)],
            }
        );
    }

    #[test]
    fn test_sql() {
        let alloc = Bump::<64>::with_min_align();
        let binding = &alloc;
        let keyword_matcher = KeywordMatcher::new();
        let mut lexer = SimdLexer::new("select * from a", &keyword_matcher, &binding).unwrap();
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            TokenTable {
                tokens: vec![
                    TokenKind::Keyword(Keyword::Select),
                    TokenKind::Multiply,
                    TokenKind::Keyword(Keyword::From),
                    TokenKind::Identifier,
                ],
                positions: vec![(0, 5), (7, 7), (9, 12), (14, 14)],
            }
        );
    }
}
