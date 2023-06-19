use crate::{
    expression::ExpressionParser, pattern::PatternParser, r#type::TypeParser, Cursor, Parse,
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

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let mut last_statement_in_block = false;
        let mut must_have_semicolon_at_the_end = true;

        let mut no_semicolon_after_expression_error_emitted = false;

        let start = cursor.next.span.start();

        let statement = match cursor.next.raw {
            Token![return] => ReturnStatementParser.parse_with(cursor)?,
            Token![defer] => DeferStatementParser.parse_with(cursor)?,
            Token![let] => LetStatementParser.parse_with(cursor)?,
            _ => {
                let expression = ExpressionParser::default().parse_with(cursor)?;

                must_have_semicolon_at_the_end = !expression.with_block();

                match cursor.next.raw {
                    Token![;] => {}
                    Token!['}'] => {
                        if must_have_semicolon_at_the_end {
                            last_statement_in_block = true;
                        }
                    }
                    _ => {
                        no_semicolon_after_expression_error_emitted = true;

                        cursor.diagnostics.push(
                            ParseDiagnostic::NoSemicolonAfterExpressionError {
                                expression_span: expression.span(),
                                span: Span::new(
                                    expression.span().end() - 1,
                                    expression.span().end(),
                                    cursor.file_id,
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

        let end = cursor.current.span.end();

        if !last_statement_in_block
            && must_have_semicolon_at_the_end
            && !no_semicolon_after_expression_error_emitted
        {
            if cursor.next.raw == Token![;] {
                cursor.next_token();
            } else {
                cursor.diagnostics.push(
                    ParseDiagnostic::NoSemicolonAfterStatementError {
                        statement_span: Span::new(start, end - 1, cursor.file_id),
                        span: Span::new(end, end, cursor.file_id),
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

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.consume(Token!['{'], "statements block")?;
        let start = cursor.current.span.start();

        let mut block = vec![];

        loop {
            match cursor.next.raw {
                Token!['}'] => break,
                RawToken::EndOfFile => {
                    cursor.diagnostics.push(
                        ParseDiagnostic::EOFInsteadOfCloseBraceForStatementsBlockError {
                            statements_block_start_span: Span::new(
                                start,
                                start + 1,
                                cursor.file_id,
                            ),
                            span: cursor.current.span,
                        }
                        .build(),
                    );

                    return None;
                }
                Token![;] => {
                    cursor.diagnostics.push(
                        ParseDiagnostic::EmptyStatementWarning {
                            span: cursor.next.span,
                        }
                        .build(),
                    );

                    // Recover
                    cursor.next_token(); // `;`
                    continue;
                }
                _ => {}
            }

            let (statement, last) = StatementParser.parse_with(cursor)?;
            block.push(statement);

            if last {
                break;
            }
        }

        cursor.next_token();

        Some(block)
    }
}

impl Parse for DeferStatementParser {
    type Output = Option<Statement>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();

        Some(Statement::Defer {
            call: ExpressionParser::default().parse_with(cursor)?,
        })
    }
}

impl Parse for ReturnStatementParser {
    type Output = Option<Statement>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();

        Some(Statement::Return {
            expression: ExpressionParser::default().parse_with(cursor)?,
        })
    }
}

impl Parse for LetStatementParser {
    type Output = Option<Statement>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `let`

        let pattern = PatternParser.parse_with(cursor)?;

        let ty = if cursor.next.raw == Token![:] {
            cursor.next_token();
            Some(TypeParser.parse_with(cursor)?)
        } else {
            None
        };

        cursor.consume(Token![=], "let statement")?;

        let value = Box::new(ExpressionParser::default().parse_with(cursor)?);

        Some(Statement::Let { pattern, value, ty })
    }
}
