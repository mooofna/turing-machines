//! The validated machine M = (Q, Σ, Γ, δ, q0, F) with a dense transition table.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::alphabet::Alphabet;
use crate::builder::MachineBuilder;
use crate::symbol::{Dir, MAX_TAPES, State, Symbol};

/// What one transition does on every tape: write a symbol, move the head, enter the next state.
///
/// Entries for tapes beyond [`Machine::tapes`] are unused and hold blank and `N`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Action {
    /// The state entered after the step.
    pub next: State,
    /// The symbol written on each tape.
    pub write: [Symbol; MAX_TAPES],
    /// The move of each head.
    pub dir: [Dir; MAX_TAPES],
}

impl Action {
    /// The action of a single-tape machine.
    ///
    /// ```
    /// use turing_core::{Action, Dir, State, Symbol};
    /// let a = Action::single(Symbol::new(1), Dir::R, State::new(2));
    /// assert_eq!(a.write[0], Symbol::new(1));
    /// assert_eq!(a.dir[0], Dir::R);
    /// ```
    pub const fn single(write: Symbol, dir: Dir, next: State) -> Self {
        let mut w = [Symbol::BLANK; MAX_TAPES];
        let mut d = [Dir::N; MAX_TAPES];
        w[0] = write;
        d[0] = dir;
        Self {
            next,
            write: w,
            dir: d,
        }
    }
}

/// One defined entry of δ.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Transition {
    /// The state in which the transition applies.
    pub from: State,
    /// The symbols under the heads; entries beyond the tape count are blank.
    pub read: [Symbol; MAX_TAPES],
    /// What the transition does.
    pub action: Action,
}

/// What a machine is used for. It decides how a halted run is read.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Kind {
    /// Accepts its input if it halts in a final state, and rejects it if it halts elsewhere.
    Acceptor,
    /// Computes a function: the output is the content of the first tape after halting.
    #[default]
    Transducer,
    /// Prints a language: every step into a final state outputs the current tape.
    Enumerator,
    /// Starts on the blank tape; only the number of steps and of non-blank cells matter.
    Beaver,
}

/// The order of bits for [`Decoder::Binary`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum BitOrder {
    /// The least significant bit comes first (leftmost).
    Lsb,
    /// The most significant bit comes first (leftmost).
    #[default]
    Msb,
}

/// How the output word of a transducer is read as a number.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", rename_all = "lowercase"))]
pub enum Decoder {
    /// The output is a word; no number is derived.
    #[default]
    Word,
    /// The number of occurrences of `symbol`.
    Unary {
        /// The tally symbol.
        symbol: char,
    },
    /// A binary number written with the glyphs `0` and `1`.
    Binary {
        /// Bit order.
        order: BitOrder,
    },
}

/// A control state with its name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StateInfo {
    /// The name used in descriptions and diagrams, for example `q0` or `A`.
    pub name: String,
    /// Whether the state belongs to F.
    pub is_final: bool,
}

/// An input word contains a glyph outside the input alphabet.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the input symbol {glyph:?} at position {position} is not in the input alphabet")]
pub struct InputError {
    /// The offending glyph.
    pub glyph: char,
    /// Its position in the word, counted in characters from 0.
    pub position: usize,
}

/// A deterministic Turing machine with one to [`MAX_TAPES`] tapes, validated and ready to run.
///
/// δ is stored as a dense table with one entry per state and tuple of read symbols, so a step
/// costs one index computation and one load.
///
/// ```
/// use turing_core::{Dir, Machine};
/// // Binary increment, least significant bit first.
/// let m = Machine::builder("increment")
///     .transition("q0", '1', "q0", '0', Dir::R)
///     .transition("q0", '0', "q1", '1', Dir::R)
///     .transition("q0", '␣', "q1", '1', Dir::R)
///     .final_state("q1")
///     .build()
///     .unwrap();
/// assert_eq!(m.num_states(), 2);
/// assert_eq!(m.state(m.start()).name, "q0");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Machine {
    pub(crate) name: String,
    pub(crate) kind: Kind,
    pub(crate) tapes: usize,
    pub(crate) alphabet: Alphabet,
    pub(crate) states: Vec<StateInfo>,
    pub(crate) start: State,
    /// `symbols ^ tapes`: the number of table entries per state.
    pub(crate) stride: usize,
    /// Index `state * stride + read_index(read)`.
    pub(crate) table: Box<[Option<Action>]>,
    /// For each table entry, the index of the transition it came from in the order it was
    /// added to the builder (which is the order of [`MachineSpec::transitions`]), or
    /// `u32::MAX` if undefined.
    ///
    /// [`MachineSpec::transitions`]: crate::MachineSpec::transitions
    pub(crate) origin: Box<[u32]>,
    pub(crate) decoder: Decoder,
    pub(crate) input: String,
}

impl Machine {
    /// Starts building a machine called `name`.
    pub fn builder(name: impl Into<String>) -> MachineBuilder {
        MachineBuilder::new(name)
    }

    /// The machine's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// What the machine is used for.
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// The number of tapes, from 1 to [`MAX_TAPES`].
    pub fn tapes(&self) -> usize {
        self.tapes
    }

    /// The tape alphabet Γ.
    pub fn alphabet(&self) -> &Alphabet {
        &self.alphabet
    }

    /// |Q|.
    pub fn num_states(&self) -> usize {
        self.states.len()
    }

    /// |Γ|, blank included.
    pub fn num_symbols(&self) -> usize {
        self.alphabet.len()
    }

    /// q0.
    pub fn start(&self) -> State {
        self.start
    }

    /// Name and finality of `q`.
    ///
    /// # Panics
    ///
    /// Panics if `q` is not a state of this machine.
    pub fn state(&self, q: State) -> &StateInfo {
        &self.states[q.index()]
    }

    /// All states in index order.
    pub fn states(&self) -> &[StateInfo] {
        &self.states
    }

    /// The state called `name`.
    pub fn state_by_name(&self, name: &str) -> Option<State> {
        self.states
            .iter()
            .position(|s| s.name == name)
            .and_then(|i| u16::try_from(i).ok())
            .map(State::new)
    }

    /// Whether `q` ∈ F.
    pub fn is_final(&self, q: State) -> bool {
        self.states.get(q.index()).is_some_and(|s| s.is_final)
    }

    /// How the output of a transducer is read as a number.
    pub fn decoder(&self) -> Decoder {
        self.decoder
    }

    /// The input word stored with the machine, used when no other input is given.
    pub fn default_input(&self) -> &str {
        &self.input
    }

    /// The column of the transition table for the tuple `read` (one symbol per tape).
    pub fn read_index(&self, read: &[Symbol]) -> usize {
        let k = self.alphabet.len();
        read.iter()
            .take(self.tapes)
            .rev()
            .fold(0, |acc, s| acc * k + s.index())
    }

    /// δ(q, read), where `read` holds one symbol per tape.
    #[inline]
    pub fn action(&self, q: State, read: &[Symbol]) -> Option<Action> {
        self.table
            .get(q.index() * self.stride + self.read_index(read))
            .copied()
            .flatten()
    }

    /// δ(q, a) for a single-tape machine.
    #[inline]
    pub fn action1(&self, q: State, a: Symbol) -> Option<Action> {
        self.table
            .get(q.index() * self.stride + a.index())
            .copied()
            .flatten()
    }

    /// The index of the transition in the builder (or spec) order that defines δ(q, read).
    pub fn origin(&self, q: State, read: &[Symbol]) -> Option<usize> {
        self.origin
            .get(q.index() * self.stride + self.read_index(read))
            .copied()
            .filter(|&i| i != u32::MAX)
            .map(|i| i as usize)
    }

    /// The number of defined entries of δ.
    pub fn num_transitions(&self) -> usize {
        self.table.iter().filter(|a| a.is_some()).count()
    }

    /// All defined entries of δ, by state and then by read tuple.
    pub fn transitions(&self) -> impl Iterator<Item = Transition> + '_ {
        let k = self.alphabet.len();
        self.table
            .iter()
            .enumerate()
            .filter_map(move |(i, action)| {
                let action = (*action)?;
                let from = State::new(u16::try_from(i / self.stride).ok()?);
                let mut column = i % self.stride;
                let mut read = [Symbol::BLANK; MAX_TAPES];
                for slot in read.iter_mut().take(self.tapes) {
                    *slot = Symbol::new(u8::try_from(column % k).ok()?);
                    column /= k;
                }
                Some(Transition { from, read, action })
            })
    }

    /// Converts an input word to symbols.
    ///
    /// The blank glyph is accepted inside the word, so that inputs such as `#x␣x` can be
    /// written; every other glyph must belong to the input alphabet Σ.
    ///
    /// # Errors
    ///
    /// Returns the first glyph that is not in Σ.
    pub fn encode_input(&self, word: &str) -> Result<Vec<Symbol>, InputError> {
        word.chars()
            .enumerate()
            .map(|(position, glyph)| {
                self.alphabet
                    .symbol(glyph)
                    .filter(|&s| s.is_blank() || self.alphabet.is_input(s))
                    .ok_or(InputError { glyph, position })
            })
            .collect()
    }
}
