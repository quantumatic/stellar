//! Defines a [`FilePathStorage`] to avoid copying and increasing comparison speed
//! of file paths.

use std::path::{Path, PathBuf};

use ry_interner::Interner;

/// Storage for file paths (to avoid copying and fast comparing, basically the same
/// movitation as with [`Interner`]).
///
/// The ID-s that correspond to file paths have a type of [`FilePathID`].
///
/// [`Interner`]: ry_interner::Interner
#[derive(Debug, Clone)]
pub struct PathInterner {
    path_string_interner: Interner,
}

/// ID of a path in the [`FilePathStorage`].
pub type PathID = usize;

/// ID of a path, that will never exist in the [`FilePathStorage`].
pub const DUMMY_PATH_ID: PathID = 0;

impl Default for PathInterner {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl PathInterner {
    /// Creates a new empty file path storage.
    #[must_use]
    pub fn new() -> Self {
        Self {
            path_string_interner: Interner::new(),
        }
    }

    /// Adds a path to the storage.
    #[inline]
    #[must_use]
    pub fn get_or_intern_path(&mut self, path: impl AsRef<Path>) -> PathID {
        self.path_string_interner
            .get_or_intern(path.as_ref().to_str().expect("Invalid UTF-8 path"))
    }

    /// Resolves a path stored in the storage.
    #[inline]
    #[must_use]
    pub fn resolve_path(&self, id: PathID) -> Option<PathBuf> {
        self.path_string_interner.resolve(id).map(PathBuf::from)
    }

    /// Resolves a path stored in the storage (same as `resolve_path()`),
    /// but panics if the path is not found.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn resolve_path_or_panic(&self, id: PathID) -> PathBuf {
        self.resolve_path(id)
            .unwrap_or_else(|| panic!("Path with id: {id} is not found"))
    }
}
