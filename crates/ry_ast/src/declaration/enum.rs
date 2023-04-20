use super::{docstring::Documented, Item};
use crate::{name::Name, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct EnumDeclarationItem {
    pub visibility: Visibility,
    pub name: Name,
    pub variants: Vec<Documented<Name>>,
}

impl From<EnumDeclarationItem> for Item {
    fn from(enum_declaration: EnumDeclarationItem) -> Self {
        Self::EnumDeclaration(enum_declaration)
    }
}
