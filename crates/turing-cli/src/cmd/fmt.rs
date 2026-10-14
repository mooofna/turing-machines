//! `tm fmt`: canonical formatting of descriptions.

use std::io::Write;
use std::path::PathBuf;

use turing_core::dsl::{self, FormatStyle};

use crate::error::{Error, Result};
use crate::load::{Format, MachineArg, Source};
use crate::report::{self, paint, style};

const AFTER_HELP: &str = "\
The canonical form writes one declaration per line and one transition δ(q, a) = (q', b, D) per
line, expands read sets such as {x, X} and the symbol * that writes back what was read, and keeps
every comment.

Examples:
  tm fmt machine.tm             print the formatted description
  tm fmt --write machine.tm     format the file in place
  tm fmt --check machine.tm     fail with a diff if the file is not formatted
  tm fmt --ascii machine.tm     write delta, Sigma, Gamma and _ instead of δ, Σ, Γ and ␣";

/// Arguments of `tm fmt`.
#[derive(clap::Args, Debug)]
#[command(after_help = AFTER_HELP)]
pub(crate) struct Args {
    /// The description to format, or `-` for standard input
    #[arg(value_name = "FILE")]
    file: PathBuf,

    /// Write `delta`, `Sigma`, `Gamma` and `_` instead of `δ`, `Σ`, `Γ` and `␣`
    #[arg(long)]
    ascii: bool,

    /// Print a diff and fail if the file is not formatted, instead of printing it
    #[arg(long, conflicts_with = "write")]
    check: bool,

    /// Rewrite the file in place
    #[arg(short, long)]
    write: bool,
}

/// Runs `tm fmt`.
pub(crate) fn exec(args: &Args) -> Result<()> {
    let mut out = anstream::stdout().lock();
    let mut err = anstream::stderr().lock();
    let source = Source::read(&MachineArg {
        file: Some(args.file.clone()),
        standard: None,
    })?;
    if source.format != Format::Dsl {
        return Err(Error::usage(format!(
            "`{}` is not written in the description language; use `tm convert --to dsl` to turn \
             it into one",
            source.origin
        )));
    }
    if args.write && source.path.is_none() {
        return Err(Error::usage(
            "--write needs a file; standard input can only be printed",
        ));
    }
    let parsed = dsl::parse(&source.text);
    let Some(spec) = parsed.spec else {
        report::diagnostics(&mut err, &source.origin, &source.text, &parsed.diagnostics)?;
        return Err(Error::Invalid {
            origin: source.origin,
            summary: report::summary(&parsed.diagnostics),
        });
    };
    let style = if args.ascii {
        FormatStyle::Ascii
    } else {
        FormatStyle::Unicode
    };
    let formatted = dsl::format(&spec, style);

    if args.check {
        if formatted == source.text {
            return Ok(());
        }
        write_diff(&mut out, &source.origin, &source.text, &formatted)?;
        return Err(Error::failed(format!(
            "`{0}` is not formatted; run `tm fmt --write {0}`",
            source.origin
        )));
    }
    if args.write {
        if formatted != source.text
            && let Some(path) = &source.path
        {
            std::fs::write(path, &formatted).map_err(|e| Error::File {
                action: "write",
                path: source.origin.clone(),
                source: e,
            })?;
        }
        return Ok(());
    }
    out.write_all(formatted.as_bytes())?;
    Ok(())
}

/// One line of a diff.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Line<'a> {
    Same(&'a str),
    Removed(&'a str),
    Added(&'a str),
}

/// The largest table the line diff may build, in cells.
const MAX_DIFF_CELLS: usize = 4_000_000;

/// A line diff by longest common subsequence, or `None` if the texts are too long.
fn diff<'a>(old: &'a str, new: &'a str) -> Option<Vec<Line<'a>>> {
    let before: Vec<&str> = old.lines().collect();
    let after: Vec<&str> = new.lines().collect();
    let (rows, cols) = (before.len(), after.len());
    if (rows + 1).saturating_mul(cols + 1) > MAX_DIFF_CELLS {
        return None;
    }
    // lcs[at(i, j)] is the length of the longest common subsequence of before[i..] and
    // after[j..].
    let mut lcs = vec![0u32; (rows + 1) * (cols + 1)];
    let at = |i: usize, j: usize| i * (cols + 1) + j;
    for i in (0..rows).rev() {
        for j in (0..cols).rev() {
            lcs[at(i, j)] = if before[i] == after[j] {
                lcs[at(i + 1, j + 1)] + 1
            } else {
                lcs[at(i + 1, j)].max(lcs[at(i, j + 1)])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut lines = Vec::with_capacity(rows.max(cols));
    while i < rows || j < cols {
        if i < rows && j < cols && before[i] == after[j] {
            lines.push(Line::Same(before[i]));
            i += 1;
            j += 1;
        } else if i < rows && (j == cols || lcs[at(i + 1, j)] >= lcs[at(i, j + 1)]) {
            lines.push(Line::Removed(before[i]));
            i += 1;
        } else {
            lines.push(Line::Added(after[j]));
            j += 1;
        }
    }
    Some(lines)
}

/// Writes the changed lines with two lines of context, like `diff -u`.
fn write_diff(out: &mut dyn Write, origin: &str, old: &str, new: &str) -> std::io::Result<()> {
    const CONTEXT: usize = 2;
    let Some(lines) = diff(old, new) else {
        return writeln!(out, "{origin}: too long to show the differences");
    };
    if old.lines().eq(new.lines()) {
        return writeln!(
            out,
            "{origin}: only line endings or the final newline differ"
        );
    }
    writeln!(out, "{}", paint(style::KEY, format!("--- {origin}")))?;
    writeln!(
        out,
        "{}",
        paint(style::KEY, format!("+++ {origin} (formatted)"))
    )?;
    let changed: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| !matches!(l, Line::Same(_)))
        .map(|(i, _)| i)
        .collect();
    let near = |i: usize| changed.iter().any(|&c| c.abs_diff(i) <= CONTEXT);
    let mut old_line = 1;
    let mut gap = true;
    for (i, line) in lines.iter().enumerate() {
        if near(i) {
            if gap {
                writeln!(
                    out,
                    "{}",
                    paint(style::NOTE, format!("@@ line {old_line} @@"))
                )?;
            }
            gap = false;
            match line {
                Line::Same(t) => writeln!(out, " {t}")?,
                Line::Removed(t) => writeln!(out, "{}", paint(style::BAD, format!("-{t}")))?,
                Line::Added(t) => writeln!(out, "{}", paint(style::GOOD, format!("+{t}")))?,
            }
        } else {
            gap = true;
        }
        if !matches!(line, Line::Added(_)) {
            old_line += 1;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diffs_lines() {
        let lines = diff("a\nb\nc\n", "a\nB\nc\nd\n").unwrap();
        assert_eq!(
            lines,
            vec![
                Line::Same("a"),
                Line::Removed("b"),
                Line::Added("B"),
                Line::Same("c"),
                Line::Added("d"),
            ]
        );
    }

    #[test]
    fn writes_hunks_with_context() {
        let old = "1\n2\n3\n4\n5\n6\n7\n8\n";
        let new = "1\n2\n3\n4\n5\n6\nseven\n8\n";
        let mut buf = Vec::new();
        write_diff(&mut buf, "m.tm", old, new).unwrap();
        let text = String::from_utf8(buf).unwrap();
        let plain: String = anstream::adapter::strip_str(&text).to_string();
        assert_eq!(
            plain,
            "--- m.tm\n+++ m.tm (formatted)\n@@ line 5 @@\n 5\n 6\n-7\n+seven\n 8\n"
        );
    }
}
