use ry_diagnostics::diagnostic::Diagnostic;
use ry_diagnostics::{BuildDiagnostic, LocationExt};
use ry_filesystem::location::Location;
use ry_interner::PathID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnnecessaryParenthesesInPatternDiagnostic {
    pub location: Location,
}

impl BuildDiagnostic for UnnecessaryParenthesesInPatternDiagnostic {
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::warning()
            .with_code("W000")
            .with_message("unnecessary parentheses in the pattern")
            .with_labels(vec![
                self.location.start_byte_location().to_primary_label(),
                self.location
                    .end_byte_location()
                    .to_primary_label()
                    .with_message("consider removing these parentheses"),
            ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnnecessaryParenthesizedExpression {
    pub location: Location,
}

impl BuildDiagnostic for UnnecessaryParenthesizedExpression {
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::warning()
            .with_code("W001")
            .with_message("unnecessary parenthesized expression")
            .with_labels(vec![
                self.location.start_byte_location().to_primary_label(),
                self.location
                    .end_byte_location()
                    .to_primary_label()
                    .with_message("consider removing these parentheses"),
            ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnnecessaryParenthesizedType {
    pub location: Location,
}

impl BuildDiagnostic for UnnecessaryParenthesizedType {
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::warning()
            .with_code("W002")
            .with_message("unnecessary parenthesized type")
            .with_labels(vec![
                self.location.start_byte_location().to_primary_label(),
                self.location
                    .end_byte_location()
                    .to_primary_label()
                    .with_message("consider removing these parentheses"),
            ])
    }
}
