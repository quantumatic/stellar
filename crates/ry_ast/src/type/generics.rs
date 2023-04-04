use super::Type;
use crate::name::Name;

pub type Generics = Vec<Generic>;

#[derive(Debug, PartialEq)]
pub struct Generic {
    pub name: Name,
    pub constraint: Option<Type>,
}

pub type TypeAnnotations = Vec<Type>;
