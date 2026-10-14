//! Soundness checks: no decider claims that a halting machine never halts, and machines
//! claimed non-halting do not halt for a long time.
//!
//! The halting nodes of the tree normal form are a demanding test set: each reaches an
//! undefined transition, so a decider that answered "never halts" for one of them would be
//! wrong. Every decider of every Coq-BB5 pipeline used here, and [`Loops`] with every step
//! budget below the halting time, is run on every halting node.

use turing_bb::deciders::{Loops, NGramCps, RepWl};
use turing_bb::{CompactTm, Decider, Enumerator, Verdict};

mod common;

/// Every decider with every parameter set used by Coq-BB5 for BB(2), BB(3), BB(4) and BB(2,3).
fn all_deciders() -> Vec<Box<dyn Decider>> {
    let mut list: Vec<Box<dyn Decider>> = vec![
        Box::new(Loops::new(6)),
        Box::new(Loops::new(21)),
        Box::new(Loops::new(38)),
        Box::new(Loops::new(107)),
        Box::new(Loops::new(1_000)),
    ];
    for (l, r, gas) in [
        (1, 1, 100),
        (2, 2, 200),
        (3, 3, 400),
        (1, 1, 400),
        (2, 2, 800),
    ] {
        list.push(Box::new(NGramCps::plain(l, r, gas)));
    }
    for (h, l, r, gas) in [
        (2, 2, 2, 1600),
        (2, 3, 3, 1600),
        (4, 2, 2, 600),
        (4, 3, 3, 1600),
        (6, 2, 2, 3200),
        (6, 3, 3, 3200),
        (8, 2, 2, 1600),
        (8, 3, 3, 1600),
        (10, 4, 4, 10_000),
    ] {
        list.push(Box::new(NGramCps::history(h, l, r, gas)));
    }
    list.push(Box::new(NGramCps::lru(2, 2, 10_000)));
    list.push(Box::new(RepWl::new(4, 3, 320, 10_000)));
    list.push(Box::new(RepWl::new(2, 3, 320, 400)));
    list
}

/// The halting nodes of an enumeration with their halting time (steps before the undefined
/// transition).
fn halting_nodes(states: u8, symbols: u8) -> Vec<(CompactTm, u64)> {
    let mut nodes = Vec::new();
    let mut e = Enumerator::new(states, symbols).expect("supported size");
    while !e.is_done() {
        e.advance_with(1 << 16, |v| {
            if let Verdict::Halts { steps, .. } = v.verdict {
                nodes.push((*v.tm, steps));
            }
        });
    }
    nodes
}

/// The nodes proved non-halting by the enumeration.
fn non_halting_nodes(states: u8, symbols: u8) -> Vec<CompactTm> {
    let mut nodes = Vec::new();
    let mut e = Enumerator::new(states, symbols).expect("supported size");
    while !e.is_done() {
        e.advance_with(1 << 16, |v| {
            if let Verdict::NonHalting(_) = v.verdict {
                nodes.push(*v.tm);
            }
        });
    }
    nodes
}

/// Runs every decider on every halting node and [`Loops`] with the step budgets in
/// `budgets(t)` for a node with halting time t.
fn check_halting_nodes(states: u8, symbols: u8, budgets: impl Fn(u64) -> Vec<u32>) {
    let nodes = halting_nodes(states, symbols);
    assert_ne!(nodes.len(), 0);
    let mut deciders = all_deciders();
    for (tm, t) in &nodes {
        let (steps, _) = common::halts_within(tm, 10_000).expect("halting node halts");
        assert_eq!(steps, *t, "{tm}");
        for decider in &mut deciders {
            match decider.decide(tm) {
                Verdict::NonHalting(_) => panic!("{} claims {tm} never halts", decider.name()),
                Verdict::Halts { steps, .. } => assert_eq!(steps, *t, "{}: {tm}", decider.name()),
                Verdict::Unknown => {}
            }
        }
        for n in budgets(*t) {
            let verdict = Loops::new(n).decide(tm);
            if u64::from(n) > *t {
                assert!(matches!(verdict, Verdict::Halts { .. }), "{n}: {tm}");
            } else {
                assert_eq!(verdict, Verdict::Unknown, "Loops({n}) on {tm}");
            }
        }
    }
}

/// Every budget from 0 to one past the halting time.
fn every_budget(t: u64) -> Vec<u32> {
    (0..=u32::try_from(t).expect("short run") + 1).collect()
}

#[test]
fn no_decider_rejects_a_halting_node_up_to_three_states() {
    for (states, symbols) in [(1, 2), (2, 2), (3, 2), (2, 3)] {
        check_halting_nodes(states, symbols, every_budget);
    }
}

#[test]
#[ignore = "about ten seconds in release mode; run with --release -- --include-ignored"]
fn no_decider_rejects_a_halting_node_with_four_states() {
    check_halting_nodes(4, 2, |t| {
        let t = u32::try_from(t).expect("short run");
        vec![0, 1, 2, t / 2, t.saturating_sub(1), t, t + 1]
    });
}

#[test]
fn non_halting_nodes_run_for_a_long_time() {
    for (states, symbols) in [(1, 2), (2, 2), (3, 2), (2, 3)] {
        for tm in non_halting_nodes(states, symbols) {
            assert_eq!(common::halts_within(&tm, 100_000), None, "{tm}");
        }
    }
}
