use ry_ast::typed::{primitive_constructor, Type};
use ry_interner::{symbols::UINT8, Interner};
use ry_semantic_analysis::scope::Scope;
use std::sync::Arc;

#[test]
fn single_scope_lookup() {
    let mut interner = Interner::default();
    let a = interner.get_or_intern("a");
    let b = interner.get_or_intern("b");

    let mut scope = Scope::new(None);
    scope.add(a, Arc::new(Type::Unit));

    assert_eq!(scope.lookup(a), Some(Arc::new(Type::Unit)));
    assert_eq!(scope.lookup(b), None);
}

// Emulates the following case:
//
// ```
// { <- parent_scope
//   let a = Unit;
//   { <- scope
//     let b = 1;
//   }
// }
// ```
#[test]
fn inherited_scope_lookup() {
    let mut interner = Interner::default();
    let a = interner.get_or_intern("a");
    let b = interner.get_or_intern("b");

    let mut parent_scope = Scope::new(None);
    parent_scope.add(a, Arc::new(Type::Unit));

    let mut scope = Scope::new(Some(&parent_scope));
    scope.add(b, Arc::new(primitive_constructor(UINT8)));

    assert_eq!(scope.lookup(a), Some(Arc::new(Type::Unit)));
    assert_eq!(
        scope.lookup(b),
        Some(Arc::new(primitive_constructor(UINT8)))
    );

    assert_eq!(parent_scope.lookup(b), None);
}
