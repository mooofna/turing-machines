//! `tm antihydra`: the cryptid Antihydra, checked on the machine and iterated as a rule.

use std::io::Write;
use std::time::Instant;

use turing_bb::cryptids::antihydra::{MachineCheck, Row, RuleWalk, STANDARD, WalkSummary};

use crate::cmd::parse_count;
use crate::error::{Error, Result};
use crate::report::{self, paint, style};
use crate::table::{Align, Cell, Table};
use crate::text;

const AFTER_HELP: &str = "\
Antihydra is the 6-state machine 1RB1RA_0LC1LE_1LD1LC_1LA0LB_1LF1RE_---0RA, found by mxdys in
2024. Write A(a, b) for its configuration 0^∞ 1^a 0 1^b E> 0^∞. From A(0, 4) it follows
A(a, 2b) -> A(a + 2, 3b + 2) and A(a + 1, 2b + 1) -> A(a, 3b + 3), and A(0, 2b + 1) halts. With
h = b + 4 and the balance c = a, every even h adds 2 to c and every odd h takes 1 away, and h
grows to h + floor(h / 2). The machine halts exactly if c ever becomes negative. Nobody knows
whether it does; the question is close to Mahler's 3/2 problem.

Examples:
  tm antihydra
  tm antihydra --iterations 1e6 --check-rows 20";

/// Arguments of `tm antihydra`.
#[derive(clap::Args, Debug)]
#[command(after_help = AFTER_HELP)]
pub(crate) struct Args {
    /// How often to iterate the rule
    #[arg(long, value_name = "N", default_value = "100000", value_parser = parse_count)]
    iterations: u64,

    /// How many rule applications to check on the machine itself
    #[arg(long, value_name = "K", default_value_t = 30)]
    check_rows: usize,
}

/// Collects the first `rows` rule applications on the machine.
fn check_machine(rows: usize) -> Vec<Row> {
    let mut check = MachineCheck::new();
    let mut found = Vec::new();
    while found.len() < rows && !check.is_done() {
        found.extend(check.advance(1 << 16));
    }
    found.truncate(rows);
    found
}

/// Writes the table of rule applications observed on the machine.
fn write_rows(out: &mut dyn Write, rows: &[Row]) -> std::io::Result<()> {
    writeln!(
        out,
        "{}",
        paint(
            style::KEY,
            "Rule applications on the machine, A(a, b) = 0^∞ 1^a 0 1^b E> 0^∞"
        )
    )?;
    writeln!(out)?;
    let mut table = Table::new(&[
        ("n", Align::Right),
        ("a", Align::Right),
        ("b", Align::Right),
        ("step", Align::Right),
        ("steps since", Align::Right),
        ("rule says", Align::Right),
        ("agrees", Align::Left),
    ])
    .indent(2);
    for row in rows {
        table.row([
            Cell::from(text::num(row.iteration)),
            Cell::from(text::num(row.a)),
            Cell::from(text::num(row.b)),
            Cell::from(text::num(row.steps)),
            Cell::from(text::num(row.delta)),
            Cell::from(text::num(row.predicted)),
            if row.agrees {
                Cell::styled("yes", style::GOOD)
            } else {
                Cell::styled("no", style::BAD)
            },
        ]);
    }
    table.write(out)?;
    writeln!(out)
}

/// Writes the summary of the rule walk.
fn write_walk(out: &mut dyn Write, summary: &WalkSummary) -> std::io::Result<()> {
    report::field(
        out,
        "rule",
        format!(
            "{} from h = 8, c = 0",
            text::count(summary.iterations, "iteration", "iterations")
        ),
    )?;
    report::continuation(
        out,
        format!(
            "h even {} times (c + 2), odd {} times (c - 1)",
            text::num(summary.evens),
            text::num(summary.odds)
        ),
    )?;
    report::continuation(
        out,
        format!(
            "balance c = {}, smallest {}",
            text::signed(summary.balance),
            text::signed(summary.min_balance)
        ),
    )?;
    report::continuation(
        out,
        format!("h has {}", text::count(summary.bits, "bit", "bits")),
    )
}

/// Writes what the walk shows, and what it does not.
fn write_conclusion(out: &mut dyn Write, summary: &WalkSummary) -> std::io::Result<()> {
    let iterations = text::count(summary.iterations, "iteration", "iterations");
    if summary.halted {
        let line = format!("The balance became negative within {iterations}: Antihydra halts.");
        writeln!(out, "{}", paint(style::NOTE, line))
    } else {
        writeln!(
            out,
            "The balance did not become negative in {iterations}. Whether it ever does is an open"
        )?;
        writeln!(
            out,
            "problem, so nobody knows whether Antihydra halts. The question is close to Mahler's"
        )?;
        writeln!(out, "3/2 problem.")
    }
}

/// Runs `tm antihydra`.
pub(crate) fn exec(args: &Args) -> Result<()> {
    let mut out = anstream::stdout().lock();
    report::field(&mut out, "machine", STANDARD)?;
    writeln!(out)?;

    let start = Instant::now();
    let rows = check_machine(args.check_rows);
    if !rows.is_empty() {
        write_rows(&mut out, &rows)?;
    }
    let agreeing = rows.iter().filter(|r| r.agrees).count();
    let check_line = format!(
        "{} of {} on the machine agree with the rule",
        text::num(agreeing as u64),
        text::count(rows.len() as u64, "rule application", "rule applications"),
    );
    let check_style = if agreeing == rows.len() {
        style::GOOD
    } else {
        style::BAD
    };
    report::field(&mut out, "check", paint(check_style, check_line))?;

    let mut walk = RuleWalk::new();
    walk.advance(args.iterations);
    let summary = walk.summary(0);
    write_walk(&mut out, &summary)?;
    if text::timing_enabled() {
        report::field(&mut out, "time", text::duration(start.elapsed()))?;
    }
    writeln!(out)?;
    write_conclusion(&mut out, &summary)?;

    if agreeing != rows.len() {
        return Err(Error::failed(
            "the machine does not follow the rule in every checked row",
        ));
    }
    Ok(())
}
