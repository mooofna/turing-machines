//! Halting, cyclers and translated cyclers, found by simulation (`loop1_decider` of Coq-BB5).
//!
//! # The algorithm
//!
//! [`Loops`] simulates the machine from the blank tape for N steps and records the
//! configurations c(0), …, c(N), each reduced to its *state-symbol pair* (the state and the
//! symbol under the head), the head position and two flags saying whether the head is on the
//! leftmost or the rightmost cell visited so far. If an undefined transition is met at some
//! c(i) with i < N, the machine halts. Otherwise the history is searched, once, after N steps,
//! for a period p with 1 ≤ p ≤ N/2 such that
//!
//! 1. c(N), c(N − p) and c(N − 2p) have the same state-symbol pair (a cheap filter), and
//! 2. walking backwards, c(N − j) and c(N − p − j) have the same state-symbol pair for
//!    j = 0, 1, …, J, for some J ≥ p at which, writing a = N − p − J and b = N − J = a + p,
//!    * d = 0 (with d the head displacement from c(N − p) to c(N)): the head is on the same
//!      cell at times a and b (a *cycler*), or
//!    * d > 0: the head is on the rightmost visited cell at time a and further right at
//!      time b (a *translated cycler* moving right), or
//!    * d < 0: the mirror image.
//!
//! This is a faithful port of `verify_loop1`, `find_loop1` and `loop1_decider0` in
//! `Decider_Loop.v`; it inspects only the history of state-symbol pairs, never the tape.
//!
//! # Soundness
//!
//! Write σ(t) for the state-symbol pair at time t and x(t) for the head position. The move made
//! at time t is a function of σ(t). From condition 2, σ(a + i) = σ(b + i) for 0 ≤ i ≤ J, hence
//! the moves agree as well and x(b + i) − x(a + i) = x(b) − x(a) = d for 0 ≤ i ≤ J + 1. We show
//! by induction on t that σ(t + p) = σ(t) and x(t + p) = x(t) + d for every t ≥ a; then every
//! state-symbol pair after time a already occurred at a time where δ was defined, so the
//! machine never halts.
//!
//! For t ≤ a + J the claim is given. Let t > a + J ≥ a + p = b and assume the claim below t.
//! The states agree because σ(t + p − 1) = σ(t − 1). For the symbols let x = x(t) and
//! y = x + d = x(t + p), and let u be the last time before t + p at which the head was on y.
//!
//! * If u ≥ b, then u − p ≥ a, the head was on x at time u − p, and it was never on x
//!   between u − p and t (any such visit would shift to a visit of y between u and t + p).
//!   So y was last written at time u and x at time u − p, by the same transition, and the
//!   symbols agree.
//! * Otherwise y was not visited during [b, t + p), so x was not visited during [a, t).
//!   If d = 0, this contradicts x(t − p) = x(t) with t − p ≥ a. If d > 0 and x ≤ R, where
//!   R = x(a) is the rightmost cell visited up to time a, then x < R because x was not
//!   visited at time a, and since the head moves one cell at a time, every position during
//!   [a, t) is right of x; among them is x(t − p), and x(t) = x(t − p) + d > x is a
//!   contradiction. If x > R, the head reached x for the first time ever at time t, so every
//!   earlier position is left of x, every position before t + p is left of y, and both cells
//!   are blank.
//!
//! The case d < 0 is symmetric. The condition J ≥ p is what makes t − p ≥ a in the
//! argument above; the halting answer is correct by construction.

use crate::compact::{CompactTm, HALT};
use crate::pipeline::{Decider, DeciderId, Verdict};

/// One recorded configuration.
#[derive(Copy, Clone, Debug, Default)]
struct Record {
    /// `state << 3 | symbol`.
    pair: u8,
    /// Head position, as an index into the tape buffer.
    pos: u32,
    /// The head is on the rightmost cell visited so far (Coq: `r = nil`).
    rightmost: bool,
    /// The head is on the leftmost cell visited so far (Coq: `l = nil`).
    leftmost: bool,
}

/// Simulates N steps and then looks for cyclers and translated cyclers.
///
/// ```
/// use turing_bb::deciders::Loops;
/// use turing_bb::{CompactTm, Decider, DeciderId, Verdict};
/// let mut loops = Loops::new(107);
/// // A translated cycler: it writes ones to the right forever.
/// let tm: CompactTm = "1RA---".parse().unwrap();
/// assert_eq!(loops.decide(&tm), Verdict::NonHalting(DeciderId::Loops));
/// // The busy beaver for two states reaches its undefined transition after 5 steps.
/// let bb2: CompactTm = "1RB1LB_1LA---".parse().unwrap();
/// assert_eq!(loops.decide(&bb2), Verdict::Halts { steps: 5, ones: 4, at: (1, 1) });
/// ```
#[derive(Debug, Clone)]
pub struct Loops {
    steps: u32,
    tape: Vec<u8>,
    history: Vec<Record>,
}

impl Loops {
    /// A decider that simulates `steps` steps (Coq-BB5's parameter of `loop1_decider`).
    pub fn new(steps: u32) -> Self {
        Self {
            steps,
            tape: Vec::new(),
            history: Vec::new(),
        }
    }

    /// The number of simulated steps.
    pub fn steps(&self) -> u32 {
        self.steps
    }

    /// `find_loop1_0`: searches the recorded history c(0), …, c(N) for a period.
    fn find_loop(&self) -> bool {
        let h = &self.history;
        let n = h.len() - 1;
        (1..=n / 2)
            .any(|p| h[n].pair == h[n - p].pair && h[n].pair == h[n - 2 * p].pair && self.verify(p))
    }

    /// `verify_loop1` for period `p`, starting from c(N) and c(N − p).
    fn verify(&self, p: usize) -> bool {
        let h = &self.history;
        let n = h.len() - 1;
        let d = i64::from(h[n].pos) - i64::from(h[n - p].pos);
        for j in 0..=n - p {
            let (later, earlier) = (&h[n - j], &h[n - p - j]);
            if later.pair != earlier.pair {
                return false;
            }
            if j >= p {
                let found = match d.signum() {
                    0 => earlier.pos == later.pos,
                    1 => earlier.rightmost && earlier.pos < later.pos,
                    _ => earlier.leftmost && later.pos < earlier.pos,
                };
                if found {
                    return true;
                }
            }
        }
        false
    }
}

impl Decider for Loops {
    fn id(&self) -> DeciderId {
        DeciderId::Loops
    }

    fn name(&self) -> String {
        format!("LOOP1_params_{}", self.steps)
    }

    fn decide(&mut self, tm: &CompactTm) -> Verdict {
        let n = self.steps as usize;
        let symbols = usize::from(tm.symbols);
        self.tape.clear();
        self.tape.resize(2 * n + 3, 0);
        self.history.clear();
        let mut head = n + 1;
        let (mut min, mut max) = (head, head);
        let mut state = 0u8;
        for step in 0..n {
            let symbol = self.tape[head];
            self.history.push(Record {
                pair: (state << 3) | symbol,
                pos: u32::try_from(head).unwrap_or(u32::MAX),
                rightmost: head == max,
                leftmost: head == min,
            });
            let byte = tm.table[usize::from(state) * symbols + usize::from(symbol)];
            let next = byte >> 4;
            if next >= HALT {
                let ones = self.tape[min..=max].iter().filter(|&&c| c != 0).count();
                return Verdict::Halts {
                    steps: step as u64,
                    ones: ones as u64,
                    at: (state, symbol),
                };
            }
            self.tape[head] = byte & 7;
            if byte & 8 == 0 {
                head -= 1;
                min = min.min(head);
            } else {
                head += 1;
                max = max.max(head);
            }
            state = next;
        }
        self.history.push(Record {
            pair: (state << 3) | self.tape[head],
            pos: u32::try_from(head).unwrap_or(u32::MAX),
            rightmost: head == max,
            leftmost: head == min,
        });
        if self.find_loop() {
            Verdict::NonHalting(DeciderId::Loops)
        } else {
            Verdict::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decide(steps: u32, tm: &str) -> Verdict {
        Loops::new(steps).decide(&tm.parse().unwrap())
    }

    #[test]
    fn the_root_halts_at_once() {
        assert_eq!(
            decide(6, "------_------"),
            Verdict::Halts {
                steps: 0,
                ones: 0,
                at: (0, 0)
            }
        );
    }

    #[test]
    fn finds_the_champions() {
        assert_eq!(
            decide(21, "1RB---_1LB0RC_1LC1LA"),
            Verdict::Halts {
                steps: 20,
                ones: 5,
                at: (0, 1)
            }
        );
        assert_eq!(
            decide(107, "1RB1LB_1LA0LC_---1LD_1RD0RA"),
            Verdict::Halts {
                steps: 106,
                ones: 12,
                at: (2, 0)
            }
        );
    }

    #[test]
    fn halting_transitions_count_as_halting() {
        assert_eq!(
            decide(6, "1RB1LB_1LA1RZ"),
            Verdict::Halts {
                steps: 5,
                ones: 4,
                at: (1, 1)
            }
        );
    }

    #[test]
    fn detects_cyclers() {
        // A0 -> 0RB, B0 -> 0LA: bounces between two cells forever.
        assert_eq!(
            decide(6, "0RB---_0LA---"),
            Verdict::NonHalting(DeciderId::Loops)
        );
    }

    #[test]
    fn needs_enough_steps() {
        // Too few steps to see two full periods.
        assert_eq!(decide(1, "0RB---_0LA---"), Verdict::Unknown);
        assert_eq!(decide(0, "0RB---_0LA---"), Verdict::Unknown);
    }

    #[test]
    fn detects_translated_cyclers_both_ways() {
        assert_eq!(decide(6, "1RA---"), Verdict::NonHalting(DeciderId::Loops));
        assert_eq!(
            decide(20, "0RB---_1LB---"),
            Verdict::NonHalting(DeciderId::Loops)
        );
    }
}
