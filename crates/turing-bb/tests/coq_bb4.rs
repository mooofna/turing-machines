//! Node-by-node comparison with Coq-BB5's verified BB(4) enumeration.
//!
//! The file (45 MB) is not part of the repository. Download it from
//! <https://docs.bbchallenge.org/CoqBB5_release_v1.0.0/BB4_verified_enumeration.csv>, check its
//! SHA-256 against `tests/data/README.md`, and run
//!
//! ```text
//! COQ_BB4_CSV=path/to/BB4_verified_enumeration.csv cargo test --release -p turing-bb --test coq_bb4
//! ```
//!
//! Without the variable the test passes without checking anything and says so.

use std::fs::File;
use std::io::{BufRead, BufReader};

mod common;

#[test]
fn bb4_matches_coq_node_by_node() {
    let Some(path) = std::env::var_os("COQ_BB4_CSV") else {
        eprintln!("COQ_BB4_CSV is not set; skipping the BB(4) comparison");
        return;
    };
    let file = File::open(&path).expect("COQ_BB4_CSV names a readable file");
    let mut lines = BufReader::new(file)
        .lines()
        .map(|l| l.expect("valid UTF-8"));
    assert_eq!(lines.next().as_deref(), Some("machine,status,decider"));
    let result = common::compare(4, 2, lines);
    common::assert_identical("BB(4,2)", &result);
    assert_eq!(result.rows, 858_909);
}
