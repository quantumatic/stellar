use super::TypeParser;
use crate::{error::ParseResult, macros::parse_list, OptionalParser, Parser, ParserState};
use ry_ast::{
    r#type::{WhereClause, WhereClauseUnit},
    Token,
};

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
            || -> ParseResult<WhereClauseUnit> {
                let r#type = TypeParser.parse_with(state)?;

                state.consume(Token![:], "where clause")?;

                let constraint = TypeParser.parse_with(state)?;

                Ok(WhereClauseUnit { r#type, constraint })
            }
        ))
    }
}
