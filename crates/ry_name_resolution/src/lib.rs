//! # Name resolution
//!
//! The name resolution allows to resolve names, after parsing all the packages in stages like
//! type checking and MIR lowering.
//!
//! See [`ResolutionEnvironment`], [`ModuleScope`] and [`NameBinding`] for more details.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]
#![warn(missing_docs, clippy::dbg_macro)]
#![warn(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    future_incompatible,
    let_underscore,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,
    // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,
    // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,
    // clippy categories https://doc.rust-lang.org/clippy/
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::option_if_let_else,
    clippy::cast_possible_truncation
)]

use std::{fmt::Debug, iter};

use derive_more::Display;
use diagnostics::{
    FailedToResolveModuleDiagnostic, FailedToResolveModuleItemDiagnostic,
    FailedToResolveNameDiagnostic, FailedToResolvePackageDiagnostic,
    FailedToResolvePrivateModuleItemDiagnostic,
    ModuleItemsExceptEnumsDoNotServeAsNamespacesDiagnostic,
};
use itertools::Itertools;
use ry_ast::{IdentifierAST, Visibility};
use ry_diagnostics::{BuildDiagnostic, GlobalDiagnostics};
use ry_fx_hash::FxHashMap;
use ry_interner::{IdentifierID, IdentifierInterner, PathID};

pub mod diagnostics;

/// An ID assigned for every package in a workspace.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct PackageID(pub IdentifierID);

/// An ID assigned for every module in a workspace.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct ModuleID(pub PathID);

/// An ID for every definition (module item) in a workspace.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct DefinitionID {
    /// Interned name of the definition.
    pub symbol: IdentifierID,

    /// ID of the module that contains the definition.
    pub module_id: ModuleID,
}

/// A data structure used to store information about modules and packages that
/// are going through the name resolution process.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ResolutionEnvironment {
    /// Packages' root modules that are analyzed in the workspace.
    pub packages_root_modules: FxHashMap<PackageID, ModuleID>,

    /// Modules, that are analyzed in the workspace.
    pub module_scopes: FxHashMap<ModuleID, ModuleScope>,

    /// Storage of absolute paths of modules in the environment, e.g. `std.io`.
    pub module_paths: FxHashMap<ModuleID, Path>,

    /// Storage of visibilities of module items.
    pub visibilities: FxHashMap<DefinitionID, Visibility>,

    /// Resolved imports in all modules.
    pub resolved_imports: FxHashMap<ModuleID, ResolvedImports>,
}

/// Resolved imports in a particular module.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ResolvedImports {
    /// List of resolved imports.
    pub imports: FxHashMap<IdentifierID, NameBinding>,
}

impl ResolutionEnvironment {
    /// Creates a new empty resolution environment
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolves all imports in modules that have been added to the environment previously.
    ///
    /// **Note**: Imports must be resolved before any name resolution process going on!!!
    ///
    /// # Panics
    /// - If the environment data is invalid.
    /// - If one of the import paths is empty.
    /// - If one of the import paths contains symbols, that cannot be resolved by an identifier interner.
    pub fn resolve_imports(
        &mut self,
        identifier_interner: &IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
    ) {
        for (module_path_id, module_scope) in &self.module_scopes {
            let mut imports = FxHashMap::default();

            for (symbol, import_path) in &module_scope.imports {
                let Some(resolved_name_binding) =
                    self.resolve_path(import_path.clone(), identifier_interner, diagnostics)
                else {
                    continue;
                };

                imports.insert(*symbol, resolved_name_binding);
            }

            self.resolved_imports
                .insert(*module_path_id, ResolvedImports { imports });
        }
    }

    /// Resolve an import path in the environment.
    ///
    /// # Panics
    /// - If the environment data is invalid.
    /// - If the path is empty.
    /// - If the path contains symbols, that cannot be resolved by an identifier interner.
    pub fn resolve_path(
        &self,
        path: ry_ast::Path,
        identifier_interner: &IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
    ) -> Option<NameBinding> {
        let mut identifiers = path.identifiers.into_iter();
        let first_identifier = identifiers.next().unwrap();

        if !self
            .packages_root_modules
            .contains_key(&PackageID(first_identifier.id))
        {
            diagnostics.add_single_file_diagnostic(
                first_identifier.location.file_path_id,
                FailedToResolvePackageDiagnostic {
                    package_name: identifier_interner
                        .resolve(first_identifier.id)
                        .unwrap()
                        .to_owned(),
                    location: first_identifier.location,
                }
                .build(),
            );
            return None;
        }

        NameBinding::Package(PackageID(first_identifier.id)).resolve_rest_of_the_path(
            first_identifier,
            identifiers,
            identifier_interner,
            diagnostics,
            self,
        )
    }
}

/// A name binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NameBinding {
    /// A package.
    Package(PackageID),

    /// A submodule.
    Module(ModuleID),

    /// An item defined in a particular module.
    ModuleItem(DefinitionID),

    /// An enum item.
    EnumItem(EnumItemID),
}

/// A trait for getting the full path of a definition.
pub trait ResolveFullPath: Sized + Debug + Copy {
    /// Returns the full path of the definition.
    fn full_path(self, environment: &ResolutionEnvironment) -> Option<Path>;

    /// Returns the full path of the definition.
    #[must_use]
    fn full_path_or_panic(self, environment: &ResolutionEnvironment) -> Path {
        self.full_path(environment)
            .unwrap_or_else(|| panic!("Failed to get full path of the definition id:\n{self:?}"))
    }
}

impl ResolveFullPath for PackageID {
    fn full_path(self, _: &ResolutionEnvironment) -> Option<Path> {
        Some(Path {
            identifiers: vec![self.0],
        })
    }
}

impl ResolveFullPath for DefinitionID {
    fn full_path(self, environment: &ResolutionEnvironment) -> Option<Path> {
        environment
            .module_paths
            .get(&self.module_id)
            .map(|module_path| Path {
                identifiers: module_path
                    .clone()
                    .identifiers
                    .into_iter()
                    .chain(iter::once(self.symbol))
                    .collect(),
            })
    }
}

impl ResolveFullPath for ModuleID {
    #[inline]
    fn full_path(self, environment: &ResolutionEnvironment) -> Option<Path> {
        environment.module_paths.get(&self).cloned()
    }
}

impl ResolveFullPath for EnumItemID {
    #[inline]
    fn full_path(self, environment: &ResolutionEnvironment) -> Option<Path> {
        self.enum_definition_id
            .full_path(environment)
            .map(|path| Path {
                identifiers: path
                    .identifiers
                    .into_iter()
                    .chain(iter::once(self.item_id))
                    .collect(),
            })
    }
}

impl ResolveFullPath for NameBinding {
    #[inline]
    fn full_path(self, environment: &ResolutionEnvironment) -> Option<Path> {
        match self {
            Self::Package(package_id) => package_id.full_path(environment),
            Self::Module(module_id) => module_id.full_path(environment),
            Self::ModuleItem(definition_id) => definition_id.full_path(environment),
            Self::EnumItem(enum_item_id) => enum_item_id.full_path(environment),
        }
    }
}

/// A kind of a name binding.
///
/// See [`NameBinding`] for more details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum NameBindingKind {
    /// A package.
    #[display(fmt = "package")]
    Package,

    /// A module.
    #[display(fmt = "module")]
    Module,

    /// A module item.
    #[display(fmt = "module item")]
    ModuleItem,

    /// An enum item.
    #[display(fmt = "enum item")]
    EnumItem,
}

impl NameBinding {
    /// Returns the kind of the name binding.
    ///
    /// See [`NameBindingKind`] for more details.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> NameBindingKind {
        match self {
            Self::Package(..) => NameBindingKind::Package,
            Self::Module(..) => NameBindingKind::Module,
            Self::ModuleItem(..) => NameBindingKind::ModuleItem,
            Self::EnumItem(..) => NameBindingKind::EnumItem,
        }
    }

    /// Resolve the rest of the path. For instance, for the path `std.io.println`, `io.println` is
    /// the value of `other_identifiers`, and the `std` is the `first_identifier`.
    ///
    /// # Panics
    /// - If the environment data is invalid.
    /// - If the path contains symbols, that cannot be resolved by an identifier interner.
    #[inline]
    pub fn resolve_rest_of_the_path(
        self,
        first_identifier: IdentifierAST,
        other_identifiers: impl IntoIterator<Item = IdentifierAST>,
        identifier_interner: &IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
        environment: &ResolutionEnvironment,
    ) -> Option<Self> {
        iter::once(first_identifier)
            .chain(other_identifiers)
            .tuple_windows()
            .try_fold(
                self,
                |binding, (previous_identifier, current_identifier)| {
                    resolve_path_segment(
                        binding,
                        previous_identifier,
                        current_identifier,
                        identifier_interner,
                        diagnostics,
                        environment,
                    )
                },
            )
    }
}

/// Resolves a path segment. Path segment in this context means a single identifier in the path,
/// that in this concrete case must be followed by another identifier at the start:
///
/// ```txt
/// std.io
/// ^^^ ^^ current_identifier
/// previous_identifier
/// ```
fn resolve_path_segment(
    binding: NameBinding,
    previous_identifier: IdentifierAST,
    current_identifier: IdentifierAST,
    identifier_interner: &IdentifierInterner,
    diagnostics: &mut GlobalDiagnostics,
    environment: &ResolutionEnvironment,
) -> Option<NameBinding> {
    match binding {
        NameBinding::Package(package_id) => {
            let Some(module) = environment
                .module_scopes
                .get(environment.packages_root_modules.get(&package_id).unwrap())
                // Module must exist at this point, or something went wrong when
                // building the name resolution environment.
                .unwrap()
                .resolve(
                    current_identifier,
                    identifier_interner,
                    diagnostics,
                    environment,
                )
            else {
                diagnostics.add_single_file_diagnostic(
                    current_identifier.location.file_path_id,
                    FailedToResolveModuleDiagnostic {
                        module_name: identifier_interner
                            .resolve(current_identifier.id)
                            .unwrap()
                            .to_owned(),
                        module_name_location: current_identifier.location,
                        package_name: identifier_interner
                            .resolve(previous_identifier.id)
                            .unwrap()
                            .to_owned(),
                        package_name_location: previous_identifier.location,
                    }
                    .build(),
                );

                return None;
            };

            Some(module)
        }
        NameBinding::EnumItem(_) => {
            diagnostics.add_single_file_diagnostic(
                current_identifier.location.file_path_id,
                ModuleItemsExceptEnumsDoNotServeAsNamespacesDiagnostic {
                    name: identifier_interner
                        .resolve(current_identifier.id)
                        .unwrap()
                        .to_owned(),
                    name_location: current_identifier.location,
                    module_item_name: identifier_interner
                        .resolve(previous_identifier.id)
                        .unwrap()
                        .to_owned(),
                    module_item_name_location: previous_identifier.location,
                }
                .build(),
            );

            None
        }
        NameBinding::ModuleItem(definition_id) => {
            if let Some(enum_scope) = environment
                .module_scopes
                .get(&definition_id.module_id)?
                .enums
                .get(&definition_id.symbol)
            {
                enum_scope
                    .items
                    .get(&current_identifier.id)
                    .copied()
                    .map(NameBinding::EnumItem)
            } else {
                diagnostics.add_single_file_diagnostic(
                    current_identifier.location.file_path_id,
                    ModuleItemsExceptEnumsDoNotServeAsNamespacesDiagnostic {
                        name: identifier_interner
                            .resolve(current_identifier.id)
                            .unwrap()
                            .to_owned(),
                        name_location: current_identifier.location,
                        module_item_name: identifier_interner
                            .resolve(previous_identifier.id)
                            .unwrap()
                            .to_owned(),
                        module_item_name_location: previous_identifier.location,
                    }
                    .build(),
                );

                None
            }
        }
        NameBinding::Module(submodule_id) => {
            let Some(binding) = environment
                .module_scopes
                .get(&submodule_id)
                .unwrap()
                .bindings
                .get(&current_identifier.id)
            else {
                diagnostics.add_single_file_diagnostic(
                    current_identifier.location.file_path_id,
                    FailedToResolveModuleItemDiagnostic {
                        item_name: identifier_interner
                            .resolve(current_identifier.id)
                            .unwrap()
                            .to_owned(),
                        item_name_location: current_identifier.location,
                        module_name: identifier_interner
                            .resolve(previous_identifier.id)
                            .unwrap()
                            .to_owned(),
                        module_name_location: previous_identifier.location,
                    }
                    .build(),
                );

                return None;
            };

            if let NameBinding::ModuleItem(definition_id) = binding {
                if *environment.visibilities.get(definition_id).unwrap() == Visibility::Private {
                    diagnostics.add_single_file_diagnostic(
                        current_identifier.location.file_path_id,
                        FailedToResolvePrivateModuleItemDiagnostic {
                            item_name: identifier_interner
                                .resolve(current_identifier.id)
                                .unwrap()
                                .to_owned(),
                            item_name_location: current_identifier.location,
                            module_name: identifier_interner
                                .resolve(previous_identifier.id)
                                .unwrap()
                                .to_owned(),
                            module_name_location: previous_identifier.location,
                        }
                        .build(),
                    );

                    return None;
                }
            }

            Some(*binding)
        }
    }
}

/// Path - a list of identifiers separated by commas. The main difference
/// between this struct and [`ry_ast::Path`] is that the former doesn't store
/// locations of identifiers.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Path {
    /// List of semantic symbols.
    pub identifiers: Vec<IdentifierID>,
}

/// Data that Ry compiler has about a module.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ModuleScope {
    /// Interned name of the module.
    pub name: IdentifierID,

    /// ID of the module's source file path.
    pub id: ModuleID,

    /// The module items name bindings.
    pub bindings: FxHashMap<IdentifierID, NameBinding>,

    /// Enums.
    pub enums: FxHashMap<IdentifierID, EnumData>,

    /// Imports used in the module.
    pub imports: FxHashMap<IdentifierID, ry_ast::Path>,
}

/// Data the Ry compiler has about a particular enum.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EnumData {
    /// Enum items.
    pub items: FxHashMap<IdentifierID, EnumItemID>,
}

impl ModuleScope {
    /// Resolve the path in the module scope.
    ///
    /// # Panics
    ///
    /// - If the environment data is invalid.
    /// - If the path is empty.
    /// - If the path contains symbols, that cannot be resolved by an identifier interner.
    pub fn resolve_path(
        &self,
        path: ry_ast::Path,
        identifier_interner: &IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
        environment: &ResolutionEnvironment,
    ) -> Option<NameBinding> {
        let mut identifiers = path.identifiers.into_iter();
        let first_identifier = identifiers.next().unwrap();

        self.resolve(
            first_identifier,
            identifier_interner,
            diagnostics,
            environment,
        )?
        .resolve_rest_of_the_path(
            first_identifier,
            identifiers,
            identifier_interner,
            diagnostics,
            environment,
        )
    }

    /// Resolves an identifier in a module scope. If resolution fails, returns [`None`]
    /// and adds a new diagnostics.
    ///
    /// # Panics
    /// - If the environment data is wrong.
    /// - If the path contains symbols, that cannot be resolved by an identifier interner.
    pub fn resolve(
        &self,
        identifier: IdentifierAST,
        identifier_interner: &IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
        environment: &ResolutionEnvironment,
    ) -> Option<NameBinding> {
        if let Some(binding) = self.bindings.get(&identifier.id).copied().or_else(|| {
            if environment
                .packages_root_modules
                .contains_key(&PackageID(identifier.id))
            {
                Some(NameBinding::Package(PackageID(identifier.id)))
            } else {
                None
            }
        }) {
            Some(binding)
        } else {
            // check for possible name binding that can come from imports
            if let Some(binding) = environment.resolved_imports[&self.id]
                .imports
                .get(&identifier.id)
            {
                Some(*binding)
            } else {
                diagnostics.add_single_file_diagnostic(
                    identifier.location.file_path_id,
                    FailedToResolveNameDiagnostic {
                        name: identifier_interner
                            .resolve(identifier.id)
                            .unwrap()
                            .to_owned(),
                        location: identifier.location,
                    }
                    .build(),
                );

                None
            }
        }
    }
}

/// Unique ID of the enum item
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct EnumItemID {
    /// ID of the enum definition.
    pub enum_definition_id: DefinitionID,

    /// Enum item symbol.
    pub item_id: IdentifierID,
}
