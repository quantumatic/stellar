use crate::{
    name::Name,
    r#type::{generics::Generics, where_clause::WhereClause},
    Visibility,
};

use super::{docstring::WithDocstring, function::Method, Item};

#[derive(Debug, PartialEq)]
pub struct TraitDeclarationItem {
    visibility: Visibility,
    name: Name,
    generics: Generics,
    r#where: WhereClause,
    methods: Vec<WithDocstring<Method>>,
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

    #[inline]
    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }

    #[inline]
    pub const fn name(&self) -> &Name {
        &self.name
    }

    #[inline]
    pub const fn generics(&self) -> &Generics {
        &self.generics
    }

    #[inline]
    pub const fn r#where(&self) -> &WhereClause {
        &self.r#where
    }

    #[inline]
    pub const fn methods(&self) -> &Vec<WithDocstring<Method>> {
        &self.methods
    }
}

impl From<TraitDeclarationItem> for Item {
    fn from(trait_declaration: TraitDeclarationItem) -> Self {
        Self::TraitDeclaration(trait_declaration)
    }
}
