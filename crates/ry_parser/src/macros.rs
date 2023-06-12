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
                        }

                        $cursor.next_token();

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
                        }

                        $cursor.next_token();

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
