//! Styled output: diagnostics with source snippets, error lines and report fields.
//!
//! Everything is written with ANSI styles; the `anstream` streams strip them when colours are
//! off (`--color never`, `NO_COLOR`, or output that is not a terminal).

use std::io::{self, Write};
use std::ops::Range;

use annotate_snippets::renderer::DecorStyle;
use annotate_snippets::{AnnotationKind, Group, Level, Origin, Renderer, Snippet};
use anstyle::{AnsiColor, Style};
use turing_core::{Diagnostic, Severity, Span};

use crate::text;

/// The styles used across the tool.
pub(crate) mod style {
    use anstyle::{AnsiColor, Style};

    /// Field names in reports and table headers.
    pub(crate) const KEY: Style = Style::new().bold();
    /// Success: halted, accepted, formatted.
    pub(crate) const GOOD: Style = AnsiColor::Green.on_default().bold();
    /// Neither success nor failure: out of steps, undecided.
    pub(crate) const WARN: Style = AnsiColor::Yellow.on_default().bold();
    /// Failure: rejected, mismatch.
    pub(crate) const BAD: Style = AnsiColor::Red.on_default().bold();
    /// Proofs and other facts worth noticing: never halts.
    pub(crate) const NOTE: Style = AnsiColor::Cyan.on_default().bold();
    /// Secondary information.
    pub(crate) const DIM: Style = Style::new().dimmed();
}

/// The width of the key column in reports.
const KEY_WIDTH: usize = 10;

/// Writes `text` in `style`.
pub(crate) fn paint(style: Style, text: impl std::fmt::Display) -> String {
    format!("{style}{text}{style:#}")
}

/// Writes one report line, `key` in bold in a column of fixed width, then `value`.
pub(crate) fn field(
    out: &mut dyn Write,
    key: &str,
    value: impl std::fmt::Display,
) -> io::Result<()> {
    let pad = " ".repeat(KEY_WIDTH.saturating_sub(key.chars().count()).max(1));
    writeln!(out, "{}{pad}{value}", paint(style::KEY, key))
}

/// Writes a continuation line aligned with the values of [`field`].
pub(crate) fn continuation(out: &mut dyn Write, value: impl std::fmt::Display) -> io::Result<()> {
    writeln!(out, "{}{value}", " ".repeat(KEY_WIDTH))
}

/// Writes `error: message` in the style of clap and cargo.
pub(crate) fn error(err: &mut dyn Write, message: impl std::fmt::Display) -> io::Result<()> {
    let style = AnsiColor::Red.on_default().bold();
    writeln!(err, "{}: {message}", paint(style, "error"))
}

/// `2 errors and 1 warning`, `1 error`, `no problems`.
pub(crate) fn summary(diagnostics: &[Diagnostic]) -> String {
    let errors = diagnostics.iter().filter(|d| d.is_error()).count();
    let warnings = diagnostics.len() - errors;
    match (errors, warnings) {
        (0, 0) => "no problems".to_owned(),
        (e, 0) => text::count(e as u64, "error", "errors"),
        (0, w) => text::count(w as u64, "warning", "warnings"),
        (e, w) => format!(
            "{} and {}",
            text::count(e as u64, "error", "errors"),
            text::count(w as u64, "warning", "warnings")
        ),
    }
}

fn renderer() -> Renderer {
    Renderer::styled().decor_style(DecorStyle::Unicode)
}

/// Clamps a span to `source` and to character boundaries.
fn clamp(span: Span, source: &str) -> Range<usize> {
    let len = source.len();
    let mut start = (span.start as usize).min(len);
    let mut end = (span.end as usize).clamp(start, len);
    while !source.is_char_boundary(start) {
        start -= 1;
    }
    while !source.is_char_boundary(end) {
        end += 1;
    }
    start..end
}

/// Renders `diagnostics` against `source`, whose name is `origin`, one block per diagnostic.
pub(crate) fn diagnostics(
    err: &mut dyn Write,
    origin: &str,
    source: &str,
    diagnostics: &[Diagnostic],
) -> io::Result<()> {
    let renderer = renderer();
    for d in diagnostics {
        let level = match d.severity {
            Severity::Error => Level::ERROR,
            Severity::Warning => Level::WARNING,
        };
        let mut title = level.primary_title(d.message.as_str());
        if !d.code.is_empty() {
            title = title.id(d.code);
        }
        let mut group = match d.span {
            Some(span) => {
                let mut snippet = Snippet::source(source)
                    .path(origin)
                    .line_start(1)
                    .annotation(AnnotationKind::Primary.span(clamp(span, source)));
                for (related, note) in &d.related {
                    snippet = snippet.annotation(
                        AnnotationKind::Context
                            .span(clamp(*related, source))
                            .label(note.as_str()),
                    );
                }
                title.element(snippet)
            }
            None => Group::with_title(title).element(Origin::path(origin)),
        };
        if let Some(help) = &d.help {
            group = group.element(Level::HELP.message(help.as_str()));
        }
        writeln!(err, "{}", renderer.render(&[group]))?;
        writeln!(err)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_spans_to_char_boundaries() {
        let source = "δ(q0, ␣)";
        assert_eq!(clamp(Span::new(1, 2), source), 0..2);
        assert_eq!(
            clamp(Span::new(100, 200), source),
            source.len()..source.len()
        );
    }

    #[test]
    fn summarises_counts() {
        let e = Diagnostic::error("E001", "x");
        let w = Diagnostic::warning("W001", "y");
        assert_eq!(summary(&[]), "no problems");
        assert_eq!(summary(std::slice::from_ref(&e)), "1 error");
        assert_eq!(summary(&[e.clone(), e, w]), "2 errors and 1 warning");
    }
}
