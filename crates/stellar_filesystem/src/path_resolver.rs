//! Allows to resolve basic paths for a given package root path.
//!
//! See [`PackagePathResolver`] for more details.

use std::path::{Path, PathBuf};

/// Allows to resolve basic paths like config storage and build directories for a given
/// package path.
#[derive(Debug, Clone)]
pub struct PackagePathResolver<'p> {
    /// The path of the package root.
    root: &'p Path,
}

impl<'p> PackagePathResolver<'p> {
    /// Constructs a new [`PackagePathResolver`].
    #[inline(always)]
    #[must_use]
    pub const fn new(root: &'p Path) -> Self {
        Self { root }
    }

    /// Returns the path of the package root.
    #[inline(always)]
    #[must_use]
    pub const fn root(&self) -> &'p Path {
        self.root
    }

    /// Returns the path of the package README.
    #[inline(always)]
    #[must_use]
    pub fn readme(&self) -> PathBuf {
        self.root.join("README.md")
    }

    /// Returns the path of the package configuration file.
    #[inline(always)]
    #[must_use]
    pub fn manifest(&self) -> PathBuf {
        self.root.join("package.toml")
    }

    /// Returns the path of the package source directory.
    #[inline(always)]
    #[must_use]
    pub fn source_directory(&self) -> PathBuf {
        self.root.join("src")
    }

    /// Returns the path of the build directory (used by the compiler).
    #[inline(always)]
    #[must_use]
    pub fn build_directory(&self) -> PathBuf {
        self.root.join("build")
    }
}
