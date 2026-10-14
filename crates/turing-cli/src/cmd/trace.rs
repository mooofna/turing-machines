//! `tm trace`: the configurations of a run in course notation.

use std::fmt::Write as _;
use std::io::Write;

use turing_core::config_text::configuration;
use turing_core::{Kind, Machine, Outcome, Simulator, Status, decode};

use crate::cmd::{self, parse_count};
use crate::error::Result;
use crate::input::{self, InputArgs};
use crate::load::{self, MachineArg};
use crate::report::{paint, style};
use crate::text;

const AFTER_HELP: &str = "\
Each line is a configuration u q v: u is the tape left of the head, q the state and v the tape
from the head on, as in the derivations k0 ⊢ k1 ⊢ ... of the course. Machines with several tapes
show one configuration per tape, separated by |.

Examples:
  tm trace machines/course/binary-increment.tm --input 1011
  tm trace machines/course/unary-multiplication.tm --unary 2 3 --steps 200";

/// Arguments of `tm trace`.
#[derive(clap::Args, Debug)]
#[command(after_help = AFTER_HELP)]
pub(crate) struct Args {
    #[command(flatten)]
    machine: MachineArg,

    #[command(flatten)]
    input: InputArgs,

    /// The largest number of steps to show
    #[arg(short = 'n', long, value_name = "N", default_value = "50", value_parser = parse_count)]
    steps: u64,

    /// Cut configurations longer than this many cells around the head
    #[arg(long, value_name = "CELLS", default_value_t = 72)]
    width: usize,
}

fn line(machine: &Machine, sim: &Simulator<&Machine>, width: usize) -> String {
    sim.tapes()
        .iter()
        .map(|t| configuration(machine, sim.state(), t, width))
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Runs `tm trace`.
pub(crate) fn exec(args: &Args) -> Result<()> {
    let mut out = anstream::stdout().lock();
    let mut err = anstream::stderr().lock();
    let loaded = load::load(&args.machine, &mut err)?;
    let m = &loaded.machine;
    let input = input::prepare(m, &args.input)?;
    let mut sim = Simulator::new(m, &input.symbols);
    let digits = text::num(args.steps).len();
    let number = |step: u64| paint(style::DIM, format!("{:>digits$}", text::num(step)));

    let mut first = format!("{}    {}", number(0), line(m, &sim, args.width));
    if m.kind() == Kind::Enumerator && m.is_final(m.start()) {
        let word = decode(m, sim.tape(0)).word;
        first.push_str(&paint(
            style::DIM,
            format!("   prints {}", text::word(&word)),
        ));
    }
    writeln!(out, "{first}")?;
    let mut halted = false;
    while sim.steps() < args.steps {
        let next = sim.next_action();
        if sim.step() == Status::Halted {
            halted = true;
            break;
        }
        let mut row = format!("{} ⊢  {}", number(sim.steps()), line(m, &sim, args.width));
        if m.kind() == Kind::Enumerator && next.is_some_and(|a| m.is_final(a.next)) {
            let word = decode(m, sim.tape(0)).word;
            row.push_str(&paint(
                style::DIM,
                format!("   prints {}", text::word(&word)),
            ));
        }
        writeln!(out, "{row}")?;
    }
    if !halted && sim.next_action().is_none() {
        halted = true;
    }

    writeln!(out)?;
    if halted {
        let outcome = Outcome::Halted {
            steps: sim.steps(),
            state: sim.state(),
        };
        let mut summary = paint(style::GOOD, cmd::outcome_text(m, outcome));
        // Writing to a `String` cannot fail.
        let _ = match m.kind() {
            Kind::Acceptor if m.is_final(sim.state()) => {
                write!(summary, ", {}", paint(style::GOOD, "accepted"))
            }
            Kind::Acceptor => write!(summary, ", {}", paint(style::BAD, "rejected")),
            Kind::Transducer => {
                let output = decode(m, sim.tape(0));
                match output.value {
                    Some(v) => write!(summary, ", output {} ({v})", text::word(&output.word)),
                    None => write!(summary, ", output {}", text::word(&output.word)),
                }
            }
            Kind::Enumerator | Kind::Beaver => Ok(()),
        };
        writeln!(out, "{summary}")?;
    } else {
        writeln!(
            out,
            "{} {}",
            paint(
                style::WARN,
                format!(
                    "not halted after {}",
                    text::count(sim.steps(), "step", "steps")
                )
            ),
            paint(style::DIM, "(use --steps to see more)")
        )?;
    }
    Ok(())
}
