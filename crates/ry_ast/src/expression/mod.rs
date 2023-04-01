pub mod array;
pub mod r#as;
pub mod binary;
pub mod bool;
pub mod call;
pub mod char;
pub mod float;
pub mod r#if;
pub mod imaginary;
pub mod integer;
pub mod name;
pub mod property;
pub mod string;
pub mod type_annotations;
pub mod unary;
pub mod r#while;

use self::{
    array::ArrayLiteralExpression, binary::BinaryExpression, bool::BoolLiteralExpression,
    call::CallExpression, char::CharLiteralExpression, float::FloatLiteralExpression,
    imaginary::ImaginaryNumberLiteralExpression, integer::IntegerLiteralExpression,
    name::IdentifierExpression, property::PropertyAccessExpression, r#as::AsExpression,
    r#if::IfExpression, r#while::WhileExpression, string::StringLiteralExpression,
    type_annotations::TypeAnnotationsExpression, unary::UnaryExpression,
};
use crate::span::WithSpan;

pub type Expression = WithSpan<Box<RawExpression>>;

#[derive(Debug, PartialEq)]
pub enum RawExpression {
    StringLiteral(StringLiteralExpression),
    Integer(IntegerLiteralExpression),
    Float(FloatLiteralExpression),
    ImaginaryNumber(ImaginaryNumberLiteralExpression),
    Bool(BoolLiteralExpression),
    Char(CharLiteralExpression),
    Array(ArrayLiteralExpression),
    Identifier(IdentifierExpression),
    Binary(BinaryExpression),
    As(AsExpression),
    Unary(UnaryExpression),
    Property(PropertyAccessExpression),
    // Struct(
    // WithSpan<DefaultSymbol>,
    // HashMap<DefaultSymbol, (Span, WithSpan<Expression>)>,
    // ),
    // Map(HashMap<DefaultSymbol, (Span, WithSpan<Expression>)>),
    // Call(Vec<Type>, Expression, Vec<Expression>),
    // Generics(Expression, Vec<Type>),
    If(IfExpression),
    While(WhileExpression),
    Call(CallExpression),
    TypeAnnotations(TypeAnnotationsExpression),
}

impl RawExpression {
    pub fn must_have_semicolon_at_the_end(&self) -> bool {
        !matches!(self, RawExpression::If(..) | RawExpression::While(..))
    }
}
