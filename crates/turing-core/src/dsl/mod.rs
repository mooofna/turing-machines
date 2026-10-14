//! The description language, in the notation of the TheoLog course.
//!
//! ```text
//! // Unary multiplication, TheoLog 2024, exercise sheet 1, task 3
//! machine "A_mul"
//! kind transducer
//! input "#xxx#xx#"
//! output unary x
//!
//! Q = {q0, q1, q2}
//! Σ = {x, #}
//! Γ = {x, X, #, ␣}
//! start = q0
//! F = {q2}
//!
//! δ(q0, #) = (q1, #, R)     // skip the first separator
//! δ(q1, {x, X}) = (q1, *, R) // `*` writes back what was read
//! ```
//!
//! The grammar, the semantics and every diagnostic code are documented in `docs/dsl.md`.

mod format;
mod lexer;
mod parser;

use alloc::string::String;
use alloc::vec::Vec;

use crate::diag::Diagnostic;
use crate::spec::MachineSpec;

/// The result of parsing: a spec if there were no errors, and every diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parsed {
    /// The machine, present if no diagnostic is an error.
    pub spec: Option<MachineSpec>,
    /// Errors and warnings, in source order.
    pub diagnostics: Vec<Diagnostic>,
}

/// Which characters the formatter uses.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum FormatStyle {
    /// `δ`, `Σ`, `Γ` and `␣`, as in the course.
    #[default]
    Unicode,
    /// `delta`, `Sigma`, `Gamma` and `_`, for plain keyboards.
    Ascii,
}

/// Parses a description. Errors are recovered per line, so all of them are reported.
pub fn parse(source: &str) -> Parsed {
    parser::parse(source)
}

/// Writes `spec` in canonical form. Comments are kept; read sets and `*` are expanded.
///
/// `parse(&format(spec, style)).spec` equals `spec` up to source spans.
pub fn format(spec: &MachineSpec, style: FormatStyle) -> String {
    format::format(spec, style)
}
