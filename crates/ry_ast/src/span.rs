//! `location.rs` - Defines the Span struct for storing source
//! Locations throughout the compiler. Most notably, these locations
//! are passed around throughout the parser and are stored in each
//! AST node.
use std::ops::Range;

use derive_more::Display;

/// Represents code block location in source text.
#[derive(Clone, Debug, PartialEq, Default, Copy, Display, Eq)]
#[display(fmt = "{}..{}", start, end)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    #[inline]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn from_location(location: usize, character_len: usize) -> Self {
        Self {
            start: location,
            end: location + character_len,
        }
    }

    #[inline]
    pub const fn start(&self) -> usize {
        self.start
    }

    #[inline]
    pub const fn end(&self) -> usize {
        self.end
    }
}

impl From<Range<usize>> for Span {
    fn from(val: Range<usize>) -> Self {
        Self::new(val.start, val.end)
    }
}

/// Represents thing located in some [`Span`].
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Spanned<T> {
    value: T,
    span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }

    #[inline]
    pub const fn unwrap(&self) -> &T {
        &self.value
    }

    #[inline]
    pub const fn span(&self) -> Span {
        self.span
    }
}

impl<T> From<(T, Span)> for Spanned<T> {
    fn from(val: (T, Span)) -> Self {
        Spanned::new(val.0, val.1)
    }
}

impl From<Span> for Range<usize> {
    fn from(value: Span) -> Self {
        value.start..value.end
    }
}
pub trait WithSpan {
    fn with_span(self, span: impl Into<Span>) -> Spanned<Self>
    where
        Self: Sized,
    {
        Spanned::new(self, span.into())
    }
}

impl<T: Sized> WithSpan for T {}
