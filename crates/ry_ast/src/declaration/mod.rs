pub use self::{
    docstring::*,
    function::{Function, FunctionArgument, FunctionDeclaration, FunctionDefinition},
    import::ImportItem,
    r#enum::EnumDeclarationItem,
    r#impl::ImplItem,
    r#struct::{StructDeclarationItem, StructMemberDeclaration},
    r#trait::TraitDeclarationItem,
};
use crate::visitor::{VisitWith, Visitor, VisitorMut};
use std::ops::ControlFlow;

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

impl VisitWith for Documented<Item> {
    fn visit_with<V>(&self, _visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor,
    {
        ControlFlow::Continue(())
    }

    fn visit_with_mut<V>(&mut self, _visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut,
    {
        ControlFlow::Continue(())
    }
}
