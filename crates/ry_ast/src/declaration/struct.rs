use crate::{
    name::Name,
    r#type::{Generics, Type, WhereClause},
    Mutability, Visibility,
};

use super::{docstring::Documented, Item};

#[derive(Debug, PartialEq)]
pub struct StructDeclarationItem {
    pub visibility: Visibility,
    pub name: Name,
    pub generics: Generics,
    pub r#where: WhereClause,
    pub members: Vec<Documented<StructMemberDeclaration>>,
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
