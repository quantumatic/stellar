use codespan_reporting::diagnostic::{Diagnostic, Label};
use ry_ast::location::Span;
use ry_report::Reporter;
use thiserror::Error;

#[derive(Error, Copy, Clone, Debug, PartialEq, Eq)]
pub enum StaticAnalysisWarning {
    #[error("unnecessary visibility qualifier in {0}")]
    UnnecessaryVisibilityQualifier(Span),
    #[error("import after first top level statement in {0}")]
    ImportAfterFirstTopLevelStatement(Span),
}

impl<'source> Reporter<'source> for StaticAnalysisWarning {
    fn build_diagnostic(&self, file_id: usize) -> Diagnostic<usize> {
        match self {
            Self::ImportAfterFirstTopLevelStatement(span) => Diagnostic::warning()
                .with_code("W001")
                .with_message("found import after another top level statement")
                .with_labels(vec![Label::primary(file_id, *span)
                    .with_message("consider placing it at the beginning of the file")])
                .with_notes(vec![
                    "`#[warn(imports_after_fst_tlstmt)]` on by default".to_owned()
                ]),
            Self::UnnecessaryVisibilityQualifier(span) => Diagnostic::warning()
                .with_code("W002")
                .with_message("unnecessary visibility qualifier found")
                .with_labels(vec![Label::primary(file_id, *span)
                    .with_message("consider removing `pub`, because it's implied")])
                .with_notes(vec![
                    "`#[warn(unnecessary_visibility_qualifier)]` on by default`".to_owned(),
                ]),
            // _ => todo!(),
        }
    }
}
