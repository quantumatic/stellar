use string_interner::DefaultSymbol;

use crate::{
    expression::Expression,
    r#type::{generics::Generics, where_clause::WhereClause, Type},
    span::WithSpan,
    statement::StatementsBlock,
    Visibility,
};

use super::Item;

#[derive(Debug, PartialEq)]
pub struct FunctionDeclarationItem {
    defition: FunctionDefition,
    body: StatementsBlock,
}

impl FunctionDeclarationItem {
    pub const fn defition(&self) -> &FunctionDefition {
        &self.defition
    }

    pub const fn body(&self) -> &StatementsBlock {
        &self.body
    }
}

impl From<FunctionDeclarationItem> for Item {
    fn from(function_declaration: FunctionDeclarationItem) -> Self {
        Self::FunctionDeclaration(function_declaration)
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionDefition {
    public: Visibility,
    generics: Generics,
    name: WithSpan<DefaultSymbol>,
    arguments: Vec<FunctionArgument>,
    return_type: Option<Type>,
    r#where: WhereClause,
}

impl FunctionDefition {
    #[inline]
    pub const fn public(&self) -> Visibility {
        self.public
    }

    #[inline]
    pub const fn generics(&self) -> &Generics {
        &self.generics
    }

    #[inline]
    pub const fn name(&self) -> &WithSpan<DefaultSymbol> {
        &self.name
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
    name: WithSpan<DefaultSymbol>,
    r#type: Type,
    default_value: Option<Expression>,
}

impl FunctionArgument {
    #[inline]
    pub const fn name(&self) -> &WithSpan<DefaultSymbol> {
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
