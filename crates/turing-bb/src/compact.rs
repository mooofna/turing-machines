//! A compact, copyable representation of small two-way machines for enumeration.

use core::fmt;
use core::str::FromStr;

use turing_core::{Dir, Machine};

/// The largest number of states of a [`CompactTm`].
pub const MAX_STATES: usize = 6;
/// The largest number of symbols of a [`CompactTm`].
pub const MAX_SYMBOLS: usize = 8;

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

impl CompactTm {
    /// A machine with every transition undefined.
    ///
    /// # Panics
    ///
    /// Panics if `states` or `symbols` exceed the limits or are 0.
    pub fn new(states: u8, symbols: u8) -> Self {
        let _ = (states, symbols);
        todo!("A3")
    }

    /// The number of states.
    pub fn states(&self) -> u8 {
        self.states
    }

    /// The number of symbols.
    pub fn symbols(&self) -> u8 {
        self.symbols
    }

    /// δ(q, a), or `None` if undefined.
    #[inline]
    pub fn get(&self, q: u8, a: u8) -> Option<CompactAction> {
        let _ = (q, a);
        todo!("A3")
    }

    /// Sets or clears δ(q, a).
    pub fn set(&mut self, q: u8, a: u8, action: Option<CompactAction>) {
        let _ = (q, a, action);
        todo!("A3")
    }

    /// The number of undefined transitions.
    pub fn undefined(&self) -> usize {
        todo!("A3")
    }

    /// The equivalent [`Machine`] (kind beaver, blank `0`, states `A`, `B`, ..., halting state
    /// `Z` if used).
    pub fn to_machine(&self) -> Machine {
        todo!("A3")
    }
}

impl TryFrom<&Machine> for CompactTm {
    type Error = ParseCompactError;

    fn try_from(machine: &Machine) -> Result<Self, Self::Error> {
        let _ = machine;
        todo!("A3")
    }
}

impl FromStr for CompactTm {
    type Err = ParseCompactError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _ = s;
        todo!("A3")
    }
}

impl fmt::Display for CompactTm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = f;
        todo!("A3")
    }
}

impl fmt::Debug for CompactTm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CompactTm({self})")
    }
}
