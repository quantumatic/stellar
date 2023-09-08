//! Terminal back-end for emitting diagnostics.

use std::str::FromStr;

use stellar_filesystem::in_memory_file_storage::InMemoryFileStorage;
use termcolor::{ColorChoice, WriteColor};

use crate::diagnostic::Diagnostic;

mod config;
mod renderer;
mod views;

pub use termcolor;

pub use self::config::{Chars, Config, DisplayStyle, Styles};

/// A command line argument that configures the coloring of the output.
///
/// This can be used with command line argument parsers like [`clap`] or [`structopt`].
///
/// [`clap`]: https://crates.io/crates/clap
/// [`structopt`]: https://crates.io/crates/structopt
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ColorArg(pub ColorChoice);

impl ColorArg {
    /// Allowed values the argument.
    ///
    /// This is useful for generating documentation via [`clap`] or `structopt`'s
    /// `possible_values` configuration.
    ///
    /// [`clap`]: https://crates.io/crates/clap
    /// [`structopt`]: https://crates.io/crates/structopt
    pub const VARIANTS: &'static [&'static str] = &["auto", "always", "ansi", "never"];
}

impl FromStr for ColorArg {
    type Err = &'static str;

    fn from_str(src: &str) -> Result<Self, &'static str> {
        match src {
            _ if src.eq_ignore_ascii_case("auto") => Ok(Self(ColorChoice::Auto)),
            _ if src.eq_ignore_ascii_case("always") => Ok(Self(ColorChoice::Always)),
            _ if src.eq_ignore_ascii_case("ansi") => Ok(Self(ColorChoice::AlwaysAnsi)),
            _ if src.eq_ignore_ascii_case("never") => Ok(Self(ColorChoice::Never)),
            _ => Err("valid values: auto, always, ansi, never"),
        }
    }
}

impl From<ColorArg> for ColorChoice {
    fn from(x: ColorArg) -> Self {
        x.0
    }
}

/// Emit a diagnostic using the given writer, context, config, and files.
///
/// # Errors
/// Error case can arise if:
/// * a file was removed from the file database.
/// * a file was changed so that it is too small to have an index
/// * IO fails
pub fn emit(
    writer: &mut dyn WriteColor,
    config: &Config,
    in_memory_file_storage: &InMemoryFileStorage,
    diagnostic: &Diagnostic,
) -> Result<(), super::files::Error> {
    use self::renderer::Renderer;
    use self::views::{RichDiagnostic, ShortDiagnostic};

    let mut renderer = Renderer::new(writer, config);
    match config.display_style {
        DisplayStyle::Rich => {
            RichDiagnostic::new(diagnostic, config).render(in_memory_file_storage, &mut renderer)
        }
        DisplayStyle::Medium => {
            ShortDiagnostic::new(diagnostic, true).render(in_memory_file_storage, &mut renderer)
        }
        DisplayStyle::Short => {
            ShortDiagnostic::new(diagnostic, false).render(in_memory_file_storage, &mut renderer)
        }
    }
}
