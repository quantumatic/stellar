//! Defines a [`Span`] for working with source text locations and some utilities.

use codespan_reporting::diagnostic::Label;
use std::{fmt::Display, ops::Range};

/// Represents location in the source text.
#[derive(Copy, Clone, Hash, Debug, Default, PartialEq, Eq)]
pub struct Span {
    /// Offset of starting byte in the source text.
    start: usize,
    /// Offset of ending byte in the source text.
    end: usize,
    /// Id of the file containing the span.
    file_id: usize,
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}..{}", self.start, self.end))
    }
}

impl Span {
    /// Creates a new span.
    #[inline]
    #[must_use]
    pub const fn new(start: usize, end: usize, file_id: usize) -> Self {
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
    pub const fn file_id(&self) -> usize {
        self.file_id
    }

    /// Gets primary diagnostics label ([`Label<usize>`] from [`codespan_reporting`])
    /// in the span.
    #[inline]
    #[must_use]
    pub fn to_primary_label(self) -> Label<usize> {
        Label::primary(self.file_id(), self)
    }

    /// Gets secondary diagnostics label ([`Label<usize>`] from [`codespan_reporting`])
    /// in the span.
    #[inline]
    #[must_use]
    pub fn to_secondary_label(self) -> Label<usize> {
        Label::secondary(self.file_id(), self)
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
    /// use ry_source_file::span::{Span, SpanIndex};
    ///
    /// let span = Span::new(0, 3, 0);
    /// assert_eq!("test".index(span), "tes");
    /// ```
    ///
    /// **Note**: use [`SourceFileManager::resolve_span()`] and
    /// [`SourceFileManager::optionally_resolve_span()`] instead.
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

/// Represents some value that has associated span ([`Span`]) with it.
#[derive(Debug, PartialEq, Clone, Default, Eq, Hash)]
pub struct Spanned<T> {
    /// Inner content.
    inner: T,
    /// Span.
    span: Span,
}

impl<T> Spanned<T> {
    /// Constructs a new [`Spanned`] object with a given content
    /// and span.
    ///
    /// > It is recommended to use [`At::at`] instead.
    #[inline]
    #[must_use]
    pub const fn new(inner: T, span: Span) -> Self {
        Self { inner, span }
    }

    /// Returns the span of this [`Spanned`] object.
    #[inline]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the immutable reference to inner content of this [`Spanned`] object.
    #[inline]
    pub const fn unwrap(&self) -> &T {
        &self.inner
    }

    /// Returns the owned inner content of this [`Spanned`] object.
    #[inline]
    #[allow(clippy::missing_const_for_fn)] // clippy issue
    pub fn take(self) -> T {
        self.inner
    }
}

impl From<Span> for Range<usize> {
    fn from(value: Span) -> Self {
        value.start..value.end
    }
}

/// Used to construct `Spanned` object.
///
/// See the documentation for [`At::at()`] for more informatiohn.
pub trait At {
    /// Used to construct `Spanned` object.
    ///
    /// # Example:
    /// ```
    /// use ry_source_file::span::{At, Span};
    ///
    /// let my_file_id = 0;
    ///
    /// let first_three = 3_i32.at(Span::new(0, 1, my_file_id));
    /// let second_three = 3_i32.at(Span::new(1, 2, my_file_id));
    ///
    /// assert_eq!(first_three.unwrap(), &3);
    /// assert_eq!(second_three.unwrap(), &3);
    /// assert_eq!(
    ///     first_three.span(),
    ///     Span::new(0, 1, my_file_id)
    /// );
    /// assert_eq!(
    ///     second_three.span(),
    ///     Span::new(1, 2, my_file_id),
    /// );
    /// ```
    #[inline]
    fn at(self, span: Span) -> Spanned<Self>
    where
        Self: Sized,
    {
        Spanned::new(self, span)
    }
}

impl<T: Sized> At for T {}
