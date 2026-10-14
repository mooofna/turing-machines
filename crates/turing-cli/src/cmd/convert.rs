//! `tm convert`: between the description language, the standard format and JSON.

use std::io::Write;

use turing_core::dsl::{self, FormatStyle};
use turing_core::standard;

use crate::error::{Error, Result};
use crate::load::{self, MachineArg};

const AFTER_HELP: &str = "\
The standard format of the bbchallenge project describes single-tape machines over the symbols
0, 1, ... with states A, B, ... and moves L and R only; converting other machines to it fails.
JSON is the serialised machine spec that the web playground uses.

Examples:
  tm convert --standard 1RB1RZ_1LB0RC_1LC1LA --to dsl
  tm convert bb3.tm --to standard
  tm convert machines/course/binary-increment.tm --to json > increment.json";

/// Arguments of `tm convert`.
#[derive(clap::Args, Debug)]
#[command(after_help = AFTER_HELP)]
pub(crate) struct Args {
    #[command(flatten)]
    machine: MachineArg,

    /// The format to write
    #[arg(long, value_enum, value_name = "FORMAT")]
    to: Target,

    /// With `--to dsl`: write `delta`, `Sigma`, `Gamma` and `_` instead of `δ`, `Σ`, `Γ` and `␣`
    #[arg(long)]
    ascii: bool,
}

/// The format `tm convert` writes.
#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
enum Target {
    /// The description language in canonical form
    Dsl,
    /// The bbchallenge standard format, for example `1RB1LB_1LA1RZ`
    Standard,
    /// The machine spec as JSON
    Json,
}

/// Runs `tm convert`.
pub(crate) fn exec(args: &Args) -> Result<()> {
    let mut out = anstream::stdout().lock();
    let mut err = anstream::stderr().lock();
    let loaded = load::load(&args.machine, &mut err)?;
    match args.to {
        Target::Dsl => {
            let style = if args.ascii {
                FormatStyle::Ascii
            } else {
                FormatStyle::Unicode
            };
            let text = dsl::format(&loaded.spec, style);
            out.write_all(text.as_bytes())?;
            if !text.ends_with('\n') {
                writeln!(out)?;
            }
        }
        Target::Standard => {
            let text = standard::to_standard(&loaded.machine).map_err(|e| {
                Error::failed(format!("cannot convert `{}`: {e}", loaded.source.origin))
            })?;
            writeln!(out, "{text}")?;
        }
        Target::Json => {
            let mut spec = loaded.spec;
            for t in &mut spec.transitions {
                t.span = None;
            }
            serde_json::to_writer_pretty(&mut out, &spec).map_err(std::io::Error::from)?;
            writeln!(out)?;
        }
    }
    Ok(())
}
