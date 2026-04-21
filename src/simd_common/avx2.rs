use super::common::SimdTrait;
use std::arch::x86_64::{self};

pub(crate) struct Avx2;

impl SimdTrait for Avx2 {
    const LENGTH: usize = 32;

    fn find_consecutive_in_range(slice: &[u8], matches: (u8, u8), start_pos: usize) -> (usize, isize) {
        let mut end_pos = -1;
        let mut pos = start_pos;
        let len = slice.len();
        unsafe {
            let matches_range = {
                let a_lane = x86_64::_mm256_set1_epi8(matches.0.cast_signed() - 1);
                let b_lane = x86_64::_mm256_set1_epi8(matches.1.cast_signed() + 1);
                (a_lane, b_lane)
            };

            while pos + Self::LENGTH <= len {
                let ptr = slice.as_ptr().add(pos).cast();
                let ptr = x86_64::_mm256_loadu_epi8(ptr);

                let cmp_a = x86_64::_mm256_cmpgt_epi8(ptr, matches_range.0);
                let cmp_b = x86_64::_mm256_cmpgt_epi8(matches_range.1, ptr);
                let cmp = x86_64::_mm256_and_si256(cmp_a, cmp_b);
                let mask = x86_64::_mm256_movemask_epi8(cmp);

                if mask != -1 {
                    let trailing_ones = mask.trailing_ones();
                    pos += trailing_ones as usize;
                    break;
                } else {
                    pos += Self::LENGTH;
                }
            }
        }
        if start_pos < pos {
            end_pos = (pos - 1).cast_signed();
        }
        (start_pos, end_pos)
    }

    fn longest_consecutive_matching<const N: usize>(slice: &[u8], matches: [u8; N], start_pos: usize) -> (usize, isize) {
        let mut end_pos = -1;
        let mut pos = start_pos;
        let len = slice.len();
        
        unsafe {
            let match_lanes = matches.map(|m| {
                x86_64::_mm256_set1_epi8(m.cast_signed())
            });

            while pos + Self::LENGTH < len {
                let ptr = slice.as_ptr().add(pos).cast();
                let ptr = x86_64::_mm256_loadu_epi8(ptr);

                let cmp = match_lanes.iter().fold(x86_64::_mm256_set1_epi8(0), |prev, &lane| {
                    let cmp = x86_64::_mm256_cmpeq_epi8(ptr, lane);
                    x86_64::_mm256_or_si256(prev, cmp)
                });

                let mask = x86_64::_mm256_movemask_epi8(cmp);

                if mask != -1 {
                    let trailing_ones = mask.trailing_ones();
                    pos += trailing_ones as usize;
                    break;
                } else {
                    pos += Self::LENGTH;
                }
            }
        }

        if start_pos < pos {
            end_pos = (pos - 1).cast_signed();
        }

        (start_pos, end_pos)
    }

    fn mixed_match<const N1: usize, const N2: usize>(slice: &[u8], match_range: [(u8, u8); N1], matches2: [u8; N2], start_pos: usize) -> (usize, isize) {
        let mut end_pos = -1;
        let mut pos = start_pos;
        let len = slice.len();

        unsafe {
            let matches_range = match_range.map(|(a, b)| {
                let a_lane = x86_64::_mm256_set1_epi8(a.cast_signed() - 1);
                let b_lane = x86_64::_mm256_set1_epi8(b.cast_signed() + 1);
                (a_lane, b_lane)
            });

            let match_lanes = matches2.map(|m| {
                x86_64::_mm256_set1_epi8(m.cast_signed())
            });

            while pos + Self::LENGTH < len {
                let ptr = slice.as_ptr().add(pos).cast();
                let ptr = x86_64::_mm256_loadu_epi8(ptr);

                let range_cmp = matches_range.iter().fold(x86_64::_mm256_set1_epi8(0), |prev, &(a_lane, b_lane)| {
                    let cmp_a = x86_64::_mm256_cmpgt_epi8(ptr, a_lane);
                    let cmp_b = x86_64::_mm256_cmpgt_epi8(b_lane, ptr);
                    let cmp = x86_64::_mm256_and_si256(cmp_a, cmp_b);
                    x86_64::_mm256_or_si256(prev, cmp)
                });

                let match_cmp = match_lanes.iter().fold(x86_64::_mm256_set1_epi8(0), |prev, &lane| {
                    let cmp = x86_64::_mm256_cmpeq_epi8(ptr, lane);
                    x86_64::_mm256_or_si256(prev, cmp)
                });

                let cmp = x86_64::_mm256_or_si256(range_cmp, match_cmp);
                let mask = x86_64::_mm256_movemask_epi8(cmp);
                // dbg!(&mask);

                if mask != -1 {
                    let trailing_ones = mask.trailing_ones();
                    pos += trailing_ones as usize;
                    break;
                } else {
                    pos += Self::LENGTH;
                }
            }
        }

        if start_pos < pos {
            end_pos = (pos - 1).cast_signed();
        }

        (start_pos, end_pos)
    }
}

mod test {
    use crate::simd_common::{Avx2, SimdTrait};


    #[test]
    fn avx2_test1() {
        let slice = b"1234567890qwertyuiopasdfghjklzxcvbnm";
        let (start, end) = Avx2::find_consecutive_in_range(slice, (b'0', b'9'), 0);
        assert_eq!(start, 0);
        assert_eq!(end, 9);

        let slice = b"1234567qwertyuiopasdfghjklzxcvbnm";
        let (start, end) = Avx2::find_consecutive_in_range(slice, (b'0', b'9'), 0);
        assert_eq!(start, 0);
        assert_eq!(end, 6);

        let slice = b"12345678901234567890123456789012334567890";
        let (start, end) = Avx2::find_consecutive_in_range(slice, (b'0', b'9'), 0);
        assert_eq!(start, 0);
        assert_eq!(end, 31);

        let slice = b"bqwertyuiopasdfghjklzxcvbnm";
        let (start, end) = Avx2::find_consecutive_in_range(slice, (b'0', b'9'), 0);
        assert_eq!(start, 0);
        assert_eq!(end, -1);

        let slice = b"b023q2w142e245rtyuiopasdfghjklzxcvbnm";
        let (start, end) = Avx2::find_consecutive_in_range(slice, (b'0', b'9'), 0);
        assert_eq!(start, 0);
        assert_eq!(end, -1);
    }

    #[test]
    fn avx2_test2() {
        let slice = b"qwertyuiopasdfghjklzxcvbnm1234567890";
        let (start, end) = Avx2::longest_consecutive_matching(slice, [b'q', b'w', b'e'], 0);
        assert_eq!(start, 0);
        assert_eq!(end, 2);

        let slice = b"aaaaaaaaaaaaaaaaaaaaaaaaabbbbbbbbbbbbbbaaaaaaaaaaaaaaaaa";
        let (start, end) = Avx2::longest_consecutive_matching(slice, [b'a', b'b', b'c'], 0);
        assert_eq!(start, 0);
        assert_eq!(end, 31);

        let slice = b"qwretyuiopasdfghjklzxcvbnm1234567890";
        let (start, end) = Avx2::longest_consecutive_matching(slice, [b'q', b'w', b'e'], 0);
        assert_eq!(start, 0);
        assert_eq!(end, 1);
    }

    #[test]
    fn avx2_test3() {
        let slice = b"qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM_";
        let (start, end) = Avx2::mixed_match(slice, [(b'a', b'z'), (b'A', b'Z')], [b'_'], 0);
        assert_eq!(start, 0);
        assert_eq!(end, 31);

        let slice = b"qweqwwe!@#$%^zxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM_";
        let (start, end) = Avx2::mixed_match(slice, [(b'a', b'z'), (b'A', b'Z')], [b'_'], 0);
        assert_eq!(start, 0);
        assert_eq!(end, 6);
    }
}