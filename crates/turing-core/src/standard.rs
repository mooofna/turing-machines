//! The bbchallenge standard text format, for example `1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA`.
//!
//! States are the letters `A`, `B`, ... in order and are separated by `_`. Each state lists one
//! transition per read symbol `0`, `1`, ...: the written digit, `L` or `R`, and the next state.
//! `---` marks an undefined transition. A next-state letter beyond the last state (usually `Z`)
//! is the halting state: the transition is executed and counted, then the machine halts.

use alloc::string::String;

use crate::machine::Machine;
use crate::spec::MachineSpec;

/// Why a text is not a valid machine in the standard format, or a machine cannot be written in
/// it.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum StandardError {
    /// A syntax error at byte `at`.
    #[error("invalid standard format at byte {at}: {message}")]
    Syntax {
        /// Byte offset.
        at: usize,
        /// What was expected.
        message: String,
    },
    /// The states do not all have the same number of transitions.
    #[error("every state must list the same number of transitions")]
    Ragged,
    /// The machine has more than one tape, a move `N`, more than 26 states or more than 10
    /// symbols, or more than one halting state.
    #[error("this machine cannot be written in the standard format: {reason}")]
    Unsupported {
        /// Why not.
        reason: String,
    },
}

/// Parses a machine in the standard format. The result is a [`Kind::Beaver`] machine over
/// `{0, 1, ...}` with blank `0`, states `A`, `B`, ... and, if any transition targets it, a final
/// halting state `Z` without transitions.
///
/// [`Kind::Beaver`]: crate::Kind::Beaver
///
/// # Errors
///
/// See [`StandardError`].
pub fn parse_standard(text: &str) -> Result<Machine, StandardError> {
    let _ = text;
    todo!("A2")
}

/// Like [`parse_standard`], but returns the editable spec.
///
/// # Errors
///
/// See [`StandardError`].
pub fn spec_from_standard(text: &str) -> Result<MachineSpec, StandardError> {
    let _ = text;
    todo!("A2")
}

/// Writes `machine` in the standard format.
///
/// # Errors
///
/// Fails for machines the format cannot express, see [`StandardError::Unsupported`].
pub fn to_standard(machine: &Machine) -> Result<String, StandardError> {
    let _ = machine;
    todo!("A2")
}
