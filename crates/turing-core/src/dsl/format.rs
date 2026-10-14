//! The canonical formatter.
//!
//! The layout has up to four blocks separated by blank lines:
//!
//! 1. header comments, then `machine`, `kind`, `tapes`, `input` and `output` where they differ
//!    from the defaults;
//! 2. the declared sets `Q`, `Σ`, `Γ`, `blank` (if declared or not `␣`), `start`, and `F`
//!    whenever a state is final;
//! 3. the transitions in spec order, each with its leading comments (preceded by a blank
//!    line) and its trailing comment. Within each run of transitions without leading comments
//!    the `=` signs and the trailing comments are aligned;
//! 4. the trailing comments.
//!
//! A header without settings is followed by a blank line, which keeps it apart from the
//! leading comments of a first transition.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::FormatStyle;
use crate::machine::{BitOrder, Decoder, Kind};
use crate::spec::{MachineSpec, TransitionSpec};

pub(super) fn format(spec: &MachineSpec, style: FormatStyle) -> String {
    let f = Formatter { spec, style };
    let blocks = [f.settings(), f.sets(), f.transitions(), f.trailing()];
    let mut out = String::new();
    for block in blocks.iter().filter(|b| !b.is_empty()) {
        if !out.is_empty() {
            out.push('\n');
        }
        for line in block {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

struct Formatter<'a> {
    spec: &'a MachineSpec,
    style: FormatStyle,
}

/// `// text` for each line of `text`.
fn comment_lines(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split('\n').map(|line| {
        let line = line.trim_end();
        if line.is_empty() {
            "//".to_string()
        } else {
            format!("// {line}")
        }
    })
}

/// A string literal with `"`, `\`, line breaks and tabs escaped.
fn string_literal(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// Whether `glyph` must be quoted to be read back as itself: whitespace and control
/// characters, the punctuation of the grammar, `/` (two would start a comment), and `_` and
/// `␣`, which unquoted stand for the blank.
fn needs_quotes(glyph: char) -> bool {
    glyph.is_whitespace()
        || glyph.is_control()
        || matches!(
            glyph,
            '(' | ')' | '{' | '}' | ',' | '=' | '\'' | '*' | ';' | '"' | '/' | '_' | '␣'
        )
}

/// `{a, b, c}`.
fn set(items: impl IntoIterator<Item = String>) -> String {
    format!("{{{}}}", items.into_iter().collect::<Vec<_>>().join(", "))
}

/// One entry, or `(a, b)` for several tapes.
fn tuple(items: &[String]) -> String {
    if let [single] = items {
        single.clone()
    } else {
        format!("({})", items.join(", "))
    }
}

impl Formatter<'_> {
    fn ascii(&self) -> bool {
        self.style == FormatStyle::Ascii
    }

    /// A symbol as the parser reads it back: the default blank as `␣` or `_`, any other glyph
    /// bare or quoted. A bare `_` or `␣` always stands for the blank; since the blank glyph
    /// itself is needed in `blank = ...`, a blank other than `␣` is written as its glyph.
    fn symbol(&self, glyph: char) -> String {
        if glyph == '␣' && glyph == self.spec.blank {
            return if self.ascii() { "_" } else { "␣" }.into();
        }
        if needs_quotes(glyph) {
            format!("'{glyph}'")
        } else {
            glyph.to_string()
        }
    }

    fn settings(&self) -> Vec<String> {
        let spec = self.spec;
        let mut lines: Vec<String> = spec
            .comments
            .header
            .iter()
            .flat_map(|c| comment_lines(c))
            .collect();
        if !spec.name.is_empty() {
            lines.push(format!("machine {}", string_literal(&spec.name)));
        }
        if spec.kind != Kind::Transducer {
            let kind = match spec.kind {
                Kind::Acceptor => "acceptor",
                Kind::Enumerator => "enumerator",
                Kind::Beaver => "beaver",
                Kind::Transducer => "transducer",
            };
            lines.push(format!("kind {kind}"));
        }
        if spec.tapes != 1 {
            lines.push(format!("tapes = {}", spec.tapes));
        }
        if !spec.input.is_empty() {
            lines.push(format!("input {}", string_literal(&spec.input)));
        }
        let output = match spec.output {
            Decoder::Word => None,
            Decoder::Unary { symbol } => Some(format!("unary {}", self.symbol(symbol))),
            Decoder::Binary {
                order: BitOrder::Lsb,
            } => Some("binary lsb".into()),
            Decoder::Binary {
                order: BitOrder::Msb,
            } => Some("binary msb".into()),
        };
        lines.extend(output.map(|o| format!("output {o}")));
        lines
    }

    fn sets(&self) -> Vec<String> {
        let spec = self.spec;
        let (sigma, gamma) = if self.ascii() {
            ("Sigma", "Gamma")
        } else {
            ("Σ", "Γ")
        };
        let symbols = |glyphs: &[char]| set(glyphs.iter().map(|&g| self.symbol(g)));
        let mut lines = Vec::new();
        if spec.declared.states {
            lines.push(format!(
                "Q = {}",
                set(spec.states.iter().map(|s| s.name.clone()))
            ));
        }
        if spec.declared.input_alphabet {
            lines.push(format!("{sigma} = {}", symbols(&spec.input_alphabet)));
        }
        if spec.declared.tape_alphabet {
            lines.push(format!("{gamma} = {}", symbols(&spec.tape_alphabet)));
        }
        if spec.declared.blank || spec.blank != '␣' {
            lines.push(format!("blank = {}", self.symbol(spec.blank)));
        }
        if spec.declared.start {
            lines.push(format!("start = {}", spec.start));
        }
        if spec.states.iter().any(|s| s.is_final) {
            lines.push(format!(
                "F = {}",
                set(spec
                    .states
                    .iter()
                    .filter(|s| s.is_final)
                    .map(|s| s.name.clone()))
            ));
        }
        lines
    }

    /// `δ(q, a)` and `(p, b, D)` of a transition.
    fn sides(&self, t: &TransitionSpec) -> (String, String) {
        let delta = if self.ascii() { "delta" } else { "δ" };
        let read: Vec<String> = t.read.iter().map(|&g| self.symbol(g)).collect();
        let write: Vec<String> = t.write.iter().map(|&g| self.symbol(g)).collect();
        let dir: Vec<String> = t.dir.iter().map(|d| d.as_char().to_string()).collect();
        let (read, write, dir) = (tuple(&read), tuple(&write), tuple(&dir));
        (
            format!("{delta}({}, {read})", t.from),
            format!("({}, {write}, {dir})", t.to),
        )
    }

    fn transitions(&self) -> Vec<String> {
        let transitions = &self.spec.transitions;
        let mut lines = Vec::new();
        let mut start = 0;
        while start < transitions.len() {
            // A group runs up to the next transition with leading comments.
            let end = transitions[start + 1..]
                .iter()
                .position(|t| !t.leading.is_empty())
                .map_or(transitions.len(), |i| start + 1 + i);
            let group = &transitions[start..end];
            if start > 0 && !group[0].leading.is_empty() {
                lines.push(String::new());
            }
            lines.extend(group[0].leading.iter().flat_map(|c| comment_lines(c)));
            self.group(group, &mut lines);
            start = end;
        }
        lines
    }

    /// Transitions with aligned `=` signs and trailing comments.
    fn group(&self, group: &[TransitionSpec], lines: &mut Vec<String>) {
        let sides: Vec<(String, String)> = group.iter().map(|t| self.sides(t)).collect();
        let lhs_width = sides
            .iter()
            .map(|(lhs, _)| lhs.chars().count())
            .max()
            .unwrap_or(0);
        let bodies: Vec<String> = sides
            .iter()
            .map(|(lhs, rhs)| {
                let pad = lhs_width - lhs.chars().count();
                format!("{lhs}{:pad$} = {rhs}", "")
            })
            .collect();
        let body_width = bodies.iter().map(|b| b.chars().count()).max().unwrap_or(0);
        for (body, t) in bodies.into_iter().zip(group) {
            match &t.comment {
                Some(comment) => {
                    let pad = body_width - body.chars().count();
                    let text = comment.replace('\n', " ");
                    let text = text.trim_end();
                    if text.is_empty() {
                        lines.push(format!("{body}{:pad$}  //", ""));
                    } else {
                        lines.push(format!("{body}{:pad$}  // {text}", ""));
                    }
                }
                None => lines.push(body),
            }
        }
    }

    fn trailing(&self) -> Vec<String> {
        self.spec
            .comments
            .trailing
            .iter()
            .flat_map(|c| comment_lines(c))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoting() {
        for glyph in [
            'x', '#', '0', 'Ä', '+', '␣', '_', '*', '\'', '"', ' ', '/', ';',
        ] {
            let quoted = needs_quotes(glyph);
            let expected = matches!(glyph, '␣' | '_' | '*' | '\'' | '"' | ' ' | '/' | ';');
            assert_eq!(quoted, expected, "{glyph:?}");
        }
    }

    #[test]
    fn literals() {
        assert_eq!(string_literal("a\"b\\c\nd"), r#""a\"b\\c\nd""#);
    }
}
