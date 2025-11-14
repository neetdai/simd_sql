use std::cmp::Ordering;

use strum::VariantArray;

use crate::keyword::Keyword;



#[derive(Debug)]
pub(crate) struct KeywordMatcher {
    short_matcher: ShortKeywordMatcher,
}

impl KeywordMatcher {
    pub(crate) fn new() -> Self {
        let keyword_list = Keyword::VARIANTS.sort_by(|k1, k2| {
            let k1_str = k1.as_str();
            let k2_str = k2.as_str();
            
            let k1_len = k1_str.len();
            let k2_len = k2_str.len();

            if k1_len < k2_len {
                Ordering::Less
            } else if k1_len > k2_len {
                Ordering::Greater
            } else {
                k1_str.cmp(k2_str)
            }
        });

        
    }
}

#[derive(Debug)]
struct ShortKeywordMatcher {

}