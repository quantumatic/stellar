#![doc(
    html_logo_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png",
    html_favicon_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png"
)]

use std::{iter, ops::Add};

use filetime::FileTime;
use paste::paste;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use stellar_ast::{IdentifierAST, ModuleItemKind, Visibility};
use stellar_diagnostics::Diagnostics;
use stellar_filesystem::location::{Location, DUMMY_LOCATION};
use stellar_fx_hash::FxHashMap;
use stellar_interner::{IdentifierId, PathId};

#[macro_use]
mod id_type;
mod symbol;
mod ty;

pub use symbol::Symbol;
use ty::{Type, TypeConstructor};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Path {
    segments: Vec<IdentifierId>,
}

impl Path {
    #[inline]
    #[must_use]
    pub fn new(segments: Vec<IdentifierId>) -> Self {
        Self { segments }
    }

    #[inline]
    #[must_use]
    pub fn segments(&self) -> &[IdentifierId] {
        &self.segments
    }
}

impl From<IdentifierId> for Path {
    fn from(id: IdentifierId) -> Self {
        Self::new(vec![id])
    }
}

impl From<IdentifierAST> for Path {
    fn from(id: IdentifierAST) -> Self {
        Self::new(vec![id.id])
    }
}

impl Add<Path> for Path {
    type Output = Path;

    #[inline]
    fn add(self, rhs: Path) -> Self::Output {
        Self {
            segments: self.segments.into_iter().chain(rhs.segments).collect(),
        }
    }
}

impl Add<IdentifierId> for Path {
    type Output = Path;

    #[inline]
    fn add(self, rhs: IdentifierId) -> Self::Output {
        Self {
            segments: self.segments.into_iter().chain(iter::once(rhs)).collect(),
        }
    }
}

impl Add<IdentifierAST> for Path {
    type Output = Path;

    #[inline]
    fn add(self, rhs: IdentifierAST) -> Self::Output {
        self + rhs.id
    }
}

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
        db.add_enum(signature.package(), Self::new(signature))
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

impl EnumId {
    /// Returns a list of predicates associated with the enum.
    #[inline]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureId {
        db.resolve_enum(self).signature
    }

    /// Returns a list of items associated with the enum.
    #[inline]
    #[must_use]
    pub fn items(self, db: &Database) -> &FxHashMap<IdentifierId, EnumItemId> {
        &db.resolve_enum(self).items
    }

    /// Returns `true` if an item with a given name is contained in the enum definition.
    #[inline]
    #[must_use]
    pub fn contains_item(self, db: &Database, name: IdentifierId) -> bool {
        db.resolve_enum(self).items.contains_key(&name)
    }

    /// Returns an item with a given name.
    pub fn item(self, db: &Database, name: IdentifierId) -> Option<EnumItemId> {
        db.resolve_enum(self).items.get(&name).copied()
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
        db.add_struct(signature.package(), Self::new(signature))
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

impl StructId {
    /// Returns a list of predicates associated with the struct.
    #[inline]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureId {
        db.resolve_struct(self).signature
    }

    /// Returns a list of fields associated with the struct.
    #[inline]
    #[must_use]
    pub fn fields(self, db: &Database) -> &FxHashMap<IdentifierId, FieldId> {
        &db.resolve_struct(self).fields
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
        db.add_tuple_like_struct(signature.package(), Self::new(signature))
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

impl TupleLikeStructId {
    /// Returns the type signature of the struct.
    #[inline]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureId {
        db.resolve_tuple_like_struct(self).signature
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

impl PredicateId {
    /// Returns the type of the predicate.
    #[inline]
    #[must_use]
    pub fn ty(self, db: &Database) -> &Type {
        &db.resolve_predicate(self).ty
    }

    /// Returns the bounds of the predicate.
    #[inline]
    #[must_use]
    pub fn bounds(self, db: &Database) -> &[TypeConstructor] {
        &db.resolve_predicate(self).bounds
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

impl GenericParameterScopeId {
    /// Returns the parent scope.
    pub fn parent_scope(self, db: &Database) -> Option<GenericParameterScopeId> {
        db.resolve_generic_parameter_scope(self).parent_scope
    }

    /// Returns the map of generic parameters in the scope.
    pub fn parameters(self, db: &Database) -> &FxHashMap<IdentifierId, GenericParameterId> {
        &db.resolve_generic_parameter_scope(self).parameters
    }

    /// Adds a generic parameter into the scope.
    #[inline]
    pub fn add_generic_parameter(
        self,
        db: &mut Database,
        parameter_name: IdentifierId,
        parameter: GenericParameterId,
    ) {
        db.resolve_generic_parameter_scope_mut(self)
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

/// A data that Stellar compiler has about an enum item.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EnumItemData {
    pub enum_: EnumId,
    pub name: IdentifierAST,
    pub module: ModuleId,
}

impl EnumItemData {
    /// Creates a new enum item data object in the database and returns its ID.
    #[inline]
    #[must_use]
    pub fn alloc(
        db: &mut Database,
        enum_: EnumId,
        name: IdentifierAST,
        module: ModuleId,
    ) -> EnumItemId {
        db.add_enum_item(module.package(), Self::new(enum_, name, module))
    }

    /// Creates a new enum item data object.
    #[inline]
    #[must_use]
    pub fn new(enum_: EnumId, name: IdentifierAST, module: ModuleId) -> Self {
        Self {
            name,
            module,
            enum_,
        }
    }
}

impl EnumItemId {
    /// Returns the name of the enum item.
    #[inline]
    #[must_use]
    pub fn name(self, db: &Database) -> IdentifierAST {
        db.resolve_enum_item(self).name
    }

    #[inline]
    #[must_use]
    pub fn module(self, db: &Database) -> ModuleId {
        db.resolve_enum_item(self).module
    }

    #[inline]
    #[must_use]
    pub fn enum_(self, db: &Database) -> EnumId {
        db.resolve_enum_item(self).enum_
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
        let generic_parameter_scope = GenericParameterScopeData::alloc(db, module.package());

        db.add_signature(
            module.package(),
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

impl SignatureId {
    /// Returns the name.
    #[inline]
    #[must_use]
    pub fn name(self, db: &Database) -> IdentifierAST {
        db.resolve_signature(self).name
    }

    /// Returns the visibility.
    #[inline]
    #[must_use]
    pub fn visibility(self, db: &Database) -> Visibility {
        db.resolve_signature(self).visibility
    }

    /// Returns the module.
    #[inline]
    #[must_use]
    pub fn module(self, db: &Database) -> ModuleId {
        db.resolve_signature(self).module
    }

    /// Returns the corresponding HIR/THIR node index.
    #[inline]
    #[must_use]
    pub fn node_idx(self, db: &Database) -> usize {
        db.resolve_signature(self).node_idx
    }

    #[inline]
    pub fn set_analyzed(self, db: &mut Database) {
        db.resolve_signature_mut(self).is_analyzed = true;
    }

    #[inline]
    #[must_use]
    pub fn is_analyzed(self, db: &Database) -> bool {
        db.resolve_signature(self).is_analyzed
    }

    #[inline]
    #[must_use]
    pub fn predicates(self, db: &Database) -> &[PredicateId] {
        &db.resolve_signature(self).predicates
    }

    #[inline]
    pub fn add_predicate(self, db: &mut Database, predicate: PredicateId) {
        db.resolve_signature_mut(self).predicates.push(predicate);
    }

    #[inline]
    pub fn add_implemented_interface(self, db: &mut Database, interface: TypeConstructor) {
        db.resolve_signature_mut(self).implements.push(interface);
    }

    #[inline]
    #[must_use]
    pub fn generic_parameter_scope(self, db: &Database) -> GenericParameterScopeId {
        db.resolve_signature(self).generic_parameter_scope
    }

    #[inline]
    pub fn set_generic_parameter_scope(
        self,
        db: &mut Database,
        generic_parameter_scope: GenericParameterScopeId,
    ) {
        db.resolve_signature_mut(self).generic_parameter_scope = generic_parameter_scope;
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
        db.add_function(signature.package(), Self::new(signature))
    }

    /// Creates a new function data object.
    #[inline]
    #[must_use]
    pub fn new(signature: SignatureId) -> Self {
        Self { signature }
    }
}

impl FunctionId {
    /// Returns the function signature.
    #[inline]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureId {
        db.resolve_function(self).signature
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
        db.add_interface(signature.package(), Self::new(signature))
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

impl InterfaceId {
    /// Returns the type signature of the interface.
    #[inline]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureId {
        db.resolve_interface(self).signature
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
        db.add_type_alias(signature.package(), Self::new(signature))
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

impl TypeAliasId {
    /// Returns the signature of the type alias.
    #[inline]
    #[must_use]
    pub fn signature(self, db: &Database) -> SignatureId {
        db.resolve_type_alias(self).signature
    }

    #[inline]
    #[must_use]
    pub fn ty(self, db: &Database) -> &Type {
        &db.resolve_type_alias(self).ty
    }

    #[inline]
    pub fn set_type(self, db: &mut Database, ty: Type) {
        db.resolve_type_alias_mut(self).ty = ty;
    }
}

/// A data that Stellar compiler has about a module.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ModuleData {
    pub name: IdentifierId,
    pub path: Path,
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
        path: Path,
        filepath: PathId,
    ) -> ModuleId {
        db.add_module(package, Self::new(name, path, filepath))
    }

    /// Creates a new module data object.
    #[inline]
    #[must_use]
    pub fn new(name: IdentifierId, path: Path, filepath: PathId) -> Self {
        Self {
            name,
            path,
            filepath,
            submodules: FxHashMap::default(),
            resolved_imports: FxHashMap::default(),
            module_item_symbols: FxHashMap::default(),
        }
    }
}

impl ModuleId {
    /// Returns module's file path ID.
    #[inline]
    #[must_use]
    pub fn filepath(self, db: &Database) -> PathId {
        db.resolve_module(self).filepath
    }

    #[inline]
    #[must_use]
    pub fn path(self, db: &Database) -> &Path {
        &db.resolve_module(self).path
    }

    /// Returns module's name.
    #[inline]
    #[must_use]
    pub fn name(self, db: &Database) -> IdentifierId {
        db.resolve_module(self).name
    }

    /// Returns an immutable reference to module item symbols.
    #[inline]
    #[must_use]
    pub fn module_item_symbols(self, db: &Database) -> &FxHashMap<IdentifierId, Symbol> {
        &db.resolve_module(self).module_item_symbols
    }

    /// Adds a module item symbol to the module.
    pub fn add_module_item(self, db: &mut Database, name: IdentifierId, symbol: Symbol) {
        db.resolve_module_mut(self)
            .module_item_symbols
            .insert(name, symbol);
    }

    /// Returns an immutable reference to submodules.
    #[inline]
    #[must_use]
    pub fn submodules(self, db: &Database) -> &FxHashMap<IdentifierId, ModuleId> {
        &db.resolve_module(self).submodules
    }

    /// Returns a mutable reference to submodules.
    #[inline]
    #[must_use]
    pub fn submodules_mut(self, db: &mut Database) -> &mut FxHashMap<IdentifierId, ModuleId> {
        &mut db.resolve_module_mut(self).submodules
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
        &db.resolve_module(self).resolved_imports
    }

    /// Adds a resolved import to the module.
    #[inline]
    pub fn add_resolved_import(self, db: &mut Database, name: IdentifierId, symbol: Symbol) {
        db.resolve_module_mut(self)
            .resolved_imports
            .insert(name, symbol);
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
    module_: Vec<ModuleData>,
    enum_: Vec<EnumData>,
    enum_item_: Vec<EnumItemData>,
    predicate_: Vec<PredicateData>,
    struct_: Vec<StructData>,
    tuple_like_struct_: Vec<TupleLikeStructData>,
    field_: Vec<FieldData>,
    function_: Vec<FunctionData>,
    interface_: Vec<InterfaceData>,
    type_alias_: Vec<TypeAliasData>,
    generic_parameter_scope_: Vec<GenericParameterScopeData>,
    generic_parameter_: Vec<GenericParameterData>,
    signature_: Vec<SignatureData>,
}

/// Returns the last modification time of a folder with a given path.
fn last_modification_time_of(path: PathId) -> Option<FileTime> {
    path.resolve_or_none()
        .and_then(|path| path.metadata().ok())
        .map(|metadata| FileTime::from_last_modification_time(&metadata))
}

impl PackageData {
    pub fn alloc(db: &mut Database, name: IdentifierId, path: PathId) -> PackageId {
        let last_modification_time = last_modification_time_of(path);

        db.packages.push(Self {
            name,
            path,
            last_modification_time,
            root_module: DUMMY_MODULE_ID,
            parent: None,
            dependencies: FxHashMap::default(),
            module_: Vec::new(),
            enum_: Vec::new(),
            enum_item_: Vec::new(),
            predicate_: Vec::new(),
            struct_: Vec::new(),
            tuple_like_struct_: Vec::new(),
            field_: Vec::new(),
            function_: Vec::new(),
            interface_: Vec::new(),
            type_alias_: Vec::new(),
            generic_parameter_scope_: Vec::new(),
            generic_parameter_: Vec::new(),
            signature_: Vec::new(),
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

    /// Returns an immutable reference to package data by its ID.
    #[inline]
    #[must_use]
    pub fn package_or_none(&self, id: PackageId) -> Option<&PackageData> {
        self.packages.get(id.0 - 1)
    }

    /// Returns a mutable reference to package data by its ID.
    #[inline]
    #[must_use]
    pub fn package_mut(&mut self, id: PackageId) -> &mut PackageData {
        &mut self.packages[id.0 - 1]
    }

    /// Returns a mutable reference to package data by its ID.
    #[inline]
    #[must_use]
    pub fn package_mut_or_none(&mut self, id: PackageId) -> Option<&mut PackageData> {
        self.packages.get_mut(id.0 - 1)
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

// Reduces another 1331 lines of boilerplate code
id_types! {
    enum,
    enum_item,
    struct,
    tuple_like_struct,
    field,
    predicate,
    generic_parameter_scope,
    generic_parameter,
    signature,
    function,
    interface,
    type_alias,
    module
}
