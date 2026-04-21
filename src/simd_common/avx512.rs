use std::arch::x86_64::{self};

use super::common::SimdTrait;

pub(crate) struct Avx512;

impl SimdTrait for Avx512 {
    const LENGTH: usize = 64;

    fn find_consecutive_in_range(
        slice: &[u8],
        matches: (u8, u8),
        start_pos: usize,
    ) -> (usize, isize) {
        let mut end_pos = -1;
        let mut pos = start_pos;
        let len = slice.len();
        unsafe {
            let matches_range = {
                let a_lane = x86_64::_mm512_set1_epi8(matches.0.cast_signed());
                let b_lane = x86_64::_mm512_set1_epi8(matches.1.cast_signed());
                (a_lane, b_lane)
            };

            while pos + Self::LENGTH <= len {
                let ptr = slice.as_ptr().add(pos).cast();
                let ptr = x86_64::_mm512_loadu_epi8(ptr);

                let cmp_a = x86_64::_mm512_cmpge_epi8_mask(ptr, matches_range.0);
                let cmp_b = x86_64::_mm512_cmpeq_epi8_mask(ptr, matches_range.1);
                let mask = cmp_a | cmp_b;
                if mask != 0 {
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

    fn longest_consecutive_matching<const N: usize>(
        slice: &[u8],
        matches: [u8; N],
        start_pos: usize,
    ) -> (usize, isize) {
        let mut end_pos = -1;
        let mut pos = start_pos;
        let len = slice.len();

        unsafe {
            let match_lanes = matches.map(|m| x86_64::_mm512_set1_epi8(m.cast_signed()));

            while pos + Self::LENGTH < len {
                let ptr = slice.as_ptr().add(pos).cast();
                let ptr = x86_64::_mm512_loadu_epi8(ptr);

                let mask = match_lanes.iter().fold(0, |prev, &lane| {
                    let cmp = x86_64::_mm512_cmpeq_epi8_mask(ptr, lane);
                    cmp | prev
                });

                if mask != 0 {
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

    fn mixed_match<const N1: usize, const N2: usize>(
        slice: &[u8],
        match_range: [(u8, u8); N1],
        matches2: [u8; N2],
        start_pos: usize,
    ) -> (usize, isize) {
        todo!()
    }
}

// mod test {
//     use super::{SimdTrait, Avx512};

//     #[test]
//     fn avx512_test1() {
//         let slice = b"1234567890qwertyuiopasdfghjklzxcvbnm1234567890qwertyuiopasdfghjklzxcvbnm";
//         let (start, end) = Avx512::find_consecutive_in_range(slice, (b'0', b'9'), 0);
//         assert_eq!(start, 0);
//         assert_eq!(end, 9);
//     }
// }
