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

/// A type the holds a value of A or B or both.
pub enum EitherOrBoth<A, B> {
    /// Both values
    Both(A, B),
    /// Just a value of type A
    Left(A),
    /// Just a value of type B
    Right(B),
}

impl<A, B> EitherOrBoth<A, B> {
    /// Get the left value if present
    pub fn left(&self) -> Option<&A> {
        match self {
            Self::Both(a, _) => Some(a),
            Self::Left(a) => Some(a),
            Self::Right(_) => None,
        }
    }

    /// Get the right value if present
    pub fn right(&self) -> Option<&B> {
        match self {
            Self::Both(_, b) => Some(b),
            Self::Left(_) => None,
            Self::Right(b) => Some(b),
        }
    }
}

impl<T> EitherOrBoth<T, T> {
    /// Get the right value if present, and the left otherwise
    pub fn right_or_left(&self) -> &T {
        match self {
            Self::Left(a) => a,
            Self::Right(b) => b,
            Self::Both(_a, b) => b,
        }
    }
}