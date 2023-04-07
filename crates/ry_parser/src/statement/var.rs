use crate::{
    error::ParseResult, expression::ExpressionParser, r#type::TypeParser, Parser, ParserState,
};
use ry_ast::{
    statement::{Statement, VarStatement},
    token::{
        Keyword::Mut,
        Punctuator::{Assign, Colon},
        RawToken::{Keyword, Punctuator},
    },
    Mutability,
};

#[derive(Default)]
pub(crate) struct VarStatementParser;

impl Parser for VarStatementParser {
    type Output = Statement;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let mut mutability = Mutability::immutable();

        if state.next.inner == Keyword(Mut) {
            mutability = Mutability::mutable(state.current.span);
            state.advance();
        }

        let name = state.consume_identifier("variable name in var statement")?;

        let r#type = if state.next.inner == Punctuator(Colon) {
            state.advance();
            Some(TypeParser.parse_with(state)?)
        } else {
            None
        };

        state.consume(Punctuator(Assign), "var statement")?;

        Ok(VarStatement {
            mutability,
            name,
            r#type,
            value: ExpressionParser::default().parse_with(state)?,
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::VarStatementParser;
    use crate::{macros::parser_test, Parser, ParserState};
    use ry_interner::Interner;

    parser_test!(VarStatementParser, imut_var, "var a = 3;");
    parser_test!(VarStatementParser, mut_var, "var b: i32 = 3;");
}
