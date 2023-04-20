use super::{docstring::Documented, function::AssociatedFunction, type_alias::TypeAlias, Item};
use crate::{
    name::Name,
    r#type::{Generics, WhereClause},
    Visibility,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TraitDeclarationItem {
    pub visibility: Visibility,
    pub name: Name,
    pub generics: Generics,
    pub r#where: WhereClause,
    pub items: Vec<Documented<TraitItem>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum TraitItem {
    TypeAlias(TypeAlias),
    AssociatedFunction(AssociatedFunction),
}

impl From<TraitDeclarationItem> for Item {
    fn from(trait_declaration: TraitDeclarationItem) -> Self {
        Self::TraitDeclaration(trait_declaration)
    }
}
