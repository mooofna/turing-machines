//! `MachineSpec`, the editable interchange form of a machine.
//!
//! A spec keeps everything an editor needs that a [`Machine`] does not: names in their
//! original order, which sets were declared explicitly, comments and source spans. The
//! description language, the transition table and the state diagram of the web front end all
//! edit a spec; [`MachineSpec::compile`] turns it into a runnable machine.

use alloc::string::String;
use alloc::vec::Vec;

use crate::diag::{Diagnostic, Span};
use crate::machine::{Decoder, Kind, Machine};
use crate::symbol::Dir;

/// A machine described by names and glyphs.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct MachineSpec {
    /// The machine's name, possibly empty.
    pub name: String,
    /// What the machine is used for.
    pub kind: Kind,
    /// The number of tapes, 1 to [`MAX_TAPES`](crate::MAX_TAPES).
    pub tapes: u8,
    /// The blank glyph.
    pub blank: char,
    /// Σ, in order. Complete even if it was inferred.
    pub input_alphabet: Vec<char>,
    /// Γ, in order, blank included. Complete even if it was inferred.
    pub tape_alphabet: Vec<char>,
    /// Q, in order. Complete even if it was inferred.
    pub states: Vec<StateSpec>,
    /// The name of q0.
    pub start: String,
    /// δ, in source order. Each entry defines exactly one δ(q, a); sugar such as read sets is
    /// expanded by the parser.
    pub transitions: Vec<TransitionSpec>,
    /// The input word used when the machine is opened.
    pub input: String,
    /// How the output is read as a number.
    pub output: Decoder,
    /// Which parts were written explicitly, so that formatting reproduces them.
    pub declared: Declared,
    /// Comments that do not belong to a transition.
    pub comments: SpecComments,
}

/// A state of a [`MachineSpec`].
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct StateSpec {
    /// The state name.
    pub name: String,
    /// Whether the state belongs to F.
    pub is_final: bool,
}

/// One entry δ(from, read) = (to, write, dir) of a [`MachineSpec`].
///
/// `read`, `write` and `dir` have one entry per tape.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TransitionSpec {
    /// The current state.
    pub from: String,
    /// The symbols under the heads.
    pub read: Vec<char>,
    /// The next state.
    pub to: String,
    /// The symbols written.
    pub write: Vec<char>,
    /// The moves.
    pub dir: Vec<Dir>,
    /// A comment at the end of the line.
    pub comment: Option<String>,
    /// Comment lines directly above the transition.
    pub leading: Vec<String>,
    /// Where the transition was written, if it came from text.
    pub span: Option<Span>,
}

/// Which parts of a [`MachineSpec`] were declared explicitly in the source.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[expect(
    clippy::struct_excessive_bools,
    reason = "one flag per optional declaration"
)]
pub struct Declared {
    /// `Q = {...}`.
    pub states: bool,
    /// `Σ = {...}`.
    pub input_alphabet: bool,
    /// `Γ = {...}`.
    pub tape_alphabet: bool,
    /// `start = ...`.
    pub start: bool,
    /// `blank = ...`.
    pub blank: bool,
}

/// Comments of a [`MachineSpec`] that are not attached to a transition.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SpecComments {
    /// Comment lines before the first declaration.
    pub header: Vec<String>,
    /// Comment lines after the last transition.
    pub trailing: Vec<String>,
}

impl MachineSpec {
    /// An empty single-tape transducer with blank `␣`.
    pub fn new() -> Self {
        Self {
            tapes: 1,
            blank: '␣',
            tape_alphabet: alloc::vec!['␣'],
            ..Self::default()
        }
    }

    /// Checks the spec and reports every error and warning.
    pub fn check(&self) -> Vec<Diagnostic> {
        todo!("A2: build with MachineBuilder, map BuildError to Diagnostic, add warnings")
    }

    /// Builds the runnable machine.
    ///
    /// The transition order of the spec is kept, so [`Machine::origin`] returns indices into
    /// [`MachineSpec::transitions`].
    ///
    /// # Errors
    ///
    /// Returns all diagnostics (errors and warnings) if there is at least one error.
    pub fn compile(&self) -> Result<Machine, Vec<Diagnostic>> {
        todo!("A2: compile via MachineBuilder, preserving transition order")
    }

    /// The index of the state called `name`.
    pub fn state_index(&self, name: &str) -> Option<usize> {
        self.states.iter().position(|s| s.name == name)
    }

    /// The index of the transition δ(`from`, `read`), if defined.
    pub fn transition_index(&self, from: &str, read: &[char]) -> Option<usize> {
        self.transitions
            .iter()
            .position(|t| t.from == from && t.read == read)
    }
}

impl From<&Machine> for MachineSpec {
    /// The canonical spec of a machine: every set declared, transitions by state and symbol,
    /// no comments.
    fn from(machine: &Machine) -> Self {
        let _ = machine;
        todo!("A2: canonical spec of a machine")
    }
}
