#![allow(warnings)]

use stellar_manifest::{TomlManifest, TomlPackage};

pub struct DependencyCycleError {
    traceback: Vec<TomlPackage>,
}
