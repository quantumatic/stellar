//! Defines the [`Span`] struct for representing
//! locations in the source code throughout the compiler.
//! Most notably, these locations are passed around throughout the parser
//! and are stored in each AST node via [`Spanned`] struct.
use codespan_reporting::{diagnostic::Label, files::SimpleFiles};
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
    pub const fn start(&self) -> usize {
        self.start
    }

    /// Returns the offset of ending byte in the source text.
    #[inline]
    pub const fn end(&self) -> usize {
        self.end
    }

    /// Returns the id of the file containing the span.
    #[inline]
    pub const fn file_id(&self) -> usize {
        self.file_id
    }

    /// Gets primary diagnostics label ([`Label<usize>`] from [`codespan_reporting`])
    /// in the span.
    pub fn to_primary_label(self) -> Label<usize> {
        Label::primary(self.file_id(), self)
    }

    /// Gets secondary diagnostics label ([`Label<usize>`] from [`codespan_reporting`])
    /// in the span.
    pub fn to_secondary_label(self) -> Label<usize> {
        Label::secondary(self.file_id(), self)
    }

    /// Returns the content of the part of the source code situated
    /// at the given span if it is valid.
    ///
    /// # Panics
    /// - If the span is out of bounds ([`Span::start`] and [`Span::end`]).
    /// - If the file with the given [`Span::file_id`] does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use codespan_reporting::files::SimpleFiles;
    /// use ry_span::Span;
    ///
    /// let mut files = SimpleFiles::new();
    /// let file_id = files.add("test.ry", "fun main() { println(\"Hello, world!\"); }");
    /// let span = Span::new(21, 36, file_id);
    /// assert_eq!(span.get_corresponding_contents(&files), "\"Hello, world!\"");
    /// ```
    pub fn get_corresponding_contents<'a>(self, files: &SimpleFiles<&str, &'a str>) -> &'a str {
        let file = files.get(self.file_id).unwrap();
        let source = file.source();
        source.get(self.start..self.end).unwrap()
    }

    /// Returns the content of the part of the source code situated
    /// at the given span.
    ///
    /// Instead of panicking in the situation when [`Span::get_corresponding_contents()`] does,
    /// the function returns [`None`]. In all other cases
    /// `Some(Span::get_corresponding_contents(...))`.
    ///
    /// # Examples
    ///
    /// ```
    /// use codespan_reporting::files::SimpleFiles;
    /// use ry_span::Span;
    ///
    /// let mut files = SimpleFiles::new();
    /// let invalid_span = Span::new(99, 100, 0);
    /// let content = invalid_span.optionally_get_corresponding_contents(&files);
    /// assert_eq!(content, None);
    pub fn optionally_get_corresponding_contents<'a>(
        self,
        files: &SimpleFiles<&str, &'a str>,
    ) -> Option<&'a str> {
        let Ok(file) = files.get(self.file_id) else {
            return None;
        };

        let source = file.source();
        source.get(self.start..self.end)
    }
}

/// For internal usage only! Used to index a string using a given span.
pub trait SpanIndex {
    type Output: ?Sized;

    /// Index a string using a given span (ignoring [`Span::file_id`]).
    ///
    /// # Example:
    /// ```
    /// use ry_span::{Span, SpanIndex};
    ///
    /// let span = Span::new(0, 3, 0);
    /// assert_eq!("test".index(span), "tes");
    /// ```
    ///
    /// > Use [`Span::get_corresponding_contents()`] and
    /// [`Span::optionally_get_corresponding_contents()`] instead.
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

/// Represents some value that has associated span ([`Span`]).
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
/// See the documentation for [`at`] for more informatiohn.
pub trait At {
    /// Used to construct `Spanned` object.
    ///
    /// # Example:
    /// ```
    /// use ry_span::{At, Span};
    ///
    /// let my_file_id = 0;
    ///
    /// let first_three = 3.at(Span::new(0, 1, my_file_id));
    /// let second_three = 3.at(Span::new(1, 2, my_file_id));
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
