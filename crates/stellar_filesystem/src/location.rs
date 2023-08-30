//! Defines a [`Location`] for working with source text locations and provides some utilities.

use std::{
    fmt::Display,
    ops::{Add, AddAssign, Range, Sub, SubAssign},
};

use derive_more::Display;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use stellar_interner::PathID;

/// Represents location in the source text.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Location {
    /// ID of the source file.
    pub filepath_id: PathID,

    /// Offset of starting byte in the source text.
    pub start: ByteOffset,

    /// Offset of ending byte in the source text.
    pub end: ByteOffset,
}

/// Offset of a byte in a source text.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ByteOffset(pub usize);

impl Add for ByteOffset {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<usize> for ByteOffset {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign for ByteOffset {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl AddAssign<usize> for ByteOffset {
    #[inline(always)]
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Sub for ByteOffset {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Sub<usize> for ByteOffset {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign for ByteOffset {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl SubAssign<usize> for ByteOffset {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}

impl ByteOffset {
    /// Returns location of the next byte relative to the current offset.
    #[inline(always)]
    #[must_use]
    pub const fn next_byte_location_at(self, filepath_id: PathID) -> Location {
        Location {
            filepath_id,
            start: self,
            end: Self(self.0 + 1),
        }
    }

    /// Returns location of the previous byte relative to the current offset.
    #[inline(always)]
    #[must_use]
    pub const fn previous_byte_location_at(self, filepath_id: PathID) -> Location {
        Location {
            filepath_id,
            start: Self(self.0 - 1),
            end: self,
        }
    }
}

/// Dummy location - location that is used as a placeholder in tests.
///
/// # Note
/// Using dummy location in code except in tests is not recommended,
/// because this can result in undefined behavior with diagnostics and
/// debug information!
pub const DUMMY_LOCATION: Location = Location {
    filepath_id: PathID(0),
    start: ByteOffset(0),
    end: ByteOffset(0),
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
    /// # use stellar_filesystem::location::{Location, LocationIndex, ByteOffset};
    /// # use stellar_interner::DUMMY_PATH_ID;
    /// let location = Location {
    ///     filepath_id: DUMMY_PATH_ID,
    ///     start: ByteOffset(0),
    ///     end: ByteOffset(3)
    /// };
    ///
    /// assert_eq!(
    ///     location.start_byte_location(),
    ///     Location {
    ///         filepath_id: DUMMY_PATH_ID,
    ///         start: ByteOffset(0),
    ///         end: ByteOffset(1)
    ///     }
    /// );
    /// ```
    #[inline(always)]
    #[must_use]
    pub const fn start_byte_location(self) -> Self {
        self.start.next_byte_location_at(self.filepath_id)
    }

    /// Returns location of the last byte corresponding to the
    /// current location.
    ///
    /// ```
    /// # use stellar_filesystem::location::{Location, LocationIndex, ByteOffset};
    /// # use stellar_interner::DUMMY_PATH_ID;
    /// let location = Location {
    ///     filepath_id: DUMMY_PATH_ID,
    ///     start: ByteOffset(0),
    ///     end: ByteOffset(3)
    /// };
    ///
    /// assert_eq!(
    ///     location.end_byte_location(),
    ///     Location {
    ///         filepath_id: DUMMY_PATH_ID,
    ///         start: ByteOffset(2),
    ///         end: ByteOffset(3)
    ///     }
    /// );
    /// ```
    #[inline(always)]
    #[must_use]
    pub const fn end_byte_location(self) -> Self {
        self.end.previous_byte_location_at(self.filepath_id)
    }
}

impl From<Location> for Range<usize> {
    fn from(location: Location) -> Self {
        location.start.0..location.end.0
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
    /// # use stellar_filesystem::location::{Location, LocationIndex, ByteOffset};
    /// # use stellar_interner::DUMMY_PATH_ID;
    /// let location = Location {
    ///     filepath_id: DUMMY_PATH_ID,
    ///     start: ByteOffset(0),
    ///     end: ByteOffset(3)
    /// };
    ///
    /// assert_eq!("test".index(location), "tes");
    /// ```
    fn index(&self, location: Location) -> &Self::Output;
}

impl<T> LocationIndex for T
where
    T: AsRef<str>,
{
    type Output = str;

    #[inline(always)]
    #[allow(clippy::indexing_slicing)]
    fn index(&self, location: Location) -> &Self::Output {
        &self.as_ref()[location.start.0..location.end.0]
    }
}
