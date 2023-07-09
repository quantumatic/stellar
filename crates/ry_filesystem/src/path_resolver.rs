//! Allows to resolve basic paths for a given project root path.
//!
//! See [`ProjectPathResolver`] for more details.

use std::path::{Path, PathBuf};

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
    pub fn manifest(&self) -> PathBuf {
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
