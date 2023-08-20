#![allow(dead_code, unused)]

use std::sync::Arc;

use hir_storage::HIRStorage;
use parking_lot::RwLock;
use ry_diagnostics::Diagnostics;
use ry_fx_hash::{FxHashMap, FxHashSet};
use ry_interner::{IdentifierInterner, PathInterner};
use ry_name_resolution::{DefinitionID, ResolutionEnvironment};
use ry_thir::{
    ty::{Type, TypeVariableID},
    ModuleItemSignature,
};
use thir_storage::THIRStorage;
use type_variable_factory::TypeVariableFactory;

pub mod diagnostics;
mod generic_parameters;
pub mod hir_storage;
pub mod resolution;
mod signature_analysis;
pub mod thir_storage;
pub mod type_variable_factory;

#[derive(Debug)]
pub enum ModuleItemState {
    HIR(ry_hir::ModuleItem),
    THIR(ry_thir::ModuleItem),
}

impl ModuleItemState {
    #[inline]
    #[must_use]
    pub const fn hir(&self) -> Option<&ry_hir::ModuleItem> {
        match self {
            Self::HIR(hir) => Some(hir),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn hir_or_panic(&self) -> &ry_hir::ModuleItem {
        self.hir()
            .unwrap_or_else(|| panic!("expected HIR, found already analyzed THIR"))
    }
}

/// Context for type checking stage of compilation.
#[derive(Debug)]
pub struct TypeCheckingContext<'i, 'p, 'd> {
    /// Global name resolution environment.
    resolution_environment: ResolutionEnvironment,

    /// HIR global storage.
    hir_storage: RwLock<HIRStorage>,

    /// THIR global storage.
    thir_storage: RwLock<THIRStorage>,

    /// List of type aliases, that have been recursivly analyzed. Used to find
    /// type alias cycles.
    type_alias_stack: FxHashSet<DefinitionID>,

    /// Identifier interner.
    identifier_interner: &'i IdentifierInterner,

    /// Path interner.
    path_interner: &'p PathInterner,

    /// Used to produce new type variables.
    type_variable_factory: TypeVariableFactory,

    /// Storage of signatures for module items.
    signatures: FxHashMap<DefinitionID, Arc<ModuleItemSignature>>,

    /// Substitutions.
    substitutions: FxHashMap<TypeVariableID, Type>,

    /// Diagnostics.
    diagnostics: &'d RwLock<Diagnostics>,
}

impl<'i, 'p, 'd> TypeCheckingContext<'i, 'p, 'd> {
    /// Creates a new empty type checking context.
    pub fn new(
        path_interner: &'p PathInterner,
        identifier_interner: &'i IdentifierInterner,
        diagnostics: &'d RwLock<Diagnostics>,
    ) -> Self {
        Self {
            path_interner,
            identifier_interner,
            diagnostics,
            type_alias_stack: FxHashSet::default(),
            hir_storage: RwLock::new(HIRStorage::new()),
            thir_storage: RwLock::new(THIRStorage::new()),
            resolution_environment: ResolutionEnvironment::new(),
            type_variable_factory: TypeVariableFactory::new(),
            substitutions: FxHashMap::default(),
            signatures: FxHashMap::default(),
        }
    }
}
