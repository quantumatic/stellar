//! `lib.rs` - defines AST nodes and additional stuff.
#![warn(
    clippy::all,
    clippy::doc_markdown,
    clippy::dbg_macro,
    clippy::todo,
    clippy::mem_forget,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::mismatched_target_os,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    rust_2018_idioms,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    nonstandard_style,
    unused_import_braces,
    unused_qualifications
)]
#![deny(
    clippy::await_holding_lock,
    clippy::if_let_mutex,
    clippy::indexing_slicing,
    clippy::mem_forget,
    clippy::ok_expect,
    clippy::unimplemented,
    clippy::unwrap_used,
    unsafe_code,
    unstable_features,
    unused_results
)]
#![allow(clippy::match_single_binding, clippy::inconsistent_struct_constructor)]

pub mod declaration;
pub mod expression;
pub mod name;
pub mod precedence;
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

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
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
    fn visit_with<V>(&self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor,
    {
        for item in &self.items {
            try_break!(item.visit_with(visitor));
        }

        ControlFlow::Continue(())
    }

    fn visit_with_mut<V>(&mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut,
    {
        for item in &mut self.items {
            try_break!(item.visit_with_mut(visitor));
        }

        ControlFlow::Continue(())
    }
}
