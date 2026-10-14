//! Reading the result of a halted run.

use alloc::string::String;

use crate::machine::{BitOrder, Decoder, Machine};
use crate::symbol::Symbol;
use crate::tape::TapeView;

/// The output of a transducer, read from the first tape.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Output {
    /// The non-blank part of the tape, from the leftmost to the rightmost non-blank cell, with
    /// inner blanks written as the blank glyph.
    pub word: String,
    /// The word read as a number, if the machine has a numeric decoder and the word fits.
    pub value: Option<u128>,
}

/// Reads the output word and its numeric value from `tape`.
///
/// ```
/// use turing_core::{Decoder, Dir, Machine, Simulator, decode};
/// let m = Machine::builder("unary")
///     .decoder(Decoder::Unary { symbol: 'x' })
///     .transition("q0", 'x', "q0", 'x', Dir::R)
///     .build()
///     .unwrap();
/// let input = m.encode_input("xxx").unwrap();
/// let mut sim = Simulator::new(&m, &input);
/// sim.run(10);
/// assert_eq!(decode(&m, sim.tape(0)).value, Some(3));
/// ```
pub fn decode(machine: &Machine, tape: &impl TapeView) -> Output {
    let _ = (machine, tape, Symbol::BLANK, BitOrder::Lsb, Decoder::Word);
    todo!("A1")
}
