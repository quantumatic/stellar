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

            if !matches!($p.next.inner, $closing_token) {
                loop {
                    result.push($fn($($fn_arg)*)?);

                    if !matches!($p.next.inner, $closing_token) {
                        if $top_level {
                            $p.consume_with_docstring(Punctuator(Comma), $name)?;
                        } else {
                            $p.consume(Punctuator(Comma), $name)?;
                        }

                        if matches!($p.next.inner, $closing_token) {
                            break;
                        }
                    } else {
                        break;
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

pub(crate) use {binop_pattern, parse_list, postfixop_pattern, prefixop_pattern};

#[cfg(test)]
pub(crate) use parser_test;
