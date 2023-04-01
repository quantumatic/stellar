use crate::{
    r#type::{generics::Generics, where_clause::WhereClause, Type},
    Visibility,
};

use super::{docstring::WithDocstring, function::FunctionDeclaration, Item};

#[derive(Debug, PartialEq)]
pub struct ImplItem {
    visibility: Visibility,
    generics: Generics,
    r#type: Type,
    r#trait: Option<Type>,
    r#where: WhereClause,
    methods: Vec<WithDocstring<FunctionDeclaration>>,
}

impl ImplItem {
    #[inline]
    pub const fn new(
        visibility: Visibility,
        generics: Generics,
        r#type: Type,
        r#trait: Option<Type>,
        r#where: WhereClause,
        methods: Vec<WithDocstring<FunctionDeclaration>>,
    ) -> Self {
        Self {
            visibility,
            generics,
            r#type,
            r#trait,
            r#where,
            methods,
        }
    }

    #[inline]
    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }

    #[inline]
    pub const fn generics(&self) -> &Generics {
        &self.generics
    }

    #[inline]
    pub const fn r#type(&self) -> &Type {
        &self.r#type
    }

    #[inline]
    pub const fn r#trait(&self) -> Option<&Type> {
        self.r#trait.as_ref()
    }

    pub const fn r#where(&self) -> &WhereClause {
        &self.r#where
    }

    pub const fn methods(&self) -> &Vec<WithDocstring<FunctionDeclaration>> {
        &self.methods
    }
}

impl From<ImplItem> for Item {
    fn from(r#impl: ImplItem) -> Self {
        Self::Impl(r#impl)
    }
}
