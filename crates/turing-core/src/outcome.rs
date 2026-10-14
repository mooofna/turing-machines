//! Results of running a simulator.

use crate::symbol::{Dir, State};

/// Whether a machine can make another step.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Status {
    /// δ is defined for the current state and symbols.
    Running,
    /// δ is undefined for the current state and symbols: the machine has halted.
    Halted,
}

/// A proof, found during simulation, that the machine never halts.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum NonHaltProof {
    /// The head is in `state` on a blank cell with only blanks in direction `dir`, and
    /// δ(`state`, ␣) moves in `dir` without leaving `state`: it walks off forever.
    ChainToInfinity {
        /// The looping state.
        state: State,
        /// The direction of travel.
        dir: Dir,
    },
    /// δ(`state`, a) = (`state`, a, N): the machine repeats the same step forever.
    StayLoop {
        /// The looping state.
        state: State,
    },
}

/// How a bounded run ended.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Outcome {
    /// The machine halted after `steps` steps in `state`.
    Halted {
        /// Total number of steps since the start.
        steps: u64,
        /// The state in which no transition was defined.
        state: State,
    },
    /// The step budget ran out after `steps` steps; the machine has not halted yet.
    OutOfFuel {
        /// Total number of steps since the start.
        steps: u64,
    },
    /// The simulator proved that the machine never halts.
    NeverHalts {
        /// Total number of steps since the start, when the proof was found.
        steps: u64,
        /// Why it never halts.
        proof: NonHaltProof,
    },
    /// A caller-supplied stop condition (a breakpoint) ended the run.
    Stopped {
        /// Total number of steps since the start.
        steps: u64,
    },
}

impl Outcome {
    /// The total number of steps since the start.
    pub const fn steps(self) -> u64 {
        match self {
            Self::Halted { steps, .. }
            | Self::OutOfFuel { steps }
            | Self::NeverHalts { steps, .. }
            | Self::Stopped { steps } => steps,
        }
    }

    /// Whether the machine has halted.
    pub const fn is_halted(self) -> bool {
        matches!(self, Self::Halted { .. })
    }
}
