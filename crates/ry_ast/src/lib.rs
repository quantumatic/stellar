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

use declaration::docstring::{Docstring, WithDocstring};
use declaration::Item;
use span::Span;

/// Represents Ry source file.
#[derive(Debug, PartialEq)]
pub struct ProgramUnit {
    /// Global source file docstring
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

pub type Visibility = Option<Span>;
pub type Mutability = Option<Span>;
