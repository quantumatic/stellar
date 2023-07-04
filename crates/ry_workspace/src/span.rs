//! Defines a [`Span`] for working with source text locations and some utilities.

use codespan_reporting::diagnostic::Label;
use std::{fmt::Display, ops::Range};

use crate::workspace::FileID;

/// Represents location in the source text.
#[derive(Copy, Clone, Hash, Debug, Default, PartialEq, Eq)]
pub struct Span {
    /// Offset of starting byte in the source text.
    pub start: usize,
    /// Offset of ending byte in the source text.
    pub end: usize,
    /// Id of the file containing the span.
    pub file_id: FileID,
}

/// Dummy span - span that is used as a placeholder in tests.
///
/// # Note
/// Using dummy span in code except in tests is not recommended,
/// because this can result in undefined behavior with diagnostics and
/// debug information, because firstly diagnostics cannot be emitted correctly
/// when start and end positions are equal, and secondly `file_id` is always starting
/// from `1` in the [`Workspace`] (see [`add_file`] for more details).
///
/// [`file_id`]: crate::span::Span::file_id
/// [`Workspace`]: crate::workspace::Workspace
/// [`add_file`]: crate::workspace::Workspace::add_file
pub const DUMMY_SPAN: Span = Span {
    start: 0,
    end: 0,
    file_id: 0,
};

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}..{}", self.start, self.end))
    }
}

impl Span {
    /// Gets primary diagnostics label ([`Label<FileID>`] from [`codespan_reporting`])
    /// in the span.
    #[inline]
    #[must_use]
    pub fn to_primary_label(self) -> Label<FileID> {
        Label::primary(self.file_id, self)
    }

    /// Gets secondary diagnostics label ([`Label<FileID>`] from [`codespan_reporting`])
    /// in the span.
    #[inline]
    #[must_use]
    pub fn to_secondary_label(self) -> Label<FileID> {
        Label::secondary(self.file_id, self)
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

/// For internal usage only! Used to index a string using a given span.
pub trait SpanIndex {
    /// Output of the indexing operation.
    type Output: ?Sized;

    /// Index a string using a given span (ignoring [`Span::file_id`]).
    ///
    /// # Example
    /// ```
    /// # use ry_workspace::span::{Span, SpanIndex};
    /// let span = Span { start: 0, end: 3, file_id: 1 };
    /// assert_eq!("test".index(span), "tes");
    /// ```
    ///
    /// **Note**: use [`resolve_span()`] and [`resolve_span_or_panic()`] instead.
    ///
    /// [`resolve_span()`]: crate::workspace::Workspace::resolve_span
    /// [`resolve_span_or_panic()`]: crate::workspace::Workspace::resolve_span_or_panic
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
