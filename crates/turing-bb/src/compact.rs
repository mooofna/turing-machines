//! A compact, copyable representation of small two-way machines for enumeration.
//!
//! # Encoding
//!
//! Every entry δ(q, a) of the transition table is one byte:
//!
//! | bits | meaning |
//! |---|---|
//! | 0 to 2 | the symbol written, `0..=7` |
//! | 3 | the move, `0` for left and `1` for right |
//! | 4 to 6 | the next state: `0..=5` for A to F, [`HALT`] (6) for the halting state `Z`, [`UNDEFINED`] (7) for an undefined entry |
//! | 7 | always `0` |
//!
//! An undefined entry is always stored as the byte `0x70`, and table entries beyond
//! `states × symbols` are undefined as well, so two machines are equal exactly when their
//! tables are equal byte for byte.

use core::fmt;
use core::str::FromStr;

use turing_core::{Dir, Kind, Machine, MachineBuilder};

/// The largest number of states of a [`CompactTm`].
pub const MAX_STATES: usize = 6;
/// The largest number of symbols of a [`CompactTm`].
pub const MAX_SYMBOLS: usize = 8;

/// The value of the next-state field that stands for the halting state `Z`.
pub const HALT: u8 = 6;
/// The value of the next-state field that marks an undefined entry.
pub const UNDEFINED: u8 = 7;

/// The byte of an undefined entry.
pub(crate) const UNDEFINED_BYTE: u8 = UNDEFINED << 4;

/// The state after a transition.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Next {
    /// Continue in state `0..states` (A, B, ...).
    State(u8),
    /// Execute the transition, count it, then halt (written `Z`).
    Halt,
}

/// A defined transition of a [`CompactTm`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CompactAction {
    /// The symbol written.
    pub write: u8,
    /// `Dir::L` or `Dir::R`.
    pub dir: Dir,
    /// The next state.
    pub next: Next,
}

impl CompactAction {
    /// Packs the action into one byte, see the [module documentation](self).
    ///
    /// # Panics
    ///
    /// Panics if the move is `Dir::N`, the symbol does not fit into three bits or the state is
    /// not below [`MAX_STATES`].
    pub(crate) fn encode(self) -> u8 {
        assert!(usize::from(self.write) < MAX_SYMBOLS, "symbol out of range");
        let right = match self.dir {
            Dir::L => 0,
            Dir::R => 1,
            Dir::N => panic!("compact machines move left or right"),
        };
        let next = match self.next {
            Next::State(q) => {
                assert!(usize::from(q) < MAX_STATES, "state out of range");
                q
            }
            Next::Halt => HALT,
        };
        self.write | (right << 3) | (next << 4)
    }

    /// Unpacks a byte produced by [`CompactAction::encode`]; `None` for an undefined entry.
    pub(crate) fn decode(byte: u8) -> Option<Self> {
        let next = match byte >> 4 {
            UNDEFINED => return None,
            HALT => Next::Halt,
            q => Next::State(q),
        };
        Some(Self {
            write: byte & 7,
            dir: if byte & 8 == 0 { Dir::L } else { Dir::R },
            next,
        })
    }
}

/// A machine with at most [`MAX_STATES`] states and [`MAX_SYMBOLS`] symbols, blank 0,
/// packed into 50 bytes. Undefined transitions halt without being counted.
///
/// It reads and writes the bbchallenge standard format:
///
/// ```
/// use turing_bb::CompactTm;
/// let bb2: CompactTm = "1RB1LB_1LA1RZ".parse().unwrap();
/// assert_eq!(bb2.states(), 2);
/// assert_eq!(bb2.to_string(), "1RB1LB_1LA1RZ");
/// ```
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct CompactTm {
    pub(crate) states: u8,
    pub(crate) symbols: u8,
    /// Index `state * symbols + symbol`; see [`CompactTm::get`] for the encoding.
    pub(crate) table: [u8; MAX_STATES * MAX_SYMBOLS],
}

/// A text that is not a valid [`CompactTm`].
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid machine string at byte {at}: {message}")]
pub struct ParseCompactError {
    /// Byte offset.
    pub at: usize,
    /// What was expected.
    pub message: &'static str,
}

impl ParseCompactError {
    const fn new(at: usize, message: &'static str) -> Self {
        Self { at, message }
    }
}

impl CompactTm {
    /// A machine with every transition undefined.
    ///
    /// # Panics
    ///
    /// Panics if `states` or `symbols` exceed the limits or are 0.
    ///
    /// ```
    /// use turing_bb::CompactTm;
    /// let root = CompactTm::new(2, 2);
    /// assert_eq!(root.to_string(), "------_------");
    /// assert_eq!(root.undefined(), 4);
    /// ```
    pub fn new(states: u8, symbols: u8) -> Self {
        assert!(
            (1..=MAX_STATES).contains(&usize::from(states)),
            "a compact machine has 1 to {MAX_STATES} states"
        );
        assert!(
            (1..=MAX_SYMBOLS).contains(&usize::from(symbols)),
            "a compact machine has 1 to {MAX_SYMBOLS} symbols"
        );
        Self {
            states,
            symbols,
            table: [UNDEFINED_BYTE; MAX_STATES * MAX_SYMBOLS],
        }
    }

    /// The number of states.
    pub fn states(&self) -> u8 {
        self.states
    }

    /// The number of symbols.
    pub fn symbols(&self) -> u8 {
        self.symbols
    }

    /// The packed byte of δ(q, a), see the [module documentation](self).
    ///
    /// # Panics
    ///
    /// Panics if `q` or `a` is out of range.
    #[inline]
    pub(crate) fn byte(&self, q: u8, a: u8) -> u8 {
        assert!(
            q < self.states && a < self.symbols,
            "δ({q}, {a}) is out of range"
        );
        self.table[usize::from(q) * usize::from(self.symbols) + usize::from(a)]
    }

    /// δ(q, a), or `None` if undefined.
    ///
    /// The table stores one byte per entry: the written symbol in bits 0 to 2, the move in
    /// bit 3 (1 = right) and the next state in bits 4 to 6, where 6 is the halting state `Z`
    /// and 7 marks an undefined entry.
    ///
    /// # Panics
    ///
    /// Panics if `q` or `a` is out of range.
    ///
    /// ```
    /// use turing_bb::{CompactTm, Next};
    /// use turing_core::Dir;
    /// let tm: CompactTm = "1RB---_1LA1RZ".parse().unwrap();
    /// let a0 = tm.get(0, 0).unwrap();
    /// assert_eq!((a0.write, a0.dir, a0.next), (1, Dir::R, Next::State(1)));
    /// assert_eq!(tm.get(0, 1), None);
    /// assert_eq!(tm.get(1, 1).unwrap().next, Next::Halt);
    /// ```
    #[inline]
    pub fn get(&self, q: u8, a: u8) -> Option<CompactAction> {
        CompactAction::decode(self.byte(q, a))
    }

    /// Sets or clears δ(q, a).
    ///
    /// # Panics
    ///
    /// Panics if `q` or `a` is out of range, or if the action writes a symbol or enters a
    /// state that the machine does not have, or moves `N`.
    pub fn set(&mut self, q: u8, a: u8, action: Option<CompactAction>) {
        assert!(
            q < self.states && a < self.symbols,
            "δ({q}, {a}) is out of range"
        );
        let byte = match action {
            None => UNDEFINED_BYTE,
            Some(action) => {
                assert!(
                    action.write < self.symbols,
                    "the machine has no symbol {}",
                    action.write
                );
                if let Next::State(next) = action.next {
                    assert!(next < self.states, "the machine has no state {next}");
                }
                action.encode()
            }
        };
        self.table[usize::from(q) * usize::from(self.symbols) + usize::from(a)] = byte;
    }

    /// The number of undefined transitions.
    pub fn undefined(&self) -> usize {
        self.entries().filter(|&b| b >> 4 == UNDEFINED).count()
    }

    /// The bytes of the `states × symbols` entries in table order.
    pub(crate) fn entries(&self) -> impl Iterator<Item = u8> + '_ {
        self.table[..usize::from(self.states) * usize::from(self.symbols)]
            .iter()
            .copied()
    }

    /// The highest state index that a new transition may enter in tree normal form: one past
    /// the largest state entered by a defined transition (at least B), capped at the last
    /// state. This is the pointer `TNF_ptr` of Coq-BB5.
    pub(crate) fn next_state_bound(&self) -> u8 {
        let largest = self
            .entries()
            .map(|b| b >> 4)
            .filter(|&next| next < HALT)
            .max();
        let bound = largest.map_or(1, |q| q + 1);
        bound.min(self.states - 1)
    }

    /// The equivalent [`Machine`] (kind beaver, blank `0`, states `A`, `B`, ..., halting state
    /// `Z` if used).
    ///
    /// The machine's name is the standard format string.
    ///
    /// # Panics
    ///
    /// Never for a machine built through this type's API; a failure of the builder would be
    /// a bug.
    pub fn to_machine(&self) -> Machine {
        let glyphs: Vec<char> = (0..self.symbols).map(digit).collect();
        let mut names: Vec<String> = (0..self.states).map(|q| letter(q).to_string()).collect();
        let uses_halt = self.entries().any(|b| b >> 4 == HALT);
        if uses_halt {
            names.push("Z".into());
        }
        let mut builder = MachineBuilder::new(self.to_string())
            .kind(Kind::Beaver)
            .blank('0')
            .tape_alphabet(glyphs.iter().copied())
            .declare_states(names.iter().map(String::as_str))
            .start("A");
        if uses_halt {
            builder = builder.final_state("Z");
        }
        for q in 0..self.states {
            for a in 0..self.symbols {
                if let Some(action) = self.get(q, a) {
                    let next = match action.next {
                        Next::State(n) => letter(n).to_string(),
                        Next::Halt => "Z".into(),
                    };
                    builder = builder.transition(
                        &names[usize::from(q)],
                        digit(a),
                        &next,
                        digit(action.write),
                        action.dir,
                    );
                }
            }
        }
        match builder.build() {
            Ok(machine) => machine,
            Err(errors) => panic!("a compact machine always builds: {errors}"),
        }
    }
}

/// The glyph of symbol `a`.
fn digit(a: u8) -> char {
    char::from(b'0' + a)
}

/// The letter of state `q`.
fn letter(q: u8) -> char {
    char::from(b'A' + q)
}

impl TryFrom<&Machine> for CompactTm {
    type Error = ParseCompactError;

    /// Converts a single-tape machine that moves only left and right.
    ///
    /// The start state becomes A and the other states keep their order. States other than
    /// the start state without any defined transition become the halting state `Z`: entering
    /// one of them and then halting takes as many steps as a transition to `Z`. Symbols keep
    /// their alphabet index, so the blank becomes 0. Errors carry `at = 0`.
    fn try_from(machine: &Machine) -> Result<Self, Self::Error> {
        if machine.tapes() != 1 {
            return Err(ParseCompactError::new(0, "compact machines have one tape"));
        }
        let symbols = machine.num_symbols();
        if !(1..=MAX_SYMBOLS).contains(&symbols) {
            return Err(ParseCompactError::new(
                0,
                "too many symbols for a compact machine",
            ));
        }
        let total = machine.num_states();
        let start = machine.start().index();
        let mut has_transitions = vec![false; total];
        for transition in machine.transitions() {
            has_transitions[transition.from.index()] = true;
        }
        // Map every state to its compact index, or to `None` for the halting state.
        let mut order = vec![start];
        order.extend((0..total).filter(|&q| q != start && has_transitions[q]));
        if order.len() > MAX_STATES {
            return Err(ParseCompactError::new(
                0,
                "too many states for a compact machine",
            ));
        }
        let mut index: Vec<Option<u8>> = vec![None; total];
        for (i, &q) in order.iter().enumerate() {
            index[q] = u8::try_from(i).ok();
        }
        let states = u8::try_from(order.len()).unwrap_or(u8::MAX);
        let symbols = u8::try_from(symbols).unwrap_or(u8::MAX);
        let mut tm = Self::new(states, symbols);
        for transition in machine.transitions() {
            let Some(from) = index[transition.from.index()] else {
                continue;
            };
            let action = transition.action;
            let dir = action.dir[0];
            if dir == Dir::N {
                return Err(ParseCompactError::new(
                    0,
                    "compact machines move left or right",
                ));
            }
            let next = index[action.next.index()].map_or(Next::Halt, Next::State);
            tm.set(
                from,
                transition.read[0].get(),
                Some(CompactAction {
                    write: action.write[0].get(),
                    dir,
                    next,
                }),
            );
        }
        Ok(tm)
    }
}

impl FromStr for CompactTm {
    type Err = ParseCompactError;

    /// Parses the standard format: one group of three characters per symbol and state,
    /// states separated by `_`. `---` is an undefined transition; a next-state letter beyond
    /// the last state (usually `Z`) is the halting state.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        let mut groups: Vec<(usize, &[u8])> = Vec::new();
        let mut start = 0;
        for (i, &b) in bytes.iter().enumerate() {
            if b == b'_' {
                groups.push((start, &bytes[start..i]));
                start = i + 1;
            }
        }
        groups.push((start, &bytes[start..]));
        if groups.len() > MAX_STATES {
            return Err(ParseCompactError::new(0, "at most 6 states"));
        }
        let width = groups[0].1.len();
        if width == 0 || !width.is_multiple_of(3) {
            return Err(ParseCompactError::new(
                width,
                "each state lists three characters per symbol",
            ));
        }
        if width / 3 > MAX_SYMBOLS {
            return Err(ParseCompactError::new(0, "at most 8 symbols"));
        }
        let states = u8::try_from(groups.len()).unwrap_or(u8::MAX);
        let symbols = u8::try_from(width / 3).unwrap_or(u8::MAX);
        let mut tm = Self::new(states, symbols);
        for (q, &(offset, group)) in (0..states).zip(&groups) {
            if group.len() != width {
                return Err(ParseCompactError::new(
                    offset,
                    "every state must list the same number of transitions",
                ));
            }
            for (a, triple) in (0..symbols).zip(group.as_chunks::<3>().0) {
                let at = offset + 3 * usize::from(a);
                if triple == b"---" {
                    continue;
                }
                let write = triple[0].wrapping_sub(b'0');
                if write >= symbols {
                    return Err(ParseCompactError::new(
                        at,
                        "expected a symbol digit or `---`",
                    ));
                }
                let dir = match triple[1] {
                    b'L' => Dir::L,
                    b'R' => Dir::R,
                    _ => return Err(ParseCompactError::new(at + 1, "expected `L` or `R`")),
                };
                let next = match triple[2] {
                    c @ b'A'..=b'Z' if c - b'A' < states => Next::State(c - b'A'),
                    b'A'..=b'Z' => Next::Halt,
                    _ => return Err(ParseCompactError::new(at + 2, "expected a state letter")),
                };
                tm.set(q, a, Some(CompactAction { write, dir, next }));
            }
        }
        Ok(tm)
    }
}

impl fmt::Display for CompactTm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for q in 0..self.states {
            if q > 0 {
                f.write_str("_")?;
            }
            for a in 0..self.symbols {
                match self.get(q, a) {
                    None => f.write_str("---")?,
                    Some(action) => {
                        let next = match action.next {
                            Next::State(n) => letter(n),
                            Next::Halt => 'Z',
                        };
                        write!(f, "{}{}{next}", digit(action.write), action.dir.as_char())?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl fmt::Debug for CompactTm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CompactTm({self})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_the_standard_format() {
        for text in [
            "1RB1LB_1LA1RZ",
            "1RB1RZ_1LB0RC_1LC1LA",
            "1RB1LB_1LA0LC_1RZ1LD_1RD0RA",
            "1RB1RA_0LC1LE_1LD1LC_1LA0LB_1LF1RE_---0RA",
            "1RB2LB1RZ_2LA2RB1LB",
            "------_------",
            "0RA---",
        ] {
            let tm: CompactTm = text.parse().unwrap();
            assert_eq!(tm.to_string(), text);
        }
    }

    #[test]
    fn halting_letters_beyond_the_states_mean_z() {
        let tm: CompactTm = "1RB1LB_1LA1RH".parse().unwrap();
        assert_eq!(tm.to_string(), "1RB1LB_1LA1RZ");
    }

    #[test]
    fn rejects_malformed_strings() {
        for (text, at) in [
            ("1RB1LB_1LA1R", 7),
            ("1RB1L!_1LA1RZ", 5),
            ("1RB1QB_1LA1RZ", 4),
            ("2RB1LB_1LA1RZ", 0),
            ("1RB1L", 5),
            ("", 0),
            ("1RB_1LA_1LA_1LA_1LA_1LA_1LA", 0),
        ] {
            let err = text.parse::<CompactTm>().unwrap_err();
            assert_eq!(err.at, at, "{text}: {err}");
        }
    }

    #[test]
    fn encoding_is_canonical() {
        let mut a: CompactTm = "1RB1LB_1LA---".parse().unwrap();
        let b: CompactTm = "1RB1LB_1LA---".parse().unwrap();
        a.set(
            1,
            1,
            Some(CompactAction {
                write: 1,
                dir: Dir::R,
                next: Next::Halt,
            }),
        );
        assert_ne!(a, b);
        a.set(1, 1, None);
        assert_eq!(a, b);
        assert_eq!(a.undefined(), 1);
    }

    #[test]
    fn converts_to_and_from_machines() {
        for text in [
            "1RB1RZ_1LB0RC_1LC1LA",
            "1RB1LB_1LA0LC_1RZ1LD_1RD0RA",
            "1RB2LB1RZ_2LA2RB1LB",
            "1RB1RA_0LC1LE_1LD1LC_1LA0LB_1LF1RE_---0RA",
            "0RA---",
        ] {
            let tm: CompactTm = text.parse().unwrap();
            let machine = tm.to_machine();
            assert_eq!(machine.kind(), Kind::Beaver);
            assert_eq!(machine.name(), text);
            assert_eq!(machine.state(machine.start()).name, "A");
            assert_eq!(machine.alphabet().blank(), '0');
            assert_eq!(CompactTm::try_from(&machine).unwrap(), tm, "{text}");
        }
    }

    #[test]
    fn machines_run_like_the_compact_form() {
        use turing_core::{Outcome, Simulator};
        let bb4: CompactTm = "1RB1LB_1LA0LC_1RZ1LD_1RD0RA".parse().unwrap();
        let machine = bb4.to_machine();
        let mut sim = Simulator::new(&machine, &[]);
        assert!(matches!(sim.run(1_000), Outcome::Halted { steps: 107, .. }));
    }

    #[test]
    fn states_without_transitions_become_the_halting_state() {
        let machine = Machine::builder("bb1")
            .blank('0')
            .transition("A", '0', "H", '1', Dir::R)
            .build()
            .unwrap();
        assert_eq!(CompactTm::try_from(&machine).unwrap().to_string(), "1RZ---");
        let stay = Machine::builder("stay")
            .blank('0')
            .transition("A", '0', "A", '1', Dir::N)
            .build()
            .unwrap();
        assert!(CompactTm::try_from(&stay).is_err());
    }

    #[test]
    fn next_state_bound_follows_the_coq_pointer() {
        let bound = |s: &str| s.parse::<CompactTm>().unwrap().next_state_bound();
        assert_eq!(bound("------_------_------_------"), 1);
        assert_eq!(bound("0RA---_------_------_------"), 1);
        assert_eq!(bound("1RB---_------_------_------"), 2);
        assert_eq!(bound("1RB---_1LC---_------_------"), 3);
        assert_eq!(bound("1RB---_1LA---"), 1);
        assert_eq!(bound("---"), 0);
    }
}
