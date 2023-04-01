use crate::{name::Name, Visibility};

use super::{docstring::WithDocstring, Item};

#[derive(Debug, PartialEq)]
pub struct EnumDeclarationItem {
    visibility: Visibility,
    name: Name,
    variants: Vec<WithDocstring<Name>>,
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

    #[inline]
    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }

    #[inline]
    pub const fn variants(&self) -> &Vec<WithDocstring<Name>> {
        &self.variants
    }

    #[inline]
    pub const fn name(&self) -> &Name {
        &self.name
    }
}

impl From<EnumDeclarationItem> for Item {
    fn from(enum_declaration: EnumDeclarationItem) -> Self {
        Item::EnumDeclaration(enum_declaration)
    }
}
