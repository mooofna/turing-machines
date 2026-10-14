//! Loading a machine from a file, standard input or a string in the standard format, and
//! reporting what is wrong with it.

use std::io::{Read, Write};
use std::path::PathBuf;

use turing_core::{Diagnostic, Machine, MachineSpec, Span, dsl, standard};

use crate::error::{Error, Result};
use crate::report;

/// Where a machine comes from.
#[derive(clap::Args, Debug, Clone)]
#[group(required = true, multiple = false)]
pub(crate) struct MachineArg {
    /// A machine description: a `.tm` file in the description language, a `.json` spec, or `-`
    /// for standard input
    #[arg(value_name = "FILE")]
    pub(crate) file: Option<PathBuf>,

    /// A single-tape machine in the bbchallenge standard format instead of a file, for example
    /// `1RB1LB_1LA1RZ`
    #[arg(long, value_name = "STRING")]
    pub(crate) standard: Option<String>,
}

/// The notation a source is written in.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Format {
    /// The description language (`.tm`).
    Dsl,
    /// A serialised [`MachineSpec`] (`.json`).
    Json,
    /// The bbchallenge standard format.
    Standard,
}

/// The text of a machine description and where it came from.
#[derive(Clone, Debug)]
pub(crate) struct Source {
    /// The name shown in messages: the path, `<stdin>` or `--standard`.
    pub(crate) origin: String,
    /// The file, if the source is one.
    pub(crate) path: Option<PathBuf>,
    /// The text.
    pub(crate) text: String,
    /// The notation.
    pub(crate) format: Format,
}

impl Source {
    /// Reads the source named by `arg`.
    pub(crate) fn read(arg: &MachineArg) -> Result<Self> {
        if let Some(text) = &arg.standard {
            return Ok(Self {
                origin: "--standard".to_owned(),
                path: None,
                text: text.trim().to_owned(),
                format: Format::Standard,
            });
        }
        let Some(path) = &arg.file else {
            return Err(Error::usage("give a FILE or --standard STRING"));
        };
        let (origin, path, text) = if path.as_os_str() == "-" {
            let mut text = String::new();
            std::io::stdin()
                .read_to_string(&mut text)
                .map_err(|source| Error::File {
                    action: "read",
                    path: "<stdin>".to_owned(),
                    source,
                })?;
            ("<stdin>".to_owned(), None, text)
        } else {
            let text = std::fs::read_to_string(path).map_err(|source| Error::File {
                action: "read",
                path: path.display().to_string(),
                source,
            })?;
            (path.display().to_string(), Some(path.clone()), text)
        };
        let is_json = path
            .as_ref()
            .and_then(|p| p.extension())
            .is_some_and(|e| e.eq_ignore_ascii_case("json"))
            || text.trim_start().starts_with('{');
        Ok(Self {
            origin,
            path,
            text,
            format: if is_json { Format::Json } else { Format::Dsl },
        })
    }
}

/// Everything known about a source: the spec and machine if they could be built, and every
/// diagnostic.
#[derive(Debug)]
pub(crate) struct Analysis {
    /// The spec, if the source could be parsed.
    pub(crate) spec: Option<MachineSpec>,
    /// The machine, if there are no errors.
    pub(crate) machine: Option<Machine>,
    /// Errors and warnings.
    pub(crate) diagnostics: Vec<Diagnostic>,
}

impl Analysis {
    /// Whether any diagnostic is an error.
    pub(crate) fn has_errors(&self) -> bool {
        self.machine.is_none() || self.diagnostics.iter().any(Diagnostic::is_error)
    }
}

/// Parses, checks and compiles `source`.
pub(crate) fn analyse(source: &Source) -> Analysis {
    let mut diagnostics = Vec::new();
    let spec = match source.format {
        Format::Dsl => {
            let parsed = dsl::parse(&source.text);
            diagnostics = parsed.diagnostics;
            parsed.spec
        }
        Format::Json => match serde_json::from_str::<MachineSpec>(&source.text) {
            Ok(mut spec) => {
                // Spans in a JSON spec point into some other text.
                for t in &mut spec.transitions {
                    t.span = None;
                }
                Some(spec)
            }
            Err(e) => {
                diagnostics.push(json_diagnostic(&source.text, &e));
                None
            }
        },
        Format::Standard => match standard::spec_from_standard(&source.text) {
            Ok(spec) => Some(spec),
            Err(e) => {
                diagnostics.push(standard_diagnostic(&e));
                None
            }
        },
    };
    let mut machine = None;
    if let Some(spec) = &spec
        && !diagnostics.iter().any(Diagnostic::is_error)
    {
        if source.format == Format::Standard {
            match standard::parse_standard(&source.text) {
                Ok(m) => machine = Some(m),
                Err(e) => diagnostics.push(standard_diagnostic(&e)),
            }
        } else {
            merge(&mut diagnostics, spec.check());
            if !diagnostics.iter().any(Diagnostic::is_error) {
                match spec.compile() {
                    Ok(m) => machine = Some(m),
                    Err(found) => merge(&mut diagnostics, found),
                }
            }
        }
    }
    if spec.is_none() && !diagnostics.iter().any(Diagnostic::is_error) {
        diagnostics.push(Diagnostic::error("", "the description defines no machine"));
    }
    Analysis {
        spec,
        machine,
        diagnostics,
    }
}

/// Appends the diagnostics of `more` that are not already in `into`.
fn merge(into: &mut Vec<Diagnostic>, more: Vec<Diagnostic>) {
    for d in more {
        if !into.contains(&d) {
            into.push(d);
        }
    }
}

fn json_diagnostic(text: &str, e: &serde_json::Error) -> Diagnostic {
    let line_start: usize = text
        .split_inclusive('\n')
        .take(e.line().saturating_sub(1))
        .map(str::len)
        .sum();
    let at = line_start + e.column().saturating_sub(1);
    let at = u32::try_from(at).unwrap_or(u32::MAX);
    let message = e.to_string();
    // serde_json appends " at line L column C", which the snippet shows anyway.
    let message = message
        .rsplit_once(" at line ")
        .map_or(message.as_str(), |(m, _)| m);
    Diagnostic::error("", format!("invalid machine spec: {message}"))
        .with_span(Span::new(at, at.saturating_add(1)))
}

fn standard_diagnostic(e: &standard::StandardError) -> Diagnostic {
    match e {
        standard::StandardError::Syntax { at, message } => {
            let at = u32::try_from(*at).unwrap_or(u32::MAX);
            Diagnostic::error("", format!("invalid standard format: {message}"))
                .with_span(Span::new(at, at.saturating_add(1)))
                .with_help("states are separated by `_`, each transition is written like `1RB`, `---` is undefined")
        }
        other => Diagnostic::error("", other.to_string()),
    }
}

/// A machine that is ready to run, with its source and spec.
#[derive(Debug)]
pub(crate) struct Loaded {
    /// Where it came from.
    pub(crate) source: Source,
    /// The spec.
    pub(crate) spec: MachineSpec,
    /// The machine.
    pub(crate) machine: Machine,
}

impl Loaded {
    /// The machine's name, or the file stem if it has none.
    pub(crate) fn name(&self) -> String {
        if !self.machine.name().is_empty() {
            return self.machine.name().to_owned();
        }
        if self.source.format == Format::Standard {
            return self.source.text.clone();
        }
        self.source
            .path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map_or_else(
                || self.source.origin.clone(),
                |s| s.to_string_lossy().into_owned(),
            )
    }
}

/// Loads the machine named by `arg`. Diagnostics are printed to `err` if it is invalid.
pub(crate) fn load(arg: &MachineArg, err: &mut dyn Write) -> Result<Loaded> {
    let source = Source::read(arg)?;
    let Analysis {
        spec,
        machine,
        diagnostics,
    } = analyse(&source);
    match (spec, machine) {
        (Some(spec), Some(machine)) if !diagnostics.iter().any(Diagnostic::is_error) => {
            Ok(Loaded {
                source,
                spec,
                machine,
            })
        }
        _ => {
            report::diagnostics(err, &source.origin, &source.text, &diagnostics)?;
            Err(Error::Invalid {
                origin: source.origin,
                summary: report::summary(&diagnostics),
            })
        }
    }
}
