//! Defines a [`InMemoryFile`] to represent a Ry source file and provides some utilities.

use std::io;
use std::ops::Range;
use std::{cmp::Ordering, path::PathBuf};

use codespan_reporting::files::{Error, Files};

use crate::location::{Location, LocationIndex};
use crate::path_storage::{PathID, PathStorage};

/// A Ry source file.
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
    #[inline]
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
    #[inline]
    pub fn new_from_path_id(
        path_storage: &PathStorage,
        path_id: PathID,
    ) -> Result<Self, io::Error> {
        Self::new(path_storage.resolve_path_or_panic(path_id))
    }

    /// Creates a new [`InMemoryFile`] and panics if its contents
    /// cannot be read.
    ///
    /// # Panics
    /// If the file contents cannot be read.
    #[inline]
    #[must_use]
    pub fn new_or_panic(path: impl Into<PathBuf> + Clone) -> Self {
        Self::new(path).expect("Cannot read the file")
    }

    /// Creates a new [`InMemoryFile`] from its path id.
    ///
    /// # Panics
    /// * If the path id cannot be resolved in the path storage.
    /// * If the source of the file cannot be read.
    #[inline]
    #[must_use]
    pub fn new_from_path_id_or_panic(path_storage: &PathStorage, path_id: PathID) -> Self {
        Self::new_or_panic(path_storage.resolve_path_or_panic(path_id))
    }

    /// Creates a new [`InMemoryFile`] with the given source.
    #[inline]
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

    /// Returns the string slice corresponding to the given location.
    #[inline]
    #[must_use]
    pub fn resolve_location(&self, location: Location) -> &str {
        self.source.index(location)
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
    /// # use ry_filesystem::file::InMemoryFile;
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
    /// # use ry_filesystem::file::InMemoryFile;
    /// let file = InMemoryFile::new_from_source(
    ///     Path::new("test.ry"),
    ///     "fun main() {
    ///     println(\"Hello, world!\");
    /// }".to_owned(),
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
    /// # use ry_filesystem::file::InMemoryFile;
    /// let file = InMemoryFile::new_from_source(
    ///     Path::new("test.ry"),
    ///     "fun main() {
    ///     println(\"Hello, world!\");
    /// }".to_owned(),
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
impl<'a> Files<'a> for InMemoryFile {
    // we don't care about file IDs, because we have only one individual file here
    type FileId = ();

    type Name = String;
    type Source = &'a str;

    #[inline]
    fn name(&self, _: ()) -> Result<Self::Name, Error> {
        Ok(format!("{}", self.path.display()))
    }

    #[inline]
    fn source(&'a self, _: ()) -> Result<Self::Source, Error> {
        Ok(&self.source)
    }

    #[inline]
    fn line_index(&self, _: (), byte_index: usize) -> Result<usize, Error> {
        Ok(self.get_line_index_by_byte_index(byte_index))
    }

    #[inline]
    fn line_range(&self, _: (), line_index: usize) -> Result<Range<usize>, Error> {
        let line_start = self.get_line_start_by_index(line_index)?;
        let next_line_start = self.get_line_start_by_index(line_index + 1)?;

        Ok(line_start..next_line_start)
    }
}
