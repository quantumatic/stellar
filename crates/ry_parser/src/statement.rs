use ry_ast::{
    token::{Keyword, Punctuator, RawToken},
    Statement,
};

use crate::{
    diagnostics::UnexpectedTokenDiagnostic, expected, expression::ExpressionParser,
    pattern::PatternParser, r#type::TypeParser, Parse, ParseState,
};

// Pattern of a token, that can appear as a beginning of some statement
// and which we can effectively jump to for error recovering.
macro_rules! possibly_first_statement_token_pattern {
    () => {
        RawToken::Keyword(Keyword::Continue | Keyword::Return | Keyword::Defer | Keyword::Let)
            | possibly_first_expression_token_pattern!()
    };
}

// Pattern of a token, that can appear as a beginning of some expression
// and which we can effectively jump to for error recovering.
macro_rules! possibly_first_expression_token_pattern {
    () => {
        RawToken::Keyword(Keyword::If | Keyword::Match | Keyword::While | Keyword::Loop)
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

pub(crate) struct StatementParser;

impl Parse for StatementParser {
    type Output = Option<(Statement, bool)>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let mut last_statement_in_block = false;
        let mut must_have_semicolon_at_the_end = true;

        let statement = match state.next_token.raw {
            RawToken::Keyword(Keyword::Return) => {
                possibly_recover!(state, ReturnStatementParser.parse(state))
            }
            RawToken::Keyword(Keyword::Defer) => {
                possibly_recover!(state, DeferStatementParser.parse(state))
            }
            RawToken::Keyword(Keyword::Let) => {
                possibly_recover!(state, LetStatementParser.parse(state))
            }
            RawToken::Keyword(Keyword::Continue) => {
                state.advance();

                Statement::Continue {
                    location: state.current_token.location,
                }
            }
            RawToken::Keyword(Keyword::Break) => {
                state.advance();

                Statement::Break {
                    location: state.current_token.location,
                }
            }
            _ => {
                let expression = ExpressionParser::default().parse(state);

                if let Some(expression) = expression {
                    must_have_semicolon_at_the_end = !expression.with_block();

                    match state.current_token.raw {
                        RawToken::Punctuator(Punctuator::Semicolon) => {}
                        RawToken::Punctuator(Punctuator::CloseBrace) => {
                            if must_have_semicolon_at_the_end {
                                last_statement_in_block = true;
                            }
                        }
                        _ => {
                            state.add_diagnostic(UnexpectedTokenDiagnostic::new(
                                Some(state.current_token.location.end),
                                state.next_token,
                                expected!(Punctuator::Semicolon),
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
                } else {
                    possibly_recover!(state, None);

                    return None;
                }
            }
        };

        if !last_statement_in_block && must_have_semicolon_at_the_end {
            state.consume(Punctuator::Semicolon, "statement")?;
        }

        Some((statement, last_statement_in_block))
    }
}

pub(crate) struct StatementsBlockParser;

impl Parse for StatementsBlockParser {
    type Output = Option<Vec<Statement>>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.consume(Punctuator::OpenBrace, "statements block")?;

        let mut block = vec![];

        loop {
            match state.next_token.raw {
                RawToken::Punctuator(Punctuator::CloseBrace) => break,
                RawToken::EndOfFile => {
                    state.add_diagnostic(UnexpectedTokenDiagnostic::new(
                        Some(state.current_token.location.end),
                        state.next_token,
                        expected!(Punctuator::CloseBrace),
                        "statements block",
                    ));

                    return None;
                }
                RawToken::Punctuator(Punctuator::Semicolon) => {
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

struct DeferStatementParser;

impl Parse for DeferStatementParser {
    type Output = Option<Statement>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        Some(Statement::Defer {
            call: ExpressionParser::default().parse(state)?,
        })
    }
}

struct ReturnStatementParser;

impl Parse for ReturnStatementParser {
    type Output = Option<Statement>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        Some(Statement::Return {
            expression: ExpressionParser::default().parse(state)?,
        })
    }
}

struct LetStatementParser;

impl Parse for LetStatementParser {
    type Output = Option<Statement>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let pattern = PatternParser.parse(state)?;

        let ty = if state.next_token.raw == Punctuator::Colon {
            state.advance();

            Some(TypeParser.parse(state)?)
        } else {
            None
        };

        state.consume(Punctuator::Eq, "let statement")?;

        let value = ExpressionParser::default().parse(state)?;

        Some(Statement::Let { pattern, value, ty })
    }
}
