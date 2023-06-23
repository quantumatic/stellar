macro_rules! parse_list {
    (
        $iterator:ident,
        $node_name:expr,
        $closing_token:expr,
        $blck:block) => {
        {
            let mut result = vec![];

            if $iterator.next_token.raw != $closing_token {
                loop {
                    result.push($blck?);

                    #[allow(unused_qualifications)]
                    if $iterator.next_token.raw != $closing_token {
                        if $iterator.next_token.raw != Token![,] {
                            $iterator.diagnostics.push(
                                ParseDiagnostic::UnexpectedTokenError {
                                    got: $iterator.next_token.clone(),
                                    expected: expected!($closing_token, Token![,]),
                                    node: $node_name.to_owned(),
                                }
                                .build(),
                            );
                            break;
                        }

                        $iterator.advance();

                        if $iterator.next_token.raw == $closing_token {
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
        $iterator:ident,
        $node_name:expr,
        ($closing_token1:expr) or ($closing_token2:expr),
        $blck:block) => {
        {
            let mut result = vec![];

            if $iterator.next_token.raw != $closing_token1 &&
                $iterator.next_token.raw != $closing_token2 {
                loop {
                    if let Some(e) = $blck {
                        result.push(e);
                    }

                    #[allow(unused_qualifications)]
                    if $iterator.next_token.raw != $closing_token1
                        && $iterator.next_token.raw != $closing_token2 {
                        if $iterator.next_token.raw != Token![,] {
                            $iterator.diagnostics.push(
                                ParseDiagnostic::UnexpectedTokenError {
                                    got: $iterator.next_token.clone(),
                                    expected: expected!($closing_token1, $closing_token2, Token![,]),
                                    node: $node_name.to_owned(),
                                }
                                .build(),
                            );
                            break;
                        }

                        $iterator.advance();

                        if $iterator.next_token.raw == $closing_token1
                            || $iterator.next_token.raw == $closing_token2 {
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
