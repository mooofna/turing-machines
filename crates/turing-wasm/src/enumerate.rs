//! The busy beaver side: enumeration in tree normal form and the decider pipeline.

use std::fmt;

use turing_bb::compact::{MAX_STATES, MAX_SYMBOLS};
use turing_bb::{CompactAction, CompactTm, DeciderId, Enumerator, Next, Pipeline, Stats, Verdict};
use turing_core::{Dir, Machine, State};

use crate::Error;
use crate::convert::{self, number};
use crate::dto::{
    DeciderCountDto, EnumProgressDto, MachineListDto, MachineSpecDto, RecordDto, VerdictDto,
};

/// An enumeration of one shard of all machines of one size.
pub(crate) struct Job {
    enumerator: Enumerator,
}

impl fmt::Debug for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnumerationJob")
            .field("nodes", &self.enumerator.stats().nodes)
            .field("done", &self.enumerator.is_done())
            .finish_non_exhaustive()
    }
}

impl Job {
    /// Enumerates shard `shard` of `shards` of the `states`-state, `symbols`-symbol machines.
    ///
    /// # Errors
    ///
    /// Fails for unsupported sizes or shards.
    pub(crate) fn new(states: u8, symbols: u8, shard: u32, shards: u32) -> Result<Self, Error> {
        let enumerator = Enumerator::new(states, symbols)
            .and_then(|e| e.with_shard(shard, shards))
            .map_err(|e| Error::new(e.to_string()))?;
        Ok(Self { enumerator })
    }

    /// Visits at most `max_nodes` more nodes.
    pub(crate) fn advance(&mut self, max_nodes: u64) -> EnumProgressDto {
        if max_nodes > 0 && !self.enumerator.is_done() {
            self.enumerator.advance(max_nodes);
        }
        progress(self.enumerator.stats(), self.enumerator.is_done())
    }

    /// Up to `limit` machines resolved by the decider with key `key`; none for unknown keys.
    pub(crate) fn examples(&self, key: &str, limit: u32) -> MachineListDto {
        let machines = DeciderId::ALL
            .iter()
            .find(|d| d.key() == key)
            .map(|&id| list(self.enumerator.examples(id), limit))
            .unwrap_or_default();
        MachineListDto { machines }
    }

    /// Up to `limit` unresolved machines.
    pub(crate) fn holdouts(&self, limit: u32) -> MachineListDto {
        MachineListDto {
            machines: list(self.enumerator.holdouts(), limit),
        }
    }
}

fn list(machines: &[CompactTm], limit: u32) -> Vec<String> {
    let limit = usize::try_from(limit).unwrap_or(usize::MAX);
    machines
        .iter()
        .take(limit)
        .map(ToString::to_string)
        .collect()
}

/// Progress in the DTO form: one entry per decider of [`DeciderId::ALL`] in pipeline order,
/// with the counts of all stages that use that decider added up.
fn progress(stats: &Stats, done: bool) -> EnumProgressDto {
    let record = |r: &Option<(u64, CompactTm)>| {
        r.as_ref().map(|(value, tm)| RecordDto {
            value: number(*value),
            machine: tm.to_string(),
        })
    };
    EnumProgressDto {
        nodes: number(stats.nodes),
        halting: number(stats.halting),
        deciders: DeciderId::ALL
            .iter()
            .map(|&id| DeciderCountDto {
                key: id.key().into(),
                label: id.label().into(),
                non_halting: number(
                    stats
                        .by_decider
                        .iter()
                        .filter(|(d, _)| *d == id)
                        .map(|(_, n)| n)
                        .sum(),
                ),
            })
            .collect(),
        holdouts: number(stats.holdouts),
        max_steps: record(&stats.max_steps),
        max_ones: record(&stats.max_ones),
        done,
    }
}

/// A machine prepared for the decider pipeline.
#[derive(Clone, Debug)]
struct Prepared {
    tm: CompactTm,
    /// For each state and symbol of `tm`, the symbol written by a transition into a halting
    /// state; such transitions are left undefined in `tm`.
    halting: [[Option<u8>; MAX_SYMBOLS]; MAX_STATES],
}

/// Converts a compiled machine for the pipeline.
///
/// Working states are the start state and every state with a transition; at most
/// [`MAX_STATES`] are supported, the start state becomes `A`. States without transitions are
/// halting states, any number of them: a transition into one is the halting transition of the
/// busy beaver convention (`1RZ` in the standard format).
fn prepare(machine: &Machine, input: &str) -> Result<Prepared, String> {
    if machine.tapes() != 1 {
        return Err(format!(
            "the deciders handle single-tape machines, this one has {} tapes",
            machine.tapes()
        ));
    }
    if !input.is_empty() {
        return Err(format!(
            "the deciders start on the blank tape, but this machine has the input `{}`",
            convert::shorten(input, 24)
        ));
    }
    let symbols = machine.num_symbols();
    if symbols > MAX_SYMBOLS {
        return Err(format!(
            "the deciders handle at most {MAX_SYMBOLS} symbols, this machine has {symbols}"
        ));
    }
    let mut working = vec![false; machine.num_states()];
    if let Some(w) = working.get_mut(machine.start().index()) {
        *w = true;
    }
    for t in machine.transitions() {
        if let Some(w) = working.get_mut(t.from.index()) {
            *w = true;
        }
    }
    // Compact index of each working state, the start state first.
    let mut index: Vec<Option<u8>> = vec![None; machine.num_states()];
    let order = std::iter::once(machine.start().index())
        .chain((0..machine.num_states()).filter(|&q| q != machine.start().index()));
    let mut next = 0u8;
    for q in order {
        if working.get(q).copied().unwrap_or(false) {
            if usize::from(next) == MAX_STATES {
                let n = working.iter().filter(|&&w| w).count();
                return Err(format!(
                    "the deciders handle at most {MAX_STATES} states with transitions, this machine has {n}"
                ));
            }
            if let Some(slot) = index.get_mut(q) {
                *slot = Some(next);
            }
            next += 1;
        }
    }
    let symbols = u8::try_from(symbols.max(2)).map_err(|_| "too many symbols".to_string())?;
    let mut tm = CompactTm::new(next, symbols);
    let mut halting = [[None; MAX_SYMBOLS]; MAX_STATES];
    for t in machine.transitions() {
        let Some(q) = index.get(t.from.index()).copied().flatten() else {
            continue;
        };
        let a = t.read[0].get();
        let write = t.action.write[0].get();
        let dir = t.action.dir[0];
        if dir == Dir::N {
            return Err(format!(
                "the deciders handle L and R moves, but δ({}, {}) does not move",
                state_name(machine, t.from),
                machine.alphabet().glyph(t.read[0])
            ));
        }
        match index.get(t.action.next.index()).copied().flatten() {
            Some(target) => tm.set(
                q,
                a,
                Some(CompactAction {
                    write,
                    dir,
                    next: Next::State(target),
                }),
            ),
            None => {
                if let Some(slot) = halting
                    .get_mut(usize::from(q))
                    .and_then(|row| row.get_mut(usize::from(a)))
                {
                    *slot = Some(write);
                }
            }
        }
    }
    Ok(Prepared { tm, halting })
}

fn state_name(machine: &Machine, q: State) -> String {
    machine
        .states()
        .get(q.index())
        .map_or_else(|| format!("#{}", q.index()), |s| s.name.clone())
}

/// Runs the decider pipeline on `spec` started on the blank tape.
///
/// A halting verdict follows the busy beaver convention: if the machine halts by a transition
/// into a state without transitions (`1RZ`), that transition is counted, so `steps` is the
/// number of steps the pipeline simulated plus 1, and `ones` counts the non-blank cells after
/// its write. If the machine instead halts because δ is undefined in a state that has other
/// transitions, `steps` and `ones` are the steps actually made and the non-blank cells at that
/// moment.
///
/// Specs with errors and machines the pipeline cannot take (see [`prepare`]) are
/// `unsupported`, with the reason.
pub(crate) fn decide(dto: &MachineSpecDto) -> VerdictDto {
    let unsupported = |reason: String| VerdictDto::Unsupported { reason };
    let spec = match convert::spec(dto) {
        Ok(spec) => spec,
        Err(problems) => return unsupported(convert::first_error(&problems).to_string()),
    };
    let machine = match spec.compile() {
        Ok(machine) => machine,
        Err(d) => {
            return unsupported(format!(
                "the machine has errors: {}",
                convert::first_diagnostic(&d)
            ));
        }
    };
    let prepared = match prepare(&machine, &spec.input) {
        Ok(p) => p,
        Err(reason) => return unsupported(reason),
    };
    match Pipeline::standard().decide(&prepared.tm) {
        Verdict::Halts { steps, ones, at } => {
            let halting = prepared
                .halting
                .get(usize::from(at.0))
                .and_then(|row| row.get(usize::from(at.1)))
                .copied()
                .flatten();
            let (steps, ones) = match halting {
                Some(write) => (
                    steps.saturating_add(1),
                    ones.saturating_sub(u64::from(at.1 != 0))
                        .saturating_add(u64::from(write != 0)),
                ),
                None => (steps, ones),
            };
            VerdictDto::Halts {
                steps: number(steps),
                ones: number(ones),
            }
        }
        Verdict::NonHalting(id) => VerdictDto::NonHalting {
            decider: id.key().into(),
            label: id.label().into(),
        },
        Verdict::Unknown => VerdictDto::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{DirDto, KindDto};
    use crate::testing::{self, multi, spec, t};

    #[test]
    fn cyclers_never_halt() {
        let verdict = decide(&testing::standard("0RB---_0LA---"));
        assert!(
            matches!(&verdict, VerdictDto::NonHalting { decider, .. } if decider == "loops"),
            "{verdict:?}"
        );
    }

    #[test]
    fn champions_halt_in_the_busy_beaver_convention() {
        for (text, steps, ones) in [
            ("1RB1LB_1LA1RZ", 6.0, 4.0),
            ("1RB1RZ_0RC1RB_1LC1LA", 21.0, 5.0),
            (testing::BB4, 107.0, 13.0),
        ] {
            assert_eq!(
                decide(&testing::standard(text)),
                VerdictDto::Halts { steps, ones },
                "{text}"
            );
        }
    }

    #[test]
    fn undefined_transitions_count_the_steps_made() {
        // A0 -> 1RB, B0 -> 1LA, then A1 is undefined: two steps, two ones.
        assert_eq!(
            decide(&testing::standard("1RB---_1LA---")),
            VerdictDto::Halts {
                steps: 2.0,
                ones: 2.0
            }
        );
    }

    #[test]
    fn course_machines_with_final_states() {
        // On the blank tape, increment writes 1 and enters q1, which has no transitions.
        let mut spec = testing::increment();
        spec.input = String::new();
        assert_eq!(
            decide(&spec),
            VerdictDto::Halts {
                steps: 1.0,
                ones: 1.0
            }
        );
    }

    #[test]
    fn unsupported_machines_have_reasons() {
        let reason = |v: VerdictDto| match v {
            VerdictDto::Unsupported { reason } => reason,
            other => panic!("expected unsupported, got {other:?}"),
        };
        assert!(reason(decide(&testing::increment())).contains("blank tape"));
        let two = spec(
            KindDto::Transducer,
            2,
            vec![multi(
                "q",
                &["␣", "␣"],
                "q",
                &["x", "x"],
                &[DirDto::R, DirDto::R],
            )],
            &[],
        );
        assert!(reason(decide(&two)).contains("single-tape"));
        let still = spec(
            KindDto::Beaver,
            1,
            vec![t("A", "␣", "B", "1", DirDto::N)],
            &[],
        );
        assert!(reason(decide(&still)).contains("does not move"));
        let names = ["A", "B", "C", "D", "E", "F", "G"];
        let seven = spec(
            KindDto::Beaver,
            1,
            names
                .iter()
                .zip(names.iter().cycle().skip(1))
                .map(|(a, b)| t(a, "␣", b, "1", DirDto::R))
                .collect(),
            &[],
        );
        assert!(reason(decide(&seven)).contains("at most 6 states"));
        let mut broken = testing::increment();
        broken.tapes = 9;
        assert!(reason(decide(&broken)).contains("errors"));
    }

    #[test]
    fn enumerates_two_and_three_states() {
        let mut job = Job::new(2, 2, 0, 1).unwrap();
        let mut progress = job.advance(0);
        assert_eq!(progress.nodes, 0.0);
        while !progress.done {
            progress = job.advance(7);
        }
        assert_eq!(progress.nodes, 61.0);
        assert_eq!(progress.max_steps.as_ref().map(|r| r.value), Some(6.0));
        assert_eq!(progress.max_ones.as_ref().map(|r| r.value), Some(4.0));
        assert_eq!(progress.holdouts, 0.0);
        let keys: Vec<&str> = progress.deciders.iter().map(|d| d.key.as_str()).collect();
        let all: Vec<&str> = DeciderId::ALL.iter().map(|d| d.key()).collect();
        assert_eq!(keys, all);
        assert_ne!(job.examples("loops", 3).machines.len(), 0);
        assert!(job.examples("loops", 3).machines.len() <= 3);
        assert_eq!(
            job.examples("no-such-decider", 3).machines,
            Vec::<String>::new()
        );
        assert_eq!(job.holdouts(10).machines, Vec::<String>::new());

        let mut job = Job::new(3, 2, 0, 1).unwrap();
        let progress = loop {
            let p = job.advance(1_000);
            if p.done {
                break p;
            }
        };
        assert_eq!(progress.nodes, 5_417.0);
        assert_eq!(progress.max_steps.map(|r| r.value), Some(21.0));
        assert_eq!(progress.max_ones.map(|r| r.value), Some(6.0));
        assert_eq!(progress.holdouts, 0.0);
    }

    #[test]
    fn shards_add_up() {
        let mut nodes = 0.0;
        for shard in 0..3 {
            let mut job = Job::new(3, 2, shard, 3).unwrap();
            let p = loop {
                let p = job.advance(10_000);
                if p.done {
                    break p;
                }
            };
            nodes += p.nodes;
        }
        assert_eq!(nodes, 5_417.0);
        assert!(Job::new(3, 2, 3, 3).is_err());
        assert!(Job::new(0, 2, 0, 1).is_err());
        assert!(Job::new(3, 0, 0, 1).is_err());
        assert!(Job::new(200, 200, 0, 1).is_err());
    }
}
