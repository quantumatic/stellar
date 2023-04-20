//! `location.rs` - Defines the Span struct for storing source
//! Locations throughout the compiler. Most notably, these locations
//! are passed around throughout the parser and are stored in each
//! AST node.
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Range};

/// Represents code block location in source text.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}..{}", self.start, self.end))
    }
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
    fn index(self, span: Span) -> &'a str {
        &self[span.start..span.end]
    }
}

/// Represents thing located in some [`Span`].
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    #[inline]
    pub const fn new(inner: T, span: Span) -> Self {
        Self { inner, span }
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
