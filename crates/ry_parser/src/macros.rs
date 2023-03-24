macro_rules! consume {
    ($p:ident) => {
        $p.advance()?;
    };
    (with_docstring $p:ident) => {
        $p.advance_with_docstring()?;
    };
    ($p:ident, $expected:expr, $for:expr) => {
        if $p.next.value.is($expected) {
            $p.advance()?;
        } else {
            return Err(ParserError::UnexpectedToken(
                $p.next.clone(),
                format!("{}", $expected),
                $for.into(),
            ));
        }
    };
    (with_docstring $p:ident, $expected:expr, $for:expr) => {
        if $p.next.value.is($expected) {
            $p.advance_with_docstring()?;
        } else {
            return Err(ParserError::UnexpectedToken(
                $p.next.clone(),
                format!("{}", $expected),
                $for.into(),
            ));
        }
    };
}

macro_rules! consume_ident {
    ($p:ident, $for:expr) => {
        if let Identifier(i) = $p.next.value {
            $p.advance()?;
            i.with_span($p.current.span)
        } else {
            return Err(ParserError::UnexpectedToken(
                $p.next.clone(),
                "identifier".to_owned(),
                $for.into(),
            ));
        }
    };
}

#[cfg(test)]
macro_rules! parser_test {
    ($name:ident, $source:literal) => {
        #[test]
        pub fn $name() {
            let mut string_interner = StringInterner::default();
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

            if let $closing_token = $p.next.value {

            } else {
                loop {
                    result.push($fn($($fn_arg)*)?);

                    if let $closing_token = $p.next.value {
                        break
                    } else {
                        if $top_level {
                            consume!(with_docstring $p, Comma, $name);
                        } else {
                            consume!($p, Comma, $name);
                        }

                        if let $closing_token = $p.next.value {
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
            | RightShift
    };
}

macro_rules! postfixop_pattern {
    () => {
        QuestionMark | PlusPlus | MinusMinus | BangBang
    };
}

macro_rules! prefixop_pattern {
    () => {
        Bang | Not | PlusPlus | MinusMinus | Minus | Plus
    };
}

pub(crate) use {
    binop_pattern, consume, consume_ident, parse_list, postfixop_pattern, prefixop_pattern,
};

#[cfg(test)]
pub(crate) use parser_test;
