use std::path::PathBuf;

use ry_ast::{IdentifierAst, Impl, ImportPath, ModuleItem, Visibility};
use ry_filesystem::span::Span;
use ry_fx_hash::FxHashMap;
use ry_interner::Symbol;
use ry_typed_ast::{ty::Type, TypeBounds};

pub mod build_resolution_tree;
pub mod diagnostics;

/// A name resolution tree - data structure used to resolve names.
#[derive(Debug, PartialEq, Clone)]
pub struct NameResolutionTree<'ast> {
    /// Projects, that are going to be resolved.
    pub projects: FxHashMap<Symbol, ProjectData<'ast>>,
}

impl Default for NameResolutionTree<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'ast> NameResolutionTree<'ast> {
    /// Creates new name empty resolution tree.
    pub fn new() -> Self {
        NameResolutionTree {
            projects: FxHashMap::default(),
        }
    }

    /// Resolves absolute path.
    ///
    /// * Path must start with a project name (not `serialize`, but `json.serialization.serialize`).
    /// * Imports are not resolved here, because the context is global.
    pub fn resolve_absolute_path(&self, path: Path) -> Option<NameBindingData<'_, 'ast>> {
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
        } else {
            return None;
        }
    }
}

/// Data that Ry compiler has about a project.
#[derive(Debug, PartialEq, Clone)]
pub struct ProjectData<'ast> {
    /// Path to the project.
    pub path: PathBuf,

    /// The root module of the project (the module that is located in the `project.ry`).
    pub root: ModuleData<'ast>,

    /// The dependencies of the project (must be included in the resolution tree).
    pub dependencies: Vec<Symbol>,
}

/// Data that Ry compiler has about a module.
#[derive(Debug, PartialEq, Clone)]
pub struct ModuleData<'ast> {
    /// Path to the module file.
    pub path: PathBuf,

    /// The module docstring.
    pub docstring: Option<&'ast str>,

    /// The module items name bindings.
    ///
    /// See [`ModuleItemNameBindingData`] for more details.
    pub bindings: FxHashMap<Symbol, ModuleItemNameBindingData<'ast>>,

    /// The submodules of the module.
    pub submodules: FxHashMap<Symbol, ModuleData<'ast>>,

    /// The type implementations, that are not yet analyzed (type checked) in the module.
    pub implementations: Vec<&'ast Impl>,

    /// The imports used in the module ([`Span`] stores a location of an entire import item).
    pub imports: Vec<(Span, &'ast ImportPath)>,
}

impl<'ast> ModuleData<'ast> {
    /// Resolves path and returns binding data (may return submodule).
    pub fn resolve_path<'tree>(
        &'tree self,
        path: Path,
        tree: &'tree NameResolutionTree<'ast>,
    ) -> Option<NameBindingData<'tree, 'ast>> {
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
                    module.resolve_path(
                        Path {
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
    fn resolve_symbol<'tree>(
        &'tree self,
        symbol: Symbol,
        tree: &'tree NameResolutionTree<'ast>,
    ) -> Option<NameBindingData<'tree, 'ast>> {
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
            match import.r#as {
                // ```
                // import a.b as foo;
                //               ^^^ function item
                // ```
                Some(IdentifierAst {
                    symbol: id_symbol, ..
                }) => {
                    if id_symbol != symbol {
                        continue;
                    }

                    return tree.resolve_absolute_path(import.path.clone().into());
                }
                // ```
                // import a.foo;
                //          ^^^ function item
                // ```
                None => {
                    let import_last_symbol = import.path.identifiers.last()?.symbol;

                    if import_last_symbol != symbol {
                        continue;
                    }

                    return tree.resolve_absolute_path(import.path.clone().into());
                }
            }
        }

        None
    }
}

/// Data that Ry compiler has about a name binding or the result of the
/// [`ModuleData::resolve_path()`] and [`NameResolutionTree::resolve_absolute_path()`].
#[derive(Debug, PartialEq, Clone)]
pub enum NameBindingData<'tree, 'ast> {
    /// A module.
    Module(&'tree ModuleData<'ast>),

    /// A module item.
    Item(&'tree ModuleItemNameBindingData<'ast>),
}

/// Data that Ry compiler has about a module item.
#[derive(Debug, PartialEq, Clone)]
pub enum ModuleItemNameBindingData<'ast> {
    /// An item that has gone through type checking.
    Analyzed(AnalyzedNameBindingData<'ast>),

    /// An item that has not been analyzed yet - AST ref.
    NotAnalyzed(&'ast ModuleItem),
}

/// Data that Ry compiler has about a module item that has gone through type checking.
#[derive(Debug, PartialEq, Clone)]
pub enum AnalyzedNameBindingData<'ast> {
    /// Data about a type alias.
    Alias(AliasModuleItemData<'ast>),

    /// Data about an enum.
    Enum(EnumData<'ast>),

    /// Data about a trait.
    Trait(TraitData<'ast>),

    /// Data about a function (associated functions and
    /// implementations are not here).
    Function(FunctionData<'ast>),

    /// Data about a struct.
    Struct(StructData<'ast>),

    /// Data about a tuple-like struct.
    TupleLikeStruct(TupleLikeStructData<'ast>),
}

/// Data that Ry compiler has about a type alias.
#[derive(Debug, PartialEq, Clone)]
pub struct AliasModuleItemData<'ast> {
    /// Alias visibility.
    pub visibility: Visibility,

    /// Location of the alias name (not the entire alias item!).
    pub span: Span,

    /// Alias docstring.
    pub docstring: Option<&'ast str>,

    /// Alias generic parameters.
    pub generic_parameters: Vec<GenericParameterData>,

    /// Alias value - `Result[T, E]` in `type Res[T] = Result[T, E]`.
    pub value: Type,
}

/// Data that Ry compiler has about a trait.
#[derive(Debug, PartialEq, Clone)]
pub struct TraitData<'ast> {
    /// Trait visibility.
    pub visibility: Visibility,

    /// Location of the trait name (not the entire trait item!).
    pub span: Span,

    /// Trait docstring.
    pub docstring: Option<&'ast str>,

    /// Trait generic parameters.
    pub generic_parameters: Vec<GenericParameterData>,

    /// Type constraints.
    pub constraints: Vec<ConstraintPair>,

    /// Trait items.
    pub items: FxHashMap<Symbol, TraitItemData<'ast>>,

    /// All the trait implementations (implementations in the foreign
    /// modules and even projects are also here).
    pub implementations: TraitImplementationData<'ast>,
}

/// Data that Ry compiler has about a trait item.
#[derive(Debug, PartialEq, Clone)]
pub enum TraitItemData<'ast> {
    /// Data about a type alias item.
    Alias {
        /// Location of the alias name (not the entire alias item!).
        span: Span,

        /// Alias docstring.
        docstring: Option<&'ast str>,

        /// Alias generic parameters.
        generic_parameters: Vec<GenericParameterData>,

        /// Type constraints.
        constraints: Vec<ConstraintPair>,
    },

    /// Data about a function item.
    ///
    /// Visibility here is ignored.
    Function(FunctionData<'ast>),
}

/// Data that Ry compiler has about a trait implementation.
#[derive(Debug, PartialEq, Clone)]
pub struct TraitImplementationData<'ast> {
    /// Path to the module in which the implementation lives (useful for diagnostics).
    pub module: Path,

    /// Location of the `impl` keyword.
    pub span: Span,

    /// Trait implementation docstring.
    pub docstring: Option<&'ast str>,

    /// Trait implementation generic parameters.
    pub generic_parameters: Vec<GenericParameterData>,

    /// Trait generic arguments.
    ///
    /// ```txt
    /// impl[T, M] Foo[T] for M {}
    ///                ^
    /// ```
    pub trait_generic_arguments: Vec<Type>,

    /// Type constraints.
    pub constraints: Vec<ConstraintPair>,

    /// The type for which the trait is implemented.
    pub ty: Type,
}

/// Data that Ry compiler has about a type implementation.
///
/// The difference between this and [`TraitImplementationData`] is that
/// this struct corresponds to raw type implementations, without traits:
///
/// ```txt
/// impl[T] Foo[T] {} => TypeImplementationData
/// impl[T, M] Foo[T] for M {} => TraitImplementationData
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct TypeImplementationData<'ast> {
    /// Location of the `impl` keyword.
    pub span: Span,

    /// Type implementation docstring.
    pub docstring: Option<&'ast str>,

    /// Type implementation generic parameters.
    pub generic_parameters: Vec<GenericParameterData>,

    /// Type constraints.
    pub constraints: Vec<ConstraintPair>,

    /// The type that is implemented.
    pub ty: Type,
}

/// Data that Ry compiler has about a function.
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionData<'ast> {
    /// Function visibility.
    pub visibility: Visibility,

    /// Location of the function name (not the entire function item!).
    pub span: Span,

    /// Function docstring.
    pub docstring: Option<&'ast str>,

    /// Function generic parameters.
    pub generic_parameters: Vec<GenericParameterData>,

    /// Type constraints.
    pub constraints: Vec<ConstraintPair>,

    /// Function parameters.
    pub parameters: Vec<FunctionParameterData>,

    /// Function return type (is it's not written in the signature, then it's `()`).
    pub return_type: Type,
}

/// Data that Ry compiler has about a function parameter.
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionParameterData {
    /// Parameter name.
    pub name: Symbol,

    /// Parameter type.
    pub ty: Type,
}

/// Data that Ry compiler has about a struct.
#[derive(Debug, PartialEq, Clone)]
pub struct StructData<'ast> {
    /// Struct visibility.
    pub visibility: Visibility,

    /// Location of the struct name (not the entire struct item!).
    pub span: Span,

    /// Struct docstring.
    pub docstring: Option<&'ast str>,

    /// Struct generic parameters.
    pub generic_parameters: Vec<GenericParameterData>,

    /// Type constraints.
    pub constraints: Vec<ConstraintPair>,

    /// Struct fields.
    pub fields: FxHashMap<Symbol, StructFieldData<'ast>>,

    /// All the struct raw type implementations (implementations in the foreign
    /// modules and even projects are also here).
    pub implementations: TraitImplementationData<'ast>,
}

/// Data that Ry compiler has about a struct field.
#[derive(Debug, PartialEq, Clone)]
pub struct StructFieldData<'ast> {
    /// Field visibility.
    pub visibility: Visibility,

    /// Location of the field name (not the entire field item!).
    pub span: Span,

    /// Field docstring.
    pub docstring: Option<&'ast str>,

    /// Field name
    pub name: Symbol,

    /// Field type.
    pub ty: Type,
}

/// Data that Ry compiler has about a tuple-like struct.
#[derive(Debug, PartialEq, Clone)]
pub struct TupleLikeStructData<'ast> {
    /// Struct visibility.
    pub visibility: Visibility,

    /// Location of the struct name (not the entire struct item!).
    pub span: Span,

    /// Struct docstring.
    pub docstring: Option<&'ast str>,

    /// Struct generic parameters.
    pub generic_parameters: Vec<GenericParameterData>,

    /// Type constraints.
    pub constraints: Vec<ConstraintPair>,

    /// Struct fields.
    pub fields: FxHashMap<Symbol, TupleLikeStructFieldData>,
    pub implementations: TraitImplementationData<'ast>,
}

/// Data that Ry compiler has about a tuple-like struct field.
#[derive(Debug, PartialEq, Clone)]
pub struct TupleLikeStructFieldData {
    /// Field visibility.
    pub visibility: Visibility,

    /// Location of the entire field.
    pub span: Span,

    /// Field type.
    pub ty: Type,
}

/// Data that Ry compiler has about an enum.
#[derive(Debug, PartialEq, Clone)]
pub struct EnumData<'ast> {
    /// Enum visibility.
    pub visibility: Visibility,

    /// Location of the enum name (not the entire enum item!).
    pub span: Span,

    /// Enum docstring.
    pub docstring: Option<&'ast str>,

    /// Enum generic parameters.
    pub generic_parameters: Vec<GenericParameterData>,

    /// Type constraints.
    pub constraints: Vec<ConstraintPair>,

    /// Enum items.
    pub items: FxHashMap<Symbol, EnumItemData<'ast>>,

    /// All the enum raw type implementations (implementations in the foreign
    /// modules and even projects are also here).
    pub implementations: TraitImplementationData<'ast>,
}

/// Data that Ry compiler has about an enum item.
#[derive(Debug, PartialEq, Clone)]
pub enum EnumItemData<'ast> {
    /// Data about an identifier item, e.g. `None` in `Option[T]`.
    Identifier {
        /// Location of the name.
        span: Span,
    },

    /// Data about a tuple like item.
    TupleLike {
        /// Location of the item name (not the entire item).
        span: Span,

        /// Fields.
        fields: FxHashMap<Symbol, TupleLikeStructFieldData>,
    },

    /// Data about a struct item.
    Struct {
        /// Location of the item name (not the entire item).
        span: Span,

        /// Fields.
        fields: FxHashMap<Symbol, StructFieldData<'ast>>,
    },
}

/// Data that Ry compiler has about a generic parameter.
#[derive(Debug, PartialEq, Clone)]
pub struct GenericParameterData {
    /// Location of the generic parameter name.
    ///
    /// ```txt
    /// fun foo[T: Into[String]]()
    ///         ^
    /// ```
    pub span: Span,

    /// Generic parameter name.
    pub name: Symbol,
}

/// Data that Ry compiler has about a constraint.
#[derive(Debug, PartialEq, Clone)]
pub enum ConstraintPair {
    Satisfies {
        /// The type that must satisfy the bounds.
        ty: Type,

        /// Location of the type that must satisfy the bounds.
        ty_span: Span,

        /// The bounds.
        bounds: TypeBounds,

        /// Location of the bounds.
        bounds_span: Span,
    },
    Eq {
        /// The left hand side type.
        left: Type,

        /// Location of the left hand side type.
        left_span: Span,

        /// The right hand side type.
        right: Type,

        /// Location of the right hand side type.
        right_span: Span,
    },
}

/// A path similiar to [`ry_ast::Path`], but which doesn't store any spans,
/// e.g. `std.path.Path`, `foo`, `json.serializer`.
#[derive(Debug, PartialEq, Clone)]
pub struct Path {
    /// Path symbols.
    pub symbols: Vec<Symbol>,
}

impl From<ry_ast::Path> for Path {
    fn from(value: ry_ast::Path) -> Self {
        Self {
            symbols: value.identifiers.iter().map(|i| i.symbol).collect(),
        }
    }
}
