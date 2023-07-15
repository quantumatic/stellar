//! # Name resolution
//!
//! The name resolution allows to resolve names, after parsing all the projects in stages like
//! type checking and MIR lowering.
//!
//! See [`GlobalContext`], [`ProjectContext`] and [`ModuleContext`] for more details.

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

use ry_ast::{IdentifierAst, Impl, ImportPath, ModuleItem};
use ry_filesystem::{location::Location, path_interner::PathID};
use ry_fx_hash::FxHashMap;
use ry_interner::Symbol;
use ry_typed_ast::{ty::Type, Path};

pub mod build_context;
pub mod diagnostics;

/// A symbol data, in which types in a definition are processed, once the the
/// definition is used somewhere else. This approach allows to resolve forward
/// references.
#[derive(Debug, PartialEq, Clone)]
pub struct GlobalContext {
    /// Projects, that are going to be resolved.
    pub projects: FxHashMap<Symbol, ProjectContext>,
}

impl Default for GlobalContext {
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalContext {
    /// Creates new name empty resolution tree.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            projects: FxHashMap::default(),
        }
    }

    /// Resolves absolute path.
    ///
    /// * Path must start with a project name (not `serialize`, but `json.serialization.serialize`).
    /// * Imports are not resolved here, because the context is global.
    #[must_use]
    pub fn resolve_module_item_by_absolute_path(&self, path: &Path) -> Option<NameBindingData<'_>> {
        fn split_first_and_last<T>(a: &[T]) -> Option<(&T, &[T], &T)> {
            let (first, rest) = a.split_first()?;
            let (last, middle) = rest.split_last()?;

            Some((first, middle, last))
        }

        let (first, middle, last) = split_first_and_last(&path.symbols)?;

        // json.serialization.serialize
        // ^^^^ module before
        let mut module = &self.projects.get(first)?.root;

        // json.serialization.serialize
        //      ^^^^^^^^^^^^^ module after
        for symbol in middle {
            module = module.submodules.get(symbol)?;
        }

        // json.serialization.serialize
        //                    ^^^^^^^^^ binding or submodule
        if let Some(binding) = module.bindings.get(last) {
            return Some(NameBindingData::Item(binding));
        } else if let Some(submodule) = module.submodules.get(last) {
            return Some(NameBindingData::Module(submodule));
        }

        None
    }
}

/// Data that Ry compiler has about a project.
#[derive(Debug, PartialEq, Clone)]
pub struct ProjectContext {
    /// Path to the project.
    pub path_id: PathID,

    /// The root module of the project (the module that is located in the `project.ry`).
    pub root: ModuleContext,

    /// The dependencies of the project (must be included in the resolution tree).
    pub dependencies: Vec<Symbol>,
}

/// Data that Ry compiler has about a module.
#[derive(Debug, PartialEq, Clone)]
pub struct ModuleContext {
    /// ID of the path of the module file/folder.
    pub path_id: PathID,

    /// The module docstring.
    pub docstring: Option<String>,

    /// The module items name bindings.
    ///
    /// See [`ModuleItemNameBindingData`] for more details.
    pub bindings: FxHashMap<Symbol, ModuleItemNameBindingData>,

    /// The submodules of the module.
    pub submodules: FxHashMap<Symbol, ModuleContext>,

    /// The type implementations, that are not yet analyzed (type checked) in the module.
    pub implementations: Vec<Impl>,

    /// The imports used in the module ([`Span`] stores a location of an entire import item).
    pub imports: Vec<(Location, ImportPath)>,
}

impl ModuleContext {
    /// Resolves path and returns binding data (may return submodule).
    #[must_use]
    pub fn resolve_module_item_path<'ctx>(
        &'ctx self,
        path: &Path,
        tree: &'ctx GlobalContext,
    ) -> Option<NameBindingData<'ctx>> {
        // serializer.serialize
        // ^^^^^^^^^^ first symbol
        //            ^^^^^^^^^ rest
        let (first_path_symbol, rest) = path.symbols.split_first()?;

        // serializer.serialize
        // ^^^^^^^^^^ module
        let first_symbol_resolved = self.resolve_symbol(*first_path_symbol, tree)?;

        match first_symbol_resolved {
            NameBindingData::Module(module) => {
                if rest.is_empty() {
                    Some(NameBindingData::Module(module))
                } else {
                    module.resolve_module_item_path(
                        &Path {
                            symbols: rest.to_vec(),
                        },
                        tree,
                    )
                }
            }
            // serializer.serialize
            //            ^^^^^^^^^ function item - result
            NameBindingData::Item(item) => {
                if rest.is_empty() {
                    Some(NameBindingData::Item(item))
                } else {
                    None
                }
            }
        }
    }

    /// Resolves a single symbol and returns binding data.
    fn resolve_symbol<'ctx>(
        &'ctx self,
        symbol: Symbol,
        tree: &'ctx GlobalContext,
    ) -> Option<NameBindingData<'ctx>> {
        // If symbol is related to an item defined in the module, return it.
        //
        // ```
        // fun foo() {}
        // fun main() { foo(); }
        //              ^^^ function item
        // ```
        if let Some(binding) = self.bindings.get(&symbol).map(NameBindingData::Item) {
            return Some(binding);
        }

        // If not found, try to find it in the imports.
        //
        // ```
        // fun main() { foo(); }
        // ```
        for (_, import) in &self.imports {
            if let Some(IdentifierAst {
                symbol: id_symbol, ..
            }) = import.r#as
            {
                // ```
                // import a.b as foo;
                //               ^^^ function item
                // ```
                if id_symbol != symbol {
                    continue;
                }

                return tree.resolve_module_item_by_absolute_path(&import.path.clone().into());
            }

            // ```
            // import a.foo;
            //          ^^^ function item
            // ```
            let import_last_symbol = import.path.identifiers.last()?.symbol;

            if import_last_symbol != symbol {
                continue;
            }

            return tree.resolve_module_item_by_absolute_path(&import.path.clone().into());
        }

        None
    }
}

/// Data that Ry compiler has about a name binding or the result of the
/// [`ModuleData::resolve_path()`] and [`NameResolutionTree::resolve_absolute_path()`].
#[derive(Debug, PartialEq, Clone)]
pub enum NameBindingData<'ctx> {
    /// A module.
    Module(&'ctx ModuleContext),

    /// A module item.
    Item(&'ctx ModuleItemNameBindingData),
}

/// Data that Ry compiler has about a module item.
#[derive(Debug, PartialEq, Clone)]
pub enum ModuleItemNameBindingData {
    /// An item that has gone through type checking.
    Analyzed(ry_typed_ast::ModuleItem),

    /// An item that has not been analyzed yet - AST ref.
    NotAnalyzed(ModuleItem),
}

/// Data that Ry compiler has about a particular symbol.
#[derive(Debug, Clone, PartialEq)]
pub struct ValueConstructor {
    /// Span where the symbol was defined.
    pub origin: Location,

    /// Type of the symbol.
    pub ty: Type,
}

/// A local scope (a scope within a particular statements block).
#[derive(Debug)]
pub struct Scope<'ctx> {
    /// Module that the scope belongs to.
    pub module_context: &'ctx ModuleContext,

    /// Symbols in the scope (not the ones contained in the parent scopes).
    entities: FxHashMap<Symbol, ValueConstructor>,

    /// Parent scope.
    pub parent: Option<&'ctx Scope<'ctx>>,
}

impl<'ctx> Scope<'ctx> {
    /// Creates a new [`Scope`] instance.
    #[inline]
    #[must_use]
    pub fn new(parent: Option<&'ctx Scope<'ctx>>, module_context: &'ctx ModuleContext) -> Self {
        Self {
            entities: FxHashMap::default(),
            parent,
            module_context,
        }
    }

    /// Adds a symbol to this scope.
    pub fn add_symbol(&mut self, symbol: Symbol, data: ValueConstructor) {
        // shadowing
        if self.entities.contains_key(&symbol) {
            self.entities.remove(&symbol);
        }

        self.entities.insert(symbol, data);
    }

    /// Returns the symbol data for the given symbol. If the symbol is not in this scope, `None` is returned.
    #[must_use]
    pub fn lookup(&self, symbol: Symbol) -> Option<&ValueConstructor> {
        if let data @ Some(..) = self.entities.get(&symbol) {
            data
        } else if let Some(parent) = self.parent {
            parent.lookup(symbol)
        } else {
            None
        }
    }
}
