//! Defines [`InMemoryFileStorage`], to avoid rereading files in some situtations.

use std::io;

use codespan_reporting::files::{self, Files};
use ry_fx_hash::FxHashMap;

use crate::{
    in_memory_file::InMemoryFile,
    path_storage::{PathID, PathStorage},
};

/// In memory file storage. The storage can be used for example when emitting
/// some diagnostics, to avoid rereading the same file multiple times.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InMemoryFileStorage<'path_storage> {
    path_storage: &'path_storage PathStorage,
    storage: FxHashMap<PathID, InMemoryFile>,
}

impl<'path_storage> InMemoryFileStorage<'path_storage> {
    /// Creates an empty storage.
    #[inline]
    #[must_use]
    pub fn new(path_storage: &'path_storage PathStorage) -> Self {
        Self {
            path_storage,
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
        let file = InMemoryFile::new_from_path_id(self.path_storage, path_id)?;
        self.add_file(path_id, file);

        Ok(())
    }

    /// Resolves a file from the storage by its path id.
    #[inline]
    #[must_use]
    pub fn resolve_file(&self, path_id: PathID) -> Option<&InMemoryFile> {
        self.storage.get(&path_id)
    }
}

impl<'a> Files<'a> for InMemoryFileStorage<'a> {
    type FileId = PathID;

    type Name = String;
    type Source = &'a str;

    #[inline]
    fn name(&'a self, id: PathID) -> Result<Self::Name, files::Error> {
        self.resolve_file(id)
            .map(|file| file.path.display().to_string())
            .ok_or(files::Error::FileMissing)
    }

    #[inline]
    fn source(&'a self, id: PathID) -> Result<Self::Source, files::Error> {
        self.resolve_file(id)
            .map(|file| file.source.as_str())
            .ok_or(files::Error::FileMissing)
    }

    #[inline]
    fn line_index(&'a self, id: PathID, byte_index: usize) -> Result<usize, files::Error> {
        self.resolve_file(id)
            .ok_or(files::Error::FileMissing)
            .and_then(|file| file.line_index((), byte_index))
    }

    #[inline]
    fn line_range(
        &'a self,
        id: PathID,
        line_index: usize,
    ) -> Result<std::ops::Range<usize>, files::Error> {
        self.resolve_file(id)
            .ok_or(files::Error::FileMissing)
            .and_then(|file| file.line_range((), line_index))
    }
}
