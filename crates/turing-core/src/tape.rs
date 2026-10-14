//! The dense two-way infinite tape.

use alloc::vec::Vec;
use core::ops::RangeInclusive;

use crate::symbol::{Dir, Symbol};

/// Read access to a tape, shared by the dense [`Tape`] and the run-length
/// [`RunTape`](crate::RunTape).
pub trait TapeView {
    /// The head position; the input starts at position 0.
    fn head(&self) -> i64;

    /// The symbol at `pos`. Cells that were never written are blank.
    fn get(&self, pos: i64) -> Symbol;

    /// The smallest range that contains every non-blank cell, if there is one.
    fn non_blank(&self) -> Option<RangeInclusive<i64>>;

    /// The number of non-blank cells.
    fn count_non_blank(&self) -> u64;

    /// The symbols at `from`, `from + 1`, ..., `from + len - 1`.
    fn window(&self, from: i64, len: usize) -> Vec<Symbol> {
        (0..len).map(|i| self.get(from + i as i64)).collect()
    }
}

/// A two-way infinite tape stored densely, growing by doubling on either side.
///
/// Reading, writing and moving are amortised O(1).
///
/// ```
/// use turing_core::{Dir, Symbol, Tape, TapeView};
/// let mut tape = Tape::blank();
/// tape.write(Symbol::new(1));
/// tape.shift(Dir::L);
/// tape.shift(Dir::L);
/// tape.write(Symbol::new(1));
/// assert_eq!(tape.head(), -2);
/// assert_eq!(tape.non_blank(), Some(-2..=0));
/// assert_eq!(tape.count_non_blank(), 2);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tape {
    pub(crate) cells: Vec<Symbol>,
    /// The index of position 0 in `cells`.
    pub(crate) origin: usize,
    /// The index of the head in `cells`.
    pub(crate) head: usize,
    /// The smallest and largest head position so far.
    pub(crate) touched: (i64, i64),
}

impl Tape {
    /// An empty tape with the head on position 0.
    pub fn blank() -> Self {
        todo!("A1")
    }

    /// A tape holding `word` from position 0, head on position 0.
    pub fn with_input(word: &[Symbol]) -> Self {
        let _ = word;
        todo!("A1")
    }

    /// The symbol under the head.
    #[inline]
    pub fn read(&self) -> Symbol {
        todo!("A1")
    }

    /// Writes `symbol` under the head.
    #[inline]
    pub fn write(&mut self, symbol: Symbol) {
        let _ = symbol;
        todo!("A1")
    }

    /// Moves the head.
    #[inline]
    pub fn shift(&mut self, dir: Dir) {
        let _ = dir;
        todo!("A1")
    }

    /// The range of positions the head has visited.
    pub fn touched(&self) -> RangeInclusive<i64> {
        self.touched.0..=self.touched.1
    }
}

impl TapeView for Tape {
    fn head(&self) -> i64 {
        todo!("A1")
    }

    fn get(&self, pos: i64) -> Symbol {
        let _ = pos;
        todo!("A1")
    }

    fn non_blank(&self) -> Option<RangeInclusive<i64>> {
        todo!("A1")
    }

    fn count_non_blank(&self) -> u64 {
        todo!("A1")
    }
}
