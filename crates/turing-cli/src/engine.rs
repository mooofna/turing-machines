//! One interface over the two simulators: chain steps for single-tape machines, the dense
//! reference simulator for machines with several tapes and for enumerators.

use turing_core::config_text::configuration;
use turing_core::{
    ChainSim, Kind, Machine, NonHaltProof, Outcome, Output, Simulator, State, Symbol, TapeView,
    decode,
};

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
    /// Machines with several tapes, and enumerators, which must stop after single steps.
    Dense(Simulator<&'m Machine>),
}

impl<'m> Engine<'m> {
    /// Starts `machine` on `input`.
    pub(crate) fn new(machine: &'m Machine, input: &[Symbol]) -> Self {
        let chain = if machine.kind() == Kind::Enumerator {
            None
        } else {
            ChainSim::new(machine, input).ok()
        };
        let sim = match chain {
            Some(sim) => Sim::Chain(sim),
            None => Sim::Dense(Simulator::new(machine, input)),
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
    /// [`Outcome::Stopped`]. This is how enumerators print words.
    ///
    /// A proved endless loop inside a final state prints on every step, so it does not end the
    /// run: the step is made and reported like any other.
    pub(crate) fn run_to_output(&mut self, fuel: u64) -> Outcome {
        let finals = &self.finals;
        let is_final = |q: State| finals.get(q.index()).copied().unwrap_or(false);
        let s = match &mut self.sim {
            Sim::Dense(s) => s,
            Sim::Chain(s) => return s.run(fuel),
        };
        let before = s.steps();
        let outcome = s.run_until(fuel, |ev| is_final(ev.action.next));
        let looping = match outcome {
            Outcome::NeverHalts {
                proof:
                    NonHaltProof::StayLoop { state } | NonHaltProof::ChainToInfinity { state, .. },
                ..
            } => state,
            other => return other,
        };
        if s.steps() - before >= fuel || !is_final(looping) {
            return outcome;
        }
        s.step();
        Outcome::Stopped { steps: s.steps() }
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

#[cfg(test)]
mod tests {
    use turing_core::{Decoder, Dir};

    use super::*;

    /// Prints x, xx, xxx, ...: every step into the final state `p` prints the tape.
    fn tallies() -> Machine {
        Machine::builder("tallies")
            .kind(Kind::Enumerator)
            .transition("q0", '␣', "p", 'x', Dir::R)
            .transition("p", '␣', "p", 'x', Dir::R)
            .final_state("p")
            .build()
            .unwrap()
    }

    #[test]
    fn enumerators_print_through_proved_loops() {
        let m = tallies();
        let mut engine = Engine::new(&m, &[]);
        let mut words = Vec::new();
        for _ in 0..4 {
            let outcome = engine.run_to_output(100);
            assert!(matches!(outcome, Outcome::Stopped { .. }), "{outcome:?}");
            words.push(engine.output().word);
        }
        assert_eq!(words, ["x", "xx", "xxx", "xxxx"]);
        assert_eq!(engine.steps(), 4);
    }

    #[test]
    fn enumerators_respect_the_budget() {
        let m = tallies();
        let mut engine = Engine::new(&m, &[]);
        assert!(matches!(
            engine.run_to_output(0),
            Outcome::OutOfFuel { steps: 0 }
        ));
    }

    #[test]
    fn single_tape_machines_use_chain_steps() {
        let m = Machine::builder("unary")
            .decoder(Decoder::Unary { symbol: 'x' })
            .transition("q0", 'x', "q0", 'x', Dir::R)
            .build()
            .unwrap();
        let input = m.encode_input("xxxx").unwrap();
        let mut engine = Engine::new(&m, &input);
        assert!(matches!(engine.sim, Sim::Chain(_)));
        assert!(matches!(engine.run(100), Outcome::Halted { steps: 4, .. }));
        assert_eq!(engine.output().value, Some(4));
        assert_eq!(engine.non_blank(), 4);
        assert_eq!(engine.configuration(40), "xxxx q0 ␣");
    }
}
