//! Defines a [`InMemoryFile`] to represent a Stellar source file and provides some utilities.

use std::io;
use std::ops::Range;
use std::{cmp::Ordering, path::PathBuf};

use stellar_interner::PathId;

use crate::location::ByteOffset;

/// A Stellar source file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InMemoryFile {
    /// The path of the source file.
    pub path: PathBuf,

    /// The source content of the file.
    pub source: String,

    /// The length of the source content (in bytes).
    pub source_len: usize,

    /// The array of line starting byte indices in the [`InMemoryFile::source`].
    pub line_starts: Vec<usize>,
}

impl InMemoryFile {
    /// Creates a new [`InMemoryFile`].
    ///
    /// # Errors
    /// If the source of the file cannot be read.
    #[inline(always)]
    pub fn new(path: impl Into<PathBuf> + Clone) -> Result<Self, io::Error> {
        Ok(Self::new_from_source(
            path.clone(),
            std::fs::read_to_string(path.into())?,
        ))
    }

    /// Creates a new [`InMemoryFile`] from its path id.
    ///
    /// # Panics
    /// If the path id cannot be resolved in the path storage.
    ///
    /// # Errors
    /// If the source of the file cannot be read.
    #[inline(always)]
    pub fn new_from_path(path: PathId) -> Result<Self, io::Error> {
        Self::new(path.resolve_or_panic())
    }

    /// Creates a new [`InMemoryFile`] and panics if its contents
    /// cannot be read.
    ///
    /// # Panics
    /// If the file contents cannot be read.
    #[inline(always)]
    #[must_use]
    pub fn new_or_panic(path: impl Into<PathBuf> + Clone) -> Self {
        Self::new(path).expect("Cannot read the file")
    }

    /// Creates a new [`InMemoryFile`] from its path id.
    ///
    /// # Panics
    /// * If the path id cannot be resolved in the path storage.
    /// * If the source of the file cannot be read.
    #[inline(always)]
    #[must_use]
    pub fn new_from_path_or_panic(path: PathId) -> Self {
        Self::new_or_panic(path.resolve_or_panic())
    }

    /// Creates a new [`InMemoryFile`] with the given source.
    #[inline(always)]
    #[must_use]
    pub fn new_from_source(path: impl Into<PathBuf>, source: String) -> Self {
        Self {
            path: path.into(),
            source_len: source.len(),
            line_starts: std::iter::once(0)
                .chain(source.match_indices('\n').map(|(i, _)| i + 1))
                .collect(),
            source,
        }
    }

    /// Returns the line starting byte index of the given byte index.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::path::Path;
    /// # use stellar_filesystem::{in_memory_file::InMemoryFile, location::ByteOffset};
    /// let file = InMemoryFile::new_from_source(
    ///     Path::new("test.sr"),
    ///     "fun main() {
    ///     println(\"Hello, world!\");
    /// }".to_owned(),
    /// );
    ///
    /// assert_eq!(file.get_line_index_by_byte_index(ByteOffset(0)), 0);
    /// assert_eq!(file.get_line_index_by_byte_index(ByteOffset(13)), 1);
    /// ```
    #[must_use]
    pub fn get_line_index_by_byte_index(&self, byte_offset: ByteOffset) -> usize {
        self.line_starts
            .binary_search(&byte_offset.0)
            .unwrap_or_else(|next_line| next_line - 1)
    }

    /// Returns the line starting byte index of the given line index.
    ///
    /// # Panics
    /// Panics if the given line index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::Path;
    /// # use stellar_filesystem::{in_memory_file::InMemoryFile, location::ByteOffset};
    /// let file = InMemoryFile::new_from_source(
    ///     Path::new("test.sr"),
    ///     "fun main() {
    ///     println(\"Hello, world!\");
    /// }".to_owned(),
    /// );
    ///
    /// assert_eq!(file.get_line_start_by_index(0).unwrap(), ByteOffset(0));
    /// assert_eq!(file.get_line_start_by_index(1).unwrap(), ByteOffset(13));
    /// assert!(file.get_line_start_by_index(5).is_err());
    /// ```
    ///
    /// # Errors
    /// When line index is out of bounds.
    pub fn get_line_start_by_index(
        &self,
        line_index: usize,
    ) -> Result<ByteOffset, LineTooLargeError> {
        match line_index.cmp(&self.line_starts.len()) {
            Ordering::Less => Ok(ByteOffset(
                *self
                    .line_starts
                    .get(line_index)
                    .expect("failed despite previous check"),
            )),
            Ordering::Equal => Ok(ByteOffset(self.source.len())),
            Ordering::Greater => Err(LineTooLargeError {
                given: line_index,
                max: self.line_starts.len() - 1,
            }),
        }
    }

    /// Returns the line range of the given line index.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::path::Path;
    /// # use stellar_filesystem::{in_memory_file::InMemoryFile, location::ByteOffset};
    /// let file = InMemoryFile::new_from_source(
    ///     Path::new("test.sr"),
    ///     "fun main() {
    ///     println(\"Hello, world!\");
    /// }".to_owned(),
    /// );
    ///
    /// assert_eq!(file.line_range_by_index(0), Some(ByteOffset(0)..ByteOffset(13)));
    /// assert_eq!(file.line_range_by_index(1), Some(ByteOffset(13)..ByteOffset(43)));
    /// assert_eq!(file.line_range_by_index(2), Some(ByteOffset(43)..ByteOffset(44)));
    /// ```
    #[must_use]
    pub fn line_range_by_index(&self, line_index: usize) -> Option<Range<ByteOffset>> {
        let current_line_start = self.get_line_start_by_index(line_index).ok()?;
        let next_line_start = self.get_line_start_by_index(line_index + 1).ok()?;

        Some(current_line_start..next_line_start)
    }
}

/// Error returned by [`InMemoryFile::get_line_start_by_index`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineTooLargeError {
    /// The given line index.
    pub given: usize,

    /// The maximum line index.
    pub max: usize,
}
