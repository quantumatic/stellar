//! Defines a [`InMemoryFileStorage`] for diagnostics reporting and advanced file management.

use crate::{file::InMemoryFile, span::Span};
use codespan_reporting::files::{Error, Files};
use std::ops::Range;

/// Sotrage for in memory storage.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InMemoryFileStorage<'storage> {
    storage: Vec<&'storage InMemoryFile<'storage>>,
}

impl Default for InMemoryFileStorage<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// An ID used to refer to storage in [`InMemoryFileStorage`].
pub type FileID = usize;

impl<'storage> InMemoryFileStorage<'storage> {
    /// Creates a new empty [`InMemoryFileStorage`].
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            storage: Vec::new(),
        }
    }

    /// Adds a new file to the [`InMemoryFileStorage`] and returns its ID.
    ///
    /// # Note
    /// File IDs start from `1`.
    ///
    /// ```
    /// # use std::path::Path;
    /// # use ry_span::{storage::InMemoryFileStorage, file::InMemoryFile, span::Span};
    /// let mut storage = InMemoryFileStorage::new();
    /// let file = InMemoryFile::new(
    ///     Path::new("test.ry"),
    ///     "fun main() { println(\"Hello, world!\"); }"
    /// );
    ///
    /// assert_eq!(storage.add_file(&file), 1);
    /// ```
    pub fn add_file(&mut self, file: &'storage InMemoryFile<'storage>) -> FileID {
        self.storage.push(file);
        self.storage.len()
    }

    /// Returns the file with the given ID.
    #[inline]
    #[must_use]
    pub fn get_file_by_id(&self, file_id: FileID) -> Option<&'storage InMemoryFile<'storage>> {
        self.storage.get(file_id - 1).copied()
    }

    /// Returns the file with the given ID, without doing bounds checking.
    ///
    /// # Safety
    /// Calling this method with an out-of-bounds index is undefined behavior even if the resulting reference is not used.
    #[inline]
    #[must_use]
    pub unsafe fn get_file_by_id_unchecked(&self, file_id: FileID) -> &InMemoryFile<'storage> {
        unsafe { self.storage.get_unchecked(file_id - 1) }
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
    /// # use ry_span::{storage::InMemoryFileStorage, file::InMemoryFile, span::Span};
    /// let mut storage = InMemoryFileStorage::new();
    /// let file = InMemoryFile::new(
    ///     Path::new("test.ry"),
    ///     "fun main() { println(\"Hello, world!\"); }"
    /// );
    ///
    /// let file_id = storage.add_file(&file);
    /// let span = Span { start: 21, end: 36, file_id };
    ///
    /// assert_eq!(storage.resolve_span_or_panic(span), "\"Hello, world!\"");
    /// ```
    ///
    /// [`start`]: crate::span::Span::start
    /// [`end`]: crate::span::Span::end
    /// [`file_id`]: crate::span::Span::file_id
    #[must_use]
    pub fn resolve_span_or_panic(&self, span: Span) -> &'storage str {
        self.get_file_by_id(span.file_id)
            .expect("File does not exist")
            .source
            .get(span.start..span.end)
            .expect("Span is out of bounds")
    }

    /// Returns the content of the part of the source code situated
    /// at the given span.
    ///
    /// Instead of panicking in the situation when [`InMemoryFileStorage::resolve_span_or_panic()`] does,
    /// the function returns [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::Path;
    /// # use ry_span::{storage::InMemoryFileStorage, span::Span, file::InMemoryFile};
    /// let mut storage = InMemoryFileStorage::new();
    /// let file = InMemoryFile::new(
    ///     Path::new("test.ry"),
    ///     "fun main() { println(\"Hello, world!\"); }"
    /// );
    /// let file_id = storage.add_file(&file);
    ///
    /// assert_eq!(
    ///     storage.resolve_span(
    ///         Span { start: 21, end: 36, file_id }
    ///     ),
    ///     Some("\"Hello, world!\"")
    /// );
    /// assert_eq!(
    ///     // file does not exist
    ///     storage.resolve_span(Span { start: 0, end: 0, file_id: file_id + 1 }),
    ///     None
    /// );
    /// assert_eq!(
    ///     // out of bounds
    ///     storage.resolve_span(Span { start: 99, end: 100, file_id }),
    ///     None
    /// );
    #[must_use]
    pub fn resolve_span(&self, span: Span) -> Option<&'storage str> {
        self.get_file_by_id(span.file_id)?
            .source
            .get(span.start..span.end)
    }
}

impl<'storage> Files<'storage> for InMemoryFileStorage<'storage> {
    type FileId = FileID;
    type Name = &'storage str;
    type Source = &'storage str;

    fn name(&self, file_id: FileID) -> Result<Self::Name, Error> {
        self.get_file_by_id(file_id)
            .map(|f| f.path_str)
            .ok_or(Error::FileMissing)
    }

    fn source(&self, file_id: FileID) -> Result<Self::Source, Error> {
        self.get_file_by_id(file_id)
            .map(|f| f.source)
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
