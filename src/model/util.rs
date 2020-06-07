use std::ops::{Bound, RangeBounds};

pub fn from<R: RangeBounds<usize>>(range: &R) -> usize {
    match range.start_bound() {
        Bound::Unbounded => 0,
        Bound::Included(x) => *x,
        Bound::Excluded(x) => x + 1,
    }
}

pub fn to<R: RangeBounds<usize>>(range: &R, max: usize) -> usize {
    match range.end_bound() {
        Bound::Unbounded => max,
        Bound::Included(x) => x + 1,
        Bound::Excluded(x) => *x,
    }
}

pub fn split_at_utf16(text: &str, mut index: usize) -> (&str, &str) {
    let mut iter = text.chars();
    while index > 0 {
        if let Some(c) = iter.next() {
            let l = c.len_utf16();
            if l > index {
                panic!("Can't split in the middle of a character")
            } else {
                index -= l;
            }
        } else {
            return (text, "");
        }
    }
    let mid = text.len() - iter.as_str().len();
    text.split_at(mid)
}
