//! `tm decide`: run the decider pipeline on one machine.

use std::time::Instant;

use turing_bb::{CompactTm, Next, Pipeline, Verdict};
use turing_core::{ChainSim, Diagnostic, Outcome, Span, TapeView};

use crate::error::{Error, Result};
use crate::report::{self, paint, style};
use crate::text;

const AFTER_HELP: &str = "\
The deciders are those of the enumeration, in the same order: cyclers and translated cyclers,
n-gram closed position sets (plain, with history and with LRU) and repeated word lists. A machine
is reported as halting only after it has actually been run to its halt.

Examples:
  tm decide 1RB1LB_1LA1RZ
  tm decide 1RB0LB_1LA0RB";

/// Arguments of `tm decide`.
#[derive(clap::Args, Debug)]
#[command(after_help = AFTER_HELP)]
pub(crate) struct Args {
    /// The machine in the bbchallenge standard format, for example `1RB1LB_1LA1RZ`
    #[arg(value_name = "STANDARD")]
    machine: String,
}

/// `δ(B, 1)`.
fn transition_name(state: u8, symbol: u8) -> String {
    format!("δ({}, {symbol})", char::from(b'A' + state))
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
    let verdict = Pipeline::standard().decide(&tm);
    report::field(&mut out, "machine", tm)?;
    match verdict {
        Verdict::Halts { steps, at, .. } => {
            // Run the machine itself, so that the numbers follow the busy beaver convention.
            let machine = tm.to_machine();
            let mut sim = ChainSim::new(&machine, &[]).map_err(|e| Error::failed(e.to_string()))?;
            let outcome = sim.run(steps.saturating_add(2));
            let how = match tm.get(at.0, at.1) {
                Some(action) if action.next == Next::Halt => format!(
                    "by {} = {}{}Z",
                    transition_name(at.0, at.1),
                    action.write,
                    action.dir
                ),
                _ => format!("at {}, which is undefined", transition_name(at.0, at.1)),
            };
            let summary = match outcome {
                Outcome::Halted { steps, .. } => format!(
                    "halts after {} with {}",
                    text::count(steps, "step", "steps"),
                    text::count(sim.tape().count_non_blank(), "one", "ones")
                ),
                _ => format!("halts after {}", text::count(steps, "step", "steps")),
            };
            report::field(
                &mut out,
                "verdict",
                format!("{} {}", paint(style::GOOD, summary), paint(style::DIM, how)),
            )?;
        }
        Verdict::NonHalting(id) => {
            report::field(
                &mut out,
                "verdict",
                format!(
                    "{} {}",
                    paint(style::NOTE, "never halts"),
                    paint(
                        style::DIM,
                        format!("(proved by {}, `{}`)", id.label(), id.key())
                    )
                ),
            )?;
        }
        Verdict::Unknown => {
            report::field(
                &mut out,
                "verdict",
                format!(
                    "{} {}",
                    paint(style::WARN, "undecided"),
                    paint(
                        style::DIM,
                        "(no decider in the pipeline settles this machine)"
                    )
                ),
            )?;
        }
    }
    if text::timing_enabled() {
        report::field(&mut out, "time", text::duration(start.elapsed()))?;
    }
    Ok(())
}
