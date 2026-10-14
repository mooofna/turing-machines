//! Node-by-node comparison with the enumerations verified in Coq by Coq-BB5.
//!
//! The CSV files in `tests/data` are the `*_verified_enumeration.csv` files of the Coq-BB5
//! release v1.0.0 (see `tests/data/README.md`). Each row is one node of the tree normal form
//! in the order of the enumeration, with its status and the decider that classified it. The
//! enumeration here must produce the same nodes in the same order with the same status and
//! the same decider (with the same parameters).

mod common;

fn check(states: u8, symbols: u8, csv: &str) {
    let lines = csv.lines().skip(1).map(str::to_owned);
    let result = common::compare(states, symbols, lines);
    common::assert_identical(&format!("BB({states},{symbols})"), &result);
}

#[test]
fn bb2_matches_coq_node_by_node() {
    check(2, 2, include_str!("data/BB2_verified_enumeration.csv"));
}

#[test]
fn bb3_matches_coq_node_by_node() {
    check(3, 2, include_str!("data/BB3_verified_enumeration.csv"));
}

#[test]
fn bb2x3_matches_coq_node_by_node() {
    check(2, 3, include_str!("data/BB2x3_verified_enumeration.csv"));
}

#[test]
fn csv_headers_are_as_expected() {
    for csv in [
        include_str!("data/BB2_verified_enumeration.csv"),
        include_str!("data/BB3_verified_enumeration.csv"),
        include_str!("data/BB2x3_verified_enumeration.csv"),
    ] {
        assert_eq!(csv.lines().next(), Some("machine,status,decider"));
    }
}
