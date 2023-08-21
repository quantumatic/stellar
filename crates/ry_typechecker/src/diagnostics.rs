use itertools::Itertools;
use ry_diagnostics::diagnostic::Diagnostic;
use ry_diagnostics::{BuildDiagnostic, LocationExt};
use ry_filesystem::location::Location;
use ry_interner::PathID;
use ry_name_resolution::NameBindingKind;

#[derive(Debug, Clone, PartialEq)]
pub struct DuplicateTraitBoundDiagnostic {
    first_bound_location: Location,
    second_bound_location: Location,
}

impl DuplicateTraitBoundDiagnostic {
    #[inline]
    #[must_use]
    pub const fn new(first_bound_location: Location, second_bound_location: Location) -> Self {
        Self {
            first_bound_location,
            second_bound_location,
        }
    }
}

impl BuildDiagnostic for DuplicateTraitBoundDiagnostic {
    fn build(self) -> Diagnostic<PathID> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnnecessaryEqualityPredicateDiagnostic {
    pub generic_parameter_location: Location,
    pub type_location: Location,
}

impl UnnecessaryEqualityPredicateDiagnostic {
    #[inline]
    #[must_use]
    pub const fn new(generic_parameter_location: Location, type_location: Location) -> Self {
        Self {
            generic_parameter_location,
            type_location,
        }
    }
}

impl BuildDiagnostic for UnnecessaryEqualityPredicateDiagnostic {
    fn build(self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E008")
            .with_message("unnecessary equality where predicate")
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
    pub name_binding_kind: NameBindingKind,
}

impl ExpectedType {
    #[inline]
    #[must_use]
    pub fn new(location: Location, name_binding_kind: NameBindingKind) -> Self {
        Self {
            location,
            name_binding_kind,
        }
    }
}

impl BuildDiagnostic for ExpectedType {
    fn build(self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E009")
            .with_message(format!("expected type, got {}", self.name_binding_kind))
            .with_labels(vec![self.location.to_primary_label()])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpectedInterface {
    location: Location,
    name_binding_kind: NameBindingKind,
}

impl ExpectedInterface {
    #[inline]
    #[must_use]
    pub fn new(location: Location, name_binding_kind: NameBindingKind) -> Self {
        Self {
            location,
            name_binding_kind,
        }
    }
}

impl BuildDiagnostic for ExpectedInterface {
    fn build(self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E009")
            .with_message(format!(
                "expected interface, got {}",
                self.name_binding_kind
            ))
            .with_labels(vec![self.location.to_primary_label()])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundsInTypeAliasDiagnostic {
    alias_name_location: Location,
    bounds_location: Location,
}

impl BoundsInTypeAliasDiagnostic {
    #[inline]
    #[must_use]
    pub const fn new(alias_name_location: Location, bounds_location: Location) -> Self {
        Self {
            alias_name_location,
            bounds_location,
        }
    }
}

impl BuildDiagnostic for BoundsInTypeAliasDiagnostic {
    fn build(self) -> Diagnostic<PathID> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateGenericParameterDiagnostic {
    previous_parameter_location: Location,
    current_parameter_location: Location,
    parameter_name: String,
}

impl DuplicateGenericParameterDiagnostic {
    #[inline]
    #[must_use]
    pub fn new(
        previous_parameter_location: Location,
        current_parameter_location: Location,
        parameter_name: impl ToString,
    ) -> Self {
        Self {
            previous_parameter_location,
            current_parameter_location,
            parameter_name: parameter_name.to_string(),
        }
    }
}

impl BuildDiagnostic for DuplicateGenericParameterDiagnostic {
    fn build(self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E011")
            .with_message(format!(
                "found duplicate generic parameter `{}`",
                self.parameter_name
            ))
            .with_labels(vec![self.previous_parameter_location.to_secondary_label()])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAliasCycleFound {
    pub stack_trace: Vec<TypeAliasCycleStackTraceItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAliasCycleStackTraceItem {
    pub alias_name: String,
    pub alias_location: Location,
}

// error[E012]: type alias cycle found
//  --> test.ry
// 1 | type A = B;
// 2 |      ^ expanding `A`, also requires expanding type alias `B`
// 3 | type B = C;
// 4 |      ^ expanding `B`, also requires expanding type alias `C`
// 5 | type C = A;
// 6 |      ^ type alias cycle found here: expanding `C`, also requires expanding type alias `A`
impl BuildDiagnostic for TypeAliasCycleFound {
    fn build(self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E012")
            .with_message(format!(
            "type alias cycle found here: expanding `{}`, also requires expanding type alias `{}`",
            self.stack_trace.last().unwrap().alias_name,
            self.stack_trace.first().unwrap().alias_name
        ))
            .with_labels(
                self.stack_trace
                    .split_last()
                    .unwrap()
                    .1
                    .iter()
                    .tuple_windows()
                    .map(|(first, second)| {
                        first
                            .alias_location
                            .to_secondary_label()
                            .with_message(format!(
                                "expanding `{}`, also requires expanding type alias `{}`",
                                first.alias_name, second.alias_name
                            ))
                    })
                    .collect::<Vec<_>>(),
            )
    }
}
