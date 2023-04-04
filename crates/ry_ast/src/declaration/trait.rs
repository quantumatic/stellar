use super::{docstring::Documented, function::Method, Item};
use crate::{
    name::Name,
    r#type::{generics::Generics, where_clause::WhereClause},
    Visibility,
};

#[derive(Debug, PartialEq)]
pub struct TraitDeclarationItem {
    pub visibility: Visibility,
    pub name: Name,
    pub generics: Generics,
    pub r#where: WhereClause,
    pub methods: Vec<Documented<Method>>,
}

impl From<TraitDeclarationItem> for Item {
    fn from(trait_declaration: TraitDeclarationItem) -> Self {
        Self::TraitDeclaration(trait_declaration)
    }
}
