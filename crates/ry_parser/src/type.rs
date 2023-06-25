use crate::{macros::parse_list, path::PathParser, OptionalParser, Parse, TokenIterator};
use ry_ast::{
    token::RawToken, GenericArgument, GenericParameter, Path, Token, TypeAst, TypeBounds, TypePath,
    TypePathSegment, WhereClause, WhereClauseItem,
};
use ry_diagnostics::{expected, parser::ParseDiagnostic, BuildDiagnostic};
use ry_source_file::span::Span;

pub(crate) struct TypeBoundsParser;

pub(crate) struct TypeParser;

struct TypeWithQualifiedPathParser;

struct TraitObjectTypeParser;

struct ParenthesizedTypeParser;

struct TypePathParser;

struct TypePathSegmentParser;

struct TupleTypeParser;

struct FunctionTypeParser;

pub(crate) struct GenericParametersParser;

pub(crate) struct GenericArgumentsParser;

pub(crate) struct WhereClauseParser;

impl Parse for TypeBoundsParser {
    type Output = Option<TypeBounds>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let mut bounds = vec![];
        bounds.push(TypePathParser.parse_using(iterator)?);

        while iterator.next_token.raw == Token![+] {
            iterator.advance();

            bounds.push(TypePathParser.parse_using(iterator)?);
        }

        Some(TypeBounds { bounds })
    }
}

impl Parse for TypeParser {
    type Output = Option<TypeAst>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        match iterator.next_token.raw {
            Token!['('] => ParenthesizedTypeParser.parse_using(iterator),
            RawToken::Identifier => TypePathParser.parse_using(iterator).map(TypeAst::Path),
            Token![#] => TupleTypeParser.parse_using(iterator),
            Token![Fun] => FunctionTypeParser.parse_using(iterator),
            Token![dyn] => TraitObjectTypeParser.parse_using(iterator),
            Token!['['] => TypeWithQualifiedPathParser.parse_using(iterator),
            _ => {
                iterator.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: iterator.next_token,
                        expected: expected!("identifier", Token!['['], Token![#], Token!['(']),
                        node: "type".to_owned(),
                    }
                    .build(),
                );

                None
            }
        }
    }
}

impl Parse for TypeWithQualifiedPathParser {
    type Output = Option<TypeAst>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next_token.span.start();
        iterator.advance(); // `[`

        let left = Box::new(TypeParser.parse_using(iterator)?);
        iterator.consume(Token![as], "type with qualified path")?;

        let right = TypePathParser.parse_using(iterator)?;

        iterator.consume(Token![']'], "type with qualified path")?;
        iterator.consume(Token![.], "type with qualified path")?;

        let mut segments = vec![TypePathSegmentParser.parse_using(iterator)?];

        while iterator.next_token.raw == Token![.] {
            iterator.advance();

            segments.push(TypePathSegmentParser.parse_using(iterator)?);
        }

        Some(TypeAst::WithQualifiedPath {
            span: Span::new(start, iterator.current_token.span.end(), iterator.file_id),
            left,
            right,
            segments,
        })
    }
}

impl Parse for TraitObjectTypeParser {
    type Output = Option<TypeAst>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next_token.span.start();

        iterator.advance(); // `dyn`

        Some(TypeAst::TraitObject {
            bounds: TypeBoundsParser.parse_using(iterator)?,
            span: Span::new(start, iterator.current_token.span.end(), iterator.file_id),
        })
    }
}

impl Parse for TupleTypeParser {
    type Output = Option<TypeAst>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next_token.span.start();
        iterator.advance(); // `#`

        iterator.consume(Token!['('], "tuple type")?;

        let element_types = parse_list!(iterator, "tuple type", Token![')'], {
            TypeParser.parse_using(iterator)
        });

        iterator.advance(); // `)`

        Some(TypeAst::Tuple {
            span: Span::new(start, iterator.current_token.span.end(), iterator.file_id),
            element_types,
        })
    }
}

impl Parse for ParenthesizedTypeParser {
    type Output = Option<TypeAst>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next_token.span.start();
        iterator.advance(); // `(`

        let inner = Box::new(TypeParser.parse_using(iterator)?);

        iterator.consume(Token![')'], "parenthesized type")?;

        Some(TypeAst::Parenthesized {
            span: Span::new(start, iterator.current_token.span.end(), iterator.file_id),
            inner,
        })
    }
}

impl Parse for FunctionTypeParser {
    type Output = Option<TypeAst>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next_token.span.start();
        iterator.advance(); // `Fun`

        iterator.consume(Token!['('], "function type")?;

        let parameter_types =
            parse_list!(iterator, "parameter types in function type", Token![')'], {
                TypeParser.parse_using(iterator)
            });

        iterator.advance(); // `)`

        iterator.consume(Token![:], "return type of function in the function type")?;

        let return_type = Box::new(TypeParser.parse_using(iterator)?);

        Some(TypeAst::Function {
            span: Span::new(start, iterator.current_token.span.end(), iterator.file_id),
            parameter_types,
            return_type,
        })
    }
}

impl OptionalParser for GenericParametersParser {
    type Output = Option<Option<Vec<GenericParameter>>>;

    fn optionally_parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        if iterator.next_token.raw != Token!['['] {
            return Some(None);
        }

        iterator.advance();

        let result = parse_list!(iterator, "generic parameters", Token![']'], {
            Some(GenericParameter {
                name: iterator.consume_identifier("generic parameter name")?,
                bounds: if iterator.next_token.raw == Token![:] {
                    iterator.advance();

                    Some(TypeBoundsParser.parse_using(iterator)?)
                } else {
                    None
                },
                default_value: if iterator.next_token.raw == Token![=] {
                    iterator.advance();

                    Some(TypeParser.parse_using(iterator)?)
                } else {
                    None
                },
            })
        });

        iterator.advance();

        Some(Some(result))
    }
}

impl Parse for TypePathParser {
    type Output = Option<TypePath>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next_token.span.start();

        let mut segments = vec![];
        segments.push(TypePathSegmentParser.parse_using(iterator)?);

        while iterator.next_token.raw == Token![.] {
            iterator.advance();

            segments.push(TypePathSegmentParser.parse_using(iterator)?);
        }

        Some(TypePath {
            span: Span::new(start, iterator.current_token.span.end(), iterator.file_id),
            segments,
        })
    }
}

impl Parse for TypePathSegmentParser {
    type Output = Option<TypePathSegment>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let path = PathParser.parse_using(iterator)?;
        let generic_arguments = GenericArgumentsParser.optionally_parse_using(iterator)?;

        Some(TypePathSegment {
            span: Span::new(
                path.span.start(),
                iterator.current_token.span.end(),
                iterator.file_id,
            ),
            path,
            generic_arguments,
        })
    }
}

impl OptionalParser for GenericArgumentsParser {
    type Output = Option<Option<Vec<GenericArgument>>>;

    fn optionally_parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        if iterator.next_token.raw != Token!['['] {
            return Some(None);
        }

        Some(self.parse_using(iterator))
    }
}
impl Parse for GenericArgumentsParser {
    type Output = Option<Vec<GenericArgument>>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.advance();

        let result = parse_list!(iterator, "generic arguments", Token![']'], {
            let ty = TypeParser.parse_using(iterator)?;

            match (iterator.next_token.raw, &ty) {
                (Token![=], TypeAst::Path(TypePath { segments, .. })) => {
                    match segments.as_slice() {
                        [TypePathSegment {
                            path: Path { identifiers, .. },
                            generic_arguments: None,
                            ..
                        }] if identifiers.len() == 1 => {
                            iterator.advance();
                            let value = TypeParser.parse_using(iterator)?;
                            Some(GenericArgument::AssociatedType {
                                name: *identifiers
                                    .first()
                                    .expect("Cannot get first identifier of type path"),
                                value,
                            })
                        }
                        _ => None,
                    }
                }
                _ => Some(GenericArgument::Type(ty)),
            }
        });

        iterator.advance();

        Some(result)
    }
}

impl OptionalParser for WhereClauseParser {
    type Output = Option<Option<WhereClause>>;

    fn optionally_parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        if iterator.next_token.raw != Token![where] {
            return Some(None);
        }

        iterator.advance();

        Some(Some(parse_list!(
            iterator,
            "where clause",
            (Token!['{']) or (Token![;]),
             {
                let left = TypeParser.parse_using(iterator)?;

                match iterator.next_token.raw {
                    Token![:] => {
                        iterator.advance();

                        Some(WhereClauseItem::Satisfies { left, right: TypeBoundsParser.parse_using(iterator)? })
                    },
                    Token![=] => {
                        iterator.advance();

                        Some(WhereClauseItem::Eq { left, right: TypeParser.parse_using(iterator)? })
                    },
                    _ => {
                        iterator.diagnostics.push(ParseDiagnostic::UnexpectedTokenError {
                            got: iterator.next_token,
                            expected: expected!(Token![=], Token![:]),
                            node: "where clause".to_owned(),
                        }.build());

                        None
                    }
                }
            }
        )))
    }
}
