//! `tm decide`: run the decider pipeline on one machine.

use std::time::Instant;

use turing_bb::{CompactTm, Next, Pipeline, Verdict};
use turing_core::{ChainSim, Diagnostic, Machine, Outcome, Span, TapeView};

use crate::cmd::parse_count;
use crate::error::{Error, Result};
use crate::report::{self, paint, style};
use crate::text;

const AFTER_HELP: &str = "\
The deciders are those of the enumeration, in the same order: cyclers and translated cyclers,
n-gram closed position sets (plain, with history and with LRU) and repeated word lists. A machine
is reported as halting only after it has actually been run to its halt. If no decider settles
the machine, it is run with chain steps for up to --max-steps steps, which finds long halting
runs such as that of the 5-state busy beaver.

Examples:
  tm decide 1RB1LB_1LA1RZ
  tm decide 1RB0LB_1LA0RB
  tm decide 1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA";

/// Arguments of `tm decide`.
#[derive(clap::Args, Debug)]
#[command(after_help = AFTER_HELP)]
pub(crate) struct Args {
    /// The machine in the bbchallenge standard format, for example `1RB1LB_1LA1RZ`
    #[arg(value_name = "STANDARD")]
    machine: String,

    /// If no decider settles the machine, run it for at most this many steps to see it halt
    #[arg(long, value_name = "N", default_value = "1e8", value_parser = parse_count)]
    max_steps: u64,
}

/// `δ(B, 1)`.
fn transition_name(state: u8, symbol: u8) -> String {
    format!("δ({}, {symbol})", char::from(b'A' + state))
}

/// Runs `machine` for at most `fuel` steps and says how the run ends.
fn simulate(machine: &Machine, fuel: u64, how: &str) -> Result<String> {
    let mut sim = ChainSim::new(machine, &[]).map_err(|e| Error::failed(e.to_string()))?;
    Ok(match sim.run(fuel) {
        Outcome::Halted { steps, .. } => format!(
            "{} {}",
            paint(
                style::GOOD,
                format!(
                    "halts after {} with {}",
                    text::count(steps, "step", "steps"),
                    text::count(sim.tape().count_non_blank(), "one", "ones")
                )
            ),
            paint(style::DIM, how)
        ),
        Outcome::NeverHalts { proof, .. } => format!(
            "{} {}",
            paint(style::NOTE, "never halts"),
            paint(
                style::DIM,
                format!("({})", super::proof_text(machine, proof))
            )
        ),
        Outcome::OutOfFuel { steps } | Outcome::Stopped { steps } => format!(
            "{} {}",
            paint(style::WARN, "undecided"),
            paint(
                style::DIM,
                format!(
                    "(no decider settles it, and it runs for more than {}; see --max-steps)",
                    text::count(steps, "step", "steps")
                )
            )
        ),
    })
}

/// Runs `tm decide`.
pub(crate) fn exec(args: &Args) -> Result<()> {
    let mut out = anstream::stdout().lock();
    let mut err = anstream::stderr().lock();
    let text_in = args.machine.trim();
    let tm: CompactTm = match text_in.parse() {
        Ok(tm) => tm,
        Err(e) => {
            let at = u32::try_from(e.at).unwrap_or(u32::MAX);
            let diagnostic = Diagnostic::error("", format!("invalid machine: {}", e.message))
                .with_span(Span::new(at, at.saturating_add(1)));
            let diagnostics = [diagnostic];
            report::diagnostics(&mut err, "STANDARD", text_in, &diagnostics)?;
            return Err(Error::Invalid {
                origin: text_in.to_owned(),
                summary: report::summary(&diagnostics),
            });
        }
    };

    let start = Instant::now();
    let machine = tm.to_machine();
    let verdict = match Pipeline::standard().decide(&tm) {
        Verdict::Halts { steps, at, .. } => {
            let how = match tm.get(at.0, at.1) {
                Some(action) if action.next == Next::Halt => format!(
                    "({} = {}{}Z)",
                    transition_name(at.0, at.1),
                    action.write,
                    action.dir
                ),
                _ => format!("({} is undefined)", transition_name(at.0, at.1)),
            };
            // Run the machine itself, so that the numbers follow the busy beaver convention.
            simulate(&machine, steps.saturating_add(2), &how)?
        }
        Verdict::NonHalting(id) => format!(
            "{} {}",
            paint(style::NOTE, "never halts"),
            paint(
                style::DIM,
                format!("(decider: {})", super::enumerate::lower_first(id.label()))
            )
        ),
        Verdict::Unknown => simulate(&machine, args.max_steps, "(found by simulation)")?,
    };
    report::field(&mut out, "machine", tm)?;
    report::field(&mut out, "verdict", verdict)?;
    if text::timing_enabled() {
        report::field(&mut out, "time", text::duration(start.elapsed()))?;
    }
    Ok(())
}
