//! # Name resolution
//!
//! The name resolution allows to resolve names, after parsing all the packages in stages like
//! type checking and MIR lowering.
//!
//! See [`NameResolver`], [`ModuleScope`] and [`NameBinding`] for more details.

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
    clippy::cast_possible_truncation,
    clippy::inline_always,
    clippy::doc_markdown
)]

use std::{fmt::Debug, iter};

use derive_more::Display;
use diagnostics::{
    FailedToResolveModule, FailedToResolveModuleItem, FailedToResolveName, FailedToResolvePackage,
    FailedToResolvePrivateModuleItem, ModuleItemsExceptEnumsDoNotServeAsNamespaces,
};
use itertools::Itertools;
use parking_lot::RwLock;
use ry_ast::{IdentifierAST, Visibility};
use ry_diagnostics::Diagnostics;
use ry_fx_hash::FxHashMap;
use ry_interner::{IdentifierID, IdentifierInterner, PathID};

pub mod diagnostics;

/// An ID assigned to every package in a workspace.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct PackageID(pub IdentifierID);

/// An ID assigned to every module in a workspace.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Display, Hash)]
pub struct ModuleID(pub PathID);

/// An ID for every definition (module item) in a workspace.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct DefinitionID {
    /// Interned name of the definition.
    pub name_id: IdentifierID,

    /// ID of the module that contains the definition.
    pub module_id: ModuleID,
}

/// A data structure used to store information about modules and packages that
/// are going through the name resolution process.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NameResolver {
    /// Packages' root modules that are analyzed in the workspace.
    packages_root_modules: FxHashMap<PackageID, ModuleID>,

    /// Modules, that are analyzed in the workspace.
    module_scopes: FxHashMap<ModuleID, ModuleScope>,

    /// Resolved imports in all modules.
    resolved_imports: FxHashMap<ModuleID, ResolvedImportsInModule>,

    /// Visibilities of module items.
    visibilities: FxHashMap<DefinitionID, RawVisibility>,
}

/// Visibility representation, that doesn't store location of `pub` keyword.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Display, Default)]
pub enum RawVisibility {
    /// Public visibility.
    #[display(fmt = "public")]
    Public,

    /// Private visibility.
    #[default]
    #[display(fmt = "private")]
    Private,
}

impl From<Visibility> for RawVisibility {
    #[inline(always)]
    fn from(value: Visibility) -> Self {
        match value {
            Visibility::Public(_) => Self::Public,
            Visibility::Private => Self::Private,
        }
    }
}

/// Resolved imports in a particular module.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ResolvedImportsInModule {
    /// List of resolved imports.
    pub imports: FxHashMap<IdentifierID, NameBinding>,
}

impl NameResolver {
    /// Creates a new empty resolution name_resolver
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a package into the name resolver storage.
    #[inline(always)]
    pub fn add_package(&mut self, package_id: PackageID, root_module_id: ModuleID) {
        self.packages_root_modules
            .insert(package_id, root_module_id);
    }

    /// Adds a list of packages into the name resolver storage.
    #[inline(always)]
    pub fn add_packages(&mut self, packages: impl IntoIterator<Item = (PackageID, ModuleID)>) {
        for (package_id, root_module_id) in packages {
            self.add_package(package_id, root_module_id);
        }
    }

    /// Adds a module scope into the name resolver storage.
    #[inline(always)]
    pub fn add_module_scope(&mut self, module_id: ModuleID, scope: ModuleScope) {
        self.module_scopes.insert(module_id, scope);
    }

    /// Adds a list of module scopes into the name resolver storage.
    #[inline(always)]
    pub fn add_module_scopes(
        &mut self,
        module_scopes: impl IntoIterator<Item = (ModuleID, ModuleScope)>,
    ) {
        for (module_id, scope) in module_scopes {
            self.add_module_scope(module_id, scope);
        }
    }

    /// Resolves a module scope by its module it.
    #[inline(always)]
    #[must_use]
    pub fn resolve_module_scope(&self, module_id: ModuleID) -> Option<&ModuleScope> {
        self.module_scopes.get(&module_id)
    }

    /// Resolve a module scope by its module it.
    ///
    /// # Panics
    /// If the module doesn't exist.
    #[inline(always)]
    #[must_use]
    pub fn resolve_module_scope_or_panic(&self, module_id: ModuleID) -> &ModuleScope {
        self.resolve_module_scope(module_id)
            .unwrap_or_else(|| panic!("Module with ID: {module_id} cannot be resolved."))
    }

    /// Adds a visibility information about definition into resolver storage.
    #[inline(always)]
    pub fn add_visibility(
        &mut self,
        definition_id: DefinitionID,
        visibility: impl Into<RawVisibility>,
    ) {
        self.visibilities.insert(definition_id, visibility.into());
    }

    /// Adds a visibility information about definition into resolver storage.
    #[inline(always)]
    #[must_use]
    pub fn resolve_visibility(&self, definition_id: DefinitionID) -> Option<RawVisibility> {
        self.visibilities.get(&definition_id).copied()
    }

    /// Adds a visibility information about a definition into resolver storage.
    ///
    /// # Panics
    /// If the visibility information about the definition doesn't exist.
    #[inline(always)]
    #[must_use]
    pub fn resolve_visibility_or_panic(&self, definition_id: DefinitionID) -> RawVisibility {
        *self.visibilities.get(&definition_id).unwrap()
    }

    /// Resolves all imports in modules that have been added to the name_resolver previously.
    ///
    /// **Note**: Imports must be resolved before any name resolution process going on!!!
    ///
    /// # Panics
    /// - If the name_resolver data is invalid.
    /// - If one of the import paths is empty.
    /// - If one of the import paths contains symbols, that cannot be resolved by an identifier interner.
    pub fn resolve_imports(
        &mut self,
        identifier_interner: &IdentifierInterner,
        diagnostics: &RwLock<Diagnostics>,
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
                .insert(*module_path_id, ResolvedImportsInModule { imports });
        }
    }

    /// Resolve a global path in the name_resolver.
    ///
    /// # Panics
    /// - If the name_resolver data is invalid.
    /// - If the path is empty.
    /// - If the path contains symbols, that cannot be resolved by an identifier interner.
    pub fn resolve_path(
        &self,
        path: ry_ast::Path,
        identifier_interner: &IdentifierInterner,
        diagnostics: &RwLock<Diagnostics>,
    ) -> Option<NameBinding> {
        let mut identifiers = path.identifiers.into_iter();
        let first_identifier = identifiers.next().unwrap();

        if !self
            .packages_root_modules
            .contains_key(&PackageID(first_identifier.id))
        {
            diagnostics.write().add_single_file_diagnostic(
                first_identifier.location.file_path_id,
                FailedToResolvePackage::new(
                    first_identifier.location,
                    identifier_interner
                        .resolve(first_identifier.id)
                        .unwrap()
                        .to_owned(),
                ),
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

    /// Resolve the path in the module scope.
    ///
    /// # Panics
    ///
    /// - If the name_resolver data is invalid.
    /// - If the path is empty.
    /// - If the path contains symbols, that cannot be resolved by an identifier interner.
    pub fn resolve_path_in_module_scope(
        &self,
        module_scope: &ModuleScope,
        path: ry_ast::Path,
        identifier_interner: &IdentifierInterner,
        diagnostics: &RwLock<Diagnostics>,
    ) -> Option<NameBinding> {
        let mut identifiers = path.identifiers.into_iter();
        let first_identifier = identifiers.next().unwrap();

        self.resolve_identifier_in_module_scope(
            module_scope,
            first_identifier,
            identifier_interner,
            diagnostics,
        )?
        .resolve_rest_of_the_path(
            first_identifier,
            identifiers,
            identifier_interner,
            diagnostics,
            self,
        )
    }

    /// Resolves an identifier in a module scope. If resolution fails, returns [`None`]
    /// and adds a new diagnostics.
    ///
    /// # Panics
    /// - If the name_resolver data is wrong.
    /// - If the path contains symbols, that cannot be resolved by an identifier interner.
    pub fn resolve_identifier_in_module_scope(
        &self,
        module_scope: &ModuleScope,
        identifier: IdentifierAST,
        identifier_interner: &IdentifierInterner,
        diagnostics: &RwLock<Diagnostics>,
    ) -> Option<NameBinding> {
        if let Some(binding) = module_scope
            .bindings
            .get(&identifier.id)
            .copied()
            .or_else(|| {
                if self
                    .packages_root_modules
                    .contains_key(&PackageID(identifier.id))
                {
                    Some(NameBinding::Package(PackageID(identifier.id)))
                } else {
                    None
                }
            })
        {
            Some(binding)
        } else {
            // check for possible name binding that can come from imports
            if let Some(binding) = self.resolved_imports[&module_scope.id]
                .imports
                .get(&identifier.id)
            {
                Some(*binding)
            } else {
                diagnostics.write().add_single_file_diagnostic(
                    identifier.location.file_path_id,
                    FailedToResolveName::new(
                        identifier_interner
                            .resolve(identifier.id)
                            .unwrap()
                            .to_owned(),
                        identifier.location,
                    ),
                );

                None
            }
        }
    }
}

/// A name binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NameBinding {
    /// A package.
    Package(PackageID),

    /// A submodule.
    Module(ModuleID),

    /// A type alias.
    TypeAlias(DefinitionID),

    /// A function.
    Function(DefinitionID),

    /// An interface.
    Interface(DefinitionID),

    /// A struct.
    Struct(DefinitionID),

    /// A tuple-like struct.
    TupleLikeStruct(DefinitionID),

    /// An enum.
    Enum(DefinitionID),

    /// An enum item.
    EnumItem(EnumItemID),
}

impl NameBinding {
    /// Returns the definition ID of the name binding, if it is a module item.
    #[inline(always)]
    #[must_use]
    pub const fn definition_id(&self) -> Option<DefinitionID> {
        match self {
            Self::Enum(definition_id)
            | Self::Function(definition_id)
            | Self::Interface(definition_id)
            | Self::Struct(definition_id)
            | Self::TypeAlias(definition_id) => Some(*definition_id),
            _ => None,
        }
    }

    /// Returns the definition ID of the name binding, if it is a module item.
    ///
    /// # Panics
    /// If the name binding is not a module item.
    #[inline(always)]
    #[must_use]
    pub fn definition_id_or_panic(&self) -> DefinitionID {
        self.definition_id().unwrap()
    }
}

/// A trait for getting the full path of a definition.
pub trait FullPathOf<X>
where
    X: Debug + Copy,
{
    /// Returns the full path of the definition.
    fn full_path_of(&self, x: X) -> Option<Path>;

    /// Returns the full path of the definition.
    #[must_use]
    fn full_path_of_or_panic(&self, x: X) -> Path {
        self.full_path_of(x)
            .unwrap_or_else(|| panic!("Failed to get full path of the definition id:\n{x:?}"))
    }
}

impl FullPathOf<PackageID> for NameResolver {
    #[inline(always)]
    fn full_path_of(&self, id: PackageID) -> Option<Path> {
        Some(Path {
            identifiers: vec![id.0],
        })
    }
}

impl FullPathOf<DefinitionID> for NameResolver {
    fn full_path_of(&self, id: DefinitionID) -> Option<Path> {
        self.full_path_of(id.module_id).map(|module_path| Path {
            identifiers: module_path
                .identifiers
                .into_iter()
                .chain(iter::once(id.name_id))
                .collect(),
        })
    }
}

impl FullPathOf<ModuleID> for NameResolver {
    fn full_path_of(&self, id: ModuleID) -> Option<Path> {
        self.module_scopes.get(&id).map(|scope| scope.path.clone())
    }
}

impl FullPathOf<EnumItemID> for NameResolver {
    #[inline(always)]
    fn full_path_of(&self, id: EnumItemID) -> Option<Path> {
        self.full_path_of(id.enum_definition_id).map(|path| Path {
            identifiers: path
                .identifiers
                .into_iter()
                .chain(iter::once(id.item_id))
                .collect(),
        })
    }
}

impl FullPathOf<NameBinding> for NameResolver {
    #[inline(always)]
    fn full_path_of(&self, binding: NameBinding) -> Option<Path> {
        match binding {
            NameBinding::Package(package_id) => self.full_path_of(package_id),
            NameBinding::Module(module_id) => self.full_path_of(module_id),
            NameBinding::TypeAlias(definition_id)
            | NameBinding::Function(definition_id)
            | NameBinding::Interface(definition_id)
            | NameBinding::Struct(definition_id)
            | NameBinding::Enum(definition_id)
            | NameBinding::TupleLikeStruct(definition_id) => self.full_path_of(definition_id),
            NameBinding::EnumItem(enum_item_id) => self.full_path_of(enum_item_id),
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

    /// An enum.
    #[display(fmt = "enum")]
    Enum,

    /// A type alias.
    #[display(fmt = "type alias")]
    TypeAlias,

    /// A struct.
    #[display(fmt = "struct")]
    Struct,

    /// A tuple-like struct.
    #[display(fmt = "tuple-like struct")]
    TupleLikeStruct,

    /// A function.
    #[display(fmt = "function")]
    Function,

    /// A interface.
    #[display(fmt = "interface")]
    Interface,

    /// An enum item.
    #[display(fmt = "enum item")]
    EnumItem,
}

impl NameBindingKind {
    /// Returns `true` if the name binding is a module item.
    #[inline(always)]
    #[must_use]
    pub const fn is_module_item(&self) -> bool {
        matches!(
            self,
            Self::Function | Self::Enum | Self::Interface | Self::Struct | Self::TypeAlias
        )
    }
}

impl NameBinding {
    /// Returns `true` if the name binding is a module item.
    #[inline(always)]
    #[must_use]
    pub const fn is_module_item(&self) -> bool {
        matches!(
            self,
            Self::TypeAlias(_)
                | Self::Function(_)
                | Self::Enum(_)
                | Self::Interface(_)
                | Self::Struct(_)
        )
    }

    /// Returns the kind of the name binding.
    ///
    /// See [`NameBindingKind`] for more details.
    #[inline(always)]
    #[must_use]
    pub const fn kind(&self) -> NameBindingKind {
        match self {
            Self::Package(..) => NameBindingKind::Package,
            Self::Module(..) => NameBindingKind::Module,
            Self::TypeAlias(..) => NameBindingKind::TypeAlias,
            Self::Function(..) => NameBindingKind::Function,
            Self::Interface(..) => NameBindingKind::Interface,
            Self::TupleLikeStruct(..) => NameBindingKind::TupleLikeStruct,
            Self::Struct(..) => NameBindingKind::Struct,
            Self::Enum(..) => NameBindingKind::Enum,
            Self::EnumItem(..) => NameBindingKind::EnumItem,
        }
    }

    /// Resolve the rest of the path. For instance, for the path `std.io.println`, `io.println` is
    /// the value of `other_identifiers`, and the `std` is the `first_identifier`.
    ///
    /// # Panics
    /// - If the name_resolver data is invalid.
    /// - If the path contains symbols, that cannot be resolved by an identifier interner.
    #[inline(always)]
    fn resolve_rest_of_the_path(
        self,
        first_identifier: IdentifierAST,
        other_identifiers: impl IntoIterator<Item = IdentifierAST>,
        identifier_interner: &IdentifierInterner,
        diagnostics: &RwLock<Diagnostics>,
        name_resolver: &NameResolver,
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
                        name_resolver,
                    )
                },
            )
    }
}

fn resolve_binding_in_module_namespace(
    module_id: ModuleID,
    namespace: IdentifierAST,
    name: IdentifierAST,
    identifier_interner: &IdentifierInterner,
    diagnostics: &RwLock<Diagnostics>,
    name_resolver: &NameResolver,
) -> Option<NameBinding> {
    let Some(binding) = name_resolver
        .module_scopes
        .get(&module_id)
        .unwrap()
        .bindings
        .get(&name.id)
    else {
        diagnostics.write().add_single_file_diagnostic(
            name.location.file_path_id,
            FailedToResolveModuleItem::new(
                identifier_interner
                    .resolve(namespace.id)
                    .unwrap()
                    .to_owned(),
                namespace.location,
                identifier_interner.resolve(name.id).unwrap().to_owned(),
                name.location,
            ),
        );

        return None;
    };

    if let NameBinding::Enum(definition_id)
    | NameBinding::Struct(definition_id)
    | NameBinding::Interface(definition_id)
    | NameBinding::Function(definition_id)
    | NameBinding::TypeAlias(definition_id) = binding
    {
        if *name_resolver.visibilities.get(definition_id).unwrap() == RawVisibility::Private {
            diagnostics.write().add_single_file_diagnostic(
                name.location.file_path_id,
                FailedToResolvePrivateModuleItem::new(
                    identifier_interner
                        .resolve(namespace.id)
                        .unwrap()
                        .to_owned(),
                    namespace.location,
                    identifier_interner.resolve(name.id).unwrap().to_owned(),
                    name.location,
                ),
            );

            return None;
        }
    }

    Some(*binding)
}

fn resolve_binding_in_module_item_namespace(
    item_definition_id: DefinitionID,
    namespace: IdentifierAST,
    name: IdentifierAST,
    identifier_interner: &IdentifierInterner,
    diagnostics: &RwLock<Diagnostics>,
    name_resolver: &NameResolver,
) -> Option<NameBinding> {
    if let Some(enum_scope) = name_resolver
        .module_scopes
        .get(&item_definition_id.module_id)?
        .enums
        .get(&item_definition_id.name_id)
    {
        enum_scope
            .items
            .get(&name.id)
            .copied()
            .map(NameBinding::EnumItem)
    } else {
        diagnostics.write().add_single_file_diagnostic(
            name.location.file_path_id,
            ModuleItemsExceptEnumsDoNotServeAsNamespaces::new(
                identifier_interner
                    .resolve(namespace.id)
                    .unwrap()
                    .to_owned(),
                namespace.location,
                identifier_interner.resolve(name.id).unwrap().to_owned(),
                name.location,
            ),
        );

        None
    }
}

fn resolve_binding_in_package_namespace(
    package_id: PackageID,
    name: IdentifierAST,
    identifier_interner: &IdentifierInterner,
    diagnostics: &RwLock<Diagnostics>,
    name_resolver: &NameResolver,
) -> Option<NameBinding> {
    let Some(module) = name_resolver.resolve_identifier_in_module_scope(
        name_resolver
            .module_scopes
            .get(
                name_resolver
                    .packages_root_modules
                    .get(&package_id)
                    .unwrap(),
            )
            // Module must exist at this point, or something went wrong when
            // building the name resolution name_resolver.
            .unwrap(),
        name,
        identifier_interner,
        diagnostics,
    ) else {
        diagnostics.write().add_single_file_diagnostic(
            name.location.file_path_id,
            FailedToResolveModule::new(
                name.location,
                identifier_interner.resolve(name.id).unwrap().to_owned(),
                name.location,
                identifier_interner.resolve(name.id).unwrap().to_owned(),
            ),
        );

        return None;
    };

    Some(module)
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
    namespace: IdentifierAST,
    name: IdentifierAST,
    identifier_interner: &IdentifierInterner,
    diagnostics: &RwLock<Diagnostics>,
    name_resolver: &NameResolver,
) -> Option<NameBinding> {
    match binding {
        NameBinding::Package(package_id) => resolve_binding_in_package_namespace(
            package_id,
            name,
            identifier_interner,
            diagnostics,
            name_resolver,
        ),
        NameBinding::EnumItem(_) => {
            // Enum items are not namespaces!

            diagnostics.write().add_single_file_diagnostic(
                name.location.file_path_id,
                ModuleItemsExceptEnumsDoNotServeAsNamespaces::new(
                    identifier_interner
                        .resolve(namespace.id)
                        .unwrap()
                        .to_owned(),
                    namespace.location,
                    identifier_interner.resolve(name.id).unwrap().to_owned(),
                    name.location,
                ),
            );

            None
        }
        NameBinding::Enum(definition_id)
        | NameBinding::TypeAlias(definition_id)
        | NameBinding::Struct(definition_id)
        | NameBinding::Function(definition_id)
        | NameBinding::Interface(definition_id)
        | NameBinding::TupleLikeStruct(definition_id) => resolve_binding_in_module_item_namespace(
            definition_id,
            namespace,
            name,
            identifier_interner,
            diagnostics,
            name_resolver,
        ),
        NameBinding::Module(module_id) => resolve_binding_in_module_namespace(
            module_id,
            namespace,
            name,
            identifier_interner,
            diagnostics,
            name_resolver,
        ),
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

/// Macro, that can be used to construct a path in tests:
///
/// # Example
///
/// ```
/// use ry_name_resolution::{path, Path};
/// use ry_interner::IdentifierID;
///
/// let a = IdentifierID(2);
/// let b = IdentifierID(3);
/// assert_eq!(path!(a, b), Path { identifiers: vec![a, b] });
/// ```
#[macro_export]
macro_rules! path {
    ($($id:expr),*) => {
        $crate::Path {
            identifiers: vec![$($id),*]
        }
    };
}

/// Data that Ry compiler has about a module.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ModuleScope {
    /// Interned name of the module.
    pub name: IdentifierID,

    /// ID of the module's source file path.
    pub id: ModuleID,

    /// Module path.
    pub path: Path,

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

/// Unique ID of the enum item
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct EnumItemID {
    /// ID of the enum definition.
    pub enum_definition_id: DefinitionID,

    /// Enum item symbol.
    pub item_id: IdentifierID,
}
