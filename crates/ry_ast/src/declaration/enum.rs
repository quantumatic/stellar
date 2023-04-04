use super::{docstring::Documented, Item};
use crate::{name::Name, Visibility};

#[derive(Debug, PartialEq)]
pub struct EnumDeclarationItem {
    pub visibility: Visibility,
    pub name: Name,
    pub variants: Vec<Documented<Name>>,
}

impl From<EnumDeclarationItem> for Item {
    fn from(enum_declaration: EnumDeclarationItem) -> Self {
        Item::EnumDeclaration(enum_declaration)
    }
}
