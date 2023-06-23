use crate::{macros::parse_list, path::PathParser, Cursor, OptionalParser, Parse};
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

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        match cursor.next.raw {
            RawToken::Identifier => TypeConstructorParser.parse_with(cursor),
            Token![#] => TupleTypeParser.parse_with(cursor),
            Token!['('] => FunctionTypeParser.parse_with(cursor),
            _ => {
                cursor.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: cursor.next,
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

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = cursor.next.span.start();
        cursor.next_token(); // `#`

        cursor.consume(Token!['('], "tuple type")?;

        let element_types = parse_list!(cursor, "tuple type", Token![')'], || {
            TypeParser.parse_with(cursor)
        });

        cursor.next_token(); // `)`

        Some(TypeAst::Tuple {
            span: Span::new(start, cursor.current.span.end(), cursor.file_id),
            element_types,
        })
    }
}

impl Parse for FunctionTypeParser {
    type Output = Option<TypeAst>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = cursor.next.span.start();
        cursor.next_token(); // `(`

        let parameter_types = parse_list!(
            cursor,
            "parameter types in function type",
            Token![')'],
            || { TypeParser.parse_with(cursor) }
        );

        cursor.next_token(); // `)`

        cursor.consume(Token![:], "return type of function in the function type")?;

        let return_type = Box::new(TypeParser.parse_with(cursor)?);

        Some(TypeAst::Function {
            span: Span::new(start, cursor.current.span.end(), cursor.file_id),
            parameter_types,
            return_type,
        })
    }
}

impl OptionalParser for GenericParametersParser {
    type Output = Option<Vec<GenericParameter>>;

    fn optionally_parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        if cursor.next.raw != Token!['['] {
            return Some(vec![]);
        }

        cursor.next_token();

        let result = parse_list!(
            cursor,
            "generic parameters",
            Token![']'],
            || -> Option<GenericParameter> {
                Some(GenericParameter {
                    name: cursor.consume_identifier("generic parameter name")?,
                    constraint: if cursor.next.raw == Token![:] {
                        cursor.next_token();

                        Some(TypeParser.parse_with(cursor)?)
                    } else {
                        None
                    },
                    default_value: if cursor.next.raw == Token![=] {
                        cursor.next_token();

                        Some(TypeParser.parse_with(cursor)?)
                    } else {
                        None
                    },
                })
            }
        );

        cursor.next_token();

        Some(result)
    }
}

impl Parse for TypeConstructorParser {
    type Output = Option<TypeAst>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let path = PathParser.parse_with(cursor)?;
        let generic_arguments = GenericArgumentsParser.optionally_parse_with(cursor)?;

        Some(TypeAst::Constructor {
            span: Span::new(path.span.start(), cursor.current.span.end(), cursor.file_id),
            path,
            generic_arguments,
        })
    }
}

impl OptionalParser for GenericArgumentsParser {
    type Output = Option<Vec<TypeAst>>;

    fn optionally_parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        if cursor.next.raw != Token!['['] {
            return Some(vec![]);
        }

        self.parse_with(cursor)
    }
}

impl Parse for GenericArgumentsParser {
    type Output = Option<Vec<TypeAst>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();

        let result = parse_list!(cursor, "generic arguments", Token![']'], || {
            TypeParser.parse_with(cursor)
        });

        cursor.next_token();

        Some(result)
    }
}

impl OptionalParser for WhereClauseParser {
    type Output = Option<WhereClause>;

    fn optionally_parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        if cursor.next.raw != Token![where] {
            return Some(vec![]);
        }

        cursor.next_token();

        Some(parse_list!(
            cursor,
            "where clause",
            (Token!['{']) or (Token![;]),
            || -> Option<WhereClauseItem> {
                let r#type = TypeParser.parse_with(cursor)?;

                cursor.consume(Token![:], "where clause")?;

                let constraint = TypeParser.parse_with(cursor)?;

                Some(WhereClauseItem { r#type, constraint })
            }
        ))
    }
}
