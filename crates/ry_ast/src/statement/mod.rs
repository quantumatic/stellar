pub use self::{
    defer::DeferStatement, expression::ExpressionStatement, r#return::ReturnStatement,
    var::VarStatement,
};

pub mod defer;
pub mod expression;
pub mod r#return;
pub mod var;

pub type StatementsBlock = Vec<Statement>;

#[derive(Debug, PartialEq)]
pub enum Statement {
    Expression(ExpressionStatement),
    Return(ReturnStatement),
    Defer(DeferStatement),
    Var(VarStatement),
}
