//! Defines a [`InMemoryFileStorage`], to avoid rereading files in some situtations.

use std::io;

use stellar_fx_hash::FxHashMap;
use stellar_interner::PathID;

use crate::in_memory_file::InMemoryFile;

/// In memory file storage. The storage can be used for example when emitting
/// some diagnostics, to avoid rereading the same file multiple times.
#[derive(Debug, Clone, Default)]
pub struct InMemoryFileStorage {
    storage: FxHashMap<PathID, InMemoryFile>,
}

impl InMemoryFileStorage {
    /// Creates an empty storage.
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a file into the storage.
    #[inline(always)]
    pub fn add_file(&mut self, path: PathID, file: InMemoryFile) {
        self.storage.insert(path, file);
    }

    /// Reads and adds a file into the storage.
    ///
    /// # Errors
    /// If the file contents cannot be read.
    #[inline(always)]
    pub fn read_and_add_file(&mut self, path: PathID) -> Result<(), io::Error> {
        let file = InMemoryFile::new_from_path(path)?;
        self.add_file(path, file);

        Ok(())
    }

    /// Reads and adds a file into the storage.
    ///
    /// # Panics
    /// If the file contents cannot be read.
    #[inline(always)]
    pub fn read_and_add_file_or_panic(&mut self, path: PathID) {
        self.storage
            .insert(path, InMemoryFile::new_or_panic(path.resolve_or_panic()));
    }

    /// Adds a file into the storage if it does not exist.
    #[inline(always)]
    pub fn add_file_if_not_exists(&mut self, path: PathID, file: InMemoryFile) {
        if !self.storage.contains_key(&path) {
            self.add_file(path, file);
        }
    }

    /// Reads and adds a file into the storage if it does not exist.
    ///
    /// # Errors
    /// If the file contents cannot be read.
    #[inline(always)]
    pub fn read_and_add_file_if_not_exists(&mut self, path: PathID) -> Result<(), io::Error> {
        if !self.storage.contains_key(&path) {
            self.read_and_add_file(path)?;
        }

        Ok(())
    }

    /// Reads and adds a file into the storage if it does not exist.
    ///
    /// # Panics
    /// If the file contents cannot be read.
    #[inline(always)]
    #[must_use]
    pub fn read_and_add_file_if_not_exists_or_panic(&mut self, path: PathID) -> InMemoryFile {
        InMemoryFile::new_or_panic(path.resolve_or_panic())
    }

    /// Resolves a file from the storage by its path id.
    #[inline(always)]
    #[must_use]
    pub fn resolve_file(&self, path: PathID) -> Option<&InMemoryFile> {
        self.storage.get(&path)
    }
}
