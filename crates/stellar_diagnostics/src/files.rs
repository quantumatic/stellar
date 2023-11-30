//! Source file support for diagnostic reporting.
//!
//! The main trait defined in this module is the [`DiagnosticsRenderHelper`] trait, which provides
//! provides the minimum amount of functionality required for printing [`Diagnostics`]
//! with the [`term::emit`] function.
//!
//! [`term::emit`]: crate::term::emit
//! [`Diagnostics`]: crate::diagnostic::Diagnostic

use stellar_filesystem::in_memory_file::LineTooLargeError;
use stellar_filesystem::in_memory_file_storage::InMemoryFileStorage;
use stellar_filesystem::location::{ByteOffset, Location};
use stellar_interner::PathId;

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
pub(crate) trait DiagnosticsRenderHelper<'a> {
    /// The user-facing name of a file.
    fn name(&'a self, filepath: PathId) -> Result<String, Error>;

    /// The source code of a file.
    fn source(&'a self, filepath: PathId) -> Result<&'a str, Error>;

    /// The index of the line at the given byte index.
    /// If the byte index is past the end of the file, returns the maximum line index in the file.
    /// This means that this function only fails if the file is not present.
    ///
    /// # Note for trait implementors
    ///
    /// This can be implemented efficiently by performing a binary search over
    /// a list of line starts. It might be useful to pre-compute and cache these
    /// line starts.
    fn line_index(&'a self, filepath: PathId, byte_offset: ByteOffset) -> Result<usize, Error>;

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
    fn line_number(&'a self, filepath: PathId, byte_offset: ByteOffset) -> Result<usize, Error> {
        self.line_index(filepath, byte_offset).map(|idx| idx + 1)
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
        filepath: PathId,
        line_index: usize,
        byte_offset: ByteOffset,
    ) -> Result<usize, Error> {
        let source = self.source(filepath)?;
        let line_range = self.line_location(filepath, line_index)?;
        let column_index = column_index(source, line_range, byte_offset);

        Ok(column_index + 1)
    }

    /// Convenience method for returning line and column number at the given
    /// byte index in the file.
    fn location(
        &'a self,
        filepath: PathId,
        byte_offset: ByteOffset,
    ) -> Result<ResolvedLocation, Error> {
        let line_index = self.line_index(filepath, byte_offset)?;

        Ok(ResolvedLocation {
            line_number: line_index + 1,
            column_number: self.column_number(filepath, line_index, byte_offset)?,
        })
    }

    /// The byte range of line in the source of the file.
    fn line_location(&'a self, filepath: PathId, line_index: usize) -> Result<Location, Error>;
}

/// A user-facing location in a source file.
///
/// Returned by [`Files::location`].
///
/// [`Files::location`]: Files::location
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedLocation {
    /// The user-facing line number.
    pub(crate) line_number: usize,
    /// The user-facing column number.
    pub(crate) column_number: usize,
}

/// The column index at the given byte index in the source file.
/// This is the number of characters to the given byte index.
///
/// If the byte index is smaller than the start of the line, then `0` is returned.
/// If the byte index is past the end of the line, the column index of the last
/// character `+ 1` is returned.
#[must_use]
pub fn column_index(source: &str, location: Location, byte_offset: ByteOffset) -> usize {
    let end_index = std::cmp::min(
        byte_offset,
        std::cmp::min(location.end, source.len().into()),
    );

    (location.start.0..end_index.0)
        .filter(|byte_index| source.is_char_boundary(byte_index + 1))
        .count()
}
impl From<LineTooLargeError> for Error {
    fn from(value: LineTooLargeError) -> Self {
        Self::LineTooLarge {
            given: value.given,
            max: value.max,
        }
    }
}

impl<'a> DiagnosticsRenderHelper<'a> for InMemoryFileStorage {
    #[inline]
    fn name(&'a self, filepath: PathId) -> Result<String, Error> {
        self.resolve_file(filepath)
            .map(|file| file.path.as_path().display().to_string())
            .ok_or(Error::FileMissing)
    }

    #[inline]
    fn source(&'a self, filepath: PathId) -> Result<&'a str, Error> {
        self.resolve_file(filepath)
            .map(|file| file.source.as_str())
            .ok_or(Error::FileMissing)
    }

    #[inline]
    fn line_index(&'a self, filepath: PathId, byte_offset: ByteOffset) -> Result<usize, Error> {
        self.resolve_file(filepath)
            .ok_or(Error::FileMissing)
            .map(|file| file.get_line_index_by_byte_index(byte_offset))
    }

    fn line_location(&'a self, filepath: PathId, line_index: usize) -> Result<Location, Error> {
        self.resolve_file(filepath)
            .ok_or(Error::FileMissing)
            .and_then(|file| {
                let line_start = file.get_line_start_by_index(line_index)?;
                let next_line_start = file.get_line_start_by_index(line_index + 1)?;

                Ok(Location {
                    filepath,
                    start: line_start,
                    end: next_line_start,
                })
            })
    }
}
