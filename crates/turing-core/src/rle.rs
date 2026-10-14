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
//!
//! # Correctness
//!
//! Number the cells from the head in direction D as c₀, c₁, ..., so that c₀ is the head cell
//! and c₁ to cₘ form the run of a's. Claim: for 0 ≤ i ≤ n ≤ m + 1, after i of the steps the
//! machine is in state q with the head on cᵢ, the cells c₀ to cᵢ₋₁ hold b, and every other
//! cell is unchanged. For i = 0 this is the starting configuration. If it holds for some
//! i < n, then cᵢ is unchanged, because the head has only written on c₀ to cᵢ₋₁, so it holds
//! a (it is c₀ or part of the run). The machine therefore applies δ(q, a) = (q, b, D): it
//! writes b on cᵢ, moves to cᵢ₊₁ and stays in q, which is the claim for i + 1. With
//! i = n the macro step of [`ChainSim`] produces exactly this configuration: it pushes n
//! cells of b behind the head and removes c₁ to cₙ from the runs ahead, cₙ becoming the
//! head cell.
//!
//! If a is the blank and no run lies ahead, every cell beyond the head is blank, so the claim
//! holds for every n: the machine never leaves q, never reads anything but blanks and never
//! halts.
//!
//! Every other transition, including those that do not move the head, is executed as a single
//! step, so the two simulators visit the same configurations at every step count that
//! [`ChainSim`] stops at.
//!
//! # Tape invariants
//!
//! A [`RunTape`] keeps three invariants, which make the run structure canonical:
//!
//! 1. every run has at least one cell;
//! 2. neighbouring runs on the same side of the head hold different symbols;
//! 3. the run farthest from the head on each side is not blank: blanks at the end of a side
//!    belong to the infinite blank remainder and are not stored.
//!
//! By (2) the run ahead of the head is the whole block of equal symbols there, and by (3) a
//! side holds only blanks if and only if its stack is empty.

use alloc::vec;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::ops::RangeInclusive;

use crate::machine::{Action, Machine};
use crate::outcome::{NonHaltProof, Outcome};
use crate::sim::assert_input_in_alphabet;
use crate::symbol::{Dir, State, Symbol};
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
/// infinity. The stacks keep the invariants listed in the [module documentation](self): runs
/// are maximal and blanks at the far end are not stored.
///
/// ```
/// use turing_core::{Run, RunTape, Symbol, TapeView};
/// let (a, b) = (Symbol::new(1), Symbol::new(2));
/// let tape = RunTape::with_input(&[a, a, a, b, Symbol::BLANK, Symbol::BLANK]);
/// assert_eq!(tape.read(), a);
/// // The nearest run is last; the trailing blanks are not stored.
/// assert_eq!(tape.right(), [Run { symbol: b, len: 1 }, Run { symbol: a, len: 2 }]);
/// assert_eq!(tape.non_blank(), Some(0..=3));
/// assert_eq!(tape.window(-1, 6), [Symbol::BLANK, a, a, a, b, Symbol::BLANK]);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RunTape {
    pub(crate) left: Vec<Run>,
    pub(crate) cur: Symbol,
    pub(crate) right: Vec<Run>,
    pub(crate) head: i64,
}

impl RunTape {
    /// A tape holding `word` from position 0, head on position 0.
    pub fn with_input(word: &[Symbol]) -> Self {
        let mut tape = Self {
            cur: word.first().copied().unwrap_or(Symbol::BLANK),
            ..Self::default()
        };
        for &symbol in word.iter().skip(1).rev() {
            push(&mut tape.right, symbol, 1);
        }
        tape
    }

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

    /// The runs in direction `dir` (left or right) of the head, nearest last.
    fn side(&self, dir: Dir) -> &[Run] {
        match dir {
            Dir::L => &self.left,
            Dir::R | Dir::N => &self.right,
        }
    }

    /// The number of cells directly beyond the head in direction `dir` that hold `symbol`.
    fn run_ahead(&self, dir: Dir, symbol: Symbol) -> u64 {
        match self.side(dir).last() {
            Some(run) if run.symbol == symbol => run.len,
            _ => 0,
        }
    }

    /// Writes `write` on the head cell and the `n - 1` cells beyond it in direction `dir`, and
    /// moves the head `n` cells in that direction.
    ///
    /// The `n - 1` cells must hold the symbol under the head, which is the case for
    /// `n <= 1 + run_ahead(dir, read())`. With `Dir::N`, `n` must be 1.
    fn apply(&mut self, write: Symbol, dir: Dir, n: u64) {
        debug_assert!(n >= 1, "a macro step makes at least one step");
        debug_assert!(
            n <= 1 + self.run_ahead(dir, self.cur),
            "a chain may only cover the run ahead"
        );
        let (behind, ahead, delta) = match dir {
            Dir::L => (&mut self.right, &mut self.left, -1),
            Dir::R => (&mut self.left, &mut self.right, 1),
            Dir::N => {
                debug_assert_eq!(n, 1, "a step that does not move cannot repeat");
                self.cur = write;
                return;
            }
        };
        push(behind, write, n);
        take(ahead, n - 1);
        self.cur = pop(ahead);
        self.head = self.head.saturating_add(delta * signed(n));
    }
}

/// Converts a run length or step count to a signed distance, saturating.
fn signed(n: u64) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

/// Puts `len` cells of `symbol` on top of `stack`, merging with an equal top run and dropping
/// blanks that would end up at the far end.
fn push(stack: &mut Vec<Run>, symbol: Symbol, len: u64) {
    match stack.last_mut() {
        _ if len == 0 => {}
        Some(top) if top.symbol == symbol => top.len += len,
        None if symbol.is_blank() => {}
        _ => stack.push(Run { symbol, len }),
    }
}

/// Removes `n` cells from the top run of `stack`, which must hold at least `n` cells.
fn take(stack: &mut Vec<Run>, n: u64) {
    if n == 0 {
        return;
    }
    if let Some(top) = stack.last_mut() {
        debug_assert!(top.len >= n, "only the top run can be taken from");
        top.len -= n;
        if top.len == 0 {
            stack.pop();
        }
    }
}

/// Removes the cell nearest to the head from `stack` and returns its symbol.
fn pop(stack: &mut Vec<Run>) -> Symbol {
    match stack.last_mut() {
        None => Symbol::BLANK,
        Some(top) => {
            let symbol = top.symbol;
            top.len -= 1;
            if top.len == 0 {
                stack.pop();
            }
            symbol
        }
    }
}

impl TapeView for RunTape {
    fn head(&self) -> i64 {
        self.head
    }

    fn get(&self, pos: i64) -> Symbol {
        let (stack, mut distance) = match pos.cmp(&self.head) {
            core::cmp::Ordering::Equal => return self.cur,
            core::cmp::Ordering::Less => (&self.left, self.head.abs_diff(pos)),
            core::cmp::Ordering::Greater => (&self.right, pos.abs_diff(self.head)),
        };
        for run in stack.iter().rev() {
            if distance <= run.len {
                return run.symbol;
            }
            distance -= run.len;
        }
        Symbol::BLANK
    }

    fn non_blank(&self) -> Option<RangeInclusive<i64>> {
        let mut extent: Option<(i64, i64)> = None;
        let mut note = |lo: i64, hi: i64| {
            extent = Some(extent.map_or((lo, hi), |(a, b)| (a.min(lo), b.max(hi))));
        };
        let mut near = self.head;
        for run in self.left.iter().rev() {
            let far = near.saturating_sub(signed(run.len));
            if !run.symbol.is_blank() {
                note(far, near - 1);
            }
            near = far;
        }
        if !self.cur.is_blank() {
            note(self.head, self.head);
        }
        let mut near = self.head;
        for run in self.right.iter().rev() {
            let far = near.saturating_add(signed(run.len));
            if !run.symbol.is_blank() {
                note(near + 1, far);
            }
            near = far;
        }
        extent.map(|(lo, hi)| lo..=hi)
    }

    fn count_non_blank(&self) -> u64 {
        let runs: u64 = self
            .left
            .iter()
            .chain(&self.right)
            .filter(|run| !run.symbol.is_blank())
            .map(|run| run.len)
            .sum();
        runs + u64::from(!self.cur.is_blank())
    }

    fn window(&self, from: i64, len: usize) -> Vec<Symbol> {
        let mut out = vec![Symbol::BLANK; len];
        let Some(last) = i64::try_from(len)
            .ok()
            .and_then(|len| from.checked_add(len - 1))
        else {
            return (0..len)
                .map(|i| self.get(from.saturating_add(i64::try_from(i).unwrap_or(i64::MAX))))
                .collect();
        };
        // Fills the cells lo..=hi, clipped to the window, with `symbol`.
        let mut fill = |lo: i64, hi: i64, symbol: Symbol| {
            let (lo, hi) = (lo.max(from), hi.min(last));
            if lo <= hi && !symbol.is_blank() {
                let start = usize::try_from(lo - from).unwrap_or(0);
                let end = usize::try_from(hi - from).unwrap_or(0);
                out[start..=end].fill(symbol);
            }
        };
        let mut near = self.head;
        for run in self.left.iter().rev() {
            if near <= from {
                break;
            }
            let far = near.saturating_sub(signed(run.len));
            fill(far, near - 1, run.symbol);
            near = far;
        }
        fill(self.head, self.head, self.cur);
        let mut near = self.head;
        for run in self.right.iter().rev() {
            if near >= last {
                break;
            }
            let far = near.saturating_add(signed(run.len));
            fill(near + 1, far, run.symbol);
            near = far;
        }
        out
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
    ///
    /// # Panics
    ///
    /// Panics if `input` contains a symbol outside the machine's alphabet. Inputs converted
    /// with [`Machine::encode_input`] never do.
    pub fn new(machine: M, input: &[Symbol]) -> Result<Self, NotSingleTape> {
        let m = machine.borrow();
        if m.tapes() != 1 {
            return Err(NotSingleTape { tapes: m.tapes() });
        }
        assert_input_in_alphabet(m, input);
        Ok(Self {
            state: m.start(),
            machine,
            tape: RunTape::with_input(input),
            steps: 0,
            macro_steps: 0,
            extent: (0, 0),
        })
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
    ///
    /// Before each macro step the run ends, in this order, with [`Outcome::Halted`] if δ is
    /// undefined, with [`Outcome::NeverHalts`] if the machine provably loops forever, with
    /// [`Outcome::OutOfFuel`] if `fuel` single steps have been made, and with
    /// [`Outcome::Stopped`] if `stop` returns `true`. The event passed to `stop` describes the
    /// macro step about to be made, already shortened to the remaining fuel. Since the step is
    /// not made, calling `run_until` again with the same condition stops at once; make one
    /// step with `run(1)` to get past a breakpoint.
    ///
    /// ```
    /// use turing_core::{ChainSim, Dir, Machine, Outcome};
    /// let m = Machine::builder("sweep")
    ///     .transition("q", 'x', "q", 'y', Dir::R)
    ///     .transition("q", '␣', "p", '␣', Dir::L)
    ///     .build()
    ///     .unwrap();
    /// let input = m.encode_input("xxxxx").unwrap();
    /// let mut sim = ChainSim::new(&m, &input).unwrap();
    /// // One macro step covers the whole run of x.
    /// let outcome = sim.run_until(100, |event| event.repeats == 5);
    /// assert_eq!(outcome, Outcome::Stopped { steps: 0 });
    /// assert!(matches!(sim.run(100), Outcome::Halted { steps: 6, .. }));
    /// assert_eq!(sim.macro_steps(), 2);
    /// ```
    pub fn run_until(&mut self, fuel: u64, mut stop: impl FnMut(&MacroEvent) -> bool) -> Outcome {
        let mut fuel = fuel;
        loop {
            let from = self.state;
            let read = self.tape.cur;
            let Some(action) = self.machine.borrow().action1(from, read) else {
                return Outcome::Halted {
                    steps: self.steps,
                    state: from,
                };
            };
            let (write, dir) = (action.write[0], action.dir[0]);
            let chain = action.next == from && dir != Dir::N;
            if action.next == from {
                let proof = if dir == Dir::N && write == read {
                    Some(NonHaltProof::StayLoop { state: from })
                } else if chain && read.is_blank() && self.tape.side(dir).is_empty() {
                    Some(NonHaltProof::ChainToInfinity { state: from, dir })
                } else {
                    None
                };
                if let Some(proof) = proof {
                    return Outcome::NeverHalts {
                        steps: self.steps,
                        proof,
                    };
                }
            }
            if fuel == 0 {
                return Outcome::OutOfFuel { steps: self.steps };
            }
            let repeats = if chain {
                self.tape.run_ahead(dir, read).saturating_add(1).min(fuel)
            } else {
                1
            };
            let event = MacroEvent {
                steps_before: self.steps,
                repeats,
                from,
                read,
                action,
            };
            if stop(&event) {
                return Outcome::Stopped { steps: self.steps };
            }
            self.tape.apply(write, dir, repeats);
            self.state = action.next;
            self.steps += repeats;
            self.macro_steps += 1;
            fuel -= repeats;
            self.extent = (
                self.extent.0.min(self.tape.head),
                self.extent.1.max(self.tape.head),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use proptest::prelude::*;

    use super::*;
    use crate::tape::Tape;

    fn s(i: u8) -> Symbol {
        Symbol::new(i)
    }

    fn run(symbol: u8, len: u64) -> Run {
        Run {
            symbol: s(symbol),
            len,
        }
    }

    /// Asserts the three invariants from the module documentation.
    fn assert_canonical(tape: &RunTape) {
        for stack in [&tape.left, &tape.right] {
            assert!(stack.iter().all(|r| r.len >= 1), "empty run in {tape:?}");
            assert!(
                stack.windows(2).all(|w| w[0].symbol != w[1].symbol),
                "unmerged runs in {tape:?}"
            );
            assert!(
                stack.first().is_none_or(|r| !r.symbol.is_blank()),
                "stored blank remainder in {tape:?}"
            );
        }
    }

    #[test]
    fn input_becomes_maximal_runs() {
        let tape = RunTape::with_input(&[s(1), s(1), s(0), s(0), s(2), s(2), s(2), s(0)]);
        assert_eq!(tape.read(), s(1));
        assert_eq!(tape.right(), [run(2, 3), run(0, 2), run(1, 1)]);
        assert_eq!(tape.left(), []);
        assert_canonical(&tape);
        assert_eq!(tape.non_blank(), Some(0..=6));
        assert_eq!(tape.count_non_blank(), 5);
        assert_eq!(RunTape::with_input(&[]), RunTape::default());
        assert_eq!(RunTape::with_input(&[s(0), s(0)]), RunTape::default());
    }

    #[test]
    fn blank_runs_inside_the_tape_are_kept() {
        let mut tape = RunTape::with_input(&[s(1), s(0), s(1)]);
        tape.apply(s(0), Dir::R, 1);
        assert_eq!(tape.left(), []);
        assert_eq!(tape.read(), s(0));
        tape.apply(s(0), Dir::R, 1);
        assert_eq!(tape.read(), s(1));
        assert_eq!(tape.left(), []);
        tape.apply(s(2), Dir::L, 1);
        assert_eq!(tape.right(), [run(2, 1)]);
        assert_eq!(tape.head(), 1);
        assert_eq!(tape.non_blank(), Some(2..=2));
        assert_canonical(&tape);
    }

    #[test]
    fn chains_move_over_whole_runs() {
        let mut tape = RunTape::with_input(&[s(1), s(1), s(1), s(1), s(2)]);
        assert_eq!(tape.run_ahead(Dir::R, s(1)), 3);
        assert_eq!(tape.run_ahead(Dir::L, s(1)), 0);
        tape.apply(s(3), Dir::R, 2);
        assert_eq!(tape.head(), 2);
        assert_eq!(tape.read(), s(1));
        assert_eq!(tape.left(), [run(3, 2)]);
        assert_eq!(tape.right(), [run(2, 1), run(1, 1)]);
        tape.apply(s(3), Dir::R, 2);
        assert_eq!(tape.head(), 4);
        assert_eq!(tape.read(), s(2));
        assert_eq!(tape.left(), [run(3, 4)]);
        assert_eq!(tape.right(), []);
        tape.apply(s(0), Dir::R, 1);
        assert_eq!(tape.window(-1, 7), [0, 3, 3, 3, 3, 0, 0].map(s));
        assert_canonical(&tape);
    }

    #[test]
    fn get_and_window_agree_on_both_sides() {
        let mut tape = RunTape::with_input(&[s(1), s(2), s(2), s(0), s(3)]);
        for _ in 0..3 {
            tape.apply(s(4), Dir::L, 1);
        }
        tape.apply(s(1), Dir::R, 1);
        for from in -8..8 {
            for len in [0, 1, 2, 5, 13] {
                let expected: Vec<Symbol> = (0..len).map(|i| tape.get(from + i)).collect();
                let len = usize::try_from(len).unwrap();
                assert_eq!(tape.window(from, len), expected, "from {from}, len {len}");
            }
        }
        assert_eq!(tape.get(i64::MIN), Symbol::BLANK);
        assert_eq!(tape.get(i64::MAX), Symbol::BLANK);
        assert_eq!(tape.window(i64::MAX - 1, 4), vec![Symbol::BLANK; 4]);
    }

    fn step_strategy() -> impl Strategy<Value = Vec<(u8, Dir)>> {
        let dir = prop_oneof![Just(Dir::L), Just(Dir::R), Just(Dir::N)];
        prop::collection::vec((0u8..3, dir), 0..200)
    }

    proptest! {
        #[test]
        fn run_tape_matches_dense_tape(
            input in prop::collection::vec(0u8..3, 0..12),
            steps in step_strategy(),
        ) {
            let input: Vec<Symbol> = input.into_iter().map(s).collect();
            let mut dense = Tape::with_input(&input);
            let mut runs = RunTape::with_input(&input);
            for (write, dir) in steps {
                dense.write(s(write));
                dense.shift(dir);
                runs.apply(s(write), dir, 1);
                assert_canonical(&runs);
                prop_assert_eq!(runs.read(), dense.read());
                prop_assert_eq!(runs.head(), dense.head());
            }
            prop_assert_eq!(runs.non_blank(), dense.non_blank());
            prop_assert_eq!(runs.count_non_blank(), dense.count_non_blank());
            let from = runs.head() - 30;
            prop_assert_eq!(runs.window(from, 60), dense.window(from, 60));
        }
    }

    fn sweeper() -> Machine {
        // Sweeps right over a block of x, turns, sweeps left over the y written and halts on
        // the blank before the block: 2n + 1 steps in 3 macro steps for n > 1.
        Machine::builder("sweep")
            .transition("r", 'x', "r", 'y', Dir::R)
            .transition("r", '␣', "l", '␣', Dir::L)
            .transition("l", 'y', "l", 'y', Dir::L)
            .build()
            .unwrap()
    }

    #[test]
    fn macro_steps_cover_runs() {
        let m = sweeper();
        let input = m.encode_input("xxxxxxxx").unwrap();
        let mut sim = ChainSim::new(&m, &input).unwrap();
        let mut events = Vec::new();
        let outcome = sim.run_until(1000, |e| {
            events.push((e.steps_before, e.repeats, e.from.index()));
            false
        });
        assert_eq!(
            outcome,
            Outcome::Halted {
                steps: 17,
                state: State::new(1)
            }
        );
        assert_eq!(events, [(0, 8, 0), (8, 1, 0), (9, 8, 1)]);
        assert_eq!(sim.macro_steps(), 3);
        assert_eq!(sim.extent(), -1..=8);
        assert_eq!(sim.tape().head(), -1);
        assert_eq!(sim.tape().count_non_blank(), 8);
    }

    #[test]
    fn partial_chains_stop_exactly_at_the_fuel_limit() {
        let m = sweeper();
        let input = m.encode_input("xxxxxxxx").unwrap();
        let mut sim = ChainSim::new(&m, &input).unwrap();
        assert_eq!(sim.run(3), Outcome::OutOfFuel { steps: 3 });
        assert_eq!(sim.tape().head(), 3);
        assert_eq!(sim.tape().read(), m.alphabet().symbol('x').unwrap());
        assert_eq!(sim.run(0), Outcome::OutOfFuel { steps: 3 });
        assert_eq!(sim.run(5), Outcome::OutOfFuel { steps: 8 });
        assert_eq!(sim.tape().head(), 8);
        assert_eq!(sim.run(4), Outcome::OutOfFuel { steps: 12 });
        assert_eq!(sim.tape().head(), 4);
        assert!(matches!(sim.run(100), Outcome::Halted { steps: 17, .. }));
    }

    #[test]
    fn stop_is_checked_before_the_macro_step() {
        let m = sweeper();
        let input = m.encode_input("xxx").unwrap();
        let mut sim = ChainSim::new(&m, &input).unwrap();
        let l = m.state_by_name("l").unwrap();
        let outcome = sim.run_until(100, |e| e.from == l);
        assert_eq!(outcome, Outcome::Stopped { steps: 4 });
        assert_eq!(sim.state(), l);
        // The breakpoint stays in place until a step is made.
        assert_eq!(
            sim.run_until(100, |e| e.from == l),
            Outcome::Stopped { steps: 4 }
        );
        assert_eq!(sim.run(1), Outcome::OutOfFuel { steps: 5 });
    }

    #[test]
    fn walking_off_over_blanks_never_halts() {
        let m = Machine::builder("walk")
            .transition("q", 'x', "q", '␣', Dir::R)
            .transition("q", '␣', "q", '1', Dir::R)
            .build()
            .unwrap();
        let input = m.encode_input("xx␣x").unwrap();
        let mut sim = ChainSim::new(&m, &input).unwrap();
        assert_eq!(
            sim.run(100),
            Outcome::NeverHalts {
                steps: 4,
                proof: NonHaltProof::ChainToInfinity {
                    state: State::new(0),
                    dir: Dir::R
                }
            }
        );
        assert_eq!(sim.macro_steps(), 3);
    }

    #[test]
    fn staying_in_place_never_halts() {
        let m = Machine::builder("stay")
            .transition("q", '␣', "q", '␣', Dir::N)
            .build()
            .unwrap();
        let mut sim = ChainSim::new(&m, &[]).unwrap();
        assert!(matches!(
            sim.run(5),
            Outcome::NeverHalts {
                steps: 0,
                proof: NonHaltProof::StayLoop { .. }
            }
        ));
    }

    #[test]
    fn moves_in_place_are_single_steps() {
        let m = Machine::builder("toggle")
            .transition("q", '␣', "q", '1', Dir::N)
            .transition("q", '1', "q", '␣', Dir::N)
            .build()
            .unwrap();
        let mut sim = ChainSim::new(&m, &[]).unwrap();
        assert_eq!(sim.run(7), Outcome::OutOfFuel { steps: 7 });
        assert_eq!(sim.macro_steps(), 7);
        assert_eq!(sim.tape().read(), m.alphabet().symbol('1').unwrap());
    }

    #[test]
    fn several_tapes_are_rejected() {
        let m = Machine::builder("two").tapes(2).state("q").build().unwrap();
        assert_eq!(
            ChainSim::new(&m, &[]).unwrap_err(),
            NotSingleTape { tapes: 2 }
        );
    }
}
