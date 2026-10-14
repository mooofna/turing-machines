//! `tm enumerate`: the busy beaver search over all machines of one size.

use std::io::{IsTerminal, Write};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use turing_bb::{CompactTm, Enumerator, Stats};

use crate::error::{Error, Result};
use crate::report::{self, paint, style};
use crate::table::{Align, Cell, Table};
use crate::text;

const AFTER_HELP: &str = "\
The machines are enumerated in tree normal form, as in the Coq-BB5 proof: a partial machine is
run until it reaches an undefined transition, which is then filled in every way that can make a
difference. Every node of this tree either halts or is proved non-halting by one of the deciders,
or it remains as a holdout. S(n) is the largest number of steps and Σ(n) the largest number of
ones that a halting machine leaves.

Examples:
  tm enumerate --states 3
  tm enumerate --states 4 --holdouts
  tm enumerate --states 2 --symbols 3 --format json";

/// Arguments of `tm enumerate`.
#[derive(clap::Args, Debug)]
#[command(after_help = AFTER_HELP)]
pub(crate) struct Args {
    /// The number of states
    #[arg(long, value_name = "N")]
    states: u8,

    /// The number of symbols, blank included
    #[arg(long, value_name = "K", default_value_t = 2)]
    symbols: u8,

    /// List the machines that no decider resolved
    #[arg(long)]
    holdouts: bool,

    /// Show a progress line on standard error even if it is not a terminal (on a terminal it
    /// is always shown)
    #[arg(long)]
    progress: bool,

    /// What to print
    #[arg(long, value_enum, value_name = "FORMAT", default_value_t = Format::Text)]
    format: Format,
}

/// The output format of `tm enumerate`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
enum Format {
    /// A funnel table for people
    Text,
    /// A JSON object with exact numbers
    Json,
}

/// Draws the progress line on standard error.
struct Progress {
    enabled: bool,
    terminal: bool,
    last: Instant,
    interval: Duration,
    drawn: bool,
}

impl Progress {
    fn new(forced: bool) -> Self {
        let terminal = std::io::stderr().is_terminal();
        Self {
            enabled: forced || terminal,
            terminal,
            last: Instant::now(),
            interval: if terminal {
                Duration::from_millis(100)
            } else {
                Duration::from_secs(1)
            },
            drawn: false,
        }
    }

    fn update(&mut self, label: &str, stats: &Stats, start: Instant) {
        if !self.enabled || self.last.elapsed() < self.interval {
            return;
        }
        self.last = Instant::now();
        let line = format!(
            "enumerating {label}: {}, {} so far, {}",
            text::count(stats.nodes, "node", "nodes"),
            text::count(stats.holdouts, "holdout", "holdouts"),
            text::duration(start.elapsed()),
        );
        let mut err = std::io::stderr().lock();
        // Progress is best effort: a closed standard error must not stop the enumeration.
        let _ = if self.terminal {
            write!(err, "\r{line}\x1b[K")
        } else {
            writeln!(err, "{line}")
        };
        let _ = err.flush();
        self.drawn = true;
    }

    fn finish(&self) {
        if self.drawn && self.terminal {
            let mut err = std::io::stderr().lock();
            let _ = write!(err, "\r\x1b[K");
            let _ = err.flush();
        }
    }
}

/// Runs `tm enumerate`.
pub(crate) fn exec(args: &Args) -> Result<()> {
    let mut enumerator =
        Enumerator::new(args.states, args.symbols).map_err(|e| Error::usage(e.to_string()))?;
    let start = Instant::now();
    let mut progress = Progress::new(args.progress);
    let label = format!("{}-state machines", args.states);
    while !enumerator.is_done() {
        enumerator.advance(1 << 10);
        progress.update(&label, enumerator.stats(), start);
    }
    progress.finish();
    let elapsed = start.elapsed();

    let mut out = anstream::stdout().lock();
    let stats = enumerator.stats();
    match args.format {
        Format::Text => write_text(&mut out, args, stats, enumerator.holdouts(), elapsed)?,
        Format::Json => write_json(&mut out, args, stats, enumerator.holdouts(), elapsed)?,
    }
    Ok(())
}

/// The label with a lower-case first letter, for use inside a sentence.
pub(crate) fn lower_first(label: &str) -> String {
    let mut chars = label.chars();
    chars
        .next()
        .map_or_else(String::new, |c| c.to_lowercase().chain(chars).collect())
}

/// `S(4)` for two symbols, `S(2, 3)` otherwise.
fn function(name: &str, states: u8, symbols: u8) -> String {
    if symbols == 2 {
        format!("{name}({states})")
    } else {
        format!("{name}({states}, {symbols})")
    }
}

fn write_text(
    out: &mut dyn Write,
    args: &Args,
    stats: &Stats,
    holdouts: &[CompactTm],
    elapsed: Duration,
) -> Result<()> {
    writeln!(
        out,
        "{}",
        paint(
            style::KEY,
            format!(
                "Busy beaver search over {} and {}, in tree normal form",
                text::count(u64::from(args.states), "state", "states"),
                text::count(u64::from(args.symbols), "symbol", "symbols"),
            )
        )
    )?;
    writeln!(out)?;

    let mut table = Table::new(&[
        ("stage", Align::Left),
        ("resolved", Align::Right),
        ("remaining", Align::Right),
    ]);
    let mut remaining = stats.nodes;
    table.row([
        Cell::from("nodes of the tree normal form"),
        Cell::default(),
        Cell::from(text::num(remaining)),
    ]);
    remaining = remaining.saturating_sub(stats.halting);
    table.row([
        Cell::from("halting"),
        Cell::from(text::num(stats.halting)),
        Cell::from(text::num(remaining)),
    ]);
    for &(id, count) in &stats.by_decider {
        remaining = remaining.saturating_sub(count);
        table.row([
            Cell::from(format!("non-halting: {}", lower_first(id.label()))),
            Cell::from(text::num(count)),
            Cell::from(text::num(remaining)),
        ]);
    }
    let holdout_style = if stats.holdouts == 0 {
        style::GOOD
    } else {
        style::WARN
    };
    table.row([
        Cell::styled("holdouts", holdout_style),
        Cell::default(),
        Cell::styled(text::num(stats.holdouts), holdout_style),
    ]);
    table.write(out)?;
    writeln!(out)?;

    let relation = if stats.holdouts == 0 { "=" } else { "≥" };
    let best = [
        ("S", stats.max_steps.as_ref()),
        ("Σ", stats.max_ones.as_ref()),
    ];
    let width = best
        .iter()
        .filter_map(|(_, b)| b.map(|(v, _)| text::num(*v).len()))
        .max()
        .unwrap_or(0);
    for (name, best) in best {
        let f = paint(style::KEY, function(name, args.states, args.symbols));
        match best {
            Some((value, tm)) => writeln!(
                out,
                "{f} {relation} {:<width$}   {}",
                text::num(*value),
                paint(style::DIM, tm)
            )?,
            None => writeln!(out, "{f}: no halting machine found")?,
        }
    }
    if stats.holdouts > 0 {
        writeln!(
            out,
            "{}",
            paint(
                style::DIM,
                format!(
                    "These are lower bounds: {} could still halt later.",
                    text::count(stats.holdouts, "holdout", "holdouts")
                )
            )
        )?;
    }
    if args.holdouts && !holdouts.is_empty() {
        writeln!(out)?;
        report::field(out, "holdouts", holdouts[0])?;
        for tm in &holdouts[1..] {
            report::continuation(out, tm)?;
        }
    }
    if text::timing_enabled() {
        writeln!(out)?;
        report::field(out, "time", text::duration(elapsed))?;
    }
    Ok(())
}

fn write_json(
    out: &mut dyn Write,
    args: &Args,
    stats: &Stats,
    holdouts: &[CompactTm],
    elapsed: Duration,
) -> Result<()> {
    let best = |b: Option<&(u64, CompactTm)>| match b {
        Some((value, tm)) => json!({ "value": value, "machine": tm.to_string() }),
        None => Value::Null,
    };
    let mut v = json!({
        "states": args.states,
        "symbols": args.symbols,
        "nodes": stats.nodes,
        "halting": stats.halting,
        "deciders": stats.by_decider.iter().map(|(id, count)| json!({
            "key": id.key(),
            "label": id.label(),
            "count": count,
        })).collect::<Vec<_>>(),
        "holdouts": stats.holdouts,
        "maxSteps": best(stats.max_steps.as_ref()),
        "maxOnes": best(stats.max_ones.as_ref()),
    });
    if args.holdouts {
        v["holdoutMachines"] = json!(holdouts.iter().map(ToString::to_string).collect::<Vec<_>>());
    }
    if text::timing_enabled() {
        v["seconds"] = json!(elapsed.as_secs_f64());
    }
    serde_json::to_writer_pretty(&mut *out, &v).map_err(std::io::Error::from)?;
    writeln!(out)?;
    Ok(())
}
