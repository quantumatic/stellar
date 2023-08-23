//! Provides a [`pluralize`] function to convert singular nouns to plural ones.

use once_cell::sync::Lazy;
use ry_fx_hash::FxHashSet;

static VOWELS: Lazy<FxHashSet<char>> = Lazy::new(|| "aeiouAEIOU".chars().collect());

/// Converts a given singular noun into a plural noun.
///
/// # Panics
/// If the given noun string is empty.
#[inline(always)]
#[must_use]
pub fn pluralize(noun: impl AsRef<str>) -> String {
    // TODO: check for irregular nouns later.
    let noun = noun.as_ref();

    if VOWELS.contains(noun.chars().last().as_ref().expect("The noun is empty")) {
        format!("{noun}s")
    } else {
        format!("{noun}es")
    }
}
