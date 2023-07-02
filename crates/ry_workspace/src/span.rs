//! Defines a [`Span`] for working with source text locations and some utilities.

use codespan_reporting::diagnostic::Label;
use std::{fmt::Display, ops::Range};

use crate::workspace::FileID;

/// Represents location in the source text.
#[derive(Copy, Clone, Hash, Debug, Default, PartialEq, Eq)]
pub struct Span {
    /// Offset of starting byte in the source text.
    start: usize,
    /// Offset of ending byte in the source text.
    end: usize,
    /// Id of the file containing the span.
    file_id: FileID,
}

/// Dummy span - span that is used as a placeholder in tests.
///
/// Note: using dummy span in code except in tests is not recommended,
/// because this can result in undefined behavior with diagnostics and
/// debug information.
pub const DUMMY_SPAN: Span = Span::new(0, 0, 0);

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}..{}", self.start, self.end))
    }
}

impl Span {
    /// Creates a new span.
    #[inline]
    #[must_use]
    pub const fn new(start: usize, end: usize, file_id: FileID) -> Self {
        Self {
            start,
            end,
            file_id,
        }
    }

    /// Returns the offset of starting byte in the source text.
    #[inline]
    #[must_use]
    pub const fn start(&self) -> usize {
        self.start
    }

    /// Returns the offset of ending byte in the source text.
    #[inline]
    #[must_use]
    pub const fn end(&self) -> usize {
        self.end
    }

    /// Returns the id of the file containing the span.
    #[inline]
    #[must_use]
    pub const fn file_id(&self) -> FileID {
        self.file_id
    }

    /// Gets primary diagnostics label ([`Label<FileID>`] from [`codespan_reporting`])
    /// in the span.
    #[inline]
    #[must_use]
    pub fn to_primary_label(self) -> Label<FileID> {
        Label::primary(self.file_id(), self)
    }

    /// Gets secondary diagnostics label ([`Label<FileID>`] from [`codespan_reporting`])
    /// in the span.
    #[inline]
    #[must_use]
    pub fn to_secondary_label(self) -> Label<FileID> {
        Label::secondary(self.file_id(), self)
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
    /// # Example:
    /// ```
    /// # use ry_workspace::span::{Span, SpanIndex};
    /// let span = Span::new(0, 3, 1);
    /// assert_eq!("test".index(span), "tes");
    /// ```
    ///
    /// **Note**: use [`crate::workspace::Workspace::resolve_span()`] and
    /// [`crate::workspace::Workspace::resolve_span_or_panic()`] instead.
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
