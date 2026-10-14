//! Recursive-descent parser with recovery per line.
//!
//! Parsing runs in two stages. [`Parser`] reads one line at a time into a [`Line`]; a syntax
//! error is reported and the rest of the line is skipped, so every line gets its own chance.
//! [`Assembler`] then turns the lines into a [`MachineSpec`]: it resolves the blank, expands
//! read sets and `*`, attaches comments and records the span of every token in a
//! [`SourceMap`], so that the checks of [`MachineSpec::analyze`] can point at single tokens.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use super::Parsed;
use super::lexer::{self, LexError, Token, TokenKind};
use crate::diag::{Diagnostic, Severity, Span};
use crate::machine::{BitOrder, Decoder, Kind};
use crate::spec::{
    MAX_SUGGESTIONS, MachineSpec, SourceMap, StateSpec, TransitionSpans, TransitionSpec, closest,
    codes, edit_distance, sort_diagnostics,
};
use crate::symbol::{Dir, MAX_TAPES};

/// The most transitions a description may expand to. It is the size of one fully expanded
/// line with three tapes and 64 symbols, and keeps memory bounded for hostile input.
const MAX_TRANSITIONS: usize = 1 << 18;

/// The longer words that can start a line, for "did you mean" suggestions.
const KEYWORDS: [&str; 10] = [
    "delta", "machine", "kind", "input", "output", "tapes", "Sigma", "Gamma", "blank", "start",
];

pub(super) fn parse(source: &str) -> Parsed {
    let tokens = lexer::tokenize(source);
    let mut parser = Parser {
        tokens: &tokens,
        pos: 0,
        diagnostics: Vec::new(),
    };
    let mut lines = Vec::new();
    while parser.pos < tokens.len() {
        lines.push(parser.line());
    }
    Assembler::new(parser.diagnostics).assemble(&lines)
}

/// A value with the span it was written at.
#[derive(Clone, Debug)]
struct Located<T> {
    value: T,
    span: Span,
}

/// A symbol as written: the blank (`_` or `␣`) or a glyph.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum RawSymbol {
    Blank,
    Glyph(char),
}

impl RawSymbol {
    fn resolve(self, blank: char) -> char {
        match self {
            Self::Blank => blank,
            Self::Glyph(c) => c,
        }
    }
}

/// An `output` declaration as written.
#[derive(Copy, Clone, Debug)]
enum RawDecoder {
    Word,
    Unary(RawSymbol),
    Binary(BitOrder),
}

/// A transition as written, before read sets and `*` are expanded.
#[derive(Clone, Debug)]
struct RawTransition {
    from: Located<String>,
    /// For each tape, the symbols of the read set (one for a plain symbol).
    read: Vec<Vec<Located<RawSymbol>>>,
    to: Located<String>,
    /// For each tape, the symbol written; `None` for `*`.
    write: Vec<Located<Option<RawSymbol>>>,
    dir: Vec<Located<Dir>>,
}

/// One item: a declaration or a transition.
#[derive(Clone, Debug)]
enum Item {
    Machine(String),
    Kind(Kind),
    Input(String),
    Output(RawDecoder),
    Tapes(u64),
    States(Vec<Located<String>>),
    InputAlphabet(Vec<Located<RawSymbol>>),
    TapeAlphabet(Vec<Located<RawSymbol>>),
    Blank(RawSymbol),
    Start(Located<String>),
    Final(Vec<Located<String>>),
    Transition(RawTransition),
}

impl Item {
    /// The name of a declaration as written in messages, or `None` for a transition.
    fn declaration(&self) -> Option<&'static str> {
        Some(match self {
            Self::Machine(_) => "machine",
            Self::Kind(_) => "kind",
            Self::Input(_) => "input",
            Self::Output(..) => "output",
            Self::Tapes(_) => "tapes",
            Self::States(_) => "Q",
            Self::InputAlphabet(_) => "Σ",
            Self::TapeAlphabet(_) => "Γ",
            Self::Blank(_) => "blank",
            Self::Start(_) => "start",
            Self::Final(_) => "F",
            Self::Transition(_) => return None,
        })
    }
}

/// One source line.
#[derive(Clone, Debug)]
enum Line {
    /// Only whitespace.
    Blank,
    /// Only a comment.
    Comment(String),
    /// An item, possibly followed by a comment.
    Item {
        item: Item,
        span: Span,
        value_span: Span,
        comment: Option<String>,
    },
    /// A line with a syntax error, already reported.
    Error,
}

/// The text of a comment without the one space that usually follows `//`.
fn comment_text(raw: &str) -> String {
    raw.strip_prefix(' ').unwrap_or(raw).to_string()
}

/// How a token is named in "expected ..., found ..." messages.
fn describe(kind: &TokenKind<'_>) -> String {
    match kind {
        TokenKind::Word(w) | TokenKind::Number(w) => format!("`{w}`"),
        TokenKind::Quoted(c) => format!("`'{c}'`"),
        TokenKind::Str(_) => "a string".into(),
        TokenKind::LParen => "`(`".into(),
        TokenKind::RParen => "`)`".into(),
        TokenKind::LBrace => "`{`".into(),
        TokenKind::RBrace => "`}`".into(),
        TokenKind::Comma => "`,`".into(),
        TokenKind::Eq => "`=`".into(),
        TokenKind::Star => "`*`".into(),
        TokenKind::Other(c) => format!("`{c}`"),
        TokenKind::Comment(_) => "a comment".into(),
        TokenKind::Newline => "the end of the line".into(),
        TokenKind::Error(LexError::UnterminatedString) => "an unterminated string".into(),
        TokenKind::Error(LexError::UnterminatedQuote) => "an unterminated quote".into(),
    }
}

/// Reads lines of tokens. Every method that returns `Err(())` has reported the error.
struct Parser<'t, 'a> {
    tokens: &'t [Token<'a>],
    pos: usize,
    diagnostics: Vec<Diagnostic>,
}

type Result<T> = core::result::Result<T, ()>;

impl<'a> Parser<'_, 'a> {
    fn peek(&self) -> &Token<'a> {
        // The token list always ends with a newline, and `line` never moves past it.
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn peek_kind(&self) -> &TokenKind<'a> {
        &self.peek().kind
    }

    /// The token after the current one.
    fn peek_second(&self) -> Option<&TokenKind<'a>> {
        self.tokens.get(self.pos + 1).map(|t| &t.kind)
    }

    fn bump(&mut self) -> Token<'a> {
        let token = self.peek().clone();
        if token.kind != TokenKind::Newline {
            self.pos += 1;
        }
        token
    }

    /// The end of the token before the current one.
    fn previous_end(&self) -> u32 {
        self.pos
            .checked_sub(1)
            .and_then(|i| self.tokens.get(i))
            .map_or(0, |t| t.span.end)
    }

    /// Skips to the start of the next line.
    fn skip_line(&mut self) {
        while self.pos < self.tokens.len() && self.tokens[self.pos].kind != TokenKind::Newline {
            self.pos += 1;
        }
        self.pos += 1;
    }

    /// Reports `diagnostic` and fails.
    fn fail<T>(&mut self, diagnostic: Diagnostic) -> Result<T> {
        self.diagnostics.push(diagnostic);
        Err(())
    }

    /// The error for a current token that is not what `expected` describes.
    fn unexpected(&self, expected: &str) -> Diagnostic {
        let token = self.peek();
        let diagnostic = match token.kind {
            TokenKind::Error(LexError::UnterminatedString) => {
                Diagnostic::error(codes::UNTERMINATED_STRING, "the string is not terminated")
                    .with_help("add a closing `\"` before the end of the line")
            }
            TokenKind::Error(LexError::UnterminatedQuote) => Diagnostic::error(
                codes::UNEXPECTED_TOKEN,
                format!("expected {expected}, found an unterminated quote"),
            )
            .with_help("a quoted symbol is one character between two quotes, as in `'*'`"),
            ref other => Diagnostic::error(
                codes::UNEXPECTED_TOKEN,
                format!("expected {expected}, found {}", describe(other)),
            ),
        };
        diagnostic.with_span(token.span)
    }

    fn expect(&mut self, kind: &TokenKind<'_>, expected: &str) -> Result<Span> {
        if self.peek_kind() == kind {
            Ok(self.bump().span)
        } else {
            self.fail(self.unexpected(expected))
        }
    }

    fn line(&mut self) -> Line {
        match self.peek_kind() {
            TokenKind::Newline => {
                self.skip_line();
                return Line::Blank;
            }
            &TokenKind::Comment(text) if self.peek_second() == Some(&TokenKind::Newline) => {
                self.skip_line();
                return Line::Comment(comment_text(text));
            }
            _ => {}
        }
        let start = self.peek().span.start;
        let Ok((item, value_span)) = self.item() else {
            self.skip_line();
            return Line::Error;
        };
        let span = Span::new(start, self.previous_end());
        let comment = match *self.peek_kind() {
            TokenKind::Comment(text) => {
                self.bump();
                Some(comment_text(text))
            }
            _ => None,
        };
        if *self.peek_kind() != TokenKind::Newline {
            self.diagnostics
                .push(self.unexpected("the end of the line"));
            self.skip_line();
            return Line::Error;
        }
        self.skip_line();
        Line::Item {
            item,
            span,
            value_span,
            comment,
        }
    }

    /// One item and the span of its value (after the keyword and `=`).
    fn item(&mut self) -> Result<(Item, Span)> {
        let token = self.peek().clone();
        let TokenKind::Word(word) = token.kind else {
            return self.fail(self.unexpected("a declaration or a transition"));
        };
        let declaration = match word {
            "machine" | "kind" | "input" | "output" => false,
            "tapes" | "Q" | "Σ" | "Sigma" | "Γ" | "Gamma" | "blank" | "start" | "F" => true,
            "δ" | "delta" => {
                self.bump();
                let item = self.transition()?;
                return Ok((item, Span::new(token.span.start, self.previous_end())));
            }
            _ => return self.fail(self.unknown_line_start(word, token.span)),
        };
        self.bump();
        if declaration {
            self.expect(&TokenKind::Eq, "`=`")?;
        }
        let start = self.peek().span.start;
        let item = match word {
            "machine" => Item::Machine(self.string()?),
            "kind" => Item::Kind(self.kind()?),
            "input" => Item::Input(self.string()?),
            "output" => self.output()?,
            "tapes" => Item::Tapes(self.number()?),
            "Q" => Item::States(self.state_set()?),
            "Σ" | "Sigma" => Item::InputAlphabet(self.symbol_set()?),
            "Γ" | "Gamma" => Item::TapeAlphabet(self.symbol_set()?),
            "blank" => Item::Blank(self.symbol()?.value),
            "start" => Item::Start(self.state()?),
            _ => Item::Final(self.state_set()?),
        };
        Ok((item, Span::new(start, self.previous_end())))
    }

    fn unknown_line_start(&self, word: &str, span: Span) -> Diagnostic {
        let diagnostic = Diagnostic::error(
            codes::UNEXPECTED_TOKEN,
            format!("expected a declaration or a transition, found `{word}`"),
        )
        .with_span(span);
        let length = word.chars().count();
        let suggestion = KEYWORDS.iter().find(|k| {
            let distance = edit_distance(word, k);
            distance <= 2 && distance < length
        });
        if self.peek_second() == Some(&TokenKind::LParen) {
            diagnostic
                .with_help("a transition starts with `δ` or `delta`, as in `δ(q0, a) = (q1, b, R)`")
        } else if let Some(keyword) = suggestion {
            diagnostic.with_help(format!("did you mean `{keyword}`?"))
        } else {
            diagnostic.with_help(
                "each line holds one declaration, such as `Q = {q0, q1}`, or one transition, \
                 such as `δ(q0, a) = (q1, b, R)`",
            )
        }
    }

    fn string(&mut self) -> Result<String> {
        if let TokenKind::Str(value) = self.peek_kind() {
            let value = value.clone();
            self.bump();
            Ok(value)
        } else {
            self.fail(self.unexpected("a string in double quotes"))
        }
    }

    fn number(&mut self) -> Result<u64> {
        if let TokenKind::Number(digits) = *self.peek_kind() {
            self.bump();
            // A number too large for u64 is certainly too many tapes.
            Ok(digits.parse().unwrap_or(u64::MAX))
        } else {
            self.fail(self.unexpected("a number"))
        }
    }

    /// A word from `choices`, with a suggestion if it is close to one.
    fn choice(&mut self, choices: &[&'static str]) -> Result<&'static str> {
        let expected = match choices {
            [single] => format!("`{single}`"),
            [init @ .., last] => {
                let init: Vec<String> = init.iter().map(|c| format!("`{c}`")).collect();
                format!("{} or `{last}`", init.join(", "))
            }
            [] => String::new(),
        };
        let token = self.peek().clone();
        if let TokenKind::Word(word) = token.kind {
            if let Some(&choice) = choices.iter().find(|&&c| c == word) {
                self.bump();
                return Ok(choice);
            }
            let mut diagnostic = Diagnostic::error(
                codes::UNEXPECTED_TOKEN,
                format!("expected {expected}, found `{word}`"),
            )
            .with_span(token.span);
            if let Some(candidate) = closest(word, choices.iter().copied()) {
                diagnostic = diagnostic.with_help(format!("did you mean `{candidate}`?"));
            }
            return self.fail(diagnostic);
        }
        self.fail(self.unexpected(&expected))
    }

    fn kind(&mut self) -> Result<Kind> {
        Ok(
            match self.choice(&["acceptor", "transducer", "enumerator", "beaver"])? {
                "acceptor" => Kind::Acceptor,
                "enumerator" => Kind::Enumerator,
                "beaver" => Kind::Beaver,
                _ => Kind::Transducer,
            },
        )
    }

    fn output(&mut self) -> Result<Item> {
        Ok(Item::Output(
            match self.choice(&["word", "unary", "binary"])? {
                "unary" => RawDecoder::Unary(self.symbol()?.value),
                "binary" => RawDecoder::Binary(match self.choice(&["lsb", "msb"])? {
                    "lsb" => BitOrder::Lsb,
                    _ => BitOrder::Msb,
                }),
                _ => RawDecoder::Word,
            },
        ))
    }

    fn state(&mut self) -> Result<Located<String>> {
        let token = self.peek().clone();
        match token.kind {
            TokenKind::Word("_") => self.fail(
                Diagnostic::error(
                    codes::UNEXPECTED_TOKEN,
                    "expected a state, found the blank symbol `_`",
                )
                .with_span(token.span)
                .with_help("a lone `_` is the blank; a state name such as `q_0` may contain `_`"),
            ),
            TokenKind::Word(name) => {
                self.bump();
                Ok(Located {
                    value: name.to_string(),
                    span: token.span,
                })
            }
            TokenKind::Number(_) => self.fail(
                self.unexpected("a state")
                    .with_help("a state name starts with a letter or `_`, as in `q0`"),
            ),
            _ => self.fail(self.unexpected("a state")),
        }
    }

    fn symbol(&mut self) -> Result<Located<RawSymbol>> {
        let token = self.peek().clone();
        let single = |text: &str| {
            let mut chars = text.chars();
            chars.next().filter(|_| chars.next().is_none())
        };
        let value = match token.kind {
            TokenKind::Word("_") | TokenKind::Other('␣') => Some(RawSymbol::Blank),
            TokenKind::Word(text) | TokenKind::Number(text) => single(text).map(RawSymbol::Glyph),
            TokenKind::Quoted(c) => Some(RawSymbol::Glyph(c)),
            TokenKind::Other(c) if c != ';' => Some(RawSymbol::Glyph(c)),
            _ => None,
        };
        let Some(value) = value else {
            let help: Option<String> = match token.kind {
                TokenKind::Word(_) | TokenKind::Number(_) => {
                    Some("a symbol is a single character, such as `x`, `#` or `'*'`".into())
                }
                TokenKind::Star => Some(
                    "`*` writes back the symbol read; write `'*'` for the symbol itself".into(),
                ),
                TokenKind::Other(c) => Some(format!("write `'{c}'` to use `{c}` as a symbol")),
                TokenKind::Str(_) => Some("write `'\"'` for the double quote symbol".into()),
                _ => None,
            };
            let diagnostic = self.unexpected("a symbol");
            return self.fail(match help {
                Some(help) => diagnostic.with_help(help),
                None => diagnostic,
            });
        };
        self.bump();
        Ok(Located {
            value,
            span: token.span,
        })
    }

    fn direction(&mut self) -> Result<Located<Dir>> {
        let token = self.peek().clone();
        let text = match token.kind {
            TokenKind::Word(text) | TokenKind::Number(text) => text.to_string(),
            TokenKind::Other(c) | TokenKind::Quoted(c) => c.to_string(),
            TokenKind::Star => "*".into(),
            _ => return self.fail(self.unexpected("a direction (`L`, `R` or `N`)")),
        };
        let mut chars = text.chars();
        let dir = chars
            .next()
            .filter(|_| chars.next().is_none())
            .and_then(Dir::from_char);
        if let Some(dir) = dir {
            self.bump();
            return Ok(Located {
                value: dir,
                span: token.span,
            });
        }
        let help = if matches!(text.as_str(), "l" | "r" | "n") {
            "directions are upper case: use L, R or N"
        } else {
            "use L, R or N"
        };
        self.fail(
            Diagnostic::error(
                codes::INVALID_DIRECTION,
                format!("`{text}` is not a direction"),
            )
            .with_span(token.span)
            .with_help(help),
        )
    }

    /// `{` items separated by `,` `}`; the set may be empty if `allow_empty`.
    fn set<T>(
        &mut self,
        allow_empty: bool,
        mut element: impl FnMut(&mut Self) -> Result<T>,
    ) -> Result<Vec<T>> {
        self.expect(&TokenKind::LBrace, "`{`")?;
        let mut items = Vec::new();
        if allow_empty && *self.peek_kind() == TokenKind::RBrace {
            self.bump();
            return Ok(items);
        }
        loop {
            items.push(element(self)?);
            match self.peek_kind() {
                TokenKind::Comma => {
                    self.bump();
                }
                TokenKind::RBrace => {
                    self.bump();
                    return Ok(items);
                }
                _ => return self.fail(self.unexpected("`,` or `}`")),
            }
        }
    }

    /// One element, or `(` elements separated by `,` `)` for several tapes.
    fn tuple<T>(&mut self, mut element: impl FnMut(&mut Self) -> Result<T>) -> Result<Vec<T>> {
        if *self.peek_kind() != TokenKind::LParen {
            return Ok(vec![element(self)?]);
        }
        self.bump();
        let mut items = Vec::new();
        loop {
            items.push(element(self)?);
            match self.peek_kind() {
                TokenKind::Comma => {
                    self.bump();
                }
                TokenKind::RParen => {
                    self.bump();
                    return Ok(items);
                }
                _ => return self.fail(self.unexpected("`,` or `)`")),
            }
        }
    }

    fn state_set(&mut self) -> Result<Vec<Located<String>>> {
        self.set(true, Self::state)
    }

    fn symbol_set(&mut self) -> Result<Vec<Located<RawSymbol>>> {
        self.set(true, Self::symbol)
    }

    /// A read entry: a symbol or a non-empty set of symbols.
    fn read_entry(&mut self) -> Result<Vec<Located<RawSymbol>>> {
        if *self.peek_kind() == TokenKind::LBrace {
            self.set(false, Self::symbol)
        } else {
            Ok(vec![self.symbol()?])
        }
    }

    /// A write entry: a symbol, or `*` for the symbol read.
    fn write_entry(&mut self) -> Result<Located<Option<RawSymbol>>> {
        if *self.peek_kind() == TokenKind::Star {
            let span = self.bump().span;
            return Ok(Located { value: None, span });
        }
        let symbol = self.symbol()?;
        Ok(Located {
            value: Some(symbol.value),
            span: symbol.span,
        })
    }

    /// Everything after `δ`: `(q, read) = (p, write, dir)`.
    fn transition(&mut self) -> Result<Item> {
        self.expect(&TokenKind::LParen, "`(`")?;
        let from = self.state()?;
        self.expect(&TokenKind::Comma, "`,`")?;
        let read = self.tuple(Self::read_entry)?;
        self.expect(&TokenKind::RParen, "`)`")?;
        self.expect(&TokenKind::Eq, "`=`")?;
        self.expect(&TokenKind::LParen, "`(`")?;
        let to = self.state()?;
        self.expect(&TokenKind::Comma, "`,`")?;
        let write = self.tuple(Self::write_entry)?;
        self.expect(&TokenKind::Comma, "`,`")?;
        let dir = self.tuple(Self::direction)?;
        self.expect(&TokenKind::RParen, "`)`")?;
        Ok(Item::Transition(RawTransition {
            from,
            read,
            to,
            write,
            dir,
        }))
    }
}

/// Turns parsed lines into a spec.
struct Assembler {
    diagnostics: Vec<Diagnostic>,
    /// Whether some part of the text could not be used, so that the spec is incomplete.
    incomplete: bool,
    /// Whether transitions were dropped because there were too many.
    truncated: bool,
    spec: MachineSpec,
    map: SourceMap,
}

impl Assembler {
    fn new(diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            incomplete: !diagnostics.is_empty(),
            truncated: false,
            diagnostics,
            spec: MachineSpec::new(),
            map: SourceMap::default(),
        }
    }

    fn assemble(mut self, lines: &[Line]) -> Parsed {
        self.spec.tape_alphabet.clear();
        self.declarations(lines);
        self.transitions_and_comments(lines);

        let (spec, checks) = self.spec.analyze(Some(&self.map));
        let incomplete = self.incomplete;
        // Warnings and "no states" are unreliable when lines were dropped.
        self.diagnostics.extend(checks.into_iter().filter(|d| {
            !incomplete || (d.severity == Severity::Error && d.code != codes::NO_STATES)
        }));
        sort_diagnostics(&mut self.diagnostics);
        let ok = !self.diagnostics.iter().any(Diagnostic::is_error);
        Parsed {
            spec: ok.then_some(spec),
            diagnostics: self.diagnostics,
        }
    }

    /// Applies every declaration, reporting the ones given twice.
    fn declarations(&mut self, lines: &[Line]) {
        let mut first: BTreeMap<&'static str, Span> = BTreeMap::new();
        let mut finals = Vec::new();
        // Declarations with symbols wait until the blank is known.
        let mut with_symbols = Vec::new();
        for line in lines {
            let Line::Item {
                item,
                span,
                value_span,
                ..
            } = line
            else {
                continue;
            };
            let Some(name) = item.declaration() else {
                continue;
            };
            if let Some(&first) = first.get(name) {
                self.diagnostics.push(
                    Diagnostic::error(
                        codes::DUPLICATE_DECLARATION,
                        format!("`{name}` is declared twice"),
                    )
                    .with_span(*span)
                    .with_related(first, "first declared here"),
                );
                continue;
            }
            first.insert(name, *span);
            let spec = &mut self.spec;
            match item {
                Item::Machine(name) => spec.name.clone_from(name),
                Item::Kind(kind) => {
                    spec.kind = *kind;
                    self.map.kind = Some(*span);
                }
                Item::Input(word) => {
                    spec.input.clone_from(word);
                    self.map.input = Some(*value_span);
                }
                &Item::Tapes(n) => {
                    self.map.tapes = Some(*value_span);
                    if let Ok(n) = u8::try_from(n) {
                        spec.tapes = n;
                    } else {
                        self.diagnostics.push(
                            Diagnostic::error(
                                codes::TOO_LARGE,
                                format!("a machine has 1 to {MAX_TAPES} tapes, not {n}"),
                            )
                            .with_span(*value_span),
                        );
                    }
                }
                Item::States(states) => {
                    spec.declared.states = true;
                    spec.states = states
                        .iter()
                        .map(|s| StateSpec {
                            name: s.value.clone(),
                            is_final: false,
                        })
                        .collect();
                    self.map.states = states.iter().map(|s| s.span).collect();
                    self.map.states_decl = Some(*span);
                }
                &Item::Blank(symbol) => {
                    spec.declared.blank = true;
                    spec.blank = symbol.resolve('␣');
                }
                Item::Start(state) => {
                    spec.declared.start = true;
                    spec.start.clone_from(&state.value);
                    self.map.start = Some(state.span);
                }
                Item::Final(states) => finals.extend(states.iter().cloned()),
                Item::Output(_) | Item::InputAlphabet(_) | Item::TapeAlphabet(_) => {
                    with_symbols.push(item);
                }
                Item::Transition(_) => {}
            }
        }
        for item in with_symbols {
            self.symbols(item);
        }
        self.final_states(&finals);
    }

    /// Σ, Γ and the unary output symbol, which may refer to the blank.
    fn symbols(&mut self, item: &Item) {
        let blank = self.spec.blank;
        let resolve = |symbols: &[Located<RawSymbol>]| -> (Vec<char>, Vec<Span>) {
            symbols
                .iter()
                .map(|s| (s.value.resolve(blank), s.span))
                .unzip()
        };
        match item {
            Item::InputAlphabet(symbols) => {
                (self.spec.input_alphabet, self.map.input_alphabet) = resolve(symbols);
                self.spec.declared.input_alphabet = true;
            }
            Item::TapeAlphabet(symbols) => {
                (self.spec.tape_alphabet, self.map.tape_alphabet) = resolve(symbols);
                self.spec.declared.tape_alphabet = true;
            }
            &Item::Output(decoder) => {
                self.spec.output = match decoder {
                    RawDecoder::Word => Decoder::Word,
                    RawDecoder::Unary(symbol) => Decoder::Unary {
                        symbol: symbol.resolve(blank),
                    },
                    RawDecoder::Binary(order) => Decoder::Binary { order },
                };
            }
            _ => {}
        }
    }

    /// Marks the states of F as final, or reports names outside a declared Q.
    fn final_states(&mut self, finals: &[Located<String>]) {
        let mut position: BTreeMap<String, usize> = BTreeMap::new();
        for (i, state) in self.spec.states.iter().enumerate() {
            position.entry(state.name.clone()).or_insert(i);
        }
        let mut seen: BTreeMap<&str, Span> = BTreeMap::new();
        let mut suggestions = MAX_SUGGESTIONS;
        for state in finals {
            let name = state.value.as_str();
            if let Some(&first) = seen.get(name) {
                self.diagnostics.push(
                    Diagnostic::error(
                        codes::DUPLICATE_DECLARATION,
                        format!("the state `{name}` is listed twice in F"),
                    )
                    .with_span(state.span)
                    .with_related(first, "first listed here"),
                );
                continue;
            }
            seen.insert(name, state.span);
            self.map.finals.push((state.value.clone(), state.span));
            if let Some(&i) = position.get(name) {
                self.spec.states[i].is_final = true;
            } else if self.spec.declared.states {
                let names = self.spec.states.iter().map(|s| s.name.as_str());
                let candidate = if suggestions > 0 {
                    suggestions -= 1;
                    closest(name, names)
                } else {
                    None
                };
                let diagnostic = Diagnostic::error(
                    codes::UNKNOWN_NAME,
                    format!("the state `{name}` is not declared in Q"),
                )
                .with_span(state.span);
                self.diagnostics.push(match candidate {
                    Some(candidate) => diagnostic.with_help(format!("did you mean `{candidate}`?")),
                    None => diagnostic.with_help(format!("add `{name}` to Q")),
                });
            } else {
                position.insert(name.into(), self.spec.states.len());
                self.spec.states.push(StateSpec {
                    name: name.into(),
                    is_final: true,
                });
            }
        }
    }

    /// Expands the transitions in source order and attaches every comment.
    fn transitions_and_comments(&mut self, lines: &[Line]) {
        let mut blocks: Vec<Vec<String>> = Vec::new();
        // Whether the last line was a comment, so that the last block is still open.
        let mut open = false;
        let mut seen_item = false;
        for line in lines {
            match line {
                Line::Blank => open = false,
                Line::Comment(text) => {
                    if !open {
                        blocks.push(Vec::new());
                    }
                    if let Some(block) = blocks.last_mut() {
                        block.push(text.clone());
                    }
                    open = true;
                }
                Line::Item {
                    item,
                    span,
                    comment,
                    ..
                } => {
                    let first = self.spec.transitions.len();
                    let is_transition = if let Item::Transition(raw) = item {
                        self.expand(raw, *span);
                        true
                    } else {
                        false
                    };
                    let mut pending = core::mem::take(&mut blocks);
                    let leading = if is_transition && seen_item {
                        pending.drain(..).flatten().collect()
                    } else if is_transition && open {
                        pending.pop().unwrap_or_default()
                    } else {
                        Vec::new()
                    };
                    self.spec
                        .comments
                        .header
                        .extend(pending.into_iter().flatten());
                    if let Some(t) = self
                        .spec
                        .transitions
                        .get_mut(first)
                        .filter(|_| is_transition)
                    {
                        t.leading = leading;
                        t.comment.clone_from(comment);
                    } else if let Some(comment) = comment.as_ref().filter(|_| !is_transition) {
                        self.spec.comments.header.push(comment.clone());
                    }
                    seen_item = true;
                    open = false;
                }
                Line::Error => {
                    blocks.clear();
                    seen_item = true;
                    open = false;
                }
            }
        }
        let rest = blocks.into_iter().flatten();
        if seen_item {
            self.spec.comments.trailing.extend(rest);
        } else {
            self.spec.comments.header.extend(rest);
        }
    }

    /// Adds one transition per combination of read symbols.
    fn expand(&mut self, raw: &RawTransition, span: Span) {
        let tapes = raw.read.len();
        if raw.write.len() != tapes || raw.dir.len() != tapes {
            let entries = |n: usize, one: &str, many: &str| {
                format!("{n} {}", if n == 1 { one } else { many })
            };
            self.incomplete = true;
            self.diagnostics.push(
                Diagnostic::error(
                    codes::WRONG_ARITY,
                    format!(
                        "the transition reads {}, writes {} and has {}",
                        entries(tapes, "symbol", "symbols"),
                        entries(raw.write.len(), "symbol", "symbols"),
                        entries(raw.dir.len(), "move", "moves"),
                    ),
                )
                .with_span(span)
                .with_help("give one read symbol, one written symbol and one move per tape"),
            );
            return;
        }
        let combinations = raw
            .read
            .iter()
            .try_fold(1_usize, |n, set| n.checked_mul(set.len()));
        let total = combinations.and_then(|n| n.checked_add(self.spec.transitions.len()));
        if total.is_none_or(|n| n > MAX_TRANSITIONS) {
            if !self.truncated {
                self.diagnostics.push(
                    Diagnostic::error(
                        codes::TOO_LARGE,
                        format!(
                            "the description expands to more than {MAX_TRANSITIONS} transitions"
                        ),
                    )
                    .with_span(span),
                );
            }
            self.truncated = true;
            self.incomplete = true;
            return;
        }
        let blank = self.spec.blank;
        // Indices into the read sets, first tape outermost.
        let mut choice = vec![0_usize; tapes];
        loop {
            let read: Vec<&Located<RawSymbol>> = raw
                .read
                .iter()
                .zip(&choice)
                .map(|(set, &i)| &set[i])
                .collect();
            let read_glyphs: Vec<char> = read.iter().map(|s| s.value.resolve(blank)).collect();
            let write = raw
                .write
                .iter()
                .zip(&read_glyphs)
                .map(|(w, &r)| w.value.map_or(r, |s| s.resolve(blank)))
                .collect();
            self.spec.transitions.push(TransitionSpec {
                from: raw.from.value.clone(),
                read: read_glyphs,
                to: raw.to.value.clone(),
                write,
                dir: raw.dir.iter().map(|d| d.value).collect(),
                comment: None,
                leading: Vec::new(),
                span: Some(span),
            });
            self.map.transitions.push(TransitionSpans {
                from: raw.from.span,
                to: raw.to.span,
                read: read.iter().map(|s| s.span).collect(),
                write: raw.write.iter().map(|w| w.span).collect(),
            });
            // Advance the last tape fastest.
            let mut tape = tapes;
            loop {
                if tape == 0 {
                    return;
                }
                tape -= 1;
                choice[tape] += 1;
                if choice[tape] < raw.read[tape].len() {
                    break;
                }
                choice[tape] = 0;
            }
        }
    }
}
