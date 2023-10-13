//! Provides utility functions for working with enumerations.
use std::fmt::Display;

/// Formats a list of strings in such a format:
///
/// ```
/// use stellar_english_commons::enumeration::one_of;
///
/// assert_eq!(one_of(["a"]), "a".to_owned());
/// assert_eq!(one_of(["a", "b"]), "a or b".to_owned());
/// assert_eq!(one_of(["a", "b", "c"]), "a, b, or c".to_owned());
/// ```
#[must_use]
pub fn one_of<I, S>(iter: I) -> String
where
    I: IntoIterator,
    I::IntoIter: ExactSizeIterator<Item = S>,
    S: Display,
{
    let iter = iter.into_iter();
    let len = iter.len();

    iter.enumerate().fold(String::new(), |acc, (idx, item)| {
        format!(
            "{acc}{}{item}",
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
}

/// Formats a list of strings in such a format:
///
/// ```
/// use stellar_english_commons::enumeration::all_of;
///
/// assert_eq!(all_of(["a"]), "a".to_owned());
/// assert_eq!(all_of(["a", "b"]), "a and b".to_owned());
/// assert_eq!(all_of(["a", "b", "c"]), "a, b, and c".to_owned());
/// ```
#[must_use]
pub fn all_of<I, S>(iter: I) -> String
where
    I: IntoIterator,
    I::IntoIter: ExactSizeIterator<Item = S>,
    S: Display,
{
    let iter = iter.into_iter();
    let len = iter.len();

    iter.enumerate().fold(String::new(), |acc, (idx, item)| {
        format!(
            "{acc}{}{item}",
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
}
