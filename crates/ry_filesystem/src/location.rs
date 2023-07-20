//! Defines a [`Location`] for working with source text locations and provides some utilities.

use std::{fmt::Display, ops::Range};

use codespan_reporting::diagnostic::Label;
use ry_interner::PathID;

/// Represents location in the source text.
#[derive(Copy, Clone, Hash, Debug, Default, PartialEq, Eq)]
pub struct Location {
    /// ID of the source file.
    pub file_path_id: PathID,

    /// Offset of starting byte in the source text.
    pub start: usize,

    /// Offset of ending byte in the source text.
    pub end: usize,
}

/// Dummy location - location that is used as a placeholder in tests.
///
/// # Note
/// Using dummy location in code except in tests is not recommended,
/// because this can result in undefined behavior with diagnostics and
/// debug information!
pub const DUMMY_LOCATION: Location = Location {
    file_path_id: 0,
    start: 0,
    end: 0,
};

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}..{}", self.start, self.end))
    }
}

impl Location {
    /// Returns location of the first byte corresponding to the
    /// current location.
    ///
    /// ```
    /// # use ry_filesystem::location::{Location, LocationIndex};
    /// # use ry_interner::DUMMY_PATH_ID;
    /// let location = Location { file_path_id: DUMMY_PATH_ID, start: 0, end: 3 };
    ///
    /// assert_eq!(
    ///     location.start_byte_location(),
    ///     Location { file_path_id: DUMMY_PATH_ID, start: 0, end: 1 }
    /// );
    /// ```
    #[inline]
    #[must_use]
    pub const fn start_byte_location(self) -> Self {
        Self {
            file_path_id: self.file_path_id,
            start: self.start,
            end: self.start + 1,
        }
    }

    /// Returns location of the last byte corresponding to the
    /// current location.
    ///
    /// ```
    /// # use ry_filesystem::location::{Location, LocationIndex};
    /// # use ry_interner::DUMMY_PATH_ID;
    /// let location = Location { file_path_id: DUMMY_PATH_ID, start: 0, end: 3 };
    ///
    /// assert_eq!(
    ///     location.end_byte_location(),
    ///     Location { file_path_id: DUMMY_PATH_ID, start: 2, end: 3 }
    /// );
    /// ```
    #[inline]
    #[must_use]
    pub const fn end_byte_location(self) -> Self {
        Self {
            file_path_id: self.file_path_id,
            start: self.end - 1,
            end: self.end,
        }
    }

    /// Gets primary diagnostics label ([`Label`] from [`codespan_reporting`])
    /// in the location.
    #[inline]
    #[must_use]
    pub fn to_primary_label(self) -> Label<PathID> {
        Label::primary(self.file_path_id, self)
    }

    /// Gets secondary diagnostics label ([`Label`] from [`codespan_reporting`])
    /// in the location.
    #[inline]
    #[must_use]
    pub fn to_secondary_label(self) -> Label<PathID> {
        Label::secondary(self.file_path_id, self)
    }
}

impl From<Location> for Range<usize> {
    fn from(location: Location) -> Self {
        location.start..location.end
    }
}

impl From<Location> for String {
    fn from(value: Location) -> Self {
        format!("{}..{}", value.start, value.end)
    }
}

/// Allows to index a string using a given location.
pub trait LocationIndex {
    /// Output of the indexing operation.
    type Output: ?Sized;

    /// Index a string using a given location.
    ///
    /// # Example
    /// ```
    /// # use ry_filesystem::{location::{Location, LocationIndex}};
    /// # use ry_interner::DUMMY_PATH_ID;
    /// let location = Location { file_path_id: DUMMY_PATH_ID, start: 0, end: 3 };
    /// assert_eq!("test".index(location), "tes");
    /// ```
    fn index(&self, location: Location) -> &Self::Output;
}

impl<T> LocationIndex for T
where
    T: AsRef<str>,
{
    type Output = str;

    #[inline]
    #[allow(clippy::indexing_slicing)]
    fn index(&self, location: Location) -> &Self::Output {
        &self.as_ref()[location.start..location.end]
    }
}
