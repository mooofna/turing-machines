//! `MachineSpec`, the editable interchange form of a machine.
//!
//! A spec keeps everything an editor needs that a [`Machine`] does not: names in their
//! original order, which sets were declared explicitly, comments and source spans. The
//! description language, the transition table and the state diagram of the web front end all
//! edit a spec; [`MachineSpec::compile`] turns it into a runnable machine.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::builder::{BuildError, MachineBuilder};
use crate::diag::{Diagnostic, Span};
use crate::dsl::is_state_name;
use crate::machine::{Decoder, Kind, Machine};
use crate::symbol::{Dir, MAX_STATES, MAX_SYMBOLS, MAX_TAPES};

/// A machine described by names and glyphs.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct MachineSpec {
    /// The machine's name, possibly empty.
    pub name: String,
    /// What the machine is used for.
    pub kind: Kind,
    /// The number of tapes, 1 to [`MAX_TAPES`].
    pub tapes: u8,
    /// The blank glyph.
    pub blank: char,
    /// Σ, in order. Complete even if it was inferred.
    pub input_alphabet: Vec<char>,
    /// Γ, in order, blank included. Complete even if it was inferred.
    pub tape_alphabet: Vec<char>,
    /// Q, in order. Complete even if it was inferred.
    pub states: Vec<StateSpec>,
    /// The name of q0.
    pub start: String,
    /// δ, in source order. Each entry defines exactly one δ(q, a); sugar such as read sets is
    /// expanded by the parser.
    pub transitions: Vec<TransitionSpec>,
    /// The input word used when the machine is opened.
    pub input: String,
    /// How the output is read as a number.
    pub output: Decoder,
    /// Which parts were written explicitly, so that formatting reproduces them.
    pub declared: Declared,
    /// Comments that do not belong to a transition.
    pub comments: SpecComments,
}

/// A state of a [`MachineSpec`].
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct StateSpec {
    /// The state name.
    pub name: String,
    /// Whether the state belongs to F.
    pub is_final: bool,
}

/// One entry δ(from, read) = (to, write, dir) of a [`MachineSpec`].
///
/// `read`, `write` and `dir` have one entry per tape.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TransitionSpec {
    /// The current state.
    pub from: String,
    /// The symbols under the heads.
    pub read: Vec<char>,
    /// The next state.
    pub to: String,
    /// The symbols written.
    pub write: Vec<char>,
    /// The moves.
    pub dir: Vec<Dir>,
    /// A comment at the end of the line.
    pub comment: Option<String>,
    /// Comment lines directly above the transition.
    pub leading: Vec<String>,
    /// Where the transition was written, if it came from text.
    pub span: Option<Span>,
}

/// Which parts of a [`MachineSpec`] were declared explicitly in the source.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[expect(
    clippy::struct_excessive_bools,
    reason = "one flag per optional declaration"
)]
pub struct Declared {
    /// `Q = {...}`.
    pub states: bool,
    /// `Σ = {...}`.
    pub input_alphabet: bool,
    /// `Γ = {...}`.
    pub tape_alphabet: bool,
    /// `start = ...`.
    pub start: bool,
    /// `blank = ...`.
    pub blank: bool,
}

/// Comments of a [`MachineSpec`] that are not attached to a transition.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SpecComments {
    /// Comment lines before the first declaration, and comments that belong to a
    /// declaration rather than a transition.
    pub header: Vec<String>,
    /// Comment lines after the last transition.
    pub trailing: Vec<String>,
}

/// The largest transition table |Q| × |Γ|^k, in entries. It is the limit of
/// [`MachineBuilder`]; a test checks that both agree.
const MAX_TABLE_ENTRIES: usize = 1 << 22;

/// The stable diagnostic codes, documented in `docs/dsl.md`.
pub(crate) mod codes {
    /// A token that does not fit the grammar.
    pub(crate) const UNEXPECTED_TOKEN: &str = "E001";
    /// A string literal without its closing quote.
    pub(crate) const UNTERMINATED_STRING: &str = "E002";
    /// A state or symbol that is not declared.
    pub(crate) const UNKNOWN_NAME: &str = "E003";
    /// δ(q, a) defined twice.
    pub(crate) const DUPLICATE_TRANSITION: &str = "E004";
    /// No states, so no start state.
    pub(crate) const NO_STATES: &str = "E005";
    /// The blank in Σ.
    pub(crate) const BLANK_IN_INPUT: &str = "E006";
    /// Σ ⊄ Γ.
    pub(crate) const INPUT_NOT_IN_TAPE: &str = "E007";
    /// A move other than `L`, `R` or `N`.
    pub(crate) const INVALID_DIRECTION: &str = "E008";
    /// A machine beyond the supported size.
    pub(crate) const TOO_LARGE: &str = "E009";
    /// A transition with the wrong number of tape entries.
    pub(crate) const WRONG_ARITY: &str = "E010";
    /// A declaration or set element given twice.
    pub(crate) const DUPLICATE_DECLARATION: &str = "E011";
    /// A state name or glyph that the description language cannot write.
    pub(crate) const INVALID_NAME: &str = "E012";
    /// A state that cannot be reached from the start state.
    pub(crate) const UNREACHABLE_STATE: &str = "W001";
    /// An acceptor or enumerator without final states.
    pub(crate) const NO_FINAL_STATE: &str = "W002";
    /// A declared symbol that no transition reads or writes.
    pub(crate) const UNUSED_SYMBOL: &str = "W003";
}

/// Where the parts of a spec were written in its source text, so that diagnostics can point at
/// single tokens instead of whole lines.
///
/// Every vector runs parallel to the corresponding list of the spec; missing entries fall back
/// to coarser spans.
#[derive(Clone, Debug, Default)]
pub(crate) struct SourceMap {
    pub(crate) transitions: Vec<TransitionSpans>,
    /// The elements of `Q = {...}`.
    pub(crate) states: Vec<Span>,
    pub(crate) states_decl: Option<Span>,
    /// The elements of `Σ = {...}`.
    pub(crate) input_alphabet: Vec<Span>,
    /// The elements of `Γ = {...}`.
    pub(crate) tape_alphabet: Vec<Span>,
    /// The elements of `F = {...}`.
    pub(crate) finals: Vec<(String, Span)>,
    pub(crate) start: Option<Span>,
    pub(crate) tapes: Option<Span>,
    pub(crate) kind: Option<Span>,
    pub(crate) input: Option<Span>,
}

/// The token spans of one expanded transition.
#[derive(Clone, Debug, Default)]
pub(crate) struct TransitionSpans {
    pub(crate) from: Span,
    pub(crate) to: Span,
    pub(crate) read: Vec<Span>,
    pub(crate) write: Vec<Span>,
}

impl MachineSpec {
    /// An empty single-tape transducer with blank `␣`.
    pub fn new() -> Self {
        Self {
            tapes: 1,
            blank: '␣',
            tape_alphabet: alloc::vec!['␣'],
            ..Self::default()
        }
    }

    /// Completes the sets that were not declared, using the same rules as the parser:
    ///
    /// * Q: the declared start state, then every state in order of first mention in the
    ///   transitions, then the remaining states already listed (for example final states).
    /// * q0: the first state of Q.
    /// * Γ: the blank, a declared Σ, every glyph in the transitions in order of use, the glyphs
    ///   already listed and, if Σ is not declared either, the glyphs of the input word.
    ///   A declared Γ only gains the blank if it lacks it.
    /// * Σ: Γ without the blank.
    ///
    /// Declared sets are left alone, so names outside them stay errors for [`check`].
    /// Inferring twice changes nothing.
    ///
    /// [`check`]: MachineSpec::check
    ///
    /// ```
    /// use turing_core::{Dir, MachineSpec, TransitionSpec};
    /// let mut spec = MachineSpec::new();
    /// spec.transitions.push(TransitionSpec {
    ///     from: "q0".into(),
    ///     read: vec!['x'],
    ///     to: "q1".into(),
    ///     write: vec!['X'],
    ///     dir: vec![Dir::R],
    ///     ..TransitionSpec::default()
    /// });
    /// spec.infer();
    /// assert_eq!(spec.start, "q0");
    /// assert_eq!(spec.states.len(), 2);
    /// assert_eq!(spec.tape_alphabet, ['␣', 'x', 'X']);
    /// assert_eq!(spec.input_alphabet, ['x', 'X']);
    /// ```
    pub fn infer(&mut self) {
        if !self.declared.states {
            let old = core::mem::take(&mut self.states);
            let start = self.declared.start.then_some(self.start.as_str());
            let mentions = start
                .into_iter()
                .chain(
                    self.transitions
                        .iter()
                        .flat_map(|t| [t.from.as_str(), t.to.as_str()]),
                )
                .chain(old.iter().map(|s| s.name.as_str()));
            let finals: BTreeSet<&str> = old
                .iter()
                .filter(|s| s.is_final)
                .map(|s| s.name.as_str())
                .collect();
            let mut seen = BTreeSet::new();
            for name in mentions {
                if !name.is_empty() && seen.insert(name) {
                    self.states.push(StateSpec {
                        name: name.into(),
                        is_final: finals.contains(name),
                    });
                }
            }
        }
        if !self.declared.start {
            self.start = self
                .states
                .first()
                .map(|s| s.name.clone())
                .unwrap_or_default();
        }
        if self.declared.tape_alphabet {
            if !self.tape_alphabet.contains(&self.blank) {
                self.tape_alphabet.push(self.blank);
            }
        } else {
            let declared_sigma = self
                .declared
                .input_alphabet
                .then_some(&self.input_alphabet)
                .into_iter()
                .flatten();
            let used = self
                .transitions
                .iter()
                .flat_map(|t| t.read.iter().chain(&t.write));
            let input = if self.declared.input_alphabet {
                ""
            } else {
                self.input.as_str()
            };
            let mut gamma = alloc::vec![self.blank];
            let mut seen = BTreeSet::from([self.blank]);
            for glyph in declared_sigma
                .chain(used)
                .chain(&self.tape_alphabet)
                .copied()
                .chain(input.chars())
            {
                if seen.insert(glyph) {
                    gamma.push(glyph);
                }
            }
            self.tape_alphabet = gamma;
        }
        if !self.declared.input_alphabet {
            let blank = self.blank;
            self.input_alphabet = self
                .tape_alphabet
                .iter()
                .copied()
                .filter(|&c| c != blank)
                .collect();
        }
    }

    /// Checks the spec and reports every error and warning, ordered by source position
    /// (diagnostics without a span come last).
    ///
    /// The checks run on a copy with the undeclared sets inferred (see
    /// [`MachineSpec::infer`]). The codes are documented in `docs/dsl.md`.
    ///
    /// ```
    /// use turing_core::dsl;
    /// let spec = dsl::parse("δ(q0, x) = (q1, x, R)").spec.unwrap();
    /// assert!(spec.check().is_empty());
    /// ```
    pub fn check(&self) -> Vec<Diagnostic> {
        self.analyze(None).1
    }

    /// Builds the runnable machine.
    ///
    /// The transition order of the spec is kept, so [`Machine::origin`] returns indices into
    /// [`MachineSpec::transitions`].
    ///
    /// # Errors
    ///
    /// Returns all diagnostics (errors and warnings) if there is at least one error.
    ///
    /// ```
    /// use turing_core::dsl;
    /// let spec = dsl::parse("δ(q0, x) = (q1, x, R)").spec.unwrap();
    /// let machine = spec.compile().unwrap();
    /// assert_eq!(machine.num_states(), 2);
    /// ```
    pub fn compile(&self) -> Result<Machine, Vec<Diagnostic>> {
        let (spec, diagnostics) = self.analyze(None);
        if diagnostics.iter().any(Diagnostic::is_error) {
            return Err(diagnostics);
        }
        let mut builder = MachineBuilder::new(spec.name.as_str())
            .kind(spec.kind)
            .tapes(usize::from(spec.tapes))
            .blank(spec.blank)
            .input_alphabet(spec.input_alphabet.iter().copied())
            .tape_alphabet(spec.tape_alphabet.iter().copied())
            .declare_states(spec.states.iter().map(|s| s.name.as_str()))
            .start(&spec.start)
            .decoder(spec.output)
            .default_input(&spec.input);
        for state in spec.states.iter().filter(|s| s.is_final) {
            builder = builder.final_state(&state.name);
        }
        for t in &spec.transitions {
            builder = builder.transition_multi(&t.from, &t.read, &t.to, &t.write, &t.dir);
        }
        builder.build().map_err(|errors| {
            let mut diagnostics: Vec<_> =
                errors.0.iter().map(|e| spec.build_diagnostic(e)).collect();
            sort_diagnostics(&mut diagnostics);
            diagnostics
        })
    }

    /// The index of the state called `name`.
    pub fn state_index(&self, name: &str) -> Option<usize> {
        self.states.iter().position(|s| s.name == name)
    }

    /// The index of the transition δ(`from`, `read`), if defined.
    pub fn transition_index(&self, from: &str, read: &[char]) -> Option<usize> {
        self.transitions
            .iter()
            .position(|t| t.from == from && t.read == read)
    }

    /// Infers the undeclared sets of a copy and checks it, using `map` for precise spans.
    pub(crate) fn analyze(&self, map: Option<&SourceMap>) -> (Self, Vec<Diagnostic>) {
        let mut spec = self.clone();
        spec.infer();
        let mut diagnostics = Checker::new(&spec, map).run();
        sort_diagnostics(&mut diagnostics);
        (spec, diagnostics)
    }

    /// The span of transition `index`, if known.
    fn transition_span(&self, index: usize) -> Option<Span> {
        self.transitions.get(index).and_then(|t| t.span)
    }

    /// Converts an error of [`MachineBuilder`] into a diagnostic. [`MachineSpec::check`]
    /// reports all of these itself, so this only runs if the two ever disagree.
    fn build_diagnostic(&self, error: &BuildError) -> Diagnostic {
        let (code, span) = match *error {
            BuildError::UnknownState { index, .. } | BuildError::UnknownSymbol { index, .. } => (
                codes::UNKNOWN_NAME,
                index.and_then(|i| self.transition_span(i)),
            ),
            BuildError::DuplicateTransition { index, .. } => {
                (codes::DUPLICATE_TRANSITION, self.transition_span(index))
            }
            BuildError::WrongArity { index, .. } => {
                (codes::WRONG_ARITY, self.transition_span(index))
            }
            BuildError::DuplicateState { .. } => (codes::DUPLICATE_DECLARATION, None),
            BuildError::MissingStart => (codes::NO_STATES, None),
            BuildError::BlankInInputAlphabet { .. } => (codes::BLANK_IN_INPUT, None),
            BuildError::InputNotInTape { .. } => (codes::INPUT_NOT_IN_TAPE, None),
            BuildError::TapeCount { .. }
            | BuildError::TooManyStates { .. }
            | BuildError::TooManySymbols { .. }
            | BuildError::TableTooLarge { .. } => (codes::TOO_LARGE, None),
        };
        let diagnostic = Diagnostic::error(code, error.to_string());
        match span {
            Some(span) => diagnostic.with_span(span),
            None => diagnostic,
        }
    }
}

/// Orders diagnostics by position; those without a span go last. The sort is stable.
pub(crate) fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by_key(|d| d.span.map_or(u32::MAX, |s| s.start));
}

/// The number of single-character edits that turn `a` into `b` (Levenshtein distance).
pub(crate) fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = alloc::vec![0; b.len() + 1];
    for (i, ca) in a.chars().enumerate() {
        current[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let substitute = previous[j] + usize::from(ca != cb);
            current[j + 1] = substitute.min(previous[j + 1] + 1).min(current[j] + 1);
        }
        core::mem::swap(&mut previous, &mut current);
    }
    previous[b.len()]
}

/// The candidate closest to `name` within two edits, preferring earlier candidates on ties.
/// A candidate that would replace every character of `name` is no suggestion.
pub(crate) fn closest<'a>(
    name: &str,
    candidates: impl IntoIterator<Item = &'a str>,
) -> Option<&'a str> {
    let length = name.chars().count();
    candidates
        .into_iter()
        .map(|c| (edit_distance(name, c), c))
        .filter(|&(distance, _)| (1..=2).contains(&distance) && distance < length)
        .min_by_key(|&(distance, _)| distance)
        .map(|(_, c)| c)
}

/// A glyph for a message: in backticks, or escaped if it would be invisible.
pub(crate) fn show_glyph(glyph: char) -> String {
    if glyph.is_whitespace() || glyph.is_control() {
        format!("{glyph:?}")
    } else {
        format!("`{glyph}`")
    }
}

/// δ(q, a) or δ(q, (a, b)) for a message.
fn show_lhs(from: &str, read: &[char]) -> String {
    let symbols: Vec<String> = read.iter().map(char::to_string).collect();
    if let [single] = symbols.as_slice() {
        format!("δ({from}, {single})")
    } else {
        format!("δ({from}, ({}))", symbols.join(", "))
    }
}

/// "1 tape", "2 tapes".
fn tapes_noun(n: usize) -> String {
    if n == 1 {
        "1 tape".into()
    } else {
        format!("{n} tapes")
    }
}

/// How many "did you mean" searches one check may run. Each compares a name with every
/// state, so the budget keeps checking near linear for any input.
pub(crate) const MAX_SUGGESTIONS: usize = 64;

/// Lookup tables over an inferred spec, built once so that every check stays near linear.
struct Index<'a> {
    states: BTreeSet<&'a str>,
    gamma: BTreeSet<char>,
    sigma: BTreeSet<char>,
    /// For each state, the transition that mentions it first, and whether as its source.
    first_mention: BTreeMap<&'a str, (usize, bool)>,
    /// The first position of each glyph in Γ and in Σ.
    gamma_position: BTreeMap<char, usize>,
    sigma_position: BTreeMap<char, usize>,
    /// Where each state of F is written.
    finals: BTreeMap<&'a str, Span>,
}

impl<'a> Index<'a> {
    fn new(spec: &'a MachineSpec, map: Option<&'a SourceMap>) -> Self {
        let positions = |glyphs: &[char]| {
            let mut map = BTreeMap::new();
            for (i, &glyph) in glyphs.iter().enumerate() {
                map.entry(glyph).or_insert(i);
            }
            map
        };
        let mut first_mention = BTreeMap::new();
        for (i, t) in spec.transitions.iter().enumerate() {
            first_mention.entry(t.from.as_str()).or_insert((i, true));
            first_mention.entry(t.to.as_str()).or_insert((i, false));
        }
        Self {
            states: spec.states.iter().map(|s| s.name.as_str()).collect(),
            gamma: spec.tape_alphabet.iter().copied().collect(),
            sigma: spec.input_alphabet.iter().copied().collect(),
            first_mention,
            gamma_position: positions(&spec.tape_alphabet),
            sigma_position: positions(&spec.input_alphabet),
            finals: map
                .iter()
                .flat_map(|m| &m.finals)
                .map(|(name, span)| (name.as_str(), *span))
                .collect(),
        }
    }
}

/// Runs every check on an inferred spec.
struct Checker<'a> {
    spec: &'a MachineSpec,
    map: Option<&'a SourceMap>,
    index: Index<'a>,
    suggestions: usize,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Checker<'a> {
    fn new(spec: &'a MachineSpec, map: Option<&'a SourceMap>) -> Self {
        Self {
            spec,
            map,
            index: Index::new(spec, map),
            suggestions: MAX_SUGGESTIONS,
            diagnostics: Vec::new(),
        }
    }

    fn run(mut self) -> Vec<Diagnostic> {
        self.tapes();
        self.names();
        self.duplicates();
        self.start();
        self.alphabets();
        self.limits();
        self.transitions();
        self.input_word();
        self.unreachable_states();
        self.final_states();
        self.unused_symbols();
        self.diagnostics
    }

    fn push(&mut self, diagnostic: Diagnostic, span: Option<Span>) {
        self.diagnostics.push(match span {
            Some(span) => diagnostic.with_span(span),
            None => diagnostic,
        });
    }

    fn transition_spans(&self, index: usize) -> Option<&'a TransitionSpans> {
        self.map.and_then(|m| m.transitions.get(index))
    }

    /// The span of transition `index`, or of a part of it selected by `part`.
    fn part_span(
        &self,
        index: usize,
        part: impl Fn(&TransitionSpans) -> Option<Span>,
    ) -> Option<Span> {
        self.transition_spans(index)
            .and_then(part)
            .or_else(|| self.spec.transition_span(index))
    }

    /// The span covering the read symbols of transition `index` in the source.
    fn read_span(&self, index: usize) -> Option<Span> {
        let spans = &self.transition_spans(index)?.read;
        Some(spans.first()?.join(*spans.last()?))
    }

    fn state_decl_span(&self, index: usize) -> Option<Span> {
        if !self.spec.declared.states {
            return None;
        }
        self.map.and_then(|m| m.states.get(index).copied())
    }

    /// Where a state is mentioned first: its declaration, F or a transition.
    fn state_span(&self, index: usize) -> Option<Span> {
        let name = self.spec.states.get(index).map(|s| s.name.as_str())?;
        self.state_decl_span(index)
            .or_else(|| self.index.finals.get(name).copied())
            .or_else(|| {
                let &(i, is_from) = self.index.first_mention.get(name)?;
                self.part_span(i, |s| Some(if is_from { s.from } else { s.to }))
            })
    }

    /// Where a glyph of a declared Γ or Σ is written.
    fn glyph_decl_span(&self, glyph: char) -> Option<Span> {
        let spec = self.spec;
        let map = self.map?;
        let in_gamma = spec
            .declared
            .tape_alphabet
            .then(|| self.index.gamma_position.get(&glyph))
            .flatten()
            .and_then(|&i| map.tape_alphabet.get(i).copied());
        in_gamma.or_else(|| {
            spec.declared
                .input_alphabet
                .then(|| self.index.sigma_position.get(&glyph))
                .flatten()
                .and_then(|&i| map.input_alphabet.get(i).copied())
        })
    }

    fn tapes(&mut self) {
        let tapes = usize::from(self.spec.tapes);
        if !(1..=MAX_TAPES).contains(&tapes) {
            let d = Diagnostic::error(
                codes::TOO_LARGE,
                format!("a machine has 1 to {MAX_TAPES} tapes, not {tapes}"),
            );
            self.push(d, self.map.and_then(|m| m.tapes));
        }
    }

    fn names(&mut self) {
        let spec = self.spec;
        for (i, state) in spec.states.iter().enumerate() {
            if !is_state_name(&state.name) {
                let message = if state.name.is_empty() {
                    "a state has an empty name".into()
                } else {
                    format!("`{}` is not a valid state name", state.name)
                };
                let d = Diagnostic::error(codes::INVALID_NAME, message).with_help(
                    "a state name starts with a letter or `_` and continues with letters, \
                     digits, `_` and `'`",
                );
                self.push(d, self.state_span(i));
            }
        }
        if self.index.gamma.contains(&'\n') {
            let d = Diagnostic::error(codes::INVALID_NAME, "a line break cannot be a symbol");
            self.push(d, None);
        }
    }

    fn duplicates(&mut self) {
        let spec = self.spec;
        let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
        for (i, state) in spec.states.iter().enumerate() {
            if let Some(&first) = seen.get(state.name.as_str()) {
                let mut d = Diagnostic::error(
                    codes::DUPLICATE_DECLARATION,
                    format!("the state `{}` is listed twice in Q", state.name),
                );
                if let Some(first) = self.state_decl_span(first) {
                    d = d.with_related(first, "first listed here");
                }
                self.push(d, self.state_decl_span(i));
            } else {
                seen.insert(&state.name, i);
            }
        }
        let alphabets = [
            (
                "Σ",
                &spec.input_alphabet,
                spec.declared.input_alphabet,
                self.map.map(|m| &m.input_alphabet),
            ),
            (
                "Γ",
                &spec.tape_alphabet,
                spec.declared.tape_alphabet,
                self.map.map(|m| &m.tape_alphabet),
            ),
        ];
        for (set, glyphs, declared, spans) in alphabets {
            let span = |i: usize| spans.and_then(|s| s.get(i).copied()).filter(|_| declared);
            let mut seen: BTreeMap<char, usize> = BTreeMap::new();
            for (i, &glyph) in glyphs.iter().enumerate() {
                if let Some(&first) = seen.get(&glyph) {
                    let mut d = Diagnostic::error(
                        codes::DUPLICATE_DECLARATION,
                        format!("the symbol {} is listed twice in {set}", show_glyph(glyph)),
                    );
                    if let Some(first) = span(first) {
                        d = d.with_related(first, "first listed here");
                    }
                    self.push(d, span(i));
                } else {
                    seen.insert(glyph, i);
                }
            }
        }
    }

    fn start(&mut self) {
        let spec = self.spec;
        let map_start = self.map.and_then(|m| m.start);
        if spec.states.is_empty() {
            let d = Diagnostic::error(codes::NO_STATES, "the machine has no states").with_help(
                "write a transition such as `δ(q0, ␣) = (q1, ␣, R)`, or declare `Q = {q0}`",
            );
            self.push(d, self.map.and_then(|m| m.states_decl));
        } else if spec.start.is_empty() {
            let d = Diagnostic::error(codes::NO_STATES, "the machine has no start state")
                .with_help("declare one with `start = ...`");
            self.push(d, map_start);
        } else if !self.index.states.contains(spec.start.as_str()) {
            let d = self.unknown_state(&spec.start);
            self.push(d, map_start);
        }
    }

    fn unknown_state(&mut self, name: &str) -> Diagnostic {
        let d = Diagnostic::error(
            codes::UNKNOWN_NAME,
            format!("the state `{name}` is not declared in Q"),
        );
        let suggestion = if self.suggestions > 0 {
            self.suggestions -= 1;
            closest(name, self.spec.states.iter().map(|s| s.name.as_str()))
        } else {
            None
        };
        match suggestion {
            Some(candidate) => d.with_help(format!("did you mean `{candidate}`?")),
            None => d.with_help(format!("add `{name}` to Q")),
        }
    }

    fn unknown_symbol(&self, glyph: char) -> Diagnostic {
        let d = Diagnostic::error(
            codes::UNKNOWN_NAME,
            format!(
                "the symbol {} is not in the tape alphabet Γ",
                show_glyph(glyph)
            ),
        );
        // Γ has at most a few dozen symbols when this matters; beyond the limit, E009 applies.
        let same_letter = self
            .spec
            .tape_alphabet
            .iter()
            .take(MAX_SYMBOLS)
            .find(|&&c| c != glyph && c.to_lowercase().eq(glyph.to_lowercase()));
        match same_letter {
            Some(&candidate) => d.with_help(format!("did you mean {}?", show_glyph(candidate))),
            None => d.with_help(format!("add {} to Γ", show_glyph(glyph))),
        }
    }

    fn alphabets(&mut self) {
        let spec = self.spec;
        if let Some(&i) = self.index.sigma_position.get(&spec.blank) {
            let d = Diagnostic::error(
                codes::BLANK_IN_INPUT,
                format!(
                    "the blank {} cannot be part of the input alphabet Σ",
                    show_glyph(spec.blank)
                ),
            )
            .with_help("Σ holds the symbols of input words; the blank belongs to Γ only");
            self.push(d, self.map.and_then(|m| m.input_alphabet.get(i).copied()));
        }
        let mut reported = BTreeSet::new();
        for (i, &glyph) in spec.input_alphabet.iter().enumerate() {
            if glyph != spec.blank && !self.index.gamma.contains(&glyph) && reported.insert(glyph) {
                let d = Diagnostic::error(
                    codes::INPUT_NOT_IN_TAPE,
                    format!(
                        "the input symbol {} is not in the tape alphabet Γ",
                        show_glyph(glyph)
                    ),
                )
                .with_help(format!(
                    "Σ must be a subset of Γ: add {} to Γ",
                    show_glyph(glyph)
                ));
                let span = self.map.and_then(|m| m.input_alphabet.get(i).copied());
                self.push(d, span);
            }
        }
    }

    fn limits(&mut self) {
        let states = self.index.states.len();
        let symbols = self.index.gamma.len();
        let declaration = self.map.and_then(|m| m.states_decl);
        if states > MAX_STATES {
            let d = Diagnostic::error(
                codes::TOO_LARGE,
                format!("the machine has {states} states, but at most {MAX_STATES} are supported"),
            );
            self.push(d, declaration);
        }
        if symbols > MAX_SYMBOLS {
            let d = Diagnostic::error(
                codes::TOO_LARGE,
                format!(
                    "the tape alphabet has {symbols} symbols, but at most {MAX_SYMBOLS} are \
                     supported"
                ),
            );
            self.push(d, None);
        }
        let tapes = usize::from(self.spec.tapes);
        if states <= MAX_STATES && symbols <= MAX_SYMBOLS && (1..=MAX_TAPES).contains(&tapes) {
            let entries = u32::try_from(tapes)
                .ok()
                .and_then(|k| symbols.checked_pow(k))
                .and_then(|columns| columns.checked_mul(states));
            if entries.is_none_or(|n| n > MAX_TABLE_ENTRIES) {
                let d = Diagnostic::error(
                    codes::TOO_LARGE,
                    format!(
                        "the transition table would have more than {MAX_TABLE_ENTRIES} entries \
                         (|Q| × |Γ|^k with {states} states, {symbols} symbols and {})",
                        tapes_noun(tapes)
                    ),
                )
                .with_help("use fewer states or symbols");
                self.push(d, None);
            }
        }
    }

    fn transitions(&mut self) {
        let spec = self.spec;
        let tapes = usize::from(spec.tapes);
        let mut first: BTreeMap<(&str, &[char]), usize> = BTreeMap::new();
        for (i, t) in spec.transitions.iter().enumerate() {
            self.arity(i, t, tapes);
            if !self.index.states.contains(t.from.as_str()) {
                let d = self.unknown_state(&t.from);
                self.push(d, self.part_span(i, |s| Some(s.from)));
            }
            if t.to != t.from && !self.index.states.contains(t.to.as_str()) {
                let d = self.unknown_state(&t.to);
                self.push(d, self.part_span(i, |s| Some(s.to)));
            }
            let mut reported = BTreeSet::new();
            let reads = t.read.iter().enumerate().map(|(k, &c)| (c, k, true));
            let writes = t.write.iter().enumerate().map(|(k, &c)| (c, k, false));
            for (glyph, k, is_read) in reads.chain(writes) {
                if !self.index.gamma.contains(&glyph) && reported.insert(glyph) {
                    let d = self.unknown_symbol(glyph);
                    let span = self.part_span(i, |s| {
                        if is_read {
                            s.read.get(k)
                        } else {
                            s.write.get(k)
                        }
                        .copied()
                    });
                    self.push(d, span);
                }
            }
            match first.get(&(t.from.as_str(), t.read.as_slice())) {
                Some(&j) => {
                    let mut d = Diagnostic::error(
                        codes::DUPLICATE_TRANSITION,
                        format!("{} is defined twice", show_lhs(&t.from, &t.read)),
                    )
                    .with_help(
                        "a deterministic machine has at most one transition per state and symbol",
                    );
                    let (mut span, mut first_span) =
                        (spec.transition_span(i), spec.transition_span(j));
                    if span == first_span {
                        // Both come from one line with a read set: point at the symbols.
                        span = self.read_span(i).or(span);
                        first_span = self.read_span(j).or(first_span);
                    }
                    if let Some(first_span) = first_span {
                        d = d.with_related(first_span, "first defined here");
                    }
                    self.push(d, span);
                }
                None => {
                    first.insert((t.from.as_str(), t.read.as_slice()), i);
                }
            }
        }
    }

    fn arity(&mut self, index: usize, t: &TransitionSpec, tapes: usize) {
        let parts = [
            ("reads", "symbols", t.read.len()),
            ("writes", "symbols", t.write.len()),
            ("has", "moves", t.dir.len()),
        ];
        let Some(&(verb, noun, found)) = parts.iter().find(|&&(_, _, n)| n != tapes) else {
            return;
        };
        if !(1..=MAX_TAPES).contains(&tapes) {
            return;
        }
        let noun = if found == 1 {
            &noun[..noun.len() - 1]
        } else {
            noun
        };
        let mut d = Diagnostic::error(
            codes::WRONG_ARITY,
            format!(
                "the transition {verb} {found} {noun}, but the machine has {}",
                tapes_noun(tapes)
            ),
        );
        let uniform = t.read.len() == found && t.write.len() == found && t.dir.len() == found;
        if uniform && (2..=MAX_TAPES).contains(&found) && tapes == 1 {
            d = d.with_help(format!(
                "declare `tapes = {found}` for a machine with {found} tapes"
            ));
        } else if tapes > 1 {
            d = d.with_help(format!(
                "write one entry per tape, as in `δ(q, ({})) = (p, ({}), ({}))`",
                ["a"; MAX_TAPES][..tapes].join(", "),
                ["b"; MAX_TAPES][..tapes].join(", "),
                ["R"; MAX_TAPES][..tapes].join(", "),
            ));
        }
        self.push(d, self.spec.transition_span(index));
    }

    fn input_word(&mut self) {
        let spec = self.spec;
        let mut reported = BTreeSet::new();
        for glyph in spec.input.chars() {
            if glyph != spec.blank && !self.index.sigma.contains(&glyph) && reported.insert(glyph) {
                let set = if spec.declared.input_alphabet {
                    "Σ"
                } else {
                    "Γ"
                };
                let d = Diagnostic::error(
                    codes::UNKNOWN_NAME,
                    format!(
                        "the input word contains {}, which is not in the input alphabet Σ",
                        show_glyph(glyph)
                    ),
                )
                .with_help(format!("add {} to {set}", show_glyph(glyph)));
                self.push(d, self.map.and_then(|m| m.input));
            }
        }
    }

    fn unreachable_states(&mut self) {
        let spec = self.spec;
        if !self.index.states.contains(spec.start.as_str()) {
            return;
        }
        let mut successors: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for t in &spec.transitions {
            successors.entry(&t.from).or_default().push(&t.to);
        }
        let mut reached = BTreeSet::from([spec.start.as_str()]);
        let mut stack = alloc::vec![spec.start.as_str()];
        while let Some(q) = stack.pop() {
            for &next in successors.get(q).into_iter().flatten() {
                if reached.insert(next) {
                    stack.push(next);
                }
            }
        }
        let mut reported = BTreeSet::new();
        for (i, state) in spec.states.iter().enumerate() {
            if !reached.contains(state.name.as_str()) && reported.insert(state.name.as_str()) {
                let d = Diagnostic::warning(
                    codes::UNREACHABLE_STATE,
                    format!(
                        "the state `{}` cannot be reached from the start state `{}`",
                        state.name, spec.start
                    ),
                )
                .with_help("no chain of transitions leads from the start state to it");
                self.push(d, self.state_span(i));
            }
        }
    }

    fn final_states(&mut self) {
        let spec = self.spec;
        if spec.states.is_empty() || spec.states.iter().any(|s| s.is_final) {
            return;
        }
        let consequence = match spec.kind {
            Kind::Acceptor => "the acceptor has no final state, so it accepts no word",
            Kind::Enumerator => "the enumerator has no final state, so it outputs no word",
            Kind::Transducer | Kind::Beaver => return,
        };
        let d = Diagnostic::warning(codes::NO_FINAL_STATE, consequence)
            .with_help("declare the final states with `F = {...}`");
        self.push(d, self.map.and_then(|m| m.kind));
    }

    fn unused_symbols(&mut self) {
        let spec = self.spec;
        let used: BTreeSet<char> = spec
            .transitions
            .iter()
            .flat_map(|t| t.read.iter().chain(&t.write).copied())
            .collect();
        let declared_gamma = spec.declared.tape_alphabet.then_some(&spec.tape_alphabet);
        let declared_sigma = spec.declared.input_alphabet.then_some(&spec.input_alphabet);
        let mut reported = BTreeSet::new();
        for &glyph in declared_gamma.into_iter().chain(declared_sigma).flatten() {
            // A glyph outside Γ already has an error (E007).
            let in_gamma = self.index.gamma.contains(&glyph);
            if glyph != spec.blank && in_gamma && !used.contains(&glyph) && reported.insert(glyph) {
                let d = Diagnostic::warning(
                    codes::UNUSED_SYMBOL,
                    format!("the symbol {} is never read or written", show_glyph(glyph)),
                )
                .with_help("remove it from the alphabet, or add the transitions that use it");
                self.push(d, self.glyph_decl_span(glyph));
            }
        }
    }
}

impl From<&Machine> for MachineSpec {
    /// The canonical spec of a machine: every set declared, transitions by state and then by
    /// read tuple (in symbol order, first tape first), no comments.
    ///
    /// ```
    /// use turing_core::{MachineSpec, standard};
    /// let machine = standard::parse_standard("1RB1LB_1LA1RZ").unwrap();
    /// let spec = MachineSpec::from(&machine);
    /// assert_eq!(spec.states.len(), 3);
    /// assert_eq!(spec.compile().unwrap(), machine);
    /// ```
    fn from(machine: &Machine) -> Self {
        let alphabet = machine.alphabet();
        let tapes = machine.tapes();
        let name = |q: crate::State| machine.state(q).name.clone();
        let mut transitions: Vec<_> = machine.transitions().collect();
        transitions.sort_by(|a, b| {
            a.from
                .cmp(&b.from)
                .then_with(|| a.read[..tapes].cmp(&b.read[..tapes]))
        });
        let transitions = transitions
            .into_iter()
            .map(|t| TransitionSpec {
                from: name(t.from),
                read: t.read[..tapes].iter().map(|&s| alphabet.glyph(s)).collect(),
                to: name(t.action.next),
                write: t.action.write[..tapes]
                    .iter()
                    .map(|&s| alphabet.glyph(s))
                    .collect(),
                dir: t.action.dir[..tapes].to_vec(),
                ..TransitionSpec::default()
            })
            .collect();
        Self {
            name: machine.name().into(),
            kind: machine.kind(),
            tapes: u8::try_from(tapes).unwrap_or(u8::MAX),
            blank: alphabet.blank(),
            input_alphabet: alphabet.input_glyphs().collect(),
            tape_alphabet: alphabet.glyphs().to_vec(),
            states: machine
                .states()
                .iter()
                .map(|s| StateSpec {
                    name: s.name.clone(),
                    is_final: s.is_final,
                })
                .collect(),
            start: name(machine.start()),
            transitions,
            input: machine.default_input().into(),
            output: machine.decoder(),
            declared: Declared {
                states: true,
                input_alphabet: true,
                tape_alphabet: true,
                start: true,
                blank: alphabet.blank() != '␣',
            },
            comments: SpecComments::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_distances() {
        assert_eq!(edit_distance("", ""), 0);
        assert_eq!(edit_distance("q1", "q1"), 0);
        assert_eq!(edit_distance("q1", "q2"), 1);
        assert_eq!(edit_distance("start", "strat"), 2);
        assert_eq!(edit_distance("δ", "d"), 1);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(closest("q9", ["a", "q1", "q2"]), Some("q1"));
        assert_eq!(closest("q1", ["q1"]), None);
        assert_eq!(closest("abcdef", ["q1"]), None);
    }

    #[test]
    fn infer_is_idempotent() {
        let mut spec = MachineSpec::new();
        spec.input = "ab".into();
        spec.states.push(StateSpec {
            name: "qf".into(),
            is_final: true,
        });
        spec.transitions.push(TransitionSpec {
            from: "q0".into(),
            read: alloc::vec!['a'],
            to: "q1".into(),
            write: alloc::vec!['X'],
            dir: alloc::vec![Dir::R],
            ..TransitionSpec::default()
        });
        spec.infer();
        let names: Vec<_> = spec.states.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["q0", "q1", "qf"]);
        assert!(spec.states[2].is_final);
        assert_eq!(spec.tape_alphabet, ['␣', 'a', 'X', 'b']);
        assert_eq!(spec.input_alphabet, ['a', 'X', 'b']);
        let again = {
            let mut s = spec.clone();
            s.infer();
            s
        };
        assert_eq!(again, spec);
    }
}
