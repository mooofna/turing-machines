//! The input word of a run: given literally, as numbers in unary or binary, or the machine's own.

use turing_core::{BitOrder, Decoder, Machine, Symbol};

use crate::error::{Error, Result};
use crate::text;

/// The largest unary input, in cells.
const MAX_UNARY_CELLS: u64 = 100_000_000;

/// Options that choose the input word.
#[derive(clap::Args, Debug, Clone, Default)]
pub(crate) struct InputArgs {
    /// The input word on the first tape. `_` stands for the blank unless the machine uses `_`
    /// itself. Without an input option the machine's own example input is used
    #[arg(short, long, value_name = "WORD")]
    pub(crate) input: Option<String>,

    /// Numbers in unary, in the course encoding: `--unary 3 4` is the input `#xxx#xxxx#`, with
    /// the machine's tally symbol. A single number is written without `#` if the machine does
    /// not use `#`. Implies `--format value`
    #[arg(long, value_name = "N", num_args = 1.., conflicts_with = "input")]
    pub(crate) unary: Option<Vec<u64>>,

    /// A number in binary, in the bit order of the machine's output (most significant bit
    /// first unless the machine says otherwise). Implies `--format value`
    #[arg(long, value_name = "N", conflicts_with_all = ["input", "unary"])]
    pub(crate) binary: Option<u128>,
}

impl InputArgs {
    /// Whether the input was given as numbers, so that a number is the natural result.
    pub(crate) fn is_numeric(&self) -> bool {
        self.unary.is_some() || self.binary.is_some()
    }
}

/// A prepared input word.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Input {
    /// The word as glyphs, for display.
    pub(crate) word: String,
    /// The word as symbols.
    pub(crate) symbols: Vec<Symbol>,
}

/// Builds the input for `machine` from `args`.
pub(crate) fn prepare(machine: &Machine, args: &InputArgs) -> Result<Input> {
    let word = if let Some(numbers) = &args.unary {
        unary(machine, numbers)?
    } else if let Some(n) = args.binary {
        binary(machine, n)
    } else if let Some(word) = &args.input {
        blank_underscores(machine, word)
    } else {
        machine.default_input().to_owned()
    };
    let symbols = machine.encode_input(&word).map_err(|e| {
        let sigma = text::glyph_set(machine.alphabet().input_glyphs());
        Error::usage(format!(
            "the input `{word}` cannot be used: {e} Σ = {sigma}"
        ))
    })?;
    Ok(Input { word, symbols })
}

/// Replaces `_` by the blank glyph if `_` is not a symbol of the machine.
fn blank_underscores(machine: &Machine, word: &str) -> String {
    let alphabet = machine.alphabet();
    if alphabet.symbol('_').is_some() {
        word.to_owned()
    } else {
        word.replace('_', &alphabet.blank().to_string())
    }
}

/// The tally symbol for unary numbers: the machine's unary output symbol, else `x`, else `1`,
/// else the first input symbol other than `#`.
fn tally(machine: &Machine) -> Option<char> {
    let alphabet = machine.alphabet();
    let is_input = |c: char| alphabet.symbol(c).is_some_and(|s| alphabet.is_input(s));
    if let Decoder::Unary { symbol } = machine.decoder()
        && is_input(symbol)
    {
        return Some(symbol);
    }
    ['x', '1']
        .into_iter()
        .find(|&c| is_input(c))
        .or_else(|| alphabet.input_glyphs().find(|&c| c != '#'))
}

fn unary(machine: &Machine, numbers: &[u64]) -> Result<String> {
    let total = numbers.iter().fold(0u64, |acc, &n| acc.saturating_add(n));
    if total > MAX_UNARY_CELLS {
        return Err(Error::usage(format!(
            "unary inputs are limited to {} cells",
            text::num(MAX_UNARY_CELLS)
        )));
    }
    let Some(x) = tally(machine) else {
        return Err(Error::usage(
            "the machine has no input symbol to write numbers in unary",
        ));
    };
    let alphabet = machine.alphabet();
    let has_hash = alphabet.symbol('#').is_some_and(|s| alphabet.is_input(s));
    let tallies = |n: u64| -> String {
        std::iter::repeat_n(x, usize::try_from(n).unwrap_or(usize::MAX)).collect()
    };
    match numbers {
        [n] if !has_hash => Ok(tallies(*n)),
        _ if has_hash => {
            let mut word = String::from("#");
            for &n in numbers {
                word.push_str(&tallies(n));
                word.push('#');
            }
            Ok(word)
        }
        _ => Err(Error::usage(
            "several unary numbers need `#` in the input alphabet to separate them",
        )),
    }
}

fn binary(machine: &Machine, n: u128) -> String {
    let msb_first = format!("{n:b}");
    match machine.decoder() {
        Decoder::Binary {
            order: BitOrder::Lsb,
        } => msb_first.chars().rev().collect(),
        _ => msb_first,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turing_core::Dir;

    fn mul() -> Machine {
        Machine::builder("mul")
            .decoder(Decoder::Unary { symbol: 'x' })
            .input_alphabet(['x', '#'])
            .transition("q0", '#', "q1", '#', Dir::R)
            .build()
            .unwrap()
    }

    fn double() -> Machine {
        Machine::builder("double")
            .input_alphabet(['x'])
            .transition("q0", 'x', "q0", 'x', Dir::R)
            .build()
            .unwrap()
    }

    #[test]
    fn unary_course_encoding() {
        assert_eq!(unary(&mul(), &[3, 4]).unwrap(), "#xxx#xxxx#");
        assert_eq!(unary(&mul(), &[0, 2]).unwrap(), "##xx#");
        assert_eq!(unary(&double(), &[3]).unwrap(), "xxx");
        assert!(unary(&double(), &[1, 2]).is_err());
    }
}
