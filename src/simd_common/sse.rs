use std::arch::x86_64;
use super::common::SimdTrait;

pub(crate) struct Sse;

impl SimdTrait for Sse {
    const LENGTH: usize = 16;

    fn find_consecutive_in_range(slice: &[u8], matches: (u8, u8), start_pos: usize) -> (usize, usize) {
        let mut end_pos = start_pos;
        let mut pos = start_pos;
        let len = slice.len();

        unsafe {
            let matches_range = {
                let lane_a = x86_64::_mm_set1_epi8(matches.0.cast_signed() - 1);
                let lane_b = x86_64::_mm_set1_epi8(matches.1.cast_signed() + 1);
                (lane_a, lane_b)
            };

            while pos + Self::LENGTH < len {
                let ptr = slice.as_ptr().add(pos).cast();
                let ptr = x86_64::_mm_loadu_si128(ptr);

                let cmp_a = x86_64::_mm_cmpgt_epi8(ptr, matches_range.0);
                let cmp_b = x86_64::_mm_cmpgt_epi8(matches_range.1, ptr);
                let cmp = x86_64::_mm_and_si128(cmp_a, cmp_b);
                let mask = x86_64::_mm_movemask_epi8(cmp);

                if mask != (u16::MAX as i32) {
                    let trailing_ones = mask.trailing_ones();
                    pos += trailing_ones as usize;
                    break;
                } else {
                    pos += Self::LENGTH;
                }
            }
        }

        if start_pos == pos {
            end_pos = pos;
        } else {
            end_pos = pos - 1;
        }

        (start_pos, end_pos)
    }

    fn longest_consecutive_matching<const N: usize>(slice: &[u8], matches: [u8; N], start_pos: usize) -> (usize, usize) {
        let mut end_pos = start_pos;
        let mut pos = start_pos;
        let len = slice.len();

        unsafe {
            let match_lanes = matches.map(|m| {
                x86_64::_mm_set1_epi8(m.cast_signed())
            });

            while pos + Self::LENGTH < len {
                let ptr = slice.as_ptr().add(pos).cast();
                let ptr = x86_64::_mm_loadu_si128(ptr);

                let cmp = match_lanes.iter().fold(x86_64::_mm_set1_epi8(0), |prev, &lane| {
                    let cmp = x86_64::_mm_cmpeq_epi8(ptr, lane);
                    x86_64::_mm_or_si128(prev, cmp)
                });

                let mask = x86_64::_mm_movemask_epi8(cmp);

                if mask != (u16::MAX as i32) {
                    let trailing_ones = mask.trailing_ones();
                    pos += trailing_ones as usize;
                    end_pos = pos - 1;
                    break;
                } else {
                    pos += Self::LENGTH;
                }
            }
        }

        if start_pos == pos {
            end_pos = pos;
        } else {
            end_pos = pos - 1;
        }

        (start_pos, end_pos)
    }
}

mod test {
    use crate::simd_common::{Sse, SimdTrait};


    #[test]
    fn sse_test1() {
        let slice = b"1234567890qwertyuiopasdfghjklzxcvbnm";
        let (start, end) = Sse::find_consecutive_in_range(slice, (b'0', b'9'), 0);
        assert_eq!(start, 0);
        assert_eq!(end, 9);

        let slice = b"12345678901234567890123456789012334567890";
        let (start, end) = Sse::find_consecutive_in_range(slice, (b'0', b'9'), 0);
        assert_eq!(start, 0);
        assert_eq!(end, 31);

        let slice = b"bqwertyuiopasdfghjklzxcvbnm";
        let (start, end) = Sse::find_consecutive_in_range(slice, (b'0', b'9'), 0);
        assert_eq!(start, 0);
        assert_eq!(end, 0);

        let slice = b"b023q2w142e245rtyuiopasdfghjklzxcvbnm";
        let (start, end) = Sse::find_consecutive_in_range(slice, (b'0', b'9'), 0);
        assert_eq!(start, 0);
        assert_eq!(end, 0);
    }

    #[test]
    fn sse_test2() {
        let slice = b"qwertyuiopasdfghjklzxcvbnm1234567890";
        let (start, end) = Sse::longest_consecutive_matching(slice, [b'q', b'w', b'e'], 0);
        assert_eq!(start, 0);
        assert_eq!(end, 2);

        let slice = b"aaaaaaaaaaaaaaaaaaaaaaaaabbbbbbbbbbbbbbaaaaaaaaaaaaaaaaa";
        let (start, end) = Sse::longest_consecutive_matching(slice, [b'a', b'b', b'c'], 0);
        assert_eq!(start, 0);
        assert_eq!(end, 47);

        let slice = b"qwretyuiopasdfghjklzxcvbnm1234567890";
        let (start, end) = Sse::longest_consecutive_matching(slice, [b'q', b'w', b'e'], 0);
        assert_eq!(start, 0);
        assert_eq!(end, 1);
    }
}