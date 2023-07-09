use ry_ast::{Expression, IdentifierAst, Literal, Pattern, Statement};
use ry_filesystem::span::Span;
use ry_interner::Interner;
use ry_parser::parse_statement;

mod r#macro;

#[test]
fn defer() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_statement("defer file.close();", &mut diagnostics, &mut interner),
        Some(Statement::Defer {
            call: Expression::Call {
                span: Span { start: 6, end: 18 },
                left: Box::new(Expression::FieldAccess {
                    span: Span { start: 6, end: 16 },
                    left: Box::new(Expression::Identifier(IdentifierAst {
                        span: Span { start: 6, end: 10 },
                        symbol: 19
                    })),
                    right: IdentifierAst {
                        span: Span { start: 11, end: 16 },
                        symbol: 20
                    }
                }),
                arguments: vec![]
            }
        })
    );
}

#[test]
fn r#break() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_statement("break;", &mut diagnostics, &mut interner),
        Some(Statement::Break {
            span: Span { start: 0, end: 5 }
        })
    );
}

#[test]
fn r#continue() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_statement("continue;", &mut diagnostics, &mut interner),
        Some(Statement::Continue {
            span: Span { start: 0, end: 8 }
        })
    );
}

#[test]
fn r#let() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_statement("let x = 1;", &mut diagnostics, &mut interner),
        Some(Statement::Let {
            pattern: Pattern::Identifier {
                span: Span { start: 4, end: 5 },
                identifier: IdentifierAst {
                    span: Span { start: 4, end: 5 },
                    symbol: 19
                },
                pattern: None
            },
            value: Expression::Literal(Literal::Integer {
                value: 1,
                span: Span { start: 8, end: 9 }
            }),
            ty: None
        })
    );
}
