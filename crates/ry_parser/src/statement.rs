use crate::{expression::ExpressionParser, r#type::TypeParser, ParseResult, Parser, ParserState};
use ry_ast::{
    statement::*,
    token::{Keyword::*, Punctuator::*, RawToken::*},
    Mutability,
};

pub(crate) struct ReturnStatementParser;

impl Parser for ReturnStatementParser {
    type Output = ReturnStatement;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        Ok(ReturnStatement {
            return_value: ExpressionParser::default().parse_with(state)?,
        })
    }
}

pub(crate) struct DeferStatementParser;

impl Parser for DeferStatementParser {
    type Output = DeferStatement;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        Ok(DeferStatement {
            call: ExpressionParser::default().parse_with(state)?,
        })
    }
}

pub(crate) struct VarStatementParser;

impl Parser for VarStatementParser {
    type Output = VarStatement;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let mut mutability = Mutability::immutable();

        if state.next.inner == Keyword(Mut) {
            mutability = Mutability::mutable(state.current.span);
            state.advance();
        }

        let name = state.consume_identifier("variable name in var statement")?;

        let mut r#type = None;

        if state.next.inner == Punctuator(Colon) {
            state.advance();
            r#type = Some(TypeParser.parse_with(state)?);
        }

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

pub(crate) struct StatementParser;

impl Parser for StatementParser {
    type Output = (Statement, bool);

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut last_statement_in_block = false;
        let mut must_have_semicolon_at_the_end = true;

        let statement = match state.next.inner {
            Keyword(Return) => ReturnStatementParser.parse_with(state)?.into(),
            Keyword(Defer) => DeferStatementParser.parse_with(state)?.into(),
            Keyword(Var) => VarStatementParser.parse_with(state)?.into(),
            _ => {
                let expression = ExpressionParser::default().parse_with(state)?;

                must_have_semicolon_at_the_end = !expression.inner.with_block();

                match state.next.inner {
                    Punctuator(Semicolon) => {}
                    _ => {
                        if must_have_semicolon_at_the_end {
                            last_statement_in_block = true;
                        }
                    }
                }

                if last_statement_in_block || !must_have_semicolon_at_the_end {
                    ExpressionStatement {
                        has_semicolon: false,
                        expression,
                    }
                    .into()
                } else {
                    ExpressionStatement {
                        has_semicolon: true,
                        expression,
                    }
                    .into()
                }
            }
        };

        if !last_statement_in_block && must_have_semicolon_at_the_end {
            state.consume(Punctuator(Semicolon), "end of the statement")?;
        }

        Ok((statement, last_statement_in_block))
    }
}

pub(crate) struct StatementsBlockParser;

impl Parser for StatementsBlockParser {
    type Output = StatementsBlock;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.consume(Punctuator(OpenBrace), "statements block")?;

        let mut block = vec![];

        while state.next.inner != Punctuator(CloseBrace) {
            let (statement, last) = StatementParser.parse_with(state)?;
            block.push(statement);

            if last {
                break;
            }
        }

        state.consume(Punctuator(CloseBrace), "end of the statements block")?;

        Ok(block)
    }
}

// #[cfg(test)]
// mod statement_tests {
//     use crate::{macros::parser_test, Parser};
//     use ry_interner::Interner;

//     parser_test!(imut_var, "fun test() { var a = 3; }");
//     parser_test!(mut_var, "fun test() { var mut a = 3; }");
//     parser_test!(
//         defer,
//         "fun test() { var f = open(\"test\"); defer f.close(); }"
//     );
//     parser_test!(r#return, "fun test(): i32 { return 2; }");
// }
