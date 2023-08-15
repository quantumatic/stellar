use ry_diagnostics::diagnostic::Diagnostic;
use ry_diagnostics::{BuildDiagnostic, LocationExt};
use ry_filesystem::location::Location;
use ry_interner::PathID;

use crate::BindingKind;

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
pub struct ExpectedType {
    pub location: Location,
    pub binding_kind: BindingKind,
}

impl BuildDiagnostic for ExpectedType {
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E009")
            .with_message(format!("expected type, got {}", self.binding_kind))
            .with_labels(vec![self.location.to_primary_label()])
    }
}

pub struct ExpectedInterface {
    pub location: Location,
    pub binding_kind: BindingKind,
}

impl BuildDiagnostic for ExpectedInterface {
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E009")
            .with_message(format!("expected interface, got {}", self.binding_kind))
            .with_labels(vec![self.location.to_primary_label()])
    }
}

pub struct BoundsInTypeAliasDiagnostic {
    pub alias_name_location: Location,
    pub bounds_location: Location,
}

impl BuildDiagnostic for BoundsInTypeAliasDiagnostic {
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E010")
            .with_message("found type bounds in type alias definition")
            .with_labels(vec![
                self.alias_name_location
                    .to_primary_label()
                    .with_message("happened when processing this type alias"),
                self.bounds_location
                    .to_primary_label()
                    .with_message("consider removing all the bounds"),
            ])
            .with_notes(vec!["note: type aliases can't have bounds".to_owned()])
    }
}
