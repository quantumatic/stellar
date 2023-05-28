//! `location.rs` - Defines the Span struct for storing source
//! Locations throughout the compiler. Most notably, these locations
//! are passed around throughout the parser and are stored in each
//! AST node.
use codespan_reporting::diagnostic::Label;
use std::{fmt::Display, ops::Range};

/// Represents code block location in source text.
#[derive(Copy, Clone, Hash, Debug, Default, PartialEq, Eq)]
pub struct Span {
    start: usize,
    end: usize,
    file_id: usize,
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}..{}", self.start, self.end))
    }
}

impl Span {
    #[inline]
    #[must_use]
    pub const fn new(start: usize, end: usize, file_id: usize) -> Self {
        Self {
            start,
            end,
            file_id,
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

    #[inline]
    pub const fn file_id(&self) -> usize {
        self.file_id
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
#[derive(Debug, PartialEq, Clone, Default, Eq, Hash)]
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

impl From<Span> for Range<usize> {
    fn from(value: Span) -> Self {
        value.start..value.end
    }
}

pub trait At {
    #[inline]
    fn at(self, span: Span) -> Spanned<Self>
    where
        Self: Sized,
    {
        Spanned::new(self, span)
    }
}

impl<T: Sized> At for T {}

pub fn make_primary_label(span: Span) -> Label<usize> {
    Label::primary(span.file_id(), span)
}

pub fn make_secondary_label(span: Span) -> Label<usize> {
    Label::secondary(span.file_id(), span)
}
