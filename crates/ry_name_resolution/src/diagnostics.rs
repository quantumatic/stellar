//! Diagnostics related to a name resolution process.

#![allow(clippy::needless_pass_by_value)]

use codespan_reporting::diagnostic::Diagnostic;
use ry_ast::ModuleItemKind;
use ry_diagnostics::BuildDiagnostic;
use ry_filesystem::{location::Location, path_interner::PathID};

/// An information about an item defined in the module.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct DefinitionInfo {
    /// Location of the item name.
    pub location: Location,

    /// Kind of the definition.
    pub kind: ModuleItemKind,
}

/// Diagnostic related to an item defined multiple times error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemDefinedMultipleTimesDiagnostic {
    /// Name of the item.
    pub name: String,

    /// First item definition.
    pub first_definition: DefinitionInfo,

    /// Second item definition.
    pub second_definition: DefinitionInfo,
}

impl ItemDefinedMultipleTimesDiagnostic {
    /// Creates a new instance of [`ItemDefinedMultipleTimesDiagnostic`].
    #[inline]
    #[must_use]
    pub fn new(
        name: impl ToString,
        first_definition: DefinitionInfo,
        second_definition: DefinitionInfo,
    ) -> Self {
        Self {
            name: name.to_string(),
            first_definition,
            second_definition,
        }
    }
}

impl BuildDiagnostic for ItemDefinedMultipleTimesDiagnostic {
    #[inline]
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E005")
            .with_message(format!(
                "the name `{}` is defined multiple times",
                self.name
            ))
            .with_labels(vec![
                self.first_definition
                    .location
                    .to_primary_label()
                    .with_message(format!(
                        "previous definition of the {} `{}` here",
                        self.first_definition.kind, self.name
                    )),
                self.second_definition
                    .location
                    .to_primary_label()
                    .with_message(format!("{} redefined here", self.name)),
            ])
    }
}

/// Diagnostic related to trying to import a package error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportingPackageDiagnostic {
    /// Location of the entire import module item.
    pub location: Location,

    /// Name of the package.
    pub package_name: String,

    /// Location of the package name in the import.
    pub package_name_location: Location,
}

impl ImportingPackageDiagnostic {
    /// Creates a new instance of [`ImportingPackageDiagnostic`].
    #[inline]
    #[must_use]
    pub fn new(
        location: Location,
        package_name: impl ToString,
        package_name_location: Location,
    ) -> Self {
        Self {
            location,
            package_name: package_name.to_string(),
            package_name_location,
        }
    }
}

impl BuildDiagnostic for ImportingPackageDiagnostic {
    #[inline]
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E005")
            .with_message("trying to import a package".to_owned())
            .with_labels(vec![
                self.location.to_primary_label()
                    .with_message("consider removing this import"),
                self.package_name_location.to_secondary_label().with_message(format!(
                    "{} is a package, not a particular module", self.package_name
                )),
            ])
            .with_notes(
                vec![
                    "note: importing a package is meaningless, you can still you its namespace without an import".to_owned(),
                ]
            )
    }
}
