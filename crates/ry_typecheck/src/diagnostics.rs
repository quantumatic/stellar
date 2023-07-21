use codespan_reporting::diagnostic::Diagnostic;
use ry_diagnostics::BuildDiagnostic;
use ry_filesystem::location::Location;
use ry_interner::PathID;

#[derive(Debug, Clone, PartialEq)]
pub struct DuplicateTraitBoundDiagnostic {
    pub first_bound_location: Location,
    pub second_bound_location: Location,
}

impl BuildDiagnostic for DuplicateTraitBoundDiagnostic {
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E007")
            .with_message("duplicate traits bounds found")
            .with_labels(vec![
                self.first_bound_location
                    .to_secondary_label()
                    .with_message("first occurrence of the trait bound"),
                self.second_bound_location
                    .to_primary_label()
                    .with_message("consider removing the trait bound"),
            ])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnnecessaryEqualityPredicateDiagnostic {
    pub generic_parameter_location: Location,
    pub type_location: Location,
}

impl BuildDiagnostic for UnnecessaryEqualityPredicateDiagnostic {
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E008")
            .with_message("unneccessary equality where predicate")
            .with_labels(vec![
                self.generic_parameter_location
                    .to_primary_label()
                    .with_message("consider using generic default value syntax"),
                self.type_location
                    .to_secondary_label()
                    .with_message("the type is not considered default for the generic!"),
            ])
    }
}
