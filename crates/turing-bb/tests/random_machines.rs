//! Property tests on random machines: the standard format round-trips, and every verdict of
//! every decider agrees with plain simulation.

use proptest::prelude::*;
use turing_bb::deciders::{Loops, NGramCps, RepWl};
use turing_bb::{CompactAction, CompactTm, Decider, Next, Verdict};
use turing_core::Dir;

mod common;

/// A random machine with 1 to 5 states and 2 or 3 symbols; about one transition in eight is
/// undefined and a few halt explicitly.
fn machine() -> impl Strategy<Value = CompactTm> {
    (1u8..=5, 2u8..=3).prop_flat_map(|(states, symbols)| {
        let entry = (0u8..16, 0..symbols, any::<bool>(), 0..states);
        proptest::collection::vec(entry, usize::from(states * symbols)).prop_map(move |entries| {
            let mut tm = CompactTm::new(states, symbols);
            for (i, (kind, write, right, next)) in entries.into_iter().enumerate() {
                let i = u8::try_from(i).expect("small table");
                let next = match kind {
                    0 | 1 => continue,
                    2 => Next::Halt,
                    _ => Next::State(next),
                };
                let dir = if right { Dir::R } else { Dir::L };
                tm.set(
                    i / symbols,
                    i % symbols,
                    Some(CompactAction { write, dir, next }),
                );
            }
            tm
        })
    })
}

fn deciders() -> Vec<Box<dyn Decider>> {
    vec![
        Box::new(Loops::new(107)),
        Box::new(Loops::new(300)),
        Box::new(NGramCps::plain(1, 1, 400)),
        Box::new(NGramCps::plain(2, 2, 800)),
        Box::new(NGramCps::plain(3, 3, 400)),
        Box::new(NGramCps::history(2, 2, 2, 1600)),
        Box::new(NGramCps::history(4, 3, 3, 1600)),
        Box::new(NGramCps::lru(2, 2, 2000)),
        Box::new(RepWl::new(2, 3, 320, 400)),
        Box::new(RepWl::new(4, 3, 320, 2000)),
    ]
}

/// Steps before the machine reaches an undefined or halting transition, the non-blank cells
/// at that moment and the state-symbol pair, by plain simulation.
fn reach_halt(tm: &CompactTm, fuel: u64) -> Option<(u64, u64, (u8, u8))> {
    let mut tape = std::collections::BTreeMap::<i64, u8>::new();
    let (mut head, mut state) = (0i64, 0u8);
    for step in 0..fuel {
        let symbol = tape.get(&head).copied().unwrap_or(0);
        let action = tm.get(state, symbol);
        let Some(CompactAction {
            write,
            dir,
            next: Next::State(next),
        }) = action
        else {
            let ones = tape.values().filter(|&&c| c != 0).count() as u64;
            return Some((step, ones, (state, symbol)));
        };
        tape.insert(head, write);
        head += if dir == Dir::R { 1 } else { -1 };
        state = next;
    }
    None
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    #[test]
    fn the_standard_format_round_trips(tm in machine()) {
        let text = tm.to_string();
        prop_assert_eq!(text.parse::<CompactTm>().unwrap(), tm);
    }

    #[test]
    fn verdicts_agree_with_simulation(tm in machine()) {
        let truth = reach_halt(&tm, 20_000);
        for mut decider in deciders() {
            match decider.decide(&tm) {
                Verdict::NonHalting(_) => prop_assert!(
                    truth.is_none(),
                    "{} claims {} never halts, it halts after {:?}",
                    decider.name(), tm, truth
                ),
                Verdict::Halts { steps, ones, at } => {
                    prop_assert_eq!(truth, Some((steps, ones, at)), "{} on {}", decider.name(), tm);
                }
                Verdict::Unknown => {}
            }
        }
        if let Some((steps, _, _)) = truth {
            // The busy beaver convention counts the halting step.
            let mut completed = tm;
            let (q, a) = truth.unwrap().2;
            if completed.get(q, a).is_none() {
                completed.set(q, a, Some(CompactAction { write: 1, dir: Dir::R, next: Next::Halt }));
            }
            prop_assert_eq!(common::halts_within(&completed, 20_001).map(|r| r.0), Some(steps + 1));
        }
    }
}
