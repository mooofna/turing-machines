//! `tm`, the command-line front end of the turing-machines workspace.
//!
//! It runs and traces machines written in the course notation of `turing-core` or in the
//! bbchallenge standard format, checks and formats descriptions, converts between formats,
//! renders space-time and state diagrams, enumerates busy beaver candidates and explores the
//! cryptid Antihydra. See `tm help` and the crate README.

mod cmd;
mod engine;
mod error;
mod input;
mod load;
mod report;
mod table;
mod text;

use std::ffi::OsString;
use std::io::Write;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use crate::error::Error;

const AFTER_HELP: &str = "\
Examples:
  tm run machines/course/unary-multiplication.tm --unary 3 4
  tm trace machines/course/binary-increment.tm --input 1011
  tm check my-machine.tm
  tm run --standard 1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA
  tm enumerate --states 4
  tm render --standard 1RB1LB_1LA1RZ --spacetime bb2.png --diagram bb2.svg

Machines are written in the description language (`.tm`), which follows the notation of the
TheoLog course at TU Dresden, as JSON specs, or in the bbchallenge standard format.

Exit codes: 0 success, 1 invalid machine or failed check, 2 usage or I/O error.
Set TM_NO_TIMING=1 to leave out wall-clock times, for reproducible output.";

/// Run, check and draw Turing machines, and search for busy beavers.
#[derive(Parser, Debug)]
#[command(name = "tm", version, after_help = AFTER_HELP)]
struct Cli {
    /// When to use colours; `auto` uses them on terminals unless `NO_COLOR` is set
    #[arg(long, global = true, value_enum, value_name = "WHEN", default_value_t = ColorWhen::Auto)]
    color: ColorWhen,

    #[command(subcommand)]
    command: Command,
}

/// When to use colours.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum ColorWhen {
    Auto,
    Always,
    Never,
}

impl ColorWhen {
    fn choice(self) -> anstream::ColorChoice {
        match self {
            Self::Auto => anstream::ColorChoice::Auto,
            Self::Always => anstream::ColorChoice::Always,
            Self::Never => anstream::ColorChoice::Never,
        }
    }
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run a machine and report how the run ends
    Run(cmd::run::Args),
    /// Print the configurations `u q v` of a run, one step per line
    Trace(cmd::trace::Args),
    /// Check a machine description and explain every problem
    Check(cmd::check::Args),
    /// Format a description in canonical form
    Fmt(cmd::fmt::Args),
    /// Convert between the description language, the standard format and JSON
    Convert(cmd::convert::Args),
    /// Enumerate all machines of one size and find the busy beaver
    Enumerate(cmd::enumerate::Args),
    /// Decide whether one machine halts on the blank tape
    Decide(cmd::decide::Args),
    /// Draw a space-time diagram (PNG) or a state diagram (SVG)
    Render(cmd::render::Args),
    /// Check the rule of the cryptid Antihydra on the machine and iterate it
    Antihydra(cmd::antihydra::Args),
}

/// Finds `--color WHEN` before clap parses the arguments, so that clap's own help and errors
/// follow it too.
fn color_from_args(args: &[OsString]) -> ColorWhen {
    let mut when = ColorWhen::Auto;
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        let Some(arg) = arg.to_str() else { continue };
        let value = if arg == "--" {
            break;
        } else if arg == "--color" {
            iter.next().and_then(|v| v.to_str())
        } else {
            arg.strip_prefix("--color=")
        };
        if let Some(value) = value
            && let Ok(parsed) = ColorWhen::from_str(value, true)
        {
            when = parsed;
        }
    }
    when
}

fn dispatch(command: Command) -> Result<(), Error> {
    match command {
        Command::Run(args) => cmd::run::exec(&args),
        Command::Trace(args) => cmd::trace::exec(&args),
        Command::Check(args) => cmd::check::exec(&args),
        Command::Fmt(args) => cmd::fmt::exec(&args),
        Command::Convert(args) => cmd::convert::exec(&args),
        Command::Enumerate(args) => cmd::enumerate::exec(&args),
        Command::Decide(args) => cmd::decide::exec(&args),
        Command::Render(args) => cmd::render::exec(&args),
        Command::Antihydra(args) => cmd::antihydra::exec(&args),
    }
}

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().collect();
    color_from_args(&args).choice().write_global();
    let cli = Cli::parse_from(args);
    cli.color.choice().write_global();
    match dispatch(cli.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) if e.is_broken_pipe() => ExitCode::SUCCESS,
        Err(e) => {
            let mut err = anstream::stderr().lock();
            // Nothing sensible is left to do if standard error is gone too.
            let _ = report::error(&mut err, &e);
            let _ = err.flush();
            e.exit_code()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    #[test]
    fn finds_color_before_parsing() {
        assert_eq!(color_from_args(&os(&["tm", "run"])), ColorWhen::Auto);
        assert_eq!(
            color_from_args(&os(&["tm", "--color", "never"])),
            ColorWhen::Never
        );
        assert_eq!(
            color_from_args(&os(&["tm", "x", "--color=always"])),
            ColorWhen::Always
        );
        assert_eq!(
            color_from_args(&os(&["tm", "--", "--color=always"])),
            ColorWhen::Auto
        );
    }

    #[test]
    fn command_line_is_consistent() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
