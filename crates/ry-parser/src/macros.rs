macro_rules! check_token {
    ($p:ident, $expected:expr, $expected_for:literal) => {
        if let Invalid(e) = $p.current.value {
            Err(ParserError::ErrorToken((e, $p.current.span.clone()).into()))
        } else if !&$p.current.value.is($expected) {
            Err(ParserError::UnexpectedToken(
                $p.current.clone(),
                format!("{}", $expected),
                Some($expected_for.to_owned()),
            ))
        } else {
            Ok(())
        }
    };
}

macro_rules! check_token0 {
    ($p:ident, $t_dump:expr, $expected:pat, $expected_for:expr) => {
        if let Invalid(e) = $p.current.value {
            Err(ParserError::ErrorToken((e, $p.current.span.clone()).into()))
        } else if let $expected = $p.current.value {
            Ok(())
        } else {
            Err(ParserError::UnexpectedToken(
                $p.current.clone(),
                $t_dump.into(),
                Some($expected_for.into()),
            ))
        }
    };
    ($p:ident, $expected_for:expr, $expected:pat) => {
        if let Invalid(e) = $p.current.value {
            Err(ParserError::ErrorToken((e, $p.current.span.clone()).into()))
        } else if let $expected = $p.current.value {
            Ok(())
        } else {
            Err(ParserError::UnexpectedToken(
                $p.current.clone(),
                $expected_for.into(),
                None,
            ))
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
    ($p:ident, $name:literal, $closing_token:expr, $top_level:expr, $fn:expr) => {
        parse_list!($p, $name, $closing_token, $top_level, $fn, )
    };
    ($p:ident, $name:literal, $closing_token:expr, $top_level:expr, $fn:expr, $($fn_arg:expr)*) => {
        {
            let mut result = vec![];

            if !$p.current.value.is($closing_token) {
                loop {
                    result.push($fn($($fn_arg)*)?);

                    if $p.current.value.is($closing_token) {
                        break
                    } else {
                        check_token0!($p, format!("`,` or {}", $closing_token), Comma, $name)?;

                        $p.advance($top_level)?; // ','

                        if $p.current.value.is($closing_token) {
                            break
                        }
                    }
                }
            }

            $p.advance($top_level)?; // $closing_token

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

pub(crate) use {binop_pattern, check_token, check_token0, parse_list, postfixop_pattern};

#[cfg(test)]
pub(crate) use parser_test;
