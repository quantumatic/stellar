use super::{Expression, RawExpression};
use crate::r#type::TypeAnnotations;

#[derive(Debug, PartialEq)]
pub struct TypeAnnotationsExpression {
    pub left: Box<Expression>,
    pub right: TypeAnnotations,
}

impl TypeAnnotationsExpression {
    #[inline]
    pub fn new(left: Expression, right: TypeAnnotations) -> Self {
        Self { left: Box::new(left), right }
    }
}

impl From<TypeAnnotationsExpression> for RawExpression {
    fn from(type_annotations: TypeAnnotationsExpression) -> Self {
        Self::TypeAnnotations(type_annotations)
    }
}
