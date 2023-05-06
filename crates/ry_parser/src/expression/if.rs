use super::ExpressionParser;
use crate::{error::ParseResult, statement::StatementsBlockParser, Parser, ParserState};
use ry_ast::{
    expression::{Expression, IfBlock, IfExpression, RawExpression},
    span::{At, Span},
    Token,
};

pub(crate) struct IfExpressionParser;

impl Parser for IfExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let start = state.current.span().start();

        let mut if_blocks = vec![IfBlock {
            condition: ExpressionParser::default().parse_with(state)?,
            body: StatementsBlockParser.parse_with(state)?,
        }];

        let mut r#else = None;

        while *state.next.unwrap() == Token![else] {
            state.next_token();

            match state.next.unwrap() {
                Token![if] => {}
                _ => {
                    r#else = Some(StatementsBlockParser.parse_with(state)?);
                    break;
                }
            }

            state.next_token();

            if_blocks.push(IfBlock {
                condition: ExpressionParser::default().parse_with(state)?,
                body: StatementsBlockParser.parse_with(state)?,
            });
        }

        Ok(
            RawExpression::from(IfExpression { if_blocks, r#else }).at(Span::new(
                start,
                state.current.span().end(),
                state.file_id,
            )),
        )
    }
}
