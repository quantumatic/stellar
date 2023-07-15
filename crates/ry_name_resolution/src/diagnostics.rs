//! Diagnostics related to a name resolution process.

use ry_ast::ModuleItemKind;
use ry_diagnostics::{BuildDiagnostic, Diagnostic};
use ry_filesystem::location::Location;

/// An information about an item defined in the module.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct DefinitionInfo {
    /// Location of the item name.
    pub location: Location,

    /// Kind of the definition.
    pub kind: ModuleItemKind,
}

/// Diagnostic encountered during name resolution.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NameResolutionDiagnostic {
    /// The name is defined multiple times.
    ItemDefinedMultipleTimes {
        /// Name of the item.
        name: String,

        /// First definition of the item.
        first_definition: DefinitionInfo,

        /// Second definition of the item.
        second_definition: DefinitionInfo,
    },

    /// Trying to import the project.
    ImportingProject {
        /// Location of the entire import module item.
        location: Location,

        /// Name of the project.
        project_name: String,

        /// Location of the project name in the import.
        project_name_location: Location,
    },
}

impl BuildDiagnostic for NameResolutionDiagnostic {
    fn build(&self) -> Diagnostic {
        match self {
            Self::ItemDefinedMultipleTimes {
                name,
                first_definition,
                second_definition,
            } => Diagnostic::error()
                .with_code("E004")
                .with_message(format!("the name `{name}` is defined multiple times"))
                .with_labels(vec![
                    first_definition
                        .location
                        .to_primary_label()
                        .with_message(format!(
                            "previous definition of the {} `{name}` here",
                            first_definition.kind
                        )),
                    second_definition
                        .location
                        .to_primary_label()
                        .with_message(format!("{name} redefined here")),
                ]),
            Self::ImportingProject {
                location,
                project_name,
                project_name_location,
            } => Diagnostic::error()
                .with_code("E005")
                .with_message("trying to import the project".to_owned())
                .with_labels(vec![
                    location.to_primary_label()
                        .with_message("consider removing this import"),
                    project_name_location.to_secondary_label().with_message(format!(
                        "{project_name} is a project, not a particular module"
                    )),
                ])
                .with_notes(
                    vec![
                        "note: importing a project is meaningless, you can still you its namespace without an import".to_owned(),
                    ]
                ),
        }
    }
}
