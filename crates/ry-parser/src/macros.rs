macro_rules! check_token {
    ($p: ident, $expected: expr, $expected_for: literal) => {
        if let RawToken::Invalid(e) = $p.current.value {
            Err(ParserError::ErrorToken((e, $p.current.span.clone()).into()))
        } else if !&$p.current.value.is($expected) {
            Err(ParserError::UnexpectedTokenExpectedX(
                $p.current.clone(),
                $expected,
                Some($expected_for.to_owned()),
            ))
        } else {
            Ok(())
        }
    };
}

macro_rules! check_token0 {
    ($p: ident, $t_dump: expr, $expected: pat, $expected_for: expr) => {
        if let RawToken::Invalid(e) = $p.current.value {
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
    ($p: ident, $expected_for: expr, $expected: pat) => {
        if let RawToken::Invalid(e) = $p.current.value {
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

macro_rules! parse_list {
    ($p: ident, $name: literal, $closing_token: expr, $top_level: expr, $fn: expr) => {
        parse_list!($p, $name, $closing_token, $top_level, $fn, )
    };
    ($p: ident, $name: literal, $closing_token: expr, $top_level: expr, $fn: expr, $($fn_arg:expr)*) => {
        {
            let mut result = vec![];

            if !$p.current.value.is($closing_token) {
                loop {
                    result.push($fn($($fn_arg)*)?);

                    if $p.current.value.is($closing_token) {
                        break
                    } else {
                        check_token0!($p, format!("`,` or {}", $closing_token), RawToken::Comma, $name)?;

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
        RawToken::Plus
            | RawToken::Minus
            | RawToken::Asterisk
            | RawToken::Slash
            | RawToken::Eq
            | RawToken::NotEq
            | RawToken::LessThan
            | RawToken::LessThanOrEq
            | RawToken::GreaterThan
            | RawToken::GreaterThanOrEq
            | RawToken::Assign
            | RawToken::OrEq
            | RawToken::XorEq
            | RawToken::PlusEq
            | RawToken::MinusEq
            | RawToken::SlashEq
            | RawToken::AsteriskEq
            | RawToken::AsteriskAsterisk
            | RawToken::Percent
            | RawToken::And
            | RawToken::Xor
            | RawToken::Or
            | RawToken::OrOr
            | RawToken::Elvis
            | RawToken::AndAnd
            | RawToken::LeftShift
            | RawToken::RightShift
    };
}

macro_rules! postfixop_pattern {
    () => {
        RawToken::QuestionMark | RawToken::PlusPlus | RawToken::MinusMinus | RawToken::BangBang
    };
}

pub(crate) use {binop_pattern, check_token, check_token0, parse_list, postfixop_pattern};
