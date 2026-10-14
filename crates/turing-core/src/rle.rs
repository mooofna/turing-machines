//! The accelerated single-tape simulator on a run-length encoded tape.
//!
//! # Chain steps
//!
//! Let the machine be in state q reading a, with δ(q, a) = (q, b, D) for D ∈ {L, R}, and let m
//! be the length of the run of a's directly beyond the head in direction D. Each of the next
//! m + 1 steps reads a in state q and applies the same transition, so they can be executed at
//! once: the m + 1 cells become b, the head ends on the first cell past the run, and the step
//! counter grows by m + 1. If a is the blank and only blanks lie in direction D, the machine
//! walks off forever and provably never halts. A chain that would exceed the remaining fuel is
//! executed partially, so a run stops exactly at any requested step.

use alloc::vec::Vec;
use core::borrow::Borrow;
use core::ops::RangeInclusive;

use crate::machine::{Action, Machine};
use crate::outcome::Outcome;
use crate::symbol::{State, Symbol};
use crate::tape::TapeView;

/// A maximal block of equal symbols.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Run {
    /// The repeated symbol.
    pub symbol: Symbol,
    /// The number of cells, at least 1.
    pub len: u64,
}

/// A tape stored as two stacks of runs around the head cell.
///
/// The top of each stack is the run next to the head. An empty stack stands for blanks to
/// infinity.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RunTape {
    pub(crate) left: Vec<Run>,
    pub(crate) cur: Symbol,
    pub(crate) right: Vec<Run>,
    pub(crate) head: i64,
}

impl RunTape {
    /// The runs left of the head, nearest last.
    pub fn left(&self) -> &[Run] {
        &self.left
    }

    /// The runs right of the head, nearest last.
    pub fn right(&self) -> &[Run] {
        &self.right
    }

    /// The symbol under the head.
    pub fn read(&self) -> Symbol {
        self.cur
    }
}

impl TapeView for RunTape {
    fn head(&self) -> i64 {
        self.head
    }

    fn get(&self, pos: i64) -> Symbol {
        let _ = pos;
        todo!("A1")
    }

    fn non_blank(&self) -> Option<RangeInclusive<i64>> {
        todo!("A1")
    }

    fn count_non_blank(&self) -> u64 {
        todo!("A1")
    }

    fn window(&self, from: i64, len: usize) -> Vec<Symbol> {
        let _ = (from, len);
        todo!("A1: materialise in O(runs + len)")
    }
}

/// One macro step of [`ChainSim`]: a transition applied `repeats` times in a row.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MacroEvent {
    /// The step count before the macro step.
    pub steps_before: u64,
    /// How many single steps the macro step covered, at least 1.
    pub repeats: u64,
    /// The state before the macro step.
    pub from: State,
    /// The symbol read on every repeated step.
    pub read: Symbol,
    /// The transition applied.
    pub action: Action,
}

/// Executes a single-tape machine with chain steps on a [`RunTape`].
///
/// The three-state busy beaver champion `1RB1RZ_1LB0RC_1LC1LA` (in the standard format of
/// [`standard`](crate::standard)) halts after 21 steps with five ones on the tape:
///
/// ```
/// use turing_core::{ChainSim, Dir, Kind, Machine, Outcome, TapeView};
/// let bb3 = Machine::builder("bb3")
///     .kind(Kind::Beaver)
///     .blank('0')
///     .tape_alphabet(['0', '1'])
///     .transition("A", '0', "B", '1', Dir::R)
///     .transition("A", '1', "Z", '1', Dir::R)
///     .transition("B", '0', "B", '1', Dir::L)
///     .transition("B", '1', "C", '0', Dir::R)
///     .transition("C", '0', "C", '1', Dir::L)
///     .transition("C", '1', "A", '1', Dir::L)
///     .final_state("Z")
///     .build()
///     .unwrap();
/// let mut sim = ChainSim::new(&bb3, &[]).unwrap();
/// assert!(matches!(sim.run(1_000), Outcome::Halted { steps: 21, .. }));
/// assert_eq!(sim.tape().count_non_blank(), 5);
/// assert!(sim.macro_steps() < 21);
/// ```
#[derive(Clone, Debug)]
pub struct ChainSim<M: Borrow<Machine>> {
    pub(crate) machine: M,
    pub(crate) state: State,
    pub(crate) tape: RunTape,
    pub(crate) steps: u64,
    pub(crate) macro_steps: u64,
    pub(crate) extent: (i64, i64),
}

/// [`ChainSim`] only runs single-tape machines.
#[derive(Copy, Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("chain simulation needs a single-tape machine, this one has {tapes} tapes")]
pub struct NotSingleTape {
    /// The machine's tape count.
    pub tapes: usize,
}

impl<M: Borrow<Machine>> ChainSim<M> {
    /// Starts in q0 with `input` from position 0.
    ///
    /// # Errors
    ///
    /// Fails for machines with more than one tape.
    pub fn new(machine: M, input: &[Symbol]) -> Result<Self, NotSingleTape> {
        let _ = (machine, input);
        todo!("A1")
    }

    /// The machine being run.
    pub fn machine(&self) -> &Machine {
        self.machine.borrow()
    }

    /// The current state.
    pub fn state(&self) -> State {
        self.state
    }

    /// The tape.
    pub fn tape(&self) -> &RunTape {
        &self.tape
    }

    /// The number of single steps taken so far.
    pub fn steps(&self) -> u64 {
        self.steps
    }

    /// The number of macro steps taken so far.
    pub fn macro_steps(&self) -> u64 {
        self.macro_steps
    }

    /// The smallest and largest head position so far.
    pub fn extent(&self) -> RangeInclusive<i64> {
        self.extent.0..=self.extent.1
    }

    /// Runs until the machine halts, is proved to never halt, or `fuel` more single steps have
    /// been made. Never overshoots `fuel`.
    pub fn run(&mut self, fuel: u64) -> Outcome {
        self.run_until(fuel, |_| false)
    }

    /// Like [`ChainSim::run`], but checks `stop` before every macro step and stops (without
    /// executing it) when it returns `true`.
    pub fn run_until(&mut self, fuel: u64, stop: impl FnMut(&MacroEvent) -> bool) -> Outcome {
        let _ = (fuel, stop);
        todo!("A1")
    }
}
