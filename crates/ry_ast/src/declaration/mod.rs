use self::{function::FunctionDeclarationItem, import::ImportItem, r#enum::EnumDeclarationItem};

pub mod docstring;
pub mod r#enum;
pub mod function;
pub mod import;

#[derive(Debug, PartialEq)]
pub enum Item {
    Import(ImportItem),
    FunctionDeclaration(FunctionDeclarationItem),
    EnumDeclaration(EnumDeclarationItem),
}
