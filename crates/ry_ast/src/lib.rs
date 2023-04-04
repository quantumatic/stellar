//! `lib.rs` - defines AST nodes and additional stuff.
pub mod declaration;
pub mod expression;
pub mod name;
pub mod precedence;
pub mod span;
pub mod statement;
pub mod token;
pub mod r#type;
pub mod visitor;

use std::ops::ControlFlow;

use declaration::{docstring::*, Item};
use span::Span;
use visitor::*;

/// Represents Ry source file.
#[derive(Debug, PartialEq)]
pub struct ProgramUnit {
    pub docstring: Docstring,
    pub items: Items,
}

pub type Items = Vec<WithDocstring<Item>>;

impl ProgramUnit {
    #[inline]
    pub const fn new(docstring: Docstring, items: Items) -> Self {
        Self { docstring, items }
    }
}
#[derive(Debug, PartialEq)]
pub struct Visibility(Option<Span>);

#[derive(Debug, PartialEq)]
pub struct Mutability(Option<Span>);

impl Visibility {
    pub fn private() -> Self {
        Self(None)
    }

    pub fn public(span: Span) -> Self {
        Self(Some(span))
    }

    pub fn span_of_pub(&self) -> Option<Span> {
        self.0
    }
}

impl Mutability {
    pub fn immutable() -> Self {
        Self(None)
    }

    pub fn mutable(span: Span) -> Self {
        Self(Some(span))
    }

    pub fn span_of_mut(&self) -> Option<Span> {
        self.0
    }
}

impl VisitWith for ProgramUnit {
    fn visit_with<V>(&self, visitor: &mut V) -> std::ops::ControlFlow<V::BreakTy>
    where
        V: Visitor,
    {
        for item in &self.items {
            try_break!(item.visit_with(visitor));
        }

        ControlFlow::Continue(())
    }

    fn visit_with_mut<V>(&mut self, visitor: &mut V) -> std::ops::ControlFlow<V::BreakTy>
    where
        V: VisitorMut,
    {
        for item in &mut self.items {
            try_break!(item.visit_with_mut(visitor));
        }

        ControlFlow::Continue(())
    }
}
