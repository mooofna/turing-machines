//! `tm check`: parse and validate a description and explain every problem.

use std::io::Write;

use turing_core::Diagnostic;

use crate::cmd;
use crate::error::{Error, Result};
use crate::load::{self, MachineArg, Source};
use crate::report::{self, paint, style};

const AFTER_HELP: &str = "\
Every error and warning is shown with the lines it refers to. Codes E001 to E099 are errors,
W001 to W099 warnings; docs/dsl.md lists them all. The exit code is 1 if there is an error and 0
otherwise, so warnings do not fail a check.

Examples:
  tm check machines/course/unary-multiplication.tm
  tm check --color never broken.tm";

/// Arguments of `tm check`.
#[derive(clap::Args, Debug)]
#[command(after_help = AFTER_HELP)]
pub(crate) struct Args {
    #[command(flatten)]
    machine: MachineArg,
}

/// Runs `tm check`.
pub(crate) fn exec(args: &Args) -> Result<()> {
    let mut out = anstream::stdout().lock();
    let mut err = anstream::stderr().lock();
    let source = Source::read(&args.machine)?;
    let analysis = load::analyse(&source);
    report::diagnostics(
        &mut err,
        &source.origin,
        &source.text,
        &analysis.diagnostics,
    )?;
    err.flush()?;
    let summary = report::summary(&analysis.diagnostics);
    match &analysis.machine {
        Some(machine) if !analysis.has_errors() => {
            let name = if machine.name().is_empty() {
                String::new()
            } else {
                format!(" `{}`", machine.name())
            };
            let warnings = if analysis
                .diagnostics
                .iter()
                .any(|d: &Diagnostic| !d.is_error())
            {
                format!(", {}", paint(style::WARN, &summary))
            } else {
                String::new()
            };
            writeln!(
                out,
                "{} {}: valid{name}, {}{warnings}",
                paint(style::GOOD, "ok"),
                source.origin,
                cmd::shape(machine),
            )?;
            Ok(())
        }
        _ => Err(Error::failed(format!(
            "`{}` is invalid: {summary}",
            source.origin
        ))),
    }
}
