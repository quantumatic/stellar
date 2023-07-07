use ry_analyze::scope::{Scope, ValueConstructor};
use ry_ast::typed::primitive_constructor;
use ry_filesystem::span::DUMMY_SPAN;
use ry_interner::{
    symbols::{STRING, UINT8},
    Interner,
};
use std::sync::Arc;

// ```
// let a = 1;
// ```
#[test]
fn single_scope_lookup() {
    let mut interner = Interner::default();
    let a = interner.get_or_intern("a");
    let b = interner.get_or_intern("b");

    let mut scope = Scope::new(None);
    scope.add_symbol(
        a,
        ValueConstructor {
            origin: DUMMY_SPAN,
            ty: Arc::new(primitive_constructor(UINT8)),
        },
    );

    assert_eq!(
        scope.lookup(a),
        Some(&ValueConstructor {
            origin: DUMMY_SPAN,
            ty: Arc::new(primitive_constructor(UINT8))
        })
    );
    assert_eq!(scope.lookup(b), None);
}

// ```
// let a = 3;
// let a = "hello world"; // shadowing
// ```
#[test]
fn single_scope_shadowed_variable_lookup() {
    let mut interner = Interner::default();
    let a = interner.get_or_intern("a");

    let mut scope = Scope::new(None);
    assert_eq!(scope.lookup(a), None);

    scope.add_symbol(
        a,
        ValueConstructor {
            origin: DUMMY_SPAN,
            ty: Arc::new(primitive_constructor(UINT8)),
        },
    );

    assert_eq!(
        scope.lookup(a),
        Some(&ValueConstructor {
            origin: DUMMY_SPAN,
            ty: Arc::new(primitive_constructor(UINT8))
        })
    );

    scope.add_symbol(
        a,
        ValueConstructor {
            origin: DUMMY_SPAN,
            ty: Arc::new(primitive_constructor(STRING)),
        },
    );

    assert_eq!(
        scope.lookup(a),
        Some(&ValueConstructor {
            origin: DUMMY_SPAN,
            ty: Arc::new(primitive_constructor(STRING))
        })
    );
}

// ```
// {
//   let a = "hello";
//   {
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
    parent_scope.add_symbol(
        a,
        ValueConstructor {
            origin: DUMMY_SPAN,
            ty: Arc::new(primitive_constructor(STRING)),
        },
    );

    let mut inner_scope = Scope::new(Some(&parent_scope));
    inner_scope.add_symbol(
        b,
        ValueConstructor {
            origin: DUMMY_SPAN,
            ty: Arc::new(primitive_constructor(UINT8)),
        },
    );

    assert_eq!(
        inner_scope.lookup(a),
        Some(&ValueConstructor {
            origin: DUMMY_SPAN,
            ty: Arc::new(primitive_constructor(STRING))
        })
    );
    assert_eq!(
        inner_scope.lookup(b),
        Some(&ValueConstructor {
            origin: DUMMY_SPAN,
            ty: Arc::new(primitive_constructor(UINT8))
        })
    );

    assert_eq!(parent_scope.lookup(b), None);
}

// ```
// let a = 3;
// {
//   let a = "string"; // shadowing `a` in the inner scope
// }
// // previous `a` is back!
// ```
#[test]
fn inherited_scope_shadowed_variable_lookup() {
    let mut interner = Interner::default();
    let a = interner.get_or_intern("a");

    let mut parent_scope = Scope::new(None);
    assert_eq!(parent_scope.lookup(a), None);

    parent_scope.add_symbol(
        a,
        ValueConstructor {
            origin: DUMMY_SPAN,
            ty: Arc::new(primitive_constructor(UINT8)),
        },
    );

    let mut inner_scope = Scope::new(Some(&parent_scope));
    inner_scope.add_symbol(
        a,
        ValueConstructor {
            origin: DUMMY_SPAN,
            ty: Arc::new(primitive_constructor(STRING)),
        },
    );

    assert_eq!(
        inner_scope.lookup(a),
        Some(&ValueConstructor {
            origin: DUMMY_SPAN,
            ty: Arc::new(primitive_constructor(STRING))
        })
    );
    assert_eq!(
        parent_scope.lookup(a),
        Some(&ValueConstructor {
            origin: DUMMY_SPAN,
            ty: Arc::new(primitive_constructor(UINT8))
        })
    );
}
