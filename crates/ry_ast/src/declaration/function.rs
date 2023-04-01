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
    definition: FunctionDefinition,
    body: StatementsBlock,
}

impl FunctionDeclaration {
    #[inline]
    pub const fn new(definition: FunctionDefinition, body: StatementsBlock) -> Self {
        Self { definition, body }
    }

    #[inline]
    pub const fn definition(&self) -> &FunctionDefinition {
        &self.definition
    }

    #[inline]
    pub const fn body(&self) -> &StatementsBlock {
        &self.body
    }
}

impl From<Function> for Item {
    fn from(function: Function) -> Self {
        Self::Function(function)
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionDefinition {
    visibility: Visibility,
    name: Name,
    generics: Generics,
    arguments: Vec<FunctionArgument>,
    return_type: Option<Type>,
    r#where: WhereClause,
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

    #[inline]
    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }

    #[inline]
    pub const fn name(&self) -> &Name {
        &self.name
    }

    #[inline]
    pub const fn generics(&self) -> &Generics {
        &self.generics
    }

    #[inline]
    pub const fn arguments(&self) -> &Vec<FunctionArgument> {
        &self.arguments
    }

    #[inline]
    pub const fn return_type(&self) -> Option<&Type> {
        self.return_type.as_ref()
    }

    #[inline]
    pub const fn r#where(&self) -> &WhereClause {
        &self.r#where
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionArgument {
    name: Name,
    r#type: Type,
    default_value: Option<Expression>,
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

    #[inline]
    pub const fn name(&self) -> &Name {
        &self.name
    }

    #[inline]
    pub const fn r#type(&self) -> &Type {
        &self.r#type
    }

    #[inline]
    pub const fn default_value(&self) -> Option<&Expression> {
        self.default_value.as_ref()
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
