//! Allows to resolve basic paths for a given package root path.
//!
//! See [`PackagePathResolver`] for more details.

use std::path::{Path, PathBuf};

/// Allows to resolve basic paths like config storage and build directories for a given
/// package path.
#[derive(Debug, Clone)]
pub struct PackagePathResolver<'path> {
    /// The path of the package root.
    pub root: &'path Path,
}

impl PackagePathResolver<'_> {
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
    pub fn src_directory(&self) -> PathBuf {
        self.root.join("src")
    }

    /// Returns the path of the build directory (used by the compiler).
    #[inline(always)]
    #[must_use]
    pub fn build_directory(&self) -> PathBuf {
        self.root.join("build")
    }
}
