//! n-gram closed position sets (`Decider_NGramCPS.v` of Coq-BB5, after Nathan Fenner).
//!
//! # The abstraction
//!
//! A *local context* (Coq: `MidWord`) is a state q, the symbol m under the head, the `len_l`
//! cells left of the head and the `len_r` cells right of it. The decider builds three finite
//! sets: a set M of local contexts, a set L of left n-grams (words of length `len_l`) and a set
//! R of right n-grams (words of length `len_r`). Initially M holds the blank context in state A
//! and L and R hold the blank n-grams.
//!
//! Processing a context (q, l, m, r) whose transition is δ(q, m) = (q', b, R) does two things.
//! The left n-gram l moves one cell further away from the head: it is added to L. The head
//! moves onto the first cell of r, the written symbol b joins the left context and the
//! farthest cell of l drops out of it, and a new cell enters the right context from the far
//! end: for every symbol x such that the rest of r followed by x is a right n-gram in R, the
//! resulting context in state q' is added to M. Moves to the left are symmetric. Contexts are
//! processed until a pass over all of M changes nothing.
//!
//! # Soundness
//!
//! Call a configuration of the machine *covered* if its local context is in M and every window
//! of `len_l` consecutive cells strictly left of the left context (and of `len_r` cells right of
//! the right context), read away from the head, is in L (respectively R). The blank
//! configuration is covered. If M, L and R are closed under the processing above, the
//! successor of a covered configuration is covered: the window that leaves the right context
//! is in R because it was a window of the old configuration or of the old context, the window
//! that falls off the left context was added to L when its context was processed, and the
//! new right context is one of the contexts added for the symbol x actually found on the tape,
//! whose window is in R by the covering hypothesis. By induction every reachable
//! configuration is covered, and since no context in M has an undefined transition, the
//! machine never halts. This is `NGramCPS_decider_spec` in Coq-BB5.
//!
//! # Faithfulness
//!
//! The decider reproduces Coq-BB5's search order and its use of the gas parameter exactly, so
//! that it resolves exactly the machines Coq-BB5 resolves with the same parameters: all known
//! contexts are revisited in passes, most recent first, newly found contexts are processed
//! next, and every processed context and every completed pass costs one unit of gas.
//! Encountering a halting context makes the answer *unknown* at once; Coq-BB5 continues until
//! the gas runs out, with the same answer.

use rustc_hash::{FxHashMap, FxHashSet};

use super::augment::{AbstractMachine, History, Lru, Plain, Sym};
use crate::compact::CompactTm;
use crate::pipeline::{Decider, DeciderId, Verdict};

/// The longest n-gram supported.
pub const MAX_NGRAM: usize = 8;

/// An n-gram or n-gram prefix, padded with zeros.
type Gram = [Sym; MAX_NGRAM];

/// A local context: state, symbol under the head and the two n-grams, nearest cell first.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct Context {
    state: u8,
    mid: Sym,
    left: Gram,
    right: Gram,
}

/// A set of n-grams, stored by their first n − 1 symbols (nearest the head first) with the list
/// of possible last symbols in insertion order (Coq: `xset_impl`).
#[derive(Clone, Debug, Default)]
struct GramSet {
    map: FxHashMap<Gram, Vec<Sym>>,
}

impl GramSet {
    fn clear(&mut self) {
        self.map.clear();
    }

    /// Inserts the n-gram `gram[..len]`; returns whether it was present already.
    fn insert(&mut self, gram: &Gram, len: usize) -> bool {
        let mut prefix = *gram;
        let last = prefix[len - 1];
        prefix[len - 1] = 0;
        let lasts = self.map.entry(prefix).or_default();
        if lasts.contains(&last) {
            true
        } else {
            lasts.push(last);
            false
        }
    }

    /// The possible last symbols after `prefix` (padded with zeros), most recent first.
    fn lasts(&self, prefix: &Gram) -> impl Iterator<Item = Sym> + '_ {
        self.map
            .get(prefix)
            .into_iter()
            .flat_map(|v| v.iter().rev().copied())
    }
}

/// The local contexts found so far, in insertion order (Coq: `mset_impl`), and the stack of
/// contexts still to process in the current pass.
#[derive(Clone, Debug, Default)]
struct ContextSet {
    list: Vec<Context>,
    seen: FxHashSet<Context>,
    stack: Vec<Context>,
}

impl ContextSet {
    fn clear(&mut self) {
        self.list.clear();
        self.seen.clear();
        self.stack.clear();
    }

    /// Adds a context and schedules it for processing; returns whether it was present already
    /// (Coq: `mset_ins0`, with the new context prepended to the work list).
    fn insert(&mut self, context: Context) -> bool {
        if self.seen.insert(context) {
            self.list.push(context);
            self.stack.push(context);
            false
        } else {
            true
        }
    }

    /// Schedules every known context, the most recent one first.
    fn start_pass(&mut self) {
        self.stack.clear();
        self.stack.extend_from_slice(&self.list);
    }
}

/// The n-gram closed position set decider over the abstract machine `M`.
///
/// ```
/// use turing_bb::deciders::NGramCps;
/// use turing_bb::{CompactTm, Decider, DeciderId, Verdict};
/// // A machine from the BB(2) enumeration that Coq-BB5 resolves with n-gram CPS.
/// let tm: CompactTm = "0RB---_1LA0LA".parse().unwrap();
/// let mut cps = NGramCps::plain(1, 1, 100);
/// assert_eq!(cps.decide(&tm), Verdict::NonHalting(DeciderId::NGramCps));
/// ```
#[derive(Clone, Debug)]
pub struct NGramCps<M> {
    machine: M,
    id: DeciderId,
    name: String,
    len_l: usize,
    len_r: usize,
    gas: u64,
    contexts: ContextSet,
    lefts: GramSet,
    rights: GramSet,
}

impl NGramCps<Plain> {
    /// The plain decider with left and right n-grams of the given lengths and `gas` units of
    /// work (`NGramCPS_decider_impl2 len_l len_r gas`).
    ///
    /// # Panics
    ///
    /// Panics unless both lengths are between 1 and [`MAX_NGRAM`].
    pub fn plain(len_l: usize, len_r: usize, gas: u64) -> Self {
        Self::with_machine(
            Plain::default(),
            DeciderId::NGramCps,
            format!("NGRAM_CPS_IMPL2_params_{len_l}_{len_r}_{gas}"),
            len_l,
            len_r,
            gas,
        )
    }
}

impl NGramCps<History> {
    /// The decider on the machine augmented with the last `history` updates of each cell
    /// (`NGramCPS_decider_impl1 history len_l len_r gas`).
    ///
    /// # Panics
    ///
    /// Panics unless both lengths are between 1 and [`MAX_NGRAM`].
    pub fn history(history: usize, len_l: usize, len_r: usize, gas: u64) -> Self {
        Self::with_machine(
            History::new(history),
            DeciderId::NGramCpsHistory,
            format!("NGRAM_CPS_IMPL1_params_{history}_{len_l}_{len_r}_{gas}"),
            len_l,
            len_r,
            gas,
        )
    }
}

impl NGramCps<Lru> {
    /// The decider on the machine augmented with a least-recently-used list of updates
    /// (`NGramCPS_LRU_decider len_l len_r gas`).
    ///
    /// # Panics
    ///
    /// Panics unless both lengths are between 1 and [`MAX_NGRAM`].
    pub fn lru(len_l: usize, len_r: usize, gas: u64) -> Self {
        Self::with_machine(
            Lru::new(),
            DeciderId::NGramCpsLru,
            format!("NGRAM_CPS_LRU_params_{len_l}_{len_r}_{gas}"),
            len_l,
            len_r,
            gas,
        )
    }
}

impl<M: AbstractMachine> NGramCps<M> {
    /// A decider over any abstract machine, reported as `id` with the given name.
    ///
    /// # Panics
    ///
    /// Panics unless both lengths are between 1 and [`MAX_NGRAM`].
    pub fn with_machine(
        machine: M,
        id: DeciderId,
        name: String,
        len_l: usize,
        len_r: usize,
        gas: u64,
    ) -> Self {
        assert!(
            (1..=MAX_NGRAM).contains(&len_l) && (1..=MAX_NGRAM).contains(&len_r),
            "n-gram lengths are between 1 and {MAX_NGRAM}"
        );
        Self {
            machine,
            id,
            name,
            len_l,
            len_r,
            gas,
            contexts: ContextSet::default(),
            lefts: GramSet::default(),
            rights: GramSet::default(),
        }
    }

    /// Processes one context (Coq: `update_AES_MidWord`). Returns `None` if its transition
    /// halts, otherwise whether nothing changed.
    fn process(&mut self, c: Context) -> Option<bool> {
        let step = self.machine.step(c.state, c.mid)?;
        // Moving right, the left n-gram moves away from the head and a new cell enters the
        // right context at its far end; moving left, the other way round.
        let (behind, ahead, len_b, len_a, behind_set, ahead_set) = if step.right {
            (
                c.left,
                c.right,
                self.len_l,
                self.len_r,
                &mut self.lefts,
                &self.rights,
            )
        } else {
            (
                c.right,
                c.left,
                self.len_r,
                self.len_l,
                &mut self.rights,
                &self.lefts,
            )
        };
        let mut unchanged = behind_set.insert(&behind, len_b);
        let mut new_behind = [0; MAX_NGRAM];
        new_behind[0] = step.write;
        new_behind[1..len_b].copy_from_slice(&behind[..len_b - 1]);
        let mut prefix = [0; MAX_NGRAM];
        prefix[..len_a - 1].copy_from_slice(&ahead[1..len_a]);
        for x in ahead_set.lasts(&prefix) {
            let mut new_ahead = prefix;
            new_ahead[len_a - 1] = x;
            let (left, right) = if step.right {
                (new_behind, new_ahead)
            } else {
                (new_ahead, new_behind)
            };
            unchanged &= self.contexts.insert(Context {
                state: step.next,
                mid: ahead[0],
                left,
                right,
            });
        }
        Some(unchanged)
    }

    /// Runs the search; `true` if a closed set without halting contexts was found
    /// (Coq: `NGramCPS_decider`).
    fn closed(&mut self, tm: &CompactTm) -> bool {
        self.machine.reset(tm);
        self.contexts.clear();
        self.lefts.clear();
        self.rights.clear();
        let blank = [0; MAX_NGRAM];
        self.lefts.insert(&blank, self.len_l);
        self.rights.insert(&blank, self.len_r);
        let init = Context {
            state: 0,
            mid: 0,
            left: blank,
            right: blank,
        };
        self.contexts.insert(init);
        let mut gas = self.gas;
        // Every pass costs at least one unit of gas, so `gas` passes are enough (Coq bounds
        // the passes by the same number).
        for _ in 0..self.gas {
            // A pass visits every known context, most recent first (Coq: `update_AES`).
            self.contexts.start_pass();
            let mut unchanged = true;
            loop {
                if gas == 0 {
                    return false;
                }
                gas -= 1;
                let Some(context) = self.contexts.stack.pop() else {
                    break;
                };
                match self.process(context) {
                    None => return false,
                    Some(same) => unchanged &= same,
                }
            }
            if unchanged {
                // `check_InitES_InAES`: true by construction, checked for faithfulness.
                return self.contexts.seen.contains(&init);
            }
        }
        false
    }
}

impl<M: AbstractMachine + Send> Decider for NGramCps<M> {
    fn id(&self) -> DeciderId {
        self.id
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn decide(&mut self, tm: &CompactTm) -> Verdict {
        if self.closed(tm) {
            Verdict::NonHalting(self.id)
        } else {
            Verdict::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halting_machines_are_never_closed() {
        for text in ["1RB1LB_1LA---", "------_------", "1RB---_1LB0RC_1LC1LA"] {
            let tm: CompactTm = text.parse().unwrap();
            assert_eq!(NGramCps::plain(2, 2, 1000).decide(&tm), Verdict::Unknown);
            assert_eq!(
                NGramCps::history(2, 2, 2, 1000).decide(&tm),
                Verdict::Unknown
            );
            assert_eq!(NGramCps::lru(2, 2, 1000).decide(&tm), Verdict::Unknown);
        }
    }

    #[test]
    fn gas_limits_the_search() {
        let tm: CompactTm = "0RB---_1LA0LA".parse().unwrap();
        assert_eq!(NGramCps::plain(1, 1, 3).decide(&tm), Verdict::Unknown);
    }

    #[test]
    fn names_follow_coq() {
        assert_eq!(
            NGramCps::history(2, 3, 3, 1600).name(),
            "NGRAM_CPS_IMPL1_params_2_3_3_1600"
        );
        assert_eq!(
            NGramCps::lru(2, 2, 10000).name(),
            "NGRAM_CPS_LRU_params_2_2_10000"
        );
    }
}
