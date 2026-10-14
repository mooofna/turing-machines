//! `tm run`: run a machine and report how the run ends.

use std::io::Write;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use turing_core::{Kind, Machine, Outcome, Output};

use crate::cmd::{self, parse_count};
use crate::engine::Engine;
use crate::error::{Error, Result};
use crate::input::{self, InputArgs};
use crate::load::{self, MachineArg};
use crate::report::{self, paint, style};
use crate::text;

const AFTER_HELP: &str = "\
Examples:
  tm run machines/course/unary-multiplication.tm --unary 3 4
  tm run machines/course/binary-increment.tm --input 1011
  tm run --standard 1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA --format json
  cat machine.tm | tm run - --input abba

Single-tape machines run with chain steps, which skip over whole runs of equal symbols, so long
runs such as the 5-state busy beaver finish in milliseconds.";

/// Arguments of `tm run`.
#[derive(clap::Args, Debug)]
#[command(after_help = AFTER_HELP)]
pub(crate) struct Args {
    #[command(flatten)]
    machine: MachineArg,

    #[command(flatten)]
    input: InputArgs,

    /// The step budget, for example 1000, 1e9 or 10^12
    #[arg(long, value_name = "N", default_value = "1e9", value_parser = parse_count)]
    max_steps: u64,

    /// How many words an enumerator may print before the run stops
    #[arg(long, value_name = "N", default_value_t = 20)]
    max_words: usize,

    /// What to print: a report, JSON, or only the result. Defaults to `value` with `--unary`
    /// or `--binary` and to `text` otherwise
    #[arg(long, value_enum, value_name = "FORMAT")]
    format: Option<Format>,
}

/// The output format of `tm run`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum Format {
    /// A report for people
    Text,
    /// A JSON object with exact numbers
    Json,
    /// Only the result: the decoded number or word, `accept` or `reject`, or the words of an
    /// enumerator
    Value,
}

/// Everything a run produced.
struct Run<'m> {
    machine: &'m Machine,
    input: String,
    outcome: Outcome,
    output: Output,
    words: Vec<String>,
    non_blank: u64,
    configuration: String,
    elapsed: Duration,
}

impl Run<'_> {
    fn halted(&self) -> bool {
        self.outcome.is_halted()
    }

    /// `Some(true)` if an acceptor halted in a final state.
    fn accepted(&self) -> Option<bool> {
        match self.outcome {
            Outcome::Halted { state, .. } if self.machine.kind() == Kind::Acceptor => {
                Some(self.machine.is_final(state))
            }
            _ => None,
        }
    }
}

/// Runs `tm run`.
pub(crate) fn exec(args: &Args) -> Result<()> {
    let mut out = anstream::stdout().lock();
    let mut err = anstream::stderr().lock();
    let loaded = load::load(&args.machine, &mut err)?;
    let machine = &loaded.machine;
    let input = input::prepare(machine, &args.input)?;
    let format = args.format.unwrap_or(if args.input.is_numeric() {
        Format::Value
    } else {
        Format::Text
    });

    let start = Instant::now();
    let mut engine = Engine::new(machine, &input.symbols);
    let mut words = Vec::new();
    let outcome = if machine.kind() == Kind::Enumerator && args.max_words > 0 {
        loop {
            let fuel = args.max_steps.saturating_sub(engine.steps());
            let outcome = engine.run_to_output(fuel);
            if !matches!(outcome, Outcome::Stopped { .. }) {
                break outcome;
            }
            words.push(engine.output().word);
            if words.len() >= args.max_words {
                break outcome;
            }
        }
    } else {
        engine.run(args.max_steps)
    };
    let elapsed = start.elapsed();

    let run = Run {
        machine,
        input: input.word,
        outcome,
        output: engine.output(),
        words,
        non_blank: engine.non_blank(),
        configuration: engine.configuration(60),
        elapsed,
    };
    match format {
        Format::Text => write_text(&mut out, &loaded.name(), &run, args)?,
        Format::Json => write_json(&mut out, &loaded.name(), &run)?,
        Format::Value => write_value(&mut out, &run, args)?,
    }
    Ok(())
}

fn write_text(out: &mut dyn Write, name: &str, run: &Run<'_>, args: &Args) -> Result<()> {
    let m = run.machine;
    report::field(out, "machine", format!("{name} ({})", cmd::shape(m)))?;
    if m.kind() != Kind::Beaver || !run.input.is_empty() {
        report::field(out, "input", text::word(&run.input))?;
    }
    let outcome = cmd::outcome_text(m, run.outcome);
    let outcome = match run.outcome {
        Outcome::Halted { .. } => paint(style::GOOD, outcome),
        Outcome::OutOfFuel { .. } => format!(
            "{} {}",
            paint(style::WARN, outcome),
            paint(style::DIM, "(raise --max-steps to run longer)")
        ),
        Outcome::NeverHalts { .. } => paint(style::NOTE, outcome),
        Outcome::Stopped { .. } => format!(
            "{} {}",
            paint(
                style::GOOD,
                format!(
                    "printed {} in {}",
                    text::count(run.words.len() as u64, "word", "words"),
                    text::count(run.outcome.steps(), "step", "steps")
                )
            ),
            paint(style::DIM, "(raise --max-words for more)")
        ),
    };
    report::field(out, "outcome", outcome)?;
    match m.kind() {
        Kind::Acceptor => {
            let verdict = match run.accepted() {
                Some(true) => paint(style::GOOD, "accepted"),
                Some(false) => paint(style::BAD, "rejected"),
                None => paint(style::WARN, "none, the machine has not halted"),
            };
            report::field(out, "verdict", verdict)?;
        }
        Kind::Transducer if run.halted() => {
            report::field(out, "output", output_text(&run.output))?;
        }
        Kind::Enumerator => {
            let mut words = run.words.iter();
            match words.next() {
                Some(first) => report::field(out, "printed", text::word(first))?,
                None => report::field(out, "printed", paint(style::DIM, "nothing"))?,
            }
            for w in words {
                report::continuation(out, text::word(w))?;
            }
            if run.words.len() >= args.max_words && args.max_words > 0 {
                report::continuation(out, paint(style::DIM, "..."))?;
            }
        }
        Kind::Transducer | Kind::Beaver => {}
    }
    report::field(out, "tape", &run.configuration)?;
    report::field(out, "non-blank", text::num(run.non_blank))?;
    if text::timing_enabled() {
        report::field(out, "time", text::duration(run.elapsed))?;
    }
    Ok(())
}

/// `xxxxxx (6)`, or the word alone if it has no numeric value.
fn output_text(output: &Output) -> String {
    match output.value {
        Some(v) => format!("{} ({})", text::word(&output.word), v),
        None => text::word(&output.word).to_owned(),
    }
}

fn outcome_json(run: &Run<'_>) -> Value {
    let m = run.machine;
    match run.outcome {
        Outcome::Halted { steps, state } => json!({
            "status": "halted",
            "steps": steps,
            "state": cmd::state_name(m, state),
        }),
        Outcome::OutOfFuel { steps } => json!({ "status": "outOfFuel", "steps": steps }),
        Outcome::NeverHalts { steps, proof } => json!({
            "status": "neverHalts",
            "steps": steps,
            "proof": cmd::proof_text(m, proof),
        }),
        Outcome::Stopped { steps } => json!({ "status": "stopped", "steps": steps }),
    }
}

/// A `u128` as a JSON number if it fits into `u64`, otherwise as a string of digits.
fn big(v: u128) -> Value {
    u64::try_from(v).map_or_else(|_| Value::String(v.to_string()), Value::from)
}

fn write_json(out: &mut dyn Write, name: &str, run: &Run<'_>) -> Result<()> {
    let m = run.machine;
    let mut v = json!({
        "machine": name,
        "kind": cmd::kind_name(m.kind()),
        "tapes": m.tapes(),
        "states": m.num_states(),
        "input": run.input,
        "outcome": outcome_json(run),
        "nonBlank": run.non_blank,
        "configuration": run.configuration,
    });
    match m.kind() {
        Kind::Acceptor => v["accepted"] = run.accepted().map_or(Value::Null, Value::from),
        Kind::Transducer if run.halted() => {
            v["output"] = json!({
                "word": run.output.word,
                "value": run.output.value.map_or(Value::Null, big),
            });
        }
        Kind::Enumerator => v["printed"] = json!(run.words),
        Kind::Transducer | Kind::Beaver => {}
    }
    if text::timing_enabled() {
        v["seconds"] = json!(run.elapsed.as_secs_f64());
    }
    serde_json::to_writer_pretty(&mut *out, &v).map_err(std::io::Error::from)?;
    writeln!(out)?;
    Ok(())
}

fn write_value(out: &mut dyn Write, run: &Run<'_>, args: &Args) -> Result<()> {
    let m = run.machine;
    if m.kind() == Kind::Enumerator {
        for w in &run.words {
            writeln!(out, "{}", text::word(w))?;
        }
        return Ok(());
    }
    if !run.halted() {
        let why = cmd::outcome_text(m, run.outcome);
        let hint = if matches!(run.outcome, Outcome::OutOfFuel { .. }) {
            format!(" (the budget is --max-steps {})", text::num(args.max_steps))
        } else {
            String::new()
        };
        return Err(Error::failed(format!("no result: the machine {why}{hint}")));
    }
    match m.kind() {
        Kind::Acceptor => {
            let verdict = if run.accepted() == Some(true) {
                "accept"
            } else {
                "reject"
            };
            writeln!(out, "{verdict}")?;
        }
        Kind::Beaver => writeln!(out, "{}", run.non_blank)?,
        Kind::Transducer | Kind::Enumerator => match run.output.value {
            Some(v) => writeln!(out, "{v}")?,
            None => writeln!(out, "{}", text::word(&run.output.word))?,
        },
    }
    Ok(())
}
