//! Provides utility functions for working with enumerations.
use std::fmt::Display;

/// Formats a list of strings in such a format:
///
/// ```
/// use stellar_english_commons::enumeration::one_of;
///
/// assert_eq!(one_of(["a"].iter()), "a".to_owned());
/// assert_eq!(one_of(["a", "b"].iter()), "a or b".to_owned());
/// assert_eq!(one_of(["a", "b", "c"].iter()), "a, b, or c".to_owned());
/// ```
#[allow(single_use_lifetimes)]
#[must_use]
pub fn one_of(list: impl ExactSizeIterator<Item = impl Display>) -> String {
    let len = list.len();

    list.enumerate()
        .map(|(idx, token)| {
            format!(
                "{}{token}",
                if idx == 0 {
                    ""
                } else if idx == len - 1 {
                    if len == 2 {
                        " or "
                    } else {
                        ", or "
                    }
                } else {
                    ", "
                }
            )
        })
        .collect::<String>()
}

/// Formats a list of strings in such a format:
///
/// ```
/// use stellar_english_commons::enumeration::all_of;
///
/// assert_eq!(all_of(["a"].iter()), "a".to_owned());
/// assert_eq!(all_of(["a", "b"].iter()), "a and b".to_owned());
/// assert_eq!(all_of(["a", "b", "c"].iter()), "a, b, and c".to_owned());
/// ```
#[allow(single_use_lifetimes)]
#[must_use]
pub fn all_of(list: impl ExactSizeIterator<Item = impl Display>) -> String {
    let len = list.len();

    list.enumerate()
        .map(|(idx, token)| {
            format!(
                "{}{token}",
                if idx == 0 {
                    ""
                } else if idx == len - 1 {
                    if len == 2 {
                        " and "
                    } else {
                        ", and "
                    }
                } else {
                    ", "
                }
            )
        })
        .collect::<String>()
}
