mod defer;
mod expression;
mod r#return;
mod var;

pub use self::{
    defer::DeferStatement, expression::ExpressionStatement, r#return::ReturnStatement,
    var::VarStatement,
};

pub type StatementsBlock = Vec<Statement>;

#[derive(Debug, PartialEq)]
pub enum Statement {
    Expression(ExpressionStatement),
    Return(ReturnStatement),
    Defer(DeferStatement),
    Var(VarStatement),
}
