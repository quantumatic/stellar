use super::Type;
use crate::name::Name;

pub type Generics = Vec<Generic>;

#[derive(Debug, PartialEq)]
pub struct Generic {
    name: Name,
    constraint: Option<Type>,
}

impl Generic {
    #[inline]
    pub const fn new(name: Name, constraint: Option<Type>) -> Self {
        Self { name, constraint }
    }

    #[inline]
    pub const fn name(&self) -> &Name {
        &self.name
    }

    #[inline]
    pub const fn constraint(&self) -> Option<&Type> {
        self.constraint.as_ref()
    }
}

pub type TypeAnnotations = Vec<Type>;
