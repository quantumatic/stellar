use ry_ast::{
    BinaryOperator, Expression, GenericArgument, IdentifierAst, LambdaFunctionParameter, Literal,
    MatchExpressionItem, Path, Pattern, PostfixOperator, PrefixOperator, RawBinaryOperator,
    RawPostfixOperator, RawPrefixOperator, Statement, StructExpressionItem, Type, TypePath,
    TypePathSegment,
};
use ry_filesystem::span::Span;
use ry_interner::{symbols, Interner};
use ry_parser::parse_expression;

mod r#macro;

#[test]
fn literal() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_expression("3", &mut diagnostics, &mut interner),
        Some(Expression::Literal(Literal::Integer {
            value: 3,
            span: Span { start: 0, end: 1 }
        }))
    );
    assert_eq!(
        parse_expression("true", &mut diagnostics, &mut interner),
        Some(Expression::Literal(Literal::Boolean {
            value: true,
            span: Span { start: 0, end: 4 }
        }))
    );
    assert_eq!(
        parse_expression("false", &mut diagnostics, &mut interner),
        Some(Expression::Literal(Literal::Boolean {
            value: false,
            span: Span { start: 0, end: 5 }
        }))
    );
    assert_eq!(
        parse_expression("\"hello\"", &mut diagnostics, &mut interner),
        Some(Expression::Literal(Literal::String {
            value: "hello".to_owned(),
            span: Span { start: 0, end: 7 }
        }))
    );
    assert_eq!(
        parse_expression("3.2e-2", &mut diagnostics, &mut interner),
        Some(Expression::Literal(Literal::Float {
            value: 3.2e-2,
            span: Span { start: 0, end: 6 }
        }))
    );
    assert_eq!(
        parse_expression("'a'", &mut diagnostics, &mut interner),
        Some(Expression::Literal(Literal::Character {
            value: 'a',
            span: Span { start: 0, end: 3 }
        }))
    );
}

#[test]
fn call() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_expression("foo()", &mut diagnostics, &mut interner),
        Some(Expression::Call {
            span: Span { start: 0, end: 5 },
            left: Box::new(Expression::Identifier(IdentifierAst {
                span: Span { start: 0, end: 3 },
                symbol: interner.get_or_intern("foo"),
            })),
            arguments: vec![],
        })
    );
}

#[test]
fn postfix() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_expression("x++", &mut diagnostics, &mut interner),
        Some(Expression::Postfix {
            span: Span { start: 0, end: 3 },
            inner: Box::new(Expression::Identifier(IdentifierAst {
                span: Span { start: 0, end: 1 },
                symbol: interner.get_or_intern("x")
            })),
            operator: PostfixOperator {
                span: Span { start: 1, end: 3 },
                raw: RawPostfixOperator::PlusPlus
            }
        })
    );
}

#[test]
fn generic_argument() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_expression("sizeof[uint32]()", &mut diagnostics, &mut interner),
        Some(Expression::Call {
            span: Span { start: 0, end: 16 },
            left: Box::new(Expression::GenericArguments {
                span: Span { start: 0, end: 14 },
                left: Box::new(Expression::Identifier(IdentifierAst {
                    span: Span { start: 0, end: 6 },
                    symbol: symbols::SIZE_OF
                })),
                generic_arguments: vec![GenericArgument::Type(Type::Path(TypePath {
                    span: Span { start: 7, end: 13 },
                    segments: vec![TypePathSegment {
                        span: Span { start: 7, end: 13 },
                        path: Path {
                            span: Span { start: 7, end: 13 },
                            identifiers: vec![IdentifierAst {
                                span: Span { start: 7, end: 13 },
                                symbol: symbols::UINT32
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
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_expression("[1, true, \"3\".into()]", &mut diagnostics, &mut interner),
        Some(Expression::List {
            span: Span { start: 0, end: 21 },
            elements: vec![
                Expression::Literal(Literal::Integer {
                    value: 1,
                    span: Span { start: 1, end: 2 }
                }),
                Expression::Literal(Literal::Boolean {
                    value: true,
                    span: Span { start: 4, end: 8 }
                }),
                Expression::Call {
                    span: Span { start: 10, end: 20 },
                    left: Box::new(Expression::FieldAccess {
                        span: Span { start: 10, end: 18 },
                        left: Box::new(Expression::Literal(Literal::String {
                            value: "3".to_owned(),
                            span: Span { start: 10, end: 13 }
                        })),
                        right: IdentifierAst {
                            span: Span { start: 14, end: 18 },
                            symbol: interner.get_or_intern("into")
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
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_expression("(1, true, \"3\".into())", &mut diagnostics, &mut interner),
        Some(Expression::Tuple {
            span: Span { start: 0, end: 21 },
            elements: vec![
                Expression::Literal(Literal::Integer {
                    value: 1,
                    span: Span { start: 1, end: 2 }
                }),
                Expression::Literal(Literal::Boolean {
                    value: true,
                    span: Span { start: 4, end: 8 }
                }),
                Expression::Call {
                    span: Span { start: 10, end: 20 },
                    left: Box::new(Expression::FieldAccess {
                        span: Span { start: 10, end: 18 },
                        left: Box::new(Expression::Literal(Literal::String {
                            value: "3".to_owned(),
                            span: Span { start: 10, end: 13 }
                        })),
                        right: IdentifierAst {
                            span: Span { start: 14, end: 18 },
                            symbol: interner.get_or_intern("into")
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
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_expression(
            "(1 + (0 * 2 + c) + d()) - !a?",
            &mut diagnostics,
            &mut interner
        ),
        Some(Expression::Binary {
            span: Span { start: 0, end: 29 },
            left: Box::new(Expression::Parenthesized {
                span: Span { start: 0, end: 23 },
                inner: Box::new(Expression::Binary {
                    span: Span { start: 1, end: 22 },
                    left: Box::new(Expression::Binary {
                        span: Span { start: 1, end: 16 },
                        left: Box::new(Expression::Literal(Literal::Integer {
                            value: 1,
                            span: Span { start: 1, end: 2 }
                        })),
                        operator: BinaryOperator {
                            span: Span { start: 3, end: 4 },
                            raw: RawBinaryOperator::Plus
                        },
                        right: Box::new(Expression::Parenthesized {
                            span: Span { start: 5, end: 16 },
                            inner: Box::new(Expression::Binary {
                                span: Span { start: 6, end: 15 },
                                left: Box::new(Expression::Binary {
                                    span: Span { start: 6, end: 11 },
                                    left: Box::new(Expression::Literal(Literal::Integer {
                                        value: 0,
                                        span: Span { start: 6, end: 7 }
                                    })),
                                    operator: BinaryOperator {
                                        span: Span { start: 8, end: 9 },
                                        raw: RawBinaryOperator::Star
                                    },
                                    right: Box::new(Expression::Literal(Literal::Integer {
                                        value: 2,
                                        span: Span { start: 10, end: 11 }
                                    }))
                                }),
                                operator: BinaryOperator {
                                    span: Span { start: 12, end: 13 },
                                    raw: RawBinaryOperator::Plus
                                },
                                right: Box::new(Expression::Identifier(IdentifierAst {
                                    span: Span { start: 14, end: 15 },
                                    symbol: interner.get_or_intern("c")
                                }))
                            })
                        })
                    }),
                    operator: BinaryOperator {
                        span: Span { start: 17, end: 18 },
                        raw: RawBinaryOperator::Plus
                    },
                    right: Box::new(Expression::Call {
                        span: Span { start: 19, end: 22 },
                        left: Box::new(Expression::Identifier(IdentifierAst {
                            span: Span { start: 19, end: 20 },
                            symbol: interner.get_or_intern("d")
                        })),
                        arguments: vec![]
                    })
                })
            }),
            operator: BinaryOperator {
                span: Span { start: 24, end: 25 },
                raw: RawBinaryOperator::Minus
            },
            right: Box::new(Expression::Postfix {
                span: Span { start: 26, end: 29 },
                inner: Box::new(Expression::Prefix {
                    span: Span { start: 26, end: 28 },
                    inner: Box::new(Expression::Identifier(IdentifierAst {
                        span: Span { start: 27, end: 28 },
                        symbol: interner.get_or_intern("a")
                    })),
                    operator: PrefixOperator {
                        span: Span { start: 26, end: 27 },
                        raw: RawPrefixOperator::Bang
                    }
                }),
                operator: PostfixOperator {
                    span: Span { start: 28, end: 29 },
                    raw: RawPostfixOperator::QuestionMark
                }
            })
        })
    );
}

#[test]
fn r#as() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_expression("1 as float32", &mut diagnostics, &mut interner),
        Some(Expression::As {
            span: Span { start: 0, end: 12 },
            left: Box::new(Expression::Literal(Literal::Integer {
                value: 1,
                span: Span { start: 0, end: 1 }
            })),
            right: Type::Path(TypePath {
                span: Span { start: 5, end: 12 },
                segments: vec![TypePathSegment {
                    span: Span { start: 5, end: 12 },
                    path: Path {
                        span: Span { start: 5, end: 12 },
                        identifiers: vec![IdentifierAst {
                            span: Span { start: 5, end: 12 },
                            symbol: symbols::FLOAT32
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
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_expression("if true { 1 } else { 0 }", &mut diagnostics, &mut interner),
        Some(Expression::If {
            span: Span { start: 0, end: 24 },
            if_blocks: vec![(
                Expression::Literal(Literal::Boolean {
                    value: true,
                    span: Span { start: 3, end: 7 }
                }),
                vec![Statement::Expression {
                    expression: Expression::Literal(Literal::Integer {
                        value: 1,
                        span: Span { start: 10, end: 11 }
                    }),
                    has_semicolon: false
                }]
            )],
            r#else: Some(vec![Statement::Expression {
                expression: Expression::Literal(Literal::Integer {
                    value: 0,
                    span: Span { start: 21, end: 22 }
                }),
                has_semicolon: false
            }])
        })
    );
}

#[test]
fn r#struct() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_expression("Person { age: 25, name }", &mut diagnostics, &mut interner),
        Some(Expression::Struct {
            span: Span { start: 0, end: 24 },
            left: Box::new(Expression::Identifier(IdentifierAst {
                span: Span { start: 0, end: 6 },
                symbol: interner.get_or_intern("Person")
            })),
            fields: vec![
                StructExpressionItem {
                    name: IdentifierAst {
                        span: Span { start: 9, end: 12 },
                        symbol: interner.get_or_intern("age")
                    },
                    value: Some(Expression::Literal(Literal::Integer {
                        span: Span { start: 14, end: 16 },
                        value: 25
                    }))
                },
                StructExpressionItem {
                    name: IdentifierAst {
                        span: Span { start: 18, end: 22 },
                        symbol: interner.get_or_intern("name")
                    },
                    value: None
                }
            ]
        })
    );
}

#[test]
fn r#while() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_expression(
            "while true { code(); eat(); sleep(); }",
            &mut diagnostics,
            &mut interner
        ),
        Some(Expression::While {
            span: Span { start: 0, end: 38 },
            condition: Box::new(Expression::Literal(Literal::Boolean {
                value: true,
                span: Span { start: 6, end: 10 }
            })),
            body: vec![
                Statement::Expression {
                    expression: Expression::Call {
                        span: Span { start: 13, end: 19 },
                        left: Box::new(Expression::Identifier(IdentifierAst {
                            span: Span { start: 13, end: 17 },
                            symbol: interner.get_or_intern("code")
                        })),
                        arguments: vec![]
                    },
                    has_semicolon: true
                },
                Statement::Expression {
                    expression: Expression::Call {
                        span: Span { start: 21, end: 26 },
                        left: Box::new(Expression::Identifier(IdentifierAst {
                            span: Span { start: 21, end: 24 },
                            symbol: interner.get_or_intern("eat")
                        })),
                        arguments: vec![]
                    },
                    has_semicolon: true
                },
                Statement::Expression {
                    expression: Expression::Call {
                        span: Span { start: 28, end: 35 },
                        left: Box::new(Expression::Identifier(IdentifierAst {
                            span: Span { start: 28, end: 33 },
                            symbol: interner.get_or_intern("sleep")
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
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_expression("|a, b: uint32| { a + b }", &mut diagnostics, &mut interner),
        Some(Expression::Function {
            span: Span { start: 0, end: 24 },
            parameters: vec![
                LambdaFunctionParameter {
                    name: IdentifierAst {
                        span: Span { start: 1, end: 2 },
                        symbol: interner.get_or_intern("a")
                    },
                    ty: None
                },
                LambdaFunctionParameter {
                    name: IdentifierAst {
                        span: Span { start: 4, end: 5 },
                        symbol: interner.get_or_intern("b")
                    },
                    ty: Some(Type::Path(TypePath {
                        span: Span { start: 7, end: 13 },
                        segments: vec![TypePathSegment {
                            span: Span { start: 7, end: 13 },
                            path: Path {
                                span: Span { start: 7, end: 13 },
                                identifiers: vec![IdentifierAst {
                                    span: Span { start: 7, end: 13 },
                                    symbol: symbols::UINT32
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
                    span: Span { start: 17, end: 22 },
                    left: Box::new(Expression::Identifier(IdentifierAst {
                        span: Span { start: 17, end: 18 },
                        symbol: interner.get_or_intern("a")
                    })),
                    operator: BinaryOperator {
                        span: Span { start: 19, end: 20 },
                        raw: RawBinaryOperator::Plus
                    },
                    right: Box::new(Expression::Identifier(IdentifierAst {
                        span: Span { start: 21, end: 22 },
                        symbol: interner.get_or_intern("b")
                    }))
                },
                has_semicolon: false
            }]
        })
    );
}

#[test]
fn r#match() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_expression(
            "match Some(3) { Some(a) => println(a), .. => {} }",
            &mut diagnostics,
            &mut interner
        ),
        Some(Expression::Match {
            span: Span { start: 0, end: 49 },
            expression: Box::new(Expression::Call {
                span: Span { start: 6, end: 13 },
                left: Box::new(Expression::Identifier(IdentifierAst {
                    span: Span { start: 6, end: 10 },
                    symbol: interner.get_or_intern("Some")
                })),
                arguments: vec![Expression::Literal(Literal::Integer {
                    value: 3,
                    span: Span { start: 11, end: 12 }
                })]
            }),
            block: vec![
                MatchExpressionItem {
                    left: Pattern::TupleLike {
                        span: Span { start: 16, end: 23 },
                        path: Path {
                            span: Span { start: 16, end: 20 },
                            identifiers: vec![IdentifierAst {
                                span: Span { start: 16, end: 20 },
                                symbol: interner.get_or_intern("Some")
                            }]
                        },
                        inner_patterns: vec![Pattern::Identifier {
                            span: Span { start: 21, end: 22 },
                            identifier: IdentifierAst {
                                span: Span { start: 21, end: 22 },
                                symbol: interner.get_or_intern("a")
                            },
                            pattern: None
                        }]
                    },
                    right: Expression::Call {
                        span: Span { start: 27, end: 37 },
                        left: Box::new(Expression::Identifier(IdentifierAst {
                            span: Span { start: 27, end: 34 },
                            symbol: interner.get_or_intern("println")
                        })),
                        arguments: vec![Expression::Identifier(IdentifierAst {
                            span: Span { start: 35, end: 36 },
                            symbol: interner.get_or_intern("a")
                        })]
                    }
                },
                MatchExpressionItem {
                    left: Pattern::Rest {
                        span: Span { start: 42, end: 44 }
                    },
                    right: Expression::StatementsBlock {
                        span: Span { start: 45, end: 47 },
                        block: vec![]
                    }
                }
            ]
        })
    );
}
