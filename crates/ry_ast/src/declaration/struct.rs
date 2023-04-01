use crate::{
    name::Name,
    r#type::{generics::Generics, where_clause::WhereClause, Type},
    Mutability, Visibility,
};

use super::{docstring::WithDocstring, Item};

#[derive(Debug, PartialEq)]
pub struct StructDeclarationItem {
    visibility: Visibility,
    name: Name,
    generics: Generics,
    r#where: WhereClause,
    members: Vec<WithDocstring<StructMemberDeclaration>>,
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
    pub const fn members(&self) -> &Vec<WithDocstring<StructMemberDeclaration>> {
        &self.members
    }
}

impl From<StructDeclarationItem> for Item {
    fn from(struct_declaration: StructDeclarationItem) -> Self {
        Self::StructDeclaration(struct_declaration)
    }
}

#[derive(Debug, PartialEq)]
pub struct StructMemberDeclaration {
    visibility: Visibility,
    mutability: Mutability,
    name: Name,
    r#type: Type,
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

    #[inline]
    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }

    #[inline]
    pub const fn mutability(&self) -> Mutability {
        self.mutability
    }

    #[inline]
    pub const fn name(&self) -> &Name {
        &self.name
    }

    #[inline]
    pub const fn r#type(&self) -> &Type {
        &self.r#type
    }
}
