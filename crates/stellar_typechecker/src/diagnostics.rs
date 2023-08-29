use itertools::Itertools;
use stellar_diagnostics::{
    define_diagnostics, diagnostic::Diagnostic, BuildDiagnostic, LocationExt,
};
use stellar_filesystem::location::Location;
use stellar_interner::PathID;
use stellar_name_resolution::NameBindingKind;

define_diagnostics! {
    diagnostic(error) DuplicateBound(
        self,
        first_bound_location: Location,
        second_bound_location: Location
    ) {
        code { "E008" }
        message { "duplicate bounds found" }
        labels {
            primary self.first_bound_location => { "first occurrence of the bound" },
            secondary self.second_bound_location => { "consider removing this bound" }
        }
        notes {}
    }
    diagnostic(error) ExpectedType(
        self,
        location: Location,
        name_binding_kind: NameBindingKind
    ) {
        code { "E009" }
        message { format!("expected type, got {}", self.name_binding_kind) }
        labels { primary self.location => {""} }
        notes {}
    }
    diagnostic(error) ExpectedInterface(
        self,
        location: Location,
        name_binding_kind: NameBindingKind
    ) {
        code { "E010" }
        message { format!("expected interface, got {}", self.name_binding_kind) }
        labels { primary self.location => {""} }
        notes {}
    }
    diagnostic(error) BoundsInTypeAlias(
        self,
        alias_name_location: Location,
        bounds_location: Location
    ) {
        code { "E011" }
        message { "found type bounds in type alias definition" }
        labels {
            primary self.alias_name_location => { "happened when processing this type alias" },
            secondary self.bounds_location => { "consider removing all the bounds" }
        }
        notes { "note: type aliases can't have bounds" }
    }
    diagnostic(error) DuplicateGenericParameter(
        self,
        previous_parameter_location: Location,
        current_parameter_location: Location,
        parameter_name: String
    ) {
        code { "E012" }
        message { format!("found duplicate generic parameter `{}`", self.parameter_name) }
        labels {
            primary self.previous_parameter_location => { "previous occurrence of the parameter" },
            secondary self.current_parameter_location => { "consider removing this parameter" }
        }
        notes {}
    }
    diagnostic(error) UnderscoreTypeInSignature(
        self,
        location: Location
    ) {
        code { "E013" }
        message { "found underscore type in signature" }
        labels {
            primary self.location => {""}
        }
        notes {}
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
//  --> test.stellar
// 1 | type A = B;
// 2 |      ^ expanding `A`, also requires expanding type alias `B`
// 3 | type B = C;
// 4 |      ^ expanding `B`, also requires expanding type alias `C`
// 5 | type C = A;
// 6 |      ^ type alias cycle found here: expanding `C`, also requires expanding type alias `A`
impl BuildDiagnostic for TypeAliasCycleFound {
    fn build(self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E013")
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
