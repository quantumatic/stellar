use crate::{
    error::ParseResult, expression::ExpressionParser, r#type::TypeParser, Parser, ParserState,
};
use ry_ast::{
    statement::{Statement, VarStatement},
    Token,
};

#[derive(Default)]
pub(crate) struct VarStatementParser;

impl Parser for VarStatementParser {
    type Output = Statement;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let name = state.consume_identifier("variable name in var statement")?;

        let r#type = if *state.next.unwrap() == Token![:] {
            state.next_token();
            Some(TypeParser.parse_with(state)?)
        } else {
            None
        };

        state.consume(Token![=], "var statement")?;

        Ok(VarStatement {
            name,
            r#type,
            value: ExpressionParser::default().parse_with(state)?,
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use crate::macros::parser_test;

    parser_test!(VarStatementParser, var_without_type, "var a = 3;");
    parser_test!(VarStatementParser, var_with_type, "var b: i32 = 3;");
}
