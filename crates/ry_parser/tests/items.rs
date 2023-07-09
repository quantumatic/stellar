use ry_ast::{
    EnumItem, Expression, Function, FunctionParameter, GenericArgument, GenericParameter,
    IdentifierAst, Impl, ImportPath, Item, JustFunctionParameter, Path, SelfParameter, Statement,
    StructField, TraitItem, TupleField, Type, TypeAlias, TypePath, TypePathSegment, Visibility,
    WhereClauseItem,
};
use ry_filesystem::span::Span;
use ry_interner::Interner;
use ry_parser::parse_item;

mod r#macro;

#[test]
fn function() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_item(
            "fun foo[T, B = Option[T]](a: B): T { a.unwrap() }",
            &mut diagnostics,
            &mut interner
        ),
        Some(Item::Function(Function {
            visibility: Visibility::private(),
            name: IdentifierAst {
                span: Span { start: 4, end: 7 },
                symbol: interner.get_or_intern("foo")
            },
            generic_parameters: Some(vec![
                GenericParameter {
                    name: IdentifierAst {
                        span: Span { start: 8, end: 9 },
                        symbol: interner.get_or_intern("T")
                    },
                    bounds: None,
                    default_value: None
                },
                GenericParameter {
                    name: IdentifierAst {
                        span: Span { start: 11, end: 12 },
                        symbol: interner.get_or_intern("B")
                    },
                    bounds: None,
                    default_value: Some(Type::Path(TypePath {
                        span: Span { start: 15, end: 24 },
                        segments: vec![TypePathSegment {
                            span: Span { start: 15, end: 24 },
                            path: Path {
                                span: Span { start: 15, end: 21 },
                                identifiers: vec![IdentifierAst {
                                    span: Span { start: 15, end: 21 },
                                    symbol: interner.get_or_intern("Option")
                                }]
                            },
                            generic_arguments: Some(vec![GenericArgument::Type(Type::Path(
                                TypePath {
                                    span: Span { start: 22, end: 23 },
                                    segments: vec![TypePathSegment {
                                        span: Span { start: 22, end: 23 },
                                        path: Path {
                                            span: Span { start: 22, end: 23 },
                                            identifiers: vec![IdentifierAst {
                                                span: Span { start: 22, end: 23 },
                                                symbol: interner.get_or_intern("T")
                                            }]
                                        },
                                        generic_arguments: None
                                    }]
                                }
                            ))])
                        }]
                    }))
                }
            ]),
            parameters: vec![FunctionParameter::Just(JustFunctionParameter {
                name: IdentifierAst {
                    span: Span { start: 26, end: 27 },
                    symbol: interner.get_or_intern("a")
                },
                ty: Type::Path(TypePath {
                    span: Span { start: 29, end: 30 },
                    segments: vec![TypePathSegment {
                        span: Span { start: 29, end: 30 },
                        path: Path {
                            span: Span { start: 29, end: 30 },
                            identifiers: vec![IdentifierAst {
                                span: Span { start: 29, end: 30 },
                                symbol: interner.get_or_intern("B")
                            }]
                        },
                        generic_arguments: None
                    }]
                })
            })],
            return_type: Some(Type::Path(TypePath {
                span: Span { start: 33, end: 34 },
                segments: vec![TypePathSegment {
                    span: Span { start: 33, end: 34 },
                    path: Path {
                        span: Span { start: 33, end: 34 },
                        identifiers: vec![IdentifierAst {
                            span: Span { start: 33, end: 34 },
                            symbol: interner.get_or_intern("T")
                        }]
                    },
                    generic_arguments: None
                }]
            })),
            where_clause: None,
            body: Some(vec![Statement::Expression {
                expression: Expression::Call {
                    span: Span { start: 37, end: 47 },
                    left: Box::new(Expression::FieldAccess {
                        span: Span { start: 37, end: 45 },
                        left: Box::new(Expression::Identifier(IdentifierAst {
                            span: Span { start: 37, end: 38 },
                            symbol: interner.get_or_intern("a")
                        })),
                        right: IdentifierAst {
                            span: Span { start: 39, end: 45 },
                            symbol: interner.get_or_intern("unwrap")
                        }
                    }),
                    arguments: vec![]
                },
                has_semicolon: false
            }]),
            docstring: None
        }))
    );
}

#[test]
fn r#impl() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_item(
            "impl[A, B] Into[Option[(A, B)]] for (A, B) {}",
            &mut diagnostics,
            &mut interner
        ),
        Some(Item::Impl(Impl {
            generic_parameters: Some(vec![
                GenericParameter {
                    name: IdentifierAst {
                        span: Span { start: 5, end: 6 },
                        symbol: 19
                    },
                    bounds: None,
                    default_value: None
                },
                GenericParameter {
                    name: IdentifierAst {
                        span: Span { start: 8, end: 9 },
                        symbol: 20
                    },
                    bounds: None,
                    default_value: None
                }
            ]),
            ty: Type::Tuple {
                span: Span { start: 36, end: 42 },
                element_types: vec![
                    Type::Path(TypePath {
                        span: Span { start: 37, end: 38 },
                        segments: vec![TypePathSegment {
                            span: Span { start: 37, end: 38 },
                            path: Path {
                                span: Span { start: 37, end: 38 },
                                identifiers: vec![IdentifierAst {
                                    span: Span { start: 37, end: 38 },
                                    symbol: 19
                                }]
                            },
                            generic_arguments: None
                        }]
                    }),
                    Type::Path(TypePath {
                        span: Span { start: 40, end: 41 },
                        segments: vec![TypePathSegment {
                            span: Span { start: 40, end: 41 },
                            path: Path {
                                span: Span { start: 40, end: 41 },
                                identifiers: vec![IdentifierAst {
                                    span: Span { start: 40, end: 41 },
                                    symbol: 20
                                }]
                            },
                            generic_arguments: None
                        }]
                    })
                ]
            },
            r#trait: Some(Type::Path(TypePath {
                span: Span { start: 11, end: 31 },
                segments: vec![TypePathSegment {
                    span: Span { start: 11, end: 31 },
                    path: Path {
                        span: Span { start: 11, end: 15 },
                        identifiers: vec![IdentifierAst {
                            span: Span { start: 11, end: 15 },
                            symbol: 21
                        }]
                    },
                    generic_arguments: Some(vec![GenericArgument::Type(Type::Path(TypePath {
                        span: Span { start: 16, end: 30 },
                        segments: vec![TypePathSegment {
                            span: Span { start: 16, end: 30 },
                            path: Path {
                                span: Span { start: 16, end: 22 },
                                identifiers: vec![IdentifierAst {
                                    span: Span { start: 16, end: 22 },
                                    symbol: 22
                                }]
                            },
                            generic_arguments: Some(vec![GenericArgument::Type(Type::Tuple {
                                span: Span { start: 23, end: 29 },
                                element_types: vec![
                                    Type::Path(TypePath {
                                        span: Span { start: 24, end: 25 },
                                        segments: vec![TypePathSegment {
                                            span: Span { start: 24, end: 25 },
                                            path: Path {
                                                span: Span { start: 24, end: 25 },
                                                identifiers: vec![IdentifierAst {
                                                    span: Span { start: 24, end: 25 },
                                                    symbol: 19
                                                }]
                                            },
                                            generic_arguments: None
                                        }]
                                    }),
                                    Type::Path(TypePath {
                                        span: Span { start: 27, end: 28 },
                                        segments: vec![TypePathSegment {
                                            span: Span { start: 27, end: 28 },
                                            path: Path {
                                                span: Span { start: 27, end: 28 },
                                                identifiers: vec![IdentifierAst {
                                                    span: Span { start: 27, end: 28 },
                                                    symbol: 20
                                                }]
                                            },
                                            generic_arguments: None
                                        }]
                                    })
                                ]
                            })])
                        }]
                    }))])
                }]
            })),
            where_clause: None,
            items: vec![],
            docstring: None
        }))
    );
}

#[test]
fn import() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_item("import std.io as myio;", &mut diagnostics, &mut interner),
        Some(Item::Import {
            path: ImportPath {
                left: Path {
                    span: Span { start: 7, end: 13 },
                    identifiers: vec![
                        IdentifierAst {
                            span: Span { start: 7, end: 10 },
                            symbol: 19
                        },
                        IdentifierAst {
                            span: Span { start: 11, end: 13 },
                            symbol: 20
                        }
                    ]
                },
                r#as: Some(IdentifierAst {
                    span: Span { start: 17, end: 21 },
                    symbol: 21
                })
            }
        })
    );
}

#[test]
fn r#struct() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_item(
            "struct Lexer[S] where S: Iterator[char] { contents: S }",
            &mut diagnostics,
            &mut interner
        ),
        Some(Item::Struct {
            visibility: Visibility::private(),
            name: IdentifierAst {
                span: Span { start: 7, end: 12 },
                symbol: 19
            },
            generic_parameters: Some(vec![GenericParameter {
                name: IdentifierAst {
                    span: Span { start: 13, end: 14 },
                    symbol: 20
                },
                bounds: None,
                default_value: None
            }]),
            where_clause: Some(vec![WhereClauseItem::Satisfies {
                ty: Type::Path(TypePath {
                    span: Span { start: 22, end: 23 },
                    segments: vec![TypePathSegment {
                        span: Span { start: 22, end: 23 },
                        path: Path {
                            span: Span { start: 22, end: 23 },
                            identifiers: vec![IdentifierAst {
                                span: Span { start: 22, end: 23 },
                                symbol: 20
                            }]
                        },
                        generic_arguments: None
                    }]
                }),
                bounds: vec![TypePath {
                    span: Span { start: 25, end: 39 },
                    segments: vec![TypePathSegment {
                        span: Span { start: 25, end: 39 },
                        path: Path {
                            span: Span { start: 25, end: 33 },
                            identifiers: vec![IdentifierAst {
                                span: Span { start: 25, end: 33 },
                                symbol: 21
                            }]
                        },
                        generic_arguments: Some(vec![GenericArgument::Type(Type::Path(
                            TypePath {
                                span: Span { start: 34, end: 38 },
                                segments: vec![TypePathSegment {
                                    span: Span { start: 34, end: 38 },
                                    path: Path {
                                        span: Span { start: 34, end: 38 },
                                        identifiers: vec![IdentifierAst {
                                            span: Span { start: 34, end: 38 },
                                            symbol: 16
                                        }]
                                    },
                                    generic_arguments: None
                                }]
                            }
                        ))])
                    }]
                }]
            }]),
            fields: vec![StructField {
                visibility: Visibility::private(),
                name: IdentifierAst {
                    span: Span { start: 42, end: 50 },
                    symbol: 22
                },
                ty: Type::Path(TypePath {
                    span: Span { start: 52, end: 53 },
                    segments: vec![TypePathSegment {
                        span: Span { start: 52, end: 53 },
                        path: Path {
                            span: Span { start: 52, end: 53 },
                            identifiers: vec![IdentifierAst {
                                span: Span { start: 52, end: 53 },
                                symbol: 20
                            }]
                        },
                        generic_arguments: None
                    }]
                }),
                docstring: None
            }],
            docstring: None
        })
    );
}

#[test]
fn into() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_item(
            "trait Into[T] { fun into(self): T; }",
            &mut diagnostics,
            &mut interner
        ),
        Some(Item::Trait {
            visibility: Visibility::private(),
            name: IdentifierAst {
                span: Span { start: 6, end: 10 },
                symbol: 19
            },
            generic_parameters: Some(vec![GenericParameter {
                name: IdentifierAst {
                    span: Span { start: 11, end: 12 },
                    symbol: 20
                },
                bounds: None,
                default_value: None
            }]),
            where_clause: None,
            items: vec![TraitItem::AssociatedFunction(Function {
                visibility: Visibility::private(),
                name: IdentifierAst {
                    span: Span { start: 20, end: 24 },
                    symbol: 21
                },
                generic_parameters: None,
                parameters: vec![FunctionParameter::Self_(SelfParameter {
                    self_span: Span { start: 25, end: 29 },
                    ty: None
                })],
                return_type: Some(Type::Path(TypePath {
                    span: Span { start: 32, end: 33 },
                    segments: vec![TypePathSegment {
                        span: Span { start: 32, end: 33 },
                        path: Path {
                            span: Span { start: 32, end: 33 },
                            identifiers: vec![IdentifierAst {
                                span: Span { start: 32, end: 33 },
                                symbol: 20
                            }]
                        },
                        generic_arguments: None
                    }]
                })),
                where_clause: None,
                body: None,
                docstring: None
            })],
            docstring: None
        })
    );
}

#[test]
fn alias() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_item(
            "type KeyValuePair[K, V] = [HashMap[K, V] as IntoIterator].Item;",
            &mut diagnostics,
            &mut interner,
        ),
        Some(Item::TypeAlias(TypeAlias {
            visibility: Visibility::private(),
            name: IdentifierAst {
                span: Span { start: 5, end: 17 },
                symbol: 19
            },
            generic_parameters: Some(vec![
                GenericParameter {
                    name: IdentifierAst {
                        span: Span { start: 18, end: 19 },
                        symbol: 20
                    },
                    bounds: None,
                    default_value: None
                },
                GenericParameter {
                    name: IdentifierAst {
                        span: Span { start: 21, end: 22 },
                        symbol: 21
                    },
                    bounds: None,
                    default_value: None
                }
            ]),
            bounds: None,
            value: Some(Type::WithQualifiedPath {
                span: Span { start: 26, end: 62 },
                left: Box::new(Type::Path(TypePath {
                    span: Span { start: 27, end: 40 },
                    segments: vec![TypePathSegment {
                        span: Span { start: 27, end: 40 },
                        path: Path {
                            span: Span { start: 27, end: 34 },
                            identifiers: vec![IdentifierAst {
                                span: Span { start: 27, end: 34 },
                                symbol: 22
                            }]
                        },
                        generic_arguments: Some(vec![
                            GenericArgument::Type(Type::Path(TypePath {
                                span: Span { start: 35, end: 36 },
                                segments: vec![TypePathSegment {
                                    span: Span { start: 35, end: 36 },
                                    path: Path {
                                        span: Span { start: 35, end: 36 },
                                        identifiers: vec![IdentifierAst {
                                            span: Span { start: 35, end: 36 },
                                            symbol: 20
                                        }]
                                    },
                                    generic_arguments: None
                                }]
                            })),
                            GenericArgument::Type(Type::Path(TypePath {
                                span: Span { start: 38, end: 39 },
                                segments: vec![TypePathSegment {
                                    span: Span { start: 38, end: 39 },
                                    path: Path {
                                        span: Span { start: 38, end: 39 },
                                        identifiers: vec![IdentifierAst {
                                            span: Span { start: 38, end: 39 },
                                            symbol: 21
                                        }]
                                    },
                                    generic_arguments: None
                                }]
                            }))
                        ])
                    }]
                })),
                right: TypePath {
                    span: Span { start: 44, end: 56 },
                    segments: vec![TypePathSegment {
                        span: Span { start: 44, end: 56 },
                        path: Path {
                            span: Span { start: 44, end: 56 },
                            identifiers: vec![IdentifierAst {
                                span: Span { start: 44, end: 56 },
                                symbol: 23
                            }]
                        },
                        generic_arguments: None
                    }]
                },
                segments: vec![TypePathSegment {
                    span: Span { start: 58, end: 62 },
                    path: Path {
                        span: Span { start: 58, end: 62 },
                        identifiers: vec![IdentifierAst {
                            span: Span { start: 58, end: 62 },
                            symbol: 24
                        }]
                    },
                    generic_arguments: None
                }]
            }),
            docstring: None
        }))
    );
}

#[test]
fn r#enum() {
    let mut interner = Interner::default();
    let mut diagnostics = vec![];

    assert_eq!(
        parse_item(
            "enum Result[T, E] { Ok(T), Err(E) }",
            &mut diagnostics,
            &mut interner
        ),
        Some(Item::Enum {
            visibility: Visibility::private(),
            name: IdentifierAst {
                span: Span { start: 5, end: 11 },
                symbol: 19
            },
            generic_parameters: Some(vec![
                GenericParameter {
                    name: IdentifierAst {
                        span: Span { start: 12, end: 13 },
                        symbol: 20
                    },
                    bounds: None,
                    default_value: None
                },
                GenericParameter {
                    name: IdentifierAst {
                        span: Span { start: 15, end: 16 },
                        symbol: 21
                    },
                    bounds: None,
                    default_value: None
                }
            ]),
            where_clause: None,
            items: vec![
                EnumItem::Tuple {
                    name: IdentifierAst {
                        span: Span { start: 20, end: 22 },
                        symbol: 22
                    },
                    fields: vec![TupleField {
                        visibility: Visibility::private(),
                        ty: Type::Path(TypePath {
                            span: Span { start: 23, end: 24 },
                            segments: vec![TypePathSegment {
                                span: Span { start: 23, end: 24 },
                                path: Path {
                                    span: Span { start: 23, end: 24 },
                                    identifiers: vec![IdentifierAst {
                                        span: Span { start: 23, end: 24 },
                                        symbol: 20
                                    }]
                                },
                                generic_arguments: None
                            }]
                        })
                    }],
                    docstring: None
                },
                EnumItem::Tuple {
                    name: IdentifierAst {
                        span: Span { start: 27, end: 30 },
                        symbol: 23
                    },
                    fields: vec![TupleField {
                        visibility: Visibility::private(),
                        ty: Type::Path(TypePath {
                            span: Span { start: 31, end: 32 },
                            segments: vec![TypePathSegment {
                                span: Span { start: 31, end: 32 },
                                path: Path {
                                    span: Span { start: 31, end: 32 },
                                    identifiers: vec![IdentifierAst {
                                        span: Span { start: 31, end: 32 },
                                        symbol: 21
                                    }]
                                },
                                generic_arguments: None
                            }]
                        })
                    }],
                    docstring: None
                }
            ],
            docstring: None
        })
    );
}
