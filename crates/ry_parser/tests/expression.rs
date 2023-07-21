use ry_ast::{
    BinaryOperator, Expression, GenericArgument, IdentifierAST, LambdaFunctionParameter, Literal,
    MatchExpressionItem, Path, Pattern, PostfixOperator, PrefixOperator, RawBinaryOperator,
    RawPostfixOperator, RawPrefixOperator, Statement, StructExpressionItem, Type, TypePath,
    TypePathSegment,
};
use ry_diagnostics::GlobalDiagnostics;
use ry_filesystem::location::Location;
use ry_interner::{builtin_symbols, IdentifierInterner, DUMMY_PATH_ID};
use ry_parser::parse_expression;

#[test]
fn literal() {
    let mut identifier_interner = IdentifierInterner::new();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "3",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::Literal(Literal::Integer {
            value: 3,
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 1
            }
        }))
    );
    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "true",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::Literal(Literal::Boolean {
            value: true,
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 4
            }
        }))
    );
    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "false",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::Literal(Literal::Boolean {
            value: false,
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 5
            }
        }))
    );
    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "\"hello\"",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::Literal(Literal::String {
            value: "hello".to_owned(),
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 7
            }
        }))
    );
    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "3.2e-2",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::Literal(Literal::Float {
            value: 3.2e-2,
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 6
            }
        }))
    );
    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "'a'",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::Literal(Literal::Character {
            value: 'a',
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 3
            }
        }))
    );
}

#[test]
fn call() {
    let mut identifier_interner = IdentifierInterner::new();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "foo()",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::Call {
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 5
            },
            callee: Box::new(Expression::Identifier(IdentifierAST {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 0,
                    end: 3
                },
                symbol: identifier_interner.get_or_intern("foo"),
            })),
            arguments: vec![],
        })
    );
}

#[test]
fn postfix() {
    let mut identifier_interner = IdentifierInterner::new();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "x++",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::Postfix {
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 3
            },
            inner: Box::new(Expression::Identifier(IdentifierAST {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 0,
                    end: 1
                },
                symbol: identifier_interner.get_or_intern("x")
            })),
            operator: PostfixOperator {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 1,
                    end: 3
                },
                raw: RawPostfixOperator::DoublePlus
            }
        })
    );
}

#[test]
fn generic_argument() {
    let mut identifier_interner = IdentifierInterner::new();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "sizeof[uint32]()",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::Call {
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 16
            },
            callee: Box::new(Expression::GenericArguments {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 0,
                    end: 14
                },
                left: Box::new(Expression::Identifier(IdentifierAST {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 0,
                        end: 6
                    },
                    symbol: builtin_symbols::SIZE_OF
                })),
                generic_arguments: vec![GenericArgument::Type(Type::Path(TypePath {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 7,
                        end: 13
                    },
                    segments: vec![TypePathSegment {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 7,
                            end: 13
                        },
                        path: Path {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 7,
                                end: 13
                            },
                            identifiers: vec![IdentifierAST {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 7,
                                    end: 13
                                },
                                symbol: builtin_symbols::UINT32
                            }]
                        },
                        generic_arguments: None
                    }]
                }))]
            }),
            arguments: vec![]
        })
    );
}

#[test]
fn list() {
    let mut identifier_interner = IdentifierInterner::new();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "[1, true, \"3\".into()]",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::List {
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 21
            },
            elements: vec![
                Expression::Literal(Literal::Integer {
                    value: 1,
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 1,
                        end: 2
                    }
                }),
                Expression::Literal(Literal::Boolean {
                    value: true,
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 4,
                        end: 8
                    }
                }),
                Expression::Call {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 10,
                        end: 20
                    },
                    callee: Box::new(Expression::FieldAccess {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 10,
                            end: 18
                        },
                        left: Box::new(Expression::Literal(Literal::String {
                            value: "3".to_owned(),
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 10,
                                end: 13
                            }
                        })),
                        right: IdentifierAST {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 14,
                                end: 18
                            },
                            symbol: identifier_interner.get_or_intern("into")
                        },
                    }),
                    arguments: vec![]
                }
            ]
        })
    );
}

#[test]
fn tuple() {
    let mut identifier_interner = IdentifierInterner::new();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "(1, true, \"3\".into())",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::Tuple {
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 21
            },
            elements: vec![
                Expression::Literal(Literal::Integer {
                    value: 1,
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 1,
                        end: 2
                    }
                }),
                Expression::Literal(Literal::Boolean {
                    value: true,
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 4,
                        end: 8
                    }
                }),
                Expression::Call {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 10,
                        end: 20
                    },
                    callee: Box::new(Expression::FieldAccess {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 10,
                            end: 18
                        },
                        left: Box::new(Expression::Literal(Literal::String {
                            value: "3".to_owned(),
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 10,
                                end: 13
                            }
                        })),
                        right: IdentifierAST {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 14,
                                end: 18
                            },
                            symbol: identifier_interner.get_or_intern("into")
                        }
                    }),
                    arguments: vec![]
                }
            ]
        })
    );
}

#[test]
fn binary() {
    let mut identifier_interner = IdentifierInterner::new();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "(1 + (0 * 2 + c) + d()) - !a?",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::Binary {
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 29
            },
            left: Box::new(Expression::Parenthesized {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 0,
                    end: 23
                },
                inner: Box::new(Expression::Binary {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 1,
                        end: 22
                    },
                    left: Box::new(Expression::Binary {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 1,
                            end: 16
                        },
                        left: Box::new(Expression::Literal(Literal::Integer {
                            value: 1,
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 1,
                                end: 2
                            }
                        })),
                        operator: BinaryOperator {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 3,
                                end: 4
                            },
                            raw: RawBinaryOperator::Plus
                        },
                        right: Box::new(Expression::Parenthesized {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 5,
                                end: 16
                            },
                            inner: Box::new(Expression::Binary {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 6,
                                    end: 15
                                },
                                left: Box::new(Expression::Binary {
                                    location: Location {
                                        file_path_id: DUMMY_PATH_ID,
                                        start: 6,
                                        end: 11
                                    },
                                    left: Box::new(Expression::Literal(Literal::Integer {
                                        value: 0,
                                        location: Location {
                                            file_path_id: DUMMY_PATH_ID,
                                            start: 6,
                                            end: 7
                                        }
                                    })),
                                    operator: BinaryOperator {
                                        location: Location {
                                            file_path_id: DUMMY_PATH_ID,
                                            start: 8,
                                            end: 9
                                        },
                                        raw: RawBinaryOperator::Star
                                    },
                                    right: Box::new(Expression::Literal(Literal::Integer {
                                        value: 2,
                                        location: Location {
                                            file_path_id: DUMMY_PATH_ID,
                                            start: 10,
                                            end: 11
                                        }
                                    }))
                                }),
                                operator: BinaryOperator {
                                    location: Location {
                                        file_path_id: DUMMY_PATH_ID,
                                        start: 12,
                                        end: 13
                                    },
                                    raw: RawBinaryOperator::Plus
                                },
                                right: Box::new(Expression::Identifier(IdentifierAST {
                                    location: Location {
                                        file_path_id: DUMMY_PATH_ID,
                                        start: 14,
                                        end: 15
                                    },
                                    symbol: identifier_interner.get_or_intern("c")
                                }))
                            })
                        })
                    }),
                    operator: BinaryOperator {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 17,
                            end: 18
                        },
                        raw: RawBinaryOperator::Plus
                    },
                    right: Box::new(Expression::Call {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 19,
                            end: 22
                        },
                        callee: Box::new(Expression::Identifier(IdentifierAST {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 19,
                                end: 20
                            },
                            symbol: identifier_interner.get_or_intern("d")
                        })),
                        arguments: vec![]
                    })
                })
            }),
            operator: BinaryOperator {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 24,
                    end: 25
                },
                raw: RawBinaryOperator::Minus
            },
            right: Box::new(Expression::Postfix {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 26,
                    end: 29
                },
                inner: Box::new(Expression::Prefix {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 26,
                        end: 28
                    },
                    inner: Box::new(Expression::Identifier(IdentifierAST {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 27,
                            end: 28
                        },
                        symbol: identifier_interner.get_or_intern("a")
                    })),
                    operator: PrefixOperator {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 26,
                            end: 27
                        },
                        raw: RawPrefixOperator::Bang
                    }
                }),
                operator: PostfixOperator {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 28,
                        end: 29
                    },
                    raw: RawPostfixOperator::QuestionMark
                }
            })
        })
    );
}

#[test]
fn r#as() {
    let mut identifier_interner = IdentifierInterner::new();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "1 as float32",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::As {
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 12
            },
            left: Box::new(Expression::Literal(Literal::Integer {
                value: 1,
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 0,
                    end: 1
                }
            })),
            right: Type::Path(TypePath {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 5,
                    end: 12
                },
                segments: vec![TypePathSegment {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 5,
                        end: 12
                    },
                    path: Path {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 5,
                            end: 12
                        },
                        identifiers: vec![IdentifierAST {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 5,
                                end: 12
                            },
                            symbol: builtin_symbols::FLOAT32
                        }]
                    },
                    generic_arguments: None
                }]
            })
        })
    );
}

#[test]
fn ifelse() {
    let mut identifier_interner = IdentifierInterner::new();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "if true { 1 } else { 0 }",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::If {
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 24
            },
            if_blocks: vec![(
                Expression::Literal(Literal::Boolean {
                    value: true,
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 3,
                        end: 7
                    }
                }),
                vec![Statement::Expression {
                    expression: Expression::Literal(Literal::Integer {
                        value: 1,
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 10,
                            end: 11
                        }
                    }),
                    has_semicolon: false
                }]
            )],
            r#else: Some(vec![Statement::Expression {
                expression: Expression::Literal(Literal::Integer {
                    value: 0,
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 21,
                        end: 22
                    }
                }),
                has_semicolon: false
            }])
        })
    );
}

#[test]
fn r#struct() {
    let mut identifier_interner = IdentifierInterner::new();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "Person { age: 25, name }",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::Struct {
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 24
            },
            left: Box::new(Expression::Identifier(IdentifierAST {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 0,
                    end: 6
                },
                symbol: identifier_interner.get_or_intern("Person")
            })),
            fields: vec![
                StructExpressionItem {
                    name: IdentifierAST {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 9,
                            end: 12
                        },
                        symbol: identifier_interner.get_or_intern("age")
                    },
                    value: Some(Expression::Literal(Literal::Integer {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 14,
                            end: 16
                        },
                        value: 25
                    }))
                },
                StructExpressionItem {
                    name: IdentifierAST {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 18,
                            end: 22
                        },
                        symbol: identifier_interner.get_or_intern("name")
                    },
                    value: None
                }
            ]
        })
    );
}

#[test]
fn r#while() {
    let mut identifier_interner = IdentifierInterner::new();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "while true { code(); eat(); sleep(); }",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::While {
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 38
            },
            condition: Box::new(Expression::Literal(Literal::Boolean {
                value: true,
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 6,
                    end: 10
                }
            })),
            statements_block: vec![
                Statement::Expression {
                    expression: Expression::Call {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 13,
                            end: 19
                        },
                        callee: Box::new(Expression::Identifier(IdentifierAST {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 13,
                                end: 17
                            },
                            symbol: identifier_interner.get_or_intern("code")
                        })),
                        arguments: vec![]
                    },
                    has_semicolon: true
                },
                Statement::Expression {
                    expression: Expression::Call {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 21,
                            end: 26
                        },
                        callee: Box::new(Expression::Identifier(IdentifierAST {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 21,
                                end: 24
                            },
                            symbol: identifier_interner.get_or_intern("eat")
                        })),
                        arguments: vec![]
                    },
                    has_semicolon: true
                },
                Statement::Expression {
                    expression: Expression::Call {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 28,
                            end: 35
                        },
                        callee: Box::new(Expression::Identifier(IdentifierAST {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 28,
                                end: 33
                            },
                            symbol: identifier_interner.get_or_intern("sleep")
                        })),
                        arguments: vec![]
                    },
                    has_semicolon: true
                }
            ]
        })
    );
}

#[test]
fn lambda() {
    let mut identifier_interner = IdentifierInterner::new();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "|a, b: uint32| { a + b }",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::Lambda {
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 24
            },
            parameters: vec![
                LambdaFunctionParameter {
                    name: IdentifierAST {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 1,
                            end: 2
                        },
                        symbol: identifier_interner.get_or_intern("a")
                    },
                    ty: None
                },
                LambdaFunctionParameter {
                    name: IdentifierAST {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 4,
                            end: 5
                        },
                        symbol: identifier_interner.get_or_intern("b")
                    },
                    ty: Some(Type::Path(TypePath {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 7,
                            end: 13
                        },
                        segments: vec![TypePathSegment {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 7,
                                end: 13
                            },
                            path: Path {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 7,
                                    end: 13
                                },
                                identifiers: vec![IdentifierAST {
                                    location: Location {
                                        file_path_id: DUMMY_PATH_ID,
                                        start: 7,
                                        end: 13
                                    },
                                    symbol: builtin_symbols::UINT32
                                }]
                            },
                            generic_arguments: None
                        }]
                    }))
                }
            ],
            return_type: None,
            block: vec![Statement::Expression {
                expression: Expression::Binary {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 17,
                        end: 22
                    },
                    left: Box::new(Expression::Identifier(IdentifierAST {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 17,
                            end: 18
                        },
                        symbol: identifier_interner.get_or_intern("a")
                    })),
                    operator: BinaryOperator {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 19,
                            end: 20
                        },
                        raw: RawBinaryOperator::Plus
                    },
                    right: Box::new(Expression::Identifier(IdentifierAST {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 21,
                            end: 22
                        },
                        symbol: identifier_interner.get_or_intern("b")
                    }))
                },
                has_semicolon: false
            }]
        })
    );
}

#[test]
fn r#match() {
    let mut identifier_interner = IdentifierInterner::new();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_expression(
            DUMMY_PATH_ID,
            "match Some(3) { Some(a) => println(a), .. => {} }",
            &mut diagnostics,
            &mut identifier_interner
        ),
        Some(Expression::Match {
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 49
            },
            expression: Box::new(Expression::Call {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 6,
                    end: 13
                },
                callee: Box::new(Expression::Identifier(IdentifierAST {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 6,
                        end: 10
                    },
                    symbol: identifier_interner.get_or_intern("Some")
                })),
                arguments: vec![Expression::Literal(Literal::Integer {
                    value: 3,
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 11,
                        end: 12
                    }
                })]
            }),
            block: vec![
                MatchExpressionItem {
                    left: Pattern::TupleLike {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 16,
                            end: 23
                        },
                        path: Path {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 16,
                                end: 20
                            },
                            identifiers: vec![IdentifierAST {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 16,
                                    end: 20
                                },
                                symbol: identifier_interner.get_or_intern("Some")
                            }]
                        },
                        inner_patterns: vec![Pattern::Identifier {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 21,
                                end: 22
                            },
                            identifier: IdentifierAST {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 21,
                                    end: 22
                                },
                                symbol: identifier_interner.get_or_intern("a")
                            },
                            pattern: None
                        }]
                    },
                    right: Expression::Call {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 27,
                            end: 37
                        },
                        callee: Box::new(Expression::Identifier(IdentifierAST {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 27,
                                end: 34
                            },
                            symbol: identifier_interner.get_or_intern("println")
                        })),
                        arguments: vec![Expression::Identifier(IdentifierAST {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 35,
                                end: 36
                            },
                            symbol: identifier_interner.get_or_intern("a")
                        })]
                    }
                },
                MatchExpressionItem {
                    left: Pattern::Rest {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 42,
                            end: 44
                        }
                    },
                    right: Expression::StatementsBlock {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 45,
                            end: 47
                        },
                        block: vec![]
                    }
                }
            ]
        })
    );
}
