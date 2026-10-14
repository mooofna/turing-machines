//! Machines over augmented alphabets for the n-gram CPS decider.
//!
//! An *augmentation* replaces every tape symbol a by a pair (a, h), where h is a bounded record
//! of the past updates of that cell, and lets the augmented machine write (b, h') whenever the
//! original machine writes b. The augmented machine makes exactly the same moves and state
//! changes as the original one from the blank tape (0, ε), so one halts if and only if the
//! other does, and a non-halting proof for the augmented machine is a non-halting proof for
//! the original (`TM_history_NonHaltsFromInit` and `TM_history_LRU_NonHaltsFromInit` in
//! Coq-BB5's `TM.v`). The extra information makes the n-gram abstraction finer.
//!
//! Augmented symbols are interned: each distinct pair gets a small integer, with 0 for the
//! blank (0, ε), so that the decider works on plain integers.

use rustc_hash::FxHashMap;

use crate::compact::{CompactTm, HALT, MAX_STATES};

/// A symbol of an abstract machine: an index into its alphabet, 0 for the blank.
pub type Sym = u32;

/// The result of one transition of an abstract machine.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Step {
    /// The next state.
    pub next: u8,
    /// Whether the head moves right.
    pub right: bool,
    /// The symbol written.
    pub write: Sym,
}

/// A machine over a possibly augmented alphabet whose symbols are created on demand.
pub trait AbstractMachine {
    /// Prepares the abstraction of `tm` and forgets every previously interned symbol.
    fn reset(&mut self, tm: &CompactTm);

    /// δ(`state`, `symbol`), or `None` if the original machine halts there (an undefined
    /// transition or a transition to `Z`).
    fn step(&mut self, state: u8, symbol: Sym) -> Option<Step>;
}

/// The machine itself, without augmentation.
#[derive(Clone, Debug, Default)]
pub struct Plain {
    tm: Option<CompactTm>,
}

impl AbstractMachine for Plain {
    fn reset(&mut self, tm: &CompactTm) {
        self.tm = Some(*tm);
    }

    #[inline]
    fn step(&mut self, state: u8, symbol: Sym) -> Option<Step> {
        let tm = self.tm.as_ref()?;
        let symbol = u8::try_from(symbol).ok()?;
        decode(tm.byte(state, symbol))
    }
}

/// Decodes a packed transition byte; `None` if it halts.
#[inline]
fn decode(byte: u8) -> Option<Step> {
    let next = byte >> 4;
    (next < HALT).then_some(Step {
        next,
        right: byte & 8 != 0,
        write: Sym::from(byte & 7),
    })
}

/// How the history of a cell is updated by a transition.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Policy {
    /// Keep the last `n` updates, most recent first (`TM_history n` in Coq-BB5).
    Queue(usize),
    /// Keep every distinct update once, most recently used first (`TM_history_LRU`).
    Lru,
}

/// An augmented machine: symbols are (original symbol, history of updates of the cell), and
/// an update is the state-symbol pair (q, a) of the transition that wrote the cell.
#[derive(Clone, Debug)]
struct Augmented {
    policy: Policy,
    tm: Option<CompactTm>,
    /// Interned symbols: byte 0 is the original symbol, then one byte `q << 3 | a` per update,
    /// most recent first.
    symbols: Vec<Box<[u8]>>,
    index: FxHashMap<Box<[u8]>, Sym>,
    /// `cache[symbol * MAX_STATES + state]`: 0 if not computed yet, 1 if halting, otherwise
    /// `2 + (write << 4 | right << 3 | next)`.
    cache: Vec<u64>,
    scratch: Vec<u8>,
}

impl Augmented {
    fn new(policy: Policy) -> Self {
        Self {
            policy,
            tm: None,
            symbols: Vec::new(),
            index: FxHashMap::default(),
            cache: Vec::new(),
            scratch: Vec::new(),
        }
    }

    fn intern(&mut self, key: &[u8]) -> Sym {
        if let Some(&id) = self.index.get(key) {
            return id;
        }
        let id = Sym::try_from(self.symbols.len()).unwrap_or(Sym::MAX);
        self.symbols.push(key.into());
        self.index.insert(key.into(), id);
        self.cache.extend(core::iter::repeat_n(0, MAX_STATES));
        id
    }

    fn reset(&mut self, tm: &CompactTm) {
        self.tm = Some(*tm);
        self.symbols.clear();
        self.index.clear();
        self.cache.clear();
        // The blank (0, ε) gets symbol 0.
        self.intern(&[0]);
    }

    fn step(&mut self, state: u8, symbol: Sym) -> Option<Step> {
        let slot = symbol as usize * MAX_STATES + usize::from(state);
        match self.cache.get(slot).copied() {
            Some(1) => return None,
            Some(packed) if packed >= 2 => {
                let packed = packed - 2;
                return Some(Step {
                    next: (packed & 7) as u8,
                    right: packed & 8 != 0,
                    write: Sym::try_from(packed >> 4).unwrap_or(Sym::MAX),
                });
            }
            _ => {}
        }
        let tm = self.tm.as_ref()?;
        let key = self.symbols.get(symbol as usize)?;
        let original = key[0];
        let Some(step) = decode(tm.byte(state, original)) else {
            self.cache[slot] = 1;
            return None;
        };
        let update = (state << 3) | original;
        let mut scratch = core::mem::take(&mut self.scratch);
        scratch.clear();
        scratch.push(u8::try_from(step.write).unwrap_or(0));
        scratch.push(update);
        match self.policy {
            Policy::Queue(n) => {
                scratch.truncate(1 + n);
                let room = n.saturating_sub(1);
                scratch.extend(key[1..].iter().take(room));
            }
            Policy::Lru => scratch.extend(key[1..].iter().filter(|&&u| u != update)),
        }
        let write = self.intern(&scratch);
        self.scratch = scratch;
        let result = Step { write, ..step };
        self.cache[slot] =
            2 + ((u64::from(write) << 4) | (u64::from(result.right) << 3) | u64::from(result.next));
        Some(result)
    }
}

/// The machine augmented with the last `n` updates of each cell (`TM_history n` in Coq-BB5,
/// used by `NGramCPS_decider_impl1`).
///
/// A cell written by the transition δ(q, a) = (q', b, D) changes from (a, h) to
/// (b, first n entries of (q, a) · h).
#[derive(Clone, Debug)]
pub struct History(Augmented);

impl History {
    /// Keeps the last `n` updates of each cell.
    pub fn new(n: usize) -> Self {
        Self(Augmented::new(Policy::Queue(n)))
    }
}

impl AbstractMachine for History {
    fn reset(&mut self, tm: &CompactTm) {
        self.0.reset(tm);
    }

    fn step(&mut self, state: u8, symbol: Sym) -> Option<Step> {
        self.0.step(state, symbol)
    }
}

/// The machine augmented with a least-recently-used list of updates of each cell
/// (`TM_history_LRU` in Coq-BB5, used by `NGramCPS_LRU_decider`).
///
/// A cell written by the transition δ(q, a) = (q', b, D) changes from (a, h) to (b, (q, a) · h'),
/// where h' is h without (q, a). The list never repeats an entry, so it has at most
/// states × symbols entries.
#[derive(Clone, Debug)]
pub struct Lru(Augmented);

impl Lru {
    /// A fresh LRU augmentation.
    pub fn new() -> Self {
        Self(Augmented::new(Policy::Lru))
    }
}

impl Default for Lru {
    fn default() -> Self {
        Self::new()
    }
}

impl AbstractMachine for Lru {
    fn reset(&mut self, tm: &CompactTm) {
        self.0.reset(tm);
    }

    fn step(&mut self, state: u8, symbol: Sym) -> Option<Step> {
        self.0.step(state, symbol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tm(s: &str) -> CompactTm {
        s.parse().unwrap()
    }

    #[test]
    fn plain_reads_the_table() {
        let mut m = Plain::default();
        m.reset(&tm("1RB0LA_1LA---"));
        assert_eq!(
            m.step(0, 1),
            Some(Step {
                next: 0,
                right: false,
                write: 0
            })
        );
        assert_eq!(m.step(1, 1), None);
    }

    #[test]
    fn history_keeps_the_last_updates() {
        let mut machine = History::new(2);
        machine.reset(&tm("1RA1RA"));
        let first = machine.step(0, 0).unwrap().write; // (1, [A0])
        let second = machine.step(0, first).unwrap().write; // (1, [A1, A0])
        let third = machine.step(0, second).unwrap().write; // (1, [A1, A1])
        let fourth = machine.step(0, third).unwrap().write; // (1, [A1, A1]) again
        assert_eq!(third, fourth);
        assert_eq!(machine.0.symbols[first as usize].as_ref(), &[1, 0]);
        assert_eq!(machine.0.symbols[second as usize].as_ref(), &[1, 1, 0]);
        assert_eq!(machine.0.symbols[third as usize].as_ref(), &[1, 1, 1]);
    }

    #[test]
    fn history_zero_forgets_everything() {
        let mut m = History::new(0);
        m.reset(&tm("1RA1RA"));
        let a = m.step(0, 0).unwrap().write;
        assert_eq!(m.0.symbols[a as usize].as_ref(), &[1]);
    }

    #[test]
    fn lru_moves_entries_to_the_front() {
        let mut machine = Lru::new();
        machine.reset(&tm("1RB0RB_1LA0LA"));
        let first = machine.step(0, 0).unwrap().write; // A0 writes 1: (1, [A0])
        let second = machine.step(0, first).unwrap().write; // A1 writes 0: (0, [A1, A0])
        let third = machine.step(1, second).unwrap().write; // B0 writes 1: (1, [B0, A1, A0])
        let fourth = machine.step(0, third).unwrap().write; // A1 writes 0: (0, [A1, B0, A0])
        assert_eq!(machine.0.symbols[fourth as usize].as_ref(), &[0, 1, 8, 0]);
    }
}
