//! Repeated word lists (`Decider_RepWL.v` of Coq-BB5, by mxdys).
//!
//! # The abstraction
//!
//! The tape is cut into words of `len` cells, counted from the head outwards, and runs of equal
//! words are compressed: a run of k copies of a word w is written wᵏ while k is below the
//! parameter m (`min_rep`), and wᵐ⁺ ("at least m copies") from m copies on. An abstract
//! configuration consists of the word list behind the head, the word list in front of it (both
//! nearest word first, the blank tail omitted), the state and the direction the head is facing.
//!
//! One abstract step pops the first word w from the list in front: the blank tail yields a
//! blank word, wᵏ yields w followed by wᵏ⁻¹, and wᵏ⁺ yields either w followed by wᵏ⁻¹ or w
//! followed by wᵏ⁺, so the step may branch. The machine is then simulated on the finite word
//! alone, entering it from the side of the head, for at most `max_steps` steps until the head
//! leaves the word. If it leaves on the far side, the updated word is pushed onto the list
//! behind the head; if it leaves on the near side, it is pushed onto the remaining list in
//! front, the two lists swap roles and the facing direction turns. Pushing a word onto a list
//! whose first run has the same word increments that run (saturating into wᵐ⁺), and a
//! blank word pushed onto an empty list is absorbed by the blank tail.
//!
//! # Soundness
//!
//! Each abstract configuration stands for the set of concrete configurations whose tape
//! matches the two lists. Every concrete configuration matching an abstract one reaches, after
//! at least one step, a configuration matching one of its successors: the pops enumerate all
//! ways the matching word sequence can begin, the word simulation is exact, and pushes only
//! enlarge the represented set. The search explores the successors from the blank
//! configuration; if it exhausts them within `gas` expansions and never meets an undefined
//! transition or a word simulation that exceeds `max_steps`, the explored set is closed, contains
//! the blank tape and no halting configuration, so the machine never halts (`T_decider_spec`
//! and `RepWL_step_spec` in Coq-BB5). Because the reachable set is explored completely, the
//! answer does not depend on the order of the search.

use rustc_hash::FxHashSet;

use crate::compact::{CompactTm, HALT};
use crate::pipeline::{Decider, DeciderId, Verdict};

/// The longest word supported.
pub const MAX_WORD: usize = 16;

/// A word of at most [`MAX_WORD`] symbols, nearest the head first, padded with zeros.
type Word = [u8; MAX_WORD];

/// `wᵏ` (exactly k copies) or `wᵏ⁺` (at least k copies).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct Run {
    word: Word,
    count: u32,
    at_least: bool,
}

/// An abstract configuration. Both lists keep the run nearest the head at the end.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Abstract {
    behind: Vec<Run>,
    ahead: Vec<Run>,
    state: u8,
    facing_right: bool,
}

/// The repeated word list decider.
///
/// ```
/// use turing_bb::deciders::RepWl;
/// use turing_bb::{CompactTm, Decider, DeciderId, Verdict};
/// // One of the two BB(4) machines that Coq-BB5 resolves with repeated word lists.
/// let tm: CompactTm = "1RB1LA_1LA0RC_1LD1RC_---0LA".parse().unwrap();
/// let mut repwl = RepWl::new(4, 3, 320, 10_000);
/// assert_eq!(repwl.decide(&tm), Verdict::NonHalting(DeciderId::RepWl));
/// ```
#[derive(Clone, Debug)]
pub struct RepWl {
    len: usize,
    min_rep: u32,
    max_steps: u32,
    gas: u64,
    seen: FxHashSet<Abstract>,
    queue: Vec<Abstract>,
}

impl RepWl {
    /// Words of `len` cells, runs of `min_rep` or more copies become unbounded, at most
    /// `max_steps` machine steps inside one word, at most `gas` abstract configurations
    /// (`RepWL_ES_decider len min_rep max_steps gas`).
    ///
    /// # Panics
    ///
    /// Panics unless `1 <= len <= MAX_WORD` and `min_rep >= 1`.
    pub fn new(len: usize, min_rep: u32, max_steps: u32, gas: u64) -> Self {
        assert!(
            (1..=MAX_WORD).contains(&len),
            "words have 1 to {MAX_WORD} cells"
        );
        assert!(min_rep >= 1, "runs repeat at least once");
        Self {
            len,
            min_rep,
            max_steps,
            gas,
            seen: FxHashSet::default(),
            queue: Vec::new(),
        }
    }

    /// The ways to split the first word off `list` (Coq: `pop`): the word and the possible
    /// remainders, or `None` for an empty run (which never occurs).
    fn pop(list: &[Run]) -> Option<(Word, Vec<Vec<Run>>)> {
        let Some((&top, rest)) = list.split_last() else {
            return Some(([0; MAX_WORD], vec![Vec::new()]));
        };
        let left = top.count.checked_sub(1)?;
        let mut exact = rest.to_vec();
        if left > 0 {
            exact.push(Run {
                word: top.word,
                count: left,
                at_least: false,
            });
        }
        let mut options = vec![exact];
        if top.at_least {
            options.push(list.to_vec());
        }
        Some((top.word, options))
    }

    /// Pushes `word` onto `list` (Coq: `push`).
    fn push(&self, list: &mut Vec<Run>, word: Word) {
        match list.last_mut() {
            Some(top) if top.word == word => {
                let count = top.count + 1;
                if count < self.min_rep {
                    top.count = count;
                } else {
                    top.count = self.min_rep;
                    top.at_least = true;
                }
            }
            None if word.iter().all(|&c| c == 0) => {}
            _ => list.push(Run {
                word,
                count: 1,
                at_least: false,
            }),
        }
    }

    /// Runs the machine on `word` entered from its near end (Coq: `WordUpdate`). Returns the
    /// new state, the updated word read from the side where the head left, and whether it left
    /// on the near side; `None` if it halts or stays inside for more than `max_steps` steps.
    fn update(
        &self,
        tm: &CompactTm,
        state: u8,
        word: Word,
        facing_right: bool,
    ) -> Option<(u8, Word, bool)> {
        let len = self.len;
        let mut cells = word;
        let mut state = state;
        let mut i = 0;
        for _ in 0..self.max_steps {
            let byte = tm.byte(state, cells[i]);
            let next = byte >> 4;
            if next >= HALT {
                return None;
            }
            cells[i] = byte & 7;
            state = next;
            let forward = (byte & 8 != 0) == facing_right;
            if forward {
                if i + 1 == len {
                    let mut out = [0; MAX_WORD];
                    for (k, slot) in out.iter_mut().take(len).enumerate() {
                        *slot = cells[len - 1 - k];
                    }
                    return Some((state, out, false));
                }
                i += 1;
            } else {
                if i == 0 {
                    return Some((state, cells, true));
                }
                i -= 1;
            }
        }
        None
    }

    /// The successors of `x` (Coq: `RepWL_step`), or `None` if a branch fails.
    fn successors(&self, tm: &CompactTm, x: &Abstract) -> Option<Vec<Abstract>> {
        let (word, remainders) = Self::pop(&x.ahead)?;
        let (state, written, back) = self.update(tm, x.state, word, x.facing_right)?;
        Some(
            remainders
                .into_iter()
                .map(|rest| {
                    if back {
                        let mut behind = rest;
                        self.push(&mut behind, written);
                        Abstract {
                            behind,
                            ahead: x.behind.clone(),
                            state,
                            facing_right: !x.facing_right,
                        }
                    } else {
                        let mut behind = x.behind.clone();
                        self.push(&mut behind, written);
                        Abstract {
                            behind,
                            ahead: rest,
                            state,
                            facing_right: x.facing_right,
                        }
                    }
                })
                .collect(),
        )
    }

    /// Searches for a closed set (Coq: `T_decider0`).
    fn closed(&mut self, tm: &CompactTm) -> bool {
        self.seen.clear();
        self.queue.clear();
        let init = Abstract {
            behind: Vec::new(),
            ahead: Vec::new(),
            state: 0,
            facing_right: true,
        };
        self.seen.insert(init.clone());
        self.queue.push(init);
        for _ in 0..self.gas {
            let Some(x) = self.queue.pop() else {
                return true;
            };
            let Some(next) = self.successors(tm, &x) else {
                return false;
            };
            for y in next {
                if !self.seen.contains(&y) {
                    self.seen.insert(y.clone());
                    self.queue.push(y);
                }
            }
        }
        self.queue.is_empty()
    }
}

impl Decider for RepWl {
    fn id(&self) -> DeciderId {
        DeciderId::RepWl
    }

    fn name(&self) -> String {
        format!(
            "REPWL_params_{}_{}_{}_{}",
            self.len, self.min_rep, self.max_steps, self.gas
        )
    }

    fn decide(&mut self, tm: &CompactTm) -> Verdict {
        if self.closed(tm) {
            Verdict::NonHalting(DeciderId::RepWl)
        } else {
            Verdict::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(cells: &[u8]) -> Word {
        let mut w = [0; MAX_WORD];
        w[..cells.len()].copy_from_slice(cells);
        w
    }

    #[test]
    fn push_compresses_runs() {
        let decider = RepWl::new(2, 3, 320, 100);
        let mut list = Vec::new();
        decider.push(&mut list, word(&[0, 0]));
        assert!(list.is_empty(), "blank words vanish into the blank tail");
        for _ in 0..5 {
            decider.push(&mut list, word(&[1, 0]));
        }
        assert_eq!(
            list,
            vec![Run {
                word: word(&[1, 0]),
                count: 3,
                at_least: true
            }]
        );
        decider.push(&mut list, word(&[0, 0]));
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn pop_branches_on_unbounded_runs() {
        let run = Run {
            word: word(&[1, 1]),
            count: 3,
            at_least: true,
        };
        let (w, options) = RepWl::pop(&[run]).unwrap();
        assert_eq!(w, word(&[1, 1]));
        assert_eq!(
            options,
            vec![
                vec![Run {
                    count: 2,
                    at_least: false,
                    ..run
                }],
                vec![run]
            ]
        );
        let (w, options) = RepWl::pop(&[]).unwrap();
        assert_eq!((w, options), (word(&[]), vec![Vec::new()]));
    }

    #[test]
    fn halting_machines_are_never_closed() {
        let tm: CompactTm = "1RB1LB_1LA---".parse().unwrap();
        assert_eq!(RepWl::new(2, 3, 320, 1000).decide(&tm), Verdict::Unknown);
    }

    #[test]
    fn decides_both_golden_machines() {
        for text in ["1RB1LA_1LA0RC_1LD1RC_---0LA", "1RB0RB_1LC1RB_---0LD_1RA1LD"] {
            let tm: CompactTm = text.parse().unwrap();
            assert_eq!(
                RepWl::new(4, 3, 320, 10_000).decide(&tm),
                Verdict::NonHalting(DeciderId::RepWl),
                "{text}"
            );
        }
    }
}
