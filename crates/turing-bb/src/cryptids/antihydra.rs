//! Antihydra, `1RB1RA_0LC1LE_1LD1LC_1LA0LB_1LF1RE_---0RA` (mxdys, 28 June 2024).
//!
//! Write A(a, b) for the configuration `0^∞ 1^a 0 1^b E> 0^∞`. The machine reaches A(0, 4)
//! and then follows
//!
//! * A(a, 2b) → A(a + 2, 3b + 2) in 2a + 3b² + 12b + 11 steps,
//! * A(a + 1, 2b + 1) → A(a, 3b + 3) in 3b² + 12b + 5 steps,
//! * A(0, 2b + 1) halts.
//!
//! With h = b + 4 and c = a this is the program `h = 8, c = 0; while c >= 0: c += 2 if h is even
//! else -1; h += h / 2 (rounded down)`. Whether c ever becomes negative is open; it is related to
//! Mahler's 3/2 problem.

/// The machine in standard format.
pub const STANDARD: &str = "1RB1RA_0LC1LE_1LD1LC_1LA0LB_1LF1RE_---0RA";

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

/// Runs the machine with chain steps and reports each rule application.
#[derive(Debug)]
pub struct MachineCheck {
    pub(crate) done: bool,
}

impl MachineCheck {
    /// Starts the machine on the blank tape.
    pub fn new() -> Self {
        todo!("A3")
    }

    /// Runs at most `max_macro_steps` macro steps and returns the rows found meanwhile.
    pub fn advance(&mut self, max_macro_steps: u64) -> Vec<Row> {
        let _ = max_macro_steps;
        todo!("A3")
    }

    /// Whether the check stopped (machine step count beyond 2^53 or a mismatch).
    pub fn is_done(&self) -> bool {
        self.done
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

/// Iterates the abstract rule with big integers, incrementally.
#[derive(Debug)]
pub struct RuleWalk {
    pub(crate) iteration: u64,
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
    /// The smallest balance seen after iteration 0.
    pub min_balance: i64,
    /// The bit length of h.
    pub bits: u64,
    /// Evenly spaced samples for plotting, at most the requested number.
    pub samples: Vec<WalkSample>,
}

impl RuleWalk {
    /// Starts at h = 8, c = 0.
    pub fn new() -> Self {
        todo!("A3")
    }

    /// Runs `iterations` more iterations.
    pub fn advance(&mut self, iterations: u64) {
        let _ = iterations;
        todo!("A3")
    }

    /// The summary with at most `samples` samples.
    pub fn summary(&self, samples: usize) -> WalkSummary {
        let _ = samples;
        todo!("A3")
    }
}

impl Default for RuleWalk {
    fn default() -> Self {
        Self::new()
    }
}
