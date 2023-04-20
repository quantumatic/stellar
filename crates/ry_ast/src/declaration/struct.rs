use super::{docstring::Documented, Item};
use crate::{
    name::Name,
    r#type::{Generics, Type, WhereClause},
    Visibility,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct StructMemberDeclaration {
    pub visibility: Visibility,
    pub name: Name,
    pub r#type: Type,
}
