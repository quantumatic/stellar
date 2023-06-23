use crate::{
    expression::ExpressionParser, pattern::PatternParser, r#type::TypeParser, Parse, TokenIterator,
};
use ry_ast::{token::RawToken, Statement, StatementsBlock, Token};
use ry_diagnostics::{parser::ParseDiagnostic, Report};
use ry_source_file::span::Span;

struct StatementParser;

pub(crate) struct StatementsBlockParser;

struct DeferStatementParser;

struct ReturnStatementParser;

struct LetStatementParser;

impl Parse for StatementParser {
    type Output = Option<(Statement, bool)>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let mut last_statement_in_block = false;
        let mut must_have_semicolon_at_the_end = true;

        let mut no_semicolon_after_expression_error_emitted = false;

        let start = iterator.next.span.start();

        let statement = match iterator.next.raw {
            Token![return] => ReturnStatementParser.parse_using(iterator)?,
            Token![defer] => DeferStatementParser.parse_using(iterator)?,
            Token![let] => LetStatementParser.parse_using(iterator)?,
            _ => {
                let expression = ExpressionParser::default().parse_using(iterator)?;

                must_have_semicolon_at_the_end = !expression.with_block();

                match iterator.next.raw {
                    Token![;] => {}
                    Token!['}'] => {
                        if must_have_semicolon_at_the_end {
                            last_statement_in_block = true;
                        }
                    }
                    _ => {
                        no_semicolon_after_expression_error_emitted = true;

                        iterator.diagnostics.push(
                            ParseDiagnostic::NoSemicolonAfterExpressionError {
                                expression_span: expression.span(),
                                span: Span::new(
                                    expression.span().end() - 1,
                                    expression.span().end(),
                                    iterator.file_id,
                                ),
                            }
                            .build(),
                        );
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

        let end = iterator.current.span.end();

        if !last_statement_in_block
            && must_have_semicolon_at_the_end
            && !no_semicolon_after_expression_error_emitted
        {
            if iterator.next.raw == Token![;] {
                iterator.next_token();
            } else {
                iterator.diagnostics.push(
                    ParseDiagnostic::NoSemicolonAfterStatementError {
                        statement_span: Span::new(start, end - 1, iterator.file_id),
                        span: Span::new(end, end, iterator.file_id),
                    }
                    .build(),
                );
            }
        }

        Some((statement, last_statement_in_block))
    }
}

impl Parse for StatementsBlockParser {
    type Output = Option<StatementsBlock>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.consume(Token!['{'], "statements block")?;
        let start = iterator.current.span.start();

        let mut block = vec![];

        loop {
            match iterator.next.raw {
                Token!['}'] => break,
                RawToken::EndOfFile => {
                    iterator.diagnostics.push(
                        ParseDiagnostic::EOFInsteadOfCloseBraceForStatementsBlockError {
                            statements_block_start_span: Span::new(
                                start,
                                start + 1,
                                iterator.file_id,
                            ),
                            span: iterator.current.span,
                        }
                        .build(),
                    );

                    return None;
                }
                Token![;] => {
                    iterator.diagnostics.push(
                        ParseDiagnostic::EmptyStatementWarning {
                            span: iterator.next.span,
                        }
                        .build(),
                    );

                    // Recover
                    iterator.next_token(); // `;`
                    continue;
                }
                _ => {}
            }

            let (statement, last) = StatementParser.parse_using(iterator)?;
            block.push(statement);

            if last {
                break;
            }
        }

        iterator.next_token();

        Some(block)
    }
}

impl Parse for DeferStatementParser {
    type Output = Option<Statement>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token();

        Some(Statement::Defer {
            call: ExpressionParser::default().parse_using(iterator)?,
        })
    }
}

impl Parse for ReturnStatementParser {
    type Output = Option<Statement>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token();

        Some(Statement::Return {
            expression: ExpressionParser::default().parse_using(iterator)?,
        })
    }
}

impl Parse for LetStatementParser {
    type Output = Option<Statement>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token(); // `let`

        let pattern = PatternParser.parse_using(iterator)?;

        let ty = if iterator.next.raw == Token![:] {
            iterator.next_token();
            Some(TypeParser.parse_using(iterator)?)
        } else {
            None
        };

        iterator.consume(Token![=], "let statement")?;

        let value = Box::new(ExpressionParser::default().parse_using(iterator)?);

        Some(Statement::Let { pattern, value, ty })
    }
}
