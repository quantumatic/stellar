//! Defines a [`InMemoryFileStorage`], to avoid rereading files in some situtations.

use std::io;

use stellar_fx_hash::FxHashMap;
use stellar_interner::PathId;

use crate::in_memory_file::InMemoryFile;

/// # In-memory file storage.
///
/// The storage can be used for example when emitting some diagnostics, to
/// avoid rereading the same file multiple times.
///
/// # Notes
///
/// The storage is instantiated only 2 times throughout the compilation:
///
/// - The first time happens when the compiler needs to parse files which requires
/// reads.
/// - The second time happens when diagnostics needs to be emitted. It being
/// separate from the first one prevents storing the whole project storage with
/// files from the whole dependency tree throughout the compilation.
///
/// The storage is represented as a simple hashmap of type [`FxHashMap<PathId, InMemoryFile>`]
/// because no smart interning mechanisms are required.
#[derive(Debug, Clone, Default)]
pub struct InMemoryFileStorage(FxHashMap<PathId, InMemoryFile>);

impl InMemoryFileStorage {
    /// Creates an empty storage.
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a file into the storage.
    #[inline(always)]
    pub fn add_file(&mut self, path: PathId, file: InMemoryFile) {
        self.0.insert(path, file);
    }

    /// Reads and adds a file into the storage.
    ///
    /// # Errors
    /// If the file contents cannot be read.
    #[inline(always)]
    pub fn read_and_add_file(&mut self, path: PathId) -> Result<(), io::Error> {
        let file = InMemoryFile::new_from_path(path)?;
        self.add_file(path, file);

        Ok(())
    }

    /// Reads and adds a file into the storage.
    ///
    /// # Panics
    /// If the file contents cannot be read.
    #[inline(always)]
    pub fn read_and_add_file_or_panic(&mut self, path: PathId) {
        self.0
            .insert(path, InMemoryFile::new_or_panic(path.resolve_or_panic()));
    }

    /// Adds a file into the storage if it does not exist.
    #[inline(always)]
    pub fn add_file_if_not_exists(&mut self, path: PathId, file: InMemoryFile) {
        if !self.0.contains_key(&path) {
            self.add_file(path, file);
        }
    }

    /// Reads and adds a file into the storage if it does not exist.
    ///
    /// # Errors
    /// If the file contents cannot be read.
    #[inline(always)]
    pub fn read_and_add_file_if_not_exists(&mut self, path: PathId) -> Result<(), io::Error> {
        if !self.0.contains_key(&path) {
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
    pub fn read_and_add_file_if_not_exists_or_panic(&mut self, path: PathId) -> InMemoryFile {
        InMemoryFile::new_or_panic(path.resolve_or_panic())
    }

    /// Resolves a file from the storage by its path id.
    #[inline(always)]
    #[must_use]
    pub fn resolve_file(&self, path: PathId) -> Option<&InMemoryFile> {
        self.0.get(&path)
    }
}
