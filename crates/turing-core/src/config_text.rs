//! Configurations in course notation, `u q v`.

use alloc::string::String;

use crate::machine::Machine;
use crate::symbol::State;
use crate::tape::TapeView;

/// Writes the configuration `u q v` of a single tape: `u` is the tape left of the head, `q`
/// the state name and `v` the tape from the head on, both trimmed to non-blank content but
/// always including the cell under the head.
///
/// If the visible part is longer than `max_cells`, it is cut around the head and the cut
/// sides are marked with `…`.
///
/// ```
/// use turing_core::{Dir, Machine, Simulator, config_text::configuration};
/// let m = Machine::builder("walk").transition("q0", 'x', "q1", 'X', Dir::R).build().unwrap();
/// let input = m.encode_input("xx").unwrap();
/// let mut sim = Simulator::new(&m, &input);
/// assert_eq!(configuration(&m, sim.state(), sim.tape(0), 40), "q0 xx");
/// sim.step();
/// assert_eq!(configuration(&m, sim.state(), sim.tape(0), 40), "X q1 x");
/// ```
///
/// Blanks between the content and the head are written with the blank glyph, and at least
/// the head cell is shown:
///
/// ```
/// use turing_core::{Dir, Machine, Simulator, config_text::configuration};
/// let m = Machine::builder("away").transition("q", 'x', "q", 'x', Dir::R).build().unwrap();
/// let input = m.encode_input("xxxxxx").unwrap();
/// let mut sim = Simulator::new(&m, &input);
/// sim.run(100);
/// assert_eq!(configuration(&m, sim.state(), sim.tape(0), 40), "xxxxxx q ␣");
/// assert_eq!(configuration(&m, sim.state(), sim.tape(0), 3), "…xx q ␣");
/// ```
pub fn configuration(
    machine: &Machine,
    state: State,
    tape: &impl TapeView,
    max_cells: usize,
) -> String {
    let head = tape.head();
    let (lo, hi) = tape.non_blank().map_or((head, head), |range| {
        ((*range.start()).min(head), (*range.end()).max(head))
    });

    // Cells to show besides the head cell, shared out evenly and given to the other side
    // where one side has fewer.
    let budget = u64::try_from(max_cells.max(1) - 1).unwrap_or(u64::MAX);
    let (have_left, have_right) = (head.abs_diff(lo), hi.abs_diff(head));
    let want_left = budget / 2 + (budget - budget / 2).saturating_sub(have_right);
    let left = have_left.min(want_left);
    let right = have_right.min(budget - left);
    let (first, last) = (head - signed(left), head + signed(right));

    let alphabet = machine.alphabet();
    let len = usize::try_from(left + right + 1).unwrap_or(usize::MAX);
    let cells = tape.window(first, len);
    let split = usize::try_from(left).unwrap_or(0);
    let name = machine
        .states()
        .get(state.index())
        .map_or("?", |s| s.name.as_str());

    let mut out = String::with_capacity(len + name.len() + 8);
    if first > lo {
        out.push('…');
    }
    out.extend(cells[..split].iter().map(|&s| alphabet.glyph(s)));
    if !out.is_empty() {
        out.push(' ');
    }
    out.push_str(name);
    out.push(' ');
    out.extend(cells[split..].iter().map(|&s| alphabet.glyph(s)));
    if last < hi {
        out.push('…');
    }
    out
}

/// Converts a distance that fits into the tape to a signed offset.
fn signed(n: u64) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;
    use crate::rle::RunTape;
    use crate::symbol::{Dir, Symbol};
    use crate::tape::Tape;

    fn machine() -> Machine {
        Machine::builder("m")
            .tape_alphabet(['a', 'b'])
            .state("q")
            .build()
            .unwrap()
    }

    /// The tape holding `word` from position 0 with the head on position `head`.
    fn tape(m: &Machine, word: &str, head: i64) -> Tape {
        let symbols: Vec<Symbol> = word
            .chars()
            .map(|c| m.alphabet().symbol(c).unwrap())
            .collect();
        let mut tape = Tape::with_input(&symbols);
        let dir = if head < 0 { Dir::L } else { Dir::R };
        for _ in 0..head.unsigned_abs() {
            tape.shift(dir);
        }
        tape
    }

    fn show(word: &str, head: i64, max_cells: usize) -> String {
        let m = machine();
        configuration(&m, State::new(0), &tape(&m, word, head), max_cells)
    }

    #[test]
    fn the_head_cell_is_always_shown() {
        assert_eq!(show("", 0, 10), "q ␣");
        assert_eq!(show("", 0, 0), "q ␣");
        assert_eq!(show("ab", 0, 10), "q ab");
        assert_eq!(show("ab", 1, 10), "a q b");
        assert_eq!(show("ab", 2, 10), "ab q ␣");
    }

    #[test]
    fn blanks_between_content_and_head_are_shown() {
        assert_eq!(show("ab", 4, 10), "ab␣␣ q ␣");
        assert_eq!(show("ab", -3, 10), "q ␣␣␣ab");
        assert_eq!(show("a␣␣b", 1, 10), "a q ␣␣b");
        assert_eq!(show("␣␣ab␣␣", 0, 10), "q ␣␣ab");
    }

    #[test]
    fn long_tapes_are_cut_around_the_head() {
        let word = "aaaaabbbbb";
        assert_eq!(show(word, 5, 10), "aaaaa q bbbbb");
        assert_eq!(show(word, 5, 5), "…aa q bbb…");
        assert_eq!(show(word, 5, 4), "…a q bbb…");
        assert_eq!(show(word, 0, 4), "q aaaa…");
        assert_eq!(show(word, 9, 4), "…bbb q b");
        assert_eq!(show(word, 8, 4), "…bb q bb");
        assert_eq!(show(word, 5, 1), "… q b…");
        assert_eq!(show(word, 12, 3), "…␣␣ q ␣");
    }

    #[test]
    fn run_tapes_give_the_same_text() {
        let m = machine();
        let (a, b) = (Symbol::new(1), Symbol::new(2));
        let runs = RunTape::with_input(&[a, a, b, Symbol::BLANK, b]);
        let dense = Tape::with_input(&[a, a, b, Symbol::BLANK, b]);
        for max_cells in [1, 2, 3, 5, 8] {
            assert_eq!(
                configuration(&m, State::new(0), &runs, max_cells),
                configuration(&m, State::new(0), &dense, max_cells)
            );
        }
        assert_eq!(configuration(&m, State::new(7), &runs, 9), "? aab␣b");
    }
}
