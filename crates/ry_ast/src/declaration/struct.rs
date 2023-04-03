use crate::{
    name::Name,
    r#type::{generics::Generics, where_clause::WhereClause, Type},
    Mutability, Visibility,
};

use super::{docstring::WithDocstring, Item};

#[derive(Debug, PartialEq)]
pub struct StructDeclarationItem {
    pub visibility: Visibility,
    pub name: Name,
    pub generics: Generics,
    pub r#where: WhereClause,
    pub members: Vec<WithDocstring<StructMemberDeclaration>>,
}

impl StructDeclarationItem {
    #[inline]
    pub const fn new(
        visibility: Visibility,
        name: Name,
        generics: Generics,
        r#where: WhereClause,
        members: Vec<WithDocstring<StructMemberDeclaration>>,
    ) -> Self {
        Self {
            visibility,
            name,
            generics,
            r#where,
            members,
        }
    }
}

impl From<StructDeclarationItem> for Item {
    fn from(struct_declaration: StructDeclarationItem) -> Self {
        Self::StructDeclaration(struct_declaration)
    }
}

#[derive(Debug, PartialEq)]
pub struct StructMemberDeclaration {
    pub visibility: Visibility,
    pub mutability: Mutability,
    pub name: Name,
    pub r#type: Type,
}

impl StructMemberDeclaration {
    #[inline]
    pub const fn new(
        visibility: Visibility,
        mutability: Mutability,
        name: Name,
        r#type: Type,
    ) -> Self {
        Self {
            visibility,
            mutability,
            name,
            r#type,
        }
    }
}
