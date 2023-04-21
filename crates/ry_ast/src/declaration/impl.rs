use super::{docstring::Documented, Item, TraitItem};
use crate::{
    r#type::{Generics, Type, WhereClause},
    Visibility,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ImplItem {
    pub visibility: Visibility,
    pub generics: Generics,
    pub r#type: Type,
    pub r#trait: Option<Type>,
    pub r#where: WhereClause,
    pub implementations: Vec<Documented<TraitItem>>,
}

impl From<ImplItem> for Item {
    fn from(r#impl: ImplItem) -> Self {
        Self::Impl(r#impl)
    }
}
