//! Reading the result of a halted run.

use alloc::string::String;

use crate::machine::{BitOrder, Decoder, Machine};
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
/// The value depends on the machine's [`Decoder`]:
///
/// * [`Decoder::Word`] gives no value.
/// * [`Decoder::Unary`] counts the occurrences of its symbol in the word.
/// * [`Decoder::Binary`] reads a word made only of the glyphs `0` and `1` in the given bit
///   order. Any other glyph, or a value above `u128::MAX`, gives no value. The empty word
///   stands for 0.
///
/// The word has one character per cell between the outermost non-blank cells, so it is as
/// long as that part of the tape.
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
    let alphabet = machine.alphabet();
    let word: String = tape
        .non_blank()
        .map(|range| {
            let len = usize::try_from(range.end().abs_diff(*range.start()))
                .map_or(usize::MAX, |d| d.saturating_add(1));
            tape.window(*range.start(), len)
                .into_iter()
                .map(|s| alphabet.glyph(s))
                .collect()
        })
        .unwrap_or_default();
    let value = match machine.decoder() {
        Decoder::Word => None,
        Decoder::Unary { symbol } => {
            u128::try_from(word.chars().filter(|&c| c == symbol).count()).ok()
        }
        Decoder::Binary { order } => binary(&word, order),
    };
    Output { word, value }
}

/// Reads `word` as a binary number, or `None` if it holds another glyph or overflows.
fn binary(word: &str, order: BitOrder) -> Option<u128> {
    let push_bit = |value: u128, glyph: char| {
        let bit = match glyph {
            '0' => 0,
            '1' => 1,
            _ => return None,
        };
        value.checked_mul(2)?.checked_add(bit)
    };
    match order {
        BitOrder::Msb => word.chars().try_fold(0, push_bit),
        BitOrder::Lsb => word.chars().rev().try_fold(0, push_bit),
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;
    use crate::symbol::{Dir, Symbol};
    use crate::tape::Tape;

    fn machine(blank: char, decoder: Decoder) -> Machine {
        Machine::builder("out")
            .blank(blank)
            .tape_alphabet(['0', '1', 'x', '#'])
            .decoder(decoder)
            .state("q")
            .build()
            .unwrap()
    }

    fn tape(m: &Machine, word: &str) -> Tape {
        let symbols: Vec<Symbol> = word
            .chars()
            .map(|c| m.alphabet().symbol(c).unwrap())
            .collect();
        let mut tape = Tape::with_input(&symbols);
        tape.shift(Dir::L);
        tape
    }

    fn value(blank: char, decoder: Decoder, word: &str) -> Option<u128> {
        let m = machine(blank, decoder);
        decode(&m, &tape(&m, word)).value
    }

    #[test]
    fn the_word_is_the_non_blank_extent() {
        let m = machine('␣', Decoder::Word);
        let out = decode(&m, &tape(&m, "␣␣x␣#x␣␣"));
        assert_eq!(out.word, "x␣#x");
        assert_eq!(out.value, None);
        assert_eq!(decode(&m, &Tape::blank()), Output::default());
    }

    #[test]
    fn unary_counts_the_tally_symbol() {
        let unary = Decoder::Unary { symbol: 'x' };
        assert_eq!(value('␣', unary, "#xx#x#"), Some(3));
        assert_eq!(value('␣', unary, ""), Some(0));
        assert_eq!(value('␣', unary, "##"), Some(0));
    }

    #[test]
    fn binary_reads_both_bit_orders() {
        let msb = Decoder::Binary {
            order: BitOrder::Msb,
        };
        let lsb = Decoder::Binary {
            order: BitOrder::Lsb,
        };
        assert_eq!(value('␣', msb, "1101"), Some(13));
        assert_eq!(value('␣', lsb, "1101"), Some(11));
        assert_eq!(value('␣', msb, "0011"), Some(3));
        assert_eq!(value('␣', msb, ""), Some(0));
        assert_eq!(value('␣', msb, "1x1"), None);
        assert_eq!(value('␣', msb, "1␣1"), None);
        // With blank `0` the trimmed word drops leading and trailing zeros.
        assert_eq!(value('␣', msb, "1"), Some(1));
        assert_eq!(value('0', msb, "00100"), Some(1));
        assert_eq!(value('0', msb, "1001"), Some(9));
    }

    #[test]
    fn binary_values_beyond_u128_are_dropped() {
        let msb = Decoder::Binary {
            order: BitOrder::Msb,
        };
        let lsb = Decoder::Binary {
            order: BitOrder::Lsb,
        };
        let max: String = "1".repeat(128);
        assert_eq!(value('␣', msb, &max), Some(u128::MAX));
        assert_eq!(value('␣', msb, &(max.clone() + "1")), None);
        let mut high = String::from("1");
        high.push_str(&"0".repeat(127));
        assert_eq!(value('␣', msb, &high), Some(1 << 127));
        assert_eq!(value('␣', lsb, &high), Some(1));
        assert_eq!(value('␣', lsb, &(high.clone() + "1")), None);
    }
}
