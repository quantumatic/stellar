use crate::name::Name;

use super::{Expression, RawExpression};

#[derive(Debug, PartialEq)]
pub struct PropertyAccessExpression {
    left: Expression,
    right: Name,
}

impl PropertyAccessExpression {
    #[inline]
    pub const fn new(left: Expression, right: Name) -> Self {
        Self { left, right }
    }

    #[inline]
    pub const fn left(&self) -> &Expression {
        &self.left
    }

    #[inline]
    pub const fn right(&self) -> &Name {
        &self.right
    }
}

impl From<PropertyAccessExpression> for RawExpression {
    fn from(property_access: PropertyAccessExpression) -> Self {
        Self::Property(property_access)
    }
}
