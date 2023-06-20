use crate::{
    items::FunctionParameterParser,
    literal::LiteralParser,
    macros::parse_list,
    pattern::PatternParser,
    r#type::{GenericArgumentsParser, TypeParser},
    statement::StatementsBlockParser,
    Cursor, Parse,
};
use ry_ast::{
    precedence::Precedence, token::RawToken, BinaryOperator, IdentifierAst, MatchExpressionItem,
    PostfixOperator, PrefixOperator, RawBinaryOperator, RawPostfixOperator, RawPrefixOperator,
    StructExpressionItem, Token, UntypedExpression,
};
use ry_diagnostics::{expected, parser::ParseDiagnostic, Report};
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

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = cursor.next.span.start();
        let mut left = PrimaryExpressionParser {
            ignore_struct: self.ignore_struct,
        }
        .parse_with(cursor)?;

        while self.precedence < cursor.next.raw.to_precedence() {
            left = match cursor.next.raw {
                Token!['('] => CallExpressionParser { start, left }.parse_with(cursor)?,
                Token![.] => PropertyAccessExpressionParser { start, left }.parse_with(cursor)?,
                Token!['['] => {
                    GenericArgumentsExpressionParser { start, left }.parse_with(cursor)?
                }
                Token![as] => CastExpressionParser { start, left }.parse_with(cursor)?,
                Token!['{'] => {
                    if self.ignore_struct {
                        return Some(left);
                    }

                    StructExpressionParser { start, left }.parse_with(cursor)?
                }
                _ => {
                    if cursor.next.raw.binary_operator() {
                        BinaryExpressionParser {
                            start,
                            left,
                            ignore_struct: self.ignore_struct,
                        }
                        .parse_with(cursor)?
                    } else if cursor.next.raw.postfix_operator() {
                        PostfixExpressionParser { start, left }.parse_with(cursor)?
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

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = cursor.next.span.start();
        cursor.next_token(); // `while`

        let condition = ExpressionParser {
            precedence: Precedence::Lowest,
            ignore_struct: true,
        }
        .parse_with(cursor)?;

        let body = StatementsBlockParser.parse_with(cursor)?;

        Some(UntypedExpression::While {
            span: Span::new(start, cursor.current.span.end(), cursor.file_id),
            condition: Box::new(condition),
            body,
        })
    }
}

impl Parse for MatchExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = cursor.next.span.start();
        cursor.next_token(); // `match`

        let expression = ExpressionParser {
            precedence: Precedence::Lowest,
            ignore_struct: true,
        }
        .parse_with(cursor)?;

        let block = MatchExpressionBlockParser.parse_with(cursor)?;

        Some(UntypedExpression::Match {
            span: Span::new(start, cursor.current.span.end(), cursor.file_id),
            expression: Box::new(expression),
            block,
        })
    }
}

impl Parse for MatchExpressionBlockParser {
    type Output = Option<Vec<MatchExpressionItem>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.consume(Token!['{'], "match expression block")?;

        let units = parse_list!(cursor, "match expression block", Token!['}'], || {
            MatchExpressionUnitParser.parse_with(cursor)
        });

        cursor.next_token(); // `}`

        Some(units)
    }
}

impl Parse for MatchExpressionUnitParser {
    type Output = Option<MatchExpressionItem>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let left = PatternParser.parse_with(cursor)?;
        cursor.consume(Token![=>], "match expression unit")?;

        let right = ExpressionParser::default().parse_with(cursor)?;

        Some(MatchExpressionItem { left, right })
    }
}

impl Parse for PrimaryExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        match cursor.next.raw {
            RawToken::IntegerLiteral
            | RawToken::FloatLiteral
            | RawToken::StringLiteral
            | RawToken::CharLiteral
            | Token![true]
            | Token![false] => Some(UntypedExpression::Literal(
                LiteralParser.parse_with(cursor)?,
            )),
            RawToken::Identifier => {
                let symbol = cursor.lexer.identifier();
                cursor.next_token();

                Some(UntypedExpression::Identifier(IdentifierAst {
                    span: cursor.current.span,
                    symbol,
                }))
            }
            Token!['('] => ParenthesizedExpressionParser.parse_with(cursor),
            Token!['['] => ListExpressionParser.parse_with(cursor),
            Token!['{'] => StatementsBlockExpressionParser.parse_with(cursor),
            Token![|] => FunctionExpressionParser.parse_with(cursor),
            Token![#] => TupleExpressionParser.parse_with(cursor),
            Token![if] => IfExpressionParser.parse_with(cursor),
            Token![match] => MatchExpressionParser.parse_with(cursor),
            Token![while] => WhileExpressionParser.parse_with(cursor),
            _ => {
                if cursor.next.raw.prefix_operator() {
                    return PrefixExpressionParser {
                        ignore_struct: self.ignore_struct,
                    }
                    .parse_with(cursor);
                }
                cursor.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: cursor.next,
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

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let arguments = GenericArgumentsParser.parse_with(cursor)?;

        Some(UntypedExpression::GenericArguments {
            span: Span::new(self.start, cursor.current.span.end(), cursor.file_id),
            left: Box::new(self.left),
            arguments,
        })
    }
}

impl Parse for PropertyAccessExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `.`

        Some(UntypedExpression::Property {
            span: Span::new(self.start, cursor.current.span.end(), cursor.file_id),
            left: Box::new(self.left),
            right: cursor.consume_identifier("property")?,
        })
    }
}

impl Parse for PrefixExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let operator_token = cursor.next;
        let operator: PrefixOperator = PrefixOperator {
            span: operator_token.span,
            raw: RawPrefixOperator::from(operator_token.raw),
        };
        cursor.next_token();

        let inner = ExpressionParser {
            precedence: Precedence::Unary,
            ignore_struct: self.ignore_struct,
        }
        .parse_with(cursor)?;

        Some(UntypedExpression::Prefix {
            span: Span::new(
                operator_token.span.start(),
                inner.span().end(),
                cursor.file_id,
            ),
            inner: Box::new(inner),
            operator,
        })
    }
}

impl Parse for PostfixExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();

        let operator: PostfixOperator = PostfixOperator {
            span: cursor.current.span,
            raw: RawPostfixOperator::from(cursor.current.raw),
        };

        Some(UntypedExpression::Postfix {
            span: Span::new(self.start, cursor.current.span.end(), cursor.file_id),
            inner: Box::new(self.left),
            operator,
        })
    }
}

impl Parse for ParenthesizedExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = cursor.next.span.start();
        cursor.next_token();

        let inner = ExpressionParser {
            precedence: Precedence::Lowest,
            ignore_struct: false,
        }
        .parse_with(cursor)?;

        cursor.consume(Token![')'], "parenthesized expression")?;

        Some(UntypedExpression::Parenthesized {
            span: Span::new(start, inner.span().end(), cursor.file_id),
            inner: Box::new(inner),
        })
    }
}

impl Parse for IfExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = cursor.next.span.start();
        cursor.next_token(); // `if`

        let condition = ExpressionParser {
            precedence: Precedence::Lowest,
            ignore_struct: true,
        }
        .parse_with(cursor)?;

        let block = StatementsBlockParser.parse_with(cursor)?;

        let mut if_blocks = vec![(condition, block)];

        let mut r#else = None;

        while cursor.next.raw == Token![else] {
            cursor.next_token();

            if cursor.next.raw != Token![if] {
                r#else = Some(StatementsBlockParser.parse_with(cursor)?);
                break;
            }

            cursor.next_token();

            let condition = ExpressionParser {
                precedence: Precedence::Lowest,
                ignore_struct: true,
            }
            .parse_with(cursor)?;
            let block = StatementsBlockParser.parse_with(cursor)?;

            if_blocks.push((condition, block));
        }

        Some(UntypedExpression::If {
            span: Span::new(start, cursor.current.span.end(), cursor.file_id),
            if_blocks,
            r#else,
        })
    }
}

impl Parse for CastExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();

        let right = TypeParser.parse_with(cursor)?;

        Some(UntypedExpression::As {
            span: Span::new(self.start, cursor.current.span.end(), cursor.file_id),
            left: Box::new(self.left),
            right,
        })
    }
}

impl Parse for CallExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `(`

        let arguments = parse_list!(cursor, "call arguments list", Token![')'], || {
            ExpressionParser::default().parse_with(cursor)
        });

        cursor.next_token();

        Some(UntypedExpression::Call {
            span: Span::new(self.start, cursor.current.span.end(), cursor.file_id),
            left: Box::new(self.left),
            arguments,
        })
    }
}

impl Parse for BinaryExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let operator_token = cursor.next;
        let operator: BinaryOperator = BinaryOperator {
            span: operator_token.span,
            raw: RawBinaryOperator::from(operator_token.raw),
        };
        let precedence = cursor.next.raw.to_precedence();

        cursor.next_token();

        let right = ExpressionParser {
            precedence,
            ignore_struct: self.ignore_struct,
        }
        .parse_with(cursor)?;

        Some(UntypedExpression::Binary {
            span: Span::new(self.start, cursor.current.span.end(), cursor.file_id),
            left: Box::new(self.left),
            right: Box::new(right),
            operator,
        })
    }
}

impl Parse for ListExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();

        let start = cursor.next.span.start();

        let elements = parse_list!(cursor, "list expression", Token![']'], || {
            ExpressionParser::default().parse_with(cursor)
        });

        cursor.next_token();

        Some(UntypedExpression::List {
            span: Span::new(start, cursor.current.span.end(), cursor.file_id),
            elements,
        })
    }
}

impl Parse for TupleExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = cursor.next.span.start();
        cursor.next_token(); // `#`

        cursor.consume(Token!['('], "tuple expression")?;

        let elements = parse_list!(cursor, "tuple expression", Token![')'], || {
            ExpressionParser::default().parse_with(cursor)
        });

        cursor.next_token(); // `)`

        Some(UntypedExpression::Tuple {
            span: Span::new(start, cursor.current.span.end(), cursor.file_id),
            elements,
        })
    }
}

impl Parse for StructExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `{`

        let fields = parse_list!(cursor, "struct expression", Token!['}'], || {
            StructExpressionUnitParser.parse_with(cursor)
        });

        cursor.next_token(); // `}`

        Some(UntypedExpression::Struct {
            span: Span::new(self.start, cursor.current.span.end(), cursor.file_id),
            left: Box::new(self.left),
            fields,
        })
    }
}

impl Parse for StructExpressionUnitParser {
    type Output = Option<StructExpressionItem>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let name = cursor.consume_identifier("struct field")?;

        let value = if cursor.next.raw == Token![:] {
            cursor.next_token();
            Some(ExpressionParser::default().parse_with(cursor)?)
        } else {
            None
        };

        Some(StructExpressionItem { name, value })
    }
}

impl Parse for StatementsBlockExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = cursor.next.span.start();
        let block = StatementsBlockParser.parse_with(cursor)?;

        Some(UntypedExpression::StatementsBlock {
            span: Span::new(start, cursor.current.span.end(), cursor.file_id),
            block,
        })
    }
}

impl Parse for FunctionExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = cursor.next.span.start();
        cursor.next_token(); // `|`

        let parameters = parse_list!(cursor, "function expression parameters", Token![|], || {
            FunctionParameterParser.parse_with(cursor)
        });

        cursor.next_token();

        let return_type = if cursor.next.raw == Token![:] {
            cursor.next_token();

            Some(TypeParser.parse_with(cursor)?)
        } else {
            None
        };

        let block = StatementsBlockParser.parse_with(cursor)?;

        Some(UntypedExpression::Function {
            span: Span::new(start, cursor.current.span.end(), cursor.file_id),
            parameters,
            return_type,
            block,
        })
    }
}
