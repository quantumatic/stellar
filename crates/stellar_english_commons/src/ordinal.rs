//! Allows to work with ordinals.

use std::fmt::Display;

/// Newtype wrapper struct that formats integers as an ordinal number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ordinal<T>(T)
where
    T: Display;

impl<T> Ordinal<T>
where
    T: Display,
{
    /// Returns the suffix for the ordinal.
    ///
    /// # Example
    ///
    /// ```
    /// use stellar_english_commons::ordinal::ordinal;
    ///
    /// assert_eq!(ordinal(2).suffix(), "nd");
    /// assert_eq!(ordinal(1).suffix(), "st");
    /// ```
    #[must_use]
    pub fn suffix(&self) -> &'static str {
        let s = self.0.to_string();

        if s.ends_with('1') && !s.ends_with("11") {
            "st"
        } else if s.ends_with('2') && !s.ends_with("12") {
            "nd"
        } else if s.ends_with('3') && !s.ends_with("13") {
            "rd"
        } else {
            "th"
        }
    }
}

impl<T> Display for Ordinal<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0, self.suffix())
    }
}

/// Constructs a new instance of [`Ordinal`].
#[inline(always)]
#[must_use]
pub const fn ordinal<T>(n: T) -> Ordinal<T>
where
    T: Display,
{
    Ordinal(n)
}

#[cfg(test)]
mod tests {
    use crate::ordinal::ordinal;

    #[test]
    fn test_display() {
        let test_cases = vec![
            (-4, "-4th"),
            (-3, "-3rd"),
            (-2, "-2nd"),
            (-1, "-1st"),
            (0, "0th"),
            (1, "1st"),
            (2, "2nd"),
            (3, "3rd"),
            (4, "4th"),
            (10, "10th"),
            (11, "11th"),
            (12, "12th"),
            (13, "13th"),
            (14, "14th"),
            (20, "20th"),
            (21, "21st"),
            (22, "22nd"),
            (23, "23rd"),
            (24, "24th"),
            (100, "100th"),
            (101, "101st"),
            (102, "102nd"),
            (103, "103rd"),
            (104, "104th"),
            (110, "110th"),
            (111, "111th"),
            (112, "112th"),
            (113, "113th"),
            (114, "114th"),
            (120, "120th"),
            (121, "121st"),
            (122, "122nd"),
            (123, "123rd"),
            (124, "124th"),
        ];

        for case in test_cases {
            assert_eq!(ordinal(case.0).to_string(), case.1);
        }
    }
}
