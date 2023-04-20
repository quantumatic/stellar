use super::Type;
use crate::name::Name;
use serde::{Deserialize, Serialize};

pub type Generics = Vec<Generic>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Generic {
    pub name: Name,
    pub constraint: Option<Type>,
}

pub type TypeAnnotations = Vec<Type>;
