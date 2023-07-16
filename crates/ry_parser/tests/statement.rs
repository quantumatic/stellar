use ry_ast::{Expression, IdentifierAst, Literal, Pattern, Statement};
use ry_diagnostics::GlobalDiagnostics;
use ry_filesystem::{location::Location, path_interner::DUMMY_PATH_ID};
use ry_interner::Interner;
use ry_parser::parse_statement;

#[test]
fn defer() {
    let mut interner = Interner::default();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_statement(
            DUMMY_PATH_ID,
            "defer file.close();",
            &mut diagnostics,
            &mut interner
        ),
        Some(Statement::Defer {
            call: Expression::Call {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 6,
                    end: 18
                },
                callee: Box::new(Expression::FieldAccess {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 6,
                        end: 16
                    },
                    left: Box::new(Expression::Identifier(IdentifierAst {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 6,
                            end: 10
                        },
                        symbol: interner.get_or_intern("file")
                    })),
                    right: IdentifierAst {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 11,
                            end: 16
                        },
                        symbol: interner.get_or_intern("close")
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
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_statement(DUMMY_PATH_ID, "break;", &mut diagnostics, &mut interner),
        Some(Statement::Break {
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 5
            }
        })
    );
}

#[test]
fn r#continue() {
    let mut interner = Interner::default();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_statement(DUMMY_PATH_ID, "continue;", &mut diagnostics, &mut interner),
        Some(Statement::Continue {
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 8
            }
        })
    );
}

#[test]
fn r#let() {
    let mut interner = Interner::default();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_statement(DUMMY_PATH_ID, "let x = 1;", &mut diagnostics, &mut interner),
        Some(Statement::Let {
            pattern: Pattern::Identifier {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 4,
                    end: 5
                },
                identifier: IdentifierAst {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 4,
                        end: 5
                    },
                    symbol: interner.get_or_intern("x")
                },
                pattern: None
            },
            value: Expression::Literal(Literal::Integer {
                value: 1,
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 8,
                    end: 9
                }
            }),
            ty: None
        })
    );
}
