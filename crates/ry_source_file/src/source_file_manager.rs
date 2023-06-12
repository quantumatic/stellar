//! Defines a [`SourceFileManager`] for diagnostics reporting and advanced file management.

use crate::{source_file::SourceFile, span::Span};
use codespan_reporting::files::{Error, Files};
use std::ops::Range;

/// A source file manager.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceFileManager<'a> {
    files: Vec<SourceFile<'a>>,
}

impl Default for SourceFileManager<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> SourceFileManager<'a> {
    /// Creates a new empty [`SourceFileManager`].
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Adds a new file to the [`SourceFileManager`] and returns its ID.
    pub fn add_file(&mut self, file: SourceFile<'a>) -> usize {
        self.files.push(file);
        self.files.len() - 1
    }

    /// Returns the file with the given ID.
    #[inline]
    #[must_use]
    pub fn get_file_by_id(&self, file_id: usize) -> Option<&SourceFile<'a>> {
        self.files.get(file_id)
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
    /// use std::path::Path;
    /// use ry_source_file::{source_file_manager::SourceFileManager, source_file::SourceFile, span::Span};
    ///
    /// let mut file_manager = SourceFileManager::new();
    /// let file_id = file_manager.add_file(
    ///     SourceFile::new(Path::new("test.ry"), "fun main() { println(\"Hello, world!\"); }")
    /// );
    /// let span = Span::new(21, 36, file_id);
    /// assert_eq!(file_manager.resolve_span(span), "\"Hello, world!\"");
    /// ```
    #[must_use]
    pub fn resolve_span(&self, span: Span) -> &'a str {
        let file = self
            .get_file_by_id(span.file_id())
            .expect("File does not exist");
        let source = file.source();
        source
            .get(span.start()..span.end())
            .expect("Span is out of bounds")
    }

    /// Returns the content of the part of the source code situated
    /// at the given span.
    ///
    /// Instead of panicking in the situation when [`SourceFileManager::resolve_span()`] does,
    /// the function returns [`None`]. In all other cases
    /// `Some(Span::resolve_span(...))`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use ry_source_file::{source_file_manager::SourceFileManager, span::Span, source_file::SourceFile};
    ///
    /// let mut file_manager = SourceFileManager::new();
    /// let file_id = file_manager.add_file(
    ///     SourceFile::new(Path::new("test.ry"), "fun main() { println(\"Hello, world!\"); }")
    /// );
    /// assert_eq!(
    ///     file_manager.optionally_resolve_span(
    ///         Span::new(21, 36, file_id)
    ///     ),
    ///     Some("\"Hello, world!\"")
    /// );
    /// assert_eq!(
    ///     // file does not exist
    ///     file_manager.optionally_resolve_span(Span::new(0, 0, file_id + 1)),
    ///     None
    /// );
    /// assert_eq!(
    ///     // out of bounds
    ///     file_manager.optionally_resolve_span(Span::new(99, 100, file_id)),
    ///     None
    /// );
    #[must_use]
    pub fn optionally_resolve_span(&self, span: Span) -> Option<&'a str> {
        let Some(file) = self.get_file_by_id(span.file_id()) else {
            return None;
        };

        let source = file.source();
        source.get(span.start()..span.end())
    }
}

impl<'a> Files<'a> for SourceFileManager<'a> {
    type FileId = usize;
    type Name = &'a str;
    type Source = &'a str;

    fn name(&self, file_id: usize) -> Result<Self::Name, Error> {
        self.get_file_by_id(file_id)
            .map(SourceFile::path_str)
            .ok_or(Error::FileMissing)
    }

    fn source(&self, file_id: usize) -> Result<Self::Source, Error> {
        self.get_file_by_id(file_id)
            .map(SourceFile::source)
            .ok_or(Error::FileMissing)
    }

    fn line_index(&self, file_id: usize, byte_index: usize) -> Result<usize, Error> {
        self.get_file_by_id(file_id)
            .ok_or(Error::FileMissing)?
            .line_index((), byte_index)
    }

    fn line_range(&self, file_id: usize, line_index: usize) -> Result<Range<usize>, Error> {
        self.get_file_by_id(file_id)
            .ok_or(Error::FileMissing)?
            .line_range((), line_index)
    }
}
