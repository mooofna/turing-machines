//! The description language, in the notation of the course Theoretische Informatik und Logik
//! (TU Dresden).
//!
//! Each line holds one declaration or one transition δ(q, a) = (p, b, D); `//` starts a
//! comment, because `#` is a common tape symbol. Every declaration is optional: undeclared sets
//! are inferred from the transitions, declared ones are enforced.
//!
//! ```
//! use turing_core::dsl::{self, FormatStyle};
//! use turing_core::{Simulator, decode};
//!
//! let source = r#"
//! // Unary addition: x^m # x^n becomes x^(m + n).
//! machine "add"
//! input "xx#xxx"
//! output unary x
//!
//! δ(q0, x) = (q0, x, R)
//! δ(q0, #) = (q1, x, R)   // join the two blocks
//! δ(q1, x) = (q1, x, R)
//! δ(q1, ␣) = (q2, ␣, L)
//! δ(q2, x) = (q3, ␣, R)   // and drop one x at the end
//! "#;
//!
//! let parsed = dsl::parse(source);
//! assert!(parsed.diagnostics.is_empty());
//! let spec = parsed.spec.unwrap();
//! let machine = spec.compile().unwrap();
//!
//! let input = machine.encode_input(machine.default_input()).unwrap();
//! let mut sim = Simulator::new(&machine, &input);
//! sim.run(1_000);
//! assert_eq!(decode(&machine, sim.tape(0)).value, Some(5));
//!
//! // Formatting is canonical and keeps the comments.
//! let text = dsl::format(&spec, FormatStyle::Unicode);
//! assert!(text.contains("δ(q0, #) = (q1, x, R)  // join the two blocks"));
//! assert_eq!(dsl::parse(&text).spec.unwrap().transitions.len(), 5);
//! ```
//!
//! Read sets and `*` keep tables short; the parser expands them into one transition each:
//!
//! ```
//! use turing_core::dsl;
//! let spec = dsl::parse("δ(q0, {a, b}) = (q0, *, R)").spec.unwrap();
//! assert_eq!(spec.transitions.len(), 2);
//! assert_eq!(spec.transitions[1].write, ['b']);
//! ```
//!
//! Errors are collected line by line, with spans and codes:
//!
//! ```
//! use turing_core::dsl;
//! let parsed = dsl::parse("Q = {q0, q1}\nδ(q0, a) = (q2, a, S)\nδ(q0, a) = (q3, a, R)");
//! let codes: Vec<_> = parsed.diagnostics.iter().map(|d| d.code).collect();
//! assert_eq!(codes, ["E008", "E003"]);
//! assert!(parsed.spec.is_none());
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
    /// The machine, present if no diagnostic is an error. Its undeclared sets are inferred.
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
///
/// Parsing never panics, whatever the input.
pub fn parse(source: &str) -> Parsed {
    parser::parse(source)
}

/// Writes `spec` in canonical form. Comments are kept; read sets and `*` are expanded.
///
/// `parse(&format(spec, style)).spec` equals `spec` up to source spans, for every spec that
/// [`parse`] can return.
pub fn format(spec: &MachineSpec, style: FormatStyle) -> String {
    format::format(spec, style)
}

/// Whether `name` can be written as a state name: a letter or `_`, followed by letters,
/// digits, `_` and `'`, but not a lone `_`, which is the blank.
pub(crate) fn is_state_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(lexer::is_word_start)
        && chars.all(lexer::is_word_continue)
        && name != "_"
}
