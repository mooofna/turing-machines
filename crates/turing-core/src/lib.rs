#![no_std]
#![doc = include_str!("../README.md")]

extern crate alloc;
#[cfg(test)]
extern crate std;

pub mod alphabet;
pub mod builder;
pub mod checkpoint;
pub mod config_text;
pub mod diag;
pub mod dsl;
pub mod machine;
pub mod outcome;
pub mod output;
pub mod rle;
pub mod sim;
pub mod spec;
pub mod standard;
pub mod symbol;
pub mod tape;

pub use alphabet::Alphabet;
pub use builder::{BuildError, BuildErrors, MachineBuilder};
pub use checkpoint::{Engine, Timeline};
pub use diag::{Diagnostic, Severity, Span};
pub use machine::{Action, BitOrder, Decoder, InputError, Kind, Machine, StateInfo, Transition};
pub use outcome::{NonHaltProof, Outcome, Status};
pub use output::{Output, decode};
pub use rle::{ChainSim, MacroEvent, NotSingleTape, Run, RunTape};
pub use sim::{Simulator, StepEvent};
pub use spec::{Declared, MachineSpec, SpecComments, StateSpec, TransitionSpec};
pub use symbol::{Dir, MAX_STATES, MAX_SYMBOLS, MAX_TAPES, State, Symbol};
pub use tape::{Tape, TapeView};
