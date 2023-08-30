#![allow(warnings)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png",
    html_favicon_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png"
)]

use parking_lot::RwLock;
use paste::paste;
use stellar_ast::{IdentifierAST, Path, Visibility};
use stellar_diagnostics::Diagnostics;
use stellar_filesystem::location::Location;
use stellar_fx_hash::FxHashMap;
use stellar_interner::{IdentifierID, PathID};
use stellar_thir::ty::{Type, TypeConstructor};

/// A symbol unique ID.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Symbol {
    Module(ModuleID),
    Enum(EnumID),
    Struct(StructID),
    Function(FunctionID),
    Interface(InterfaceID),
}

impl Symbol {
    /// Returns `true` if the symbol is a module.
    #[inline(always)]
    #[must_use]
    pub const fn is_module(self) -> bool {
        matches!(self, Self::Module(_))
    }

    /// Returns `true` if the symbol is an enum.
    #[inline(always)]
    #[must_use]
    pub fn is_enum(self) -> bool {
        matches!(self, Self::Enum(_))
    }

    /// Returns `true` if the symbol is a struct.
    #[inline(always)]
    #[must_use]
    pub fn is_struct(self) -> bool {
        matches!(self, Self::Struct(_))
    }

    /// Returns `true` if the symbol is a function.
    #[inline(always)]
    #[must_use]
    pub fn is_function(self) -> bool {
        matches!(self, Self::Function(_))
    }

    /// Returns `true` if the symbol is an interface.
    #[inline(always)]
    #[must_use]
    pub fn is_interface(self) -> bool {
        matches!(self, Self::Interface(_))
    }

    /// Returns the name of the symbol.
    #[inline(always)]
    #[must_use]
    pub fn name(self, db: &Database) -> Option<IdentifierAST> {
        match self {
            Self::Module(module) => unreachable!(),
            Self::Enum(id) => db.get_enum(id).map(|e| e.name),
            Self::Struct(id) => db.get_struct(id).map(|s| s.name),
            Self::Function(id) => db.get_function(id).map(|f| f.name),
            Self::Interface(id) => db.get_interface(id).map(|i| i.name),
        }
    }

    /// Returns the name's identifier ID of the symbol.
    #[inline(always)]
    #[must_use]
    pub fn name_id(self, db: &Database) -> Option<IdentifierID> {
        self.name(db).map(|name| name.id)
    }

    /// Returns the name's identifier ID location of the symbol.
    ///
    /// # Panics
    /// Panics if the symbol doesn't have a name.
    #[inline(always)]
    #[must_use]
    pub fn name_id_or_panic(self, db: &Database) -> IdentifierID {
        self.name_id(db).unwrap()
    }

    /// Returns the name's location of the symbol.
    #[inline(always)]
    #[must_use]
    pub fn name_location(self, db: &Database) -> Option<Location> {
        self.name(db).map(|name| name.location)
    }

    /// Returns the name's location of the symbol.
    ///
    /// # Panics
    /// Panics if the symbol doesn't have a name.
    #[inline(always)]
    #[must_use]
    pub fn name_location_or_panic(self, db: &Database) -> Location {
        self.name_location(db).unwrap()
    }
}

/// A data that Stellar compiler has about an enum.
pub struct EnumData {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub module: ModuleID,
    pub implements: Vec<TypeConstructor>,
    pub predicates: Vec<PredicateID>,
    pub items: FxHashMap<IdentifierID, EnumItemID>,
    pub methods: FxHashMap<IdentifierID, FunctionID>,
}

impl EnumData {
    /// Creates a new enum data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(
        db: &mut Database,
        visibility: Visibility,
        name: IdentifierAST,
        module: ModuleID,
    ) -> EnumID {
        db.add_enum(Self::new(visibility, name, module))
    }

    /// Creates a new enum data object.
    #[inline(always)]
    #[must_use]
    pub fn new(visibility: Visibility, name: IdentifierAST, module: ModuleID) -> Self {
        Self {
            visibility,
            name,
            module,
            implements: Vec::new(),
            predicates: Vec::new(),
            items: FxHashMap::default(),
            methods: FxHashMap::default(),
        }
    }
}

/// A unique ID that maps to [`EnumData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct EnumID(pub usize);

/// A data that Stellar compiler has about a struct.
pub struct StructData {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub module: ModuleID,
    pub predicates: Vec<PredicateID>,
    pub fields: FxHashMap<IdentifierID, FieldID>,
    pub methods: FxHashMap<IdentifierID, FunctionID>,
}

impl StructData {
    /// Creates a new struct data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(
        db: &mut Database,
        visibility: Visibility,
        name: IdentifierAST,
        module: ModuleID,
    ) -> StructID {
        db.add_struct(Self::new(visibility, name, module))
    }

    /// Creates a new struct data object.
    #[inline(always)]
    #[must_use]
    pub fn new(visibility: Visibility, name: IdentifierAST, module: ModuleID) -> Self {
        Self {
            visibility,
            name,
            module,
            predicates: Vec::new(),
            fields: FxHashMap::default(),
            methods: FxHashMap::default(),
        }
    }
}

/// A unique ID that maps to [`StructData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct StructID(pub usize);

/// A data that Stellar compiler has about a field.
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
pub struct FieldID(pub usize);

/// A data that Stellar compiler has about a predicate.
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
pub struct PredicateID(pub usize);

/// A data that Stellar compiler has about an enum item.
pub struct EnumItemData {
    pub name: IdentifierID,
}

/// A unique ID that maps to [`EnumItemData`].
pub struct EnumItemID(pub usize);

/// A data that Stellar compiler has about a function.
pub struct FunctionData {
    pub name: IdentifierAST,
    pub visibility: Visibility,
    pub module: ModuleID,
}

/// A unique ID that maps to [`FunctionData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct FunctionID(pub usize);

impl FunctionData {
    /// Creates a new function data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(
        db: &mut Database,
        name: IdentifierAST,
        visibility: Visibility,
        module: ModuleID,
    ) -> FunctionID {
        db.add_function(Self::new(name, visibility, module))
    }

    /// Creates a new function data object.
    #[inline(always)]
    #[must_use]
    pub fn new(name: IdentifierAST, visibility: Visibility, module: ModuleID) -> Self {
        Self {
            name,
            visibility,
            module,
        }
    }
}

/// A data that Stellar compiler has about an interface.
pub struct InterfaceData {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub module: ModuleID,
    pub predicates: Vec<PredicateID>,
    pub methods: FxHashMap<IdentifierID, FunctionID>,
}

impl InterfaceData {
    /// Creates a new interface data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(
        db: &mut Database,
        visibility: Visibility,
        name: IdentifierAST,
        module: ModuleID,
    ) -> InterfaceID {
        db.add_interface(Self::new(visibility, name, module))
    }

    /// Creates a new interface data object.
    #[inline(always)]
    #[must_use]
    pub fn new(visibility: Visibility, name: IdentifierAST, module: ModuleID) -> Self {
        Self {
            visibility,
            name,
            module,
            predicates: Vec::new(),
            methods: FxHashMap::default(),
        }
    }
}

/// A unique ID that maps to [`InterfaceData`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct InterfaceID(pub usize);

/// A data that Stellar compiler has about a module.
pub struct ModuleData {
    pub name: IdentifierID,
    pub filepath: PathID,
    pub symbols: FxHashMap<IdentifierID, Symbol>,
    pub imports: FxHashMap<IdentifierID, Path>,
    pub resolved_imports: FxHashMap<IdentifierID, Symbol>,
}

impl ModuleData {
    /// Creates a new module data object in the database and returns its ID.
    #[inline(always)]
    #[must_use]
    pub fn alloc(db: &mut Database, name: IdentifierID, filepath: PathID) -> ModuleID {
        let id = db.add_module(Self::new(name, filepath));
        db.module_mappings.insert(filepath, id);

        id
    }

    /// Creates a new module data object.
    #[inline(always)]
    #[must_use]
    pub fn new(name: IdentifierID, filepath: PathID) -> Self {
        Self {
            name,
            filepath,
            imports: FxHashMap::default(),
            resolved_imports: FxHashMap::default(),
            symbols: FxHashMap::default(),
        }
    }

    /// Resolves a symbol in the module.
    #[inline(always)]
    #[must_use]
    pub fn get_symbol(&self, id: IdentifierID) -> Option<Symbol> {
        self.symbols.get(&id).copied()
    }

    /// Resolves a symbol in the module.
    ///
    /// # Panics
    /// Panics if the symbol cannot be resolved.
    #[inline(always)]
    #[must_use]
    pub fn get_symbol_or_panic(&self, id: IdentifierID) -> Symbol {
        self.get_symbol(id).unwrap()
    }

    /// Adds a symbol to the module.
    #[inline(always)]
    pub fn add_symbol(&mut self, name: IdentifierID, symbol: Symbol) {
        self.symbols.insert(name, symbol);
    }

    /// Checks if a symbol is contained in the module.
    #[inline(always)]
    #[must_use]
    pub fn contains_symbol(&self, id: IdentifierID) -> bool {
        self.symbols.contains_key(&id)
    }
}

/// A unique ID that maps to [`ModuleData`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleID(pub usize);

/// Storage for Stellar compiler entities.
#[derive(Default)]
pub struct Database {
    module_mappings: FxHashMap<PathID, ModuleID>,
    modules: Vec<ModuleData>,
    enums: Vec<EnumData>,
    enum_items: Vec<EnumItemData>,
    predicates: Vec<PredicateData>,
    structs: Vec<StructData>,
    fields: Vec<FieldData>,
    functions: Vec<FunctionData>,
    interfaces: Vec<InterfaceData>,
}

macro_rules! db_methods {
    (
        $($what:ident: $id_ty:ty => $data_ty:ty),*
    ) => {
        $(
            paste! {
                #[doc = "Returns an immutable reference to " $what " data by its ID."]
                #[doc = ""]
                #[doc = "_This function is automatically generated using a macro!_"]
                #[inline(always)]
                #[must_use]
                pub fn [<get_ $what>](&self, id: $id_ty) -> Option<&$data_ty> {
                    self.[<$what s>].get(id.0)
                }

                #[doc = "Returns a mutable reference to " $what " data by its ID."]
                #[doc = ""]
                #[doc = "_This function is automatically generated using a macro!_"]
                #[inline(always)]
                #[must_use]
                pub fn [<get_ $what _mut>](&mut self, id: $id_ty) -> Option<&mut $data_ty> {
                    self.[<$what s>].get_mut(id.0)
                }

                #[doc = "Returns an immutable reference to " $what " data by its ID."]
                #[doc = "# Panics"]
                #[doc = "Panics if " $what " with the given ID is not present in the database storage."]
                #[doc = ""]
                #[doc = "_This function is automatically generated using a macro!_"]
                #[inline(always)]
                #[must_use]
                pub fn [<get_ $what _or_panic>](&self, id: $id_ty) -> &$data_ty {
                    self.[<$what s>].get(id.0).unwrap()
                }

                #[doc = "Returns a mutable reference to " $what " data by its ID."]
                #[doc = "# Panics"]
                #[doc = "Panics if " $what " with the given ID is not present in the database storage."]
                #[doc = ""]
                #[doc = "_This function is automatically generated using a macro!_"]
                #[inline(always)]
                #[must_use]
                pub fn [<get_ $what _mut_or_panic>](&mut self, id: $id_ty) -> &mut $data_ty {
                    self.[<$what s>].get_mut(id.0).unwrap()
                }

                #[doc = "Returns whether " $what " with a given ID is in the database storage."]
                #[doc = ""]
                #[doc = "_This function is automatically generated using a macro!_"]
                #[inline(always)]
                #[must_use]
                pub fn [<contains_ $what>](&self, id: $id_ty) -> bool {
                    id.0 < self.[<$what s>].len()
                }

                #[doc = "Adds a " $what " to the database storage."]
                #[doc = ""]
                #[doc = "_This function is automatically generated using a macro!_"]
                #[inline(always)]
                #[must_use]
                pub fn [<add_ $what>](&mut self, [<$what _>]: $data_ty) -> $id_ty {
                    self.[<$what s>].push([<$what _>]);

                    $id_ty(self.[<$what s>].len() - 1)
                }
            }
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

    // reduces the size of code in hundreds of times!
    db_methods! {
        module: ModuleID => ModuleData,
        enum: EnumID => EnumData,
        struct: StructID => StructData,
        function: FunctionID => FunctionData,
        interface: InterfaceID => InterfaceData,
        predicate: PredicateID => PredicateData,
        enum_item: EnumItemID => EnumItemData,
        field: FieldID => FieldData
    }
}

/// Contains database and diagnostics.
#[derive(Default)]
pub struct State {
    db: RwLock<Database>,
    diagnostics: RwLock<Diagnostics>,
}

impl State {
    /// Creates a new empty state.
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to database.
    #[inline(always)]
    #[must_use]
    pub const fn db(&self) -> &RwLock<Database> {
        &self.db
    }

    /// Returns a reference to database.
    #[inline(always)]
    #[must_use]
    pub const fn diagnostics(&self) -> &RwLock<Diagnostics> {
        &self.diagnostics
    }
}
