use crate::{
    items::FunctionParameterParser,
    literal::LiteralParser,
    macros::parse_list,
    pattern::PatternParser,
    r#type::{GenericArgumentsParser, TypeParser},
    statement::StatementsBlockParser,
    Parse, ParseState,
};
use ry_ast::{
    precedence::Precedence, token::RawToken, BinaryOperator, IdentifierAst, MatchExpressionItem,
    PostfixOperator, PrefixOperator, RawBinaryOperator, RawPostfixOperator, RawPrefixOperator,
    StructExpressionItem, Token, UntypedExpression,
};
use ry_diagnostics::{expected, parser::ParseDiagnostic, BuildDiagnostic};
use ry_source_file::span::Span;

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
    pub(crate) start: usize,
    pub(crate) left: UntypedExpression,
}

struct PropertyAccessExpressionParser {
    pub(crate) start: usize,
    pub(crate) left: UntypedExpression,
}

struct PrefixExpressionParser {
    pub(crate) ignore_struct: bool,
}

struct PostfixExpressionParser {
    pub(crate) start: usize,
    pub(crate) left: UntypedExpression,
}

struct CastExpressionParser {
    pub(crate) start: usize,
    pub(crate) left: UntypedExpression,
}

struct IfExpressionParser;

struct ParenthesizedExpressionParser;

struct CallExpressionParser {
    pub(crate) start: usize,
    pub(crate) left: UntypedExpression,
}

struct BinaryExpressionParser {
    pub(crate) start: usize,
    pub(crate) left: UntypedExpression,
    pub(crate) ignore_struct: bool,
}

struct ListExpressionParser;

struct TupleExpressionParser;

struct StructExpressionParser {
    pub(crate) start: usize,
    pub(crate) left: UntypedExpression,
}

struct StructExpressionUnitParser;

struct FunctionExpressionParser;

struct StatementsBlockExpressionParser;

impl Parse for ExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start();
        let mut left = PrimaryExpressionParser {
            ignore_struct: self.ignore_struct,
        }
        .parse(state)?;

        while self.precedence < state.next_token.raw.to_precedence() {
            left = match state.next_token.raw {
                Token!['('] => CallExpressionParser { start, left }.parse(state)?,
                Token![.] => PropertyAccessExpressionParser { start, left }.parse(state)?,
                Token!['['] => GenericArgumentsExpressionParser { start, left }.parse(state)?,
                Token![as] => CastExpressionParser { start, left }.parse(state)?,
                Token!['{'] => {
                    if self.ignore_struct {
                        return Some(left);
                    }

                    StructExpressionParser { start, left }.parse(state)?
                }
                _ => {
                    if state.next_token.raw.binary_operator() {
                        BinaryExpressionParser {
                            start,
                            left,
                            ignore_struct: self.ignore_struct,
                        }
                        .parse(state)?
                    } else if state.next_token.raw.postfix_operator() {
                        PostfixExpressionParser { start, left }.parse(state)?
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
    type Output = Option<UntypedExpression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start();
        state.advance(); // `while`

        let condition = ExpressionParser {
            precedence: Precedence::Lowest,
            ignore_struct: true,
        }
        .parse(state)?;

        let body = StatementsBlockParser.parse(state)?;

        Some(UntypedExpression::While {
            span: Span::new(start, state.current_token.span.end(), state.file_id),
            condition: Box::new(condition),
            body,
        })
    }
}

impl Parse for MatchExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start();
        state.advance(); // `match`

        let expression = ExpressionParser {
            precedence: Precedence::Lowest,
            ignore_struct: true,
        }
        .parse(state)?;

        let block = MatchExpressionBlockParser.parse(state)?;

        Some(UntypedExpression::Match {
            span: Span::new(start, state.current_token.span.end(), state.file_id),
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
    type Output = Option<UntypedExpression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        match state.next_token.raw {
            RawToken::IntegerLiteral
            | RawToken::FloatLiteral
            | RawToken::StringLiteral
            | RawToken::CharLiteral
            | Token![true]
            | Token![false] => Some(UntypedExpression::Literal(LiteralParser.parse(state)?)),
            RawToken::Identifier => {
                let symbol = state.lexer.identifier();
                state.advance();

                Some(UntypedExpression::Identifier(IdentifierAst {
                    span: state.current_token.span,
                    symbol,
                }))
            }
            Token!['('] => ParenthesizedExpressionParser.parse(state),
            Token!['['] => ListExpressionParser.parse(state),
            Token!['{'] => StatementsBlockExpressionParser.parse(state),
            Token![|] => FunctionExpressionParser.parse(state),
            Token![#] => TupleExpressionParser.parse(state),
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
                            Token![#],
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
    type Output = Option<UntypedExpression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let generic_arguments = GenericArgumentsParser.parse(state)?;

        Some(UntypedExpression::GenericArguments {
            span: Span::new(self.start, state.current_token.span.end(), state.file_id),
            left: Box::new(self.left),
            generic_arguments,
        })
    }
}

impl Parse for PropertyAccessExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance(); // `.`

        Some(UntypedExpression::FieldAccess {
            span: Span::new(self.start, state.current_token.span.end(), state.file_id),
            left: Box::new(self.left),
            right: state.consume_identifier("property")?,
        })
    }
}

impl Parse for PrefixExpressionParser {
    type Output = Option<UntypedExpression>;

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

        Some(UntypedExpression::Prefix {
            span: Span::new(
                operator_token.span.start(),
                inner.span().end(),
                state.file_id,
            ),
            inner: Box::new(inner),
            operator,
        })
    }
}

impl Parse for PostfixExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let operator: PostfixOperator = PostfixOperator {
            span: state.current_token.span,
            raw: RawPostfixOperator::from(state.current_token.raw),
        };

        Some(UntypedExpression::Postfix {
            span: Span::new(self.start, state.current_token.span.end(), state.file_id),
            inner: Box::new(self.left),
            operator,
        })
    }
}

impl Parse for ParenthesizedExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start();
        state.advance();

        let inner = ExpressionParser {
            precedence: Precedence::Lowest,
            ignore_struct: false,
        }
        .parse(state)?;

        state.consume(Token![')'], "parenthesized expression")?;

        Some(UntypedExpression::Parenthesized {
            span: Span::new(start, inner.span().end(), state.file_id),
            inner: Box::new(inner),
        })
    }
}

impl Parse for IfExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start();
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

        Some(UntypedExpression::If {
            span: Span::new(start, state.current_token.span.end(), state.file_id),
            if_blocks,
            r#else,
        })
    }
}

impl Parse for CastExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let right = TypeParser.parse(state)?;

        Some(UntypedExpression::As {
            span: Span::new(self.start, state.current_token.span.end(), state.file_id),
            left: Box::new(self.left),
            right,
        })
    }
}

impl Parse for CallExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance(); // `(`

        let arguments = parse_list!(state, "call arguments list", Token![')'], {
            ExpressionParser::default().parse(state)
        });

        state.advance();

        Some(UntypedExpression::Call {
            span: Span::new(self.start, state.current_token.span.end(), state.file_id),
            left: Box::new(self.left),
            arguments,
        })
    }
}

impl Parse for BinaryExpressionParser {
    type Output = Option<UntypedExpression>;

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

        Some(UntypedExpression::Binary {
            span: Span::new(self.start, state.current_token.span.end(), state.file_id),
            left: Box::new(self.left),
            right: Box::new(right),
            operator,
        })
    }
}

impl Parse for ListExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let start = state.next_token.span.start();

        let elements = parse_list!(state, "list expression", Token![']'], {
            ExpressionParser::default().parse(state)
        });

        state.advance();

        Some(UntypedExpression::List {
            span: Span::new(start, state.current_token.span.end(), state.file_id),
            elements,
        })
    }
}

impl Parse for TupleExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start();
        state.advance(); // `#`

        state.consume(Token!['('], "tuple expression")?;

        let elements = parse_list!(state, "tuple expression", Token![')'], {
            ExpressionParser::default().parse(state)
        });

        state.advance(); // `)`

        Some(UntypedExpression::Tuple {
            span: Span::new(start, state.current_token.span.end(), state.file_id),
            elements,
        })
    }
}

impl Parse for StructExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance(); // `{`

        let fields = parse_list!(state, "struct expression", Token!['}'], {
            StructExpressionUnitParser.parse(state)
        });

        state.advance(); // `}`

        Some(UntypedExpression::Struct {
            span: Span::new(self.start, state.current_token.span.end(), state.file_id),
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
    type Output = Option<UntypedExpression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start();
        let block = StatementsBlockParser.parse(state)?;

        Some(UntypedExpression::StatementsBlock {
            span: Span::new(start, state.current_token.span.end(), state.file_id),
            block,
        })
    }
}

impl Parse for FunctionExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start();
        state.advance(); // `|`

        let parameters = parse_list!(state, "function expression parameters", Token![|], {
            FunctionParameterParser.parse(state)
        });

        state.advance();

        let return_type = if state.next_token.raw == Token![:] {
            state.advance();

            Some(TypeParser.parse(state)?)
        } else {
            None
        };

        let block = StatementsBlockParser.parse(state)?;

        Some(UntypedExpression::Function {
            span: Span::new(start, state.current_token.span.end(), state.file_id),
            parameters,
            return_type,
            block,
        })
    }
}
