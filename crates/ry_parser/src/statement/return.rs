use crate::{error::ParseResult, expression::ExpressionParser, Parser, ParserState};
use ry_ast::statement::{ReturnStatement, Statement};

#[derive(Default)]
pub(crate) struct ReturnStatementParser;

impl Parser for ReturnStatementParser {
    type Output = Statement;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        Ok(ReturnStatement {
            return_value: ExpressionParser::default().parse_with(state)?,
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use crate::macros::parser_test;

    parser_test!(ReturnStatementParser, defer, "return a?.b ?: 0;");
}
