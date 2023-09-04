#![doc(
    html_logo_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png",
    html_favicon_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png"
)]

use derive_more::Display;
use paste::paste;
use stellar_ast::{IdentifierAST, ModuleItemKind, Visibility};
use stellar_diagnostics::Diagnostics;
use stellar_filesystem::location::{Location, DUMMY_LOCATION};
use stellar_fx_hash::FxHashMap;
use stellar_interner::{IdentifierID, PathID};
use stellar_thir::ty::{Type, TypeConstructor};

macro_rules! define_symbol_struct {
    ($($name:ident),*) => {
        paste! {
            /// A symbol's unique ID.
            #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
            pub enum Symbol {
                $(
                    [<$name:camel>]([<$name:camel ID>]),
                )*
            }

            impl Symbol {
                $(
                    #[doc = "Returns `true` if the symbol is a [`Symbol::" [<$name:camel>] "`]."]
                    #[doc = ""]
                    #[doc = "_This function is automatically generated by a macro._"]
                    #[inline(always)]
                    #[must_use]
                    pub const fn [<is_ $name>](&self) -> bool {
                        matches!(self, Self::[<$name:camel>](_))
                    }

                    #[doc = "Returns [`" [<$name:camel ID>] "`] if the symbol is a [`Symbol::" [<$name:camel>] "`]."]
                    #[doc = ""]
                    #[doc = "_This function is automatically generated by a macro._"]
                    #[inline(always)]
                    #[must_use]
                    pub fn [<to_ $name _or_none>](self) -> Option<[<$name:camel ID>]> {
                        match self {
                            Self::[<$name:camel>](id) => Some(id),
                            _ => None
                        }
                    }

                    #[doc = "Returns [`" [<$name:camel ID>] "`] if the symbol is a [`Symbol::" [<$name:camel>] "`]."]
                    #[doc = "# Panics"]
                    #[doc = "Panics if the symbol is not [`Symbol::" [<$name:camel>] "`]."]
                    #[doc = ""]
                    #[doc = "_This function is automatically generated by a macro._"]
                    #[inline(always)]
                    #[must_use]
                    pub fn [<to_ $name>](self) -> [<$name:camel ID>] {
                        self.[<to_ $name _or_none>]().unwrap()
                    }
                )*
            }
        }
    };
}

define_symbol_struct! {
    module,
    enum,
    struct,
    function,
    interface,
    tuple_like_struct,
    type_alias,
    enum_item
}

impl Symbol {
    /// Returns the name of the symbol.
    #[inline(always)]
    #[must_use]
    pub fn name(self, db: &Database) -> IdentifierAST {
        match self {
            Self::Module(module) => IdentifierAST {
                location: DUMMY_LOCATION,
                id: db.module(module).name,
            },
            Self::Enum(enum_) => enum_.signature(db).name(db),
            Self::Struct(struct_) => struct_.signature(db).name(db),
            Self::Function(function) => function.signature(db).name(db),
            Self::Interface(interface) => interface.signature(db).name(db),
            Self::TupleLikeStruct(struct_) => struct_.signature(db).name(db),
            Self::TypeAlias(alias) => alias.signature(db).name(db),
            Self::EnumItem(item) => item.name(db),
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn module_item_kind_or_none(self) -> Option<ModuleItemKind> {
        match self {
            Self::Enum(_) => Some(ModuleItemKind::Enum),
            Self::Struct(_) => Some(ModuleItemKind::Struct),
            Self::Function(_) => Some(ModuleItemKind::Function),
            Self::Interface(_) => Some(ModuleItemKind::Interface),
            Self::TupleLikeStruct(_) => Some(ModuleItemKind::TupleLikeStruct),
            Self::TypeAlias(_) => Some(ModuleItemKind::TypeAlias),
            Self::EnumItem(_) | Self::Module(_) => None,
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn module_item_kind(self) -> ModuleItemKind {
        self.module_item_kind_or_none().unwrap()
    }
}

/// A data that Stellar compiler has about an enum.
#[derive(Debug)]
pub struct EnumData {
    pub signature: SignatureID,
    pub items: FxHashMap<IdentifierID, EnumItemID>,
    pub methods: FxHashMap<IdentifierID, FunctionID>,
}

impl EnumData {
    /// Creates a new enum data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(db: &mut Database, signature: SignatureID) -> EnumID {
        db.add_enum(Self::new(signature))
    }

    /// Creates a new enum data object.
    #[inline(always)]
    #[must_use]
    pub fn new(signature: SignatureID) -> Self {
        Self {
            signature,
            items: FxHashMap::default(),
            methods: FxHashMap::default(),
        }
    }
}

/// A unique ID that maps to [`EnumData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct EnumID(pub usize);

impl EnumID {
    /// Returns a list of predicates associated with the enum.
    #[inline(always)]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureID {
        db.enum_(self).signature
    }

    /// Returns a list of items associated with the enum.
    #[inline(always)]
    #[must_use]
    pub fn items(self, db: &Database) -> &FxHashMap<IdentifierID, EnumItemID> {
        &db.enum_(self).items
    }

    /// Returns `true` if an item with a given name is contained in the enum definition.
    #[inline(always)]
    #[must_use]
    pub fn contains_item(self, db: &Database, name: IdentifierID) -> bool {
        db.enum_(self).items.contains_key(&name)
    }

    /// Returns an item with a given name.
    pub fn item(self, db: &Database, name: IdentifierID) -> Option<EnumItemID> {
        db.enum_(self).items.get(&name).copied()
    }
}

/// A data that Stellar compiler has about a struct.
#[derive(Debug)]
pub struct StructData {
    pub signature: SignatureID,
    pub fields: FxHashMap<IdentifierID, FieldID>,
    pub methods: FxHashMap<IdentifierID, FunctionID>,
}

impl StructData {
    /// Creates a new struct data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(db: &mut Database, signature: SignatureID) -> StructID {
        db.add_struct(Self::new(signature))
    }

    /// Creates a new struct data object.
    #[inline(always)]
    #[must_use]
    pub fn new(signature: SignatureID) -> Self {
        Self {
            signature,
            fields: FxHashMap::default(),
            methods: FxHashMap::default(),
        }
    }
}

/// A unique ID that maps to [`StructData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct StructID(pub usize);

impl StructID {
    /// Returns a list of predicates associated with the struct.
    #[inline(always)]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureID {
        db.struct_(self).signature
    }

    /// Returns a list of fields associated with the struct.
    #[inline(always)]
    #[must_use]
    pub fn fields(self, db: &Database) -> &FxHashMap<IdentifierID, FieldID> {
        &db.struct_(self).fields
    }
}

/// A data that Stellar compiler has about a function.
#[derive(Debug)]
pub struct TupleLikeStructData {
    pub signature: SignatureID,
    pub fields: Vec<(Visibility, Type)>,
}

impl TupleLikeStructData {
    /// Creates a new tuple-like struct data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(db: &mut Database, signature: SignatureID) -> TupleLikeStructID {
        db.add_tuple_like_struct(Self::new(signature))
    }

    /// Creates a new tuple-like struct data object.
    #[inline(always)]
    #[must_use]
    pub fn new(signature: SignatureID) -> Self {
        Self {
            signature,
            fields: Vec::new(),
        }
    }
}

/// A unique ID that maps to [`TupleLikeStructData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct TupleLikeStructID(pub usize);

impl TupleLikeStructID {
    /// Returns the type signature of the struct.
    #[inline(always)]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureID {
        db.tuple_like_struct(self).signature
    }
}

/// A data that Stellar compiler has about a field.
#[derive(Debug)]
pub struct FieldData {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub ty: Type,
}

impl FieldData {
    /// Creates a new field data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(
        db: &mut Database,
        visibility: Visibility,
        name: IdentifierAST,
        ty: Type,
    ) -> FieldID {
        db.add_field(Self::new(visibility, name, ty))
    }

    /// Creates a new field data object.
    #[inline(always)]
    #[must_use]
    pub fn new(visibility: Visibility, name: IdentifierAST, ty: Type) -> Self {
        Self {
            visibility,
            name,
            ty,
        }
    }
}

/// A unique ID that maps to [`FieldData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct FieldID(pub usize);

/// A data that Stellar compiler has about a predicate.
#[derive(Debug)]
pub struct PredicateData {
    pub ty: Type,
    pub bounds: Vec<TypeConstructor>,
}

impl PredicateData {
    /// Creates a new predicate data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(db: &mut Database, ty: Type, bounds: Vec<TypeConstructor>) -> PredicateID {
        db.add_predicate(Self::new(ty, bounds))
    }

    /// Creates a new predicate data object.
    #[inline(always)]
    #[must_use]
    pub fn new(ty: Type, bounds: Vec<TypeConstructor>) -> Self {
        Self { ty, bounds }
    }
}

/// A unique ID that maps to [`PredicateData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct PredicateID(pub usize);

/// A data that Stellar compiler has about a generic parameter scope.
#[derive(Default, PartialEq, Clone, Debug)]
pub struct GenericParameterScopeData {
    /// A parent scope, for example:
    ///
    /// ```stellar
    /// interface Foo[T] { // self.parent = Scope { parent: None, parameters: [T] }
    ///     fun bar[M]();  // self = Scope { parent: ..., parameters: [M] }
    /// }
    /// ```
    pub parent_scope: Option<GenericParameterScopeID>,

    /// A map of generic parameters in the scope.
    pub parameters: FxHashMap<IdentifierID, GenericParameterID>,
}

impl GenericParameterScopeData {
    /// Creates a new generic parameter scope data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(db: &mut Database) -> GenericParameterScopeID {
        db.add_generic_parameter_scope(Self::new(None))
    }

    /// Creates a new empty generic parameter scope.
    #[inline(always)]
    #[must_use]
    pub fn new(parent_scope: Option<GenericParameterScopeID>) -> Self {
        Self {
            parent_scope,
            parameters: FxHashMap::default(),
        }
    }
}

/// A unique ID that maps to [`GenericParameterScopeData`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GenericParameterScopeID(pub usize);

impl GenericParameterScopeID {
    /// Returns the parent scope.
    pub fn parent_scope(self, db: &Database) -> Option<GenericParameterScopeID> {
        db.generic_parameter_scope(self).parent_scope
    }

    /// Returns the map of generic parameters in the scope.
    pub fn parameters(self, db: &Database) -> &FxHashMap<IdentifierID, GenericParameterID> {
        &db.generic_parameter_scope(self).parameters
    }

    /// Adds a generic parameter into the scope.
    #[inline(always)]
    pub fn add_generic_parameter(
        self,
        db: &mut Database,
        parameter_name: IdentifierID,
        parameter: GenericParameterID,
    ) {
        db.generic_parameter_scope_mut(self)
            .parameters
            .insert(parameter_name, parameter);
    }

    /// Resolves a data about generic parameter in the scope.
    ///
    /// **Note**: the method shouldn't be used to check if the parameter exists
    /// in the scope. Use the [`contains()`] method.
    ///
    /// [`contains()`]: GenericParameterScopeID::contains
    #[inline(always)]
    #[must_use]
    pub fn resolve(
        &self,
        db: &Database,
        parameter_name: IdentifierID,
    ) -> Option<GenericParameterID> {
        if let Some(parameter_id) = self.parameters(db).get(&parameter_name) {
            Some(*parameter_id)
        } else if let Some(parent_scope_id) = &self.parent_scope(db) {
            parent_scope_id.resolve(db, parameter_name)
        } else {
            None
        }
    }

    /// Checks if the generic parameter exists in the scope.
    #[inline(always)]
    #[must_use]
    pub fn contains(&self, db: &Database, parameter_name: IdentifierID) -> bool {
        self.parameters(db).contains_key(&parameter_name)
            || if let Some(parent_scope_id) = &self.parent_scope(db) {
                parent_scope_id.contains(db, parameter_name)
            } else {
                false
            }
    }
}

/// A data, that the Stellar compiler has about a generic parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericParameterData {
    /// Location of the name of the generic parameter.
    ///
    /// ```txt
    /// foo[T: ToString = String]
    ///     ^
    /// ```
    pub location: Location,

    /// Default value of the generic parameter.
    ///
    /// ```txt
    /// foo[T: ToString = String]
    ///                   ^^^^^^
    /// ```
    pub default_value: Option<Type>,
}

impl GenericParameterData {
    /// Creates a new generic parameter data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(
        db: &mut Database,
        location: Location,
        default_value: Option<Type>,
    ) -> GenericParameterID {
        db.add_generic_parameter(Self::new(location, default_value))
    }

    /// Creates a new generic parameter data object.
    #[inline(always)]
    #[must_use]
    pub fn new(location: Location, default_value: Option<Type>) -> Self {
        Self {
            location,
            default_value,
        }
    }
}

/// A unique ID that maps to [`GenericParameterData`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GenericParameterID(pub usize);

/// A data that Stellar compiler has about an enum item.
#[derive(Debug)]
pub struct EnumItemData {
    pub name: IdentifierAST,
    pub module: ModuleID,
}

impl EnumItemData {
    /// Creates a new enum item data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(db: &mut Database, name: IdentifierAST, module: ModuleID) -> EnumItemID {
        db.add_enum_item(Self::new(name, module))
    }

    /// Creates a new enum item data object.
    #[inline(always)]
    #[must_use]
    pub fn new(name: IdentifierAST, module: ModuleID) -> Self {
        Self { name, module }
    }
}

/// A unique ID that maps to [`EnumItemData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct EnumItemID(pub usize);

impl EnumItemID {
    /// Returns the name of the enum item.
    #[inline(always)]
    #[must_use]
    pub fn name(self, db: &Database) -> IdentifierAST {
        db.enum_item(self).name
    }

    #[inline(always)]
    #[must_use]
    pub fn module(self, db: &Database) -> ModuleID {
        db.enum_item(self).module
    }
}

/// A data that Stellar compiler has about a particular type signature.
#[derive(Debug)]
pub struct SignatureData {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub module: ModuleID,
    pub generic_parameter_scope: GenericParameterScopeID,
    pub predicates: Vec<PredicateID>,
    pub implements: Vec<TypeConstructor>,
    pub is_analyzed: bool,
}

impl SignatureData {
    /// Creates a new signature data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(
        db: &mut Database,
        visibility: Visibility,
        name: IdentifierAST,
        module: ModuleID,
    ) -> SignatureID {
        let generic_parameter_scope = GenericParameterScopeData::alloc(db);

        db.add_signature(Self::new(visibility, name, generic_parameter_scope, module))
    }

    /// Creates a new signature data object.
    #[inline(always)]
    #[must_use]
    pub fn new(
        visibility: Visibility,
        name: IdentifierAST,
        generic_parameter_scope: GenericParameterScopeID,
        module: ModuleID,
    ) -> Self {
        Self {
            visibility,
            name,
            module,
            generic_parameter_scope,
            predicates: Vec::new(),
            implements: Vec::new(),
            is_analyzed: false,
        }
    }
}

/// A unique ID that maps to [`SignatureData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct SignatureID(pub usize);

impl SignatureID {
    /// Returns the name.
    #[inline(always)]
    #[must_use]
    pub fn name(self, db: &Database) -> IdentifierAST {
        db.signature(self).name
    }

    /// Returns the visibility.
    #[inline(always)]
    #[must_use]
    pub fn visibility(self, db: &Database) -> Visibility {
        db.signature(self).visibility
    }

    /// Returns the module.
    #[inline(always)]
    #[must_use]
    pub fn module(self, db: &Database) -> ModuleID {
        db.signature(self).module
    }

    #[inline(always)]
    pub fn analyzed(self, db: &mut Database) {
        db.signature_mut(self).is_analyzed = true;
    }

    #[inline(always)]
    pub fn add_predicate(self, db: &mut Database, predicate: PredicateID) {
        db.signature_mut(self).predicates.push(predicate);
    }
}

/// A data that Stellar compiler has about a function.
#[derive(Debug)]
pub struct FunctionData {
    pub signature: SignatureID,
}

impl FunctionData {
    /// Creates a new function data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(db: &mut Database, signature: SignatureID) -> FunctionID {
        db.add_function(Self::new(signature))
    }

    /// Creates a new function data object.
    #[inline(always)]
    #[must_use]
    pub fn new(signature: SignatureID) -> Self {
        Self { signature }
    }
}

/// A unique ID that maps to [`FunctionData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct FunctionID(pub usize);

impl FunctionID {
    /// Returns the function signature.
    #[inline(always)]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureID {
        db.function(self).signature
    }
}

/// A data that Stellar compiler has about an interface.
#[derive(Debug)]
pub struct InterfaceData {
    pub signature: SignatureID,
    pub methods: FxHashMap<IdentifierID, FunctionID>,
}

impl InterfaceData {
    /// Creates a new interface data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(db: &mut Database, signature: SignatureID) -> InterfaceID {
        db.add_interface(Self::new(signature))
    }

    /// Creates a new interface data object.
    #[inline(always)]
    #[must_use]
    pub fn new(signature: SignatureID) -> Self {
        Self {
            signature,
            methods: FxHashMap::default(),
        }
    }
}

/// A unique ID that maps to [`InterfaceData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct InterfaceID(pub usize);

impl InterfaceID {
    /// Returns the type signature of the interface.
    #[inline(always)]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureID {
        db.interface(self).signature
    }
}

/// A data that Stellar compiler has about a module.
#[derive(Debug)]
pub struct TypeAliasData {
    pub signature: SignatureID,
    pub ty: Type,
}

impl TypeAliasData {
    /// Creates a new type alias data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(db: &mut Database, signature: SignatureID) -> TypeAliasID {
        db.add_type_alias(Self::new(signature))
    }

    /// Creates a new type alias data object.
    #[inline(always)]
    #[must_use]
    pub fn new(signature: SignatureID) -> Self {
        Self {
            signature,
            ty: Type::Unknown,
        }
    }
}

/// A unique ID that maps to [`TypeAliasData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct TypeAliasID(pub usize);

impl TypeAliasID {
    /// Returns the signature of the type alias.
    #[inline(always)]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureID {
        db.type_alias(self).signature
    }

    #[inline(always)]
    #[must_use]
    pub fn ty(self, db: &Database) -> &Type {
        &db.type_alias(self).ty
    }

    #[inline(always)]
    #[must_use]
    pub fn ty_mut(self, db: &mut Database) -> &mut Type {
        &mut db.type_alias_mut(self).ty
    }
}

/// A data that Stellar compiler has about a module.
#[derive(Debug)]
pub struct ModuleData {
    pub name: IdentifierID,
    pub filepath: PathID,
    pub module_item_symbols: FxHashMap<IdentifierID, Symbol>,
    pub submodules: FxHashMap<IdentifierID, ModuleID>,
    pub resolved_imports: FxHashMap<IdentifierID, Symbol>,
}

impl ModuleData {
    /// Creates a new module data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(db: &mut Database, name: IdentifierID, filepath: PathID) -> ModuleID {
        db.add_module(Self::new(name, filepath))
    }

    /// Creates a new module data object.
    #[inline(always)]
    #[must_use]
    pub fn new(name: IdentifierID, filepath: PathID) -> Self {
        Self {
            name,
            filepath,
            submodules: FxHashMap::default(),
            resolved_imports: FxHashMap::default(),
            module_item_symbols: FxHashMap::default(),
        }
    }
}

/// A unique ID that maps to [`ModuleData`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
pub struct ModuleID(pub usize);

impl ModuleID {
    /// Returns module's file path ID.
    #[inline(always)]
    #[must_use]
    pub fn filepath(self, db: &Database) -> PathID {
        db.module(self).filepath
    }

    /// Returns module's name.
    #[inline(always)]
    #[must_use]
    pub fn name(self, db: &Database) -> IdentifierID {
        db.module(self).name
    }

    /// Returns an immutable reference to module item symbols.
    #[inline(always)]
    #[must_use]
    pub fn module_item_symbols(self, db: &Database) -> &FxHashMap<IdentifierID, Symbol> {
        &db.module(self).module_item_symbols
    }

    /// Returns a mutable reference to module item symbols.
    #[inline(always)]
    #[must_use]
    pub fn module_item_symbols_mut(
        self,
        db: &mut Database,
    ) -> &mut FxHashMap<IdentifierID, Symbol> {
        &mut db.module_mut(self).module_item_symbols
    }

    /// Returns an immutable reference to submodules.
    #[inline(always)]
    #[must_use]
    pub fn submodules(self, db: &Database) -> &FxHashMap<IdentifierID, ModuleID> {
        &db.module(self).submodules
    }

    /// Returns a mutable reference to submodules.
    #[inline(always)]
    #[must_use]
    pub fn submodules_mut(self, db: &mut Database) -> &mut FxHashMap<IdentifierID, ModuleID> {
        &mut db.module_mut(self).submodules
    }

    /// Resolves a symbol related to only module item in the module.
    ///
    /// If you want to additionally resolve submodules, use [`ModuleID::symbol()`].
    #[inline(always)]
    pub fn module_item_symbol_or_none(
        self,
        db: &Database,
        item_name: IdentifierID,
    ) -> Option<Symbol> {
        self.module_item_symbols(db).get(&item_name).copied()
    }

    /// Resolves a symbol in the module.
    #[inline(always)]
    pub fn symbol_or_none(self, db: &Database, name: IdentifierID) -> Option<Symbol> {
        self.module_item_symbol_or_none(db, name)
            .or(self.submodule(db, name).map(Symbol::Module))
    }

    /// Resolves a symbol in the module.
    ///
    /// # Panics
    /// Panics if the symbol cannot be resolved.
    #[inline(always)]
    #[must_use]
    pub fn symbol(self, db: &Database, name: IdentifierID) -> Symbol {
        self.symbol_or_none(db, name).unwrap()
    }

    /// Resolves a symbol in the module.
    ///
    /// # Panics
    /// Panics if the symbol cannot be resolved.
    #[inline(always)]
    #[must_use]
    pub fn module_item_symbol(self, db: &Database, name: IdentifierID) -> Symbol {
        self.module_item_symbol_or_none(db, name).unwrap()
    }

    /// Adds a module item information to the module.
    #[inline(always)]
    pub fn add_module_item(self, db: &mut Database, name: IdentifierID, symbol: Symbol) {
        self.module_item_symbols_mut(db).insert(name, symbol);
    }

    /// Checks if a symbol is contained in the module.
    #[inline(always)]
    #[must_use]
    pub fn contains_module_item_symbol(self, db: &Database, item_name: IdentifierID) -> bool {
        self.module_item_symbols(db).contains_key(&item_name)
    }

    /// Returns an ID of the submodule of the module by its name.
    #[inline(always)]
    pub fn submodule(self, db: &Database, name: IdentifierID) -> Option<ModuleID> {
        self.submodules(db).get(&name).copied()
    }

    /// Adds a submodule to the module.
    #[inline(always)]
    pub fn add_submodule(self, db: &mut Database, module: ModuleID) {
        let name = module.name(db);

        self.submodules_mut(db).insert(name, module);
    }

    /// Checks if a submodule with a given name is contained in the module.
    #[inline(always)]
    #[must_use]
    pub fn contains_submodule_with_name(self, db: &Database, name: IdentifierID) -> bool {
        self.submodules(db).contains_key(&name)
    }

    /// Checks if a submodule with a given ID is contained in the module.
    #[inline(always)]
    #[must_use]
    pub fn contains_submodule_with_id(self, db: &Database, id: ModuleID) -> bool {
        self.submodules(db)
            .values()
            .any(|&submodule| submodule == id)
    }

    /// Returns an immutable reference to imports.
    #[inline(always)]
    #[must_use]
    pub fn resolved_imports(self, db: &Database) -> &FxHashMap<IdentifierID, Symbol> {
        &db.module(self).resolved_imports
    }

    /// Returns a mutable reference to imports.
    #[inline(always)]
    #[must_use]
    pub fn resolved_imports_mut(self, db: &mut Database) -> &mut FxHashMap<IdentifierID, Symbol> {
        &mut db.module_mut(self).resolved_imports
    }

    /// Adds a resolved import to the module.
    #[inline(always)]
    pub fn add_resolved_import(self, db: &mut Database, name: IdentifierID, symbol: Symbol) {
        self.resolved_imports_mut(db).insert(name, symbol);
    }
}

/// Storage for Stellar compiler entities.
#[derive(Default, Debug)]
pub struct Database {
    packages: FxHashMap<IdentifierID, ModuleID>,
    modules: Vec<ModuleData>,
    enums: Vec<EnumData>,
    enum_items: Vec<EnumItemData>,
    predicates: Vec<PredicateData>,
    structs: Vec<StructData>,
    tuple_like_structs: Vec<TupleLikeStructData>,
    fields: Vec<FieldData>,
    functions: Vec<FunctionData>,
    interfaces: Vec<InterfaceData>,
    type_aliases: Vec<TypeAliasData>,
    generic_parameter_scopes: Vec<GenericParameterScopeData>,
    generic_parameters: Vec<GenericParameterData>,
    signatures: Vec<SignatureData>,
}

macro_rules! __db_first_access_method {
    (
        {
            name_singular: $what:ident,
            name_plural: $whats:ident,
            id_ty: $id_ty:ty,
            data_ty: $data_ty:ty
        }
    ) => {
            paste! {
                #[doc = "Returns an immutable reference to [`" $data_ty "`] by its ID ([`" $id_ty "`])."]
                #[doc = "# Panics"]
                #[doc = "Panics if an object with the given ID is not present in the database storage."]
                #[doc = ""]
                #[doc = "_This function is automatically generated using a macro!_"]
                #[inline(always)]
                #[must_use]
                pub fn $what(&self, id: $id_ty) -> &$data_ty {
                    &self.$whats[id.0]
                }
            }
    };
    (
        {
            reserved_name,
            name_singular: $what:ident,
            name_plural: $whats:ident,
            id_ty: $id_ty:ty,
            data_ty: $data_ty:ty
        }
    ) => {
            paste! {
                #[doc = "Returns an immutable reference to [`" $data_ty "`] by its ID ([`" $id_ty "`])."]
                #[doc = "# Panics"]
                #[doc = "Panics if an object with the given ID is not present in the database storage."]
                #[doc = ""]
                #[doc = "_This function is automatically generated using a macro!_"]
                #[inline(always)]
                #[must_use]
                pub fn [<$what _>](&self, id: $id_ty) -> &$data_ty {
                    &self.$whats[id.0]
                }
            }
    };
}

macro_rules! __db_rest_of_access_methods {
    (
        {
            name_singular: $what:ident,
            name_plural: $whats:ident,
            id_ty: $id_ty:ty,
            data_ty: $data_ty:ty
        }
    ) => {
        paste! {
            #[doc = "Returns a mutable reference to [`" $data_ty "`] by its ID ([`" $id_ty "`])."]
            #[doc = "# Panics"]
            #[doc = "Panics if an object with the given ID is not present in the database storage."]
            #[doc = ""]
            #[doc = "_This function is automatically generated using a macro!_"]
            #[inline(always)]
            #[must_use]
            pub fn [<$what _mut>](&mut self, id: $id_ty) -> &mut $data_ty {
                &mut self.$whats[id.0]
            }

            #[doc = "Returns an immutable reference to [`" $data_ty "`] by its ID ([`" $id_ty "`])."]
            #[doc = ""]
            #[doc = "_This function is automatically generated using a macro!_"]
            #[inline(always)]
            #[must_use]
            pub fn [<$what _or_none>](&self, id: $id_ty) -> Option<&$data_ty> {
                self.$whats.get(id.0)
            }

            #[doc = "Returns a mutable reference to [`" $data_ty "`] by its ID ([`" $id_ty "`])."]
            #[doc = ""]
            #[doc = "_This function is automatically generated using a macro!_"]
            #[inline(always)]
            #[must_use]
            pub fn [<$what _mut_or_none>](&mut self, id: $id_ty) -> Option<&mut $data_ty> {
                self.$whats.get_mut(id.0)
            }

            #[doc = "Returns whether a [`" $data_ty "`] with a given ID ([`" $id_ty "`]) is present in the database storage."]
            #[doc = ""]
            #[doc = "_This function is automatically generated using a macro!_"]
            #[inline(always)]
            #[must_use]
            pub fn [<contains_ $what>](&self, id: $id_ty) -> bool {
                id.0 < self.$whats.len()
            }

            #[doc = "Adds an object of type [`" $data_ty "`] to the database storage and returns its ID ([`" $id_ty "`])."]
            #[doc = ""]
            #[doc = "_This function is automatically generated using a macro!_"]
            #[inline(always)]
            #[must_use]
            pub fn [<add_ $what>](&mut self, [<$what _>]: $data_ty) -> $id_ty {
                self.$whats.push([<$what _>]);

                $id_ty(self.$whats.len() - 1)
            }
        }
    };
    (
        {
            reserved_name,
            name_singular: $what:ident,
            name_plural: $whats:ident,
            id_ty: $id_ty:ty,
            data_ty: $data_ty:ty
        }
    ) => {
        __db_rest_of_access_methods! {
            {
                name_singular: $what,
                name_plural: $whats,
                id_ty: $id_ty,
                data_ty: $data_ty
            }
        }
    }
}

macro_rules! __db_data_access_methods {
    ($($tt:tt),*) => {
        $(
            __db_first_access_method! { $tt }
            __db_rest_of_access_methods! { $tt }
        )*
    };
}

impl Database {
    /// Creates a new empty database.
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // Returns a package's root module ID data by package ID.
    #[inline(always)]
    pub fn package_root_module(&self, package_name: IdentifierID) -> Option<ModuleID> {
        self.packages.get(&package_name).copied()
    }

    /// Returns a package's root module ID data by package ID.
    /// # Panics
    /// Panics if the package information is not present in the database storage.
    #[inline(always)]
    #[must_use]
    pub fn package_root_module_or_panic(&self, package_name: IdentifierID) -> ModuleID {
        *self.packages.get(&package_name).unwrap()
    }

    /// Returns wether a package with a given name is present in the database storage.
    #[inline(always)]
    #[must_use]
    pub fn contains_package(&self, package_name: IdentifierID) -> bool {
        self.packages.contains_key(&package_name)
    }

    /// Adds a package to the database storage.
    #[inline(always)]
    pub fn add_package(&mut self, root_module: ModuleID) {
        let name = root_module.name(self);
        self.packages.insert(name, root_module);
    }

    __db_data_access_methods! {
        {
            name_singular: module,
            name_plural: modules,
            id_ty: ModuleID,
            data_ty: ModuleData
        },
        {
            reserved_name,
            name_singular: enum,
            name_plural: enums,
            id_ty: EnumID,
            data_ty: EnumData
        },
        {
            name_singular: enum_item,
            name_plural: enum_items,
            id_ty: EnumItemID,
            data_ty: EnumItemData
        },
        {
            name_singular: predicate,
            name_plural: predicates,
            id_ty: PredicateID,
            data_ty: PredicateData
        },
        {
            reserved_name,
            name_singular: struct,
            name_plural: structs,
            id_ty: StructID,
            data_ty: StructData
        },
        {
            name_singular: tuple_like_struct,
            name_plural: tuple_like_structs,
            id_ty: TupleLikeStructID,
            data_ty: TupleLikeStructData
        },
        {
            name_singular: field,
            name_plural: fields,
            id_ty: FieldID,
            data_ty: FieldData
        },
        {
            name_singular: function,
            name_plural: functions,
            id_ty: FunctionID,
            data_ty: FunctionData
        },
        {
            name_singular: type_alias,
            name_plural: type_aliases,
            id_ty: TypeAliasID,
            data_ty: TypeAliasData
        },
        {
            name_singular: interface,
            name_plural: interfaces,
            id_ty: InterfaceID,
            data_ty: InterfaceData
        },
        {
            name_singular: generic_parameter_scope,
            name_plural: generic_parameter_scopes,
            id_ty: GenericParameterScopeID,
            data_ty: GenericParameterScopeData
        },
        {
            name_singular: generic_parameter,
            name_plural: generic_parameters,
            id_ty: GenericParameterID,
            data_ty: GenericParameterData
        },
        {
            name_singular: signature,
            name_plural: signatures,
            id_ty: SignatureID,
            data_ty: SignatureData
        }
    }
}

/// Contains database and diagnostics.
#[derive(Default)]
pub struct State {
    db: Database,
    diagnostics: Diagnostics,
    config: Config,
}

#[derive(Default)]
pub struct Config {}

impl Config {
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl State {
    /// Creates a new empty state.
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a new state with given configuration.
    #[inline(always)]
    #[must_use]
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    /// Returns a reference to config.
    #[inline(always)]
    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    /// Returns an immutable reference to a database object.
    #[inline(always)]
    #[must_use]
    pub const fn db(&self) -> &Database {
        &self.db
    }

    /// Returns a mutable reference to a database object.
    #[inline(always)]
    #[must_use]
    pub fn db_mut(&mut self) -> &mut Database {
        &mut self.db
    }

    /// Gives an ownership over database object inside the state.
    #[inline(always)]
    #[must_use]
    pub fn into_db(self) -> Database {
        self.db
    }

    /// Returns an immutable reference to diagnostics.
    #[inline(always)]
    #[must_use]
    pub const fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// Returns a mutable reference to diagnostics.
    #[inline(always)]
    #[must_use]
    pub fn diagnostics_mut(&mut self) -> &mut Diagnostics {
        &mut self.diagnostics
    }

    /// Gives an ownership over diagnostics object inside the state.
    #[inline(always)]
    #[must_use]
    pub fn into_diagnostics(self) -> Diagnostics {
        self.diagnostics
    }
}
