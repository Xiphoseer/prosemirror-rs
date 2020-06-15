//! # Generic utilities

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

pub(crate) fn then_some<T>(b: bool, v: T) -> Option<T> {
    if b {
        Some(v)
    } else {
        None
    }
}
