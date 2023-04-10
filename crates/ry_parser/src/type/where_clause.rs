use super::TypeParser;
use crate::{error::ParseResult, macros::parse_list, OptionalParser, Parser, ParserState};
use ry_ast::{
    r#type::{WhereClause, WhereClauseUnit},
    token::{
        Keyword::Where,
        Punctuator::{Colon, OpenBrace, Semicolon},
        RawToken::{Keyword, Punctuator},
    },
};

pub(crate) struct WhereClauseParser;

impl OptionalParser for WhereClauseParser {
    type Output = WhereClause;

    fn optionally_parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        if state.next.inner != Keyword(Where) {
            return Ok(vec![]);
        }

        state.next_token();

        Ok(parse_list!(
            state,
            "where clause",
            Punctuator(OpenBrace | Semicolon),
            || -> ParseResult<WhereClauseUnit> {
                let r#type = TypeParser.parse_with(state)?;

                state.consume(Punctuator(Colon), "where clause")?;

                let constraint = TypeParser.parse_with(state)?;

                Ok(WhereClauseUnit { r#type, constraint })
            }
        ))
    }
}
