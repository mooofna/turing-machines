//! The bbchallenge standard text format, for example `1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA`.
//!
//! States are the letters `A`, `B`, ... in order and are separated by `_`; `A` is the start
//! state. Each state lists one transition per read symbol `0`, `1`, ...: the written digit, `L`
//! or `R`, and the next state. `---` marks an undefined transition. A next-state letter beyond
//! the last state (usually `Z`) is the halting state: the transition is executed and counted,
//! then the machine halts.
//!
//! # Reading
//!
//! [`parse_standard`] and [`spec_from_standard`] return a [`Kind::Beaver`] machine over the
//! digits `0` to `k - 1`, where k is the number of transitions per state, with blank `0`. All
//! next-state letters beyond the last state map to one final state `Z` without transitions,
//! which is added only if some transition targets it.
//!
//! # Writing
//!
//! [`to_standard`] writes a machine back. The format can only express single-tape machines
//! that move `L` or `R`, have at most 10 symbols, written as the digits `0` to `k - 1` with
//! blank `0`, and at most 26 states. State names are not kept: the start state becomes `A` and
//! the others follow in index order. A state without transitions that some transition enters
//! (other than the start state) is a halting state and is written as `Z`; a machine with two
//! such states cannot be written, because the format has only one way to halt.
//!
//! ```
//! use turing_core::standard::{parse_standard, to_standard};
//! let bb2 = parse_standard("1RB1LB_1LA1RZ").unwrap();
//! assert_eq!(bb2.num_states(), 3); // A, B and the halting state Z
//! assert_eq!(to_standard(&bb2).unwrap(), "1RB1LB_1LA1RZ");
//! ```
//!
//! [`Kind::Beaver`]: crate::Kind::Beaver

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::machine::{Kind, Machine};
use crate::spec::{Declared, MachineSpec, StateSpec, TransitionSpec};
use crate::symbol::{Dir, State, Symbol};

/// The most states the format can name: the letters `A` to `Z`.
const MAX_LETTERS: usize = 26;

/// The most symbols the format can write: the digits `0` to `9`.
const MAX_DIGITS: usize = 10;

/// The name of the halting state.
const HALT: &str = "Z";

/// Why a text is not a valid machine in the standard format, or a machine cannot be written in
/// it.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum StandardError {
    /// A syntax error at byte `at`.
    #[error("invalid standard format at byte {at}: {message}")]
    Syntax {
        /// Byte offset.
        at: usize,
        /// What was expected.
        message: String,
    },
    /// The states do not all have the same number of transitions.
    #[error("every state must list the same number of transitions")]
    Ragged,
    /// The machine has more than one tape, a move `N`, more than 26 states or more than 10
    /// symbols, or more than one halting state.
    #[error("this machine cannot be written in the standard format: {reason}")]
    Unsupported {
        /// Why not.
        reason: String,
    },
}

/// One parsed transition: written digit, move, next-state letter index.
type Entry = Option<(u8, Dir, usize)>;

/// Parses a machine in the standard format. The result is a [`Kind::Beaver`] machine over
/// `{0, 1, ...}` with blank `0`, states `A`, `B`, ... and, if any transition targets it, a final
/// halting state `Z` without transitions.
///
/// Surrounding whitespace is ignored.
///
/// [`Kind::Beaver`]: crate::Kind::Beaver
///
/// ```
/// use turing_core::{ChainSim, Outcome, standard::parse_standard};
/// let bb3 = parse_standard("1RB1RZ_1LB0RC_1LC1LA").unwrap();
/// let mut sim = ChainSim::new(&bb3, &[]).unwrap();
/// assert!(matches!(sim.run(1_000), Outcome::Halted { steps: 21, .. }));
/// ```
///
/// # Errors
///
/// See [`StandardError`].
pub fn parse_standard(text: &str) -> Result<Machine, StandardError> {
    spec_from_standard(text)?
        .compile()
        .map_err(|diagnostics| StandardError::Unsupported {
            reason: diagnostics
                .into_iter()
                .find(crate::Diagnostic::is_error)
                .map_or_else(|| "the machine is invalid".into(), |d| d.message),
        })
}

/// Like [`parse_standard`], but returns the editable spec.
///
/// The spec declares Q (so the states keep their letter order), Γ and the blank `0`.
///
/// ```
/// use turing_core::dsl::{self, FormatStyle};
/// use turing_core::standard::spec_from_standard;
/// let spec = spec_from_standard("1RZ---").unwrap();
/// let text = dsl::format(&spec, FormatStyle::Unicode);
/// assert!(text.contains("δ(A, 0) = (Z, 1, R)"));
/// ```
///
/// # Errors
///
/// See [`StandardError`].
pub fn spec_from_standard(text: &str) -> Result<MachineSpec, StandardError> {
    let offset = text.len() - text.trim_start().len();
    let text = text.trim();
    let rows = parse_rows(text, offset)?;
    let symbols = rows.first().map_or(0, Vec::len);
    let letters = rows.len();
    let letter = |i: usize| char::from(b'A' + u8::try_from(i).unwrap_or(25)).to_string();
    let digit = |d: usize| char::from(b'0' + u8::try_from(d).unwrap_or(9));

    let mut transitions = Vec::new();
    let mut halts = false;
    for (q, row) in rows.iter().enumerate() {
        for (read, entry) in row.iter().enumerate() {
            let Some((write, dir, next)) = *entry else {
                continue;
            };
            let to = if next < letters {
                letter(next)
            } else {
                halts = true;
                HALT.to_string()
            };
            transitions.push(TransitionSpec {
                from: letter(q),
                read: alloc::vec![digit(read)],
                to,
                write: alloc::vec![char::from(b'0' + write)],
                dir: alloc::vec![dir],
                ..TransitionSpec::default()
            });
        }
    }
    let mut states: Vec<StateSpec> = (0..letters)
        .map(|q| StateSpec {
            name: letter(q),
            is_final: false,
        })
        .collect();
    if halts {
        states.push(StateSpec {
            name: HALT.into(),
            is_final: true,
        });
    }
    let tape_alphabet: Vec<char> = (0..symbols).map(digit).collect();
    let mut spec = MachineSpec {
        kind: Kind::Beaver,
        tapes: 1,
        blank: '0',
        input_alphabet: tape_alphabet[1..].to_vec(),
        tape_alphabet,
        start: letter(0),
        states,
        transitions,
        declared: Declared {
            states: true,
            tape_alphabet: true,
            blank: true,
            ..Declared::default()
        },
        ..MachineSpec::new()
    };
    spec.infer();
    Ok(spec)
}

/// Splits `text` into rows of entries. `offset` is the position of `text` in the original.
fn parse_rows(text: &str, offset: usize) -> Result<Vec<Vec<Entry>>, StandardError> {
    let syntax = |at: usize, message: &str| StandardError::Syntax {
        at: offset + at,
        message: message.into(),
    };
    if text.is_empty() {
        return Err(syntax(0, "expected a transition such as `1RB`"));
    }
    let mut rows: Vec<Vec<Entry>> = Vec::new();
    let mut start = 0;
    for segment in text.split('_') {
        let bytes = segment.as_bytes();
        if bytes.is_empty() {
            return Err(syntax(start, "expected a transition such as `1RB`"));
        }
        if let Some(i) = segment.find(|c: char| !c.is_ascii_alphanumeric() && c != '-') {
            return Err(syntax(
                start + i,
                "expected a digit, `L`, `R`, a state letter or `-`",
            ));
        }
        if bytes.len() % 3 != 0 {
            return Err(syntax(
                start + bytes.len(),
                "every transition has three characters",
            ));
        }
        let row = bytes
            .chunks(3)
            .enumerate()
            .map(|(i, chunk)| parse_entry(chunk).map_err(|(j, m)| syntax(start + 3 * i + j, m)))
            .collect::<Result<Vec<_>, _>>()?;
        if rows.first().is_some_and(|first| first.len() != row.len()) {
            return Err(StandardError::Ragged);
        }
        rows.push(row);
        start += segment.len() + 1;
    }
    let symbols = rows[0].len();
    if rows.len() > MAX_LETTERS {
        return Err(StandardError::Unsupported {
            reason: format!(
                "it names {} states, but the letters A to Z name 26",
                rows.len()
            ),
        });
    }
    if symbols > MAX_DIGITS {
        return Err(StandardError::Unsupported {
            reason: format!("it has {symbols} symbols, but the digits 0 to 9 name 10"),
        });
    }
    for (q, row) in rows.iter().enumerate() {
        for (i, &(write, _, _)) in row
            .iter()
            .enumerate()
            .filter_map(|(i, e)| Some((i, e.as_ref()?)))
        {
            if usize::from(write) >= symbols {
                let at = offset + q * (3 * symbols + 1) + 3 * i;
                return Err(StandardError::Syntax {
                    at,
                    message: format!(
                        "the symbol {write} does not exist: the symbols are 0 to {}",
                        symbols - 1
                    ),
                });
            }
        }
    }
    Ok(rows)
}

/// One three-byte transition; on error, the offset within it and a message.
fn parse_entry(chunk: &[u8]) -> Result<Entry, (usize, &'static str)> {
    match *chunk {
        [b'-', b'-', b'-'] => Ok(None),
        [write, dir, next] => {
            if !write.is_ascii_digit() {
                return Err((0, "expected the digit to write, or `---`"));
            }
            let dir = match dir {
                b'L' => Dir::L,
                b'R' => Dir::R,
                _ => return Err((1, "expected the move `L` or `R`")),
            };
            if !next.is_ascii_uppercase() {
                return Err((2, "expected a state letter `A` to `Z`"));
            }
            Ok(Some((write - b'0', dir, usize::from(next - b'A'))))
        }
        _ => Err((0, "every transition has three characters")),
    }
}

/// Writes `machine` in the standard format. See the [module documentation](self) for what the
/// format can express.
///
/// ```
/// use turing_core::{Dir, Machine, standard::to_standard};
/// let m = Machine::builder("two steps")
///     .blank('0')
///     .transition("start", '0', "next", '1', Dir::R)
///     .transition("next", '0', "done", '1', Dir::L)
///     .build()
///     .unwrap();
/// assert_eq!(to_standard(&m).unwrap(), "1RB---_1LZ---");
/// ```
///
/// # Errors
///
/// Fails for machines the format cannot express, see [`StandardError::Unsupported`].
pub fn to_standard(machine: &Machine) -> Result<String, StandardError> {
    let unsupported = |reason: String| StandardError::Unsupported { reason };
    if machine.tapes() != 1 {
        return Err(unsupported(format!(
            "it has {} tapes, but the format describes single-tape machines",
            machine.tapes()
        )));
    }
    let alphabet = machine.alphabet();
    let k = alphabet.len();
    if k > MAX_DIGITS {
        return Err(unsupported(format!(
            "it has {k} symbols, but the digits 0 to 9 name 10"
        )));
    }
    // The symbol written as digit d, for d = 0, 1, ..., k - 1.
    let columns: Vec<Symbol> = (0..k)
        .map(|d| {
            let glyph = char::from(b'0' + u8::try_from(d).unwrap_or(0));
            alphabet.symbol(glyph)
        })
        .collect::<Option<_>>()
        .filter(|_| alphabet.blank() == '0')
        .ok_or_else(|| {
            unsupported(format!(
                "its symbols must be the digits 0 to {} with blank 0, but they are {}",
                k - 1,
                alphabet
                    .glyphs()
                    .iter()
                    .map(char::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })?;

    let n = machine.num_states();
    let mut has_transitions = alloc::vec![false; n];
    let mut is_target = alloc::vec![false; n];
    for t in machine.transitions() {
        has_transitions[t.from.index()] = true;
        is_target[t.action.next.index()] = true;
        if t.action.dir[0] == Dir::N {
            return Err(unsupported(format!(
                "the transition δ({}, {}) moves N, but the format only has L and R",
                machine.state(t.from).name,
                alphabet.glyph(t.read[0]),
            )));
        }
    }
    let halting: Vec<usize> = (0..n)
        .filter(|&q| !has_transitions[q] && is_target[q] && q != machine.start().index())
        .collect();
    if let [first, second, ..] = halting[..] {
        return Err(unsupported(format!(
            "the states `{}` and `{}` both halt, but the format has one halting state",
            machine.states()[first].name,
            machine.states()[second].name,
        )));
    }
    let halt = halting.first().copied();
    // The start state first, then the others in index order; the halting state is `Z`.
    let order: Vec<usize> = core::iter::once(machine.start().index())
        .chain((0..n).filter(|&q| q != machine.start().index() && Some(q) != halt))
        .collect();
    let limit = if halt.is_some() {
        MAX_LETTERS - 1
    } else {
        MAX_LETTERS
    };
    if order.len() > limit {
        return Err(unsupported(format!(
            "it has {} states besides the halting state, but the letters name at most {limit}",
            order.len()
        )));
    }
    let mut letters = alloc::vec!['Z'; n];
    for (i, &q) in order.iter().enumerate() {
        letters[q] = char::from(b'A' + u8::try_from(i).unwrap_or(25));
    }

    let rows: Vec<String> = order
        .iter()
        .map(|&q| {
            let state = State::new(u16::try_from(q).unwrap_or(u16::MAX));
            columns
                .iter()
                .map(|&read| match machine.action1(state, read) {
                    None => "---".to_string(),
                    Some(action) => {
                        let write = alphabet.glyph(action.write[0]);
                        let dir = action.dir[0].as_char();
                        format!("{write}{dir}{}", letters[action.next.index()])
                    }
                })
                .collect()
        })
        .collect();
    Ok(rows.join("_"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries() {
        assert_eq!(parse_entry(b"---"), Ok(None));
        assert_eq!(parse_entry(b"1RB"), Ok(Some((1, Dir::R, 1))));
        assert_eq!(parse_entry(b"0LZ"), Ok(Some((0, Dir::L, 25))));
        assert!(parse_entry(b"xRB").is_err());
        assert!(parse_entry(b"1NB").is_err());
        assert!(parse_entry(b"1Rb").is_err());
        assert!(parse_entry(b"1R-").is_err());
    }
}
