use itertools::Itertools;
use stellar_ast::{IdentifierAST, ModuleItemKind};
use stellar_diagnostics::{
    define_diagnostics,
    diagnostic::{Diagnostic, Label},
    BuildDiagnostic,
};
use stellar_english_commons::pluralize::PluralizeExt;
use stellar_filesystem::location::Location;
use stellar_interner::PathId;

define_diagnostics! {
    /// Diagnostic related to an item defined multiple times error.
    diagnostic(error) ItemDefinedMultipleTimes(
        self,
        name: String,
        first_definition_location: Location,
        second_definition_location: Location
    ) {
        code { "E005" }
        message { format!("the name `{}` is defined multiple times", self.name) }
        labels {
            primary {
                self.first_definition_location => format!("previous definition of `{}` is here", self.name)
            }
            secondary {
                self.second_definition_location => format!("{} redefined here", self.name)
            }
        }
    }

    diagnostic(error) EnumItemDefinedMultipleTimes(
        self,
        enum_name: String,
        item_name: String,
        first_definition_location: Location,
        second_definition_location: Location
    ) {
        code { "E006" }
        message { format!("duplicate definition of the enum item `{}` in `{}`", self.item_name, self.enum_name) }
        labels {
            primary {
                self.first_definition_location => format!("first definition of `{}`", self.item_name)
            }
            secondary { self.second_definition_location => format!("second, conflicting definition of `{}`", self.item_name) }
        }
    }

    /// Diagnostic related to trying to import a package error.
    diagnostic(error) PackageImport(
        self,
        location: Location,
        package_name: IdentifierAST
    ) {
        code { "E007" }
        message { format!("trying to import package `{}`", self.package_name.id) }
        labels {
            primary { self.location => format!("help: remove this import")
            }
            secondary {
                self.package_name.location => format!("{} is a package, not a particular module", self.package_name.id)
            }
        }
        notes {
            "note: importing a package is meaningless, you can still use its namespace without an import"
        }
    }

    /// Diagnostic, that occurs when the compiler tries to resolve a package that
    /// doesn't exist.
    diagnostic(error) FailedToResolvePackage(
        self,
        location: Location,
        package_name: String
    ) {
        code { "E008" }
        message { format!("failed to resolve the package `{}`", self.package_name) }
        labels {
            primary { self.location }
        }
        notes {
            format!("help: add `{}` into the manifest file's [dependencies] section", self.package_name)
        }
    }

    /// Diagnostic, that occurs when the compiler tries to resolve a submodule of a particular package/module that doesn't exist.
    diagnostic(error) FailedToResolveNameInModule(
        self,
        module_name: String,
        module_name_location: Location,
        item_name: String,
        item_name_location: Location
    ) {
        code { "E008" }
        message { format!("failed to resolve the module item `{}`", self.item_name) }
        labels {
            primary { self.item_name_location }
            secondary {
                self.module_name_location => format!("module `{}` doesn't contain the item `{}`", self.module_name, self.item_name)
            }
        }

    }

    /// Diagnostic, that occurs when the compiler tries to resolve a module's item that is defined as private.
    diagnostic(error) FailedToResolvePrivateModuleItem(
        self,
        module_name: String,
        module_name_location: Location,
        item_name: String,
        item_name_location: Location
    ) {
        code { "E008" }
        message { format!("failed to resolve private module item `{}`", self.item_name) }
        labels {
            primary { self.item_name_location }
            secondary {
                self.module_name_location => format!("module `{}` contains the item `{}`, but it is defined as private", self.module_name, self.item_name)
            }
        }
    }

    /// Diagnostic, that appears when you try to access a name in a namespace of
    /// a module item except an enum, for example:
    ///
    /// ```txt
    /// pub fn foo() -> A.B {} // A is a struct
    ///                   ^ wrong
    /// ```
    diagnostic(error) ModuleItemsExceptEnumsDoNotServeAsNamespaces(
        self,
        module_item_name: IdentifierAST,
        module_item_kind: ModuleItemKind,
        name: IdentifierAST
    ) {
        code { "E008" }
        message { format!("failed to resolve the name `{}`", self.name.id) }
        labels {
            primary {
                self.name.location => format!("cannot find the name `{}` in `{}`",
                    self.name.id, self.module_item_name.id)
            }
            secondary {
                self.module_item_name.location => format!("`{}` is not a module or an enum, so it cannot directly contain individual names",
                    self.module_item_name.id)
            }
        }
        notes {
            format!("note: {} don't serve as namespaces", self.module_item_kind.pluralize())
        }
    }

    /// Diagnostic, that appears when you try to access a name in a namespace of
    /// a enum item, for example:
    ///
    /// ```txt
    /// pub fn foo() -> Option.None.A {} // Option.None is an enum item
    ///                             ^ wrong
    /// ```
    diagnostic(error) EnumItemsDoNotServeAsNamespaces(
        self,
        enum_item_name: IdentifierAST,
        name: IdentifierAST
    ) {
        code { "E008" }
        message { format!("failed to resolve the name `{}`", self.name.id) }
        labels {
            primary {
                self.name.location => format!("cannot find the name `{}` in the namespace `{}`",
                    self.name.id, self.enum_item_name.id)
            }
            secondary {
                self.enum_item_name.location => format!("`{}` is not a module or an enum, so it cannot directly contain individual names",
                    self.enum_item_name.id)
            }
        }
        notes {
            "note: enum items don't serve as namespaces"
        }
    }

    diagnostic(error) FailedToResolveEnumItem(
        self,
        enum_name: IdentifierAST,
        enum_item_name: IdentifierAST
    ) {
        code { "E008" }
        message { format!("failed to resolve enum item `{}`", self.enum_item_name.id) }
        labels {
            primary {
                self.enum_item_name.location => format!("cannot find the name `{}` in the definition of enum `{}`",
                    self.enum_item_name.id, self.enum_name.id)
            }
        }
    }

    /// Diagnostic, that occurs when the compiler tries to resolve a name in a module scope.
    diagnostic(error) FailedToResolveName(
        self,
        name: IdentifierAST
    ) {
        code { "E008" }
        message { format!("failed to resolve the name `{}`", self.name.id) }
        labels {
            primary { self.name.location }
        }
    }

    diagnostic(error) UnderscoreTypeInSignature(
        self,
        item_name: IdentifierAST,
        location: Location
    ) {
        code { "E009" }
        message { format!("found underscore type in signature of `{}`", self.item_name.id) }
        labels {
            primary { self.location }
            secondary { self.item_name.location
                => format!("happened when analyzing `{}`", self.item_name.id)}
        }
        notes {
            "note: types cannot be inferred in signatures, because of explicitness."
        }
    }
}

pub struct CycleDetectedWhenComputingSignatureOf {
    pub backtrace: Vec<IdentifierAST>,
}

impl CycleDetectedWhenComputingSignatureOf {
    pub fn new(backtrace: Vec<IdentifierAST>) -> Self {
        Self { backtrace }
    }
}

impl BuildDiagnostic for CycleDetectedWhenComputingSignatureOf {
    fn build(self) -> Diagnostic {
        Diagnostic::error()
            .with_message(format!(
                "cycle detected when computing signature of {}",
                self.backtrace.first().unwrap().id
            ))
            .with_code("E009")
            .with_labels(
                self.backtrace
                    .iter()
                    .tuple_windows()
                    .enumerate()
                    .map(|(idx, (a, b))| {
                        Label::secondary(a.location).with_message(format!(
                            "computing signature of {} requires also to compute signature of {}",
                            a.id, b.id
                        ))
                    })
                    .collect::<Vec<_>>(),
            )
    }
}
