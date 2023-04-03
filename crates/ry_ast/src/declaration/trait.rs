use crate::{
    name::Name,
    r#type::{generics::Generics, where_clause::WhereClause},
    Visibility,
};

use super::{docstring::WithDocstring, function::Method, Item};

#[derive(Debug, PartialEq)]
pub struct TraitDeclarationItem {
    pub visibility: Visibility,
    pub name: Name,
    pub generics: Generics,
    pub r#where: WhereClause,
    pub methods: Vec<WithDocstring<Method>>,
}

impl TraitDeclarationItem {
    #[inline]
    pub const fn new(
        visibility: Visibility,
        name: Name,
        generics: Generics,
        r#where: WhereClause,
        methods: Vec<WithDocstring<Method>>,
    ) -> Self {
        Self {
            visibility,
            name,
            generics,
            r#where,
            methods,
        }
    }
}

impl From<TraitDeclarationItem> for Item {
    fn from(trait_declaration: TraitDeclarationItem) -> Self {
        Self::TraitDeclaration(trait_declaration)
    }
}
