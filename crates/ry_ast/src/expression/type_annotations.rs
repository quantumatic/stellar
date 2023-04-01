use super::{Expression, RawExpression};
use crate::r#type::TypeAnnotations;

#[derive(Debug, PartialEq)]
pub struct TypeAnnotationsExpression {
    left: Expression,
    right: TypeAnnotations,
}

impl TypeAnnotationsExpression {
    #[inline]
    pub const fn new(left: Expression, right: TypeAnnotations) -> Self {
        Self { left, right }
    }

    #[inline]
    pub const fn left(&self) -> &Expression {
        &self.left
    }

    #[inline]
    pub const fn right(&self) -> &TypeAnnotations {
        &self.right
    }
}

impl From<TypeAnnotationsExpression> for RawExpression {
    fn from(type_annotations: TypeAnnotationsExpression) -> Self {
        Self::TypeAnnotations(type_annotations)
    }
}
