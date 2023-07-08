//! Defines a [`Span`] for working with source text locations and some utilities.

use codespan_reporting::diagnostic::Label;
use std::{fmt::Display, ops::Range};

/// Represents location in the source text.
///
/// Implements [`Copy`], when [`Range<usize>`] does not.
#[derive(Copy, Clone, Hash, Debug, Default, PartialEq, Eq)]
pub struct Span {
    /// Offset of starting byte in the source text.
    pub start: usize,
    /// Offset of ending byte in the source text.
    pub end: usize,
}

/// Dummy span - span that is used as a placeholder in tests.
///
/// # Note
/// Using dummy span in code except in tests is not recommended,
/// because this can result in undefined behavior with diagnostics and
/// debug information, because firstly diagnostics cannot be emitted correctly
/// when start and end positions are equal
pub const DUMMY_SPAN: Span = Span { start: 0, end: 0 };

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}..{}", self.start, self.end))
    }
}

impl Span {
    /// Gets primary diagnostics label ([`Label`] from [`codespan_reporting`])
    /// in the span.
    #[inline]
    #[must_use]
    pub fn to_primary_label(self) -> Label<()> {
        Label::primary((), self)
    }

    /// Gets secondary diagnostics label ([`Label`] from [`codespan_reporting`])
    /// in the span.
    #[inline]
    #[must_use]
    pub fn to_secondary_label(self) -> Label<()> {
        Label::secondary((), self)
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

/// For internal usage only! Used to index a string using a given span.
pub trait SpanIndex {
    /// Output of the indexing operation.
    type Output: ?Sized;

    /// Index a string using a given span.
    ///
    /// # Example
    /// ```
    /// # use ry_filesystem::span::{Span, SpanIndex};
    /// let span = Span { start: 0, end: 3 };
    /// assert_eq!("test".index(span), "tes");
    /// ```
    fn index(&self, span: Span) -> &Self::Output;
}

impl<T> SpanIndex for T
where
    T: AsRef<str>,
{
    type Output = str;

    #[inline]
    #[allow(clippy::indexing_slicing)]
    fn index(&self, span: Span) -> &Self::Output {
        &self.as_ref()[span.start..span.end]
    }
}
