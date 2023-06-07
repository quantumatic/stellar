#[cfg(test)]
macro_rules! parse_test {
    ($parser: expr, $name:ident, $source:literal) => {
        #[test]
        #[allow(unused_qualifications)]
        fn $name() {
            let mut diagnostics = vec![];
            let mut string_interner = ry_interner::Interner::default();
            let mut cursor = crate::Cursor::new(0, $source, &mut string_interner, &mut diagnostics);
            let node = crate::Parse::parse_with($parser, &mut cursor);
            assert!(node.is_some());
        }
    };
}

macro_rules! parse_list {
    (
        $cursor:ident,
        $node_name:literal,
        $closing_token:expr,
        $fn:expr) => {
        {
            let mut result = vec![];

            if $cursor.next.unwrap() != &$closing_token {
                loop {
                    result.push($fn()?);

                    #[allow(unused_qualifications)]
                    if $cursor.next.unwrap() != &$closing_token {
                        if $cursor.next.unwrap() != &Token![,] {
                            $cursor.diagnostics.push(
                                ParseDiagnostic::UnexpectedTokenError {
                                    got: $cursor.next.clone(),
                                    expected: expected!($closing_token, Token![,]),
                                    node: $node_name.to_owned(),
                                }
                                .build(),
                            );
                            break;
                        } else {
                            $cursor.next_token();
                        }

                        if $cursor.next.unwrap() == &$closing_token {
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
    (
        $cursor:ident,
        $node_name:literal,
        ($closing_token1:expr) or ($closing_token2:expr),
        $fn:expr) => {
        {
            let mut result = vec![];

            if $cursor.next.unwrap() != &$closing_token1 &&
                $cursor.next.unwrap() != &$closing_token2 {
                loop {
                    result.push($fn()?);

                    #[allow(unused_qualifications)]
                    if $cursor.next.unwrap() != &$closing_token1
                        && $cursor.next.unwrap() != &$closing_token2 {
                        if $cursor.next.unwrap() != &Token![,] {
                            $cursor.diagnostics.push(
                                ParseDiagnostic::UnexpectedTokenError {
                                    got: $cursor.next.clone(),
                                    expected: expected!($closing_token1, $closing_token2, Token![,]),
                                    node: $node_name.to_owned(),
                                }
                                .build(),
                            );
                            break;
                        } else {
                            $cursor.next_token();
                        }

                        if $cursor.next.unwrap() == &$closing_token1
                            || $cursor.next.unwrap() == &$closing_token2 {
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

pub(crate) use parse_list;

#[cfg(test)]
pub(crate) use parse_test;
