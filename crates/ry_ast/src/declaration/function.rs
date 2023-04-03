use super::Item;
use crate::{
    expression::Expression,
    name::Name,
    r#type::{generics::Generics, where_clause::WhereClause, Type},
    statement::StatementsBlock,
    Visibility,
};

#[derive(Debug, PartialEq)]
pub struct FunctionDeclaration {
    pub definition: FunctionDefinition,
    pub body: StatementsBlock,
}

impl FunctionDeclaration {
    #[inline]
    pub const fn new(definition: FunctionDefinition, body: StatementsBlock) -> Self {
        Self { definition, body }
    }
}

impl From<Function> for Item {
    fn from(function: Function) -> Self {
        Self::Function(function)
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionDefinition {
    pub visibility: Visibility,
    pub name: Name,
    pub generics: Generics,
    pub arguments: Vec<FunctionArgument>,
    pub return_type: Option<Type>,
    pub r#where: WhereClause,
}

impl FunctionDefinition {
    #[inline]
    pub const fn new(
        visibility: Visibility,
        name: Name,
        generics: Generics,
        arguments: Vec<FunctionArgument>,
        return_type: Option<Type>,
        r#where: WhereClause,
    ) -> Self {
        Self {
            visibility,
            name,
            generics,
            arguments,
            return_type,
            r#where,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionArgument {
    pub name: Name,
    pub r#type: Type,
    pub default_value: Option<Expression>,
}

impl FunctionArgument {
    #[inline]
    pub const fn new(name: Name, r#type: Type, default_value: Option<Expression>) -> Self {
        Self {
            name,
            r#type,
            default_value,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Function {
    Definition(FunctionDefinition),
    Declaration(FunctionDeclaration),
}

impl From<FunctionDefinition> for Function {
    fn from(definition: FunctionDefinition) -> Self {
        Self::Definition(definition)
    }
}

impl From<FunctionDeclaration> for Function {
    fn from(declaration: FunctionDeclaration) -> Self {
        Self::Declaration(declaration)
    }
}

pub type Method = Function;
