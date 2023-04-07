use crate::{error::ParseResult, expression::ExpressionParser, Parser, ParserState};
use ry_ast::statement::{DeferStatement, Statement};

#[derive(Default)]
pub(crate) struct DeferStatementParser;

impl Parser for DeferStatementParser {
    type Output = Statement;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        Ok(DeferStatement {
            call: ExpressionParser::default().parse_with(state)?,
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use crate::macros::parser_test;

    parser_test!(DeferStatementParser, defer, "defer call();");
}
