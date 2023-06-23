use crate::{macros::parse_list, path::PathParser, OptionalParser, Parse, TokenIterator};
use ry_ast::{
    token::RawToken, GenericArgument, GenericParameter, IdentifierAst, Token, TypeAst, WhereClause,
    WhereClauseItem,
};
use ry_diagnostics::{expected, parser::ParseDiagnostic, Report};
use ry_source_file::span::Span;

pub(crate) struct TypeParser;

struct TypeConstructorParser;

struct TupleTypeParser;

struct FunctionTypeParser;

pub(crate) struct GenericParametersParser;

pub(crate) struct GenericArgumentsParser;

pub(crate) struct WhereClauseParser;

impl Parse for TypeParser {
    type Output = Option<TypeAst>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        match iterator.next_token.raw {
            RawToken::Identifier => TypeConstructorParser.parse_using(iterator),
            Token![#] => TupleTypeParser.parse_using(iterator),
            Token!['('] => FunctionTypeParser.parse_using(iterator),
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

impl Parse for FunctionTypeParser {
    type Output = Option<TypeAst>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next_token.span.start();
        iterator.advance(); // `(`

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
    type Output = Option<Vec<GenericParameter>>;

    fn optionally_parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        if iterator.next_token.raw != Token!['['] {
            return Some(vec![]);
        }

        iterator.advance();

        let result = parse_list!(iterator, "generic parameters", Token![']'], {
            Some(GenericParameter {
                name: iterator.consume_identifier("generic parameter name")?,
                constraint: if iterator.next_token.raw == Token![:] {
                    iterator.advance();

                    Some(TypeParser.parse_using(iterator)?)
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

        Some(result)
    }
}

impl Parse for TypeConstructorParser {
    type Output = Option<TypeAst>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let path = PathParser.parse_using(iterator)?;
        let generic_arguments = GenericArgumentsParser.optionally_parse_using(iterator)?;

        Some(TypeAst::Constructor {
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
    type Output = Option<Vec<GenericArgument>>;

    fn optionally_parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        if iterator.next_token.raw != Token!['['] {
            return Some(vec![]);
        }

        self.parse_using(iterator)
    }
}
impl Parse for GenericArgumentsParser {
    type Output = Option<Vec<GenericArgument>>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.advance();

        let result = parse_list!(iterator, "generic arguments", Token![']'], {
            let ty = TypeParser.parse_using(iterator)?;

            match (iterator.next_token.raw, &ty) {
                (Token![=], TypeAst::Constructor { span, path, .. }) if *span == path.span => {
                    match path.symbols.as_slice() {
                        [IdentifierAst { symbol, .. }] => {
                            iterator.advance();
                            let value = TypeParser.parse_using(iterator)?;
                            Some(GenericArgument::AssociatedType {
                                name: IdentifierAst {
                                    span: *span,
                                    symbol: *symbol,
                                },
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
    type Output = Option<WhereClause>;

    fn optionally_parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        if iterator.next_token.raw != Token![where] {
            return Some(vec![]);
        }

        iterator.advance();

        Some(parse_list!(
            iterator,
            "where clause",
            (Token!['{']) or (Token![;]),
             {
                let r#type = TypeParser.parse_using(iterator)?;

                iterator.consume(Token![:], "where clause")?;

                let constraint = TypeParser.parse_using(iterator)?;

                Some(WhereClauseItem { r#type, constraint })
            }
        ))
    }
}
