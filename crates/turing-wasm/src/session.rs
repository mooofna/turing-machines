//! Interactive runs for the playground: step, seek in both directions, run with breakpoints.
//!
//! # Engines
//!
//! A session keeps a [`Timeline`], so that every step can be reached exactly in both
//! directions. Single-tape machines other than enumerators run on the chain simulator
//! ([`ChainSim`]), which executes runs of a self-looping transition as one macro step and so runs
//! the BB(5) champion's 47 million steps in a fraction of a second. Machines with several tapes
//! and enumerators run on the reference simulator ([`Simulator`]) one step at a time, which an
//! enumerator needs to see every step into a final state.
//!
//! # Exactness
//!
//! A session never goes beyond step 2^53 - 1, the largest integer a JavaScript number holds
//! exactly; a run that reaches it stops with reason `budget`. Every other number of a snapshot
//! is bounded by the step count, so all numbers that cross the boundary are exact.
//!
//! # Breakpoints
//!
//! A run stops before applying a transition marked as a breakpoint, after a step that enters a
//! state marked as a breakpoint from a different state (self-loops do not count, so a scan in
//! that state does not stop at every cell), and, for enumerators with `onOutput`, after every
//! step that prints a word. The first step of a run is never stopped by a transition
//! breakpoint, so that running again after a breakpoint continues instead of stopping on the
//! spot. Unknown states and transition indices are ignored.
//!
//! A run with breakpoints first simulates a copy of the current position to find where it
//! stops, then moves the timeline there; this doubles the cost of such runs, while runs without
//! breakpoints move the timeline directly.
//!
//! # Non-halting
//!
//! The status `never-halts` is reported when the current configuration repeats forever, which
//! the session checks directly: the next transition stays in the current state, every tape that
//! does not move reads what it writes, and every tape that moves reads a blank with only blanks
//! ahead. That covers machines that walk off on blank cells and machines that stay in place.
//! A run started in such a configuration stops at once with reason `never-halts`.

use std::collections::VecDeque;
use std::fmt;
use std::ops::RangeInclusive;
use std::rc::Rc;

use turing_core::config_text::configuration;
use turing_core::{
    ChainSim, Dir, Kind, MAX_TAPES, Machine, NonHaltProof, Outcome, RunTape, Simulator, State,
    Status, Symbol, Tape, TapeView, Timeline, decode,
};

use crate::Error;
use crate::convert::{self, MAX_SAFE, number, signed};
use crate::dto::{
    BreakpointsDto, MachineSpecDto, OutputDto, RunResultDto, RunStatusDto, SnapshotDto,
    StopReasonDto, TapeWindowDto, VerdictKindDto,
};

/// The largest number of cells shown on either side of a head.
pub(crate) const MAX_RADIUS: u32 = 4096;

/// The largest number of cells of a configuration line.
const CONFIGURATION_CELLS: usize = 60;

/// The number of printed words a snapshot lists.
const EMITTED_SHOWN: usize = 50;

/// The largest number of glyphs of a printed word in a snapshot; longer words are shortened in
/// the middle to this length, with `…` in place of the omitted part.
pub(crate) const MAX_WORD: usize = 256;

/// The emission index of an enumerator has one entry every this many steps.
const SEGMENT: u64 = 1 << 14;

/// A tape of either simulator, so that the generic helpers of the core accept both.
#[derive(Copy, Clone, Debug)]
enum TapeRef<'a> {
    Dense(&'a Tape),
    Runs(&'a RunTape),
}

impl TapeView for TapeRef<'_> {
    fn head(&self) -> i64 {
        match self {
            Self::Dense(t) => t.head(),
            Self::Runs(t) => t.head(),
        }
    }

    fn get(&self, pos: i64) -> Symbol {
        match self {
            Self::Dense(t) => t.get(pos),
            Self::Runs(t) => t.get(pos),
        }
    }

    fn non_blank(&self) -> Option<RangeInclusive<i64>> {
        match self {
            Self::Dense(t) => t.non_blank(),
            Self::Runs(t) => t.non_blank(),
        }
    }

    fn count_non_blank(&self) -> u64 {
        match self {
            Self::Dense(t) => t.count_non_blank(),
            Self::Runs(t) => t.count_non_blank(),
        }
    }

    fn window(&self, from: i64, len: usize) -> Vec<Symbol> {
        match self {
            Self::Dense(t) => t.window(from, len),
            Self::Runs(t) => t.window(from, len),
        }
    }
}

/// The simulator behind a session.
#[derive(Debug)]
enum Engine {
    /// One step at a time, any number of tapes.
    Exact(Timeline<Simulator<Rc<Machine>>>),
    /// Chain steps, one tape.
    Chain(Timeline<ChainSim<Rc<Machine>>>),
}

impl Engine {
    fn steps(&self) -> u64 {
        match self {
            Self::Exact(t) => t.current().steps(),
            Self::Chain(t) => t.current().steps(),
        }
    }

    fn state(&self) -> State {
        match self {
            Self::Exact(t) => t.current().state(),
            Self::Chain(t) => t.current().state(),
        }
    }

    /// The symbols under the heads, blank beyond the tape count.
    fn read(&self) -> [Symbol; MAX_TAPES] {
        match self {
            Self::Exact(t) => t.current().read(),
            Self::Chain(t) => {
                let mut read = [Symbol::BLANK; MAX_TAPES];
                read[0] = t.current().tape().read();
                read
            }
        }
    }

    fn tape(&self, i: usize) -> Option<TapeRef<'_>> {
        match self {
            Self::Exact(t) => t.current().tapes().get(i).map(TapeRef::Dense),
            Self::Chain(t) => (i == 0).then(|| TapeRef::Runs(t.current().tape())),
        }
    }

    /// The smallest and largest head position of the first tape so far.
    fn extent(&self) -> RangeInclusive<i64> {
        match self {
            Self::Exact(t) => t.current().tapes().first().map_or(0..=0, Tape::touched),
            Self::Chain(t) => t.current().extent(),
        }
    }

    fn seek(&mut self, step: u64) -> Outcome {
        match self {
            Self::Exact(t) => t.seek(step),
            Self::Chain(t) => t.seek(step),
        }
    }
}

/// Breakpoints resolved against the machine.
#[derive(Clone, Debug, Default)]
struct Stops {
    /// By spec index.
    transitions: Vec<bool>,
    /// By state index.
    states: Vec<bool>,
    /// Stop after a printed word.
    on_output: bool,
}

impl Stops {
    fn new(dto: &BreakpointsDto, machine: &Machine, spec_transitions: usize) -> Self {
        let mut transitions = vec![false; spec_transitions];
        for &i in &dto.transitions {
            if let Some(slot) = transitions.get_mut(i) {
                *slot = true;
            }
        }
        let mut states = vec![false; machine.num_states()];
        for name in &dto.states {
            if let Some(q) = machine.state_by_name(name)
                && let Some(slot) = states.get_mut(q.index())
            {
                *slot = true;
            }
        }
        Self {
            transitions,
            states,
            on_output: dto.on_output && machine.kind() == Kind::Enumerator,
        }
    }

    fn is_active(&self) -> bool {
        self.on_output || self.transitions.contains(&true) || self.states.contains(&true)
    }

    /// Whether the transition with spec index `origin` is a breakpoint.
    fn at_transition(&self, origin: Option<usize>) -> bool {
        origin
            .and_then(|i| self.transitions.get(i))
            .copied()
            .unwrap_or(false)
    }

    /// Whether a step from `from` to `to` enters a breakpoint state.
    fn enters(&self, from: State, to: State) -> bool {
        from != to && self.states.get(to.index()).copied().unwrap_or(false)
    }
}

/// Words printed by an enumerator, exact at every step of the timeline.
///
/// Forward moves observe every step. Backward moves rebuild the count from an index of the
/// cumulative count every [`SEGMENT`] steps and the last words by replaying the few segments
/// that contain them, so memory stays proportional to the steps explored divided by
/// [`SEGMENT`], not to the number of words.
#[derive(Clone, Debug, Default)]
struct Emissions {
    /// `counts[j]` is the number of words printed in steps 1 to `j * SEGMENT`.
    counts: Vec<u64>,
    /// Words printed up to the current step.
    count: u64,
    /// The last words up to the current step with their steps, oldest first.
    recent: VecDeque<(u64, String)>,
}

impl Emissions {
    fn new() -> Self {
        Self {
            counts: vec![0],
            count: 0,
            recent: VecDeque::new(),
        }
    }

    /// Records step `step`, which printed `word` if it entered a final state.
    fn observe(&mut self, step: u64, word: Option<String>) {
        if let Some(word) = word {
            self.count += 1;
            self.recent.push_back((step, word));
            if self.recent.len() > EMITTED_SHOWN {
                self.recent.pop_front();
            }
        }
        if step.is_multiple_of(SEGMENT) && step / SEGMENT == self.counts.len() as u64 {
            self.counts.push(self.count);
        }
    }

    /// Moves back to step `target` (before the current step) and leaves the timeline there.
    fn rewind(
        &mut self,
        timeline: &mut Timeline<Simulator<Rc<Machine>>>,
        machine: &Machine,
        target: u64,
    ) {
        // Dropping the words after `target` is enough if `recent` still lists the last words up
        // to `target` afterwards: if it holds every word printed so far, or if no word is newer.
        let newer = self
            .recent
            .iter()
            .rev()
            .take_while(|(s, _)| *s > target)
            .count();
        let complete = self.recent.len() as u64 == self.count;
        if complete || newer == 0 {
            self.recent.truncate(self.recent.len() - newer);
            self.count -= newer as u64;
            timeline.seek(target);
            return;
        }

        let segment = usize::try_from(target / SEGMENT).unwrap_or(usize::MAX);
        let (segment, base) = match self.counts.get(segment) {
            Some(&base) => (segment, base),
            None => (0, 0),
        };
        let start = segment as u64 * SEGMENT;
        let (n, mut words) = replay(timeline, machine, start, target, EMITTED_SHOWN);
        self.count = base + n;
        let mut need = EMITTED_SHOWN.saturating_sub(words.len());
        let mut j = segment;
        while need > 0 && j > 0 {
            j -= 1;
            let (Some(&before), Some(&after)) = (self.counts.get(j), self.counts.get(j + 1)) else {
                break;
            };
            if after == before {
                continue;
            }
            let from = j as u64 * SEGMENT;
            let (_, older) = replay(timeline, machine, from, from + SEGMENT, need);
            need -= older.len();
            for entry in older.into_iter().rev() {
                words.push_front(entry);
            }
        }
        self.recent = words;
        timeline.seek(target);
    }
}

/// Replays steps `from + 1` to `to` and returns the number of printed words and the last
/// `keep` of them.
fn replay(
    timeline: &mut Timeline<Simulator<Rc<Machine>>>,
    machine: &Machine,
    from: u64,
    to: u64,
    keep: usize,
) -> (u64, VecDeque<(u64, String)>) {
    timeline.seek(from);
    let mut sim = timeline.current().clone();
    let mut count = 0;
    let mut words = VecDeque::new();
    while sim.steps() < to {
        if sim.step() == Status::Halted {
            break;
        }
        if machine.is_final(sim.state()) {
            count += 1;
            if keep > 0 {
                if words.len() == keep {
                    words.pop_front();
                }
                words.push_back((sim.steps(), printed_word(machine, sim.tapes().first())));
            }
        }
    }
    (count, words)
}

/// The word an enumerator prints: the first tape from the leftmost to the rightmost non-blank
/// cell, shortened to [`MAX_WORD`] glyphs.
fn printed_word(machine: &Machine, tape: Option<&Tape>) -> String {
    /// Glyphs kept before and after the `…` of a shortened word.
    const HEAD: usize = MAX_WORD / 2;
    const TAIL: usize = MAX_WORD - HEAD - 1;
    let Some(tape) = tape else {
        return String::new();
    };
    let Some(range) = tape.non_blank() else {
        return String::new();
    };
    let (first, last) = (*range.start(), *range.end());
    let alphabet = machine.alphabet();
    let glyphs = |from: i64, n: usize| -> String {
        tape.window(from, n)
            .into_iter()
            .map(|s| alphabet.glyph(s))
            .collect()
    };
    match usize::try_from(last - first + 1) {
        Ok(len) if len <= MAX_WORD => glyphs(first, len),
        _ => {
            let mut word = glyphs(first, HEAD);
            word.push('…');
            word.push_str(&glyphs(last + 1 - i64::try_from(TAIL).unwrap_or(0), TAIL));
            word
        }
    }
}

/// What a forward move found out.
#[derive(Copy, Clone, Debug, Default)]
struct Probe {
    /// The step reached.
    steps: u64,
    /// A breakpoint stopped the move.
    hit: bool,
    /// Whether `last` is known.
    stepped: bool,
    /// The spec index of the transition of the last step, if `stepped`.
    last: Option<usize>,
}

/// An interactive run of one machine.
pub(crate) struct Session {
    machine: Rc<Machine>,
    engine: Engine,
    /// The number of transitions of the spec, the range of breakpoint indices.
    spec_transitions: usize,
    /// The spec index of the transition applied by the step that reached `.0`.
    last: (u64, Option<usize>),
    /// A proof of non-halting reported by the simulator at step `.0`.
    hint: Option<(u64, NonHaltProof)>,
    /// Printed words, for enumerators.
    emissions: Option<Emissions>,
}

impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Session")
            .field("machine", &self.machine.name())
            .field(
                "engine",
                &match self.engine {
                    Engine::Exact(_) => "exact",
                    Engine::Chain(_) => "chain",
                },
            )
            .field("step", &self.engine.steps())
            .finish_non_exhaustive()
    }
}

impl Session {
    /// Compiles `spec` and places `input` on the first tape.
    ///
    /// # Errors
    ///
    /// Fails with the first error of the spec, or if the input contains a glyph outside Σ.
    pub(crate) fn new(dto: &MachineSpecDto, input: &str) -> Result<Self, Error> {
        let spec = convert::spec(dto).map_err(|p| convert::first_error(&p))?;
        let machine = spec.compile().map_err(|d| convert::first_diagnostic(&d))?;
        let word = machine
            .encode_input(input)
            .map_err(|e| Error::new(e.to_string()))?;
        let machine = Rc::new(machine);
        let enumerator = machine.kind() == Kind::Enumerator;
        let engine = if machine.tapes() == 1 && !enumerator {
            let sim =
                ChainSim::new(Rc::clone(&machine), &word).map_err(|e| Error::new(e.to_string()))?;
            Engine::Chain(Timeline::new(sim))
        } else {
            Engine::Exact(Timeline::new(Simulator::new(Rc::clone(&machine), &word)))
        };
        Ok(Self {
            machine,
            engine,
            spec_transitions: spec.transitions.len(),
            last: (0, None),
            hint: None,
            emissions: enumerator.then(Emissions::new),
        })
    }

    /// The current step.
    pub(crate) fn steps(&self) -> u64 {
        self.engine.steps()
    }

    /// Moves to `target`, or as far as the machine runs.
    pub(crate) fn seek(&mut self, target: u64) {
        let target = target.min(MAX_SAFE);
        let current = self.steps();
        if target > current {
            self.forward(target - current, &Stops::default());
        } else if target < current {
            self.backward(target);
        }
    }

    /// Makes up to `count` steps.
    pub(crate) fn step(&mut self, count: u64) {
        self.forward(count, &Stops::default());
    }

    /// Goes back up to `count` steps.
    pub(crate) fn back(&mut self, count: u64) {
        let target = self.steps().saturating_sub(count);
        self.seek(target);
    }

    /// Runs at most `max_steps` steps with breakpoints.
    pub(crate) fn run(
        &mut self,
        max_steps: u64,
        breakpoints: &BreakpointsDto,
        radius: u32,
    ) -> RunResultDto {
        let stops = Stops::new(breakpoints, &self.machine, self.spec_transitions);
        let start = self.steps();
        let mut hit = false;
        if !self.is_halted() && self.proof().is_none() {
            hit = self.forward(max_steps, &stops).hit;
        }
        let reason = if self.is_halted() {
            StopReasonDto::Halted
        } else if hit {
            StopReasonDto::Breakpoint
        } else if self.proof().is_some() {
            StopReasonDto::NeverHalts
        } else {
            StopReasonDto::Budget
        };
        RunResultDto {
            snapshot: self.snapshot(radius),
            reason,
            steps_done: number(self.steps() - start),
        }
    }

    /// Moves forward by at most `budget` steps.
    fn forward(&mut self, budget: u64, stops: &Stops) -> Probe {
        let start = self.steps();
        let budget = budget.min(MAX_SAFE.saturating_sub(start));
        if budget == 0 {
            return Probe {
                steps: start,
                ..Probe::default()
            };
        }
        let probe = if stops.is_active() || self.emissions.is_some() {
            self.probe(budget, stops)
        } else {
            self.direct(start + budget)
        };
        if probe.stepped {
            self.last = (probe.steps, probe.last);
        }
        self.settle();
        probe
    }

    /// Moves the timeline to `target` without observing the steps; records the last
    /// transition on the way when it can do so for free.
    fn direct(&mut self, target: u64) -> Probe {
        let outcome = self.engine.seek(target - 1);
        self.note(outcome);
        let mut probe = Probe::default();
        if self.steps() == target - 1 && !self.is_halted() {
            let origin = self.next_origin();
            let outcome = self.engine.seek(target);
            self.note(outcome);
            if self.steps() == target {
                probe.stepped = true;
                probe.last = origin;
            }
        }
        probe.steps = self.steps();
        probe
    }

    /// Simulates a copy to find where the move stops, then moves the timeline there.
    fn probe(&mut self, budget: u64, stops: &Stops) -> Probe {
        let machine = Rc::clone(&self.machine);
        let probe = match &mut self.engine {
            Engine::Chain(timeline) => {
                let mut sim = timeline.current().clone();
                probe_chain(&machine, &mut sim, budget, stops)
            }
            Engine::Exact(timeline) => {
                let mut sim = timeline.current().clone();
                probe_exact(&machine, &mut sim, self.emissions.as_mut(), budget, stops)
            }
        };
        let outcome = self.engine.seek(probe.steps);
        self.note(outcome);
        if self.steps() == probe.steps {
            probe
        } else {
            // The simulator stopped earlier than the copy, which only happens if it proved the
            // machine non-halting on the way. Keep the words consistent with where it is.
            let reached = self.steps();
            if let (Engine::Exact(timeline), Some(emissions)) =
                (&mut self.engine, &mut self.emissions)
            {
                emissions.rewind(timeline, &machine, reached);
            }
            Probe {
                steps: reached,
                ..Probe::default()
            }
        }
    }

    /// Moves back to `target`.
    fn backward(&mut self, target: u64) {
        if let (Engine::Exact(timeline), Some(emissions)) = (&mut self.engine, &mut self.emissions)
        {
            emissions.rewind(timeline, &self.machine, target);
        } else {
            let outcome = self.engine.seek(target);
            self.note(outcome);
        }
        self.settle();
    }

    /// Keeps a proof reported by the simulator.
    fn note(&mut self, outcome: Outcome) {
        if let Outcome::NeverHalts { steps, proof } = outcome {
            self.hint = Some((steps, proof));
        }
    }

    /// Makes sure the last transition is known for the current step, replaying one step from
    /// the nearest checkpoint if necessary.
    fn settle(&mut self) {
        let s = self.steps();
        if s == 0 {
            self.last = (0, None);
            return;
        }
        if self.last.0 == s {
            return;
        }
        let outcome = self.engine.seek(s - 1);
        self.note(outcome);
        let origin = (self.steps() == s - 1)
            .then(|| self.next_origin())
            .flatten();
        let outcome = self.engine.seek(s);
        self.note(outcome);
        self.last = (self.steps(), origin);
    }

    /// The spec index of the transition the next step applies.
    fn next_origin(&self) -> Option<usize> {
        let read = self.engine.read();
        self.machine.origin(self.engine.state(), &read)
    }

    fn is_halted(&self) -> bool {
        let read = self.engine.read();
        self.machine.action(self.engine.state(), &read).is_none()
    }

    /// Why the machine never halts from the current configuration, if that is certain.
    fn proof(&self) -> Option<String> {
        let machine = &*self.machine;
        let q = self.engine.state();
        let read = self.engine.read();
        let action = machine.action(q, &read)?;
        let name = state_name(machine, q);
        let mut walking: Vec<(usize, Dir)> = Vec::new();
        let repeats = action.next == q
            && (0..machine.tapes()).all(|i| {
                let (r, w, d) = (read[i], action.write[i], action.dir[i]);
                match d {
                    Dir::N => w == r,
                    Dir::L | Dir::R => {
                        let clear = r.is_blank()
                            && self.engine.tape(i).is_some_and(|t| {
                                let head = t.head();
                                t.non_blank().is_none_or(|range| match d {
                                    Dir::R => *range.end() < head,
                                    _ => *range.start() > head,
                                })
                            });
                        if clear {
                            walking.push((i, d));
                        }
                        clear
                    }
                }
            });
        if repeats {
            return Some(describe_repeat(machine.tapes(), &name, &walking));
        }
        match self.hint {
            Some((step, proof)) if step == self.steps() => Some(describe_proof(machine, proof)),
            _ => None,
        }
    }

    /// The current configuration with `radius` cells on either side of each head.
    pub(crate) fn snapshot(&self, radius: u32) -> SnapshotDto {
        let machine = &*self.machine;
        let r = i64::from(radius.min(MAX_RADIUS));
        let width = usize::try_from(2 * r + 1).unwrap_or(1);
        let q = self.engine.state();
        let halted = self.is_halted();
        let reason = if halted { None } else { self.proof() };
        let status = if halted {
            RunStatusDto::Halted
        } else if reason.is_some() {
            RunStatusDto::NeverHalts
        } else {
            RunStatusDto::Running
        };
        let alphabet = machine.alphabet();
        let tapes: Vec<TapeRef<'_>> = (0..machine.tapes())
            .filter_map(|i| self.engine.tape(i))
            .collect();
        let windows = tapes
            .iter()
            .map(|t| {
                let head = t.head();
                TapeWindowDto {
                    head: signed(head),
                    start: signed(head - r),
                    cells: t
                        .window(head - r, width)
                        .into_iter()
                        .map(|s| alphabet.glyph(s).to_string())
                        .collect(),
                }
            })
            .collect();
        let configuration = tapes
            .iter()
            .map(|t| configuration(machine, q, t, CONFIGURATION_CELLS))
            .collect::<Vec<_>>()
            .join("\n");
        let first = tapes.first();
        let output = (halted && machine.kind() == Kind::Transducer)
            .then(|| first.map(|t| decode(machine, t)))
            .flatten()
            .map(|o| OutputDto {
                word: o.word,
                value: o.value.map(|v| v.to_string()),
            });
        let verdict = (halted && machine.kind() == Kind::Acceptor).then(|| {
            if machine.is_final(q) {
                VerdictKindDto::Accepted
            } else {
                VerdictKindDto::Rejected
            }
        });
        let extent = self.engine.extent();
        let (emitted, emitted_count) = self.emissions.as_ref().map_or((Vec::new(), 0), |e| {
            (e.recent.iter().map(|(_, w)| w.clone()).collect(), e.count)
        });
        SnapshotDto {
            step: number(self.steps()),
            state: state_name(machine, q),
            status,
            verdict,
            reason,
            tapes: windows,
            extent: [signed(*extent.start()), signed(*extent.end())],
            configuration,
            next_transition: if halted { None } else { self.next_origin() },
            last_transition: (self.last.0 == self.steps())
                .then_some(self.last.1)
                .flatten(),
            ones: number(first.map_or(0, TapeView::count_non_blank)),
            output,
            emitted,
            emitted_count: number(emitted_count),
        }
    }
}

/// Simulates a copy of a chain simulator for at most `budget` steps.
fn probe_chain(
    machine: &Machine,
    sim: &mut ChainSim<Rc<Machine>>,
    budget: u64,
    stops: &Stops,
) -> Probe {
    let start = sim.steps();
    let q = sim.state();
    let a = sim.tape().read();
    let Some(first) = machine.action1(q, a) else {
        return Probe {
            steps: start,
            ..Probe::default()
        };
    };
    // The first step on its own: it never stops at a transition breakpoint, and a partial
    // chain of one step keeps it exact.
    let first_origin = machine.origin(q, &[a]);
    sim.run(1);
    if sim.steps() == start {
        return Probe {
            steps: start,
            ..Probe::default()
        };
    }
    let mut probe = Probe {
        steps: sim.steps(),
        hit: stops.enters(q, first.next),
        stepped: true,
        last: first_origin,
    };
    if probe.hit || budget == 1 {
        return probe;
    }
    // The macro steps announced so far: (steps before, spec index), latest last.
    let mut seen: [Option<(u64, Option<usize>)>; 2] = [None, None];
    let mut pending = false;
    let mut hit = false;
    let outcome = sim.run_until(budget - 1, |event| {
        if pending {
            hit = true;
            return true;
        }
        let origin = machine.origin(event.from, &[event.read]);
        if stops.at_transition(origin) {
            hit = true;
            return true;
        }
        pending = stops.enters(event.from, event.action.next);
        seen = [seen[1], Some((event.steps_before, origin))];
        false
    });
    if pending && !hit && matches!(outcome, Outcome::OutOfFuel { .. }) {
        hit = true;
    }
    probe.steps = sim.steps();
    probe.hit = hit;
    // The latest announced macro step that was executed, at least partially.
    if let Some((_, origin)) = seen
        .iter()
        .rev()
        .flatten()
        .find(|(before, _)| *before < probe.steps)
    {
        probe.last = *origin;
    }
    probe
}

/// Steps a copy of the reference simulator for at most `budget` steps, observing printed
/// words.
fn probe_exact(
    machine: &Machine,
    sim: &mut Simulator<Rc<Machine>>,
    mut emissions: Option<&mut Emissions>,
    budget: u64,
    stops: &Stops,
) -> Probe {
    let mut done = 0;
    let mut probe = Probe {
        steps: sim.steps(),
        ..Probe::default()
    };
    while done < budget {
        let q = sim.state();
        let read = sim.read();
        let Some(action) = machine.action(q, &read) else {
            break;
        };
        // A step that changes nothing repeats forever; there is nothing more to observe.
        let still = action.next == q
            && (0..machine.tapes()).all(|i| action.dir[i] == Dir::N && action.write[i] == read[i]);
        if still && done > 0 {
            break;
        }
        let origin = machine.origin(q, &read);
        if done > 0 && stops.at_transition(origin) {
            probe.hit = true;
            break;
        }
        if sim.step() == Status::Halted {
            break;
        }
        done += 1;
        probe.stepped = true;
        probe.last = origin;
        let mut printed = false;
        if let Some(emissions) = emissions.as_deref_mut() {
            printed = machine.is_final(action.next);
            let word = printed.then(|| printed_word(machine, sim.tapes().first()));
            emissions.observe(sim.steps(), word);
        }
        if stops.enters(q, action.next) || (stops.on_output && printed) {
            probe.hit = true;
            break;
        }
        if still {
            break;
        }
    }
    probe.steps = sim.steps();
    probe
}

fn state_name(machine: &Machine, q: State) -> String {
    machine
        .states()
        .get(q.index())
        .map_or_else(|| format!("#{}", q.index()), |s| s.name.clone())
}

fn direction(d: Dir) -> &'static str {
    match d {
        Dir::L => "left",
        Dir::R => "right",
        Dir::N => "nowhere",
    }
}

/// "walks off to the right in state C on blank cells" and the like.
fn describe_repeat(tapes: usize, state: &str, walking: &[(usize, Dir)]) -> String {
    if walking.is_empty() {
        return format!("stays in state {state} and repeats the same step forever");
    }
    if tapes == 1 {
        let d = walking.first().map_or(Dir::R, |w| w.1);
        return format!(
            "walks off to the {} in state {state} on blank cells",
            direction(d)
        );
    }
    let parts: Vec<String> = walking
        .iter()
        .map(|(i, d)| format!("to the {} on tape {}", direction(*d), i + 1))
        .collect();
    let joined = match parts.as_slice() {
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
        [] => String::new(),
    };
    format!("walks off {joined} in state {state} on blank cells")
}

fn describe_proof(machine: &Machine, proof: NonHaltProof) -> String {
    match proof {
        NonHaltProof::ChainToInfinity { state, dir } => format!(
            "walks off to the {} in state {} on blank cells",
            direction(dir),
            state_name(machine, state)
        ),
        NonHaltProof::StayLoop { state } => format!(
            "stays in state {} and repeats the same step forever",
            state_name(machine, state)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeats_are_described() {
        assert_eq!(
            describe_repeat(1, "C", &[(0, Dir::R)]),
            "walks off to the right in state C on blank cells"
        );
        assert_eq!(
            describe_repeat(1, "q0", &[]),
            "stays in state q0 and repeats the same step forever"
        );
        assert_eq!(
            describe_repeat(3, "q", &[(0, Dir::L), (2, Dir::R)]),
            "walks off to the left on tape 1 and to the right on tape 3 in state q on blank cells"
        );
    }

    #[test]
    fn emissions_index_every_segment() {
        let mut e = Emissions::new();
        for step in 1..=4 * SEGMENT {
            let word = step.is_multiple_of(1000).then(|| step.to_string());
            e.observe(step, word);
        }
        assert_eq!(e.counts, [0, 16, 32, 49, 65]);
        assert_eq!(e.count, 65);
        assert_eq!(e.recent.len(), EMITTED_SHOWN);
        assert_eq!(e.recent.front().map(|(s, _)| *s), Some(16_000));
        assert_eq!(e.recent.back(), Some(&(65_000, "65000".to_string())));
    }

    mod runs {
        use super::*;
        use crate::dto::{DirDto, KindDto};
        use crate::testing::{self, multi, spec, t};

        fn open(dto: &MachineSpecDto, input: &str) -> Session {
            Session::new(dto, input).expect("valid session")
        }

        fn no_breakpoints() -> BreakpointsDto {
            BreakpointsDto::default()
        }

        #[test]
        fn increment_steps_to_halt_and_back() {
            let mut s = open(&testing::increment(), "1101");
            let start = s.snapshot(3);
            assert_eq!(start.step, 0.0);
            assert_eq!(start.state, "q0");
            assert_eq!(start.status, RunStatusDto::Running);
            assert_eq!(start.configuration, "q0 1101");
            assert_eq!(start.next_transition, Some(0));
            assert_eq!(start.last_transition, None);
            assert_eq!(start.tapes[0].start, -3.0);
            assert_eq!(start.tapes[0].cells.concat(), "␣␣␣1101");
            assert_eq!(start.ones, 4.0);

            s.step(1);
            let one = s.snapshot(3);
            assert_eq!(one.step, 1.0);
            assert_eq!(one.last_transition, Some(0));
            assert_eq!(one.next_transition, Some(0));
            assert_eq!(one.tapes[0].head, 1.0);

            s.step(1);
            let two = s.snapshot(3);
            assert_eq!(two.next_transition, Some(1));
            s.step(10);
            let end = s.snapshot(3);
            assert_eq!(end.step, 3.0);
            assert_eq!(end.status, RunStatusDto::Halted);
            assert_eq!(end.state, "q1");
            assert_eq!(end.last_transition, Some(1));
            assert_eq!(end.next_transition, None);
            // 1011 + 1 = 1100 in LSB-first order: 1101 is 11, the result 0011 is 12.
            let output = end.output.clone().expect("halted transducer has an output");
            assert_eq!(output.word, "0011");
            assert_eq!(output.value.as_deref(), Some("12"));
            assert_eq!(end.extent, [0.0, 3.0]);

            s.back(2);
            assert_eq!(s.snapshot(3), one);
            s.back(100);
            assert_eq!(s.snapshot(3), start);
            s.seek(3);
            assert_eq!(s.snapshot(3), end);
            s.seek(2);
            assert_eq!(s.snapshot(3), two);
        }

        #[test]
        fn bb5_runs_to_halt() {
            let mut s = open(&testing::standard(testing::BB5), "");
            let mut total = 0.0;
            let result = loop {
                let r = s.run(10_000_000, &no_breakpoints(), 0);
                total += r.steps_done;
                if r.reason != StopReasonDto::Budget {
                    break r;
                }
            };
            assert_eq!(result.reason, StopReasonDto::Halted);
            assert_eq!(total, 47_176_870.0);
            assert_eq!(result.snapshot.step, 47_176_870.0);
            assert_eq!(result.snapshot.ones, 4_098.0);
            assert_eq!(result.snapshot.status, RunStatusDto::Halted);
            // Back and forth near the end is exact.
            s.back(1);
            assert_eq!(s.snapshot(0).step, 47_176_869.0);
            assert_eq!(s.snapshot(0).status, RunStatusDto::Running);
            s.step(1);
            assert_eq!(s.snapshot(0), result.snapshot);
        }

        #[test]
        fn transition_breakpoints_stop_before_and_resume() {
            let mut s = open(&testing::standard(testing::BB4), "");
            let mut plain = open(&testing::standard(testing::BB4), "");
            let bp = BreakpointsDto {
                transitions: vec![3],
                ..BreakpointsDto::default()
            };
            let mut stops = Vec::new();
            loop {
                let r = s.run(1_000, &bp, 0);
                if r.reason != StopReasonDto::Breakpoint {
                    assert_eq!(r.reason, StopReasonDto::Halted);
                    break;
                }
                assert_eq!(r.snapshot.next_transition, Some(3));
                stops.push(r.snapshot.step);
            }
            assert_ne!(stops.len(), 0);
            // Every stop is exactly a step before which transition 3 applies.
            for &step in &stops {
                plain.seek(convert::count(step));
                assert_eq!(plain.snapshot(0).next_transition, Some(3));
            }
            assert_eq!(s.snapshot(0).step, 107.0);
        }

        #[test]
        fn state_breakpoints_stop_after_entering() {
            let mut s = open(&testing::standard(testing::BB4), "");
            let bp = BreakpointsDto {
                states: vec!["C".into()],
                ..BreakpointsDto::default()
            };
            let r = s.run(1_000, &bp, 0);
            assert_eq!(r.reason, StopReasonDto::Breakpoint);
            assert_eq!(r.snapshot.state, "C");
            let at = r.snapshot.step;
            // The step before was in another state.
            s.back(1);
            assert_ne!(s.snapshot(0).state, "C");
            s.seek(convert::count(at));
            let again = s.run(1_000, &bp, 0);
            assert!(again.steps_done > 0.0);
        }

        #[test]
        fn chain_steps_stop_exactly_at_breakpoints() {
            // A long scan: q0 moves right over 1000 x's, then halts on the blank.
            let mut dto = spec(
                KindDto::Transducer,
                1,
                vec![t("q0", "x", "q0", "y", DirDto::R)],
                &[],
            );
            dto.input_alphabet = vec!["x".into()];
            let mut s = open(&dto, &"x".repeat(1_000));
            let bp = BreakpointsDto {
                transitions: vec![0],
                ..BreakpointsDto::default()
            };
            // The first step of a run passes the breakpoint; every next step stops at it.
            for expected in 1..=5 {
                let r = s.run(1_000_000, &bp, 0);
                assert_eq!(r.reason, StopReasonDto::Breakpoint);
                assert_eq!(r.steps_done, 1.0);
                assert_eq!(r.snapshot.step, f64::from(expected));
            }
            let r = s.run(1_000_000, &no_breakpoints(), 0);
            assert_eq!(r.reason, StopReasonDto::Halted);
            assert_eq!(r.snapshot.step, 1_000.0);
            assert_eq!(r.snapshot.last_transition, Some(0));
        }

        #[test]
        fn budgets_are_never_exceeded() {
            let mut s = open(&testing::standard(testing::BB5), "");
            for budget in [1, 2, 3, 1_000, 12_345, 1_000_000] {
                let before = s.steps();
                let r = s.run(budget, &no_breakpoints(), 0);
                assert_eq!(r.reason, StopReasonDto::Budget);
                assert_eq!(s.steps() - before, budget);
                assert_eq!(r.steps_done, number(budget));
            }
        }

        #[test]
        fn walking_off_never_halts() {
            let dto = spec(
                KindDto::Transducer,
                1,
                vec![
                    t("A", "␣", "B", "1", DirDto::R),
                    t("B", "␣", "B", "␣", DirDto::R),
                ],
                &[],
            );
            let mut s = open(&dto, "");
            assert_eq!(s.snapshot(0).status, RunStatusDto::Running);
            let r = s.run(1_000, &no_breakpoints(), 0);
            assert_eq!(r.reason, StopReasonDto::NeverHalts);
            assert_eq!(r.snapshot.status, RunStatusDto::NeverHalts);
            assert_eq!(
                r.snapshot.reason.as_deref(),
                Some("walks off to the right in state B on blank cells")
            );
            // A further run stops at once.
            let again = s.run(1_000, &no_breakpoints(), 0);
            assert_eq!(again.reason, StopReasonDto::NeverHalts);
            assert_eq!(again.steps_done, 0.0);
        }

        #[test]
        fn two_tapes_use_the_reference_simulator() {
            // Copies the input to the second tape, then walks off on both tapes.
            let dto = spec(
                KindDto::Transducer,
                2,
                vec![
                    multi("c", &["x", "␣"], "c", &["x", "x"], &[DirDto::R, DirDto::R]),
                    multi("c", &["␣", "␣"], "w", &["␣", "␣"], &[DirDto::N, DirDto::L]),
                    multi("w", &["␣", "x"], "w", &["␣", "x"], &[DirDto::N, DirDto::L]),
                    multi("w", &["␣", "␣"], "z", &["␣", "␣"], &[DirDto::R, DirDto::N]),
                    multi("z", &["␣", "␣"], "z", &["y", "␣"], &[DirDto::R, DirDto::N]),
                ],
                &[],
            );
            let mut s = open(&dto, "xxx");
            assert!(matches!(s.engine, Engine::Exact(_)));
            s.step(4);
            let snap = s.snapshot(1);
            assert_eq!(snap.tapes.len(), 2);
            assert_eq!(snap.configuration.lines().count(), 2);
            let r = s.run(100, &no_breakpoints(), 1);
            assert_eq!(r.reason, StopReasonDto::NeverHalts);
            assert_eq!(
                r.snapshot.reason.as_deref(),
                Some("walks off to the right on tape 1 in state z on blank cells")
            );
            s.back(3);
            s.step(3);
            assert_eq!(s.snapshot(1), r.snapshot);
        }

        #[test]
        fn staying_in_place_never_halts() {
            let dto = spec(
                KindDto::Transducer,
                2,
                vec![multi(
                    "q",
                    &["␣", "␣"],
                    "q",
                    &["␣", "␣"],
                    &[DirDto::N, DirDto::N],
                )],
                &[],
            );
            let mut s = open(&dto, "");
            let r = s.run(1_000, &no_breakpoints(), 0);
            assert_eq!(r.reason, StopReasonDto::NeverHalts);
            assert_eq!(
                r.snapshot.reason.as_deref(),
                Some("stays in state q and repeats the same step forever")
            );
        }

        #[test]
        fn acceptors_report_verdicts() {
            let mut dto = spec(
                KindDto::Acceptor,
                1,
                vec![
                    t("even", "a", "odd", "a", DirDto::R),
                    t("odd", "a", "even", "a", DirDto::R),
                ],
                &["even"],
            );
            dto.input_alphabet = vec!["a".into()];
            let mut s = open(&dto, "aaaa");
            s.step(100);
            assert_eq!(s.snapshot(0).verdict, Some(VerdictKindDto::Accepted));
            let mut s = open(&dto, "aaa");
            s.step(100);
            assert_eq!(s.snapshot(0).verdict, Some(VerdictKindDto::Rejected));
            assert_eq!(s.snapshot(0).output, None);
        }

        #[test]
        fn invalid_specs_and_inputs_are_errors() {
            let mut dto = testing::increment();
            dto.transitions[0].to = String::new();
            dto.transitions[1].read = vec!["01".into()];
            let error = Session::new(&dto, "").unwrap_err();
            assert_eq!(
                error.to_string(),
                "the read symbol of transition 2 must be one character, not `01`"
            );
            let error = Session::new(&testing::increment(), "12").unwrap_err();
            assert!(error.to_string().contains("'2'"), "{error}");
        }

        #[test]
        fn radius_is_capped() {
            let s = open(&testing::increment(), "1");
            let snap = s.snapshot(u32::MAX);
            assert_eq!(snap.tapes[0].cells.len(), 2 * MAX_RADIUS as usize + 1);
        }

        #[test]
        fn enumerators_print_exactly_in_both_directions() {
            let dto = testing::tally_enumerator();
            let mut a = open(&dto, "");
            a.step(5_000);
            let mut b = open(&dto, "");
            for _ in 0..5_000 {
                b.step(1);
            }
            let at_5000 = a.snapshot(2);
            assert_eq!(b.snapshot(2), at_5000);
            assert!(at_5000.emitted_count > 50.0);
            assert_eq!(at_5000.emitted.len(), 50);
            let n = at_5000.emitted.last().map_or(0, String::len);
            assert_eq!(number(n as u64), at_5000.emitted_count);

            // Far beyond several index segments, then back and forth.
            a.seek(100_000);
            let far = a.snapshot(2);
            let longest = far.emitted.last().cloned().unwrap_or_default();
            assert_eq!(longest.chars().count(), MAX_WORD);
            assert!(longest.contains('…'));
            for target in [99_999, 70_000, 40_000, 16_384, 16_383, 2_500, 1, 0] {
                a.seek(target);
                let mut fresh = open(&dto, "");
                fresh.seek(target);
                assert_eq!(a.snapshot(2), fresh.snapshot(2), "step {target}");
            }
            a.seek(100_000);
            assert_eq!(a.snapshot(2), far);
            a.seek(5_000);
            assert_eq!(a.snapshot(2), at_5000);
        }

        #[test]
        fn output_breakpoints_stop_after_printing() {
            let dto = testing::tally_enumerator();
            let mut s = open(&dto, "");
            let bp = BreakpointsDto {
                on_output: true,
                ..BreakpointsDto::default()
            };
            for words in 1..=5_usize {
                let r = s.run(1_000_000, &bp, 0);
                assert_eq!(r.reason, StopReasonDto::Breakpoint);
                assert_eq!(r.snapshot.state, "p");
                assert_eq!(r.snapshot.emitted_count, number(words as u64));
                assert_eq!(r.snapshot.emitted.last().map(String::len), Some(words));
            }
        }
    }
}
