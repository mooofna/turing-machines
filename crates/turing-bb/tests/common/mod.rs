//! Helpers shared by the integration tests.

#![allow(dead_code, reason = "each test crate uses a different subset")]

use turing_bb::{Enumerator, Verdict};

/// One row of a Coq-BB5 `*_verified_enumeration.csv` file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Row {
    /// The partial machine in standard format.
    pub machine: String,
    /// `halt` or `nonhalt`.
    pub status: String,
    /// Coq-BB5's decider identifier, for example `LOOP1_params_107`.
    pub decider: String,
}

impl Row {
    /// Parses a data line `machine,status,decider`.
    pub(crate) fn parse(line: &str) -> Self {
        let mut fields = line.trim_end().split(',');
        let mut next = || fields.next().expect("three fields").to_owned();
        Self {
            machine: next(),
            status: next(),
            decider: next(),
        }
    }
}

/// The outcome of a node-by-node comparison.
#[derive(Debug, Default)]
pub(crate) struct Comparison {
    /// Rows compared.
    pub rows: u64,
    /// Rows that differ, with the first few kept for the report.
    pub mismatches: u64,
    /// The first mismatches as `(index, expected, found)`.
    pub first: Vec<(u64, Row, Row)>,
    /// Rows left over on either side.
    pub extra_expected: u64,
    /// Nodes visited after the expected rows ran out.
    pub extra_found: u64,
}

/// Enumerates with the exact Coq-BB5 pipeline for this size and compares every visited node,
/// in order, with the expected rows (header excluded).
pub(crate) fn compare(
    states: u8,
    symbols: u8,
    mut expected: impl Iterator<Item = String>,
) -> Comparison {
    let mut result = Comparison::default();
    let mut e = Enumerator::new(states, symbols).expect("supported size");
    while !e.is_done() {
        e.advance_with(1 << 16, |visit| {
            let found = Row {
                machine: visit.tm.to_string(),
                status: match visit.verdict {
                    Verdict::Halts { .. } => "halt".into(),
                    Verdict::NonHalting(_) => "nonhalt".into(),
                    Verdict::Unknown => "unknown".into(),
                },
                decider: visit.stage.unwrap_or("DECIDER_NIL").into(),
            };
            let Some(line) = expected.next() else {
                result.extra_found += 1;
                return;
            };
            let want = Row::parse(&line);
            if want != found {
                result.mismatches += 1;
                if result.first.len() < 10 {
                    result.first.push((result.rows, want, found));
                }
            }
            result.rows += 1;
        });
    }
    result.extra_expected = expected.count() as u64;
    result
}

/// Asserts that a comparison found no difference.
pub(crate) fn assert_identical(name: &str, c: &Comparison) {
    assert!(
        c.mismatches == 0 && c.extra_expected == 0 && c.extra_found == 0,
        "{name}: {} rows compared, {} mismatches, {} rows missing, {} rows extra; first: {:#?}",
        c.rows,
        c.mismatches,
        c.extra_expected,
        c.extra_found,
        c.first
    );
}

/// Runs `tm` naively on a two-way tape for at most `fuel` steps; returns the number of steps
/// (the halting transition counted) and of non-blank cells if it halts, otherwise `None`.
/// Reaching an undefined transition counts as halting without a final step.
pub(crate) fn halts_within(tm: &turing_bb::CompactTm, fuel: u64) -> Option<(u64, u64)> {
    use std::collections::VecDeque;
    let mut tape: VecDeque<u8> = VecDeque::from([0]);
    let mut head = 0usize;
    let mut state = 0u8;
    let ones = |tape: &VecDeque<u8>| tape.iter().filter(|&&c| c != 0).count() as u64;
    for step in 0..fuel {
        let Some(action) = tm.get(state, tape[head]) else {
            return Some((step, ones(&tape)));
        };
        tape[head] = action.write;
        if action.dir == turing_core::Dir::L {
            if head == 0 {
                tape.push_front(0);
            } else {
                head -= 1;
            }
        } else {
            head += 1;
            if head == tape.len() {
                tape.push_back(0);
            }
        }
        match action.next {
            turing_bb::Next::State(q) => state = q,
            turing_bb::Next::Halt => return Some((step + 1, ones(&tape))),
        }
    }
    None
}
