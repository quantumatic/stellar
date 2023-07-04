//! Defines a [`ProjectInterface`] to work with Ry projects.

use ry_interner::Symbol;
use std::path;

/// An interface that allows to interact with Ry projects.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ProjectInterface<'path> {
    /// Canonical path to the project.
    path: &'path path::Path,

    /// Data about the project.
    metadata: ProjectMetadata,
}

/// All information about a project that is stored in its configuration file.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ProjectMetadata {
    /// Names (interned symbols) of other projects that the project depends on.
    depedencies: Vec<Symbol>,

    // TODO
}
