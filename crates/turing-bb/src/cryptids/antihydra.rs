//! Antihydra, `1RB1RA_0LC1LE_1LD1LC_1LA0LB_1LF1RE_---0RA` (mxdys, 28 June 2024).
//!
//! Write A(a, b) for the configuration `0^∞ 1^a 0 1^b E> 0^∞`: state E with the head on the
//! first blank right of the block 1^b. From the blank tape the machine reaches A(0, 4) after 11
//! steps and then follows
//!
//! * A(a, 2b) → A(a + 2, 3b + 2) in 2a + 3b² + 12b + 11 steps,
//! * A(a + 1, 2b + 1) → A(a, 3b + 3) in 3b² + 12b + 5 steps,
//! * A(0, 2b + 1) halts, after 3b² + 9b steps for b ≥ 1 (the final step into the undefined
//!   transition counted).
//!
//! These step counts were checked by simulation; the halting rule is never reached in
//! practice. With h = b + 4 and c = a this is the program `h = 8, c = 0; while c >= 0: c += 2
//! if h is even else -1; h += h / 2 (rounded down)`: an even h = 2b + 4 becomes 3b + 6 = 3h/2
//! and an odd h = 2b + 5 becomes 3b + 7 = h + ⌊h/2⌋. Whether c ever becomes negative is open; it
//! is related to Mahler's 3/2 problem.
//!
//! [`MachineCheck`] runs the machine itself and checks every rule application against these
//! formulas; [`RuleWalk`] iterates the program with big integers far beyond what the machine
//! can reach.

use num_bigint::BigUint;
use turing_core::{ChainSim, Machine, Outcome, Run, State, Symbol};

use crate::compact::CompactTm;

/// The machine in standard format.
pub const STANDARD: &str = "1RB1RA_0LC1LE_1LD1LC_1LA0LB_1LF1RE_---0RA";

/// Machine steps the check stays below, so that step counts are exact in JavaScript.
pub const MAX_CHECKED_STEPS: u64 = 1 << 53;

/// Steps from the blank tape to A(0, 4).
const FIRST_STEPS: u64 = 11;

/// One rule application observed on the machine itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// 0 for A(0, 4), then 1, 2, ...
    pub iteration: u32,
    /// The parameter a after the iteration.
    pub a: u64,
    /// The parameter b after the iteration.
    pub b: u64,
    /// Machine steps at the moment the configuration A(a, b) was reached.
    pub steps: u64,
    /// Steps taken since the previous row.
    pub delta: u64,
    /// Steps predicted by the rule for this iteration.
    pub predicted: u64,
    /// Whether a, b and the step delta match the rule.
    pub agrees: bool,
}

/// What the rules predict after A(a, b): the next (a, b) and the steps to get there, or `None`
/// if the machine halts.
fn predict(a: u64, b: u64) -> Option<(u64, u64, u64)> {
    let k = u128::from(b / 2);
    let steps = |s: u128| u64::try_from(s).unwrap_or(u64::MAX);
    if b.is_multiple_of(2) {
        let s = 2 * u128::from(a) + 3 * k * k + 12 * k + 11;
        Some((a + 2, 3 * (b / 2) + 2, steps(s)))
    } else if a > 0 {
        let s = 3 * k * k + 12 * k + 5;
        Some((a - 1, 3 * (b / 2) + 3, steps(s)))
    } else {
        None
    }
}

/// Reads A(a, b) off a configuration in state E reading a blank, if it has that form.
fn configuration(left: &[Run], right: &[Run]) -> Option<(u64, u64)> {
    if !right.is_empty() {
        return None;
    }
    let one = Symbol::new(1);
    let mut runs = left.iter().rev().peekable();
    let b = runs
        .next_if(|run| run.symbol == one)
        .map_or(0, |run| run.len);
    match (runs.next(), runs.next(), runs.next()) {
        (None, _, _) => Some((0, b)),
        (Some(gap), Some(block), None)
            if gap.symbol == Symbol::BLANK && gap.len == 1 && block.symbol == one =>
        {
            Some((block.len, b))
        }
        _ => None,
    }
}

/// Runs the machine with chain steps and reports each rule application.
///
/// ```
/// use turing_bb::cryptids::antihydra::MachineCheck;
/// let mut check = MachineCheck::new();
/// let rows = check.advance(1_000);
/// assert_eq!((rows[0].a, rows[0].b, rows[0].steps), (0, 4, 11));
/// assert_eq!((rows[1].a, rows[1].b, rows[1].delta), (2, 8, 47));
/// assert!(rows.iter().all(|r| r.agrees));
/// ```
#[derive(Debug)]
pub struct MachineCheck {
    pub(crate) done: bool,
    sim: ChainSim<Machine>,
    /// The next row's iteration number.
    iteration: u32,
    /// The previous row: a, b and the step count.
    last: Option<(u64, u64, u64)>,
}

impl MachineCheck {
    /// Starts the machine on the blank tape.
    ///
    /// # Panics
    ///
    /// Never: the machine is a fixed valid single-tape machine.
    pub fn new() -> Self {
        let tm: CompactTm = STANDARD.parse().expect("Antihydra is a valid machine");
        let sim = ChainSim::new(tm.to_machine(), &[]).expect("a single-tape machine");
        Self {
            done: false,
            sim,
            iteration: 0,
            last: None,
        }
    }

    /// Runs at most `max_macro_steps` macro steps and returns the rows found meanwhile.
    pub fn advance(&mut self, max_macro_steps: u64) -> Vec<Row> {
        const E: State = State::new(4);
        let mut rows = Vec::new();
        let mut budget = max_macro_steps;
        while budget > 0 && !self.done {
            let mut executed = 0;
            let mut candidate = false;
            let fuel = MAX_CHECKED_STEPS.saturating_sub(self.sim.steps());
            let outcome = self.sim.run_until(fuel, |event| {
                if executed >= budget {
                    return true;
                }
                if event.from == E && event.read == Symbol::BLANK {
                    candidate = true;
                    return true;
                }
                executed += 1;
                false
            });
            budget -= executed;
            match outcome {
                Outcome::Stopped { .. } if candidate => {
                    let tape = self.sim.tape();
                    if let Some((a, b)) = configuration(tape.left(), tape.right()) {
                        let row = self.row(a, b);
                        self.done |= !row.agrees;
                        rows.push(row);
                    }
                    // Step past the configuration: E reading 0 is a single step.
                    self.sim.run(1);
                    budget = budget.saturating_sub(1);
                }
                Outcome::Stopped { .. } => {}
                Outcome::Halted { .. } | Outcome::NeverHalts { .. } | Outcome::OutOfFuel { .. } => {
                    self.done = true;
                }
            }
        }
        rows
    }

    /// The row for A(`a`, `b`) reached now.
    fn row(&mut self, a: u64, b: u64) -> Row {
        let steps = self.sim.steps();
        let (expected, delta, predicted) = match self.last {
            None => (Some((0, 4)), steps, FIRST_STEPS),
            Some((pa, pb, previous)) => {
                let next = predict(pa, pb);
                (
                    next.map(|(na, nb, _)| (na, nb)),
                    steps - previous,
                    next.map_or(0, |(_, _, s)| s),
                )
            }
        };
        let row = Row {
            iteration: self.iteration,
            a,
            b,
            steps,
            delta,
            predicted,
            agrees: expected == Some((a, b)) && delta == predicted,
        };
        self.iteration += 1;
        self.last = Some((a, b, steps));
        row
    }

    /// Whether the check stopped (machine step count beyond 2^53 or a mismatch).
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// Machine steps so far.
    pub fn steps(&self) -> u64 {
        self.sim.steps()
    }
}

impl Default for MachineCheck {
    fn default() -> Self {
        Self::new()
    }
}

/// A sample of the rule walk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalkSample {
    /// The iteration.
    pub iteration: u64,
    /// The balance c.
    pub balance: i64,
    /// The bit length of h.
    pub bits: u64,
}

/// How many samples [`RuleWalk`] keeps at most; it thins them out by halves when full.
pub const SAMPLE_CAPACITY: usize = 4096;

/// Iterates the abstract rule with big integers, incrementally.
///
/// After iteration i, c and h − 4 equal a and b of row i of [`MachineCheck`].
///
/// ```
/// use turing_bb::cryptids::antihydra::RuleWalk;
/// let mut walk = RuleWalk::new();
/// walk.advance(4);
/// let s = walk.summary(10);
/// // h: 8, 12, 18, 27, 40; c: 0, 2, 4, 6, 5.
/// assert_eq!((s.iterations, s.balance, s.evens, s.odds), (4, 5, 3, 1));
/// assert!(!s.halted);
/// ```
#[derive(Debug)]
pub struct RuleWalk {
    pub(crate) iteration: u64,
    h: BigUint,
    c: i64,
    halted: bool,
    evens: u64,
    odds: u64,
    min_balance: Option<i64>,
    /// Samples at every `stride`-th iteration, starting with iteration 0.
    samples: Vec<WalkSample>,
    stride: u64,
}

impl RuleWalk {
    /// Starts at h = 8, c = 0.
    pub fn new() -> Self {
        let h = BigUint::from(8u32);
        let first = WalkSample {
            iteration: 0,
            balance: 0,
            bits: h.bits(),
        };
        Self {
            iteration: 0,
            h,
            c: 0,
            halted: false,
            evens: 0,
            odds: 0,
            min_balance: None,
            samples: vec![first],
            stride: 1,
        }
    }

    /// Runs `iterations` more iterations, or fewer if c becomes negative (the machine would
    /// halt).
    pub fn advance(&mut self, iterations: u64) {
        for _ in 0..iterations {
            if self.halted {
                return;
            }
            if self.h.bit(0) {
                self.c -= 1;
                self.odds += 1;
            } else {
                self.c += 2;
                self.evens += 1;
            }
            let half = &self.h >> 1u32;
            self.h += half;
            self.iteration += 1;
            self.min_balance = Some(self.min_balance.map_or(self.c, |m| m.min(self.c)));
            self.halted = self.c < 0;
            if self.iteration.is_multiple_of(self.stride) {
                self.samples.push(self.sample());
                if self.samples.len() >= SAMPLE_CAPACITY {
                    self.stride *= 2;
                    let stride = self.stride;
                    self.samples.retain(|s| s.iteration.is_multiple_of(stride));
                }
            }
        }
    }

    /// The current state as a sample.
    fn sample(&self) -> WalkSample {
        WalkSample {
            iteration: self.iteration,
            balance: self.c,
            bits: self.h.bits(),
        }
    }

    /// The summary with at most `samples` samples.
    ///
    /// The samples are evenly spaced over the iterations done so far and include the first
    /// and the current one.
    pub fn summary(&self, samples: usize) -> WalkSummary {
        let mut pool = self.samples.clone();
        if pool.last().is_none_or(|s| s.iteration != self.iteration) {
            pool.push(self.sample());
        }
        let picked = match samples {
            0 => Vec::new(),
            1 => vec![self.sample()],
            n if pool.len() <= n => pool,
            n => (0..n)
                .map(|i| pool[i * (pool.len() - 1) / (n - 1)].clone())
                .collect(),
        };
        WalkSummary {
            iterations: self.iteration,
            halted: self.halted,
            evens: self.evens,
            odds: self.odds,
            balance: self.c,
            min_balance: self.min_balance.unwrap_or(0),
            bits: self.h.bits(),
            samples: picked,
        }
    }

    /// h, the current value of the iterated number.
    pub fn h(&self) -> &BigUint {
        &self.h
    }
}

impl Default for RuleWalk {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of a [`RuleWalk`].
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct WalkSummary {
    /// Iterations done.
    pub iterations: u64,
    /// Whether c became negative (the machine would halt).
    pub halted: bool,
    /// How many iterations had h even.
    pub evens: u64,
    /// How many iterations had h odd.
    pub odds: u64,
    /// The current balance c.
    pub balance: i64,
    /// The smallest balance seen after iteration 0 (0 before the first iteration).
    pub min_balance: i64,
    /// The bit length of h.
    pub bits: u64,
    /// Evenly spaced samples for plotting, at most the requested number.
    pub samples: Vec<WalkSample>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predictions_follow_the_rules() {
        assert_eq!(predict(0, 4), Some((2, 8, 47)));
        assert_eq!(predict(6, 23), Some((5, 36, 500)));
        assert_eq!(predict(0, 23), None);
    }

    #[test]
    fn configurations_are_recognised() {
        let one = |len| Run {
            symbol: Symbol::new(1),
            len,
        };
        let gap = Run {
            symbol: Symbol::BLANK,
            len: 1,
        };
        assert_eq!(configuration(&[one(4)], &[]), Some((0, 4)));
        assert_eq!(configuration(&[one(2), gap, one(8)], &[]), Some((2, 8)));
        assert_eq!(configuration(&[one(2), gap], &[]), Some((2, 0)));
        assert_eq!(configuration(&[one(2), gap, one(8)], &[one(1)]), None);
        let wide = Run {
            symbol: Symbol::BLANK,
            len: 2,
        };
        assert_eq!(configuration(&[one(2), wide, one(8)], &[]), None);
        assert_eq!(
            configuration(&[one(1), gap, one(2), gap, one(8)], &[]),
            None
        );
    }

    #[test]
    fn the_walk_samples_evenly() {
        let mut walk = RuleWalk::new();
        walk.advance(10_000);
        let s = walk.summary(11);
        assert_eq!(s.samples.len(), 11);
        assert_eq!(s.samples[0].iteration, 0);
        assert_eq!(s.samples[10].iteration, 10_000);
        assert!(walk.samples.len() < SAMPLE_CAPACITY);
        assert_eq!(s.evens + s.odds, 10_000);
        assert_eq!(
            s.balance,
            2 * i64::try_from(s.evens).unwrap() - i64::try_from(s.odds).unwrap()
        );
        assert!(s.min_balance >= 0);
    }
}
