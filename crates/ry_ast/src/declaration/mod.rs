pub use self::{
    docstring::{Docstring, WithDocstring, WithDocstringable},
    function::{Function, FunctionArgument, FunctionDeclaration, FunctionDefinition},
    import::ImportItem,
    r#enum::EnumDeclarationItem,
    r#impl::ImplItem,
    r#struct::{StructDeclarationItem, StructMemberDeclaration},
    r#trait::TraitDeclarationItem,
};

pub mod attribute;
pub mod docstring;
pub mod r#enum;
pub mod function;
pub mod r#impl;
pub mod import;
pub mod r#struct;
pub mod r#trait;

#[derive(Debug, PartialEq)]
pub enum Item {
    Import(ImportItem),
    Function(Function),
    EnumDeclaration(EnumDeclarationItem),
    TraitDeclaration(TraitDeclarationItem),
    StructDeclaration(StructDeclarationItem),
    Impl(ImplItem),
}
