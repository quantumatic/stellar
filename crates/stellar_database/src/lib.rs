#![allow(warnings)]

use parking_lot::RwLock;
use stellar_ast::{IdentifierAST, Visibility};
use stellar_diagnostics::Diagnostics;
use stellar_fx_hash::FxHashMap;
use stellar_interner::{IdentifierID, PathID};
use stellar_thir::ty::{Type, TypeConstructor};

pub enum Symbol {
    Module(ModuleID),
    Enum(EnumID),
    Struct(StructID),
    Function(FunctionID),
    Interface(InterfaceID),
}

pub struct EnumData {
    docstring: Option<String>,
    visibility: Visibility,
    name: IdentifierID,
    module: ModuleID,
    implements: Vec<TypeConstructor>,
    predicates: Vec<PredicateID>,
    items: FxHashMap<IdentifierID, EnumItemID>,
    methods: FxHashMap<IdentifierID, FunctionID>,
}

impl EnumData {
    pub fn alloc(
        db: &mut Database,
        docstring: Option<String>,
        visibility: Visibility,
        name: IdentifierID,
        module: ModuleID,
    ) -> EnumID {
        db.enums
            .push(EnumData::new(docstring, visibility, name, module));

        EnumID(db.enums.len() - 1)
    }

    pub fn new(
        docstring: Option<String>,
        visibility: Visibility,
        name: IdentifierID,
        module: ModuleID,
    ) -> Self {
        Self {
            docstring,
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

pub struct EnumID(pub usize);

pub struct StructData {
    docstring: Option<String>,
    visibility: Visibility,
    name: IdentifierID,
    module: ModuleID,
    predicates: Vec<PredicateID>,
    fields: FxHashMap<IdentifierID, FieldID>,
    methods: FxHashMap<IdentifierID, FunctionID>,
}

pub struct StructID(pub usize);

pub struct FieldData {
    docstring: Option<String>,
    visibility: Visibility,
    name: IdentifierAST,
    ty: Type,
}

impl FieldData {
    pub fn alloc(
        db: &mut Database,
        docstring: Option<String>,
        visibility: Visibility,
        name: IdentifierAST,
        ty: Type,
    ) -> FieldID {
        db.fields
            .push(FieldData::new(docstring, visibility, name, ty));

        FieldID(db.fields.len() - 1)
    }

    pub fn new(
        docstring: Option<String>,
        visibility: Visibility,
        name: IdentifierAST,
        ty: Type,
    ) -> Self {
        Self {
            docstring,
            visibility,
            name,
            ty,
        }
    }
}

pub struct FieldID(pub usize);

pub struct PredicateData {
    ty: Type,
    bounds: Vec<TypeConstructor>,
}

impl PredicateData {
    pub fn alloc(db: &mut Database, ty: Type, bounds: Vec<TypeConstructor>) -> PredicateID {
        db.predicates.push(PredicateData::new(ty, bounds));

        PredicateID(db.predicates.len() - 1)
    }

    pub fn new(ty: Type, bounds: Vec<TypeConstructor>) -> Self {
        Self { ty, bounds }
    }
}

pub struct PredicateID(pub usize);

pub struct EnumItemData {
    docstring: Option<String>,
    name: IdentifierID,
}

pub struct EnumItemID(pub usize);

pub struct FunctionData {
    docstring: Option<String>,
    name: IdentifierID,
    visibility: Visibility,
    module: ModuleID,
}

pub struct FunctionID(pub usize);

impl FunctionData {
    pub fn alloc(
        db: &mut Database,
        docstring: Option<String>,
        name: IdentifierID,
        visibility: Visibility,
        module: ModuleID,
    ) -> FunctionID {
        db.functions
            .push(FunctionData::new(docstring, name, visibility, module));

        FunctionID(db.functions.len() - 1)
    }

    pub fn new(
        docstring: Option<String>,
        name: IdentifierID,
        visibility: Visibility,
        module: ModuleID,
    ) -> Self {
        Self {
            docstring,
            name,
            visibility,
            module,
        }
    }
}

pub struct InterfaceData {
    docstring: Option<String>,
    visibility: Visibility,
    name: IdentifierID,
    module: ModuleID,
    predicates: Vec<PredicateID>,
    methods: FxHashMap<IdentifierID, FunctionID>,
}

impl InterfaceData {
    pub fn alloc(
        db: &mut Database,
        docstring: Option<String>,
        visibility: Visibility,
        name: IdentifierID,
        module: ModuleID,
    ) -> InterfaceID {
        db.interfaces
            .push(InterfaceData::new(docstring, visibility, name, module));

        InterfaceID(db.interfaces.len() - 1)
    }

    pub fn new(
        docstring: Option<String>,
        visibility: Visibility,
        name: IdentifierID,
        module: ModuleID,
    ) -> Self {
        Self {
            docstring,
            visibility,
            name,
            module,
            predicates: Vec::new(),
            methods: FxHashMap::default(),
        }
    }
}

pub struct InterfaceID(pub usize);

pub struct ModuleData {
    docstring: Option<String>,
    name: IdentifierID,
    filepath: PathID,
    symbols: FxHashMap<IdentifierID, Symbol>,
}

impl ModuleData {
    pub fn alloc(
        db: &mut Database,
        name: IdentifierID,
        filepath: PathID,
        docstring: Option<String>,
    ) -> ModuleID {
        db.modules.push(ModuleData {
            docstring,
            name,
            filepath,
            symbols: FxHashMap::default(),
        });

        ModuleID(db.modules.len() - 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleID(pub usize);

#[derive(Default)]
pub struct Database {
    modules: Vec<ModuleData>,
    enums: Vec<EnumData>,
    enum_items: Vec<EnumItemData>,
    predicates: Vec<PredicateData>,
    structs: Vec<StructData>,
    fields: Vec<FieldData>,
    functions: Vec<FunctionData>,
    interfaces: Vec<InterfaceData>,
}

impl Database {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn module(&self, id: ModuleID) -> &ModuleData {
        &self.modules[id.0]
    }
}

#[derive(Default)]
pub struct State {
    db: Database,
    diagnostics: RwLock<Diagnostics>,
}

impl State {
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    #[must_use]
    pub const fn db(&self) -> &Database {
        &self.db
    }

    #[inline(always)]
    #[must_use]
    pub fn db_mut(&mut self) -> &mut Database {
        &mut self.db
    }

    #[inline(always)]
    #[must_use]
    pub fn diagnostics(&self) -> &RwLock<Diagnostics> {
        &self.diagnostics
    }
}
