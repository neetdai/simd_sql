/// 检查 position 处的字节是否被奇数个连续反斜杠转义。
pub(crate) fn is_escaped(slice: &[u8], pos: usize, start_pos: usize) -> bool {
    let mut count = 0u32;
    let mut p = pos;
    while p > start_pos && slice[p - 1] == b'\\' {
        count += 1;
        p -= 1;
    }
    count & 1 == 1
}

pub(crate) trait SimdTrait {
    const LENGTH: usize;

    fn find_consecutive_in_range(
        slice: &[u8],
        matches: (u8, u8),
        start_pos: usize,
    ) -> (usize, isize);

    fn longest_consecutive_matching<const N: usize>(
        slice: &[u8],
        matches: [u8; N],
        start_pos: usize,
    ) -> (usize, isize);

    fn mixed_match<const N1: usize, const N2: usize>(
        slice: &[u8],
        match_range: [(u8, u8); N1],
        matches2: [u8; N2],
        start_pos: usize,
    ) -> (usize, isize);

    /// 跳过非匹配字节，找到首个匹配集合中任一字节的位置。
    /// 返回 `(start, match_pos)` 或 `(start, -1)`。
    fn skip_until_match<const N: usize>(
        slice: &[u8],
        matches: [u8; N],
        start_pos: usize,
    ) -> (usize, isize);

    /// 跳过字节直到出现完整连续序列。前进步长带 (N-1) 重叠防止跨边界遗漏。
    /// 返回 `(start, 序列起始位置)` 或 `(start, -1)`。
    fn skip_until_sequence<const N: usize>(
        slice: &[u8],
        sequence: [u8; N],
        start_pos: usize,
    ) -> (usize, isize);
}
