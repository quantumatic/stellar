use crate::{
    r#type::{generics::Generics, where_clause::WhereClause, Type},
    Visibility,
};

use super::{docstring::WithDocstring, function::FunctionDeclaration, Item};

#[derive(Debug, PartialEq)]
pub struct ImplItem {
    pub visibility: Visibility,
    pub generics: Generics,
    pub r#type: Type,
    pub r#trait: Option<Type>,
    pub r#where: WhereClause,
    pub methods: Vec<WithDocstring<FunctionDeclaration>>,
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
}

impl From<ImplItem> for Item {
    fn from(r#impl: ImplItem) -> Self {
        Self::Impl(r#impl)
    }
}
