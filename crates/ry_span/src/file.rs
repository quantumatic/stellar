//! Defines a [`InMemoryFile`] to represent a Ry source file and provides some utilities.

use crate::span::{Span, SpanIndex};
use codespan_reporting::files::{Error, Files};
use std::cmp::Ordering;
use std::ops::Range;
use std::path::Path;

/// A Ry source file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InMemoryFile<'storage> {
    /// The path of the source file.
    pub path: &'storage Path,

    /// The path of the source file as a string slice.
    pub path_str: &'storage str,

    /// The source content of the file.
    pub source: &'storage str,

    /// The length of the source content (in bytes).
    pub source_len: usize,

    /// The array of line starting byte indices in the [`InMemoryFile::source`].
    pub line_starts: Vec<usize>,
}

impl<'storage> InMemoryFile<'storage> {
    /// Creates a new [`InMemoryFile`].
    #[inline]
    #[must_use]
    pub fn new(path: &'storage Path, source: &'storage str) -> Self {
        Self {
            path,
            path_str: path.to_str().expect("Invalid UTF-8 data in path"),
            source,
            source_len: source.len(),
            line_starts: std::iter::once(0)
                .chain(source.match_indices('\n').map(|(i, _)| i + 1))
                .collect(),
        }
    }

    /// Returns the string slice corresponding to the given location.
    #[inline]
    #[must_use]
    pub fn resolve_span(&self, span: Span) -> &str {
        self.source.index(span)
    }

    /// Returns the line starting byte index of the given line index.
    ///
    /// # Panics
    /// Panics if the given line index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use std::path::Path;
    /// # use ry_span::file::InMemoryFile;
    /// let file = InMemoryFile::new(
    ///     Path::new("test.ry"),
    ///     "fun main() {
    ///     println(\"Hello, world!\");
    /// }",
    /// );
    ///
    /// assert_eq!(file.get_line_start_by_index(0), 0);
    /// assert_eq!(file.get_line_start_by_index(1), 13);
    /// ```
    pub(crate) fn get_line_start_by_index(&self, line_index: usize) -> Result<usize, Error> {
        match line_index.cmp(&self.line_starts.len()) {
            Ordering::Less => Ok(self
                .line_starts
                .get(line_index)
                .copied()
                .expect("failed despite previous check")),
            Ordering::Equal => Ok(self.source.len()),
            Ordering::Greater => Err(Error::LineTooLarge {
                given: line_index,
                max: self.line_starts.len() - 1,
            }),
        }
    }

    /// Returns the line starting byte index of the given byte index.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::path::Path;
    /// # use ry_span::file::InMemoryFile;
    /// let file = InMemoryFile::new(
    ///     Path::new("test.ry"),
    ///     "fun main() {
    ///     println(\"Hello, world!\");
    /// }",
    /// );
    ///
    /// assert_eq!(file.get_line_index_by_byte_index(0), 0);
    /// assert_eq!(file.get_line_index_by_byte_index(13), 1);
    /// ```
    #[must_use]
    pub fn get_line_index_by_byte_index(&self, byte_index: usize) -> usize {
        self.line_starts
            .binary_search(&byte_index)
            .unwrap_or_else(|next_line| next_line - 1)
    }

    /// Returns the line range of the given line index.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::path::Path;
    /// # use ry_span::file::InMemoryFile;
    /// let file = InMemoryFile::new(
    ///     Path::new("test.ry"),
    ///     "fun main() {
    ///     println(\"Hello, world!\");
    /// }",
    /// );
    ///
    /// assert_eq!(file.line_range_by_index(0), Some(0..13));
    /// assert_eq!(file.line_range_by_index(1), Some(13..43));
    /// assert_eq!(file.line_range_by_index(2), Some(43..44));
    /// ```
    #[must_use]
    pub fn line_range_by_index(&self, line_index: usize) -> Option<Range<usize>> {
        let current_line_start = self.get_line_start_by_index(line_index).ok()?;
        let next_line_start = self.get_line_start_by_index(line_index + 1).ok()?;

        Some(current_line_start..next_line_start)
    }
}

// For proper error reporting
impl<'storage> Files<'storage> for InMemoryFile<'storage> {
    // we don't care about file IDs, because we have only one individual file here
    type FileId = ();

    type Name = &'storage str;
    type Source = &'storage str;

    fn name(&'storage self, _: ()) -> Result<Self::Name, Error> {
        Ok(self.path_str)
    }

    fn source(&'storage self, _: ()) -> Result<Self::Source, Error> {
        Ok(self.source)
    }

    fn line_index(&'storage self, _: (), byte_index: usize) -> Result<usize, Error> {
        Ok(self.get_line_index_by_byte_index(byte_index))
    }

    fn line_range(&'storage self, _: (), line_index: usize) -> Result<Range<usize>, Error> {
        let line_start = self.get_line_start_by_index(line_index)?;
        let next_line_start = self.get_line_start_by_index(line_index + 1)?;

        Ok(line_start..next_line_start)
    }
}
