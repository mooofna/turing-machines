//! Benchmarks of the enumeration. Run with `cargo bench -p turing-bb`.

use divan::{Bencher, black_box};
use turing_bb::cryptids::antihydra::RuleWalk;
use turing_bb::uparrow::tetration;
use turing_bb::{CompactTm, Decider, Enumerator, Pipeline};

fn main() {
    divan::main();
}

/// Enumerates and classifies every node of the tree normal form.
fn enumerate(states: u8, symbols: u8) -> u64 {
    let mut e = Enumerator::new(states, symbols).expect("supported size");
    e.run_to_completion().nodes
}

/// BB(2): 61 nodes.
#[divan::bench]
fn enumerate_bb2() -> u64 {
    enumerate(black_box(2), 2)
}

/// BB(3): 5,417 nodes.
#[divan::bench]
fn enumerate_bb3() -> u64 {
    enumerate(black_box(3), 2)
}

/// BB(2,3): 6,019 nodes.
#[divan::bench]
fn enumerate_bb2x3() -> u64 {
    enumerate(black_box(2), 3)
}

/// BB(4): 858,909 nodes.
#[divan::bench(sample_count = 10, sample_size = 1)]
fn enumerate_bb4() -> u64 {
    enumerate(black_box(4), 2)
}

/// The full BB(4) pipeline on one of the two machines that need repeated word lists.
#[divan::bench]
fn pipeline_on_a_repwl_machine(bencher: Bencher<'_, '_>) {
    let tm: CompactTm = "1RB1LA_1LA0RC_1LD1RC_---0LA"
        .parse()
        .expect("valid machine");
    let mut pipeline = Pipeline::standard();
    bencher.bench_local(|| pipeline.decide(black_box(&tm)));
}

/// The loop decider with 107 steps on the BB(4) champion's halting node.
#[divan::bench]
fn loops_on_the_bb4_champion(bencher: Bencher<'_, '_>) {
    let tm: CompactTm = "1RB1LB_1LA0LC_---1LD_1RD0RA"
        .parse()
        .expect("valid machine");
    let mut loops = turing_bb::deciders::Loops::new(107);
    bencher.bench_local(|| loops.decide(black_box(&tm)));
}

/// 100,000 iterations of Antihydra's rule.
#[divan::bench(sample_count = 10, sample_size = 1)]
fn antihydra_walk_100k() -> u64 {
    let mut walk = RuleWalk::new();
    walk.advance(black_box(100_000));
    walk.summary(0).bits
}

/// 2↑↑5 = 2^65536 in decimal.
#[divan::bench(sample_count = 10)]
fn two_up_up_five() -> usize {
    tetration(black_box(2), 5, 20_000)
        .exact
        .map_or(0, |s| s.len())
}
