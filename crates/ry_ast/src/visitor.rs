// use std::ops::ControlFlow;

// use crate::*;

// macro_rules! try_break {
//     ($expr:expr) => {
//         match $expr {
//             core::ops::ControlFlow::Continue(c) => c,
//             core::ops::ControlFlow::Break(b) => return core::ops::ControlFlow::Break(b),
//         }
//     };
// }

// macro_rules! define_visit {
//     ($name:ident, $type:ty) => {
//         #[doc = concat!("Visits a `", stringify!($type), "` with this visitor")]
//         fn $name(&mut self, node: &$type) {
//             node.visit_with(self)
//         }
//     };
// }

// macro_rules! define_visit_mut {
//     ($name:ident, $type:ty) => {
//         #[doc = concat!("Visits a `", stringify!($type), "` with this visitor, mutably")]
//         fn $name(&mut self, node: &mut $type) {
//             node.visit_with_mut(self)
//         }
//     };
// }

// pub trait VisitWith {
//     fn visit_with<V>(&self, visitor: &mut V) -> ControlFlow<V::BreakTy>
//     where
//         V: Visitor;

//     fn visit_with_mut<V>(&self, visitor: &mut V) -> ControlFlow<V::BreakTy>
//     where
//         V: Visitor;
// }

// pub trait Visitor: Sized {
//     type BreakTy;

//     define_visit!(visit_program_unit, ProgramUnit);
//     define_visit!(visit_item, WithDocstring<Item>);
//     define_visit!(visit_import, Import);
// }

// pub trait VisitorMut: Sized {
//     type BreakTy;

//     define_visit_mut!(visit_program_unit, ProgramUnit);
//     define_visit_mut!(visit_item, WithDocstring<Item>);
//     define_visit_mut!(visit_import, Import);
// }
