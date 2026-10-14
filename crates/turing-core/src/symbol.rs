//! Tape symbols, states and head moves.

use core::fmt;

/// The largest number of tapes a [`Machine`](crate::Machine) can have.
pub const MAX_TAPES: usize = 3;

/// The largest tape alphabet, blank included.
pub const MAX_SYMBOLS: usize = 64;

/// The largest number of states of a [`Machine`](crate::Machine).
pub const MAX_STATES: usize = 4096;

/// A tape symbol, stored as its index in the machine's [`Alphabet`](crate::Alphabet).
///
/// Index 0 is always the blank symbol.
///
/// ```
/// use turing_core::Symbol;
/// assert_eq!(Symbol::BLANK.index(), 0);
/// assert_eq!(Symbol::new(3).index(), 3);
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Symbol(u8);

impl Symbol {
    /// The blank symbol, which fills every cell that was never written.
    pub const BLANK: Self = Self(0);

    /// The symbol with alphabet index `index`.
    pub const fn new(index: u8) -> Self {
        Self(index)
    }

    /// The alphabet index as a `usize`, for table lookups.
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// The alphabet index as stored.
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Whether this is the blank symbol.
    pub const fn is_blank(self) -> bool {
        self.0 == 0
    }
}

/// A control state, stored as its index in the machine's state list.
///
/// ```
/// use turing_core::State;
/// assert_eq!(State::new(2).index(), 2);
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct State(u16);

impl State {
    /// The state with index `index`.
    pub const fn new(index: u16) -> Self {
        Self(index)
    }

    /// The index as a `usize`, for table lookups.
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// The index as stored.
    pub const fn get(self) -> u16 {
        self.0
    }
}

/// The move of a head after writing: one cell left, one cell right, or no move.
///
/// The course uses only `L` and `R`; `N` is an extension that keeps the head in place.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Dir {
    /// Move one cell to the left.
    L,
    /// Move one cell to the right.
    R,
    /// Stay on the current cell.
    #[default]
    N,
}

impl Dir {
    /// The change of the head position: -1, +1 or 0.
    ///
    /// ```
    /// use turing_core::Dir;
    /// assert_eq!(Dir::L.delta() + Dir::R.delta() + Dir::N.delta(), 0);
    /// ```
    pub const fn delta(self) -> i64 {
        match self {
            Self::L => -1,
            Self::R => 1,
            Self::N => 0,
        }
    }

    /// The opposite move; `N` stays `N`.
    #[must_use]
    pub const fn reversed(self) -> Self {
        match self {
            Self::L => Self::R,
            Self::R => Self::L,
            Self::N => Self::N,
        }
    }

    /// The letter used in the description language and the standard format.
    pub const fn as_char(self) -> char {
        match self {
            Self::L => 'L',
            Self::R => 'R',
            Self::N => 'N',
        }
    }

    /// Parses `L`, `R` or `N`.
    ///
    /// ```
    /// use turing_core::Dir;
    /// assert_eq!(Dir::from_char('R'), Some(Dir::R));
    /// assert_eq!(Dir::from_char('x'), None);
    /// ```
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            'L' => Some(Self::L),
            'R' => Some(Self::R),
            'N' => Some(Self::N),
            _ => None,
        }
    }
}

impl fmt::Display for Dir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}
