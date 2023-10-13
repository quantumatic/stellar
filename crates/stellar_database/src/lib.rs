#![doc(
    html_logo_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png",
    html_favicon_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png"
)]

use filetime::FileTime;
use paste::paste;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use stellar_ast::{IdentifierAST, ModuleItemKind, Visibility};
use stellar_diagnostics::Diagnostics;
use stellar_filesystem::location::{Location, DUMMY_LOCATION};
use stellar_fx_hash::FxHashMap;
use stellar_interner::{IdentifierId, PathId};
use stellar_thir::ty::{Type, TypeConstructor};

#[macro_use]
mod access_methods_macro;
mod symbol;

pub use symbol::Symbol;

/// A data that Stellar compiler has about an enum.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EnumData {
    pub signature: SignatureId,
    pub items: FxHashMap<IdentifierId, EnumItemId>,
    pub methods: FxHashMap<IdentifierId, FunctionId>,
}

impl EnumData {
    /// Creates a new enum data object in the database and returns its ID.
    #[inline]
    #[must_use]
    pub fn alloc(db: &mut Database, signature: SignatureId) -> EnumId {
        db.add_enum(signature.module(db).0, Self::new(signature))
    }

    /// Creates a new enum data object.
    #[inline]
    #[must_use]
    pub fn new(signature: SignatureId) -> Self {
        Self {
            signature,
            items: FxHashMap::default(),
            methods: FxHashMap::default(),
        }
    }
}

/// A unique ID that maps to [`EnumData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EnumId(pub PackageId, pub usize);

impl EnumId {
    /// Returns a list of predicates associated with the enum.
    #[inline]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureId {
        db.enum_(self).signature
    }

    /// Returns a list of items associated with the enum.
    #[inline]
    #[must_use]
    pub fn items(self, db: &Database) -> &FxHashMap<IdentifierId, EnumItemId> {
        &db.enum_(self).items
    }

    /// Returns `true` if an item with a given name is contained in the enum definition.
    #[inline]
    #[must_use]
    pub fn contains_item(self, db: &Database, name: IdentifierId) -> bool {
        db.enum_(self).items.contains_key(&name)
    }

    /// Returns an item with a given name.
    pub fn item(self, db: &Database, name: IdentifierId) -> Option<EnumItemId> {
        db.enum_(self).items.get(&name).copied()
    }
}

/// A data that Stellar compiler has about a struct.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StructData {
    pub signature: SignatureId,
    pub fields: FxHashMap<IdentifierId, FieldId>,
    pub methods: FxHashMap<IdentifierId, FunctionId>,
}

impl StructData {
    /// Creates a new struct data object in the database and returns its ID.
    #[inline]
    #[must_use]
    pub fn alloc(db: &mut Database, signature: SignatureId) -> StructId {
        db.add_struct(signature.module(db).0, Self::new(signature))
    }

    /// Creates a new struct data object.
    #[inline]
    #[must_use]
    pub fn new(signature: SignatureId) -> Self {
        Self {
            signature,
            fields: FxHashMap::default(),
            methods: FxHashMap::default(),
        }
    }
}

/// A unique ID that maps to [`StructData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StructId(pub PackageId, pub usize);

impl StructId {
    /// Returns a list of predicates associated with the struct.
    #[inline]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureId {
        db.struct_(self).signature
    }

    /// Returns a list of fields associated with the struct.
    #[inline]
    #[must_use]
    pub fn fields(self, db: &Database) -> &FxHashMap<IdentifierId, FieldId> {
        &db.struct_(self).fields
    }
}

/// A data that Stellar compiler has about a function.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TupleLikeStructData {
    pub signature: SignatureId,
    pub fields: Vec<(Visibility, Type)>,
}

impl TupleLikeStructData {
    /// Creates a new tuple-like struct data object in the database and returns its ID.
    #[inline]
    #[must_use]
    pub fn alloc(db: &mut Database, signature: SignatureId) -> TupleLikeStructId {
        db.add_tuple_like_struct(signature.module(db).0, Self::new(signature))
    }

    /// Creates a new tuple-like struct data object.
    #[inline]
    #[must_use]
    pub fn new(signature: SignatureId) -> Self {
        Self {
            signature,
            fields: Vec::new(),
        }
    }
}

/// A unique ID that maps to [`TupleLikeStructData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TupleLikeStructId(pub PackageId, pub usize);

impl TupleLikeStructId {
    /// Returns the type signature of the struct.
    #[inline]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureId {
        db.tuple_like_struct(self).signature
    }
}

/// A data that Stellar compiler has about a field.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FieldData {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub ty: Type,
}

impl FieldData {
    /// Creates a new field data object in the database and returns its ID.
    #[inline]
    #[must_use]
    pub fn alloc(
        db: &mut Database,
        package: PackageId,
        visibility: Visibility,
        name: IdentifierAST,
        ty: Type,
    ) -> FieldId {
        db.add_field(package, Self::new(visibility, name, ty))
    }

    /// Creates a new field data object.
    #[inline]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FieldId(pub PackageId, pub usize);

/// A data that Stellar compiler has about a predicate.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PredicateData {
    pub ty: Type,
    pub bounds: Vec<TypeConstructor>,
}

impl PredicateData {
    /// Creates a new predicate data object in the database and returns its ID.
    #[inline]
    #[must_use]
    pub fn alloc(
        db: &mut Database,
        package: PackageId,
        ty: Type,
        bounds: Vec<TypeConstructor>,
    ) -> PredicateId {
        db.add_predicate(package, Self::new(ty, bounds))
    }

    /// Creates a new predicate data object.
    #[inline]
    #[must_use]
    pub fn new(ty: Type, bounds: Vec<TypeConstructor>) -> Self {
        Self { ty, bounds }
    }
}

/// A unique ID that maps to [`PredicateData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PredicateId(pub PackageId, pub usize);

impl PredicateId {
    /// Returns the type of the predicate.
    #[inline]
    #[must_use]
    pub fn ty(self, db: &Database) -> &Type {
        &db.predicate(self).ty
    }

    /// Returns the bounds of the predicate.
    #[inline]
    #[must_use]
    pub fn bounds(self, db: &Database) -> &[TypeConstructor] {
        &db.predicate(self).bounds
    }
}

/// A data that Stellar compiler has about a generic parameter scope.
#[derive(Default, PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GenericParameterScopeData {
    /// A parent scope, for example:
    ///
    /// ```stellar
    /// interface Foo[T] { // self.parent = Scope { parent: None, parameters: [T] }
    ///     fun bar[M]();  // self = Scope { parent: ..., parameters: [M] }
    /// }
    /// ```
    pub parent_scope: Option<GenericParameterScopeId>,

    /// A map of generic parameters in the scope.
    pub parameters: FxHashMap<IdentifierId, GenericParameterId>,
}

impl GenericParameterScopeData {
    /// Creates a new generic parameter scope data object in the database and returns its ID.
    #[inline]
    #[must_use]
    pub fn alloc(db: &mut Database, package: PackageId) -> GenericParameterScopeId {
        db.add_generic_parameter_scope(package, Self::new(None))
    }

    /// Creates a new empty generic parameter scope.
    #[inline]
    #[must_use]
    pub fn new(parent_scope: Option<GenericParameterScopeId>) -> Self {
        Self {
            parent_scope,
            parameters: FxHashMap::default(),
        }
    }
}

/// A unique ID that maps to [`GenericParameterScopeData`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GenericParameterScopeId(pub PackageId, pub usize);

impl GenericParameterScopeId {
    /// Returns the parent scope.
    pub fn parent_scope(self, db: &Database) -> Option<GenericParameterScopeId> {
        db.generic_parameter_scope(self).parent_scope
    }

    /// Returns the map of generic parameters in the scope.
    pub fn parameters(self, db: &Database) -> &FxHashMap<IdentifierId, GenericParameterId> {
        &db.generic_parameter_scope(self).parameters
    }

    /// Adds a generic parameter into the scope.
    #[inline]
    pub fn add_generic_parameter(
        self,
        db: &mut Database,
        parameter_name: IdentifierId,
        parameter: GenericParameterId,
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
    /// [`contains()`]: GenericParameterScopeId::contains
    #[inline]
    #[must_use]
    pub fn resolve(
        &self,
        db: &Database,
        parameter_name: IdentifierId,
    ) -> Option<GenericParameterId> {
        if let Some(parameter_id) = self.parameters(db).get(&parameter_name) {
            Some(*parameter_id)
        } else if let Some(parent_scope_id) = &self.parent_scope(db) {
            parent_scope_id.resolve(db, parameter_name)
        } else {
            None
        }
    }

    /// Checks if the generic parameter exists in the scope.
    #[inline]
    #[must_use]
    pub fn contains(&self, db: &Database, parameter_name: IdentifierId) -> bool {
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    #[inline]
    #[must_use]
    pub fn alloc(
        db: &mut Database,
        package: PackageId,
        location: Location,
        default_value: Option<Type>,
    ) -> GenericParameterId {
        db.add_generic_parameter(package, Self::new(location, default_value))
    }

    /// Creates a new generic parameter data object.
    #[inline]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GenericParameterId(pub PackageId, pub usize);

/// A data that Stellar compiler has about an enum item.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EnumItemData {
    pub name: IdentifierAST,
    pub module: ModuleId,
}

impl EnumItemData {
    /// Creates a new enum item data object in the database and returns its ID.
    #[inline]
    #[must_use]
    pub fn alloc(db: &mut Database, name: IdentifierAST, module: ModuleId) -> EnumItemId {
        db.add_enum_item(module.0, Self::new(name, module))
    }

    /// Creates a new enum item data object.
    #[inline]
    #[must_use]
    pub fn new(name: IdentifierAST, module: ModuleId) -> Self {
        Self { name, module }
    }
}

/// A unique ID that maps to [`EnumItemData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EnumItemId(pub PackageId, pub usize);

impl EnumItemId {
    /// Returns the name of the enum item.
    #[inline]
    #[must_use]
    pub fn name(self, db: &Database) -> IdentifierAST {
        db.enum_item(self).name
    }

    #[inline]
    #[must_use]
    pub fn module(self, db: &Database) -> ModuleId {
        db.enum_item(self).module
    }
}

/// A data that Stellar compiler has about a particular type signature.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SignatureData {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub node_idx: usize,
    pub module: ModuleId,
    pub generic_parameter_scope: GenericParameterScopeId,
    pub predicates: Vec<PredicateId>,
    pub implements: Vec<TypeConstructor>,
    pub is_analyzed: bool,
}

impl SignatureData {
    /// Creates a new signature data object in the database and returns its ID.
    #[inline]
    #[must_use]
    pub fn alloc(
        db: &mut Database,
        visibility: Visibility,
        name: IdentifierAST,
        node_idx: usize,
        module: ModuleId,
    ) -> SignatureId {
        let package = module.0;

        let generic_parameter_scope = GenericParameterScopeData::alloc(db, package);

        db.add_signature(
            package,
            Self::new(visibility, name, node_idx, generic_parameter_scope, module),
        )
    }

    /// Creates a new signature data object.
    #[inline]
    #[must_use]
    pub fn new(
        visibility: Visibility,
        name: IdentifierAST,
        node_idx: usize,
        generic_parameter_scope: GenericParameterScopeId,
        module: ModuleId,
    ) -> Self {
        Self {
            visibility,
            name,
            node_idx,
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SignatureId(pub PackageId, pub usize);

impl SignatureId {
    /// Returns the name.
    #[inline]
    #[must_use]
    pub fn name(self, db: &Database) -> IdentifierAST {
        db.signature(self).name
    }

    /// Returns the visibility.
    #[inline]
    #[must_use]
    pub fn visibility(self, db: &Database) -> Visibility {
        db.signature(self).visibility
    }

    /// Returns the module.
    #[inline]
    #[must_use]
    pub fn module(self, db: &Database) -> ModuleId {
        db.signature(self).module
    }

    /// Returns the corresponding HIR/THIR node index.
    #[inline]
    #[must_use]
    pub fn node_idx(self, db: &Database) -> usize {
        db.signature(self).node_idx
    }

    #[inline]
    pub fn set_analyzed(self, db: &mut Database) {
        db.signature_mut(self).is_analyzed = true;
    }

    #[inline]
    #[must_use]
    pub fn is_analyzed(self, db: &Database) -> bool {
        db.signature(self).is_analyzed
    }

    #[inline]
    #[must_use]
    pub fn predicates(self, db: &Database) -> &[PredicateId] {
        &db.signature(self).predicates
    }

    #[inline]
    pub fn add_predicate(self, db: &mut Database, predicate: PredicateId) {
        db.signature_mut(self).predicates.push(predicate);
    }

    #[inline]
    pub fn add_implemented_interface(self, db: &mut Database, interface: TypeConstructor) {
        db.signature_mut(self).implements.push(interface);
    }

    #[inline]
    #[must_use]
    pub fn generic_parameter_scope(self, db: &Database) -> GenericParameterScopeId {
        db.signature(self).generic_parameter_scope
    }

    #[inline]
    pub fn set_generic_parameter_scope(
        self,
        db: &mut Database,
        generic_parameter_scope: GenericParameterScopeId,
    ) {
        db.signature_mut(self).generic_parameter_scope = generic_parameter_scope;
    }
}

/// A data that Stellar compiler has about a function.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FunctionData {
    pub signature: SignatureId,
}

impl FunctionData {
    /// Creates a new function data object in the database and returns its ID.
    #[inline]
    #[must_use]
    pub fn alloc(db: &mut Database, signature: SignatureId) -> FunctionId {
        db.add_function(signature.module(db).0, Self::new(signature))
    }

    /// Creates a new function data object.
    #[inline]
    #[must_use]
    pub fn new(signature: SignatureId) -> Self {
        Self { signature }
    }
}

/// A unique ID that maps to [`FunctionData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FunctionId(pub PackageId, pub usize);

impl FunctionId {
    /// Returns the function signature.
    #[inline]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureId {
        db.function(self).signature
    }
}

/// A data that Stellar compiler has about an interface.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InterfaceData {
    pub signature: SignatureId,
    pub methods: FxHashMap<IdentifierId, FunctionId>,
}

impl InterfaceData {
    /// Creates a new interface data object in the database and returns its ID.
    #[inline]
    #[must_use]
    pub fn alloc(db: &mut Database, signature: SignatureId) -> InterfaceId {
        db.add_interface(signature.module(db).0, Self::new(signature))
    }

    /// Creates a new interface data object.
    #[inline]
    #[must_use]
    pub fn new(signature: SignatureId) -> Self {
        Self {
            signature,
            methods: FxHashMap::default(),
        }
    }
}

/// A unique ID that maps to [`InterfaceData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InterfaceId(pub PackageId, pub usize);

impl InterfaceId {
    /// Returns the type signature of the interface.
    #[inline]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureId {
        db.interface(self).signature
    }
}

/// A data that Stellar compiler has about a module.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TypeAliasData {
    pub signature: SignatureId,
    pub ty: Type,
}

impl TypeAliasData {
    /// Creates a new type alias data object in the database and returns its ID.
    #[inline]
    #[must_use]
    pub fn alloc(db: &mut Database, signature: SignatureId) -> TypeAliasId {
        db.add_type_alias(signature.module(db).0, Self::new(signature))
    }

    /// Creates a new type alias data object.
    #[inline]
    #[must_use]
    pub fn new(signature: SignatureId) -> Self {
        Self {
            signature,
            ty: Type::Unknown,
        }
    }
}

/// A unique ID that maps to [`TypeAliasData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TypeAliasId(pub PackageId, pub usize);

impl TypeAliasId {
    /// Returns the signature of the type alias.
    #[inline]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureId {
        db.type_alias(self).signature
    }

    #[inline]
    #[must_use]
    pub fn ty(self, db: &Database) -> &Type {
        &db.type_alias(self).ty
    }

    #[inline]
    #[must_use]
    pub fn ty_mut(self, db: &mut Database) -> &mut Type {
        &mut db.type_alias_mut(self).ty
    }
}

/// A data that Stellar compiler has about a module.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ModuleData {
    pub name: IdentifierId,
    pub filepath: PathId,
    pub module_item_symbols: FxHashMap<IdentifierId, Symbol>,
    pub submodules: FxHashMap<IdentifierId, ModuleId>,
    pub resolved_imports: FxHashMap<IdentifierId, Symbol>,
}

impl ModuleData {
    /// Creates a new module data object in the database and returns its ID.
    #[inline]
    #[must_use]
    pub fn alloc(
        db: &mut Database,
        package: PackageId,
        name: IdentifierId,
        filepath: PathId,
    ) -> ModuleId {
        db.add_module(package, Self::new(name, filepath))
    }

    /// Creates a new module data object.
    #[inline]
    #[must_use]
    pub fn new(name: IdentifierId, filepath: PathId) -> Self {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ModuleId(pub PackageId, pub usize);

pub const DUMMY_MODULE_ID: ModuleId = ModuleId(DUMMY_PACKAGE_ID, 0);

impl ModuleId {
    /// Returns module's file path ID.
    #[inline]
    #[must_use]
    pub fn filepath(self, db: &Database) -> PathId {
        db.module(self).filepath
    }

    /// Returns module's name.
    #[inline]
    #[must_use]
    pub fn name(self, db: &Database) -> IdentifierId {
        db.module(self).name
    }

    /// Returns an immutable reference to module item symbols.
    #[inline]
    #[must_use]
    pub fn module_item_symbols(self, db: &Database) -> &FxHashMap<IdentifierId, Symbol> {
        &db.module(self).module_item_symbols
    }

    /// Returns a mutable reference to module item symbols.
    #[inline]
    #[must_use]
    pub fn module_item_symbols_mut(
        self,
        db: &mut Database,
    ) -> &mut FxHashMap<IdentifierId, Symbol> {
        &mut db.module_mut(self).module_item_symbols
    }

    /// Returns an immutable reference to submodules.
    #[inline]
    #[must_use]
    pub fn submodules(self, db: &Database) -> &FxHashMap<IdentifierId, ModuleId> {
        &db.module(self).submodules
    }

    /// Returns a mutable reference to submodules.
    #[inline]
    #[must_use]
    pub fn submodules_mut(self, db: &mut Database) -> &mut FxHashMap<IdentifierId, ModuleId> {
        &mut db.module_mut(self).submodules
    }

    /// Resolves a symbol related to only module item in the module.
    ///
    /// If you want to additionally resolve submodules, use [`ModuleId::symbol()`].
    #[inline]
    pub fn module_item_symbol_or_none(
        self,
        db: &Database,
        item_name: IdentifierId,
    ) -> Option<Symbol> {
        self.module_item_symbols(db).get(&item_name).copied()
    }

    /// Resolves a symbol in the module.
    #[inline]
    pub fn symbol_or_none(self, db: &Database, name: IdentifierId) -> Option<Symbol> {
        self.module_item_symbol_or_none(db, name)
            .or(self.submodule(db, name).map(Symbol::Module))
    }

    /// Resolves a symbol in the module.
    ///
    /// # Panics
    /// Panics if the symbol cannot be resolved.
    #[inline]
    #[must_use]
    pub fn symbol(self, db: &Database, name: IdentifierId) -> Symbol {
        self.symbol_or_none(db, name).unwrap()
    }

    /// Resolves a symbol in the module.
    ///
    /// # Panics
    /// Panics if the symbol cannot be resolved.
    #[inline]
    #[must_use]
    pub fn module_item_symbol(self, db: &Database, name: IdentifierId) -> Symbol {
        self.module_item_symbol_or_none(db, name).unwrap()
    }

    /// Adds a module item information to the module.
    #[inline]
    pub fn add_module_item(self, db: &mut Database, name: IdentifierId, symbol: Symbol) {
        self.module_item_symbols_mut(db).insert(name, symbol);
    }

    /// Checks if a symbol is contained in the module.
    #[inline]
    #[must_use]
    pub fn contains_module_item_symbol(self, db: &Database, item_name: IdentifierId) -> bool {
        self.module_item_symbols(db).contains_key(&item_name)
    }

    /// Returns the ID of the submodule of the module by its name.
    #[inline]
    pub fn submodule(self, db: &Database, name: IdentifierId) -> Option<ModuleId> {
        self.submodules(db).get(&name).copied()
    }

    /// Adds a submodule to the module.
    #[inline]
    pub fn add_submodule(self, db: &mut Database, module: ModuleId) {
        let name = module.name(db);

        self.submodules_mut(db).insert(name, module);
    }

    /// Checks if a submodule with a given name is contained in the module.
    #[inline]
    #[must_use]
    pub fn contains_submodule_with_name(self, db: &Database, name: IdentifierId) -> bool {
        self.submodules(db).contains_key(&name)
    }

    /// Checks if a submodule with a given ID is contained in the module.
    #[inline]
    #[must_use]
    pub fn contains_submodule_with_id(self, db: &Database, id: ModuleId) -> bool {
        self.submodules(db)
            .values()
            .any(|&submodule| submodule == id)
    }

    /// Returns an immutable reference to imports.
    #[inline]
    #[must_use]
    pub fn resolved_imports(self, db: &Database) -> &FxHashMap<IdentifierId, Symbol> {
        &db.module(self).resolved_imports
    }

    /// Returns a mutable reference to imports.
    #[inline]
    #[must_use]
    pub fn resolved_imports_mut(self, db: &mut Database) -> &mut FxHashMap<IdentifierId, Symbol> {
        &mut db.module_mut(self).resolved_imports
    }

    /// Adds a resolved import to the module.
    #[inline]
    pub fn add_resolved_import(self, db: &mut Database, name: IdentifierId, symbol: Symbol) {
        self.resolved_imports_mut(db).insert(name, symbol);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PackageId(pub usize);

pub const DUMMY_PACKAGE_ID: PackageId = PackageId(0);

impl PackageId {
    #[inline]
    #[must_use]
    pub fn name(self, db: &Database) -> IdentifierId {
        db.packages[self.0 - 1].name
    }

    #[inline]
    #[must_use]
    pub fn parent(self, db: &Database) -> Option<PackageId> {
        db.packages[self.0 - 1].parent
    }

    #[inline]
    #[must_use]
    pub fn parent_or_none(self, db: &Database) -> Option<PackageId> {
        db.packages
            .get(self.0 - 1)
            .and_then(|package| package.parent)
    }

    #[inline]
    #[must_use]
    pub fn dependencies(self, db: &Database) -> &FxHashMap<IdentifierId, PackageId> {
        &db.packages[self.0 - 1].dependencies
    }

    #[inline]
    #[must_use]
    pub fn dependencies_or_none(
        self,
        db: &Database,
    ) -> Option<&FxHashMap<IdentifierId, PackageId>> {
        db.packages
            .get(self.0 - 1)
            .map(|package| &package.dependencies)
    }

    #[inline]
    #[must_use]
    pub fn root_module(self, db: &Database) -> ModuleId {
        db.packages[self.0 - 1].root_module
    }

    #[inline]
    #[must_use]
    pub fn root_module_or_none(self, db: &Database) -> Option<ModuleId> {
        db.packages
            .get(self.0 - 1)
            .map(|package| package.root_module)
    }

    pub fn set_root_module(self, db: &mut Database, module: ModuleId) {
        db.packages[self.0 - 1].root_module = module;
    }
}

/// The information Stellar compiler has about a particular package.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PackageData {
    #[allow(dead_code)]
    name: IdentifierId,

    #[allow(dead_code)]
    path: PathId,

    /// The ID of the root module of the package.
    ///
    /// ```txt
    /// test 1.0.0
    /// |_ src
    ///    |_ main.sr (not a root module)
    ///    |_ package.sr (the root module)
    /// ```
    root_module: ModuleId,

    /// If the package is local, this is the package ID of
    /// the package that uses it as a dependency and is
    /// in the Stelar repository index or it is the one currently processed.
    ///
    /// ```txt
    /// entry_point 1.0.0
    /// |_ A 0.0.0 (in the same filesystem as `entry_point`)
    /// |_ B 1.0.0 (in Stellar repository index)
    ///    |_ C 0.0.0 (in the same filesystem as `B`)
    /// ```
    #[allow(dead_code)]
    parent: Option<PackageId>,

    /// List of packages that the package depends on.
    #[allow(dead_code)]
    dependencies: FxHashMap<IdentifierId, PackageId>,

    /// The time of the last modification of the package folder.
    #[allow(dead_code)]
    last_modification_time: Option<FileTime>,

    // Information about all package-related compiler entities.
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

/// Returns the last modification time of a folder with a given path.
fn last_modification_time_of(path: PathId) -> Option<FileTime> {
    path.resolve_or_none()
        .and_then(|path| path.metadata().ok())
        .map(|metadata| FileTime::from_last_modification_time(&metadata))
}

impl PackageData {
    pub fn alloc(db: &mut Database, name: IdentifierId, path: PathId) -> PackageId {
        dbg!(path);
        let last_modification_time = last_modification_time_of(path);
        dbg!("ok");
        db.packages.push(Self {
            name,
            path,
            last_modification_time,
            root_module: DUMMY_MODULE_ID,
            parent: None,
            dependencies: FxHashMap::default(),
            modules: Vec::new(),
            enums: Vec::new(),
            enum_items: Vec::new(),
            predicates: Vec::new(),
            structs: Vec::new(),
            tuple_like_structs: Vec::new(),
            fields: Vec::new(),
            functions: Vec::new(),
            interfaces: Vec::new(),
            type_aliases: Vec::new(),
            generic_parameter_scopes: Vec::new(),
            generic_parameters: Vec::new(),
            signatures: Vec::new(),
        });

        PackageId(db.packages.len())
    }

    #[inline]
    #[must_use]
    #[cfg(feature = "bincode")]
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    #[inline]
    #[must_use]
    #[cfg(feature = "bincode")]
    pub fn deserialize(bytes: Vec<u8>) -> Self {
        bincode::deserialize(&bytes).unwrap()
    }
}

#[cfg(feature = "bincode")]
impl From<PackageData> for Vec<u8> {
    #[inline]
    fn from(value: PackageData) -> Self {
        value.serialize()
    }
}

#[cfg(feature = "bincode")]
impl From<Vec<u8>> for PackageData {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        Self::deserialize(value)
    }
}

/// Storage for Stellar compiler entities.
#[derive(Default, Debug)]
pub struct Database {
    packages: Vec<PackageData>,
}

impl Database {
    /// Creates a new empty database.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns an immutable reference to package data by its ID.
    #[inline]
    #[must_use]
    pub fn package(&self, id: PackageId) -> &PackageData {
        &self.packages[id.0 - 1]
    }

    /// Returns a mutable reference to package data by its ID.
    #[inline]
    #[must_use]
    pub fn package_mut(&mut self, id: PackageId) -> &mut PackageData {
        &mut self.packages[id.0 - 1]
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
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl State {
    /// Creates a new empty state.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a new state with given configuration.
    #[inline]
    #[must_use]
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    /// Returns a reference to config.
    #[inline]
    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    /// Returns an immutable reference to a database object.
    #[inline]
    #[must_use]
    pub const fn db(&self) -> &Database {
        &self.db
    }

    /// Returns a mutable reference to a database object.
    #[inline]
    #[must_use]
    pub fn db_mut(&mut self) -> &mut Database {
        &mut self.db
    }

    /// Gives an ownership over database object inside the state.
    #[inline]
    #[must_use]
    pub fn into_db(self) -> Database {
        self.db
    }

    /// Returns an immutable reference to diagnostics.
    #[inline]
    #[must_use]
    pub const fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// Returns a mutable reference to diagnostics.
    #[inline]
    #[must_use]
    pub fn diagnostics_mut(&mut self) -> &mut Diagnostics {
        &mut self.diagnostics
    }

    /// Gives an ownership over diagnostics object inside the state.
    #[inline]
    #[must_use]
    pub fn into_diagnostics(self) -> Diagnostics {
        self.diagnostics
    }
}

// # What?
// This macro automatically generates methods like `module`, `module_mut`,
// `add_module`, `contains_module`, etc. It is here because it saves up to
// 1k lines of unrequired boilerplate code.
//
// # Macro input format
// * `method_name` is the name of the method that returns entity data by ID.
// * `field_name` is the name of the field in [`PackageData`] type that stores
// the entity data.
// * `entity_id_ty` is the type of the entity ID.
// * `entity_data_ty` is the type of the entity data.
// * `reserved_name` stands for situations in which `method_name` is a keyword,
// so additional `_` will be added.
//
// TODO: refactor this to be more readable and easy to edit in the near future.
define_db_access_methods! {
    { method_name: module, field_name: modules,
      entity_id_ty: ModuleId, entity_data_ty: ModuleData },
    { reserved_name, method_name: enum, field_name: enums,
      entity_id_ty: EnumId, entity_data_ty: EnumData },
    { method_name: enum_item, field_name: enum_items,
      entity_id_ty: EnumItemId, entity_data_ty: EnumItemData },
    { method_name: predicate, field_name: predicates,
      entity_id_ty: PredicateId, entity_data_ty: PredicateData },
    { reserved_name, method_name: struct, field_name: structs,
      entity_id_ty: StructId, entity_data_ty: StructData },
    { method_name: tuple_like_struct, field_name: tuple_like_structs,
      entity_id_ty: TupleLikeStructId, entity_data_ty: TupleLikeStructData },
    { method_name: field, field_name: fields,
      entity_id_ty: FieldId, entity_data_ty: FieldData },
    { method_name: function, field_name: functions,
      entity_id_ty: FunctionId, entity_data_ty: FunctionData },
    { method_name: type_alias, field_name: type_aliases,
      entity_id_ty: TypeAliasId, entity_data_ty: TypeAliasData },
    { method_name: interface, field_name: interfaces,
      entity_id_ty: InterfaceId, entity_data_ty: InterfaceData },
    { method_name: generic_parameter_scope, field_name: generic_parameter_scopes,
      entity_id_ty: GenericParameterScopeId, entity_data_ty: GenericParameterScopeData },
    { method_name: generic_parameter, field_name: generic_parameters,
      entity_id_ty: GenericParameterId, entity_data_ty: GenericParameterData },
    { method_name: signature, field_name: signatures,
      entity_id_ty: SignatureId, entity_data_ty: SignatureData }
}
