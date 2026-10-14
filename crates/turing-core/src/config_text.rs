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
pub fn configuration(
    machine: &Machine,
    state: State,
    tape: &impl TapeView,
    max_cells: usize,
) -> String {
    let _ = (machine, state, tape, max_cells);
    todo!("A1")
}
