//! Differential tests: on random machines, the dense simulator, the chain simulator and a
//! deliberately naive reference written here must agree step for step.

use std::collections::BTreeMap;

use proptest::prelude::*;
use turing_core::{
    ChainSim, Dir, Engine, Machine, NonHaltProof, Outcome, Simulator, State, Symbol, TapeView,
    Timeline,
};

/// One defined entry of δ, with one written symbol and move per tape.
#[derive(Clone, Debug)]
struct Entry {
    write: Vec<u8>,
    dir: Vec<Dir>,
    next: usize,
}

/// A random machine as plain data, independent of the crate's table layout.
#[derive(Clone, Debug)]
struct Random {
    tapes: usize,
    states: usize,
    symbols: u8,
    /// Indexed by `state * stride` plus the read tuple with tape 0 varying fastest.
    delta: Vec<Option<Entry>>,
    /// The number of read tuples, `symbols ^ tapes`.
    stride: usize,
    input: Vec<u8>,
}

/// The state with index `q`.
fn state(q: usize) -> State {
    State::new(u16::try_from(q).expect("few states"))
}

impl Random {
    fn entry(&self, state: usize, read: &[u8]) -> Option<&Entry> {
        let symbols = usize::from(self.symbols);
        let column = read
            .iter()
            .rev()
            .fold(0, |acc, &s| acc * symbols + usize::from(s));
        self.delta[state * self.stride + column].as_ref()
    }

    fn glyph(symbol: u8) -> char {
        char::from(b'0' + symbol)
    }

    /// The read tuple of table column `column`.
    fn read_of(&self, column: usize) -> Vec<u8> {
        let symbols = usize::from(self.symbols);
        (0..self.tapes)
            .scan(column, |rest, _| {
                let symbol = *rest % symbols;
                *rest /= symbols;
                Some(u8::try_from(symbol).expect("a symbol index"))
            })
            .collect()
    }

    fn build(&self) -> Machine {
        let names: Vec<String> = (0..self.states).map(|q| format!("S{q}")).collect();
        let mut builder = Machine::builder("random")
            .tapes(self.tapes)
            .blank('0')
            .tape_alphabet((0..self.symbols).map(Random::glyph))
            .declare_states(names.iter().map(String::as_str));
        for (index, entry) in self.delta.iter().enumerate() {
            let Some(entry) = entry else { continue };
            let read: Vec<char> = self
                .read_of(index % self.stride)
                .into_iter()
                .map(Random::glyph)
                .collect();
            let write: Vec<char> = entry.write.iter().copied().map(Random::glyph).collect();
            builder = builder.transition_multi(
                &names[index / self.stride],
                &read,
                &names[entry.next],
                &write,
                &entry.dir,
            );
        }
        builder.build().expect("random machines are valid")
    }

    fn input(&self) -> Vec<Symbol> {
        self.input.iter().copied().map(Symbol::new).collect()
    }
}

fn dir() -> impl Strategy<Value = Dir> {
    prop_oneof![Just(Dir::L), Just(Dir::R), Just(Dir::N)]
}

/// Machines with 1 to `max_states` states, 2 to `max_symbols` symbols and `tapes` tapes, where
/// about one entry in eight is undefined.
fn machines(tapes: usize, max_states: usize, max_symbols: u8) -> impl Strategy<Value = Random> {
    (1..=max_states, 2..=max_symbols).prop_flat_map(move |(states, symbols)| {
        let entry = (
            prop::collection::vec(0..symbols, tapes),
            prop::collection::vec(dir(), tapes),
            0..states,
        )
            .prop_map(|(write, dir, next)| Entry { write, dir, next });
        let stride = (0..tapes).fold(1, |acc, _| acc * usize::from(symbols));
        (
            prop::collection::vec(prop::option::weighted(0.875, entry), states * stride),
            prop::collection::vec(0..symbols, 0..8),
        )
            .prop_map(move |(delta, input)| Random {
                tapes,
                states,
                symbols,
                delta,
                stride,
                input,
            })
    })
}

/// The naive reference: sparse tapes in ordered maps, one step at a time, nothing clever.
struct Reference<'a> {
    machine: &'a Random,
    /// Only non-blank cells are stored.
    tapes: Vec<BTreeMap<i64, u8>>,
    heads: Vec<i64>,
    visited: Vec<(i64, i64)>,
    state: usize,
    steps: u64,
}

impl<'a> Reference<'a> {
    fn new(machine: &'a Random) -> Self {
        let mut tapes = vec![BTreeMap::new(); machine.tapes];
        for (pos, &symbol) in (0..).zip(&machine.input) {
            if symbol != 0 {
                tapes[0].insert(pos, symbol);
            }
        }
        Self {
            machine,
            tapes,
            heads: vec![0; machine.tapes],
            visited: vec![(0, 0); machine.tapes],
            state: 0,
            steps: 0,
        }
    }

    /// Runs like `Simulator::run`, or like `Engine::advance` without the final status check
    /// if `prove` is false.
    fn run(&mut self, fuel: u64, prove: bool) -> Outcome {
        let limit = self.steps + fuel;
        loop {
            let read: Vec<u8> = (0..self.machine.tapes)
                .map(|t| self.tapes[t].get(&self.heads[t]).copied().unwrap_or(0))
                .collect();
            let Some(entry) = self.machine.entry(self.state, &read) else {
                return Outcome::Halted {
                    steps: self.steps,
                    state: state(self.state),
                };
            };
            if prove && let Some(proof) = self.proof(&read, entry) {
                return Outcome::NeverHalts {
                    steps: self.steps,
                    proof,
                };
            }
            if self.steps == limit {
                return Outcome::OutOfFuel { steps: self.steps };
            }
            for t in 0..self.machine.tapes {
                if entry.write[t] == 0 {
                    self.tapes[t].remove(&self.heads[t]);
                } else {
                    self.tapes[t].insert(self.heads[t], entry.write[t]);
                }
                self.heads[t] += entry.dir[t].delta();
                let (lo, hi) = self.visited[t];
                self.visited[t] = (lo.min(self.heads[t]), hi.max(self.heads[t]));
            }
            self.state = entry.next;
            self.steps += 1;
        }
    }

    /// Runs like `Engine::advance`.
    fn advance(&mut self, fuel: u64) -> Outcome {
        match self.run(fuel, false) {
            Outcome::OutOfFuel { .. } => self.run(0, true),
            outcome => outcome,
        }
    }

    fn proof(&self, read: &[u8], entry: &Entry) -> Option<NonHaltProof> {
        if entry.next != self.state {
            return None;
        }
        let state = state(self.state);
        let stays =
            (0..self.machine.tapes).all(|t| entry.dir[t] == Dir::N && entry.write[t] == read[t]);
        if stays {
            return Some(NonHaltProof::StayLoop { state });
        }
        let head = self.heads[0];
        let only_blanks = match entry.dir[0] {
            Dir::L => self.tapes[0].range(..head).next().is_none(),
            Dir::R => self.tapes[0].range(head + 1..).next().is_none(),
            Dir::N => false,
        };
        (self.machine.tapes == 1 && read[0] == 0 && only_blanks).then_some(
            NonHaltProof::ChainToInfinity {
                state,
                dir: entry.dir[0],
            },
        )
    }

    fn snapshot(&self, outcome: Outcome) -> Snapshot {
        Snapshot {
            outcome,
            state: state(self.state),
            steps: self.steps,
            tapes: (0..self.machine.tapes)
                .map(|t| TapeState {
                    head: self.heads[t],
                    visited: self.visited[t],
                    cells: self.tapes[t].clone(),
                })
                .collect(),
        }
    }
}

/// Everything the engines must agree on.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Snapshot {
    outcome: Outcome,
    state: State,
    steps: u64,
    tapes: Vec<TapeState>,
}

/// What the engines must agree on for each tape.
#[derive(Clone, Debug, PartialEq, Eq)]
struct TapeState {
    head: i64,
    /// The smallest and largest head position so far.
    visited: (i64, i64),
    /// The non-blank cells.
    cells: BTreeMap<i64, u8>,
}

/// The non-blank cells of a tape.
fn cells(tape: &impl TapeView) -> BTreeMap<i64, u8> {
    let Some(range) = tape.non_blank() else {
        return BTreeMap::new();
    };
    let len = usize::try_from(range.end() - range.start() + 1).expect("a small tape");
    (*range.start()..)
        .zip(tape.window(*range.start(), len))
        .filter(|(_, s)| !s.is_blank())
        .map(|(pos, s)| (pos, s.get()))
        .collect()
}

fn dense(sim: &Simulator<&Machine>, outcome: Outcome) -> Snapshot {
    Snapshot {
        outcome,
        state: sim.state(),
        steps: sim.steps(),
        tapes: sim
            .tapes()
            .iter()
            .map(|t| TapeState {
                head: t.head(),
                visited: (*t.touched().start(), *t.touched().end()),
                cells: cells(t),
            })
            .collect(),
    }
}

fn chained(sim: &ChainSim<&Machine>, outcome: Outcome) -> Snapshot {
    let extent = sim.extent();
    Snapshot {
        outcome,
        state: sim.state(),
        steps: sim.steps(),
        tapes: vec![TapeState {
            head: sim.tape().head(),
            visited: (*extent.start(), *extent.end()),
            cells: cells(sim.tape()),
        }],
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn simulators_agree_with_the_reference(r in machines(1, 5, 3), fuel in 0u64..2000) {
        let m = r.build();
        let mut reference = Reference::new(&r);
        let expected = reference.run(fuel, true);
        let expected = reference.snapshot(expected);

        let mut sim = Simulator::new(&m, &r.input());
        let outcome = sim.run(fuel);
        prop_assert_eq!(&dense(&sim, outcome), &expected);

        let mut chain = ChainSim::new(&m, &r.input()).unwrap();
        let outcome = chain.run(fuel);
        prop_assert!(chain.steps() <= fuel);
        prop_assert!(chain.macro_steps() <= chain.steps());
        prop_assert_eq!(&chained(&chain, outcome), &expected);
    }
}

proptest! {
    #[test]
    fn runs_can_be_split_anywhere(
        r in machines(1, 5, 3),
        first in 0u64..1000,
        second in 0u64..1000,
    ) {
        let m = r.build();
        let mut whole = Simulator::new(&m, &r.input());
        let outcome = whole.run(first + second);
        let mut split = Simulator::new(&m, &r.input());
        split.run(first);
        let split_outcome = split.run(second);
        prop_assert_eq!(dense(&split, split_outcome), dense(&whole, outcome));

        let mut whole = ChainSim::new(&m, &r.input()).unwrap();
        let outcome = whole.run(first + second);
        let mut split = ChainSim::new(&m, &r.input()).unwrap();
        split.run(first);
        let split_outcome = split.run(second);
        prop_assert_eq!(chained(&split, split_outcome), chained(&whole, outcome));
    }

    #[test]
    fn stop_conditions_see_every_step(r in machines(1, 5, 3), fuel in 0u64..2000) {
        let m = r.build();
        let mut chain = ChainSim::new(&m, &r.input()).unwrap();
        let mut covered = 0;
        let mut macro_steps = 0;
        chain.run_until(fuel, |event| {
            assert_eq!(event.steps_before, covered);
            assert!(event.repeats >= 1);
            covered += event.repeats;
            macro_steps += 1;
            false
        });
        prop_assert_eq!(covered, chain.steps());
        prop_assert_eq!(macro_steps, chain.macro_steps());

        let mut sim = Simulator::new(&m, &r.input());
        let mut seen = 0;
        sim.run_until(fuel, |event| {
            seen += 1;
            assert_eq!(event.steps, seen);
            false
        });
        prop_assert_eq!(seen, sim.steps());
    }

    #[test]
    fn breakpoints_stop_at_the_same_step(r in machines(1, 5, 3), at in 1u64..500) {
        let m = r.build();
        let mut sim = Simulator::new(&m, &r.input());
        let outcome = sim.run_until(1000, |event| event.steps == at);
        let mut reference = Reference::new(&r);
        let expected = if let Outcome::Stopped { steps } = outcome {
            prop_assert_eq!(steps, at);
            reference.run(at, true);
            reference.snapshot(outcome)
        } else {
            let expected = reference.run(1000, true);
            reference.snapshot(expected)
        };
        prop_assert_eq!(dense(&sim, outcome), expected);
    }

    #[test]
    fn timelines_match_fresh_runs(
        r in machines(1, 5, 3),
        targets in prop::collection::vec(0u64..20_000, 1..6),
    ) {
        let m = r.build();
        let mut dense_timeline = Timeline::new(Simulator::new(&m, &r.input()));
        let mut chain_timeline = Timeline::new(ChainSim::new(&m, &r.input()).unwrap());
        for target in targets {
            let mut reference = Reference::new(&r);
            let expected = reference.advance(target);
            let expected = reference.snapshot(expected);

            let outcome = dense_timeline.seek(target);
            prop_assert_eq!(dense(dense_timeline.current(), outcome), expected.clone());
            let mut fresh = Simulator::new(&m, &r.input());
            let fresh_outcome = fresh.advance(target);
            prop_assert_eq!(dense(&fresh, fresh_outcome), expected.clone());

            let outcome = chain_timeline.seek(target);
            prop_assert_eq!(chained(chain_timeline.current(), outcome), expected.clone());
            let mut fresh = ChainSim::new(&m, &r.input()).unwrap();
            let fresh_outcome = fresh.advance(target);
            prop_assert_eq!(chained(&fresh, fresh_outcome), expected);
        }
    }

    #[test]
    fn multi_tape_simulator_agrees_with_the_reference(
        r in prop_oneof![machines(2, 3, 2), machines(2, 2, 3), machines(3, 2, 2)],
        fuel in 0u64..2000,
    ) {
        let m = r.build();
        let mut reference = Reference::new(&r);
        let expected = reference.run(fuel, true);
        let expected = reference.snapshot(expected);
        let mut sim = Simulator::new(&m, &r.input());
        let outcome = sim.run(fuel);
        prop_assert_eq!(dense(&sim, outcome), expected);
    }
}
