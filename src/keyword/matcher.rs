use std::{cmp::Ordering, collections::VecDeque, simd::{Simd, cmp::SimdPartialOrd}};

use minivec::MiniVec;
use strum::VariantArray;

use crate::keyword::Keyword;


#[derive(Debug)]
pub(crate) struct KeywordMatcher {
    ultra_short_matcher: UltraShortMatcher,
    short_matcher: ShortMatcher,
}

impl KeywordMatcher {
    pub(crate) fn new() -> Self {

        let mut ultra_short_keyword = Keyword::VARIANTS.iter().filter(|k| k.as_ref().as_bytes().len() ==2).copied().collect::<Vec<_>>();
        ultra_short_keyword.sort_by(|k1, k2| {
            let k1_str = k1.as_ref().as_bytes();
            let k2_str = k2.as_ref().as_bytes();
            k1_str.cmp(&k2_str)
        });

        let short_keyword = Keyword::VARIANTS.iter().filter(|k| {
            let len = k.as_ref().as_bytes().len();
            len > 2 && len < 8
        }).copied().collect::<MiniVec<_>>();

        Self {
            ultra_short_matcher: UltraShortMatcher::new(ultra_short_keyword.as_slice()),
            short_matcher: ShortMatcher::new(short_keyword),
        }
    }

    pub(crate) fn match_keyword(&self, bytes: &[u8]) -> Option<Keyword> {
        // let upper_bytes = bytes.to_ascii_uppercase();
        let upper_bytes = Self::to_ascii_uppercase(bytes);

        if upper_bytes.len() == 2 {
            return  self.ultra_short_matcher.match_keyword(&upper_bytes);
        } else if upper_bytes.len() > 2 && upper_bytes.len() < 8 {
            return self.short_matcher.match_keyword(&upper_bytes);
        } else {
            None
        }
    }

    fn to_ascii_uppercase(bytes: &[u8]) -> Vec<u8> {
        let len = bytes.len();
        let mut result = Vec::with_capacity(len);
        let mut pos = 0;

        while pos + 32 < len {
            let slice = Simd::<u8, 32>::from_slice(&bytes[pos..pos+32]);
            let lower_mask = slice.simd_gt(Simd::<u8, 32>::splat(b'a' - 1)) & slice.simd_lt(Simd::<u8, 32>::splat(b'z' + 1));
            let upper = slice - (lower_mask.select(Simd::<u8, 32>::splat(32), Simd::<u8, 32>::splat(0)));
        
            result.extend_from_slice(upper.as_array().as_slice());
            pos += 32;
        }

        while pos + 16 < len {
            let slice = Simd::<u8, 16>::from_slice(&bytes[pos..pos+16]);
            let lower_mask = slice.simd_gt(Simd::<u8, 16>::splat(b'a' - 1)) & slice.simd_lt(Simd::<u8, 16>::splat(b'z' + 1));
            let upper = slice - (lower_mask.select(Simd::<u8, 16>::splat(32), Simd::<u8, 16>::splat(0)));

            result.extend_from_slice(upper.as_array().as_slice());
            pos += 16;
        }

        while pos + 4 < len {
            let slice = Simd::<u8, 4>::from_slice(&bytes[pos..pos+4]);
            let lower_mask = slice.simd_gt(Simd::<u8, 4>::splat(b'a' - 1)) & slice.simd_lt(Simd::<u8, 4>::splat(b'z' + 1));
            let upper = slice - (lower_mask.select(Simd::<u8, 4>::splat(32), Simd::<u8, 4>::splat(0)));

            result.extend_from_slice(upper.as_array().as_slice());
            pos += 4;
        }

        while pos < len {
            let mut c = bytes[pos];
            c.make_ascii_uppercase();
            result.push(c);
            pos += 1;
        }

        result
    }
}

#[derive(Debug)]
struct UltraShortMatcher {
    two_char: [Option<Keyword>; 65536],
}

impl UltraShortMatcher {
    fn new(two_char_list: &[Keyword]) -> Self {
        let mut two_char = [None; 65536];

        for keyword in two_char_list {
            let bytes = keyword.as_ref().as_bytes();
            let index = ((bytes[0] as usize) << 8) | bytes[1] as usize;

            two_char[index] = Some(*keyword);
        }

        Self { 
            two_char
        }
    }

    fn match_keyword(&self, bytes: &[u8]) -> Option<Keyword> {
        let index = ((bytes[0] as usize) << 8) | bytes[1] as usize;
        self.two_char[index]
    }
}

#[derive(Debug)]
struct ShortMatcher {
    keys: MiniVec<u64>,
    keywords: MiniVec<Keyword>,
}

impl ShortMatcher {
    fn new(mut keywords: MiniVec<Keyword>) -> Self {
        keywords.sort_by(|k1, k2| {
            let k1_pattern = Self::create_short_pattern(k1.as_ref().as_bytes());
            let k2_pattern = Self::create_short_pattern(k2.as_ref().as_bytes());

            k1_pattern.cmp(&k2_pattern)
        });

        let keys = keywords.iter().map(|keyword| Self::create_short_pattern(keyword.as_ref().as_bytes())).collect();

        Self {
            keys,
            keywords,
        }
    }

    fn create_short_pattern(bytes: &[u8]) -> u64 {
        let mut data = 0u64;
        
        // 将关键字字节打包到 u64 中（小端序）
        for (i, &byte) in bytes.iter().enumerate() {
            data |= (byte as u64) << (i * 8);
        }
        
        data
    }

    fn match_keyword(&self, bytes: &[u8]) -> Option<Keyword> {
        let data = Self::create_short_pattern(bytes);

        if let Ok(position ) = self.keys.binary_search(&data) {
            Some(self.keywords[position])
        } else {
            None
        }
    }
}