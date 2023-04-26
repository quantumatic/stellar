//! `location.rs` - Defines the Span struct for storing source
//! Locations throughout the compiler. Most notably, these locations
//! are passed around throughout the parser and are stored in each
//! AST node.
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Range};

/// Represents code block location in source text.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}..{}", self.start, self.end))
    }
}

impl Span {
    #[inline]
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[inline]
    #[must_use]
    pub const fn from_location(location: usize, character_len: usize) -> Self {
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
    #[inline]
    fn from(val: Range<usize>) -> Self {
        Self::new(val.start, val.end)
    }
}

pub trait SpanIndex {
    fn index(self, span: Span) -> Self;
}

impl<'a> SpanIndex for &'a str {
    #[inline]
    #[allow(clippy::indexing_slicing)]
    fn index(self, span: Span) -> &'a str {
        &self[span.start..span.end]
    }
}

/// Represents thing located in some [`Span`].
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct Spanned<T> {
    inner: T,
    span: Span,
}

impl<T> Spanned<T> {
    #[inline]
    #[must_use]
    pub const fn new(inner: T, span: Span) -> Self {
        Self { inner, span }
    }

    #[inline]
    pub const fn span(&self) -> Span {
        self.span
    }

    #[inline]
    pub const fn unwrap(&self) -> &T {
        &self.inner
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

pub trait At {
    #[inline]
    fn at(self, span: impl Into<Span>) -> Spanned<Self>
    where
        Self: Sized,
    {
        Spanned::new(self, span.into())
    }
}

impl<T: Sized> At for T {}
