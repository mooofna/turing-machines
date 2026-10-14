//! Exact seeking in both directions by replaying from checkpoints.

use alloc::vec;
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
    ///
    /// Unlike `run`, the simulators implement this without stopping at proofs of
    /// non-termination: they make exactly `fuel` steps unless the machine halts first, so a
    /// [`Timeline`] can step through a machine that loops forever. The result is the outcome
    /// at the final position: [`Outcome::Halted`] if the machine has halted,
    /// [`Outcome::NeverHalts`] if it is provably stuck in a loop there, and
    /// [`Outcome::OutOfFuel`] otherwise.
    fn advance(&mut self, fuel: u64) -> Outcome;
}

impl<M: Borrow<Machine> + Clone> Engine for Simulator<M> {
    fn steps(&self) -> u64 {
        Simulator::steps(self)
    }

    fn advance(&mut self, fuel: u64) -> Outcome {
        match self.run_inner(fuel, &mut |_| false, false) {
            Outcome::OutOfFuel { .. } => self.run(0),
            outcome => outcome,
        }
    }
}

impl<M: Borrow<Machine> + Clone> Engine for ChainSim<M> {
    fn steps(&self) -> u64 {
        ChainSim::steps(self)
    }

    fn advance(&mut self, fuel: u64) -> Outcome {
        match self.run_inner(fuel, &mut |_| false, false) {
            Outcome::OutOfFuel { .. } => self.run(0),
            outcome => outcome,
        }
    }
}

/// The distance between checkpoints of a new [`Timeline`], in steps.
const INITIAL_INTERVAL: u64 = 4096;

/// The number of checkpoints above which every second one is dropped.
const MAX_CHECKPOINTS: usize = 512;

/// A run that can be moved to any step, forwards or backwards.
///
/// Checkpoints are taken every `interval` steps. When there are more than 512, every second
/// one is dropped and the interval doubles, so memory stays bounded and seeking backwards costs
/// at most one interval of replay.
///
/// The checkpoint with index i is the engine after exactly `start + i × interval` steps,
/// where `start` is the step count of the engine the timeline was created with. Seeking
/// restores the nearest checkpoint at or before the target if the target lies behind the
/// current position, and then advances. Since [`Engine::advance`] never overshoots, every
/// position reached this way equals a fresh run to it.
///
/// Every checkpoint is a clone of the engine, so the engine should hold its machine by a cheap
/// handle such as `&Machine` or `Rc<Machine>` rather than by value.
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
    /// The outcome reported at the current position, if it is known.
    pub(crate) last: Option<Outcome>,
}

impl<E: Engine> Timeline<E> {
    /// A timeline starting at the engine's current position.
    pub fn new(engine: E) -> Self {
        Self {
            checkpoints: vec![engine.clone()],
            current: engine,
            interval: INITIAL_INTERVAL,
            last: None,
        }
    }

    /// The engine at the current position.
    pub fn current(&self) -> &E {
        &self.current
    }

    /// Moves to step `step`, or as far as the machine runs if it halts earlier.
    ///
    /// Targets before the start of the timeline move to its start. The result is the outcome
    /// at the final position, as described for [`Engine::advance`]: a machine that provably
    /// loops forever still moves to `step` and reports [`Outcome::NeverHalts`] there. The cost
    /// is the number of steps between the target and the current position or the nearest
    /// checkpoint before the target, whichever is closer.
    ///
    /// ```
    /// use turing_core::{Dir, Machine, Outcome, Simulator, Timeline};
    /// let m = Machine::builder("three")
    ///     .transition("a", '␣', "b", '1', Dir::R)
    ///     .transition("b", '␣', "c", '1', Dir::R)
    ///     .transition("c", '␣', "d", '1', Dir::R)
    ///     .build()
    ///     .unwrap();
    /// let mut t = Timeline::new(Simulator::new(&m, &[]));
    /// assert!(matches!(t.seek(100), Outcome::Halted { steps: 3, .. }));
    /// assert_eq!(t.back(1), Outcome::OutOfFuel { steps: 2 });
    /// ```
    pub fn seek(&mut self, step: u64) -> Outcome {
        let start = self.start();
        let target = step.max(start);
        let here = self.current.steps();
        if target == here
            && let Some(outcome) = self.last
        {
            return outcome;
        }
        if target < here {
            let index = usize::try_from((target - start) / self.interval)
                .unwrap_or(usize::MAX)
                .min(self.checkpoints.len() - 1);
            self.current = self.checkpoints[index].clone();
        }
        let outcome = self.advance_to(target);
        self.last = Some(outcome);
        outcome
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

    /// The step count of the first checkpoint.
    fn start(&self) -> u64 {
        self.checkpoints.first().map_or(0, Engine::steps)
    }

    /// The step count at which the next new checkpoint is due.
    fn next_checkpoint(&self) -> u64 {
        let recorded = u64::try_from(self.checkpoints.len()).unwrap_or(u64::MAX);
        self.start()
            .saturating_add(recorded.saturating_mul(self.interval))
    }

    /// Advances the current engine to `target`, which is not behind it, recording checkpoints
    /// on the way. Ends early if the engine cannot move any more.
    fn advance_to(&mut self, target: u64) -> Outcome {
        loop {
            let now = self.current.steps();
            if now == self.next_checkpoint() {
                self.record();
            }
            let stop = target.min(self.next_checkpoint()).max(now);
            let outcome = self.current.advance(stop - now);
            let reached = self.current.steps();
            if reached >= target || reached == now {
                return outcome;
            }
        }
    }

    /// Stores the current engine as the next checkpoint and thins the list if it is full.
    fn record(&mut self) {
        self.checkpoints.push(self.current.clone());
        if self.checkpoints.len() > MAX_CHECKPOINTS {
            let mut index = 0;
            self.checkpoints.retain(|_| {
                index += 1;
                index % 2 == 1
            });
            self.interval = self.interval.saturating_mul(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;
    use crate::outcome::NonHaltProof;
    use crate::symbol::{Dir, State, Symbol};
    use crate::tape::TapeView;

    /// A binary counter that never halts: it runs for a long time with a growing tape.
    fn counter() -> Machine {
        Machine::builder("counter")
            .blank('0')
            .tape_alphabet(['0', '1'])
            .transition("inc", '0', "back", '1', Dir::L)
            .transition("inc", '1', "inc", '0', Dir::R)
            .transition("back", '0', "inc", '0', Dir::R)
            .transition("back", '1', "back", '1', Dir::L)
            .build()
            .unwrap()
    }

    type Snapshot = (u64, State, i64, Vec<Symbol>);

    fn snapshot(steps: u64, state: State, tape: &impl TapeView) -> Snapshot {
        (steps, state, tape.head(), tape.window(-40, 120))
    }

    fn dense(sim: &Simulator<&Machine>) -> Snapshot {
        snapshot(sim.steps(), sim.state(), sim.tape(0))
    }

    fn chained(sim: &ChainSim<&Machine>) -> Snapshot {
        snapshot(sim.steps(), sim.state(), sim.tape())
    }

    fn fresh_dense(m: &Machine, steps: u64) -> Snapshot {
        let mut sim = Simulator::new(m, &[]);
        sim.advance(steps);
        dense(&sim)
    }

    #[test]
    fn seeking_matches_a_fresh_run_in_both_directions() {
        let m = counter();
        let mut timeline = Timeline::new(Simulator::new(&m, &[]));
        for target in [5, 100_000, 3, 99_999, 8192, 8191, 8193, 0, 50_000, 50_000] {
            assert_eq!(timeline.seek(target), Outcome::OutOfFuel { steps: target });
            assert_eq!(dense(timeline.current()), fresh_dense(&m, target));
        }
        assert_eq!(timeline.forward(7), Outcome::OutOfFuel { steps: 50_007 });
        assert_eq!(timeline.back(10), Outcome::OutOfFuel { steps: 49_997 });
        assert_eq!(timeline.back(u64::MAX), Outcome::OutOfFuel { steps: 0 });
    }

    #[test]
    fn checkpoints_are_thinned_out() {
        let m = counter();
        let mut timeline = Timeline::new(ChainSim::new(&m, &[]).unwrap());
        let far = 3 * 512 * INITIAL_INTERVAL;
        timeline.seek(far);
        assert!(timeline.checkpoints.len() <= MAX_CHECKPOINTS);
        assert_eq!(timeline.interval, 4 * INITIAL_INTERVAL);
        for (i, checkpoint) in timeline.checkpoints.iter().enumerate() {
            assert_eq!(
                checkpoint.steps(),
                u64::try_from(i).unwrap() * timeline.interval
            );
        }
        for target in [far - 1, 1_234_567, 17, far / 2] {
            timeline.seek(target);
            let mut fresh = ChainSim::new(&m, &[]).unwrap();
            fresh.advance(target);
            assert_eq!(chained(timeline.current()), chained(&fresh));
        }
    }

    #[test]
    fn seeking_past_the_halt_stops_there() {
        let m = Machine::builder("three")
            .transition("a", '␣', "b", '1', Dir::R)
            .transition("b", '␣', "c", '1', Dir::R)
            .transition("c", '␣', "d", '1', Dir::R)
            .build()
            .unwrap();
        let mut timeline = Timeline::new(ChainSim::new(&m, &[]).unwrap());
        let halted = Outcome::Halted {
            steps: 3,
            state: State::new(3),
        };
        assert_eq!(timeline.seek(10), halted);
        assert_eq!(timeline.current().steps(), 3);
        assert_eq!(timeline.forward(1), halted);
        assert_eq!(timeline.seek(1), Outcome::OutOfFuel { steps: 1 });
        assert_eq!(timeline.seek(3), halted);
    }

    #[test]
    fn machines_that_never_halt_can_still_be_stepped() {
        let m = Machine::builder("walk")
            .transition("q", '␣', "q", '1', Dir::L)
            .build()
            .unwrap();
        let walking = Outcome::NeverHalts {
            steps: 1000,
            proof: NonHaltProof::ChainToInfinity {
                state: State::new(0),
                dir: Dir::L,
            },
        };
        let mut dense = Timeline::new(Simulator::new(&m, &[]));
        assert_eq!(dense.seek(1000), walking);
        assert_eq!(dense.current().tape(0).head(), -1000);
        let mut chain = Timeline::new(ChainSim::new(&m, &[]).unwrap());
        assert_eq!(chain.seek(1000), walking);
        assert_eq!(chain.current().tape().head(), -1000);
        assert_eq!(chain.current().tape().count_non_blank(), 1000);
        assert_eq!(chain.current().macro_steps(), 1);

        let stay = Machine::builder("stay")
            .transition("q", '␣', "q", '␣', Dir::N)
            .build()
            .unwrap();
        let mut chain = Timeline::new(ChainSim::new(&stay, &[]).unwrap());
        assert!(matches!(
            chain.seek(1 << 40),
            Outcome::NeverHalts {
                steps: 1_099_511_627_776,
                proof: NonHaltProof::StayLoop { .. }
            }
        ));
    }

    #[test]
    fn a_timeline_starts_where_its_engine_is() {
        let m = counter();
        let mut sim = Simulator::new(&m, &[]);
        sim.run(100);
        let mut timeline = Timeline::new(sim);
        assert_eq!(timeline.seek(10), Outcome::OutOfFuel { steps: 100 });
        assert_eq!(timeline.seek(5000), Outcome::OutOfFuel { steps: 5000 });
        assert_eq!(timeline.seek(4196), Outcome::OutOfFuel { steps: 4196 });
        assert_eq!(timeline.checkpoints.len(), 2);
        assert_eq!(dense(timeline.current()), fresh_dense(&m, 4196));
    }
}
