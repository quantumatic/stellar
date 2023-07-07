//! Allows to resolve basic paths for a given project root path.
//!
//! See [`ProjectPathResolver`] for more details.

use std::path::{Path, PathBuf};

/// Allows to resolve pathes by their file ID.
#[derive(Debug, Clone)]
pub struct PathResolver<'resolver> {
    /// The pathes inside the project.
    pub pathes: Vec<&'resolver Path>,
}

/// An ID used to refer to storage in [`ProjectPathResolver`].
pub type FileID = usize;

impl<'resolver> PathResolver<'resolver> {
    /// Creates a new project path resolver instance for the given project root path.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { pathes: vec![] }
    }

    /// Adds a path into the path resolver.
    #[inline]
    #[must_use]
    pub fn add_path(&mut self, path: &'resolver Path) -> FileID {
        self.pathes.push(path);
        self.pathes.len()
    }

    /// Resolves a path by its file ID.
    #[inline]
    #[must_use]
    pub fn resolve_path(&self, id: FileID) -> Option<&'resolver Path> {
        self.pathes.get(id - 1).copied()
    }

    /// Resolves a path by its file ID, and panics if the ID is invalid.
    #[inline]
    #[must_use]
    pub fn resolve_path_or_panic(&self, id: FileID) -> &'resolver Path {
        self.resolve_path(id).expect("Invalid file ID")
    }
}

/// Allows to resolve basic paths like config storage and build directories for a given
/// project path.
#[derive(Debug, Clone)]
pub struct ProjectPathResolver<'path> {
    /// The path of the project root.
    pub root: &'path Path,
}

impl ProjectPathResolver<'_> {
    /// Returns the path of the project README.
    #[inline]
    #[must_use]
    pub fn readme(&self) -> PathBuf {
        self.root.join("README.md")
    }

    /// Returns the path of the project configuration file.
    #[inline]
    #[must_use]
    pub fn config(&self) -> PathBuf {
        self.root.join("project.toml")
    }

    /// Returns the path of the project source directory.
    #[inline]
    #[must_use]
    pub fn src_directory(&self) -> PathBuf {
        self.root.join("src")
    }

    /// Returns the path of the build directory (used by the compiler).
    #[inline]
    #[must_use]
    pub fn build_directory(&self) -> PathBuf {
        self.root.join("build")
    }
}
