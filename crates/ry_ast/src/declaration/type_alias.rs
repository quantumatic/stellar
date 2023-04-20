use super::{Item, TraitItem};
use crate::{
    name::Name,
    r#type::{Generics, Type},
    Visibility,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TypeAlias {
    pub visibility: Visibility,
    pub name: Name,
    pub generics: Generics,
    pub r#for: Option<Type>,
}

impl From<TypeAlias> for Item {
    fn from(alias: TypeAlias) -> Self {
        Self::TypeAlias(alias)
    }
}

impl From<TypeAlias> for TraitItem {
    fn from(alias: TypeAlias) -> Self {
        Self::TypeAlias(alias)
    }
}