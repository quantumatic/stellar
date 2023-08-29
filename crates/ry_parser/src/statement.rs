use ry_ast::{
    token::{Keyword, Punctuator, RawToken},
    Statement,
};

use crate::{
    diagnostics::UnexpectedToken, expression::ExpressionParser, pattern::PatternParser,
    r#type::TypeParser, Parse, ParseState,
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

pub(crate) struct StatementParserResult {
    pub(crate) statement: Statement,
    pub(crate) last_expression_in_block: bool,
}

impl Parse for StatementParser {
    type Output = Option<StatementParserResult>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let (statement, last_expression_in_block) = match state.next_token.raw {
            RawToken::Keyword(Keyword::Return) => (
                possibly_recover!(state, ReturnStatementParser.parse(state)),
                false,
            ),
            RawToken::Keyword(Keyword::Defer) => (
                possibly_recover!(state, DeferStatementParser.parse(state)),
                false,
            ),
            RawToken::Keyword(Keyword::Let) => (
                possibly_recover!(state, LetStatementParser.parse(state)),
                false,
            ),
            RawToken::Keyword(Keyword::Continue) => (
                possibly_recover!(state, ContinueStatementParser.parse(state)),
                false,
            ),
            RawToken::Keyword(Keyword::Break) => (
                possibly_recover!(state, BreakStatementParser.parse(state)),
                false,
            ),
            _ => {
                let expression_statement_parser_result = ExpressionStatementParser.parse(state);

                possibly_recover!(
                    state,
                    expression_statement_parser_result
                        .map(|r| (r.expression_statement, r.last_expression_in_block))
                )
            }
        };

        Some(StatementParserResult {
            statement,
            last_expression_in_block,
        })
    }
}

pub(crate) struct ExpressionStatementParser;

pub(crate) struct ExpressionStatementParserResult {
    pub(crate) expression_statement: Statement,
    pub(crate) last_expression_in_block: bool,
}

impl Parse for ExpressionStatementParser {
    type Output = Option<ExpressionStatementParserResult>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let expression = ExpressionParser::new().in_statements_block().parse(state)?;

        let (last_expression_in_block, has_semicolon) =
            if state.current_token.raw == Punctuator::CloseBrace {
                // 1. `ExpressionWithBlocks` are treated as individual statements
                //    (last_expression_in_block = false)
                // 2. Semicolons after them are also treated as individual statements
                //    (has_semicolon = false)
                (false, false)
            } else if state.next_token.raw == Punctuator::Semicolon {
                state.advance();

                (false, true)
            } else {
                (true, false)
            };

        Some(ExpressionStatementParserResult {
            expression_statement: Statement::Expression {
                expression,
                has_semicolon,
            },
            last_expression_in_block,
        })
    }
}

pub(crate) struct StatementsBlockParser;

impl Parse for StatementsBlockParser {
    type Output = Option<Vec<Statement>>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.consume(Punctuator::OpenBrace)?;

        let mut block = vec![];

        loop {
            match state.next_token.raw {
                RawToken::Punctuator(Punctuator::CloseBrace) => break,
                RawToken::EndOfFile => {
                    state.add_diagnostic(UnexpectedToken::new(
                        state.current_token.location.end,
                        state.next_token,
                        Punctuator::CloseBrace,
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

            let StatementParserResult {
                statement,
                last_expression_in_block,
            } = StatementParser.parse(state)?;
            block.push(statement);

            if last_expression_in_block {
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

        let call = ExpressionParser::default().parse(state)?;

        state.consume(Punctuator::Semicolon)?;

        Some(Statement::Defer { call })
    }
}

struct ReturnStatementParser;

impl Parse for ReturnStatementParser {
    type Output = Option<Statement>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let expression = ExpressionParser::default().parse(state)?;

        state.consume(Punctuator::Semicolon)?;

        Some(Statement::Return { expression })
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

        state.consume(Punctuator::Eq)?;

        let value = ExpressionParser::default().parse(state)?;

        state.consume(Punctuator::Semicolon)?;

        Some(Statement::Let { pattern, value, ty })
    }
}

struct ContinueStatementParser;

impl Parse for ContinueStatementParser {
    type Output = Option<Statement>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let location = state.current_token.location;

        state.consume(Punctuator::Semicolon)?;

        Some(Statement::Continue { location })
    }
}

struct BreakStatementParser;

impl Parse for BreakStatementParser {
    type Output = Option<Statement>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let location = state.current_token.location;

        state.consume(Punctuator::Semicolon)?;

        Some(Statement::Break { location })
    }
}
