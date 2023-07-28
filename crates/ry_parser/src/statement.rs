use ry_ast::{token::RawToken, Statement, StatementsBlock, Token};

use crate::{
    diagnostics::UnexpectedTokenDiagnostic, expected, expression::ExpressionParser,
    pattern::PatternParser, r#type::TypeParser, Parse, ParseState,
};

pub(crate) struct StatementParser;

pub(crate) struct StatementsBlockParser;

struct DeferStatementParser;

struct ReturnStatementParser;

struct LetStatementParser;

// Pattern of a token, that can appear as a beginning of some statement
// and which we can effectively jump to for error recovering.
macro_rules! possibly_first_statement_token_pattern {
    () => {
        Token![continue]
            | Token![return]
            | Token![defer]
            | Token![let]
            | possibly_first_expression_token_pattern!()
    };
}

// Pattern of a token, that can appear as a beginning of some expression
// and which we can effectively jump to for error recovering.
macro_rules! possibly_first_expression_token_pattern {
    () => {
        Token![if] | Token![match] | Token![while] | Token![loop]
    };
}

// If parsing some statement fails, to recover the error and avoid unnecessary diagnostics,
// we go to the next statement.
macro_rules! possibly_recover {
    ($state:ident, $statement:expr) => {
        if let Some(statement) = $statement {
            statement
        } else {
            loop {
                match $state.next_token.raw {
                    possibly_first_statement_token_pattern!() | RawToken::EndOfFile => break,
                    _ => $state.advance(),
                }
            }

            return None;
        }
    };
}

impl Parse for StatementParser {
    type Output = Option<(Statement, bool)>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let mut last_statement_in_block = false;
        let mut must_have_semicolon_at_the_end = true;

        let statement = match state.next_token.raw {
            Token![return] => possibly_recover!(state, ReturnStatementParser.parse(state)),
            Token![defer] => possibly_recover!(state, DeferStatementParser.parse(state)),
            Token![let] => possibly_recover!(state, LetStatementParser.parse(state)),
            Token![continue] => {
                state.advance();

                Statement::Continue {
                    location: state.current_token.location,
                }
            }
            Token![break] => {
                state.advance();

                Statement::Break {
                    location: state.current_token.location,
                }
            }
            _ => {
                let expression = possibly_recover!(state, ExpressionParser::default().parse(state));

                must_have_semicolon_at_the_end = !expression.with_block();

                match state.next_token.raw {
                    Token![;] => {}
                    Token!['}'] => {
                        if must_have_semicolon_at_the_end {
                            last_statement_in_block = true;
                        }
                    }
                    _ => {
                        state.add_diagnostic(UnexpectedTokenDiagnostic::new(
                            state.next_token,
                            expected!(";"),
                            "expression statement",
                        ));
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
                    state.add_diagnostic(UnexpectedTokenDiagnostic::new(
                        state.next_token,
                        expected!("}"),
                        "statements block",
                    ));

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

        let value = ExpressionParser::default().parse(state)?;

        Some(Statement::Let { pattern, value, ty })
    }
}
