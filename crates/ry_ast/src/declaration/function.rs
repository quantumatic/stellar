use super::{Item, TraitItem};
use crate::{
    expression::Expression,
    name::Name,
    r#type::{Generics, Type, WhereClause},
    statement::StatementsBlock,
    Visibility,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionDeclaration {
    pub signature: FunctionTypeSignature,
    pub body: StatementsBlock,
}

impl From<Function> for Item {
    fn from(function: Function) -> Self {
        Self::Function(function)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionTypeSignature {
    pub visibility: Visibility,
    pub name: Name,
    pub generics: Generics,
    pub arguments: Vec<FunctionArgument>,
    pub return_type: Option<Type>,
    pub r#where: WhereClause,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionArgument {
    pub name: Name,
    pub r#type: Type,
    pub default_value: Option<Expression>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Function {
    Definition(FunctionTypeSignature),
    Declaration(FunctionDeclaration),
}

impl From<FunctionTypeSignature> for Function {
    fn from(definition: FunctionTypeSignature) -> Self {
        Self::Definition(definition)
    }
}

impl From<FunctionDeclaration> for Function {
    fn from(declaration: FunctionDeclaration) -> Self {
        Self::Declaration(declaration)
    }
}

pub type AssociatedFunction = Function;

impl From<AssociatedFunction> for TraitItem {
    fn from(function: AssociatedFunction) -> Self {
        Self::AssociatedFunction(function)
    }
}
