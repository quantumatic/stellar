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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpectedNominalTypeInInherentImplDiagnostic {
    pub location: Location,
}

impl BuildDiagnostic for ExpectedNominalTypeInInherentImplDiagnostic {
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E009")
            .with_message("expected nominal type in inherent implementation")
            .with_labels(vec![self.location.to_primary_label()])
            .with_notes(vec![
                "either implement a trait or create a new wrapper type".to_owned(),
            ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimitiveTypeInInherentImplDiagnostic {
    pub location: Location,
}

impl BuildDiagnostic for PrimitiveTypeInInherentImplDiagnostic {
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E010")
            .with_message("cannot define inherent impl for primitive types")
            .with_labels(vec![self.location.to_primary_label()])
            .with_notes(vec!["consider using an extension trait instead".to_owned()])
    }
}
