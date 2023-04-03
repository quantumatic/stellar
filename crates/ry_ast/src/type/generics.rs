use super::Type;
use crate::name::Name;

pub type Generics = Vec<Generic>;

#[derive(Debug, PartialEq)]
pub struct Generic {
    pub name: Name,
    pub constraint: Option<Type>,
}

impl Generic {
    #[inline]
    pub const fn new(name: Name, constraint: Option<Type>) -> Self {
        Self { name, constraint }
    }
}

pub type TypeAnnotations = Vec<Type>;
