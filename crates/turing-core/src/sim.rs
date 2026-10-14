//! The reference simulator: one step at a time on dense tapes, any number of tapes.

use alloc::vec::Vec;
use core::borrow::Borrow;

use crate::machine::{Action, Machine};
use crate::outcome::{Outcome, Status};
use crate::symbol::{MAX_TAPES, State, Symbol};
use crate::tape::Tape;

/// One executed step, passed to stop conditions.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StepEvent {
    /// The number of steps after this one, counted from the start.
    pub steps: u64,
    /// The state before the step.
    pub from: State,
    /// The symbols read.
    pub read: [Symbol; MAX_TAPES],
    /// What the step did.
    pub action: Action,
}

/// Executes a machine step by step from an input on the first tape.
///
/// `M` is anything that borrows a [`Machine`]: `&Machine`, `Machine`, `Rc<Machine>` or
/// `Arc<Machine>`.
///
/// ```
/// use turing_core::{Dir, Machine, Outcome, Simulator};
/// let m = Machine::builder("flip")
///     .transition("q0", '0', "q0", '1', Dir::R)
///     .transition("q0", '1', "q0", '0', Dir::R)
///     .build()
///     .unwrap();
/// let input = m.encode_input("0110").unwrap();
/// let mut sim = Simulator::new(&m, &input);
/// assert!(matches!(sim.run(100), Outcome::Halted { steps: 4, .. }));
/// ```
#[derive(Clone, Debug)]
pub struct Simulator<M: Borrow<Machine>> {
    pub(crate) machine: M,
    pub(crate) state: State,
    pub(crate) tapes: Vec<Tape>,
    pub(crate) steps: u64,
}

impl<M: Borrow<Machine>> Simulator<M> {
    /// Starts in q0 with `input` on the first tape and blank further tapes.
    pub fn new(machine: M, input: &[Symbol]) -> Self {
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

    /// Tape `i` (0-based).
    ///
    /// # Panics
    ///
    /// Panics if the machine has no tape `i`.
    pub fn tape(&self, i: usize) -> &Tape {
        &self.tapes[i]
    }

    /// All tapes.
    pub fn tapes(&self) -> &[Tape] {
        &self.tapes
    }

    /// The number of steps taken so far.
    pub fn steps(&self) -> u64 {
        self.steps
    }

    /// The symbols under the heads; entries beyond the tape count are blank.
    pub fn read(&self) -> [Symbol; MAX_TAPES] {
        todo!("A1")
    }

    /// The transition that the next step would apply.
    pub fn next_action(&self) -> Option<Action> {
        todo!("A1")
    }

    /// Makes one step, or reports that the machine has halted.
    pub fn step(&mut self) -> Status {
        todo!("A1")
    }

    /// Runs until the machine halts or `fuel` more steps have been made.
    pub fn run(&mut self, fuel: u64) -> Outcome {
        self.run_until(fuel, |_| false)
    }

    /// Like [`Simulator::run`], but stops after any step for which `stop` returns `true`.
    pub fn run_until(&mut self, fuel: u64, stop: impl FnMut(&StepEvent) -> bool) -> Outcome {
        let _ = (fuel, stop);
        todo!("A1")
    }
}
