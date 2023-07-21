macro_rules! parse_list {
    (
        $state:expr,
        $node_name:expr,
        $closing_token:expr,
        $blck:block) => {
        {
            let mut result = vec![];

            if $state.next_token.raw != $closing_token {
                loop {
                    result.push($blck?);

                    if $state.next_token.raw != $closing_token {
                        if $state.next_token.raw != Token![,] {
                            use crate::diagnostics::UnexpectedTokenDiagnostic;

                            $state.add_diagnostic(
                                UnexpectedTokenDiagnostic::new(
                                    $state.next_token,
                                    $crate::expected!($closing_token, Token![,]),
                                    $node_name,
                                )
                            );
                            break;
                        }

                        $state.advance();

                        if $state.next_token.raw == $closing_token {
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
        $state:expr,
        $node_name:expr,
        ($closing_token1:expr) or ($closing_token2:expr),
        $blck:block) => {
        {
            let mut result = vec![];

            if $state.next_token.raw != $closing_token1 &&
                $state.next_token.raw != $closing_token2 {
                loop {
                    if let Some(e) = $blck {
                        result.push(e);
                    }

                    #[allow(unused_qualifications)]
                    if $state.next_token.raw != $closing_token1
                        && $state.next_token.raw != $closing_token2 {
                        if $state.next_token.raw != Token![,] {
                            use crate::diagnostics::UnexpectedTokenDiagnostic;

                            $state.add_diagnostic(
                                UnexpectedTokenDiagnostic::new(
                                    $state.next_token,
                                    $crate::expected!($closing_token1, $closing_token2, Token![,]),
                                    $node_name,
                                )
                            );
                            break;
                        }

                        $state.advance();

                        if $state.next_token.raw == $closing_token1
                            || $state.next_token.raw == $closing_token2 {
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
