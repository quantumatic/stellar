//! Diagnostic data structures.

use std::string::ToString;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use stellar_filesystem::location::Location;

/// A severity level for diagnostic messages.
///
/// These are ordered in the following way:
///
/// ```
/// use stellar_diagnostics::diagnostic::Severity;
///
/// assert!(Severity::Bug > Severity::Error);
/// assert!(Severity::Error > Severity::Warning);
/// assert!(Severity::Warning > Severity::Note);
/// assert!(Severity::Note > Severity::Help);
/// ```
#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Severity {
    /// A help message.
    Help,
    /// A note.
    Note,
    /// A warning.
    Warning,
    /// An error.
    Error,
    /// An unexpected bug.
    Bug,
}

/// A style of a diagnostic label.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub enum LabelStyle {
    /// Labels that describe the primary cause of a diagnostic.
    Primary,
    /// Labels that provide additional context for a diagnostic.
    Secondary,
}

/// A label describing an underlined region of code associated with a diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Label {
    /// The style of the label.
    pub style: LabelStyle,
    /// The location of the snippet.
    pub location: Location,
    /// An optional message to provide some additional information for the
    /// underlined code. These should not include line breaks.
    pub message: String,
}

impl Label {
    /// Create a new label.
    #[inline(always)]
    #[must_use]
    pub const fn new(style: LabelStyle, location: Location) -> Self {
        Self {
            style,
            location,
            message: String::new(),
        }
    }

    /// Create a new label with a style of [`LabelStyle::Primary`].
    ///
    /// [`LabelStyle::Primary`]: LabelStyle::Primary
    #[inline(always)]
    #[must_use]
    pub const fn primary(location: Location) -> Self {
        Self::new(LabelStyle::Primary, location)
    }

    /// Create a new label with a style of [`LabelStyle::Secondary`].
    ///
    /// [`LabelStyle::Secondary`]: LabelStyle::Secondary
    #[inline(always)]
    #[must_use]
    pub const fn secondary(location: Location) -> Self {
        Self::new(LabelStyle::Secondary, location)
    }

    /// Add a message to the diagnostic.
    #[inline(always)]
    #[must_use]
    pub fn with_message(mut self, message: impl ToString) -> Self {
        self.message = message.to_string();
        self
    }
}

/// Represents a diagnostic message that can provide information like errors and
/// warnings to the user.
///
/// The position of a Diagnostic is considered to be the position of the [`Label`] that has the earliest starting position and has the highest style which appears in all the labels of the diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Diagnostic {
    /// The overall severity of the diagnostic
    pub severity: Severity,
    /// An optional code that identifies this diagnostic.
    pub code: Option<String>,
    /// The main message associated with this diagnostic.
    ///
    /// These should not include line breaks, and in order support the 'short'
    /// diagnostic display mod, the message should be specific enough to make
    /// sense on its own, without additional context provided by labels and notes.
    pub message: String,
    /// Source labels that describe the cause of the diagnostic.
    /// The order of the labels inside the vector does not have any meaning.
    /// The labels are always arranged in the order they appear in the source code.
    pub labels: Vec<Label>,
    /// Notes that are associated with the primary cause of the diagnostic.
    /// These can include line breaks for improved formatting.
    pub notes: Vec<String>,
}

impl Diagnostic {
    /// Create a new diagnostic.
    #[inline(always)]
    #[must_use]
    pub const fn new(severity: Severity) -> Self {
        Self {
            severity,
            code: None,
            message: String::new(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Create a new diagnostic with a severity of [`Severity::Bug`].
    ///
    /// [`Severity::Bug`]: Severity::Bug
    #[inline(always)]
    #[must_use]
    pub const fn bug() -> Self {
        Self::new(Severity::Bug)
    }

    /// Create a new diagnostic with a severity of [`Severity::Error`].
    ///
    /// [`Severity::Error`]: Severity::Error
    #[inline(always)]
    #[must_use]
    pub const fn error() -> Self {
        Self::new(Severity::Error)
    }

    /// Create a new diagnostic with a severity of [`Severity::Warning`].
    ///
    /// [`Severity::Warning`]: Severity::Warning
    #[inline(always)]
    #[must_use]
    pub const fn warning() -> Self {
        Self::new(Severity::Warning)
    }

    /// Create a new diagnostic with a severity of [`Severity::Note`].
    ///
    /// [`Severity::Note`]: Severity::Note
    #[inline(always)]
    #[must_use]
    pub const fn note() -> Self {
        Self::new(Severity::Note)
    }

    /// Create a new diagnostic with a severity of [`Severity::Help`].
    ///
    /// [`Severity::Help`]: Severity::Help
    #[inline(always)]
    #[must_use]
    pub const fn help() -> Self {
        Self::new(Severity::Help)
    }

    /// Set the error code of the diagnostic.
    #[inline(always)]
    #[must_use]
    pub fn with_code(mut self, code: impl ToString) -> Self {
        self.code = Some(code.to_string());
        self
    }

    /// Set the message of the diagnostic.
    #[inline(always)]
    #[must_use]
    pub fn with_message(mut self, message: impl ToString) -> Self {
        self.message = message.to_string();
        self
    }

    /// Add a label to the diagnostic.
    #[inline(always)]
    #[must_use]
    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    /// Add some labels to the diagnostic.
    #[inline(always)]
    #[must_use]
    pub fn with_labels(mut self, labels: impl IntoIterator<Item = Label>) -> Self {
        self.labels.append(&mut labels.into_iter().collect());
        self
    }

    /// Add some notes to the diagnostic.
    #[inline(always)]
    #[must_use]
    pub fn with_notes(mut self, notes: impl IntoIterator<Item = impl ToString>) -> Self {
        self.notes
            .append(&mut notes.into_iter().map(|note| note.to_string()).collect());
        self
    }

    /// Returns the files involved in the diagnostic.
    #[inline(always)]
    #[must_use]
    pub fn files_involved(&self) -> Vec<stellar_interner::PathId> {
        self.labels
            .iter()
            .map(|label| label.location.filepath)
            .collect()
    }
}
