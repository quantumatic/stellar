use ry_ast::ModuleItemKind;
use ry_diagnostics::{BuildDiagnostic, Diagnostic};
use ry_filesystem::span::Span;

pub struct DefinitionInfo {
    pub span: Span,
    pub kind: ModuleItemKind,
}

pub enum NameResolutionDiagnostics {
    OverwrittingModuleItem {
        name: String,
        first_definition: DefinitionInfo,
        second_definition: DefinitionInfo,
    },
    ImportingProject {
        span: Span,
        project_name: String,
        project_name_span: Span,
    },
}

impl BuildDiagnostic for NameResolutionDiagnostics {
    fn build(&self) -> Diagnostic {
        match self {
            NameResolutionDiagnostics::OverwrittingModuleItem {
                name,
                first_definition,
                second_definition,
            } => Diagnostic::error()
                .with_code("E004")
                .with_message(format!("the name `{name}` is defined multiple times"))
                .with_labels(vec![
                    first_definition
                        .span
                        .to_primary_label()
                        .with_message(format!(
                            "previous definition of the {} `{name}` here",
                            first_definition.kind
                        )),
                    second_definition
                        .span
                        .to_primary_label()
                        .with_message(format!("{name} redefined here")),
                ]),
            Self::ImportingProject {
                span,
                project_name,
                project_name_span,
            } => Diagnostic::error()
                .with_code("E005")
                .with_message(format!("trying to import the project"))
                .with_labels(vec![
                    span.to_primary_label()
                        .with_message("consider removing this import"),
                    project_name_span.to_secondary_label().with_message(format!(
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
