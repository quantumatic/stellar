use codespan_reporting::{diagnostic::Diagnostic, files::SimpleFiles};
use error::StaticAnalysisError;

mod error;
mod warning;

use ry_ast::ProgramUnit;
use ry_report::Reporter;
use warning::StaticAnalysisWarning;

pub enum StaticAnalysisOutputUnit {
    Warning(StaticAnalysisWarning),
    Error(StaticAnalysisError),
}

impl<'source> Reporter<'source> for StaticAnalysisOutputUnit {
    fn build_diagnostic(&self, file_id: usize) -> Diagnostic<usize> {
        match self {
            Self::Warning(w) => w.build_diagnostic(file_id),
            Self::Error(e) => e.build_diagnostic(file_id),
        }
    }
}

pub type StaticAnalysisOutput = Vec<(usize, StaticAnalysisOutputUnit)>;

pub struct StaticAnalyzer<'a> {
    current_file_id: usize,
    initial_ast: ProgramUnit,

    #[allow(dead_code)]
    files: &'a SimpleFiles<&'a str, &'a str>,

    pub output: StaticAnalysisOutput,
}

impl<'a> StaticAnalyzer<'a> {
    pub fn new(
        initial_file_id: usize,
        initial_ast: ProgramUnit,
        files: &'a SimpleFiles<&'a str, &'a str>,
    ) -> Self {
        Self {
            current_file_id: initial_file_id,
            initial_ast,
            files,
            output: vec![],
        }
    }

    pub fn analyze(&mut self) {
        for tlstmt in self.initial_ast.top_level_statements.iter() {
            match &tlstmt.1 {
                ry_ast::TopLevelStatement::Import(filename) => {
                    self.output.push((
                        self.current_file_id,
                        StaticAnalysisOutputUnit::Warning(
                            StaticAnalysisWarning::ImportAfterFirstTopLevelStatement(
                                filename.path.span,
                            ),
                        ),
                    ));
                }
                ry_ast::TopLevelStatement::TraitDecl(t) => {
                    if let Some(p) = t.public {
                        self.output.push((
                            self.current_file_id,
                            StaticAnalysisOutputUnit::Warning(
                                StaticAnalysisWarning::UnnecessaryVisibilityQualifier(p),
                            ),
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    pub fn add(&mut self, output: StaticAnalysisOutputUnit) {
        self.output.push((self.current_file_id, output));
    }
}
