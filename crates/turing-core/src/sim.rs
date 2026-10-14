//! The reference simulator: one step at a time on dense tapes, any number of tapes.

use alloc::vec::Vec;
use core::borrow::Borrow;

use crate::machine::{Action, Machine};
use crate::outcome::{NonHaltProof, Outcome, Status};
use crate::symbol::{Dir, MAX_TAPES, State, Symbol};
use crate::tape::{Tape, TapeView};

/// One executed step, passed to stop conditions.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StepEvent {
    /// The number of steps after this one, counted from the start.
    pub steps: u64,
    /// The state before the step.
    pub from: State,
    /// The symbols read.
    pub read: [Symbol; MAX_TAPES],
    /// What the step did.
    pub action: Action,
}

/// Executes a machine step by step from an input on the first tape.
///
/// `M` is anything that borrows a [`Machine`]: `&Machine`, `Machine`, `Rc<Machine>` or
/// `Arc<Machine>`.
///
/// Besides halting and running out of fuel, [`Simulator::run`] recognises two simple kinds of
/// endless loops, described by [`NonHaltProof`]. Both are checked before a step is made, so a
/// run that ends with [`Outcome::NeverHalts`] has not executed the looping step.
///
/// ```
/// use turing_core::{Dir, Machine, Outcome, Simulator};
/// let m = Machine::builder("flip")
///     .transition("q0", '0', "q0", '1', Dir::R)
///     .transition("q0", '1', "q0", '0', Dir::R)
///     .build()
///     .unwrap();
/// let input = m.encode_input("0110").unwrap();
/// let mut sim = Simulator::new(&m, &input);
/// assert!(matches!(sim.run(100), Outcome::Halted { steps: 4, .. }));
/// ```
#[derive(Clone, Debug)]
pub struct Simulator<M: Borrow<Machine>> {
    pub(crate) machine: M,
    pub(crate) state: State,
    pub(crate) tapes: Vec<Tape>,
    pub(crate) steps: u64,
    /// Positions of non-blank cells found left and right of the head on the first tape when
    /// looking for a [`NonHaltProof::ChainToInfinity`]. They are hints only: a hint is used if
    /// it still lies beyond the head and still holds a non-blank symbol, which takes O(1) to
    /// check, and a scan of the tape replaces it otherwise.
    pub(crate) witness: (i64, i64),
}

impl<M: Borrow<Machine>> Simulator<M> {
    /// Starts in q0 with `input` on the first tape and blank further tapes.
    ///
    /// # Panics
    ///
    /// Panics if `input` contains a symbol outside the machine's alphabet. Inputs converted
    /// with [`Machine::encode_input`] never do.
    pub fn new(machine: M, input: &[Symbol]) -> Self {
        let m = machine.borrow();
        assert_input_in_alphabet(m, input);
        let tapes = (0..m.tapes())
            .map(|i| {
                if i == 0 {
                    Tape::with_input(input)
                } else {
                    Tape::blank()
                }
            })
            .collect();
        Self {
            state: m.start(),
            machine,
            tapes,
            steps: 0,
            witness: (0, 0),
        }
    }

    /// The machine being run.
    pub fn machine(&self) -> &Machine {
        self.machine.borrow()
    }

    /// The current state.
    pub fn state(&self) -> State {
        self.state
    }

    /// Tape `i` (0-based).
    ///
    /// # Panics
    ///
    /// Panics if the machine has no tape `i`.
    pub fn tape(&self, i: usize) -> &Tape {
        &self.tapes[i]
    }

    /// All tapes.
    pub fn tapes(&self) -> &[Tape] {
        &self.tapes
    }

    /// The number of steps taken so far.
    pub fn steps(&self) -> u64 {
        self.steps
    }

    /// The symbols under the heads; entries beyond the tape count are blank.
    pub fn read(&self) -> [Symbol; MAX_TAPES] {
        let mut read = [Symbol::BLANK; MAX_TAPES];
        for (slot, tape) in read.iter_mut().zip(&self.tapes) {
            *slot = tape.read();
        }
        read
    }

    /// The transition that the next step would apply.
    pub fn next_action(&self) -> Option<Action> {
        let machine = self.machine.borrow();
        match self.tapes.as_slice() {
            [tape] => machine.action1(self.state, tape.read()),
            _ => machine.action(self.state, &self.read()),
        }
    }

    /// Makes one step, or reports that the machine has halted.
    ///
    /// Unlike [`Simulator::run`], this does not look for proofs of non-termination: a step
    /// that loops forever is still a step.
    pub fn step(&mut self) -> Status {
        match self.next_action() {
            Some(action) => {
                self.apply(action);
                Status::Running
            }
            None => Status::Halted,
        }
    }

    /// Runs until the machine halts or `fuel` more steps have been made.
    pub fn run(&mut self, fuel: u64) -> Outcome {
        self.run_until(fuel, |_| false)
    }

    /// Like [`Simulator::run`], but stops after any step for which `stop` returns `true`.
    ///
    /// Before each step the run ends, in this order, with [`Outcome::Halted`] if δ is
    /// undefined, with [`Outcome::NeverHalts`] if the step provably loops forever, and with
    /// [`Outcome::OutOfFuel`] if `fuel` steps have been made. So `run_until(0, ..)` reports
    /// the status of the current configuration without changing it.
    pub fn run_until(&mut self, fuel: u64, mut stop: impl FnMut(&StepEvent) -> bool) -> Outcome {
        if self.tapes.len() == 1 {
            self.run_single(fuel, &mut stop)
        } else {
            self.run_multi(fuel, &mut stop)
        }
    }

    /// [`Simulator::run_until`] for single-tape machines.
    fn run_single(&mut self, fuel: u64, stop: &mut impl FnMut(&StepEvent) -> bool) -> Outcome {
        let limit = self.steps.saturating_add(fuel);
        loop {
            let from = self.state;
            let read = self.tapes[0].read();
            let Some(action) = self.machine.borrow().action1(from, read) else {
                return Outcome::Halted {
                    steps: self.steps,
                    state: from,
                };
            };
            // Only steps that stay in their state can loop on their own.
            if action.next == from
                && (action.dir[0] == Dir::N || read.is_blank())
                && let Some(proof) = self.non_halt_proof(&[read], action)
            {
                return Outcome::NeverHalts {
                    steps: self.steps,
                    proof,
                };
            }
            if self.steps == limit {
                return Outcome::OutOfFuel { steps: self.steps };
            }
            let tape = &mut self.tapes[0];
            tape.write(action.write[0]);
            tape.shift(action.dir[0]);
            self.state = action.next;
            self.steps += 1;
            let event = StepEvent {
                steps: self.steps,
                from,
                read: [read, Symbol::BLANK, Symbol::BLANK],
                action,
            };
            if stop(&event) {
                return Outcome::Stopped { steps: self.steps };
            }
        }
    }

    /// [`Simulator::run_until`] for machines with several tapes.
    fn run_multi(&mut self, fuel: u64, stop: &mut impl FnMut(&StepEvent) -> bool) -> Outcome {
        let limit = self.steps.saturating_add(fuel);
        let tapes = self.tapes.len();
        loop {
            let from = self.state;
            let read = self.read();
            let Some(action) = self.machine.borrow().action(from, &read) else {
                return Outcome::Halted {
                    steps: self.steps,
                    state: from,
                };
            };
            if action.next == from
                && let Some(proof) = self.non_halt_proof(&read[..tapes], action)
            {
                return Outcome::NeverHalts {
                    steps: self.steps,
                    proof,
                };
            }
            if self.steps == limit {
                return Outcome::OutOfFuel { steps: self.steps };
            }
            self.apply(action);
            let event = StepEvent {
                steps: self.steps,
                from,
                read,
                action,
            };
            if stop(&event) {
                return Outcome::Stopped { steps: self.steps };
            }
        }
    }

    /// Executes `action` on every tape.
    fn apply(&mut self, action: Action) {
        for ((tape, &write), &dir) in self.tapes.iter_mut().zip(&action.write).zip(&action.dir) {
            tape.write(write);
            tape.shift(dir);
        }
        self.state = action.next;
        self.steps += 1;
    }

    /// A proof that applying `action` to the symbols `read` (one per tape) in the current
    /// state starts an endless loop.
    ///
    /// * [`NonHaltProof::StayLoop`]: the action keeps the state, writes what it reads and
    ///   moves no head, so the configuration never changes.
    /// * [`NonHaltProof::ChainToInfinity`] (single tape only): the action keeps the state,
    ///   reads a blank and moves the head towards a side holding only blanks. The next cell is
    ///   blank again, so by induction the same action applies forever.
    fn non_halt_proof(&mut self, read: &[Symbol], action: Action) -> Option<NonHaltProof> {
        let state = self.state;
        if action.next != state {
            return None;
        }
        let stays = read
            .iter()
            .zip(action.write.iter().zip(&action.dir))
            .all(|(&r, (&w, &d))| d == Dir::N && w == r);
        if stays {
            return Some(NonHaltProof::StayLoop { state });
        }
        match (read, action.dir[0]) {
            ([r], dir @ (Dir::L | Dir::R)) if r.is_blank() && self.only_blanks_beyond(dir) => {
                Some(NonHaltProof::ChainToInfinity { state, dir })
            }
            _ => None,
        }
    }

    /// Whether every cell of the first tape beyond the head in direction `dir` is blank.
    fn only_blanks_beyond(&mut self, dir: Dir) -> bool {
        let tape = &self.tapes[0];
        let head = tape.head();
        let witness = match dir {
            Dir::L => &mut self.witness.0,
            Dir::R => &mut self.witness.1,
            Dir::N => return false,
        };
        let beyond = (*witness - head).signum() == dir.delta();
        if beyond && !tape.get(*witness).is_blank() {
            return false;
        }
        match tape.find_non_blank(head + dir.delta(), dir) {
            Some(pos) => {
                *witness = pos;
                false
            }
            None => true,
        }
    }
}

/// Panics with a clear message if `input` contains a symbol outside the machine's alphabet.
pub(crate) fn assert_input_in_alphabet(machine: &Machine, input: &[Symbol]) {
    if let Some(symbol) = input.iter().find(|s| s.index() >= machine.num_symbols()) {
        panic!(
            "the input symbol with index {} is not in the alphabet of `{}`, which has {} symbols",
            symbol.index(),
            machine.name(),
            machine.num_symbols()
        );
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use super::*;

    fn sym(m: &Machine, glyph: char) -> Symbol {
        m.alphabet().symbol(glyph).unwrap()
    }

    fn walker() -> Machine {
        // Erases a block of x and walks off to the right on the blank tape beyond it.
        Machine::builder("walker")
            .transition("q", 'x', "q", '␣', Dir::R)
            .transition("q", '␣', "q", '␣', Dir::R)
            .build()
            .unwrap()
    }

    #[test]
    fn a_step_on_an_undefined_entry_is_not_counted() {
        let m = Machine::builder("one")
            .transition("a", '␣', "b", '1', Dir::R)
            .build()
            .unwrap();
        let mut sim = Simulator::new(&m, &[]);
        assert_eq!(sim.next_action().map(|a| a.next), Some(State::new(1)));
        assert_eq!(sim.step(), Status::Running);
        assert_eq!(sim.next_action(), None);
        assert_eq!(sim.step(), Status::Halted);
        assert_eq!(sim.steps(), 1);
        assert_eq!(sim.tape(0).head(), 1);
        assert_eq!(sim.tape(0).get(0), sym(&m, '1'));
        assert_eq!(
            sim.run(10),
            Outcome::Halted {
                steps: 1,
                state: State::new(1)
            }
        );
    }

    #[test]
    fn fuel_is_never_exceeded() {
        let m = Machine::builder("ping-pong")
            .transition("a", '␣', "b", '1', Dir::R)
            .transition("b", '␣', "a", '1', Dir::L)
            .transition("a", '1', "b", '1', Dir::R)
            .transition("b", '1', "a", '1', Dir::L)
            .build()
            .unwrap();
        let mut sim = Simulator::new(&m, &[]);
        assert_eq!(sim.run(0), Outcome::OutOfFuel { steps: 0 });
        assert_eq!(sim.run(7), Outcome::OutOfFuel { steps: 7 });
        assert_eq!(sim.run(5), Outcome::OutOfFuel { steps: 12 });
        assert_eq!(sim.steps(), 12);
        assert_eq!(sim.tape(0).touched(), 0..=1);
    }

    #[test]
    fn a_halting_step_on_the_last_unit_of_fuel_reports_the_halt() {
        let m = Machine::builder("one")
            .transition("a", '␣', "b", '1', Dir::R)
            .build()
            .unwrap();
        let mut sim = Simulator::new(&m, &[]);
        assert!(matches!(sim.run(1), Outcome::Halted { steps: 1, .. }));
    }

    #[test]
    fn staying_in_place_forever_is_detected() {
        let m = Machine::builder("stay")
            .transition("a", '␣', "b", 'x', Dir::R)
            .transition("b", '␣', "b", '␣', Dir::N)
            .build()
            .unwrap();
        let mut sim = Simulator::new(&m, &[]);
        assert_eq!(
            sim.run(100),
            Outcome::NeverHalts {
                steps: 1,
                proof: NonHaltProof::StayLoop {
                    state: State::new(1)
                }
            }
        );
        // The looping step itself is still available to `step`.
        assert_eq!(sim.step(), Status::Running);
        assert_eq!(sim.steps(), 2);
    }

    #[test]
    fn staying_and_writing_something_else_is_not_a_loop() {
        let m = Machine::builder("stay")
            .transition("a", '␣', "a", 'x', Dir::N)
            .build()
            .unwrap();
        let mut sim = Simulator::new(&m, &[]);
        assert!(matches!(sim.run(100), Outcome::Halted { steps: 1, .. }));
    }

    #[test]
    fn walking_off_to_the_right_is_detected_after_the_input() {
        let m = walker();
        let input = m.encode_input("xx").unwrap();
        let mut sim = Simulator::new(&m, &input);
        assert_eq!(
            sim.run(100),
            Outcome::NeverHalts {
                steps: 2,
                proof: NonHaltProof::ChainToInfinity {
                    state: State::new(0),
                    dir: Dir::R
                }
            }
        );
        assert_eq!(sim.tape(0).head(), 2);
    }

    #[test]
    fn blanks_inside_the_input_do_not_prove_anything() {
        let m = walker();
        let input = m.encode_input("␣␣x␣x").unwrap();
        let mut sim = Simulator::new(&m, &input);
        // The hint found at position 2 goes stale when the head erases it.
        assert!(matches!(
            sim.run(100),
            Outcome::NeverHalts {
                steps: 5,
                proof: NonHaltProof::ChainToInfinity { dir: Dir::R, .. }
            }
        ));
    }

    #[test]
    fn walking_off_to_the_left_is_detected() {
        let m = Machine::builder("left")
            .transition("q", 'x', "q", 'x', Dir::L)
            .transition("q", '␣', "q", 'y', Dir::L)
            .build()
            .unwrap();
        let input = m.encode_input("xxx").unwrap();
        let mut sim = Simulator::new(&m, &input);
        assert!(matches!(
            sim.run(100),
            Outcome::NeverHalts {
                steps: 1,
                proof: NonHaltProof::ChainToInfinity { dir: Dir::L, .. }
            }
        ));
    }

    #[test]
    fn a_blank_loop_inside_the_visited_region_is_proved_once_the_rest_is_blank() {
        // Writes `1` at -2 and -1, comes back and then walks left over the erased cells.
        let m = Machine::builder("erase")
            .transition("a", '␣', "b", '1', Dir::L)
            .transition("b", '␣', "c", '1', Dir::R)
            .transition("c", '1', "d", '␣', Dir::L)
            .transition("d", '1', "e", '␣', Dir::R)
            .transition("e", '␣', "e", '␣', Dir::L)
            .build()
            .unwrap();
        let mut sim = Simulator::new(&m, &[]);
        assert!(matches!(
            sim.run(100),
            Outcome::NeverHalts {
                steps: 4,
                proof: NonHaltProof::ChainToInfinity { dir: Dir::L, .. }
            }
        ));
        assert_eq!(sim.tape(0).head(), 0);
        assert_eq!(sim.tape(0).touched(), -1..=0);
    }

    #[test]
    fn stop_conditions_see_every_step() {
        let m = walker();
        let input = m.encode_input("xxxx").unwrap();
        let mut sim = Simulator::new(&m, &input);
        let mut seen = Vec::new();
        let outcome = sim.run_until(100, |event| {
            seen.push(event.steps);
            event.steps == 3
        });
        assert_eq!(outcome, Outcome::Stopped { steps: 3 });
        assert_eq!(seen, [1, 2, 3]);
        let mut events = Vec::new();
        sim.run_until(1, |event| {
            events.push(*event);
            false
        });
        let x = sym(&m, 'x');
        assert_eq!(
            events,
            [StepEvent {
                steps: 4,
                from: State::new(0),
                read: [x, Symbol::BLANK, Symbol::BLANK],
                action: Action::single(Symbol::BLANK, Dir::R, State::new(0)),
            }]
        );
    }

    #[test]
    fn several_tapes_move_independently() {
        let m = Machine::builder("pair")
            .tapes(2)
            .transition_multi("q", &['a', '␣'], "q", &['b', 'a'], &[Dir::R, Dir::L])
            .transition_multi("q", &['␣', '␣'], "r", &['␣', 'b'], &[Dir::N, Dir::N])
            .build()
            .unwrap();
        let input = m.encode_input("aa").unwrap();
        let mut sim = Simulator::new(&m, &input);
        assert_eq!(sim.tapes().len(), 2);
        assert_eq!(sim.read(), [sym(&m, 'a'), Symbol::BLANK, Symbol::BLANK]);
        assert!(matches!(sim.run(10), Outcome::Halted { steps: 3, .. }));
        let (b, a) = (sym(&m, 'b'), sym(&m, 'a'));
        assert_eq!(sim.tape(0).window(0, 3), vec![b, b, Symbol::BLANK]);
        assert_eq!(sim.tape(1).window(-2, 3), vec![b, a, a]);
        assert_eq!(sim.tape(0).head(), 2);
        assert_eq!(sim.tape(1).head(), -2);
        assert_eq!(sim.state(), m.state_by_name("r").unwrap());
    }

    #[test]
    fn several_tapes_detect_staying_loops_only_if_every_tape_stays() {
        let m = Machine::builder("stay2")
            .tapes(2)
            .transition_multi("q", &['␣', '␣'], "q", &['␣', '␣'], &[Dir::N, Dir::N])
            .build()
            .unwrap();
        let mut sim = Simulator::new(&m, &[]);
        assert!(matches!(
            sim.run(10),
            Outcome::NeverHalts {
                steps: 0,
                proof: NonHaltProof::StayLoop { .. }
            }
        ));

        let walks = Machine::builder("walk2")
            .tapes(2)
            .transition_multi("q", &['␣', '␣'], "q", &['␣', '␣'], &[Dir::N, Dir::R])
            .build()
            .unwrap();
        let mut sim = Simulator::new(&walks, &[]);
        assert_eq!(sim.run(10), Outcome::OutOfFuel { steps: 10 });
    }

    #[test]
    #[should_panic(expected = "not in the alphabet")]
    fn foreign_input_symbols_are_rejected() {
        let m = walker();
        let _ = Simulator::new(&m, &[Symbol::new(9)]);
    }
}
