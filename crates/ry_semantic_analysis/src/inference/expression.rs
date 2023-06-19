use ry_ast::{Expression, Literal};
use ry_interner::{BOOL, CHAR, FLOAT64, STRING, UINT16, UINT32, UINT64, UINT8};

use super::InferenceContext;

impl InferenceContext {
    fn infer_literal_type(&mut self, literal: &Literal) -> Type {
        match literal {
            Literal::Integer { value, .. } => match value {
                0..=255u64 => self.primitive_type_constructor(UINT8),
                256..=65_535u64 => self.primitive_type_constructor(UINT16),
                65_536..=4_294_967_295u64 => self.primitive_type_constructor(UINT32),
                4_294_967_296..=18_446_744_073_709_551_615u64 => {
                    self.primitive_type_constructor(UINT64)
                }
            },
            Literal::String { .. } => self.primitive_type_constructor(STRING),
            Literal::Character { .. } => self.primitive_type_constructor(CHAR),
            Literal::Boolean { .. } => self.primitive_type_constructor(BOOL),
            Literal::Float { .. } => self.primitive_type_constructor(FLOAT64),
        }
    }

    fn infer_expression_type(&mut self, expression: &Expression) -> Type {
        match expression {
            Expression::Literal(literal) => self.infer_literal_type(literal),
            _ => todo!(),
        }
    }
}
