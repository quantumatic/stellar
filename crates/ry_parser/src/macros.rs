#[cfg(test)]
macro_rules! parser_test {
    ($parser: ty, $name:ident, $source:literal) => {
        #[test]
        fn $name() {
            let mut string_interner = Interner::default();
            let mut parser_state = ParserState::new($source, &mut string_interner);
            assert!(<$parser>::default().parse_with(&mut parser_state).is_ok());
        }
    };
}

macro_rules! parse_list {
    ($p:ident, $name:literal, $closing_token:pat, $fn:expr) => {
        parse_list!($p, $name, $closing_token, $fn, )
    };
    ($p:ident, $name:literal, $closing_token:pat, $fn:expr, $($fn_arg:expr)*) => {
        {
            let mut result = vec![];

            if !matches!($p.next.inner, $closing_token) {
                loop {
                    result.push($fn($($fn_arg)*)?);

                    #[allow(unused_qualifications)]
                    if !matches!($p.next.inner, $closing_token) {
                        $p.consume(Punctuator(ry_ast::token::Punctuator::Comma), $name)?;

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
