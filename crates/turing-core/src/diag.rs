//! Diagnostics with source spans, shared by the parser, the validator and the front ends.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// A half-open range of UTF-8 byte offsets into a source text.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Span {
    /// First byte.
    pub start: u32,
    /// One past the last byte.
    pub end: u32,
}

impl Span {
    /// The span from `start` to `end`.
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// The smallest span covering both.
    pub fn join(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// The length in bytes.
    pub const fn len(self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    /// Whether the span covers no bytes.
    pub const fn is_empty(self) -> bool {
        self.end <= self.start
    }
}

/// How serious a diagnostic is.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Severity {
    /// The machine cannot be built.
    Error,
    /// The machine can be built, but something looks unintended.
    Warning,
}

/// A message about a machine description, optionally pointing into its source text.
///
/// Codes are stable: `E001` to `E099` are errors and `W001` to `W099` are warnings. They are
/// listed in `docs/dsl.md`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// Error or warning.
    pub severity: Severity,
    /// The stable code, for example `E004`.
    pub code: &'static str,
    /// One sentence that says what is wrong.
    pub message: String,
    /// Where in the source, if the description came from text.
    pub span: Option<Span>,
    /// A suggestion, for example "did you mean `q1`?".
    pub help: Option<String>,
    /// Other places that explain the problem, for example the first definition.
    pub related: Vec<(Span, String)>,
}

impl Diagnostic {
    /// An error with `code` and `message` and no span.
    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            code,
            message: message.into(),
            span: None,
            help: None,
            related: Vec::new(),
        }
    }

    /// A warning with `code` and `message` and no span.
    pub fn warning(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            ..Self::error(code, message)
        }
    }

    /// Attaches a span.
    #[must_use]
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// Attaches a help text.
    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Adds a related span with a note.
    #[must_use]
    pub fn with_related(mut self, span: Span, note: impl Into<String>) -> Self {
        self.related.push((span, note.into()));
        self
    }

    /// Whether this is an error.
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(f, "{kind}[{}]: {}", self.code, self.message)
    }
}
