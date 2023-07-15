//! Diagnostics related to a name resolution process.

#![allow(clippy::needless_pass_by_value)]

use codespan_reporting::diagnostic::Diagnostic;
use ry_ast::ModuleItemKind;
use ry_diagnostics::BuildSingleFileDiagnostic;
use ry_filesystem::location::Location;

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

impl BuildSingleFileDiagnostic for ItemDefinedMultipleTimesDiagnostic {
    #[inline]
    fn build(&self) -> Diagnostic<()> {
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

/// Diagnostic related to trying to import a project error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportingProjectDiagnostic {
    /// Location of the entire import module item.
    pub location: Location,

    /// Name of the project.
    pub project_name: String,

    /// Location of the project name in the import.
    pub project_name_location: Location,
}

impl ImportingProjectDiagnostic {
    /// Creates a new instance of [`ImportingProjectDiagnostic`].
    #[inline]
    #[must_use]
    pub fn new(
        location: Location,
        project_name: impl ToString,
        project_name_location: Location,
    ) -> Self {
        Self {
            location,
            project_name: project_name.to_string(),
            project_name_location,
        }
    }
}

impl BuildSingleFileDiagnostic for ImportingProjectDiagnostic {
    #[inline]
    fn build(&self) -> Diagnostic<()> {
        Diagnostic::error()
                .with_code("E005")
                .with_message("trying to import the project".to_owned())
                .with_labels(vec![
                    self.location.to_primary_label()
                        .with_message("consider removing this import"),
                    self.project_name_location.to_secondary_label().with_message(format!(
                        "{} is a project, not a particular module", self.project_name
                    )),
                ])
                .with_notes(
                    vec![
                        "note: importing a project is meaningless, you can still you its namespace without an import".to_owned(),
                    ]
                )
    }
}
