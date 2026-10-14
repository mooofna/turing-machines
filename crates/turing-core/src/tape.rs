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
        (0..len)
            .map(|i| self.get(from.saturating_add(signed(i))))
            .collect()
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

/// Blank cells allocated on each side of the input when a tape is created.
const INITIAL_PADDING: usize = 16;

/// Converts a cell index or length to a signed offset.
#[expect(
    clippy::cast_possible_wrap,
    reason = "a Vec holds at most isize::MAX elements, so every index fits in an i64"
)]
#[inline]
const fn signed(index: usize) -> i64 {
    index as i64
}

impl Tape {
    /// An empty tape with the head on position 0.
    pub fn blank() -> Self {
        Self::with_input(&[])
    }

    /// A tape holding `word` from position 0, head on position 0.
    pub fn with_input(word: &[Symbol]) -> Self {
        let mut cells = Vec::with_capacity(word.len() + 2 * INITIAL_PADDING);
        cells.resize(INITIAL_PADDING, Symbol::BLANK);
        cells.extend_from_slice(word);
        cells.resize(word.len() + 2 * INITIAL_PADDING, Symbol::BLANK);
        Self {
            cells,
            origin: INITIAL_PADDING,
            head: INITIAL_PADDING,
            touched: (0, 0),
        }
    }

    /// The symbol under the head.
    #[inline]
    pub fn read(&self) -> Symbol {
        self.cells[self.head]
    }

    /// Writes `symbol` under the head.
    #[inline]
    pub fn write(&mut self, symbol: Symbol) {
        self.cells[self.head] = symbol;
    }

    /// Moves the head.
    #[inline]
    pub fn shift(&mut self, dir: Dir) {
        match dir {
            Dir::L => {
                if self.head == 0 {
                    self.grow_left();
                }
                self.head -= 1;
                self.touched.0 = self.touched.0.min(self.head());
            }
            Dir::R => {
                self.head += 1;
                if self.head == self.cells.len() {
                    self.grow_right();
                }
                self.touched.1 = self.touched.1.max(self.head());
            }
            Dir::N => {}
        }
    }

    /// The range of positions the head has visited.
    pub fn touched(&self) -> RangeInclusive<i64> {
        self.touched.0..=self.touched.1
    }

    /// The positions currently held in memory. Every cell outside is blank.
    pub fn allocated(&self) -> RangeInclusive<i64> {
        let first = self.position(0);
        first..=first + signed(self.cells.len()) - 1
    }

    /// The nearest non-blank cell at `from` or beyond in direction `dir`, scanning only the
    /// allocated cells. `Dir::N` looks at `from` alone.
    pub fn find_non_blank(&self, from: i64, dir: Dir) -> Option<i64> {
        let len = signed(self.cells.len());
        let start = signed(self.origin).checked_add(from)?;
        let found = match dir {
            Dir::N => self.index(from).filter(|&i| !self.cells[i].is_blank()),
            Dir::R => {
                let start = usize::try_from(start.max(0)).ok()?;
                self.cells
                    .get(start..)?
                    .iter()
                    .position(|s| !s.is_blank())
                    .map(|i| start + i)
            }
            Dir::L => {
                let end = usize::try_from(start.min(len - 1).checked_add(1)?).ok()?;
                self.cells[..end].iter().rposition(|s| !s.is_blank())
            }
        };
        found.map(|i| self.position(i))
    }

    /// The position of the cell at `index`.
    #[inline]
    fn position(&self, index: usize) -> i64 {
        signed(index) - signed(self.origin)
    }

    /// The index of the cell at `pos`, if it is allocated.
    fn index(&self, pos: i64) -> Option<usize> {
        let index = usize::try_from(signed(self.origin).checked_add(pos)?).ok()?;
        (index < self.cells.len()).then_some(index)
    }

    /// Doubles the allocation by adding blank cells on the left.
    #[cold]
    #[inline(never)]
    fn grow_left(&mut self) {
        let extra = self.cells.len().max(INITIAL_PADDING);
        let mut cells = Vec::with_capacity(2 * (extra + self.cells.len()));
        cells.resize(extra, Symbol::BLANK);
        cells.extend_from_slice(&self.cells);
        self.cells = cells;
        self.origin += extra;
        self.head += extra;
    }

    /// Doubles the allocation by adding blank cells on the right.
    #[cold]
    #[inline(never)]
    fn grow_right(&mut self) {
        let len = self.cells.len() + self.cells.len().max(INITIAL_PADDING);
        self.cells.resize(len, Symbol::BLANK);
    }
}

impl TapeView for Tape {
    #[inline]
    fn head(&self) -> i64 {
        self.position(self.head)
    }

    fn get(&self, pos: i64) -> Symbol {
        self.index(pos).map_or(Symbol::BLANK, |i| self.cells[i])
    }

    fn non_blank(&self) -> Option<RangeInclusive<i64>> {
        let first = self.cells.iter().position(|s| !s.is_blank())?;
        let last = self.cells.iter().rposition(|s| !s.is_blank())?;
        Some(self.position(first)..=self.position(last))
    }

    fn count_non_blank(&self) -> u64 {
        self.cells.iter().filter(|s| !s.is_blank()).count() as u64
    }

    fn window(&self, from: i64, len: usize) -> Vec<Symbol> {
        let mut out = alloc::vec![Symbol::BLANK; len];
        let allocated = self.allocated();
        let lo = from.max(*allocated.start());
        let hi = from
            .saturating_add(signed(len))
            .saturating_sub(1)
            .min(*allocated.end());
        if let (Some(src), true) = (self.index(lo), lo <= hi) {
            let n = usize::try_from(hi - lo + 1).unwrap_or(0);
            let dst = usize::try_from(lo - from).unwrap_or(0);
            out[dst..dst + n].copy_from_slice(&self.cells[src..src + n]);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    fn s(i: u8) -> Symbol {
        Symbol::new(i)
    }

    #[test]
    fn a_blank_tape_is_empty() {
        let tape = Tape::blank();
        assert_eq!(tape.head(), 0);
        assert_eq!(tape.read(), Symbol::BLANK);
        assert_eq!(tape.non_blank(), None);
        assert_eq!(tape.count_non_blank(), 0);
        assert_eq!(tape.touched(), 0..=0);
        assert_eq!(tape.get(i64::MIN), Symbol::BLANK);
        assert_eq!(tape.get(i64::MAX), Symbol::BLANK);
    }

    #[test]
    fn input_starts_at_position_zero() {
        let tape = Tape::with_input(&[s(1), s(0), s(2)]);
        assert_eq!(tape.read(), s(1));
        assert_eq!(tape.get(1), Symbol::BLANK);
        assert_eq!(tape.get(2), s(2));
        assert_eq!(tape.get(-1), Symbol::BLANK);
        assert_eq!(tape.non_blank(), Some(0..=2));
        assert_eq!(tape.count_non_blank(), 2);
        assert_eq!(tape.touched(), 0..=0);
    }

    #[test]
    fn the_tape_grows_on_both_sides() {
        let mut tape = Tape::blank();
        for i in 0u16..1000 {
            tape.write(s(1 + u8::try_from(i % 3).unwrap()));
            tape.shift(Dir::L);
        }
        assert_eq!(tape.head(), -1000);
        assert_eq!(tape.touched(), -1000..=0);
        for _ in 0..3000 {
            tape.shift(Dir::R);
        }
        tape.write(s(2));
        assert_eq!(tape.head(), 2000);
        assert_eq!(tape.touched(), -1000..=2000);
        assert_eq!(tape.non_blank(), Some(-999..=2000));
        assert_eq!(tape.count_non_blank(), 1001);
        for i in 0u16..1000 {
            assert_eq!(tape.get(-i64::from(i)), s(1 + u8::try_from(i % 3).unwrap()));
        }
        assert!(tape.allocated().contains(&-1000) && tape.allocated().contains(&2000));
        assert!(tape.cells.len() < 2 * 3001 + 2 * INITIAL_PADDING);
    }

    #[test]
    fn staying_does_not_move_the_head() {
        let mut tape = Tape::with_input(&[s(1)]);
        tape.shift(Dir::N);
        assert_eq!(tape.head(), 0);
        assert_eq!(tape.touched(), 0..=0);
    }

    #[test]
    fn window_matches_get_everywhere() {
        let mut tape = Tape::with_input(&[s(1), s(2), s(3)]);
        for _ in 0..40 {
            tape.shift(Dir::L);
        }
        tape.write(s(4));
        let allocated = tape.allocated();
        let froms = [
            i64::MIN,
            *allocated.start() - 5,
            -41,
            -40,
            -3,
            0,
            2,
            *allocated.end() - 1,
            *allocated.end() + 3,
            i64::MAX - 2,
        ];
        for from in froms {
            for len in [0, 1, 3, 10, 200] {
                let expected: Vec<Symbol> =
                    (0..len).map(|i| tape.get(from.saturating_add(i))).collect();
                let len = usize::try_from(len).unwrap();
                assert_eq!(tape.window(from, len), expected, "from {from}, len {len}");
            }
        }
        assert_eq!(tape.window(-40, 4), vec![s(4), s(0), s(0), s(0)]);
    }

    #[test]
    fn find_non_blank_scans_in_the_given_direction() {
        let mut tape = Tape::with_input(&[s(1), s(0), s(0), s(2)]);
        for _ in 0..5 {
            tape.shift(Dir::L);
        }
        tape.write(s(3));
        assert_eq!(tape.find_non_blank(1, Dir::R), Some(3));
        assert_eq!(tape.find_non_blank(0, Dir::R), Some(0));
        assert_eq!(tape.find_non_blank(4, Dir::R), None);
        assert_eq!(tape.find_non_blank(1000, Dir::R), None);
        assert_eq!(tape.find_non_blank(-1000, Dir::R), Some(-5));
        assert_eq!(tape.find_non_blank(2, Dir::L), Some(0));
        assert_eq!(tape.find_non_blank(-1, Dir::L), Some(-5));
        assert_eq!(tape.find_non_blank(-6, Dir::L), None);
        assert_eq!(tape.find_non_blank(-1000, Dir::L), None);
        assert_eq!(tape.find_non_blank(1000, Dir::L), Some(3));
        assert_eq!(tape.find_non_blank(3, Dir::N), Some(3));
        assert_eq!(tape.find_non_blank(2, Dir::N), None);
    }
}
