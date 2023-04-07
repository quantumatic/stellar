mod array;
mod r#as;
mod binary;
mod bool;
mod call;
mod char;
mod float;
mod r#if;
mod imaginary;
mod integer;
mod name;
mod property;
mod string;
mod type_annotations;
mod unary;
mod r#while;

pub use self::{
    array::ArrayLiteralExpression,
    binary::BinaryExpression,
    bool::BoolLiteralExpression,
    call::CallExpression,
    char::CharLiteralExpression,
    float::FloatLiteralExpression,
    imaginary::ImaginaryNumberLiteralExpression,
    integer::IntegerLiteralExpression,
    name::IdentifierExpression,
    property::PropertyAccessExpression,
    r#as::AsExpression,
    r#if::{IfBlock, IfExpression},
    r#while::WhileExpression,
    string::StringLiteralExpression,
    type_annotations::TypeAnnotationsExpression,
    unary::UnaryExpression,
};
use crate::span::Spanned;

pub type Expression = Spanned<RawExpression>;

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
    // At<usize>,
    // HashMap<usize, (Span, At<Expression>)>,
    // ),
    // Map(HashMap<usize, (Span, At<Expression>)>),
    // Call(Vec<Type>, Expression, Vec<Expression>),
    // Generics(Expression, Vec<Type>),
    If(IfExpression),
    While(WhileExpression),
    Call(CallExpression),
    TypeAnnotations(TypeAnnotationsExpression),
}

impl RawExpression {
    /// Returns `true` if expression contains block on its right
    /// hand side (last token is `}`).
    pub fn with_block(&self) -> bool {
        matches!(self, RawExpression::If(..) | RawExpression::While(..))
    }
}
