use crate::{
    diagnostics::ParseDiagnostic, expected, expression::ExpressionParser, pattern::PatternParser,
    r#type::TypeParser, Parse, ParseState,
};
use ry_ast::{token::RawToken, Statement, StatementsBlock, Token};
use ry_diagnostics::BuildDiagnostic;

struct StatementParser;

pub(crate) struct StatementsBlockParser;

struct DeferStatementParser;

struct ReturnStatementParser;

struct LetStatementParser;

impl Parse for StatementParser {
    type Output = Option<(Statement, bool)>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let mut last_statement_in_block = false;
        let mut must_have_semicolon_at_the_end = true;

        let statement = match state.next_token.raw {
            Token![return] => ReturnStatementParser.parse(state)?,
            Token![defer] => DeferStatementParser.parse(state)?,
            Token![let] => LetStatementParser.parse(state)?,
            Token![continue] => {
                state.advance();

                Statement::Continue {
                    span: state.current_token.span,
                }
            }
            Token![break] => {
                state.advance();

                Statement::Break {
                    span: state.current_token.span,
                }
            }
            _ => {
                let expression = ExpressionParser::default().parse(state)?;

                must_have_semicolon_at_the_end = !expression.with_block();

                match state.next_token.raw {
                    Token![;] => {}
                    Token!['}'] => {
                        if must_have_semicolon_at_the_end {
                            last_statement_in_block = true;
                        }
                    }
                    _ => {
                        state.diagnostics.push(
                            ParseDiagnostic::UnexpectedTokenError {
                                got: state.next_token,
                                expected: expected!(";"),
                                node: "expression statement".to_owned(),
                            }
                            .build(),
                        );
                        return None;
                    }
                }

                if last_statement_in_block || !must_have_semicolon_at_the_end {
                    Statement::Expression {
                        has_semicolon: false,
                        expression,
                    }
                } else {
                    Statement::Expression {
                        has_semicolon: true,
                        expression,
                    }
                }
            }
        };

        if !last_statement_in_block && must_have_semicolon_at_the_end {
            state.consume(Token![;], "statement")?;
        }

        Some((statement, last_statement_in_block))
    }
}

impl Parse for StatementsBlockParser {
    type Output = Option<StatementsBlock>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.consume(Token!['{'], "statements block")?;

        let mut block = vec![];

        loop {
            match state.next_token.raw {
                Token!['}'] => break,
                RawToken::EndOfFile => {
                    state.diagnostics.push(
                        ParseDiagnostic::UnexpectedTokenError {
                            got: state.next_token,
                            expected: expected!("}"),
                            node: "statements block".to_owned(),
                        }
                        .build(),
                    );

                    return None;
                }
                Token![;] => {
                    // Skip
                    state.advance();

                    continue;
                }
                _ => {}
            }

            let (statement, last) = StatementParser.parse(state)?;
            block.push(statement);

            if last {
                break;
            }
        }

        state.advance();

        Some(block)
    }
}

impl Parse for DeferStatementParser {
    type Output = Option<Statement>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        Some(Statement::Defer {
            call: ExpressionParser::default().parse(state)?,
        })
    }
}

impl Parse for ReturnStatementParser {
    type Output = Option<Statement>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        Some(Statement::Return {
            expression: ExpressionParser::default().parse(state)?,
        })
    }
}

impl Parse for LetStatementParser {
    type Output = Option<Statement>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance(); // `let`

        let pattern = PatternParser.parse(state)?;

        let ty = if state.next_token.raw == Token![:] {
            state.advance();
            Some(TypeParser.parse(state)?)
        } else {
            None
        };

        state.consume(Token![=], "let statement")?;

        let value = Box::new(ExpressionParser::default().parse(state)?);

        Some(Statement::Let { pattern, value, ty })
    }
}
