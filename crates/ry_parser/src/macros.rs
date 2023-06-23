macro_rules! parse_list {
    (
        $iterator:ident,
        $node_name:expr,
        $closing_token:expr,
        $fn:expr) => {
        {
            let mut result = vec![];

            if $iterator.next.raw != $closing_token {
                loop {
                    result.push($fn()?);

                    #[allow(unused_qualifications)]
                    if $iterator.next.raw != $closing_token {
                        if $iterator.next.raw != Token![,] {
                            $iterator.diagnostics.push(
                                ParseDiagnostic::UnexpectedTokenError {
                                    got: $iterator.next.clone(),
                                    expected: expected!($closing_token, Token![,]),
                                    node: $node_name.to_owned(),
                                }
                                .build(),
                            );
                            break;
                        }

                        $iterator.next_token();

                        if $iterator.next.raw == $closing_token {
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
        $fn:expr) => {
        {
            let mut result = vec![];

            if $iterator.next.raw != $closing_token1 &&
                $iterator.next.raw != $closing_token2 {
                loop {
                    result.push($fn()?);

                    #[allow(unused_qualifications)]
                    if $iterator.next.raw != $closing_token1
                        && $iterator.next.raw != $closing_token2 {
                        if $iterator.next.raw != Token![,] {
                            $iterator.diagnostics.push(
                                ParseDiagnostic::UnexpectedTokenError {
                                    got: $iterator.next.clone(),
                                    expected: expected!($closing_token1, $closing_token2, Token![,]),
                                    node: $node_name.to_owned(),
                                }
                                .build(),
                            );
                            break;
                        }

                        $iterator.next_token();

                        if $iterator.next.raw == $closing_token1
                            || $iterator.next.raw == $closing_token2 {
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
