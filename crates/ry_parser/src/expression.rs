use ry_ast::{
    precedence::Precedence, token::RawToken, BinaryOperator, Expression, IdentifierAst,
    LambdaFunctionParameter, MatchExpressionItem, PostfixOperator, PrefixOperator,
    RawBinaryOperator, RawPostfixOperator, RawPrefixOperator, StructExpressionItem, Token,
};
use ry_diagnostics::BuildDiagnostic;
use ry_filesystem::span::Span;

use crate::{
    diagnostics::ParseDiagnostic,
    expected,
    literal::LiteralParser,
    macros::parse_list,
    pattern::PatternParser,
    r#type::{GenericArgumentsParser, TypeParser},
    statement::StatementsBlockParser,
    Parse, ParseState,
};

#[derive(Default)]
pub(crate) struct ExpressionParser {
    pub(crate) precedence: Precedence,
    pub(crate) ignore_struct: bool,
}

struct WhileExpressionParser;

struct MatchExpressionParser;

struct MatchExpressionBlockParser;

struct MatchExpressionUnitParser;

struct PrimaryExpressionParser {
    pub(crate) ignore_struct: bool,
}

struct GenericArgumentsExpressionParser {
    pub(crate) left: Expression,
}

struct PropertyAccessExpressionParser {
    pub(crate) left: Expression,
}

struct PrefixExpressionParser {
    pub(crate) ignore_struct: bool,
}

struct PostfixExpressionParser {
    pub(crate) left: Expression,
}

struct CastExpressionParser {
    pub(crate) left: Expression,
}

struct IfExpressionParser;

struct ParenthesizedOrTupleExpressionParser;

struct CallExpressionParser {
    pub(crate) left: Expression,
}

struct BinaryExpressionParser {
    pub(crate) left: Expression,
    pub(crate) ignore_struct: bool,
}

struct ListExpressionParser;

struct StructExpressionParser {
    pub(crate) left: Expression,
}

struct StructExpressionUnitParser;

struct FunctionExpressionParser;

struct StatementsBlockExpressionParser;

impl Parse for ExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let mut left = PrimaryExpressionParser {
            ignore_struct: self.ignore_struct,
        }
        .parse(state)?;

        while self.precedence < state.next_token.raw.to_precedence() {
            left = match state.next_token.raw {
                Token!['('] => CallExpressionParser { left }.parse(state)?,
                Token![.] => PropertyAccessExpressionParser { left }.parse(state)?,
                Token!['['] => GenericArgumentsExpressionParser { left }.parse(state)?,
                Token![as] => CastExpressionParser { left }.parse(state)?,
                Token!['{'] => {
                    if self.ignore_struct {
                        return Some(left);
                    }

                    StructExpressionParser { left }.parse(state)?
                }
                _ => {
                    if state.next_token.raw.binary_operator() {
                        BinaryExpressionParser {
                            left,
                            ignore_struct: self.ignore_struct,
                        }
                        .parse(state)?
                    } else if state.next_token.raw.postfix_operator() {
                        PostfixExpressionParser { left }.parse(state)?
                    } else {
                        break;
                    }
                }
            };
        }

        Some(left)
    }
}

impl Parse for WhileExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start;
        state.advance(); // `while`

        let condition = ExpressionParser {
            precedence: Precedence::Lowest,
            ignore_struct: true,
        }
        .parse(state)?;

        let body = StatementsBlockParser.parse(state)?;

        Some(Expression::While {
            span: state.span_from(start),
            condition: Box::new(condition),
            body,
        })
    }
}

impl Parse for MatchExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start;
        state.advance(); // `match`

        let expression = ExpressionParser {
            precedence: Precedence::Lowest,
            ignore_struct: true,
        }
        .parse(state)?;

        let block = MatchExpressionBlockParser.parse(state)?;

        Some(Expression::Match {
            span: state.span_from(start),
            expression: Box::new(expression),
            block,
        })
    }
}

impl Parse for MatchExpressionBlockParser {
    type Output = Option<Vec<MatchExpressionItem>>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.consume(Token!['{'], "match expression block")?;

        let units = parse_list!(state, "match expression block", Token!['}'], {
            MatchExpressionUnitParser.parse(state)
        });

        state.advance(); // `}`

        Some(units)
    }
}

impl Parse for MatchExpressionUnitParser {
    type Output = Option<MatchExpressionItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let left = PatternParser.parse(state)?;
        state.consume(Token![=>], "match expression unit")?;

        let right = ExpressionParser::default().parse(state)?;

        Some(MatchExpressionItem { left, right })
    }
}

impl Parse for PrimaryExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        match state.next_token.raw {
            RawToken::IntegerLiteral
            | RawToken::FloatLiteral
            | RawToken::StringLiteral
            | RawToken::CharLiteral
            | Token![true]
            | Token![false] => Some(Expression::Literal(LiteralParser.parse(state)?)),
            RawToken::Identifier => {
                let symbol = state.lexer.scanned_identifier;
                state.advance();

                Some(Expression::Identifier(IdentifierAst {
                    span: state.current_token.span,
                    symbol,
                }))
            }
            Token!['('] => ParenthesizedOrTupleExpressionParser.parse(state),
            Token!['['] => ListExpressionParser.parse(state),
            Token!['{'] => StatementsBlockExpressionParser.parse(state),
            Token![|] => FunctionExpressionParser.parse(state),
            Token![if] => IfExpressionParser.parse(state),
            Token![match] => MatchExpressionParser.parse(state),
            Token![while] => WhileExpressionParser.parse(state),
            _ => {
                if state.next_token.raw.prefix_operator() {
                    return PrefixExpressionParser {
                        ignore_struct: self.ignore_struct,
                    }
                    .parse(state);
                }
                state.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: state.next_token,
                        expected: expected!(
                            "integer literal",
                            "float literal",
                            "string literal",
                            "char literal",
                            "boolean literal",
                            Token![|],
                            Token!['('],
                            Token!['{'],
                            Token!['['],
                            "identifier",
                            Token![if],
                            Token![while],
                            Token![match]
                        ),
                        node: "expression".to_owned(),
                    }
                    .build(),
                );
                None
            }
        }
    }
}

impl Parse for GenericArgumentsExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let generic_arguments = GenericArgumentsParser.parse(state)?;

        Some(Expression::GenericArguments {
            span: state.span_from(self.left.span().start),
            left: Box::new(self.left),
            generic_arguments,
        })
    }
}

impl Parse for PropertyAccessExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance(); // `.`

        let right = state.consume_identifier("property")?;

        Some(Expression::FieldAccess {
            span: state.span_from(self.left.span().start),
            left: Box::new(self.left),
            right,
        })
    }
}

impl Parse for PrefixExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let operator_token = state.next_token;
        let operator: PrefixOperator = PrefixOperator {
            span: operator_token.span,
            raw: RawPrefixOperator::from(operator_token.raw),
        };
        state.advance();

        let inner = ExpressionParser {
            precedence: Precedence::Unary,
            ignore_struct: self.ignore_struct,
        }
        .parse(state)?;

        Some(Expression::Prefix {
            span: Span {
                start: operator_token.span.start,
                end: inner.span().end,
            },
            inner: Box::new(inner),
            operator,
        })
    }
}

impl Parse for PostfixExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let operator: PostfixOperator = PostfixOperator {
            span: state.current_token.span,
            raw: RawPostfixOperator::from(state.current_token.raw),
        };

        Some(Expression::Postfix {
            span: state.span_from(self.left.span().start),
            inner: Box::new(self.left),
            operator,
        })
    }
}

impl Parse for ParenthesizedOrTupleExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start;
        state.advance();

        let elements = parse_list!(state, "parenthesized or tuple expression", Token![')'], {
            ExpressionParser::default().parse(state)
        });

        state.advance(); // `)`

        let span = state.span_from(start);

        let mut elements = elements.into_iter();

        match (elements.next(), elements.next()) {
            (Some(element), None) => {
                if state
                    .resolve_span(state.span_from(element.span().end))
                    .contains(',')
                {
                    Some(Expression::Tuple {
                        span,
                        elements: vec![element],
                    })
                } else {
                    Some(Expression::Parenthesized {
                        span,
                        inner: Box::from(element),
                    })
                }
            }
            (None, None) => Some(Expression::Tuple {
                span,
                elements: vec![],
            }),
            (Some(previous), Some(next)) => {
                let mut new_elements = vec![];
                new_elements.push(previous);
                new_elements.push(next);

                new_elements.append(&mut elements.collect::<Vec<_>>());

                Some(Expression::Tuple {
                    span,
                    elements: new_elements,
                })
            }
            _ => unreachable!(),
        }
    }
}

impl Parse for IfExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start;
        state.advance(); // `if`

        let condition = ExpressionParser {
            precedence: Precedence::Lowest,
            ignore_struct: true,
        }
        .parse(state)?;

        let block = StatementsBlockParser.parse(state)?;

        let mut if_blocks = vec![(condition, block)];

        let mut r#else = None;

        while state.next_token.raw == Token![else] {
            state.advance();

            if state.next_token.raw != Token![if] {
                r#else = Some(StatementsBlockParser.parse(state)?);
                break;
            }

            state.advance();

            let condition = ExpressionParser {
                precedence: Precedence::Lowest,
                ignore_struct: true,
            }
            .parse(state)?;
            let block = StatementsBlockParser.parse(state)?;

            if_blocks.push((condition, block));
        }

        Some(Expression::If {
            span: state.span_from(start),
            if_blocks,
            r#else,
        })
    }
}

impl Parse for CastExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let right = TypeParser.parse(state)?;

        Some(Expression::As {
            span: state.span_from(self.left.span().start),
            left: Box::new(self.left),
            right,
        })
    }
}

impl Parse for CallExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance(); // `(`

        let arguments = parse_list!(state, "call arguments list", Token![')'], {
            ExpressionParser::default().parse(state)
        });

        state.advance();

        Some(Expression::Call {
            span: state.span_from(self.left.span().start),
            left: Box::new(self.left),
            arguments,
        })
    }
}

impl Parse for BinaryExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let operator_token = state.next_token;
        let operator: BinaryOperator = BinaryOperator {
            span: operator_token.span,
            raw: RawBinaryOperator::from(operator_token.raw),
        };
        let precedence = state.next_token.raw.to_precedence();

        state.advance();

        let right = ExpressionParser {
            precedence,
            ignore_struct: self.ignore_struct,
        }
        .parse(state)?;

        Some(Expression::Binary {
            span: state.span_from(self.left.span().start),
            left: Box::new(self.left),
            right: Box::new(right),
            operator,
        })
    }
}

impl Parse for ListExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start;

        state.advance();

        let elements = parse_list!(state, "list expression", Token![']'], {
            ExpressionParser::default().parse(state)
        });

        state.advance();

        Some(Expression::List {
            span: state.span_from(start),
            elements,
        })
    }
}

impl Parse for StructExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance(); // `{`

        let fields = parse_list!(state, "struct expression", Token!['}'], {
            StructExpressionUnitParser.parse(state)
        });

        state.advance(); // `}`

        Some(Expression::Struct {
            span: state.span_from(self.left.span().start),
            left: Box::new(self.left),
            fields,
        })
    }
}

impl Parse for StructExpressionUnitParser {
    type Output = Option<StructExpressionItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let name = state.consume_identifier("struct field")?;

        let value = if state.next_token.raw == Token![:] {
            state.advance();
            Some(ExpressionParser::default().parse(state)?)
        } else {
            None
        };

        Some(StructExpressionItem { name, value })
    }
}

impl Parse for StatementsBlockExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start;
        let block = StatementsBlockParser.parse(state)?;

        Some(Expression::StatementsBlock {
            span: state.span_from(start),
            block,
        })
    }
}

impl Parse for FunctionExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start;
        state.advance(); // `|`

        let parameters = parse_list!(state, "function expression parameters", Token![|], {
            let name = state.consume_identifier("function parameter name")?;

            let ty = if state.next_token.raw == Token![:] {
                state.advance();

                Some(TypeParser.parse(state)?)
            } else {
                None
            };

            Some(LambdaFunctionParameter { name, ty })
        });

        state.advance();

        let return_type = if state.next_token.raw == Token![:] {
            state.advance();

            Some(TypeParser.parse(state)?)
        } else {
            None
        };

        let block = StatementsBlockParser.parse(state)?;

        Some(Expression::Function {
            span: state.span_from(start),
            parameters,
            return_type,
            block,
        })
    }
}
