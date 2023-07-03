//! Defines a [`Workspace`] for diagnostics reporting and advanced file management.

use crate::{file::SourceFile, span::Span};
use codespan_reporting::files::{Error, Files};
use std::ops::Range;

/// A source file manager.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Workspace<'workspace> {
    files: Vec<&'workspace SourceFile<'workspace>>,
}

impl Default for Workspace<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// An ID used to refer to files in [`Workspace`].
pub type FileID = usize;

impl<'workspace> Workspace<'workspace> {
    /// Creates a new empty [`Workspace`].
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Adds a new file to the [`Workspace`] and returns its ID.
    ///
    /// # Note
    /// File IDs start from `1`.
    ///
    /// ```
    /// # use std::path::Path;
    /// # use ry_workspace::{workspace::Workspace, file::SourceFile, span::Span};
    /// let mut workspace = Workspace::new();
    /// let source_file = SourceFile::new(
    ///     Path::new("test.ry"),
    ///     "fun main() { println(\"Hello, world!\"); }"
    /// );
    ///
    /// assert_eq!(workspace.add_file(&source_file), 1);
    /// ```
    pub fn add_file(&mut self, file: &'workspace SourceFile<'workspace>) -> FileID {
        self.files.push(file);
        self.files.len()
    }

    /// Returns the file with the given ID.
    #[inline]
    #[must_use]
    pub fn get_file_by_id(&self, file_id: FileID) -> Option<&'workspace SourceFile<'workspace>> {
        self.files.get(file_id - 1).copied()
    }

    /// Returns the file with the given ID, without doing bounds checking.
    ///
    /// # Safety
    /// Calling this method with an out-of-bounds index is undefined behavior even if the resulting reference is not used.
    #[inline]
    #[must_use]
    pub unsafe fn get_file_by_id_unchecked(&self, file_id: FileID) -> &SourceFile<'workspace> {
        unsafe { self.files.get_unchecked(file_id - 1) }
    }

    /// Returns the content of the part of the source code situated
    /// at the given span if it is valid.
    ///
    /// # Panics
    /// - If the span is out of bounds ([`start`] and [`end`]).
    /// - If the file with the given [`file_id`] does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::Path;
    /// # use ry_workspace::{workspace::Workspace, file::SourceFile, span::Span};
    /// let mut workspace = Workspace::new();
    /// let source_file = SourceFile::new(
    ///     Path::new("test.ry"),
    ///     "fun main() { println(\"Hello, world!\"); }"
    /// );
    ///
    /// let file_id = workspace.add_file(&source_file);
    /// let span = Span::new(21, 36, file_id);
    ///
    /// assert_eq!(workspace.resolve_span_or_panic(span), "\"Hello, world!\"");
    /// ```
    ///
    /// [`start`]: crate::span::Span::start
    /// [`end`]: crate::span::Span::end
    /// [`file_id`]: crate::span::Span::file_id
    #[must_use]
    pub fn resolve_span_or_panic(&self, span: Span) -> &'workspace str {
        self.get_file_by_id(span.file_id())
            .expect("File does not exist")
            .source()
            .get(span.start()..span.end())
            .expect("Span is out of bounds")
    }

    /// Returns the content of the part of the source code situated
    /// at the given span.
    ///
    /// Instead of panicking in the situation when [`Workspace::resolve_span_or_panic()`] does,
    /// the function returns [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::Path;
    /// # use ry_workspace::{workspace::Workspace, span::Span, file::SourceFile};
    /// let mut workspace = Workspace::new();
    /// let source_file = SourceFile::new(
    ///     Path::new("test.ry"),
    ///     "fun main() { println(\"Hello, world!\"); }"
    /// );
    /// let file_id = workspace.add_file(&source_file);
    ///
    /// assert_eq!(
    ///     workspace.resolve_span(
    ///         Span::new(21, 36, file_id)
    ///     ),
    ///     Some("\"Hello, world!\"")
    /// );
    /// assert_eq!(
    ///     // file does not exist
    ///     workspace.resolve_span(Span::new(0, 0, file_id + 1)),
    ///     None
    /// );
    /// assert_eq!(
    ///     // out of bounds
    ///     workspace.resolve_span(Span::new(99, 100, file_id)),
    ///     None
    /// );
    #[must_use]
    pub fn resolve_span(&self, span: Span) -> Option<&'workspace str> {
        self.get_file_by_id(span.file_id())?
            .source()
            .get(span.start()..span.end())
    }
}

impl<'workspace> Files<'workspace> for Workspace<'workspace> {
    type FileId = FileID;
    type Name = &'workspace str;
    type Source = &'workspace str;

    fn name(&self, file_id: FileID) -> Result<Self::Name, Error> {
        self.get_file_by_id(file_id)
            .map(SourceFile::path_str)
            .ok_or(Error::FileMissing)
    }

    fn source(&self, file_id: FileID) -> Result<Self::Source, Error> {
        self.get_file_by_id(file_id)
            .map(SourceFile::source)
            .ok_or(Error::FileMissing)
    }

    fn line_index(&self, file_id: FileID, byte_index: usize) -> Result<usize, Error> {
        self.get_file_by_id(file_id)
            .ok_or(Error::FileMissing)?
            .line_index((), byte_index)
    }

    fn line_range(&self, file_id: FileID, line_index: usize) -> Result<Range<usize>, Error> {
        self.get_file_by_id(file_id)
            .ok_or(Error::FileMissing)?
            .line_range((), line_index)
    }
}
