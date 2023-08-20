//! Defines a [`InMemoryFileStorage`], to avoid rereading files in some situtations.

use std::io;

use ry_fx_hash::FxHashMap;
use ry_interner::{PathID, PathInterner};

use crate::in_memory_file::InMemoryFile;

/// In memory file storage. The storage can be used for example when emitting
/// some diagnostics, to avoid rereading the same file multiple times.
#[derive(Debug, Clone)]
pub struct InMemoryFileStorage<'p> {
    path_interner: &'p PathInterner,
    storage: FxHashMap<PathID, InMemoryFile>,
}

impl<'p> InMemoryFileStorage<'p> {
    /// Creates an empty storage.
    #[inline]
    #[must_use]
    pub fn new(path_interner: &'p PathInterner) -> Self {
        Self {
            path_interner,
            storage: FxHashMap::default(),
        }
    }

    /// Adds a file into the storage.
    #[inline]
    pub fn add_file(&mut self, path_id: PathID, file: InMemoryFile) {
        self.storage.insert(path_id, file);
    }

    /// Reads and adds a file into the storage.
    ///
    /// # Errors
    /// If the file contents cannot be read.
    #[inline]
    pub fn read_and_add_file(&mut self, path_id: PathID) -> Result<(), io::Error> {
        let file = InMemoryFile::new_from_path_id(self.path_interner, path_id)?;
        self.add_file(path_id, file);

        Ok(())
    }

    /// Reads and adds a file into the storage.
    ///
    /// # Panics
    /// If the file contents cannot be read.
    #[inline]
    pub fn read_and_add_file_or_panic(&mut self, path_id: PathID) {
        self.storage.insert(
            path_id,
            InMemoryFile::new_or_panic(self.path_interner.resolve_or_panic(path_id)),
        );
    }

    /// Adds a file into the storage if it does not exist.
    #[inline]
    pub fn add_file_if_not_exists(&mut self, path_id: PathID, file: InMemoryFile) {
        if !self.storage.contains_key(&path_id) {
            self.add_file(path_id, file);
        }
    }

    /// Reads and adds a file into the storage if it does not exist.
    ///
    /// # Errors
    /// If the file contents cannot be read.
    #[inline]
    pub fn read_and_add_file_if_not_exists(&mut self, path_id: PathID) -> Result<(), io::Error> {
        if !self.storage.contains_key(&path_id) {
            self.read_and_add_file(path_id)?;
        }

        Ok(())
    }

    /// Reads and adds a file into the storage if it does not exist.
    ///
    /// # Panics
    /// If the file contents cannot be read.
    #[inline]
    #[must_use]
    pub fn read_and_add_file_if_not_exists_or_panic(&mut self, path_id: PathID) -> InMemoryFile {
        InMemoryFile::new_or_panic(self.path_interner.resolve_or_panic(path_id))
    }

    /// Resolves a file from the storage by its path id.
    #[inline]
    #[must_use]
    pub fn resolve_file(&self, path_id: PathID) -> Option<&InMemoryFile> {
        self.storage.get(&path_id)
    }
}
