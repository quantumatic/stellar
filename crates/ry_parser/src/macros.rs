macro_rules! parse_list {
    (
        $state:expr,
        node_name: $node_name:expr,
        closing_token: one_of($closing_token1:expr, $closing_token2:expr),
        parse_element_expr: $expr:expr) => {{
        let mut result = vec![];

        if $state.next_token.raw != $closing_token1 && $state.next_token.raw != $closing_token2 {
            loop {
                if let Some(e) = $expr {
                    result.push(e);
                }

                #[allow(unused_qualifications)]
                if $state.next_token.raw != $closing_token1
                    && $state.next_token.raw != $closing_token2
                {
                    if $state.next_token.raw != Punctuator::Comma {
                        use crate::diagnostics::UnexpectedTokenDiagnostic;

                        $state.add_diagnostic(UnexpectedTokenDiagnostic::new(
                            $state.next_token,
                            $crate::expected!($closing_token1, $closing_token2, Punctuator::Comma),
                            $node_name,
                        ));
                        break;
                    }

                    $state.advance();

                    if $state.next_token.raw == $closing_token1
                        || $state.next_token.raw == $closing_token2
                    {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        result
    }};
    (
        $state:expr,
        node_name: $node_name:expr,
        closing_token: $closing_token:expr,
        parse_element_expr: $expr:expr) => {{
        let mut result = vec![];

        if $state.next_token.raw != $closing_token {
            loop {
                result.push($expr?);

                if $state.next_token.raw != $closing_token {
                    if $state.next_token.raw != Punctuator::Comma {
                        use crate::diagnostics::UnexpectedTokenDiagnostic;

                        $state.add_diagnostic(UnexpectedTokenDiagnostic::new(
                            $state.next_token,
                            $crate::expected!($closing_token, Punctuator::Comma),
                            $node_name,
                        ));
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
    }};
}

pub(crate) use parse_list;
