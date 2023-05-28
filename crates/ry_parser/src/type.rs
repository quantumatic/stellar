use crate::{
    error::{expected, ParseError, ParseResult},
    macros::parse_list,
    path::PathParser,
    OptionalParser, Parser, ParserState,
};
use ry_ast::{
    span::{At, Span, Spanned},
    token::RawToken,
    GenericParameter, Token, Type, WhereClause, WhereClauseItem,
};

#[derive(Default)]
pub(crate) struct TypeParser;

impl Parser for TypeParser {
    type Output = Spanned<Type>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let r#type = match state.next.unwrap() {
            RawToken::Identifier(..) => PrimaryTypeParser.parse_with(state)?,
            Token!['['] => ArrayTypeParser.parse_with(state)?,
            _ => {
                return Err(ParseError::unexpected_token(
                    state.next.clone(),
                    expected!("identifier", Token![&], Token!['[']),
                    "type",
                ));
            }
        };

        Ok(r#type)
    }
}

pub(crate) struct ArrayTypeParser;

impl Parser for ArrayTypeParser {
    type Output = Spanned<Type>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();
        let start = state.current.span().start();

        let element_type = TypeParser.parse_with(state)?;

        state.consume(Token![']'], "array type")?;

        Ok(Type::Array {
            element_type: Box::new(element_type),
        }
        .at(Span::new(start, state.current.span().end(), state.file_id)))
    }
}

pub(crate) struct GenericParametersParser;

impl OptionalParser for GenericParametersParser {
    type Output = Vec<GenericParameter>;

    fn optionally_parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        if *state.next.unwrap() != Token!['['] {
            return Ok(vec![]);
        }

        state.next_token();

        let result = parse_list!(
            state,
            "generics",
            Token![']'],
            || -> ParseResult<GenericParameter> {
                let name = state.consume_identifier("generic name")?;

                let constraint = if *state.next.unwrap() == Token![:] {
                    state.next_token();
                    Some(TypeParser.parse_with(state)?)
                } else {
                    None
                };

                Ok(GenericParameter { name, constraint })
            }
        );

        state.next_token();

        Ok(result)
    }
}

pub(crate) struct PrimaryTypeParser;

impl Parser for PrimaryTypeParser {
    type Output = Spanned<Type>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let start = state.next.span().start();
        let path = PathParser.parse_with(state)?;
        let generic_arguments = GenericArgumentsParser.optionally_parse_with(state)?;

        Ok(Type::Primary {
            path,
            generic_arguments,
        }
        .at(Span::new(start, state.current.span().end(), state.file_id)))
    }
}

pub(crate) struct GenericArgumentsParser;

impl OptionalParser for GenericArgumentsParser {
    type Output = Vec<Spanned<Type>>;

    fn optionally_parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        if *state.next.unwrap() != Token!['['] {
            return Ok(vec![]);
        }

        self.parse_with(state)
    }
}

impl Parser for GenericArgumentsParser {
    type Output = Vec<Spanned<Type>>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let result = parse_list!(state, "type annotations", Token![']'], || {
            TypeParser.parse_with(state)
        });

        state.next_token();

        Ok(result)
    }
}

pub(crate) struct WhereClauseParser;

impl OptionalParser for WhereClauseParser {
    type Output = WhereClause;

    fn optionally_parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        if *state.next.unwrap() != Token![where] {
            return Ok(vec![]);
        }

        state.next_token();

        Ok(parse_list!(
            state,
            "where clause",
            Token!['{'] | Token![;],
            || -> ParseResult<WhereClauseItem> {
                let r#type = TypeParser.parse_with(state)?;

                state.consume(Token![:], "where clause")?;

                let constraint = TypeParser.parse_with(state)?;

                Ok(WhereClauseItem { r#type, constraint })
            }
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::macros::parser_test;

    parser_test!(TypeParser, primary1, "i32");
    parser_test!(TypeParser, primary, "Result[T, DivisionError]");
    parser_test!(TypeParser, array, "[i32]");
}
