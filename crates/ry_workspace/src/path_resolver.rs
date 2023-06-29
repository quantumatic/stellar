//! Allows to resolve basic paths for a given project root path.
//!
//! See [`ProjectPathResolver`] for more details.

use std::path::{Path, PathBuf};

/// Allows to resolve basic paths like config files and build directories for a given
/// project path.
#[derive(Debug, Clone)]
pub struct ProjectPathResolver<'workspace> {
    /// The project root path.
    root: &'workspace Path,
}

impl<'workspace> ProjectPathResolver<'workspace> {
    /// Creates a new project path resolver instance for the given project root path.
    #[inline]
    #[must_use]
    pub const fn new(root: &'workspace Path) -> Self {
        Self { root }
    }

    /// Returns the project root path.
    #[inline]
    #[must_use]
    pub const fn root(&self) -> &'workspace Path {
        self.root
    }

    /// Returns the path of the project configuration file.
    #[inline]
    #[must_use]
    pub fn config(&self) -> PathBuf {
        self.root.join("project.toml")
    }

    /// Returns the path of the project README.
    #[inline]
    #[must_use]
    pub fn readme(&self) -> PathBuf {
        self.root.join("README.md")
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

    /// Returns the path of the build directory for a given project name. So that
    /// if we have a project called `A`, that has dependencies being: `B` and `C`,
    /// then the build directory would look like:
    ///
    /// ```txt
    /// A/build
    ///     |__ A
    ///     |__ B
    ///     |__ C
    /// ```
    ///
    /// and the function returns `A/build/A`, `A/build/B` and `A/build/C` for all
    /// three project names: `A`, `B`, `C` in the project root `A`.
    #[inline]
    #[must_use]
    pub fn project_build_directory<S>(&self, project_name: S) -> PathBuf
    where
        S: AsRef<str>,
    {
        self.build_directory().join(project_name.as_ref())
    }
}
