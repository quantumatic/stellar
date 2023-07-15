//! Defines a [`FilePathStorage`] to avoid copying and increasing comparison speed
//! of file paths.

use std::path::PathBuf;

/// Storage for file paths (to avoid copying and fast comparing, basically the same
/// movitation as with [`Interner`]).
///
/// The ID-s that correspond to file paths have a type of [`FilePathID`].
///
/// [`Interner`]: ry_interner::Interner
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PathStorage {
    storage: Vec<PathBuf>,
}

/// ID of a path in the [`FilePathStorage`].
pub type PathID = usize;

/// ID of a path, that will never exist in the [`FilePathStorage`].
pub const DUMMY_PATH_ID: PathID = 0;

impl Default for PathStorage {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl PathStorage {
    /// Creates a new empty file path storage.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            storage: Vec::new(),
        }
    }

    /// Adds a path to the storage.
    #[inline]
    #[must_use]
    pub fn add_path(&mut self, path: PathBuf) -> PathID {
        self.storage.push(path);
        self.storage.len() - 1
    }

    /// Resolves a path stored in the storage.
    #[inline]
    #[must_use]
    pub fn resolve_path(&self, id: PathID) -> Option<PathBuf> {
        self.storage.get(id).cloned()
    }

    /// Resolves a path stored in the storage (same as `resolve_path()`),
    /// but panics if the path is not found.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn resolve_path_or_panic(&self, id: PathID) -> PathBuf {
        self.storage
            .get(id)
            .unwrap_or_else(|| panic!("Path with id: {id} is not found"))
            .clone()
    }
}
