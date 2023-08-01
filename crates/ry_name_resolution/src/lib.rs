//! # Name resolution
//!
//! The name resolution allows to resolve names, after parsing all the packages in stages like
//! type checking and MIR lowering.
//!
//! See [`GlobalContext`], [`PackageContext`] and [`ModuleContext`] for more details.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]
#![warn(missing_docs, clippy::dbg_macro)]
#![deny(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    warnings,
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

use diagnostics::{
    FailedToResolveModuleDiagnostic, FailedToResolveModuleItemDiagnostic,
    FailedToResolveNameDiagnostic, FailedToResolvePackageDiagnostic,
    ModuleItemsExceptEnumsDoNotServeAsNamespacesDiagnostic,
};
use ry_ast::{DefinitionID, IdentifierAST};
use ry_diagnostics::{BuildDiagnostic, GlobalDiagnostics};
use ry_fx_hash::FxHashMap;
use ry_interner::{IdentifierInterner, PathID, Symbol};

pub mod diagnostics;

/// A data structure used to store information about modules and packages that
/// are going through the name resolution process.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ResolutionEnvironment {
    /// Packages' root modules that are analyzed in the workspace.
    pub packages_root_modules: FxHashMap<Symbol, PathID>,

    /// Modules, that are analyzed in the workspace.
    pub modules: FxHashMap<PathID, ModuleScope>,

    /// Storage of absolute paths of modules in the environment, e.g. `std.io`.
    pub module_paths: FxHashMap<PathID, Path>,
}

impl ResolutionEnvironment {
    /// Creates a new empty resolution environment
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolve a path in the environment.
    #[allow(clippy::missing_panics_doc)]
    pub fn resolve_path(
        &self,
        path: ry_ast::Path,
        identifier_interner: &IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
    ) -> Option<NameBinding> {
        let mut identifiers = path.identifiers.into_iter();
        let first_identifier = identifiers.next().unwrap();

        let mut previous_identifier = first_identifier;

        if !self
            .packages_root_modules
            .contains_key(&first_identifier.symbol)
        {
            diagnostics.add_single_file_diagnostic(
                first_identifier.location.file_path_id,
                FailedToResolvePackageDiagnostic {
                    package_name: identifier_interner
                        .resolve(first_identifier.symbol)
                        .unwrap()
                        .to_owned(),
                    location: first_identifier.location,
                }
                .build(),
            );
            return None;
        }

        let mut binding = Some(NameBinding::Package(first_identifier.symbol));

        for identifier in identifiers {
            binding = binding.map(|binding| match binding {
                NameBinding::Package(package_symbol) => {
                    let Some(module) = self
                    .modules
                    .get(self.packages_root_modules.get(&package_symbol).unwrap())
                    // Module must exist at this point, or something went wrong when
                    // building the name resolution environment.
                    .unwrap()
                    .resolve(identifier, identifier_interner, diagnostics, self) else {
                        diagnostics.add_single_file_diagnostic(identifier.location.file_path_id, FailedToResolveModuleDiagnostic {
                            module_name: identifier_interner.resolve(identifier.symbol).unwrap().to_owned(),
                            module_name_location: identifier.location,
                            package_name: identifier_interner.resolve(previous_identifier.symbol).unwrap().to_owned(),
                            package_name_location: previous_identifier.location,
                        }.build());

                        return None;
                    };

                    Some(module)
                },
                NameBinding::EnumItem(_) => {
                    diagnostics.add_single_file_diagnostic(
                        identifier.location.file_path_id,
                        ModuleItemsExceptEnumsDoNotServeAsNamespacesDiagnostic {
                            name: identifier_interner
                                .resolve(identifier.symbol)
                                .unwrap()
                                .to_owned(),
                            name_location: identifier.location,
                            module_item_name: identifier_interner
                                .resolve(previous_identifier.symbol)
                                .unwrap()
                                .to_owned(),
                            module_item_name_location: previous_identifier.location,
                        }
                        .build(),
                    );

                    None
                }
                NameBinding::ModuleItem(definition_id) => {
                    if let Some(enum_scope) = self
                        .modules
                        .get(&definition_id.module_path_id)?
                        .enums
                        .get(&definition_id)
                    {
                        enum_scope
                            .items
                            .get(&identifier.symbol)
                            .copied()
                            .map(NameBinding::EnumItem)
                    } else {
                        diagnostics.add_single_file_diagnostic(
                            identifier.location.file_path_id,
                            ModuleItemsExceptEnumsDoNotServeAsNamespacesDiagnostic {
                                name: identifier_interner
                                    .resolve(identifier.symbol)
                                    .unwrap()
                                    .to_owned(),
                                name_location: identifier.location,
                                module_item_name: identifier_interner
                                    .resolve(previous_identifier.symbol)
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
                    let Some(binding) = self.modules.get(&submodule_id).unwrap().bindings.get(&identifier.symbol) else {
                        diagnostics.add_single_file_diagnostic(identifier.location.file_path_id, FailedToResolveModuleItemDiagnostic {
                            item_name: identifier_interner.resolve(identifier.symbol).unwrap().to_owned(),
                            item_name_location: identifier.location,
                            module_name: identifier_interner.resolve(previous_identifier.symbol).unwrap().to_owned(),
                            module_name_location: previous_identifier.location,
                        }.build());

                        return None;
                    };

                    Some(*binding)
                }
            })?;

            previous_identifier = identifier;
        }

        binding
    }
}

/// A name binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NameBinding {
    /// A package.
    Package(Symbol),

    /// A submodule.
    Module(PathID),

    /// An item defined in a particular module.
    ModuleItem(DefinitionID),

    /// An enum item.
    EnumItem(EnumItemID),
}

/// Path - a list of identifiers separated by commas. The main difference
/// between this struct and [`ry_ast::Path`] is that the former doesn't store
/// locations of identifiers.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Path {
    /// List of semantic symbols.
    pub symbols: Vec<Symbol>,
}

/// Data that Ry compiler has about a module.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ModuleScope {
    /// The interned name of the module.
    pub name: Symbol,

    /// The id of the path to the module source file.
    pub path_id: PathID,

    /// The module items name bindings.
    pub bindings: FxHashMap<Symbol, NameBinding>,

    /// Enums.
    pub enums: FxHashMap<DefinitionID, EnumData>,

    /// The imports used in the module.
    pub imports: FxHashMap<Symbol, ry_ast::Path>,
}

/// Data the Ry compiler has about a particular enum.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EnumData {
    /// Enum items.
    pub items: FxHashMap<Symbol, EnumItemID>,
}

impl ModuleScope {
    /// Resolves an identifier in a module scope. If resolution fails, returns [`None`]
    /// and adds a new diagnostics.
    pub fn resolve(
        &self,
        identifier: IdentifierAST,
        identifier_interner: &IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
        environment: &ResolutionEnvironment,
    ) -> Option<NameBinding> {
        if let Some(inner_binding) = self.bindings.get(&identifier.symbol).copied().or_else(|| {
            if environment
                .packages_root_modules
                .contains_key(&identifier.symbol)
            {
                Some(NameBinding::Package(identifier.symbol))
            } else {
                None
            }
        }) {
            Some(inner_binding)
        } else {
            // check for possible name binding that can come from imports
            if let Some(import_path) = self.imports.get(&identifier.symbol) {
                environment.resolve_path(import_path.clone(), identifier_interner, diagnostics)
            } else {
                diagnostics.add_single_file_diagnostic(
                    identifier.location.file_path_id,
                    FailedToResolveNameDiagnostic {
                        name: identifier_interner
                            .resolve(identifier.symbol)
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

/// Data that Ry compiler has about a name binding in a module.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ModuleNameBinding {
    /// Submodule.
    Submodule(PathID),

    /// Item defined in the module.
    ModuleItem(DefinitionID),

    /// Enum item.
    EnumItem(EnumItemID),
}

/// Unique ID of the enum item
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct EnumItemID {
    enum_definition_id: DefinitionID,
    item_name: Symbol,
}
