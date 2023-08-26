//! Provides a [`pluralize`] function to convert singular nouns to plural ones.

use once_cell::sync::Lazy;
use ry_fx_hash::{FxHashMap, FxHashSet};
use ry_stable_likely::unlikely;

static VOWELS: Lazy<FxHashSet<char>> = Lazy::new(|| "aeiouAEIOU".chars().collect());
static IRREGULAR_NOUNS: Lazy<FxHashMap<&'static str, &'static str>> =
    Lazy::new(|| FxHashMap::from_iter([("index", "indices"), ("analysis", "analyses")]));

/// Provides [`PluralizeExt::pluralize()`] method, to convert a given singular
/// noun into a plural noun.
trait PluralizeExt {
    /// Converts a given singular noun into a plural noun.
    ///
    /// # Panics
    /// If the given noun string is empty.
    fn pluralize(self) -> String;
}

impl<S> PluralizeExt for S
where
    S: Into<String>,
{
    fn pluralize(self) -> String {
        let noun = self.into();

        if unlikely(noun.is_empty()) {
            String::new()
        } else if let Some(pluralized) = IRREGULAR_NOUNS.get(&*noun) {
            (*pluralized).to_owned()
        } else if VOWELS.contains(noun.chars().last().as_ref().expect("The noun is empty")) {
            format!("{noun}s")
        } else {
            format!("{noun}es")
        }
    }
}
