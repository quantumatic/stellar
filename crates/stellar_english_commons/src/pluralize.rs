//! Provides a [`PluralizeExt`] trait to convert singular nouns to plural ones.

use once_cell::sync::Lazy;
use stellar_fx_hash::{FxHashMap, FxHashSet};
use stellar_stable_likely::unlikely;

static VOWELS: Lazy<FxHashSet<char>> = Lazy::new(|| "aeiouAEIOU".chars().collect());
static IRREGULAR_NOUNS: Lazy<FxHashMap<&'static str, &'static str>> =
    Lazy::new(|| FxHashMap::from_iter([("index", "indices"), ("analysis", "analyses")]));

/// Provides [`PluralizeExt::pluralize()`] method, to convert a given singular
/// noun into a plural noun.
pub trait PluralizeExt {
    /// Converts a given singular noun into a plural noun.
    ///
    /// # Panics
    /// If the given noun string is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use stellar_english_commons::pluralize::PluralizeExt;
    ///
    /// assert_eq!("cat".pluralize(), "cats".to_owned());
    /// assert_eq!("bee".to_owned().pluralize(), "bees".to_owned());
    /// assert_eq!("index".pluralize(), "indices".to_owned());
    /// ```
    fn pluralize(self) -> String;
}

impl<S> PluralizeExt for S
where
    S: Into<String>,
{
    fn pluralize(self) -> String {
        let mut noun: String = self.into();

        if unlikely(noun.is_empty()) {
            String::new()
        } else if let Some(pluralized) = IRREGULAR_NOUNS.get(&*noun) {
            (*pluralized).to_owned()
        } else {
            let last_char = noun
                .chars()
                .last()
                .unwrap_or_else(|| panic!("The noun is empty"));
            let penultimate_char = noun.chars().nth(noun.len() - 2);

            if last_char == 'y' {
                if VOWELS
                    .contains(&penultimate_char.unwrap_or_else(|| panic!("Word 'y' is not valid")))
                {
                    format!("{noun}s") // toy -> toys
                } else {
                    noun.pop(); // candy -> cand
                    format!("{noun}ies") // cand -> candies
                }
            } else if last_char == 'f' {
                noun.pop(); // loaf -> loa
                format!("{noun}ves") // loa -> loaves
            } else if last_char == 'e' && penultimate_char == Some('f') {
                noun.pop();
                noun.pop(); // knife -> kni
                format!("{noun}ves") // kni -> knives
            } else {
                format!("{noun}s") // cat -> cats
            }
        }
    }
}
