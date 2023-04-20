//! `lib.rs` - defines AST nodes and additional stuff.
pub mod declaration;
pub mod expression;
pub mod name;
pub mod precedence;
pub mod serialize;
pub mod span;
pub mod statement;
pub mod token;
pub mod r#type;
pub mod visitor;

use declaration::{Docstring, Documented, Item};
use serde::{Deserialize, Serialize};
use span::Span;
use std::ops::ControlFlow;
use visitor::*;

/// Represents Ry source file.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProgramUnit {
    pub docstring: Docstring,
    pub items: Items,
}

pub type Items = Vec<Documented<Item>>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Visibility(Option<Span>);

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

impl Default for Visibility {
    fn default() -> Self {
        Self::private()
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
