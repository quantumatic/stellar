//! Diagnostics related to a name resolution process.

#![allow(clippy::needless_pass_by_value)]

use stellar_ast::ModuleItemKind;
use stellar_diagnostics::define_diagnostics;
use stellar_filesystem::location::Location;

define_diagnostics! {
    /// Diagnostic related to an item defined multiple times error.
    diagnostic(error) ItemDefinedMultipleTimes(
        self,
        name: String,
        first_definition_info: ModuleItemInfo,
        second_definition_info: ModuleItemInfo
    ) {
        code { "E005" }
        message { format!("the name `{}` is defined multiple times", self.name) }
        labels {
            primary self.first_definition_info.location => {
                format!("previous definition of {} `{}` is here",
                    self.first_definition_info.kind, self.name)
            },
            secondary self.second_definition_info.location => {
                format!("{} redefined here", self.name)
            }
        }
        notes {}
    }

    /// Diagnostic related to trying to import a package error.
    diagnostic(error) PackageImport(
        self,
        location: Location,
        package_name: String,
        package_name_location: Location
    ) {
        code { "E006" }
        message { format!("trying to import package `{}`", self.package_name) }
        labels {
            primary self.location => {
                format!("consider: removing this import")
            },
            secondary self.package_name_location => {
                format!("{} is a package, not a particular module", self.package_name)
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
        code { "E007" }
        message { format!("failed to resolve the package `{}`", self.package_name) }
        labels {
            primary self.location => {""}
        }
        notes {
            format!("consider: adding `{}` into the manifest file's [dependencies] section", self.package_name)
        }
    }

    /// Diagnostic, that occurs when the compiler tries to resolve a submodule of a
    /// particular package that doesn't exist.
    diagnostic(error) FailedToResolveModule(
        self,
        module_name_location: Location,
        module_name: String,
        package_name_location: Location,
        package_name: String
    ) {
        code { "E007" }
        message { format!("failed to resolve the module `{}`", self.module_name) }
        labels {
            primary self.module_name_location => {""},
            secondary self.package_name_location => {
                format!("package `{}` doesn't contain the submodule `{}`",
                    self.package_name, self.module_name)
            }
        }
        notes {}
    }

    /// Diagnostic, that occurs when the compiler tries to resolve a submodule of a particular package/module that doesn't exist.
    diagnostic(error) FailedToResolveModuleItem(
        self,
        module_name: String,
        module_name_location: Location,
        item_name: String,
        item_name_location: Location
    ) {
        code { "E007" }
        message { format!("failed to resolve the module item `{}`", self.item_name) }
        labels {
            primary self.item_name_location => {""},
            secondary self.module_name_location => {
                format!("module `{}` doesn't contain the item `{}`", self.module_name, self.item_name)
            }
        }
        notes {}
    }

    /// Diagnostic, that occurs when the compiler tries to resolve a module's item that is defined as private.
    diagnostic(error) FailedToResolvePrivateModuleItem(
        self,
        module_name: String,
        module_name_location: Location,
        item_name: String,
        item_name_location: Location
    ) {
        code { "E007" }
        message { format!("failed to resolve private module item `{}`", self.item_name) }
        labels {
            primary self.item_name_location => {""},
            secondary self.module_name_location => {
                format!("module `{}` contains the item `{}`, but it is defined as private", self.module_name, self.item_name)
            }
        }
        notes {}
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
        module_item_name: String,
        module_item_name_location: Location,
        name: String,
        name_location: Location
    ) {
        code { "E007" }
        message { format!("failed to resolve the name `{}`", self.name) }
        labels {
            primary self.name_location => {
                format!("cannot find the name `{}` in the namespace `{}`",
                    self.name, self.module_item_name)
            },
            secondary self.module_item_name_location => {
                format!("`{}` is not a module or a package, so it cannot directly contain individual names",
                    self.module_item_name)
            }
        }
        notes {
            "note: module items other than enums don't serve as namespaces"
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
        enum_item_name: String,
        enum_item_name_location: Location,
        name: String,
        name_location: Location
    ) {
        code { "E007" }
        message { format!("failed to resolve the name `{}`", self.name) }
        labels {
            primary self.name_location => {
                format!("cannot find the name `{}` in the namespace `{}`",
                    self.name, self.enum_item_name)
            },
            secondary self.enum_item_name_location => {
                format!("`{}` is not a module or a package, so it cannot directly contain individual names",
                    self.enum_item_name)
            }
        }
        notes {
            "note: enum items don't serve as namespaces"
        }
    }

    /// Diagnostic, that occurs when the compiler tries to resolve a name in a module scope.
    diagnostic(error) FailedToResolveName(
        self,
        name: String,
        location: Location
    ) {
        code { "E007" }
        message { format!("failed to resolve the name `{}`", self.name) }
        labels {
            primary self.location => {""}
        }
        notes {}
    }
}

/// An information about an item defined in the module.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ModuleItemInfo {
    /// Location of the item name.
    pub location: Location,

    /// Kind of the definition.
    pub kind: ModuleItemKind,
}
