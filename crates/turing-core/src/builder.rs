//! Building and validating a [`Machine`] from names and glyphs.

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use crate::alphabet::Alphabet;
use crate::machine::{Action, Decoder, Kind, Machine, Packed, StateInfo};
use crate::symbol::{Dir, MAX_STATES, MAX_SYMBOLS, MAX_TAPES, State, Symbol};

/// The largest transition table, in entries: |Q| × |Γ|^k must not exceed it.
const MAX_TABLE_ENTRIES: usize = 1 << 22;

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
    /// The tape count is 0 or larger than [`MAX_TAPES`].
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
///   (start, transitions, final states). Within a transition the source state comes before
///   the target state.
/// * q0 is the declared start state, otherwise the first state in Q.
/// * Γ is the blank, the declared tape symbols and every glyph used in a transition. If Γ is
///   not declared, it is the blank, the declared input alphabet and then every glyph used in a
///   transition in order of first appearance, read symbols before written ones.
/// * Σ is the declared input alphabet, otherwise Γ without the blank.
///
/// Symbols are numbered in the order of Γ, so the blank is always symbol 0, and states in the
/// order of Q.
///
/// If Q or Γ is declared explicitly ([`MachineBuilder::declare_states`],
/// [`MachineBuilder::tape_alphabet`]), names outside it are errors instead.
///
/// ```
/// use turing_core::{BuildError, Dir, Machine};
/// let errors = Machine::builder("broken")
///     .declare_states(["q0"])
///     .tape_alphabet(['a'])
///     .transition("q0", 'a', "q1", 'b', Dir::R)
///     .transition("q0", 'a', "q0", 'a', Dir::L)
///     .build()
///     .unwrap_err();
/// assert_eq!(
///     errors.0,
///     [
///         BuildError::UnknownState { name: "q1".into(), index: Some(0) },
///         BuildError::UnknownSymbol { glyph: 'b', index: Some(0) },
///         BuildError::DuplicateTransition {
///             state: "q0".into(),
///             read: "a".into(),
///             index: 1,
///             first: 0,
///         },
///     ]
/// );
/// ```
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

    /// Sets the number of tapes (1 to [`MAX_TAPES`]).
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
        let mut errors = Vec::new();
        let tapes_ok = (1..=MAX_TAPES).contains(&self.tapes);
        if !tapes_ok {
            errors.push(BuildError::TapeCount { max: MAX_TAPES });
        }

        let states = self.collect_states(&mut errors);
        let gamma = self.collect_tape_alphabet();
        let sigma = self.collect_input_alphabet(&gamma, &mut errors);

        // Names of undeclared states mentioned outside transitions, reported once each.
        let mut unknown_outside = BTreeSet::new();
        if self.states_declared
            && let Some(start) = &self.start
            && states.get(start.as_str()).is_none()
            && unknown_outside.insert(start.as_str())
        {
            errors.push(BuildError::UnknownState {
                name: start.clone(),
                index: None,
            });
        }

        let resolved = self.resolve_transitions(tapes_ok, &states, &gamma, &mut errors);

        if self.states_declared {
            for name in &self.finals {
                if states.get(name.as_str()).is_none() && unknown_outside.insert(name.as_str()) {
                    errors.push(BuildError::UnknownState {
                        name: name.clone(),
                        index: None,
                    });
                }
            }
        }

        if states.is_empty() {
            errors.push(BuildError::MissingStart);
        }
        if states.len() > MAX_STATES {
            errors.push(BuildError::TooManyStates { max: MAX_STATES });
        }
        if gamma.len() > MAX_SYMBOLS {
            errors.push(BuildError::TooManySymbols { max: MAX_SYMBOLS });
        }
        let entries = table_entries(states.len(), gamma.len(), self.tapes);
        if tapes_ok && entries.is_none_or(|n| n > MAX_TABLE_ENTRIES) {
            errors.push(BuildError::TableTooLarge {
                max: MAX_TABLE_ENTRIES,
            });
        }

        if let (true, Some(resolved), Some(entries)) = (errors.is_empty(), resolved, entries) {
            self.assemble(&states, &gamma, &sigma, &resolved, entries)
        } else {
            debug_assert!(!errors.is_empty(), "an unbuildable machine must report why");
            Err(BuildErrors(errors))
        }
    }

    /// Q in index order: the explicitly added states, then (unless Q is declared) every state
    /// in order of first mention. Reports states that were added twice.
    fn collect_states(&self, errors: &mut Vec<BuildError>) -> Interner<&str> {
        let mut states = Interner::default();
        let mut reported = BTreeSet::new();
        for name in &self.states {
            if !states.insert(name.as_str()) && reported.insert(name.as_str()) {
                errors.push(BuildError::DuplicateState { name: name.clone() });
            }
        }
        if !self.states_declared {
            let mentions = self
                .start
                .iter()
                .chain(self.transitions.iter().flat_map(|t| [&t.from, &t.to]))
                .chain(&self.finals);
            for name in mentions {
                states.insert(name.as_str());
            }
        }
        states
    }

    /// Γ in index order, blank first. See the type documentation for the inference rule.
    fn collect_tape_alphabet(&self) -> Interner<char> {
        let mut gamma = Interner::default();
        gamma.insert(self.blank);
        if let Some(declared) = &self.tape_alphabet {
            for &glyph in declared {
                gamma.insert(glyph);
            }
        } else {
            for &glyph in self.input_alphabet.iter().flatten() {
                gamma.insert(glyph);
            }
            for t in &self.transitions {
                for &glyph in t.read.iter().chain(&t.write) {
                    gamma.insert(glyph);
                }
            }
        }
        gamma
    }

    /// Σ in the order given, without duplicates. Reports a blank in Σ and glyphs outside Γ.
    fn collect_input_alphabet(
        &self,
        gamma: &Interner<char>,
        errors: &mut Vec<BuildError>,
    ) -> Vec<char> {
        let Some(declared) = &self.input_alphabet else {
            return gamma.items[1..].to_vec();
        };
        if declared.contains(&self.blank) {
            errors.push(BuildError::BlankInInputAlphabet { blank: self.blank });
        }
        let mut sigma = Vec::with_capacity(declared.len());
        for &glyph in declared {
            if glyph == self.blank || sigma.contains(&glyph) {
                continue;
            }
            if gamma.get(glyph).is_none() {
                errors.push(BuildError::InputNotInTape { glyph });
            }
            sigma.push(glyph);
        }
        sigma
    }

    /// Checks every transition in order and converts it to indices.
    ///
    /// Returns `None` if any transition could not be resolved; an error has been reported for
    /// it then.
    fn resolve_transitions(
        &self,
        tapes_ok: bool,
        states: &Interner<&str>,
        gamma: &Interner<char>,
        errors: &mut Vec<BuildError>,
    ) -> Option<Vec<Resolved>> {
        let mut first_definition: BTreeMap<(&str, &[char]), usize> = BTreeMap::new();
        let mut resolved = Vec::with_capacity(self.transitions.len());
        let mut complete = true;
        for (index, t) in self.transitions.iter().enumerate() {
            let arity = [t.read.len(), t.write.len(), t.dir.len()]
                .into_iter()
                .find(|&found| found != self.tapes);
            if let (true, Some(found)) = (tapes_ok, arity) {
                errors.push(BuildError::WrongArity {
                    tapes: self.tapes,
                    found,
                    index,
                });
            }

            let from = states.get(t.from.as_str());
            let to = states.get(t.to.as_str());
            if from.is_none() {
                errors.push(BuildError::UnknownState {
                    name: t.from.clone(),
                    index: Some(index),
                });
            }
            if to.is_none() && t.to != t.from {
                errors.push(BuildError::UnknownState {
                    name: t.to.clone(),
                    index: Some(index),
                });
            }

            let mut unknown_glyphs: Vec<char> = Vec::new();
            for &glyph in t.read.iter().chain(&t.write) {
                if gamma.get(glyph).is_none() && !unknown_glyphs.contains(&glyph) {
                    unknown_glyphs.push(glyph);
                    errors.push(BuildError::UnknownSymbol {
                        glyph,
                        index: Some(index),
                    });
                }
            }

            match first_definition.get(&(t.from.as_str(), t.read.as_slice())) {
                Some(&first) => errors.push(BuildError::DuplicateTransition {
                    state: t.from.clone(),
                    read: join_glyphs(&t.read),
                    index,
                    first,
                }),
                None => {
                    first_definition.insert((t.from.as_str(), t.read.as_slice()), index);
                }
            }

            let r = if tapes_ok && arity.is_none() {
                Resolved::new(t, from, to, gamma)
            } else {
                None
            };
            match r {
                Some(r) => resolved.push(r),
                None => complete = false,
            }
        }
        complete.then_some(resolved)
    }

    /// Builds the machine from fully validated parts.
    fn assemble(
        &self,
        states: &Interner<&str>,
        gamma: &Interner<char>,
        sigma: &[char],
        resolved: &[Resolved],
        entries: usize,
    ) -> Result<Machine, BuildErrors> {
        let alphabet = Alphabet::new(self.blank, sigma, &gamma.items[1..])
            .map_err(|e| BuildErrors(vec![e]))?;
        let stride = entries / states.len();
        let mut table: Box<[Option<Action>]> = vec![None; entries].into_boxed_slice();
        let mut origin: Box<[u32]> = vec![u32::MAX; entries].into_boxed_slice();
        for (index, r) in resolved.iter().enumerate() {
            let column = r.read[..self.tapes]
                .iter()
                .rev()
                .fold(0, |acc, s| acc * gamma.len() + s.index());
            let slot = r.from.index() * stride + column;
            table[slot] = Some(r.action);
            // Distinct transitions fit into the table, which has at most 2^22 entries.
            origin[slot] = u32::try_from(index).unwrap_or(u32::MAX);
        }

        let packed = if self.tapes == 1 {
            table
                .iter()
                .map(|&entry| Packed::new(entry, stride))
                .collect()
        } else {
            Box::default()
        };

        let finals: BTreeSet<&str> = self.finals.iter().map(String::as_str).collect();
        let start = self
            .start
            .as_deref()
            .and_then(|name| states.get(name))
            .unwrap_or(0);
        let state_infos = states
            .items
            .iter()
            .map(|&name| StateInfo {
                name: name.into(),
                is_final: finals.contains(name),
            })
            .collect();

        Ok(Machine {
            name: self.name.clone(),
            kind: self.kind,
            tapes: self.tapes,
            alphabet,
            states: state_infos,
            start: State::new(u16::try_from(start).unwrap_or(0)),
            stride,
            table,
            origin,
            packed,
            decoder: self.decoder,
            input: self.input.clone(),
        })
    }
}

/// |Q| × |Γ|^k, or `None` on overflow.
fn table_entries(states: usize, symbols: usize, tapes: usize) -> Option<usize> {
    let tapes = u32::try_from(tapes).ok()?;
    symbols.checked_pow(tapes)?.checked_mul(states)
}

/// Joins glyphs with `, ` for messages.
fn join_glyphs(glyphs: &[char]) -> String {
    let mut out = String::new();
    for (i, &glyph) in glyphs.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push(glyph);
    }
    out
}

/// A list of distinct items in insertion order with a reverse index.
#[derive(Debug)]
struct Interner<T> {
    items: Vec<T>,
    index: BTreeMap<T, usize>,
}

impl<T> Default for Interner<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            index: BTreeMap::new(),
        }
    }
}

impl<T: Ord + Copy> Interner<T> {
    /// Appends `item` unless present; returns whether it was new.
    fn insert(&mut self, item: T) -> bool {
        if self.index.contains_key(&item) {
            return false;
        }
        self.index.insert(item, self.items.len());
        self.items.push(item);
        true
    }

    /// The index of `item`.
    fn get(&self, item: T) -> Option<usize> {
        self.index.get(&item).copied()
    }

    /// The number of distinct items.
    fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether nothing was inserted.
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// A transition converted to state and symbol indices.
#[derive(Copy, Clone, Debug)]
struct Resolved {
    from: State,
    read: [Symbol; MAX_TAPES],
    action: Action,
}

impl Resolved {
    /// Converts `t`, whose arity is known to be right. `None` if a name is unknown or an index
    /// exceeds the limits.
    fn new(
        t: &PendingTransition,
        from: Option<usize>,
        to: Option<usize>,
        gamma: &Interner<char>,
    ) -> Option<Self> {
        let state = |i: usize| u16::try_from(i).ok().map(State::new);
        let symbol = |glyph: char| {
            gamma
                .get(glyph)
                .and_then(|i| u8::try_from(i).ok())
                .map(Symbol::new)
        };
        let mut read = [Symbol::BLANK; MAX_TAPES];
        let mut write = [Symbol::BLANK; MAX_TAPES];
        let mut dir = [Dir::N; MAX_TAPES];
        for (tape, ((&r, &w), &d)) in t.read.iter().zip(&t.write).zip(&t.dir).enumerate() {
            read[tape] = symbol(r)?;
            write[tape] = symbol(w)?;
            dir[tape] = d;
        }
        Some(Self {
            from: state(from?)?,
            read,
            action: Action {
                next: state(to?)?,
                write,
                dir,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::ToString;

    use super::*;
    use crate::machine::Transition;

    fn names(m: &Machine) -> Vec<&str> {
        m.states().iter().map(|s| s.name.as_str()).collect()
    }

    fn errors(builder: MachineBuilder) -> Vec<BuildError> {
        builder
            .build()
            .expect_err("the machine should be rejected")
            .0
    }

    #[test]
    fn states_are_inferred_in_order_of_first_mention() {
        let m = Machine::builder("order")
            .state("s")
            .start("a")
            .transition("b", 'x', "c", 'x', Dir::R)
            .transition("a", 'x', "b", 'x', Dir::R)
            .transition("c", 'x', "d", 'x', Dir::R)
            .final_state("e")
            .final_state("d")
            .build()
            .unwrap();
        assert_eq!(names(&m), ["s", "a", "b", "c", "d", "e"]);
        assert_eq!(m.start(), m.state_by_name("a").unwrap());
        assert!(m.is_final(m.state_by_name("d").unwrap()));
        assert!(m.is_final(m.state_by_name("e").unwrap()));
        assert!(!m.is_final(m.state_by_name("a").unwrap()));
    }

    #[test]
    fn the_first_state_is_the_default_start() {
        let m = Machine::builder("start")
            .transition("p", '1', "q", '1', Dir::L)
            .build()
            .unwrap();
        assert_eq!(m.start(), State::new(0));
        assert_eq!(m.state(m.start()).name, "p");

        let declared = Machine::builder("start")
            .declare_states(["z", "p", "q"])
            .transition("p", '1', "q", '1', Dir::L)
            .build()
            .unwrap();
        assert_eq!(names(&declared), ["z", "p", "q"]);
        assert_eq!(declared.state(declared.start()).name, "z");
    }

    #[test]
    fn tape_alphabet_is_inferred_from_input_then_transitions() {
        let m = Machine::builder("gamma")
            .blank('_')
            .input_alphabet(['b', 'a'])
            .transition("q", 'c', "q", 'd', Dir::R)
            .transition("q", 'a', "q", '_', Dir::R)
            .transition("q", 'e', "q", 'c', Dir::R)
            .build()
            .unwrap();
        assert_eq!(m.alphabet().glyphs(), ['_', 'b', 'a', 'c', 'd', 'e']);
        assert_eq!(m.alphabet().input_glyphs().collect::<Vec<_>>(), ['b', 'a']);
    }

    #[test]
    fn input_alphabet_defaults_to_all_non_blank_symbols() {
        let m = Machine::builder("sigma")
            .transition("q", 'x', "q", 'X', Dir::R)
            .build()
            .unwrap();
        assert_eq!(m.alphabet().glyphs(), ['␣', 'x', 'X']);
        assert_eq!(m.alphabet().input_glyphs().collect::<Vec<_>>(), ['x', 'X']);
        assert_eq!(m.alphabet().blank(), '␣');
    }

    #[test]
    fn a_declared_tape_alphabet_keeps_its_order() {
        let m = Machine::builder("declared")
            .tape_alphabet(['#', '␣', 'x', 'x'])
            .input_alphabet(['x'])
            .transition("q", 'x', "q", '#', Dir::R)
            .build()
            .unwrap();
        assert_eq!(m.alphabet().glyphs(), ['␣', '#', 'x']);
        assert_eq!(m.alphabet().input_glyphs().collect::<Vec<_>>(), ['x']);
    }

    #[test]
    fn table_and_origins_follow_builder_order() {
        let m = Machine::builder("table")
            .transition("q0", '1', "q1", '0', Dir::R)
            .transition("q1", '␣', "q0", '1', Dir::L)
            .transition("q0", '␣', "q0", '1', Dir::N)
            .build()
            .unwrap();
        let blank = Symbol::BLANK;
        let one = m.alphabet().symbol('1').unwrap();
        let zero = m.alphabet().symbol('0').unwrap();
        let (q0, q1) = (State::new(0), State::new(1));
        assert_eq!(m.stride, 3);
        assert_eq!(m.table.len(), 6);
        assert_eq!(m.action1(q0, one), Some(Action::single(zero, Dir::R, q1)));
        assert_eq!(m.action1(q1, blank), Some(Action::single(one, Dir::L, q0)));
        assert_eq!(m.action1(q0, blank), Some(Action::single(one, Dir::N, q0)));
        assert_eq!(m.action1(q1, one), None);
        assert_eq!(m.origin(q0, &[one]), Some(0));
        assert_eq!(m.origin(q1, &[blank]), Some(1));
        assert_eq!(m.origin(q0, &[blank]), Some(2));
        assert_eq!(m.origin(q1, &[zero]), None);
        assert_eq!(m.num_transitions(), 3);
    }

    #[test]
    fn multi_tape_columns_are_little_endian() {
        let m = Machine::builder("two")
            .tapes(2)
            .tape_alphabet(['a', 'b'])
            .transition_multi("q", &['b', '␣'], "r", &['a', 'b'], &[Dir::R, Dir::L])
            .transition_multi("q", &['␣', 'a'], "q", &['␣', 'a'], &[Dir::N, Dir::R])
            .build()
            .unwrap();
        let (a, b) = (Symbol::new(1), Symbol::new(2));
        assert_eq!(m.stride, 9);
        assert_eq!(m.read_index(&[b, Symbol::BLANK]), 2);
        assert_eq!(m.read_index(&[Symbol::BLANK, a]), 3);
        let action = m.action(State::new(0), &[b, Symbol::BLANK]).unwrap();
        assert_eq!(action.next, State::new(1));
        assert_eq!(action.write, [a, b, Symbol::BLANK]);
        assert_eq!(action.dir, [Dir::R, Dir::L, Dir::N]);
        assert_eq!(m.origin(State::new(0), &[Symbol::BLANK, a]), Some(1));
        let listed: Vec<Transition> = m.transitions().collect();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].read, [b, Symbol::BLANK, Symbol::BLANK]);
        assert_eq!(listed[1].read, [Symbol::BLANK, a, Symbol::BLANK]);
    }

    #[test]
    fn metadata_is_kept() {
        let m = Machine::builder("meta")
            .kind(Kind::Acceptor)
            .decoder(Decoder::Unary { symbol: 'x' })
            .default_input("xx")
            .transition("q", 'x', "q", 'x', Dir::R)
            .build()
            .unwrap();
        assert_eq!(m.name(), "meta");
        assert_eq!(m.kind(), Kind::Acceptor);
        assert_eq!(m.decoder(), Decoder::Unary { symbol: 'x' });
        assert_eq!(m.default_input(), "xx");
        assert_eq!(m.tapes(), 1);
    }

    #[test]
    fn a_machine_without_transitions_is_valid() {
        let m = Machine::builder("idle").state("q").build().unwrap();
        assert_eq!(m.num_states(), 1);
        assert_eq!(m.num_symbols(), 1);
        assert_eq!(m.num_transitions(), 0);
    }

    #[test]
    fn tape_count_is_checked() {
        for tapes in [0, MAX_TAPES + 1] {
            let errs = errors(Machine::builder("tapes").tapes(tapes).transition(
                "q",
                'a',
                "q",
                'a',
                Dir::R,
            ));
            assert_eq!(errs, [BuildError::TapeCount { max: MAX_TAPES }]);
        }
    }

    #[test]
    fn duplicate_states_are_reported_once() {
        let errs = errors(
            Machine::builder("dup")
                .declare_states(["a", "b", "a", "a"])
                .state("b"),
        );
        assert_eq!(
            errs,
            [
                BuildError::DuplicateState { name: "a".into() },
                BuildError::DuplicateState { name: "b".into() },
            ]
        );
    }

    #[test]
    fn undeclared_states_are_reported_where_they_occur() {
        let errs = errors(
            Machine::builder("unknown")
                .declare_states(["q0"])
                .start("s")
                .transition("q0", 'a', "q0", 'a', Dir::R)
                .transition("p", 'a', "p", 'a', Dir::R)
                .transition("q0", 'b', "r", 'a', Dir::R)
                .final_state("s")
                .final_state("f"),
        );
        assert_eq!(
            errs,
            [
                BuildError::UnknownState {
                    name: "s".into(),
                    index: None
                },
                BuildError::UnknownState {
                    name: "p".into(),
                    index: Some(1)
                },
                BuildError::UnknownState {
                    name: "r".into(),
                    index: Some(2)
                },
                BuildError::UnknownState {
                    name: "f".into(),
                    index: None
                },
            ]
        );
    }

    #[test]
    fn unknown_symbols_are_reported_once_per_transition() {
        let errs = errors(
            Machine::builder("symbols")
                .tape_alphabet(['a'])
                .transition("q", 'b', "q", 'b', Dir::R)
                .transition("q", 'a', "q", 'c', Dir::R),
        );
        assert_eq!(
            errs,
            [
                BuildError::UnknownSymbol {
                    glyph: 'b',
                    index: Some(0)
                },
                BuildError::UnknownSymbol {
                    glyph: 'c',
                    index: Some(1)
                },
            ]
        );
    }

    #[test]
    fn duplicate_transitions_name_both_definitions() {
        let errs = errors(
            Machine::builder("dup")
                .tapes(2)
                .transition_multi("q", &['a', '␣'], "q", &['a', 'a'], &[Dir::R, Dir::R])
                .transition_multi("q", &['a', 'a'], "q", &['a', 'a'], &[Dir::R, Dir::R])
                .transition_multi("q", &['a', '␣'], "r", &['␣', 'a'], &[Dir::L, Dir::R]),
        );
        assert_eq!(
            errs,
            [BuildError::DuplicateTransition {
                state: "q".into(),
                read: "a, ␣".into(),
                index: 2,
                first: 0,
            }]
        );
        assert_eq!(errs[0].to_string(), "δ(q, a, ␣) is defined twice");
    }

    #[test]
    fn arity_is_checked_for_reads_writes_and_moves() {
        let errs = errors(
            Machine::builder("arity")
                .tapes(2)
                .transition("q", 'a', "q", 'a', Dir::R)
                .transition_multi("q", &['a', 'a'], "q", &['a'], &[Dir::R, Dir::R])
                .transition_multi("q", &['a', '␣'], "q", &['a', 'a'], &[Dir::R; 3])
                .transition_multi("q", &['␣', '␣'], "q", &['a', 'a'], &[Dir::R, Dir::R]),
        );
        assert_eq!(
            errs,
            [
                BuildError::WrongArity {
                    tapes: 2,
                    found: 1,
                    index: 0
                },
                BuildError::WrongArity {
                    tapes: 2,
                    found: 1,
                    index: 1
                },
                BuildError::WrongArity {
                    tapes: 2,
                    found: 3,
                    index: 2
                },
            ]
        );
    }

    #[test]
    fn an_empty_machine_has_no_start() {
        assert_eq!(
            errors(Machine::builder("empty")),
            [BuildError::MissingStart]
        );
        assert_eq!(
            errors(Machine::builder("empty").declare_states([])),
            [BuildError::MissingStart]
        );
    }

    #[test]
    fn the_blank_cannot_be_an_input_symbol() {
        let errs = errors(
            Machine::builder("blank")
                .input_alphabet(['x', '␣'])
                .transition("q", 'x', "q", 'x', Dir::R),
        );
        assert_eq!(errs, [BuildError::BlankInInputAlphabet { blank: '␣' }]);
    }

    #[test]
    fn input_symbols_must_be_tape_symbols() {
        let errs = errors(
            Machine::builder("sigma")
                .tape_alphabet(['x'])
                .input_alphabet(['x', 'y', 'z', 'y'])
                .transition("q", 'x', "q", 'x', Dir::R),
        );
        assert_eq!(
            errs,
            [
                BuildError::InputNotInTape { glyph: 'y' },
                BuildError::InputNotInTape { glyph: 'z' },
            ]
        );
    }

    #[test]
    fn state_limit_is_enforced() {
        let names: Vec<String> = (0..=MAX_STATES).map(|i| format!("q{i}")).collect();
        let errs = errors(Machine::builder("big").declare_states(names.iter().map(String::as_str)));
        assert_eq!(errs, [BuildError::TooManyStates { max: MAX_STATES }]);
    }

    #[test]
    fn symbol_limit_is_enforced() {
        let glyphs = (0x100..).filter_map(char::from_u32).take(MAX_SYMBOLS);
        let errs = errors(Machine::builder("wide").state("q").tape_alphabet(glyphs));
        assert_eq!(errs, [BuildError::TooManySymbols { max: MAX_SYMBOLS }]);
    }

    #[test]
    fn table_size_is_limited() {
        // 64 symbols on 3 tapes give 2^18 columns, so 17 states exceed 2^22 entries.
        let glyphs: Vec<char> = (0x100..)
            .filter_map(char::from_u32)
            .take(MAX_SYMBOLS - 1)
            .collect();
        let names: Vec<String> = (0..17).map(|i| format!("q{i}")).collect();
        let build = |n: usize| {
            Machine::builder("huge")
                .tapes(3)
                .tape_alphabet(glyphs.iter().copied())
                .declare_states(names[..n].iter().map(String::as_str))
                .build()
        };
        assert_eq!(
            build(17).unwrap_err().0,
            [BuildError::TableTooLarge {
                max: MAX_TABLE_ENTRIES
            }]
        );
        assert_eq!(build(16).unwrap().table.len(), MAX_TABLE_ENTRIES);
    }

    #[test]
    fn all_errors_are_collected() {
        let errs = errors(
            Machine::builder("many")
                .tapes(1)
                .declare_states(["a", "a"])
                .input_alphabet(['␣'])
                .tape_alphabet(['x'])
                .transition("b", 'y', "a", 'x', Dir::R)
                .transition_multi("a", &['x', 'x'], "a", &['x'], &[Dir::R]),
        );
        assert_eq!(
            errs,
            [
                BuildError::DuplicateState { name: "a".into() },
                BuildError::BlankInInputAlphabet { blank: '␣' },
                BuildError::UnknownState {
                    name: "b".into(),
                    index: Some(0)
                },
                BuildError::UnknownSymbol {
                    glyph: 'y',
                    index: Some(0)
                },
                BuildError::WrongArity {
                    tapes: 1,
                    found: 2,
                    index: 1
                },
            ]
        );
        let shown = BuildErrors(errs).to_string();
        assert_eq!(shown.lines().count(), 5);
        assert!(shown.starts_with("the state `a` is declared twice\n"));
    }
}
