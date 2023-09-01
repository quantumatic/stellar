//! Defines diagnostics for parser.

use stellar_ast::token::{LexError, Token};
use stellar_diagnostics::{define_diagnostics, diagnostic::Diagnostic};
use stellar_diagnostics::{BuildDiagnostic, LocationExt};
use stellar_filesystem::location::{ByteOffset, Location};
use stellar_interner::PathID;

/// Context in which the unnecessary visibility qualifier error is found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnnecessaryVisibilityQualifierContext {
    /// ```stellar
    /// pub interface F {
    ///     pub fun t() {}
    ///     ^^^
    /// }
    /// ```
    InterfaceMethod {
        /// Location of a method name.
        name_location: Location,
    },

    /// ```stellar
    /// pub import ...;
    /// ^^^
    /// ```
    Import,
}

define_diagnostics! {
    /// Diagnostic related to an error occured when tokenizing.
    diagnostic(error) LexErrorDiagnostic(self, error: LexError) {
        code { "E000" }
        message { format!("{}", self.error.raw) }
        labels {
            primary self.error.location => {""}
        }
        notes {}
    }

    /// Diagnostic related to an integer overflow error.
    diagnostic(error) IntegerOverflow(self, location: Location) {
        code { "E002" }
        message { "unexpected integer overflow" }
        labels {
            primary self.location => {"error appeared when parsing this integer"}
        }
        notes {
            "note: integer cannot exceed the maximum value of `u64` (u64.max() == 18_446_744_073_709_551_615)"
            "note: you can use exponent to do so, but be careful!"
        }
    }

    /// Diagnostic related to an float overflow error.
    diagnostic(error) FloatOverflow(self, location: Location) {
        code { "E003" }
        message { "unexpected float overflow" }
        labels {
            primary self.location => {"error appeared when parsing this float literal"}
        }
        notes {
            "note: float cannot exceed the maximum value of `f64` (f64.max() == 1.7976931348623157e+308)"
            "note: you can use exponent to do so, but be careful!"
        }
    }

    /// Diagnostic related to an unexpected token error.
    diagnostic(error) UnexpectedToken(
        self,
        offset: ByteOffset,
        got: Token,
        expected: String
    ) {
        code { "E001" }
        message { format!("expected {}, found {}", self.expected, self.got.raw) }
        labels {
            primary self.offset.next_byte_location_at(self.got.location.filepath) => {
                format!("expected {}", self.expected)
            },
            secondary self.got.location => { "unexpected token" }
        }
        notes {}
    }
}

/// Diagnostic related to an unnecessary visibility qualifier error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnnecessaryVisibilityQualifierDiagnostic {
    /// Location of `pub`.
    pub location: Location,

    /// Context in which the error is found.
    pub context: UnnecessaryVisibilityQualifierContext,
}

impl BuildDiagnostic for UnnecessaryVisibilityQualifierDiagnostic {
    #[inline(always)]
    fn build(self) -> Diagnostic<PathID> {
        let mut labels = vec![self
            .location
            .to_primary_label()
            .with_message("consider removing this `pub`")];

        if let UnnecessaryVisibilityQualifierContext::InterfaceMethod { name_location } =
            self.context
        {
            labels.push(
                name_location
                    .to_secondary_label()
                    .with_message("happened when analyzing the interface method"),
            );
        }

        Diagnostic::error()
            .with_message("unnecessary visibility qualifier".to_owned())
            .with_code("E004")
            .with_labels(labels)
            .with_notes(match self.context {
                UnnecessaryVisibilityQualifierContext::InterfaceMethod { .. } => {
                    vec![
                        "note: using `pub` for interface method will not make the method public"
                            .to_owned(),
                        "note: all interface methods are public by default".to_owned(),
                    ]
                }
                UnnecessaryVisibilityQualifierContext::Import => {
                    vec!["note: using `pub` will not make the import public.".to_owned()]
                }
            })
    }
}
