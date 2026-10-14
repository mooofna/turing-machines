//! One interface over the two simulators: chain steps for single-tape machines, the dense
//! reference simulator for machines with several tapes.

use turing_core::config_text::configuration;
use turing_core::{ChainSim, Machine, Outcome, Output, Simulator, State, Symbol, TapeView, decode};

/// A run of a machine.
#[derive(Debug)]
pub(crate) struct Engine<'m> {
    machine: &'m Machine,
    finals: Vec<bool>,
    sim: Sim<'m>,
}

#[derive(Debug)]
enum Sim<'m> {
    /// Single-tape machines, with chain steps.
    Chain(ChainSim<&'m Machine>),
    /// Machines with several tapes.
    Dense(Simulator<&'m Machine>),
}

impl<'m> Engine<'m> {
    /// Starts `machine` on `input`.
    pub(crate) fn new(machine: &'m Machine, input: &[Symbol]) -> Self {
        let sim = match ChainSim::new(machine, input) {
            Ok(sim) => Sim::Chain(sim),
            Err(_) => Sim::Dense(Simulator::new(machine, input)),
        };
        Self {
            machine,
            finals: machine.states().iter().map(|q| q.is_final).collect(),
            sim,
        }
    }

    /// Steps so far.
    pub(crate) fn steps(&self) -> u64 {
        match &self.sim {
            Sim::Chain(s) => s.steps(),
            Sim::Dense(s) => s.steps(),
        }
    }

    /// Runs at most `fuel` more steps.
    pub(crate) fn run(&mut self, fuel: u64) -> Outcome {
        match &mut self.sim {
            Sim::Chain(s) => s.run(fuel),
            Sim::Dense(s) => s.run(fuel),
        }
    }

    /// Runs at most `fuel` more steps and stops right after a step into a final state, with
    /// [`Outcome::Stopped`]. This is how enumerators emit words.
    pub(crate) fn run_to_output(&mut self, fuel: u64) -> Outcome {
        let finals = &self.finals;
        let is_final = |q: State| finals.get(q.index()).copied().unwrap_or(false);
        match &mut self.sim {
            Sim::Dense(s) => s.run_until(fuel, |ev| is_final(ev.action.next)),
            Sim::Chain(s) => {
                let before = s.steps();
                let outcome = s.run_until(fuel, |ev| is_final(ev.action.next));
                if !matches!(outcome, Outcome::Stopped { .. }) {
                    return outcome;
                }
                // The chain simulator stops before the step into the final state; make it.
                if s.steps() - before >= fuel {
                    return Outcome::OutOfFuel { steps: s.steps() };
                }
                let at = s.steps();
                let outcome = s.run(1);
                if s.steps() > at {
                    Outcome::Stopped { steps: s.steps() }
                } else {
                    outcome
                }
            }
        }
    }

    /// The output read from the first tape.
    pub(crate) fn output(&self) -> Output {
        match &self.sim {
            Sim::Chain(s) => decode(self.machine, s.tape()),
            Sim::Dense(s) => decode(self.machine, s.tape(0)),
        }
    }

    /// The number of non-blank cells on all tapes.
    pub(crate) fn non_blank(&self) -> u64 {
        match &self.sim {
            Sim::Chain(s) => s.tape().count_non_blank(),
            Sim::Dense(s) => s.tapes().iter().map(TapeView::count_non_blank).sum(),
        }
    }

    /// The configuration in course notation, tapes separated by ` | `.
    pub(crate) fn configuration(&self, max_cells: usize) -> String {
        match &self.sim {
            Sim::Chain(s) => configuration(self.machine, s.state(), s.tape(), max_cells),
            Sim::Dense(s) => s
                .tapes()
                .iter()
                .map(|t| configuration(self.machine, s.state(), t, max_cells))
                .collect::<Vec<_>>()
                .join(" | "),
        }
    }
}
