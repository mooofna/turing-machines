//! Antihydra: the machine follows its rules, and the rule walk follows the machine.

use std::time::Instant;

use num_bigint::BigUint;
use turing_bb::cryptids::antihydra::{MachineCheck, Row, RuleWalk};

/// The first `n` rows of the machine check.
fn machine_rows(n: usize) -> Vec<Row> {
    let mut check = MachineCheck::new();
    let mut rows = Vec::new();
    while rows.len() < n && !check.is_done() {
        rows.extend(check.advance(1 << 20));
    }
    rows.truncate(n);
    rows
}

#[test]
fn the_machine_follows_the_rules() {
    let rows = machine_rows(30);
    assert_eq!(rows.len(), 30);
    for (i, row) in rows.iter().enumerate() {
        assert!(row.agrees, "{row:?}");
        assert_eq!(row.iteration as usize, i);
        assert_eq!(row.delta, row.predicted);
    }
    let first: Vec<(u64, u64, u64)> = rows[..6].iter().map(|r| (r.a, r.b, r.steps)).collect();
    assert_eq!(
        first,
        [
            (0, 4, 11),
            (2, 8, 58),
            (4, 14, 169),
            (6, 23, 419),
            (5, 36, 919),
            (7, 56, 2128)
        ]
    );
}

#[test]
fn the_walk_matches_the_machine() {
    let rows = machine_rows(30);
    let mut walk = RuleWalk::new();
    for row in &rows {
        let summary = walk.summary(0);
        assert_eq!(summary.iterations, u64::from(row.iteration));
        assert_eq!(summary.balance, i64::try_from(row.a).unwrap(), "{row:?}");
        assert_eq!(*walk.h(), BigUint::from(row.b + 4), "{row:?}");
        walk.advance(1);
    }
}

#[test]
fn the_walk_is_fast() {
    let start = Instant::now();
    let mut walk = RuleWalk::new();
    walk.advance(100_000);
    let elapsed = start.elapsed();
    let summary = walk.summary(100);
    assert_eq!(summary.iterations, 100_000);
    assert!(!summary.halted, "Antihydra would halt: {summary:?}");
    assert_eq!(summary.samples.len(), 100);
    // h = 8 grows by a factor of about 3/2 per iteration: 3 + 100000 log2(3/2) = 58499.25.
    assert!(
        (58_490..=58_500).contains(&summary.bits),
        "{} bits",
        summary.bits
    );
    if cfg!(not(debug_assertions)) {
        assert!(elapsed.as_secs_f64() < 1.0, "{elapsed:?}");
    }
}

#[test]
#[ignore = "runs the machine for 2^53 steps, about ten seconds in release mode"]
fn the_machine_follows_the_rules_up_to_two_to_the_53() {
    let start = Instant::now();
    let mut check = MachineCheck::new();
    let mut rows = Vec::new();
    while !check.is_done() {
        rows.extend(check.advance(1 << 24));
    }
    let last = rows.last().unwrap();
    eprintln!(
        "{} rows, last A({}, {}) at step {}, in {:?}",
        rows.len(),
        last.a,
        last.b,
        last.steps,
        start.elapsed()
    );
    assert!(rows.iter().all(|r| r.agrees));
    assert!(check.steps() <= turing_bb::cryptids::antihydra::MAX_CHECKED_STEPS);
}
