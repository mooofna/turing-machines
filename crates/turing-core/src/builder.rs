//! Building and validating a [`Machine`] from names and glyphs.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::machine::{Decoder, Kind, Machine};
use crate::symbol::Dir;

/// One problem found while building a machine.
///
/// `index` fields refer to transitions in the order they were added to the builder.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum BuildError {
    /// The same state was declared twice.
    #[error("the state `{name}` is declared twice")]
    DuplicateState {
        /// The state name.
        name: String,
    },
    /// A transition or the start or final set names a state that was not declared, while the
    /// state set was declared explicitly.
    #[error("the state `{name}` is not declared in Q")]
    UnknownState {
        /// The state name.
        name: String,
        /// The transition that mentions it, if any.
        index: Option<usize>,
    },
    /// A transition uses a glyph outside Γ, while Γ was declared explicitly.
    #[error("the symbol `{glyph}` is not in the tape alphabet")]
    UnknownSymbol {
        /// The glyph.
        glyph: char,
        /// The transition that uses it, if any.
        index: Option<usize>,
    },
    /// δ(q, a) is defined twice.
    #[error("δ({state}, {read}) is defined twice")]
    DuplicateTransition {
        /// The state name.
        state: String,
        /// The read symbols, comma separated.
        read: String,
        /// The later definition.
        index: usize,
        /// The first definition.
        first: usize,
    },
    /// A transition has the wrong number of read symbols, written symbols or moves.
    #[error("the transition has {found} entries, but the machine has {tapes} tapes")]
    WrongArity {
        /// The number of tapes.
        tapes: usize,
        /// The number of entries found.
        found: usize,
        /// The transition.
        index: usize,
    },
    /// There are no states at all.
    #[error("the machine has no states")]
    MissingStart,
    /// The blank was put into the input alphabet.
    #[error("the blank `{blank}` cannot be part of the input alphabet")]
    BlankInInputAlphabet {
        /// The blank glyph.
        blank: char,
    },
    /// Σ ⊄ Γ.
    #[error("the input symbol `{glyph}` is not in the tape alphabet")]
    InputNotInTape {
        /// The glyph.
        glyph: char,
    },
    /// The tape count is 0 or larger than [`MAX_TAPES`](crate::MAX_TAPES).
    #[error("a machine has 1 to {max} tapes")]
    TapeCount {
        /// The largest supported count.
        max: usize,
    },
    /// Too many states.
    #[error("a machine has at most {max} states")]
    TooManyStates {
        /// The limit.
        max: usize,
    },
    /// Too many symbols.
    #[error("a tape alphabet has at most {max} symbols")]
    TooManySymbols {
        /// The limit.
        max: usize,
    },
    /// The transition table would be too large (|Q| × |Γ|^k entries).
    #[error("the transition table would have more than {max} entries")]
    TableTooLarge {
        /// The limit.
        max: usize,
    },
}

/// Every problem found by [`MachineBuilder::build`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildErrors(pub Vec<BuildError>);

impl fmt::Display for BuildErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, e) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{e}")?;
        }
        Ok(())
    }
}

impl core::error::Error for BuildErrors {}

/// A transition as given to the builder, by names and glyphs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PendingTransition {
    pub(crate) from: String,
    pub(crate) read: Vec<char>,
    pub(crate) to: String,
    pub(crate) write: Vec<char>,
    pub(crate) dir: Vec<Dir>,
}

/// Builds a [`Machine`] from state names and glyphs and reports every problem at once.
///
/// Sets that are not given are inferred:
///
/// * Q is the declared states followed by every other state in order of first mention
///   (start, transitions, final states).
/// * q0 is the declared start state, otherwise the first state in Q.
/// * Γ is the blank, the declared tape symbols and every glyph used in a transition.
/// * Σ is the declared input alphabet, otherwise Γ without the blank.
///
/// If Q or Γ is declared explicitly ([`MachineBuilder::declare_states`],
/// [`MachineBuilder::tape_alphabet`]), names outside it are errors instead.
#[derive(Clone, Debug)]
pub struct MachineBuilder {
    pub(crate) name: String,
    pub(crate) kind: Kind,
    pub(crate) tapes: usize,
    pub(crate) blank: char,
    pub(crate) input_alphabet: Option<Vec<char>>,
    pub(crate) tape_alphabet: Option<Vec<char>>,
    pub(crate) states: Vec<String>,
    pub(crate) states_declared: bool,
    pub(crate) start: Option<String>,
    pub(crate) finals: Vec<String>,
    pub(crate) transitions: Vec<PendingTransition>,
    pub(crate) decoder: Decoder,
    pub(crate) input: String,
}

impl MachineBuilder {
    /// A builder for a single-tape transducer called `name` with blank `␣`.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: Kind::default(),
            tapes: 1,
            blank: '␣',
            input_alphabet: None,
            tape_alphabet: None,
            states: Vec::new(),
            states_declared: false,
            start: None,
            finals: Vec::new(),
            transitions: Vec::new(),
            decoder: Decoder::default(),
            input: String::new(),
        }
    }

    /// Sets what the machine is used for.
    #[must_use]
    pub fn kind(mut self, kind: Kind) -> Self {
        self.kind = kind;
        self
    }

    /// Sets the number of tapes (1 to [`MAX_TAPES`](crate::MAX_TAPES)).
    #[must_use]
    pub fn tapes(mut self, tapes: usize) -> Self {
        self.tapes = tapes;
        self
    }

    /// Sets the blank glyph.
    #[must_use]
    pub fn blank(mut self, blank: char) -> Self {
        self.blank = blank;
        self
    }

    /// Declares Σ.
    #[must_use]
    pub fn input_alphabet(mut self, glyphs: impl IntoIterator<Item = char>) -> Self {
        self.input_alphabet = Some(glyphs.into_iter().collect());
        self
    }

    /// Declares Γ. Glyphs outside it become errors.
    #[must_use]
    pub fn tape_alphabet(mut self, glyphs: impl IntoIterator<Item = char>) -> Self {
        self.tape_alphabet = Some(glyphs.into_iter().collect());
        self
    }

    /// Adds a state to Q without declaring Q explicitly.
    #[must_use]
    pub fn state(mut self, name: &str) -> Self {
        self.states.push(name.into());
        self
    }

    /// Declares Q explicitly. States outside it become errors.
    #[must_use]
    pub fn declare_states<'a>(mut self, names: impl IntoIterator<Item = &'a str>) -> Self {
        self.states.extend(names.into_iter().map(String::from));
        self.states_declared = true;
        self
    }

    /// Sets q0.
    #[must_use]
    pub fn start(mut self, name: &str) -> Self {
        self.start = Some(name.into());
        self
    }

    /// Adds a state to F.
    #[must_use]
    pub fn final_state(mut self, name: &str) -> Self {
        self.finals.push(name.into());
        self
    }

    /// Sets how the output is read as a number.
    #[must_use]
    pub fn decoder(mut self, decoder: Decoder) -> Self {
        self.decoder = decoder;
        self
    }

    /// Stores a default input word.
    #[must_use]
    pub fn default_input(mut self, word: &str) -> Self {
        self.input = word.into();
        self
    }

    /// Adds δ(`from`, `read`) = (`to`, `write`, `dir`) for a single-tape machine.
    #[must_use]
    pub fn transition(self, from: &str, read: char, to: &str, write: char, dir: Dir) -> Self {
        self.transition_multi(from, &[read], to, &[write], &[dir])
    }

    /// Adds a transition with one read symbol, written symbol and move per tape.
    #[must_use]
    pub fn transition_multi(
        mut self,
        from: &str,
        read: &[char],
        to: &str,
        write: &[char],
        dir: &[Dir],
    ) -> Self {
        self.transitions.push(PendingTransition {
            from: from.into(),
            read: read.to_vec(),
            to: to.into(),
            write: write.to_vec(),
            dir: dir.to_vec(),
        });
        self
    }

    /// Validates everything and builds the machine.
    ///
    /// # Errors
    ///
    /// Returns every [`BuildError`] found, not only the first.
    pub fn build(self) -> Result<Machine, BuildErrors> {
        todo!("A1: validate, infer Q, Γ, Σ and q0, fill the dense table and origins")
    }
}
