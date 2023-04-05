macro_rules! consume {
    ($p:ident, $expected:expr, $node:expr) => {
        if $p.next.unwrap().is($expected) {
            $p.advance()?;
        } else {
            return Err(ParseError::unexpected_token(
                $p.next.clone(),
                $expected,
                $node,
            ));
        }
    };
    (with_docstring $p:ident, $expected:expr, $node:expr) => {
        if $p.next.unwrap().is($expected) {
            $p.advance_with_docstring()?;
        } else {
            return Err(ParseError::unexpected_token(
                $p.next.clone(),
                $expected,
                $node,
            ));
        }
    };
}

macro_rules! consume_ident {
    ($p:ident, $for:expr) => {{
        if let Identifier(i) = $p.next.unwrap() {
            let identifier = *i;

            $p.advance()?;

            identifier.at($p.current.span())
        } else {
            return Err(ParseError::unexpected_token(
                $p.next.clone(),
                "identifier",
                $for,
            ));
        }
    }};
}

#[cfg(test)]
macro_rules! parser_test {
    ($name:ident, $source:literal) => {
        #[test]
        fn $name() {
            let mut string_interner = Interner::default();
            let mut parser = Parser::new($source, &mut string_interner);
            assert!(parser.parse().is_ok());
        }
    };
}

macro_rules! parse_list {
    ($p:ident, $name:literal, $closing_token:pat, $top_level:expr, $fn:expr) => {
        parse_list!($p, $name, $closing_token, $top_level, $fn, )
    };
    ($p:ident, $name:literal, $closing_token:pat, $top_level:expr, $fn:expr, $($fn_arg:expr)*) => {
        {
            let mut result = vec![];

            if let $closing_token = $p.next.unwrap() {

            } else {
                loop {
                    result.push($fn($($fn_arg)*)?);

                    if let $closing_token = $p.next.unwrap() {
                        break
                    } else {
                        if $top_level {
                            consume!(with_docstring $p, Punctuator(Comma), $name);
                        } else {
                            consume!($p, Punctuator(Comma), $name);
                        }

                        if let $closing_token = $p.next.unwrap() {
                            break
                        }
                    }
                }
            }

            result
        }
    };
}

macro_rules! binop_pattern {
    () => {
        Punctuator(
            Plus | Minus
                | Asterisk
                | Slash
                | Eq
                | NotEq
                | LessThan
                | LessThanOrEq
                | GreaterThan
                | GreaterThanOrEq
                | Assign
                | OrEq
                | XorEq
                | PlusEq
                | MinusEq
                | SlashEq
                | AsteriskEq
                | AsteriskAsterisk
                | Percent
                | And
                | Xor
                | Or
                | OrOr
                | Elvis
                | AndAnd
                | LeftShift
                | RightShift,
        )
    };
}

macro_rules! postfixop_pattern {
    () => {
        Punctuator(QuestionMark | PlusPlus | MinusMinus)
    };
}

macro_rules! prefixop_pattern {
    () => {
        Punctuator(Bang | Not | PlusPlus | MinusMinus | Minus | Plus)
    };
}

pub(crate) use {
    binop_pattern, consume, consume_ident, parse_list, postfixop_pattern, prefixop_pattern,
};

#[cfg(test)]
pub(crate) use parser_test;
