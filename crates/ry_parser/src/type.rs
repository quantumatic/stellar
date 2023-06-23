use crate::{macros::parse_list, path::PathParser, OptionalParser, Parse, TokenIterator};
use ry_ast::{token::RawToken, GenericParameter, Token, TypeAst, WhereClause, WhereClauseItem};
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
        match iterator.next.raw {
            RawToken::Identifier => TypeConstructorParser.parse_using(iterator),
            Token![#] => TupleTypeParser.parse_using(iterator),
            Token!['('] => FunctionTypeParser.parse_using(iterator),
            _ => {
                iterator.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: iterator.next,
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
        let start = iterator.next.span.start();
        iterator.next_token(); // `#`

        iterator.consume(Token!['('], "tuple type")?;

        let element_types = parse_list!(iterator, "tuple type", Token![')'], || {
            TypeParser.parse_using(iterator)
        });

        iterator.next_token(); // `)`

        Some(TypeAst::Tuple {
            span: Span::new(start, iterator.current.span.end(), iterator.file_id),
            element_types,
        })
    }
}

impl Parse for FunctionTypeParser {
    type Output = Option<TypeAst>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next.span.start();
        iterator.next_token(); // `(`

        let parameter_types = parse_list!(
            iterator,
            "parameter types in function type",
            Token![')'],
            || { TypeParser.parse_using(iterator) }
        );

        iterator.next_token(); // `)`

        iterator.consume(Token![:], "return type of function in the function type")?;

        let return_type = Box::new(TypeParser.parse_using(iterator)?);

        Some(TypeAst::Function {
            span: Span::new(start, iterator.current.span.end(), iterator.file_id),
            parameter_types,
            return_type,
        })
    }
}

impl OptionalParser for GenericParametersParser {
    type Output = Option<Vec<GenericParameter>>;

    fn optionally_parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        if iterator.next.raw != Token!['['] {
            return Some(vec![]);
        }

        iterator.next_token();

        let result = parse_list!(
            iterator,
            "generic parameters",
            Token![']'],
            || -> Option<GenericParameter> {
                Some(GenericParameter {
                    name: iterator.consume_identifier("generic parameter name")?,
                    constraint: if iterator.next.raw == Token![:] {
                        iterator.next_token();

                        Some(TypeParser.parse_using(iterator)?)
                    } else {
                        None
                    },
                    default_value: if iterator.next.raw == Token![=] {
                        iterator.next_token();

                        Some(TypeParser.parse_using(iterator)?)
                    } else {
                        None
                    },
                })
            }
        );

        iterator.next_token();

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
                iterator.current.span.end(),
                iterator.file_id,
            ),
            path,
            generic_arguments,
        })
    }
}

impl OptionalParser for GenericArgumentsParser {
    type Output = Option<Vec<TypeAst>>;

    fn optionally_parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        if iterator.next.raw != Token!['['] {
            return Some(vec![]);
        }

        self.parse_using(iterator)
    }
}

impl Parse for GenericArgumentsParser {
    type Output = Option<Vec<TypeAst>>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token();

        let result = parse_list!(iterator, "generic arguments", Token![']'], || {
            TypeParser.parse_using(iterator)
        });

        iterator.next_token();

        Some(result)
    }
}

impl OptionalParser for WhereClauseParser {
    type Output = Option<WhereClause>;

    fn optionally_parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        if iterator.next.raw != Token![where] {
            return Some(vec![]);
        }

        iterator.next_token();

        Some(parse_list!(
            iterator,
            "where clause",
            (Token!['{']) or (Token![;]),
            || -> Option<WhereClauseItem> {
                let r#type = TypeParser.parse_using(iterator)?;

                iterator.consume(Token![:], "where clause")?;

                let constraint = TypeParser.parse_using(iterator)?;

                Some(WhereClauseItem { r#type, constraint })
            }
        ))
    }
}
