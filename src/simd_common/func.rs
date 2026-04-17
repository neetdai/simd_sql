use super::{SimdTrait, Avx2, Sse};

pub(crate) fn find_consecutive_in_range(slice: &[u8], matches: (u8, u8), start_pos: usize) -> (usize, usize) {
    cfg_select! {
        target_arch = "x86" => {
            find_consecutive_in_range_basic_x86(slice, matches, start_pos)
        }
        target_arch = "x86_64" => {
            find_consecutive_in_range_basic_x86(slice, matches, start_pos)
        }
        _ => {
            find_consecutive_in_range_basic(slice, matches, start_pos)
        }
    }
}

pub(crate) fn longest_consecutive_matching<const N: usize>(slice: &[u8], matches: [u8; N], start_pos: usize) -> (usize, usize) {
    cfg_select! {
        target_arch = "x86" => {
            longest_consecutive_matching_basic_x86(slice, matches, start_pos)
        }
        target_arch = "x86_64" => {
            longest_consecutive_matching_basic_x86(slice, matches, start_pos)
        }
        _ => {
            longest_consecutive_matching_basic(slice, matches, start_pos)
        }
    }
}

fn find_consecutive_in_range_basic(slice: &[u8], matches: (u8, u8), start_pos: usize) -> (usize, usize) {
    let len = slice.len();
    let mut end_pos = start_pos;

    for index in start_pos..len {
        let c = &slice[index];
        if *c >= matches.0 && *c <= matches.1 {
            end_pos = index;
        } else {
            break;
        }
    }
    (start_pos, end_pos)
}

fn find_consecutive_in_range_basic_x86(slice: &[u8], matches: (u8, u8), start_pos: usize) -> (usize, usize) {
    let mut end_pos = start_pos;
    let len = slice.len();
    if is_x86_feature_detected!("avx2") && len >= Avx2::LENGTH {
        (_, end_pos) = Avx2::find_consecutive_in_range(slice, matches, end_pos);
    }

    if is_x86_feature_detected!("sse2") && len >= Sse::LENGTH {
        (_, end_pos) = Sse::find_consecutive_in_range(slice, matches, end_pos);
    }

    (_, end_pos) = find_consecutive_in_range_basic(slice, matches, end_pos);
    (start_pos, end_pos)
}

fn longest_consecutive_matching_basic<const N: usize>(slice: &[u8], matches: [u8; N], start_pos: usize) -> (usize, usize) {
    let len = slice.len();
    let mut end_pos = start_pos;

    for index in start_pos..len {
        let c = &slice[index];
        if matches.contains(c) {
            end_pos = index;
        } else {
            break;
        }
    }
    (start_pos, end_pos)
}

fn longest_consecutive_matching_basic_x86<const N: usize>(slice: &[u8], matches: [u8; N], start_pos: usize) -> (usize, usize) {
    let mut end_pos = start_pos;
    let len = slice.len();
    if is_x86_feature_detected!("avx2") && len >= Avx2::LENGTH {
        (_, end_pos) = Avx2::longest_consecutive_matching(slice, matches, end_pos);
    }

    if is_x86_feature_detected!("sse2") && len >= Sse::LENGTH {
        (_, end_pos) = Sse::longest_consecutive_matching(slice, matches, end_pos);
    }

    (_, end_pos) = longest_consecutive_matching_basic(slice, matches, end_pos);
    (start_pos, end_pos)
}

mod test {
    use crate::simd_common::func::longest_consecutive_matching_basic;

    use super::find_consecutive_in_range_basic;

    #[test]
    fn test_find_consecutive_in_range_basic() {
        let slice = b"123456789";
        let (start, end) = find_consecutive_in_range_basic(slice, (b'0', b'9'), 0);
        assert_eq!(start, 0);
        assert_eq!(end, 8);
    }

    #[test]
    fn test_longest_consecutive_matching_basic() {
        let slice = b"abcde12345fghij67890";
        let matches = [b'a', b'b', b'c', b'd', b'e'];
        let (start, end) = longest_consecutive_matching_basic(slice, matches, 0);
        assert_eq!(start, 0);
        assert_eq!(end, 4);
    }
}