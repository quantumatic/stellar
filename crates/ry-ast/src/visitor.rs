use ry_proc_macros::visit_fn;
use string_interner::DefaultSymbol;

use crate::{
    location::WithSpan, EnumDecl, FunctionDecl, Impl, Import, Item, Items, ProgramUnit, StructDecl,
    TraitDecl,
};

pub trait Visitor: Sized {
    visit_fn!(program_unit, &ProgramUnit);

    visit_fn!(import, &Import);
    visit_fn!(import_after_first_item, &Import);

    visit_fn!(items, &Items);
    visit_fn!(item, (&str, &Item));

    visit_fn!(enum_decl, (&str, &EnumDecl));
    visit_fn!(function_decl, (&str, &FunctionDecl));
    visit_fn!(struct_decl, (&str, &StructDecl));
    visit_fn!(trait_decl, (&str, &TraitDecl));
    visit_fn!(r#impl, (&str, &Impl));

    visit_fn!(static_name, &WithSpan<Vec<DefaultSymbol>>);
}

pub fn walk_program_unit<V: Visitor>(visitor: &mut V, node: &ProgramUnit) {
    for import in &node.imports {
        visitor.visit_import(import);
    }

    for item in &node.items {
        visitor.visit_item((&item.0, &item.1));
    }
}

pub fn walk_import<V: Visitor>(_visitor: &mut V, _node: &Import) {
    // No-op
}

pub fn walk_import_after_first_item<V: Visitor>(_visitor: &mut V, _node: &Import) {
    // No-op
}

pub fn walk_items<V: Visitor>(visitor: &mut V, nodes: &Items) {
    for node in nodes {
        visitor.visit_item((&node.0, &node.1));
    }
}

pub fn walk_item<V: Visitor>(visitor: &mut V, node: (&str, &Item)) {
    match node.1 {
        Item::EnumDecl(e) => visitor.visit_enum_decl((node.0, e)),
        Item::FunctionDecl(f) => visitor.visit_function_decl((node.0, f)),
        Item::StructDecl(s) => visitor.visit_struct_decl((node.0, s)),
        Item::Impl(i) => visitor.visit_impl((node.0, i)),
        Item::TraitDecl(t) => visitor.visit_trait_decl((node.0, t)),
        Item::Import(i) => visitor.visit_import_after_first_item(i),
    }
}

#[allow(unused_variables)]
pub fn walk_enum_decl<V: Visitor>(visitor: &mut V, node: (&str, &EnumDecl)) {}

#[allow(unused_variables)]
pub fn walk_function_decl<V: Visitor>(visitor: &mut V, node: (&str, &FunctionDecl)) {}

#[allow(unused_variables)]
pub fn walk_struct_decl<V: Visitor>(visitor: &mut V, node: (&str, &StructDecl)) {}

#[allow(unused_variables)]
pub fn walk_impl<V: Visitor>(visitor: &mut V, node: (&str, &Impl)) {}

#[allow(unused_variables)]
pub fn walk_trait_decl<V: Visitor>(visitor: &mut V, node: (&str, &TraitDecl)) {}

#[allow(unused_variables)]
pub fn walk_static_name<V: Visitor>(visitor: &mut V, node: &WithSpan<Vec<DefaultSymbol>>) {}
