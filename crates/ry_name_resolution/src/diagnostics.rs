//! Diagnostics related to a name resolution process.

#![allow(clippy::needless_pass_by_value)]

use ry_ast::ModuleItemKind;
use ry_diagnostics::diagnostic::Diagnostic;
use ry_diagnostics::{BuildDiagnostic, LocationExt};
use ry_filesystem::location::Location;
use ry_interner::PathID;

/// An information about an item defined in the module.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct DefinitionInfo {
    /// Location of the item name.
    pub location: Location,

    /// Kind of the definition.
    pub kind: ModuleItemKind,
}

/// Diagnostic related to an item defined multiple times error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemDefinedMultipleTimesDiagnostic {
    /// Name of the item.
    pub name: String,

    /// First item definition.
    pub first_definition: DefinitionInfo,

    /// Second item definition.
    pub second_definition: DefinitionInfo,
}

impl ItemDefinedMultipleTimesDiagnostic {
    /// Creates a new instance of [`ItemDefinedMultipleTimesDiagnostic`].
    #[inline]
    #[must_use]
    pub fn new(
        name: impl ToString,
        first_definition: DefinitionInfo,
        second_definition: DefinitionInfo,
    ) -> Self {
        Self {
            name: name.to_string(),
            first_definition,
            second_definition,
        }
    }
}

impl BuildDiagnostic for ItemDefinedMultipleTimesDiagnostic {
    #[inline]
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E005")
            .with_message(format!(
                "the name `{}` is defined multiple times",
                self.name
            ))
            .with_labels(vec![
                self.first_definition
                    .location
                    .to_primary_label()
                    .with_message(format!(
                        "previous definition of the {} `{}` here",
                        self.first_definition.kind, self.name
                    )),
                self.second_definition
                    .location
                    .to_primary_label()
                    .with_message(format!("{} redefined here", self.name)),
            ])
    }
}

/// Diagnostic related to trying to import a package error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportingPackageDiagnostic {
    /// Location of the entire import module item.
    pub location: Location,

    /// Name of the package.
    pub package_name: String,

    /// Location of the package name in the import.
    pub package_name_location: Location,
}

impl ImportingPackageDiagnostic {
    /// Creates a new instance of [`ImportingPackageDiagnostic`].
    #[inline]
    #[must_use]
    pub fn new(
        location: Location,
        package_name: impl ToString,
        package_name_location: Location,
    ) -> Self {
        Self {
            location,
            package_name: package_name.to_string(),
            package_name_location,
        }
    }
}

impl BuildDiagnostic for ImportingPackageDiagnostic {
    #[inline]
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E006")
            .with_message("trying to import a package".to_owned())
            .with_labels(vec![
                self.location.to_primary_label()
                    .with_message("consider removing this import"),
                self.package_name_location.to_secondary_label().with_message(format!(
                    "{} is a package, not a particular module", self.package_name
                )),
            ])
            .with_notes(
                vec![
                    "note: importing a package is meaningless, you can still you its namespace without an import".to_owned(),
                ]
            )
    }
}

/// Diagnostic, that appears when you try to access a name in a namespace of
/// a module item except an enum, for example:
///
/// ```txt
/// pub fn foo() -> A.B {} // A is a struct
///                   ^ wrong
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleItemsExceptEnumsDoNotServeAsNamespacesDiagnostic {
    /// Name that was tried to be resolved.
    pub name: String,
    /// Location of the name.
    pub name_location: Location,
    /// Module item name.
    pub module_item_name: String,
    /// Location of the module item name.
    pub module_item_name_location: Location,
}

impl BuildDiagnostic for ModuleItemsExceptEnumsDoNotServeAsNamespacesDiagnostic {
    #[inline]
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E007")
            .with_message(format!("failed to resolve the name `{}`", self.name))
            .with_labels(vec![self.name_location.to_primary_label().with_message(
                format!(
                    "cannot find the name `{}` in the namespace `{}`",
                    self.name, self.module_item_name
                ),
            ), self.module_item_name_location.to_secondary_label().with_message(
                format!(
                    "`{}` is not a module or a package, so it cannot directly contain individual names",
                    self.module_item_name,
                ),
            )])
            .with_notes(vec!["module items except enums don't serve as namespaces".to_owned()])
    }
}

/// Diagnostic, that appears when you try to access a name in a namespace of
/// a enum item, for example:
///
/// ```txt
/// pub fn foo() -> Option.None.A {} // Option.None is an enum item
///                             ^ wrong
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumItemsDoNotServeAsNamespacesDiagnostic {
    /// Name that was tried to be resolved.
    pub name: String,
    /// Location of the name that was tried to be resolved.
    pub name_location: Location,
    /// Enum item name.
    pub enum_item_name: String,
    /// Location of the enum item name.
    pub enum_item_name_location: Location,
}

impl BuildDiagnostic for EnumItemsDoNotServeAsNamespacesDiagnostic {
    #[inline]
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E007")
            .with_message(format!("failed to resolve the name `{}`", self.name))
            .with_labels(vec![self.name_location.to_primary_label().with_message(
                format!(
                    "cannot find the name `{}` in the namespace `{}`",
                    self.name, self.enum_item_name
                ),
            ), self.enum_item_name_location.to_secondary_label().with_message(
                format!(
                    "`{}` is not a module or a package, so it cannot directly contain individual names",
                    self.enum_item_name,
                ),
            )])
            .with_notes(vec!["module items except enums don't serve as namespaces".to_owned()])
    }
}

/// Diagnostic, that occurs when the compiler tries to resolve a package that doesn't exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedToResolvePackageDiagnostic {
    /// Name of the package that can't be resolved.
    pub package_name: String,
    /// Location of the name of the package that can't be resolved.
    pub location: Location,
}

impl BuildDiagnostic for FailedToResolvePackageDiagnostic {
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E007")
            .with_message(format!(
                "failed to resolve the package `{}`",
                self.package_name
            ))
            .with_labels(vec![self
                .location
                .to_primary_label()
                .with_message("cannot find the package in this scope")])
            .with_notes(vec![format!(
                "consider adding `{}` into your package's manifest file `[dependencies]` section",
                self.package_name
            )])
    }
}

/// Diagnostic, that occurs when the compiler tries to resolve a submodule of a particular package that doesn't exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedToResolveModuleDiagnostic {
    /// Name of the module that doesn't exist in the current scope.
    pub module_name: String,
    /// Location of the name of the module that doesn't exist in the current scope.
    pub module_name_location: Location,
    /// Name of the module's parent package.
    pub package_name: String,
    /// Location of the name of the module's parent package.
    pub package_name_location: Location,
}

impl BuildDiagnostic for FailedToResolveModuleDiagnostic {
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E007")
            .with_message(format!(
                "failed to resolve the module `{}`",
                self.module_name
            ))
            .with_labels(vec![
                self.module_name_location.to_primary_label(),
                self.package_name_location
                    .to_secondary_label()
                    .with_message(format!(
                        "package `{}` doesn't contain the submodule `{}`",
                        self.package_name, self.module_name
                    )),
            ])
    }
}

/// Diagnostic, that occurs when the compiler tries to resolve a submodule of a particular package/module that doesn't exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedToResolveModuleItemDiagnostic {
    /// Name of the module that doesn't exist in the current scope.
    pub module_name: String,
    /// Location of the name of the module that doesn't exist in the current scope.
    pub module_name_location: Location,
    /// Item name.
    pub item_name: String,
    /// Location of the item name.
    pub item_name_location: Location,
}

impl BuildDiagnostic for FailedToResolveModuleItemDiagnostic {
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E007")
            .with_message(format!(
                "failed to resolve the module item `{}`",
                self.item_name
            ))
            .with_labels(vec![
                self.item_name_location.to_primary_label(),
                self.module_name_location
                    .to_secondary_label()
                    .with_message(format!(
                        "module `{}` doesn't contain the item `{}`",
                        self.module_name, self.item_name
                    )),
            ])
    }
}

/// Diagnostic, that occurs when the compiler tries to resolve a module's item that is defined as private.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedToResolvePrivateModuleItemDiagnostic {
    /// Name of the module that doesn't exist in the current scope.
    pub module_name: String,
    /// Location of the name of the module that doesn't exist in the current scope.
    pub module_name_location: Location,
    /// Item name.
    pub item_name: String,
    /// Location of the item name.
    pub item_name_location: Location,
}

impl BuildDiagnostic for FailedToResolvePrivateModuleItemDiagnostic {
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E007")
            .with_message(format!(
                "failed to resolve private module item `{}`",
                self.item_name
            ))
            .with_labels(vec![
                self.item_name_location.to_primary_label(),
                self.module_name_location
                    .to_secondary_label()
                    .with_message(format!(
                        "module `{}` contains the item `{}`, but it is defined as private",
                        self.module_name, self.item_name
                    )),
            ])
    }
}

/// Diagnostic, that occurs when the compiler tries to resolve a name in a module scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedToResolveNameDiagnostic {
    /// Name.
    pub name: String,
    /// Location of the name.
    pub location: Location,
}

impl BuildDiagnostic for FailedToResolveNameDiagnostic {
    fn build(&self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_code("E007")
            .with_message(format!("failed to resolve `{}`", self.name))
            .with_labels(vec![self.location.to_primary_label()])
    }
}
