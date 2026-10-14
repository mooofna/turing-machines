//! Exact seeking in both directions by replaying from checkpoints.

use alloc::vec::Vec;
use core::borrow::Borrow;

use crate::machine::Machine;
use crate::outcome::Outcome;
use crate::rle::ChainSim;
use crate::sim::Simulator;

/// A simulator that a [`Timeline`] can checkpoint and replay.
pub trait Engine: Clone {
    /// The number of single steps taken so far.
    fn steps(&self) -> u64;

    /// Runs at most `fuel` more single steps and never overshoots.
    fn advance(&mut self, fuel: u64) -> Outcome;
}

impl<M: Borrow<Machine> + Clone> Engine for Simulator<M> {
    fn steps(&self) -> u64 {
        Simulator::steps(self)
    }

    fn advance(&mut self, fuel: u64) -> Outcome {
        self.run(fuel)
    }
}

impl<M: Borrow<Machine> + Clone> Engine for ChainSim<M> {
    fn steps(&self) -> u64 {
        ChainSim::steps(self)
    }

    fn advance(&mut self, fuel: u64) -> Outcome {
        self.run(fuel)
    }
}

/// A run that can be moved to any step, forwards or backwards.
///
/// Checkpoints are taken every `interval` steps. When there are more than 512, every second
/// one is dropped and the interval doubles, so memory stays bounded and seeking backwards costs
/// at most one interval of replay.
///
/// ```
/// use turing_core::{Dir, Machine, Simulator, Timeline};
/// let m = Machine::builder("walk").transition("q0", '␣', "q0", '␣', Dir::R).build().unwrap();
/// let mut t = Timeline::new(Simulator::new(&m, &[]));
/// t.seek(10);
/// t.seek(3);
/// assert_eq!(t.current().steps(), 3);
/// ```
#[derive(Clone, Debug)]
pub struct Timeline<E: Engine> {
    pub(crate) current: E,
    pub(crate) checkpoints: Vec<E>,
    pub(crate) interval: u64,
    pub(crate) last: Option<Outcome>,
}

impl<E: Engine> Timeline<E> {
    /// A timeline starting at the engine's current position.
    pub fn new(engine: E) -> Self {
        let _ = engine;
        todo!("A1")
    }

    /// The engine at the current position.
    pub fn current(&self) -> &E {
        &self.current
    }

    /// Moves to step `step`, or as far as the machine runs if it halts earlier.
    pub fn seek(&mut self, step: u64) -> Outcome {
        let _ = step;
        todo!("A1")
    }

    /// Moves `n` steps forward.
    pub fn forward(&mut self, n: u64) -> Outcome {
        let target = self.current.steps().saturating_add(n);
        self.seek(target)
    }

    /// Moves `n` steps back (not before step 0).
    pub fn back(&mut self, n: u64) -> Outcome {
        let target = self.current.steps().saturating_sub(n);
        self.seek(target)
    }
}
