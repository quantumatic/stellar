//! Provides various utility functions for working with files.

use std::fs::{metadata, File};
use std::io;

/// Creates a file with a unique name. For example:
///
/// If `hir.json` already exists, and you invoke
/// `make_unique_file("hir", "json")`, then it will create
/// a file with the path: `hir (2).json`.
#[inline]
pub fn make_unique_file(
    name: impl AsRef<str>,
    extension: impl AsRef<str>,
) -> (String, Result<File, io::Error>) {
    let name = name.as_ref();
    let extension = extension.as_ref();

    let mut path = format!("{name}.{extension}");
    let mut idx = 1;

    while metadata(path.clone()).is_ok() {
        path = format!("{name} ({idx}).{extension}");
        idx += 1;
    }

    (path.clone(), File::create(path))
}
