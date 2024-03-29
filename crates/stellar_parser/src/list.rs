use std::iter;

use stellar_ast::token::{Punctuator, RawToken};
use stellar_english_commons::enumeration::one_of;

use crate::{Parse, ParseState};

pub(crate) struct ListParser<'a, P, E>
where
    P: for<'s, 'd> Fn(&mut ParseState<'s, 'd>) -> Option<E>,
{
    closing_tokens: &'a [RawToken],
    parse_element_fn: P,
}

impl<'a, P, E> ListParser<'a, P, E>
where
    P: for<'s, 'd> Fn(&mut ParseState<'s, 'd>) -> Option<E>,
{
    #[must_use]
    pub(crate) const fn new(closing_tokens: &'a [RawToken], parse_element_fn: P) -> Self {
        Self {
            closing_tokens,
            parse_element_fn,
        }
    }
}

impl<P, E> Parse for ListParser<'_, P, E>
where
    P: for<'s, 'd> Fn(&mut ParseState<'s, 'd>) -> Option<E>,
{
    type Output = Option<Vec<E>>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        let mut result = vec![];

        // For instance: `(` `)` - empty list.
        if self.closing_tokens.contains(&state.next_token.raw) {
            return Some(result);
        }

        loop {
            // `(` element
            if let Some(element) = (self.parse_element_fn)(state) {
                result.push(element);
            } else {
                return None;
            }

            // `(` element `)`
            if self.closing_tokens.contains(&state.next_token.raw) {
                break;
            }

            // `(` element `?` (invalid token)
            if state.next_token.raw != Punctuator::Comma {
                #[allow(clippy::needless_collect)]
                state.add_unexpected_token_diagnostic(one_of(
                    self.closing_tokens
                        .iter()
                        .map(ToString::to_string)
                        .chain(iter::once("`,`".to_owned()))
                        .collect::<Vec<_>>(),
                ));

                return None;
            }

            // `(` element `,`

            state.advance();

            // `(` element `,` `)`
            if self.closing_tokens.contains(&state.next_token.raw) {
                break;
            }
        }

        Some(result)
    }
}
