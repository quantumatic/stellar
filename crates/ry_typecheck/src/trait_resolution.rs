use ry_ast::{IdentifierAst, TypeBounds};
use ry_fx_hash::FxHashMap;
use ry_hir::ty::{Path, Type};

pub struct TraitResolutionContext {
    traits: FxHashMap<Path, TraitData>,
    implementations: FxHashMap<Path, TraitImplementationData>,
}

impl Default for TraitResolutionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TraitResolutionContext {
    pub fn new() -> Self {
        Self {
            traits: FxHashMap::default(),
            implementations: FxHashMap::default(),
        }
    }

    pub fn check_overlap(&self, trait_: Path, data: &TraitImplementationData) {
        todo!()
    }
}

pub struct TraitData {
    generics: Vec<IdentifierAst>,
    constraints: FxHashMap<Type, TypeBounds>,
}

pub struct TraitImplementationData {
    generics: Vec<IdentifierAst>,
    constraints: FxHashMap<Type, TypeBounds>,
    r#for: Type,
}
