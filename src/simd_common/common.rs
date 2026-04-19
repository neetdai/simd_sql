
pub(crate) trait SimdTrait {
    const LENGTH: usize;

    // 扫描元素，找出连续符合范围的元素下标，返回起始下标和结束下标
    fn find_consecutive_in_range(slice: &[u8], matches: (u8, u8), start_pos: usize) -> (usize, usize);

    // 扫描元素, 找出连续符合匹配的元素下标，返回起始下标和结束下标
    fn longest_consecutive_matching<const N: usize>(slice: &[u8], matches: [u8; N], start_pos: usize) -> (usize, usize);

    // 混合匹配,可以按多种范围和特定匹配式进行匹配，返回起始下标和结束下标
    fn mixed_match<const N1: usize, const N2: usize>(slice: &[u8], match_range: [(u8, u8); N1], matches2: [u8; N2], start_pos: usize) -> (usize, usize);
}