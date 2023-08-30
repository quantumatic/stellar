//! Source file support for diagnostic reporting.
//!
//! The main trait defined in this module is the [`Files`] trait, which provides
//! provides the minimum amount of functionality required for printing [`Diagnostics`]
//! with the [`term::emit`] function.
//!
//! [`term::emit`]: crate::term::emit
//! [`Diagnostics`]: crate::diagnostic::Diagnostic
//! [`Files`]: Files

use std::ops::Range;

use stellar_filesystem::in_memory_file::{InMemoryFile, LineTooLargeError};
use stellar_filesystem::in_memory_file_storage::InMemoryFileStorage;
use stellar_interner::PathID;

/// An enum representing an error that happened while looking up a file or a piece of content in that file.
#[derive(Debug)]
pub enum Error {
    /// A required file is not in the file database.
    FileMissing,

    /// The file is present, but does not contain the specified byte index.
    IndexTooLarge {
        /// The given byte index.
        given: usize,

        /// The maximum byte index.
        max: usize,
    },

    /// The file is present, but does not contain the specified line index.
    LineTooLarge {
        /// The given line index.
        given: usize,

        /// The maximum line index.
        max: usize,
    },

    /// The file is present and contains the specified line index, but the line does not contain
    /// the specified column index.
    ColumnTooLarge {
        /// The given column index.
        given: usize,

        /// The maximum column index.
        max: usize,
    },

    /// The given index is contained in the file, but is not a boundary of a UTF-8 code point.
    InvalidCharBoundary {
        /// The given index.
        given: usize,
    },

    /// There was a error while doing IO.
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileMissing => write!(f, "file missing"),
            Self::IndexTooLarge { given, max } => {
                write!(f, "invalid index {given}, maximum index is {max}")
            }
            Self::LineTooLarge { given, max } => {
                write!(f, "invalid line {given}, maximum line is {max}")
            }
            Self::ColumnTooLarge { given, max } => {
                write!(f, "invalid column {given}, maximum column {max}")
            }
            Self::InvalidCharBoundary { .. } => {
                write!(f, "index is not a code point boundary")
            }
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

/// A minimal interface for accessing source files when rendering diagnostics.
///
/// A lifetime parameter `'a` is provided to allow any of the returned values to returned by reference.
/// This is to workaround the lack of higher kinded lifetime parameters.
/// This can be ignored if this is not needed, however.
#[allow(clippy::missing_errors_doc)]
pub trait Files<'a> {
    /// A unique identifier for files in the file provider. This will be used
    /// for rendering `diagnostic::Label`s in the corresponding source files.
    type FileId: 'a + Copy + PartialEq;
    /// The user-facing name of a file, to be displayed in diagnostics.
    type Name: 'a + std::fmt::Display;
    /// The source code of a file.
    type Source: 'a + AsRef<str>;

    /// The user-facing name of a file.
    fn name(&'a self, id: Self::FileId) -> Result<Self::Name, Error>;

    /// The source code of a file.
    fn source(&'a self, id: Self::FileId) -> Result<Self::Source, Error>;

    /// The index of the line at the given byte index.
    /// If the byte index is past the end of the file, returns the maximum line index in the file.
    /// This means that this function only fails if the file is not present.
    ///
    /// # Note for trait implementors
    ///
    /// This can be implemented efficiently by performing a binary search over
    /// a list of line starts. It might be useful to pre-compute and cache these
    /// line starts.
    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Result<usize, Error>;

    /// The user-facing line number at the given line index.
    /// It is not necessarily checked that the specified line index
    /// is actually in the file.
    ///
    /// # Note for trait implementors
    ///
    /// This is usually 1-indexed from the beginning of the file, but
    /// can be useful for implementing something like the
    /// [C preprocessor's `#line` macro][line-macro].
    ///
    /// [line-macro]: https://en.cppreference.com/w/c/preprocessor/line
    #[allow(unused_variables)]
    fn line_number(&'a self, id: Self::FileId, line_index: usize) -> Result<usize, Error> {
        Ok(line_index + 1)
    }

    /// The user-facing column number at the given line index and byte index.
    ///
    /// # Note for trait implementors
    ///
    /// This is usually 1-indexed from the the start of the line.
    /// A default implementation is provided, based on the [`column_index`]
    /// function that is exported from the [`files`] module.
    ///
    /// [`files`]: crate::files
    /// [`column_index`]: crate::files::column_index
    fn column_number(
        &'a self,
        id: Self::FileId,
        line_index: usize,
        byte_index: usize,
    ) -> Result<usize, Error> {
        let source = self.source(id)?;
        let line_range = self.line_range(id, line_index)?;
        let column_index = column_index(source.as_ref(), line_range, byte_index);

        Ok(column_index + 1)
    }

    /// Convenience method for returning line and column number at the given
    /// byte index in the file.
    fn location(&'a self, id: Self::FileId, byte_index: usize) -> Result<Location, Error> {
        let line_index = self.line_index(id, byte_index)?;

        Ok(Location {
            line_number: self.line_number(id, line_index)?,
            column_number: self.column_number(id, line_index, byte_index)?,
        })
    }

    /// The byte range of line in the source of the file.
    fn line_range(&'a self, id: Self::FileId, line_index: usize) -> Result<Range<usize>, Error>;
}

/// A user-facing location in a source file.
///
/// Returned by [`Files::location`].
///
/// [`Files::location`]: Files::location
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Location {
    /// The user-facing line number.
    pub line_number: usize,
    /// The user-facing column number.
    pub column_number: usize,
}

/// The column index at the given byte index in the source file.
/// This is the number of characters to the given byte index.
///
/// If the byte index is smaller than the start of the line, then `0` is returned.
/// If the byte index is past the end of the line, the column index of the last
/// character `+ 1` is returned.
///
/// # Example
///
/// ```rust
/// use stellar_diagnostics::files;
///
/// let source = "\n\n🗻∈🌏\n\n";
///
/// assert_eq!(files::column_index(source, 0..1, 0), 0);
/// assert_eq!(files::column_index(source, 2..13, 0), 0);
/// assert_eq!(files::column_index(source, 2..13, 2 + 0), 0);
/// assert_eq!(files::column_index(source, 2..13, 2 + 1), 0);
/// assert_eq!(files::column_index(source, 2..13, 2 + 4), 1);
/// assert_eq!(files::column_index(source, 2..13, 2 + 8), 2);
/// assert_eq!(files::column_index(source, 2..13, 2 + 10), 2);
/// assert_eq!(files::column_index(source, 2..13, 2 + 11), 3);
/// assert_eq!(files::column_index(source, 2..13, 2 + 12), 3);
/// ```
#[must_use]
pub fn column_index(source: &str, line_range: Range<usize>, byte_index: usize) -> usize {
    let end_index = std::cmp::min(byte_index, std::cmp::min(line_range.end, source.len()));

    (line_range.start..end_index)
        .filter(|byte_index| source.is_char_boundary(byte_index + 1))
        .count()
}

impl<'a> Files<'a> for InMemoryFile {
    // we don't care about file IDs, because we have only one individual file here
    type FileId = ();

    type Name = String;
    type Source = &'a str;

    #[inline(always)]
    fn name(&self, _: ()) -> Result<Self::Name, Error> {
        Ok(format!("{}", self.path.display()))
    }

    #[inline(always)]
    fn source(&'a self, _: ()) -> Result<Self::Source, Error> {
        Ok(&self.source)
    }

    #[inline(always)]
    fn line_index(&self, _: (), byte_index: usize) -> Result<usize, Error> {
        Ok(self.get_line_index_by_byte_index(byte_index))
    }

    #[inline(always)]
    fn line_range(&self, _: (), line_index: usize) -> Result<Range<usize>, Error> {
        let line_start = self.get_line_start_by_index(line_index)?;
        let next_line_start = self.get_line_start_by_index(line_index + 1)?;

        Ok(line_start..next_line_start)
    }
}

impl From<LineTooLargeError> for Error {
    fn from(value: LineTooLargeError) -> Self {
        Self::LineTooLarge {
            given: value.given,
            max: value.max,
        }
    }
}

impl<'a> Files<'a> for InMemoryFileStorage {
    type FileId = PathID;

    type Name = String;
    type Source = &'a str;

    #[inline(always)]
    fn name(&'a self, id: PathID) -> Result<Self::Name, Error> {
        self.resolve_file(id)
            .map(|file| file.path.display().to_string())
            .ok_or(Error::FileMissing)
    }

    #[inline(always)]
    fn source(&'a self, id: PathID) -> Result<Self::Source, Error> {
        self.resolve_file(id)
            .map(|file| file.source.as_str())
            .ok_or(Error::FileMissing)
    }

    #[inline(always)]
    fn line_index(&'a self, id: PathID, byte_index: usize) -> Result<usize, Error> {
        self.resolve_file(id)
            .ok_or(Error::FileMissing)
            .and_then(|file| file.line_index((), byte_index))
    }

    #[inline(always)]
    fn line_range(&'a self, id: PathID, line_index: usize) -> Result<Range<usize>, Error> {
        self.resolve_file(id)
            .ok_or(Error::FileMissing)
            .and_then(|file| file.line_range((), line_index))
    }
}
