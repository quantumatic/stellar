//! # Typed AST
//!
//! Typed AST is a representation of a Ry program, that is produced
//! after the process called type inference.
//!
//! Typed AST is similar to the AST, but with type annotations.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]
#![warn(clippy::dbg_macro)]
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
    //rustdoc::missing_crate_level_docs,
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
    clippy::unnested_or_patterns
)]

use std::path::PathBuf;

use ry_ast::Visibility;
use ry_filesystem::location::Location;
use ry_fx_hash::FxHashMap;
use ry_interner::Symbol;
use ty::Type;

pub mod ty;

#[derive(Debug, PartialEq, Clone)]
pub struct TypePath {
    segments: Vec<TypePathSegment>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypePathSegment {
    left: Path,
    right: Vec<Type>,
}

pub type TypeBounds = Vec<TypePath>;

#[derive(Debug, PartialEq, Clone)]
pub struct Module {
    pub path: PathBuf,
    pub items: Vec<ModuleItem>,
}

/// A module item.
#[derive(Debug, PartialEq, Clone)]
pub enum ModuleItem {
    /// A type alias.
    Alias(TypeAliasModuleItem),

    /// An enum.
    Enum(EnumModuleItem),

    /// A trait.
    Trait(TraitModuleItem),

    /// A function (associated functions and implementations are not here).
    Function(Function),

    /// A struct.
    Struct(StructModuleItem),

    /// A tuple-like struct.
    TupleLikeStruct(TupleLikeStructModuleItem),
}

/// A type alias.
#[derive(Debug, PartialEq, Clone)]
pub struct TypeAliasModuleItem {
    /// Alias visibility.
    pub visibility: Visibility,

    /// Location of the alias name (not the entire alias item!).
    pub location: Location,

    /// Alias docstring.
    pub docstring: Option<String>,

    /// Alias generic parameters.
    pub generic_parameters: Vec<GenericParameter>,

    /// Alias value - `Result[T, E]` in `type Res[T] = Result[T, E]`.
    pub value: Type,
}

/// A trait.
#[derive(Debug, PartialEq, Clone)]
pub struct TraitModuleItem {
    /// Trait visibility.
    pub visibility: Visibility,

    /// Location of the trait name (not the entire trait item!).
    pub location: Location,

    /// Trait docstring.
    pub docstring: Option<String>,

    /// Trait generic parameters.
    pub generic_parameters: Vec<GenericParameter>,

    /// Type constraints.
    pub constraints: Vec<ConstraintPair>,

    /// Trait items.
    pub items: FxHashMap<Symbol, TraitItem>,

    /// All the trait implementations (implementations in the foreign
    /// modules and even projects are also here).
    pub implementations: TraitImplementation,
}

/// A trait item.
#[derive(Debug, PartialEq, Clone)]
pub enum TraitItem {
    /// A type alias item.
    Alias(TraitAliasTraitItem),

    /// A function item.
    ///
    /// Visibility here is ignored.
    Function(Function),
}

/// A type alias trait item.
#[derive(Debug, PartialEq, Clone)]
pub struct TraitAliasTraitItem {
    /// Location of the trait name (not the entire trait item!).
    pub location: Location,

    /// Alias docstring.
    pub docstring: Option<String>,

    /// Alias generic parameters.
    pub generic_parameters: Vec<GenericParameter>,

    /// Type constraints.
    pub constraints: Vec<ConstraintPair>,
}

/// A trait implementation.
#[derive(Debug, PartialEq, Clone)]
pub struct TraitImplementation {
    /// Path to the module in which the implementation lives (useful for diagnostics).
    pub module: Path,

    /// Location of the `impl` keyword.
    pub location: Location,

    /// Trait implementation docstring.
    pub docstring: Option<String>,

    /// Trait implementation generic parameters.
    pub generic_parameters: Vec<GenericParameter>,

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

/// A type implementation.
///
/// The difference between this and [`TraitImplementation`] is that
/// this struct corresponds to raw type implementations, without traits:
///
/// ```txt
/// impl[T] Foo[T] {} => TypeImplementation
/// impl[T, M] Foo[T] for M {} => TraitImplementation
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct TypeImplementation {
    /// Location of the `impl` keyword.
    pub location: Location,

    /// Type implementation docstring.
    pub docstring: Option<String>,

    /// Type implementation generic parameters.
    pub generic_parameters: Vec<GenericParameter>,

    /// Type constraints.
    pub constraints: Vec<ConstraintPair>,

    /// The type that is implemented.
    pub ty: Type,
}

/// A function.
#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    /// Function visibility.
    pub visibility: Visibility,

    /// Location of the function name (not the entire function item!).
    pub location: Location,

    /// Function docstring.
    pub docstring: Option<String>,

    /// Function generic parameters.
    pub generic_parameters: Vec<GenericParameter>,

    /// Type constraints.
    pub constraints: Vec<ConstraintPair>,

    /// Function parameters.
    pub parameters: Vec<FunctionParameter>,

    /// Function return type (is it's not written in the signature, then it's `()`).
    pub return_type: Type,
}

/// A function parameter.
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionParameter {
    /// Parameter name.
    pub name: Symbol,

    /// Parameter type.
    pub ty: Type,
}

/// A struct.
#[derive(Debug, PartialEq, Clone)]
pub struct StructModuleItem {
    /// Struct visibility.
    pub visibility: Visibility,

    /// Location of the struct name (not the entire struct item!).
    pub location: Location,

    /// Struct docstring.
    pub docstring: Option<String>,

    /// Struct generic parameters.
    pub generic_parameters: Vec<GenericParameter>,

    /// Type constraints.
    pub constraints: Vec<ConstraintPair>,

    /// Struct fields.
    pub fields: FxHashMap<Symbol, StructField>,

    /// All the struct raw type implementations (implementations in the foreign
    /// modules and even projects are also here).
    pub implementations: TraitImplementation,
}

/// A struct field.
#[derive(Debug, PartialEq, Clone)]
pub struct StructField {
    /// Field visibility.
    pub visibility: Visibility,

    /// Location of the field name (not the entire field item!).
    pub location: Location,

    /// Field docstring.
    pub docstring: Option<String>,

    /// Field name
    pub name: Symbol,

    /// Field type.
    pub ty: Type,
}

/// A tuple-like struct.
#[derive(Debug, PartialEq, Clone)]
pub struct TupleLikeStructModuleItem {
    /// Struct visibility.
    pub visibility: Visibility,

    /// Location of the struct name (not the entire struct item!).
    pub location: Location,

    /// Struct docstring.
    pub docstring: Option<String>,

    /// Struct generic parameters.
    pub generic_parameters: Vec<GenericParameter>,

    /// Type constraints.
    pub constraints: Vec<ConstraintPair>,

    /// Struct fields.
    pub fields: FxHashMap<Symbol, TupleLikeStructField>,
    pub implementations: TraitImplementation,
}

/// A tuple-like struct field.
#[derive(Debug, PartialEq, Clone)]
pub struct TupleLikeStructField {
    /// Field visibility.
    pub visibility: Visibility,

    /// Location of the entire field.
    pub location: Location,

    /// Field type.
    pub ty: Type,
}

/// An enum.
#[derive(Debug, PartialEq, Clone)]
pub struct EnumModuleItem {
    /// Enum visibility.
    pub visibility: Visibility,

    /// Location of the enum name (not the entire enum item!).
    pub location: Location,

    /// Enum docstring.
    pub docstring: Option<String>,

    /// Enum generic parameters.
    pub generic_parameters: Vec<GenericParameter>,

    /// Type constraints.
    pub constraints: Vec<ConstraintPair>,

    /// Enum items.
    pub items: FxHashMap<Symbol, EnumItem>,

    /// All the enum raw type implementations (implementations in the foreign
    /// modules and even projects are also here).
    pub implementations: TraitImplementation,
}

/// An enum item.
#[derive(Debug, PartialEq, Clone)]
pub enum EnumItem {
    /// An identifier item, e.g. `None` in `Option[T]`.
    Identifier {
        /// Location of the name.
        location: Location,
    },

    /// A tuple like item.
    TupleLike {
        /// Location of the item name (not the entire item).
        location: Location,

        /// Fields.
        fields: FxHashMap<Symbol, TupleLikeStructField>,
    },

    /// A struct item.
    Struct {
        /// Location of the item name (not the entire item).
        location: Location,

        /// Fields.
        fields: FxHashMap<Symbol, StructField>,
    },
}

/// A generic parameter.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct GenericParameter {
    /// Location of the generic parameter name.
    ///
    /// ```txt
    /// fun foo[T: Into[String]]()
    ///         ^
    /// ```
    pub location: Location,

    /// Generic parameter name.
    pub name: Symbol,
}

/// A type constraint.
#[derive(Debug, PartialEq, Clone)]
pub enum ConstraintPair {
    Satisfies {
        /// The type that must satisfy the bounds.
        ty: Type,

        /// Location of the type that must satisfy the bounds.
        ty_location: Location,

        /// The bounds.
        bounds: TypeBounds,

        /// Location of the bounds.
        bounds_location: Location,
    },
    Eq {
        /// The left hand side type.
        left: Type,

        /// Location of the left hand side type.
        left_location: Location,

        /// The right hand side type.
        right: Type,

        /// Location of the right hand side type.
        right_location: Location,
    },
}

/// A path similiar to [`ry_ast::Path`], but which doesn't store any locations,
/// e.g. `std.path.Path`, `foo`, `json.serializer`.
#[derive(Debug, PartialEq, Eq, Clone)]
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
