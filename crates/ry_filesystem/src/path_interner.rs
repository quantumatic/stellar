//! Defines a [`FilePathStorage`] to avoid copying and increasing comparison speed
//! of file paths.

use std::path::PathBuf;

use ry_fx_hash::FxHashMap;

/// Storage for file paths (to avoid copying and fast comparing, basically the same
/// movitation as with [`Interner`]).
///
/// The ID-s that correspond to file paths have a type of [`FilePathID`].
///
/// [`Interner`]: ry_interner::Interner
#[derive(Debug, Clone)]
pub struct PathInterner {
    paths: Vec<PathBuf>,
    map: FxHashMap<PathBuf, PathID>,
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
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            map: FxHashMap::default(),
        }
    }

    /// Adds a path to the storage.
    #[inline]
    #[must_use]
    pub fn get_or_intern_path(&mut self, path: PathBuf) -> PathID {
        if let Some(&path_id) = self.map.get(&path) {
            return path_id;
        }

        let path_id = self.paths.len() + 1;
        self.paths.push(path.clone());
        self.map.insert(path, path_id);

        path_id
    }

    /// Resolves a path stored in the storage.
    #[inline]
    #[must_use]
    pub fn resolve_path(&self, id: PathID) -> Option<PathBuf> {
        self.paths.get(id - 1).cloned()
    }

    /// Resolves a path stored in the storage (same as `resolve_path()`),
    /// but panics if the path is not found.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn resolve_path_or_panic(&self, id: PathID) -> PathBuf {
        self.paths
            .get(id - 1)
            .unwrap_or_else(|| panic!("Path with id: {id} is not found"))
            .clone()
    }
}
