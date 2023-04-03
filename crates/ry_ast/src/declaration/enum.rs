use crate::{name::Name, Visibility};

use super::{docstring::WithDocstring, Item};

#[derive(Debug, PartialEq)]
pub struct EnumDeclarationItem {
    pub visibility: Visibility,
    pub name: Name,
    pub variants: Vec<WithDocstring<Name>>,
}

impl EnumDeclarationItem {
    #[inline]
    pub const fn new(
        visibility: Visibility,
        name: Name,
        variants: Vec<WithDocstring<Name>>,
    ) -> EnumDeclarationItem {
        Self {
            visibility,
            name,
            variants,
        }
    }
}

impl From<EnumDeclarationItem> for Item {
    fn from(enum_declaration: EnumDeclarationItem) -> Self {
        Item::EnumDeclaration(enum_declaration)
    }
}
