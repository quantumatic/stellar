use ry_ast::typed::primitive_constructor;
use ry_interner::{
    symbols::{STRING, UINT8},
    Interner,
};
use ry_semantic_analysis::scope::{Scope, SymbolData};
use ry_source_file::span::DUMMY_SPAN;
use std::sync::Arc;

#[test]
fn single_scope_lookup() {
    let mut interner = Interner::default();
    let a = interner.get_or_intern("a");
    let b = interner.get_or_intern("b");

    let mut scope = Scope::new(None);
    scope.add(
        a,
        SymbolData::new(DUMMY_SPAN, Arc::new(primitive_constructor(UINT8))),
    );

    assert_eq!(
        scope.lookup(a),
        Some(&SymbolData::new(
            DUMMY_SPAN,
            Arc::new(primitive_constructor(UINT8))
        ))
    );
    assert_eq!(scope.lookup(b), None);
}

// Emulates the following case:
//
// ```
// { <- parent_scope
//   let a = "hello";
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
    parent_scope.add(
        a,
        SymbolData::new(DUMMY_SPAN, Arc::new(primitive_constructor(STRING))),
    );

    let mut scope = Scope::new(Some(&parent_scope));
    scope.add(
        b,
        SymbolData::new(DUMMY_SPAN, Arc::new(primitive_constructor(UINT8))),
    );

    assert_eq!(
        scope.lookup(a),
        Some(&SymbolData::new(
            DUMMY_SPAN,
            Arc::new(primitive_constructor(STRING))
        ))
    );
    assert_eq!(
        scope.lookup(b),
        Some(&SymbolData::new(
            DUMMY_SPAN,
            Arc::new(primitive_constructor(UINT8))
        ))
    );

    assert_eq!(parent_scope.lookup(b), None);
}
