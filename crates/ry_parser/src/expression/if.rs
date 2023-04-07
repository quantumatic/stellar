use super::ExpressionParser;
use crate::{error::ParseResult, statement::StatementsBlockParser, Parser, ParserState};
use ry_ast::{
    expression::{Expression, IfBlock, IfExpression, RawExpression},
    span::At,
    token::{
        Keyword::{Else, If},
        RawToken::Keyword,
    },
};

pub(crate) struct IfExpressionParser;

impl Parser for IfExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let start = state.current.span.start;

        let mut if_blocks = vec![IfBlock {
            condition: ExpressionParser::default().parse_with(state)?,
            body: StatementsBlockParser.parse_with(state)?,
        }];

        let mut r#else = None;

        while state.next.inner == Keyword(Else) {
            state.advance();

            match state.next.inner {
                Keyword(If) => {}
                _ => {
                    r#else = Some(StatementsBlockParser.parse_with(state)?);
                    break;
                }
            }

            state.advance();

            if_blocks.push(IfBlock {
                condition: ExpressionParser::default().parse_with(state)?,
                body: StatementsBlockParser.parse_with(state)?,
            });
        }

        let end = state.current.span.end;

        Ok(RawExpression::from(IfExpression { if_blocks, r#else }).at(start..end))
    }
}
