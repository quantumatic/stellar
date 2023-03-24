use codespan_reporting::{
    diagnostic::Diagnostic,
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};

/// Stores basic `codespan_reporting` structs for reporting errors.
pub struct ReporterState<'f> {
    pub writer: StandardStream,
    pub config: Config,
    pub files: SimpleFiles<&'f str, &'f str>,
}

impl<'f> ReporterState<'f> {
    pub fn emit_global_error(&self, msg: &str) {
        term::emit(
            &mut self.writer.lock(),
            &self.config,
            &self.files,
            &Diagnostic::error().with_message(msg),
        )
        .expect("emit_global_diagnostic() failed");
    }
}

impl Default for ReporterState<'_> {
    fn default() -> Self {
        Self {
            writer: StandardStream::stderr(ColorChoice::Always),
            config: codespan_reporting::term::Config::default(),
            files: SimpleFiles::new(),
        }
    }
}

pub trait Reporter<'source> {
    fn emit_diagnostic(
        &self,
        reporter: &ReporterState,
        files: &SimpleFiles<&str, &str>,
        file_id: usize,
    ) {
        term::emit(
            &mut reporter.writer.lock(),
            &reporter.config,
            files,
            &self.build_diagnostic(file_id),
        )
        .expect("emit_diagnostic() failed")
    }

    fn build_diagnostic(&self, file_id: usize) -> Diagnostic<usize>;
}
