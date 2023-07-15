use ry_ast::{
    EnumItem, Expression, Function, FunctionParameter, FunctionSignature, GenericArgument,
    GenericParameter, IdentifierAst, Impl, ImportPath, JustFunctionParameter, ModuleItem, Path,
    SelfParameter, Statement, StructField, TraitItem, TupleField, Type, TypeAlias, TypePath,
    TypePathSegment, Visibility, WhereClauseItem,
};
use ry_diagnostics::GlobalDiagnostics;
use ry_filesystem::{location::Location, path_interner::DUMMY_PATH_ID};
use ry_interner::{symbols, Interner};
use ry_parser::parse_item;

#[test]
fn function() {
    let mut interner = Interner::default();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_item(
            DUMMY_PATH_ID,
            "fun foo[T, B = Option[T]](a: B): T { a.unwrap() }",
            &mut diagnostics,
            &mut interner
        ),
        Some(ModuleItem::Function(Function {
            signature: FunctionSignature {
                visibility: Visibility::private(),
                name: IdentifierAst {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 4,
                        end: 7
                    },
                    symbol: interner.get_or_intern("foo")
                },
                generic_parameters: Some(vec![
                    GenericParameter {
                        name: IdentifierAst {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 8,
                                end: 9
                            },
                            symbol: interner.get_or_intern("T")
                        },
                        bounds: None,
                        default_value: None
                    },
                    GenericParameter {
                        name: IdentifierAst {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 11,
                                end: 12
                            },
                            symbol: interner.get_or_intern("B")
                        },
                        bounds: None,
                        default_value: Some(Type::Path(TypePath {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 15,
                                end: 24
                            },
                            segments: vec![TypePathSegment {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 15,
                                    end: 24
                                },
                                path: Path {
                                    location: Location {
                                        file_path_id: DUMMY_PATH_ID,
                                        start: 15,
                                        end: 21
                                    },
                                    identifiers: vec![IdentifierAst {
                                        location: Location {
                                            file_path_id: DUMMY_PATH_ID,
                                            start: 15,
                                            end: 21
                                        },
                                        symbol: interner.get_or_intern("Option")
                                    }]
                                },
                                generic_arguments: Some(vec![GenericArgument::Type(Type::Path(
                                    TypePath {
                                        location: Location {
                                            file_path_id: DUMMY_PATH_ID,
                                            start: 22,
                                            end: 23
                                        },
                                        segments: vec![TypePathSegment {
                                            location: Location {
                                                file_path_id: DUMMY_PATH_ID,
                                                start: 22,
                                                end: 23
                                            },
                                            path: Path {
                                                location: Location {
                                                    file_path_id: DUMMY_PATH_ID,
                                                    start: 22,
                                                    end: 23
                                                },
                                                identifiers: vec![IdentifierAst {
                                                    location: Location {
                                                        file_path_id: DUMMY_PATH_ID,
                                                        start: 22,
                                                        end: 23
                                                    },
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
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 26,
                            end: 27
                        },
                        symbol: interner.get_or_intern("a")
                    },
                    ty: Type::Path(TypePath {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 29,
                            end: 30
                        },
                        segments: vec![TypePathSegment {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 29,
                                end: 30
                            },
                            path: Path {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 29,
                                    end: 30
                                },
                                identifiers: vec![IdentifierAst {
                                    location: Location {
                                        file_path_id: DUMMY_PATH_ID,
                                        start: 29,
                                        end: 30
                                    },
                                    symbol: interner.get_or_intern("B")
                                }]
                            },
                            generic_arguments: None
                        }]
                    })
                })],
                return_type: Some(Type::Path(TypePath {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 33,
                        end: 34
                    },
                    segments: vec![TypePathSegment {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 33,
                            end: 34
                        },
                        path: Path {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 33,
                                end: 34
                            },
                            identifiers: vec![IdentifierAst {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 33,
                                    end: 34
                                },
                                symbol: interner.get_or_intern("T")
                            }]
                        },
                        generic_arguments: None
                    }]
                })),
                where_clause: None,
                docstring: None,
            },
            body: Some(vec![Statement::Expression {
                expression: Expression::Call {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 37,
                        end: 47
                    },
                    left: Box::new(Expression::FieldAccess {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 37,
                            end: 45
                        },
                        left: Box::new(Expression::Identifier(IdentifierAst {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 37,
                                end: 38
                            },
                            symbol: interner.get_or_intern("a")
                        })),
                        right: IdentifierAst {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 39,
                                end: 45
                            },
                            symbol: interner.get_or_intern("unwrap")
                        }
                    }),
                    arguments: vec![]
                },
                has_semicolon: false
            }]),
        }))
    );
}

#[test]
fn r#impl() {
    let mut interner = Interner::default();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_item(
            DUMMY_PATH_ID,
            "impl[A, B] Into[Option[(A, B)]] for (A, B) {}",
            &mut diagnostics,
            &mut interner,
        ),
        Some(ModuleItem::Impl(Impl {
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 4
            },
            generic_parameters: Some(vec![
                GenericParameter {
                    name: IdentifierAst {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 5,
                            end: 6
                        },
                        symbol: interner.get_or_intern("A")
                    },
                    bounds: None,
                    default_value: None
                },
                GenericParameter {
                    name: IdentifierAst {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 8,
                            end: 9
                        },
                        symbol: interner.get_or_intern("B")
                    },
                    bounds: None,
                    default_value: None
                }
            ]),
            ty: Type::Tuple {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 36,
                    end: 42
                },
                element_types: vec![
                    Type::Path(TypePath {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 37,
                            end: 38
                        },
                        segments: vec![TypePathSegment {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 37,
                                end: 38
                            },
                            path: Path {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 37,
                                    end: 38
                                },
                                identifiers: vec![IdentifierAst {
                                    location: Location {
                                        file_path_id: DUMMY_PATH_ID,
                                        start: 37,
                                        end: 38
                                    },
                                    symbol: interner.get_or_intern("A")
                                }]
                            },
                            generic_arguments: None
                        }]
                    }),
                    Type::Path(TypePath {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 40,
                            end: 41
                        },
                        segments: vec![TypePathSegment {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 40,
                                end: 41
                            },
                            path: Path {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 40,
                                    end: 41
                                },
                                identifiers: vec![IdentifierAst {
                                    location: Location {
                                        file_path_id: DUMMY_PATH_ID,
                                        start: 40,
                                        end: 41
                                    },
                                    symbol: interner.get_or_intern("B")
                                }]
                            },
                            generic_arguments: None
                        }]
                    })
                ]
            },
            r#trait: Some(Type::Path(TypePath {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 11,
                    end: 31
                },
                segments: vec![TypePathSegment {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 11,
                        end: 31
                    },
                    path: Path {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 11,
                            end: 15
                        },
                        identifiers: vec![IdentifierAst {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 11,
                                end: 15
                            },
                            symbol: interner.get_or_intern("Into")
                        }]
                    },
                    generic_arguments: Some(vec![GenericArgument::Type(Type::Path(TypePath {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 16,
                            end: 30
                        },
                        segments: vec![TypePathSegment {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 16,
                                end: 30
                            },
                            path: Path {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 16,
                                    end: 22
                                },
                                identifiers: vec![IdentifierAst {
                                    location: Location {
                                        file_path_id: DUMMY_PATH_ID,
                                        start: 16,
                                        end: 22
                                    },
                                    symbol: interner.get_or_intern("Option")
                                }]
                            },
                            generic_arguments: Some(vec![GenericArgument::Type(Type::Tuple {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 23,
                                    end: 29
                                },
                                element_types: vec![
                                    Type::Path(TypePath {
                                        location: Location {
                                            file_path_id: DUMMY_PATH_ID,
                                            start: 24,
                                            end: 25
                                        },
                                        segments: vec![TypePathSegment {
                                            location: Location {
                                                file_path_id: DUMMY_PATH_ID,
                                                start: 24,
                                                end: 25
                                            },
                                            path: Path {
                                                location: Location {
                                                    file_path_id: DUMMY_PATH_ID,
                                                    start: 24,
                                                    end: 25
                                                },
                                                identifiers: vec![IdentifierAst {
                                                    location: Location {
                                                        file_path_id: DUMMY_PATH_ID,
                                                        start: 24,
                                                        end: 25
                                                    },
                                                    symbol: interner.get_or_intern("A")
                                                }]
                                            },
                                            generic_arguments: None
                                        }]
                                    }),
                                    Type::Path(TypePath {
                                        location: Location {
                                            file_path_id: DUMMY_PATH_ID,
                                            start: 27,
                                            end: 28
                                        },
                                        segments: vec![TypePathSegment {
                                            location: Location {
                                                file_path_id: DUMMY_PATH_ID,
                                                start: 27,
                                                end: 28
                                            },
                                            path: Path {
                                                location: Location {
                                                    file_path_id: DUMMY_PATH_ID,
                                                    start: 27,
                                                    end: 28
                                                },
                                                identifiers: vec![IdentifierAst {
                                                    location: Location {
                                                        file_path_id: DUMMY_PATH_ID,
                                                        start: 27,
                                                        end: 28
                                                    },
                                                    symbol: interner.get_or_intern("B")
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
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_item(
            DUMMY_PATH_ID,
            "import std.io as myio;",
            &mut diagnostics,
            &mut interner
        ),
        Some(ModuleItem::Import {
            location: Location {
                file_path_id: DUMMY_PATH_ID,
                start: 0,
                end: 22
            },
            path: ImportPath {
                path: Path {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 7,
                        end: 13
                    },
                    identifiers: vec![
                        IdentifierAst {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 7,
                                end: 10
                            },
                            symbol: symbols::STD
                        },
                        IdentifierAst {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 11,
                                end: 13
                            },
                            symbol: interner.get_or_intern("io")
                        }
                    ]
                },
                r#as: Some(IdentifierAst {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 17,
                        end: 21
                    },
                    symbol: interner.get_or_intern("myio")
                })
            }
        })
    );
}

#[test]
fn r#struct() {
    let mut interner = Interner::default();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_item(
            DUMMY_PATH_ID,
            "struct Lexer[S] where S: Iterator[char] { contents: S }",
            &mut diagnostics,
            &mut interner
        ),
        Some(ModuleItem::Struct {
            visibility: Visibility::private(),
            name: IdentifierAst {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 7,
                    end: 12
                },
                symbol: interner.get_or_intern("Lexer")
            },
            generic_parameters: Some(vec![GenericParameter {
                name: IdentifierAst {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 13,
                        end: 14
                    },
                    symbol: interner.get_or_intern("S")
                },
                bounds: None,
                default_value: None
            }]),
            where_clause: Some(vec![WhereClauseItem::Satisfies {
                ty: Type::Path(TypePath {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 22,
                        end: 23
                    },
                    segments: vec![TypePathSegment {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 22,
                            end: 23
                        },
                        path: Path {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 22,
                                end: 23
                            },
                            identifiers: vec![IdentifierAst {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 22,
                                    end: 23
                                },
                                symbol: interner.get_or_intern("S")
                            }]
                        },
                        generic_arguments: None
                    }]
                }),
                bounds: vec![TypePath {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 25,
                        end: 39
                    },
                    segments: vec![TypePathSegment {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 25,
                            end: 39
                        },
                        path: Path {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 25,
                                end: 33
                            },
                            identifiers: vec![IdentifierAst {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 25,
                                    end: 33
                                },
                                symbol: interner.get_or_intern("Iterator")
                            }]
                        },
                        generic_arguments: Some(vec![GenericArgument::Type(Type::Path(
                            TypePath {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 34,
                                    end: 38
                                },
                                segments: vec![TypePathSegment {
                                    location: Location {
                                        file_path_id: DUMMY_PATH_ID,
                                        start: 34,
                                        end: 38
                                    },
                                    path: Path {
                                        location: Location {
                                            file_path_id: DUMMY_PATH_ID,
                                            start: 34,
                                            end: 38
                                        },
                                        identifiers: vec![IdentifierAst {
                                            location: Location {
                                                file_path_id: DUMMY_PATH_ID,
                                                start: 34,
                                                end: 38
                                            },
                                            symbol: symbols::CHAR
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
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 42,
                        end: 50
                    },
                    symbol: interner.get_or_intern("contents")
                },
                ty: Type::Path(TypePath {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 52,
                        end: 53
                    },
                    segments: vec![TypePathSegment {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 52,
                            end: 53
                        },
                        path: Path {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 52,
                                end: 53
                            },
                            identifiers: vec![IdentifierAst {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 52,
                                    end: 53
                                },
                                symbol: interner.get_or_intern("S")
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
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_item(
            DUMMY_PATH_ID,
            "trait Into[T] { fun into(self): T; }",
            &mut diagnostics,
            &mut interner
        ),
        Some(ModuleItem::Trait {
            visibility: Visibility::private(),
            name: IdentifierAst {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 6,
                    end: 10
                },
                symbol: interner.get_or_intern("Into")
            },
            generic_parameters: Some(vec![GenericParameter {
                name: IdentifierAst {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 11,
                        end: 12
                    },
                    symbol: interner.get_or_intern("T")
                },
                bounds: None,
                default_value: None
            }]),
            where_clause: None,
            items: vec![TraitItem::AssociatedFunction(Function {
                signature: FunctionSignature {
                    visibility: Visibility::private(),
                    name: IdentifierAst {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 20,
                            end: 24
                        },
                        symbol: interner.get_or_intern("into")
                    },
                    generic_parameters: None,
                    parameters: vec![FunctionParameter::Self_(SelfParameter {
                        self_location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 25,
                            end: 29
                        },
                        ty: None
                    })],
                    return_type: Some(Type::Path(TypePath {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 32,
                            end: 33
                        },
                        segments: vec![TypePathSegment {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 32,
                                end: 33
                            },
                            path: Path {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 32,
                                    end: 33
                                },
                                identifiers: vec![IdentifierAst {
                                    location: Location {
                                        file_path_id: DUMMY_PATH_ID,
                                        start: 32,
                                        end: 33
                                    },
                                    symbol: interner.get_or_intern("T")
                                }]
                            },
                            generic_arguments: None
                        }]
                    })),
                    where_clause: None,
                    docstring: None
                },
                body: None,
            })],
            docstring: None,
        })
    );
}

#[test]
fn alias() {
    let mut interner = Interner::default();
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_item(
            DUMMY_PATH_ID,
            "type KeyValuePair[K, V] = [HashMap[K, V] as IntoIterator].Item;",
            &mut diagnostics,
            &mut interner,
        ),
        Some(ModuleItem::TypeAlias(TypeAlias {
            visibility: Visibility::private(),
            name: IdentifierAst {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 5,
                    end: 17
                },
                symbol: interner.get_or_intern("KeyValuePair")
            },
            generic_parameters: Some(vec![
                GenericParameter {
                    name: IdentifierAst {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 18,
                            end: 19
                        },
                        symbol: interner.get_or_intern("K")
                    },
                    bounds: None,
                    default_value: None
                },
                GenericParameter {
                    name: IdentifierAst {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 21,
                            end: 22
                        },
                        symbol: interner.get_or_intern("V")
                    },
                    bounds: None,
                    default_value: None
                }
            ]),
            bounds: None,
            value: Some(Type::WithQualifiedPath {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 26,
                    end: 62
                },
                left: Box::new(Type::Path(TypePath {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 27,
                        end: 40
                    },
                    segments: vec![TypePathSegment {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 27,
                            end: 40
                        },
                        path: Path {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 27,
                                end: 34
                            },
                            identifiers: vec![IdentifierAst {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 27,
                                    end: 34
                                },
                                symbol: interner.get_or_intern("HashMap")
                            }]
                        },
                        generic_arguments: Some(vec![
                            GenericArgument::Type(Type::Path(TypePath {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 35,
                                    end: 36
                                },
                                segments: vec![TypePathSegment {
                                    location: Location {
                                        file_path_id: DUMMY_PATH_ID,
                                        start: 35,
                                        end: 36
                                    },
                                    path: Path {
                                        location: Location {
                                            file_path_id: DUMMY_PATH_ID,
                                            start: 35,
                                            end: 36
                                        },
                                        identifiers: vec![IdentifierAst {
                                            location: Location {
                                                file_path_id: DUMMY_PATH_ID,
                                                start: 35,
                                                end: 36
                                            },
                                            symbol: interner.get_or_intern("K")
                                        }]
                                    },
                                    generic_arguments: None
                                }]
                            })),
                            GenericArgument::Type(Type::Path(TypePath {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 38,
                                    end: 39
                                },
                                segments: vec![TypePathSegment {
                                    location: Location {
                                        file_path_id: DUMMY_PATH_ID,
                                        start: 38,
                                        end: 39
                                    },
                                    path: Path {
                                        location: Location {
                                            file_path_id: DUMMY_PATH_ID,
                                            start: 38,
                                            end: 39
                                        },
                                        identifiers: vec![IdentifierAst {
                                            location: Location {
                                                file_path_id: DUMMY_PATH_ID,
                                                start: 38,
                                                end: 39
                                            },
                                            symbol: interner.get_or_intern("V")
                                        }]
                                    },
                                    generic_arguments: None
                                }]
                            }))
                        ])
                    }]
                })),
                right: TypePath {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 44,
                        end: 56
                    },
                    segments: vec![TypePathSegment {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 44,
                            end: 56
                        },
                        path: Path {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 44,
                                end: 56
                            },
                            identifiers: vec![IdentifierAst {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 44,
                                    end: 56
                                },
                                symbol: interner.get_or_intern("IntoIterator")
                            }]
                        },
                        generic_arguments: None
                    }]
                },
                segments: vec![TypePathSegment {
                    location: Location {
                        file_path_id: DUMMY_PATH_ID,
                        start: 58,
                        end: 62
                    },
                    path: Path {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 58,
                            end: 62
                        },
                        identifiers: vec![IdentifierAst {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 58,
                                end: 62
                            },
                            symbol: interner.get_or_intern("Item")
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
    let mut diagnostics = GlobalDiagnostics::new();

    assert_eq!(
        parse_item(
            DUMMY_PATH_ID,
            "enum Result[T, E] { Ok(T), Err(E) }",
            &mut diagnostics,
            &mut interner
        ),
        Some(ModuleItem::Enum {
            visibility: Visibility::private(),
            name: IdentifierAst {
                location: Location {
                    file_path_id: DUMMY_PATH_ID,
                    start: 5,
                    end: 11
                },
                symbol: interner.get_or_intern("Result")
            },
            generic_parameters: Some(vec![
                GenericParameter {
                    name: IdentifierAst {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 12,
                            end: 13
                        },
                        symbol: interner.get_or_intern("T")
                    },
                    bounds: None,
                    default_value: None
                },
                GenericParameter {
                    name: IdentifierAst {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 15,
                            end: 16
                        },
                        symbol: interner.get_or_intern("E")
                    },
                    bounds: None,
                    default_value: None
                }
            ]),
            where_clause: None,
            items: vec![
                EnumItem::TupleLike {
                    name: IdentifierAst {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 20,
                            end: 22
                        },
                        symbol: interner.get_or_intern("Ok")
                    },
                    fields: vec![TupleField {
                        visibility: Visibility::private(),
                        ty: Type::Path(TypePath {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 23,
                                end: 24
                            },
                            segments: vec![TypePathSegment {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 23,
                                    end: 24
                                },
                                path: Path {
                                    location: Location {
                                        file_path_id: DUMMY_PATH_ID,
                                        start: 23,
                                        end: 24
                                    },
                                    identifiers: vec![IdentifierAst {
                                        location: Location {
                                            file_path_id: DUMMY_PATH_ID,
                                            start: 23,
                                            end: 24
                                        },
                                        symbol: interner.get_or_intern("T")
                                    }]
                                },
                                generic_arguments: None
                            }]
                        })
                    }],
                    docstring: None
                },
                EnumItem::TupleLike {
                    name: IdentifierAst {
                        location: Location {
                            file_path_id: DUMMY_PATH_ID,
                            start: 27,
                            end: 30
                        },
                        symbol: interner.get_or_intern("Err")
                    },
                    fields: vec![TupleField {
                        visibility: Visibility::private(),
                        ty: Type::Path(TypePath {
                            location: Location {
                                file_path_id: DUMMY_PATH_ID,
                                start: 31,
                                end: 32
                            },
                            segments: vec![TypePathSegment {
                                location: Location {
                                    file_path_id: DUMMY_PATH_ID,
                                    start: 31,
                                    end: 32
                                },
                                path: Path {
                                    location: Location {
                                        file_path_id: DUMMY_PATH_ID,
                                        start: 31,
                                        end: 32
                                    },
                                    identifiers: vec![IdentifierAst {
                                        location: Location {
                                            file_path_id: DUMMY_PATH_ID,
                                            start: 31,
                                            end: 32
                                        },
                                        symbol: interner.get_or_intern("E")
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
