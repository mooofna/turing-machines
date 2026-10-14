//! Exact counts of the tree normal form, busy beaver values and the decider funnel.
//!
//! The expected numbers are those of Coq-BB5 (READMEs of `CoqBB5/BB2`, `BB3`, `BB4` and
//! `BB2x3`, and the verified enumeration CSVs). The BB(4) tests take about a second in release
//! mode and much longer in debug mode, so they are ignored by default; CI runs them with
//! `cargo test --release -p turing-bb -- --include-ignored`.

use turing_bb::{DeciderId, Enumerator, Stats};

/// Runs a full enumeration with the default pipeline for the size.
fn run(states: u8, symbols: u8) -> (Stats, Vec<String>) {
    let mut e = Enumerator::new(states, symbols).expect("supported size");
    let result = e.run_to_completion().clone();
    let holdouts = e.holdouts().iter().map(ToString::to_string).collect();
    (result, holdouts)
}

/// Checks nodes, halting nodes, non-halting nodes, holdouts, S and Σ.
fn check_totals(stats: &Stats, nodes: u64, halting: u64, s: u64, sigma: u64) {
    assert_eq!(stats.nodes, nodes);
    assert_eq!(stats.halting, halting);
    assert_eq!(stats.non_halting(), nodes - halting);
    assert_eq!(stats.holdouts, 0);
    assert_eq!(stats.max_steps.map(|(v, _)| v), Some(s));
    assert_eq!(stats.max_ones.map(|(v, _)| v), Some(sigma));
}

fn stages(stats: &Stats) -> Vec<(&str, u64)> {
    stats
        .by_stage
        .iter()
        .map(|(name, n)| (name.as_str(), *n))
        .collect()
}

#[test]
fn one_state() {
    let (stats, holdouts) = run(1, 2);
    check_totals(&stats, 3, 1, 1, 1);
    assert_eq!(holdouts, Vec::<String>::new());
    assert_eq!(stats.resolved_by(DeciderId::Loops), 2);
    let (_, champion) = stats.max_steps.unwrap();
    assert_eq!(champion.to_string(), "1RZ---");
}

#[test]
fn two_states() {
    let (stats, holdouts) = run(2, 2);
    check_totals(&stats, 61, 19, 6, 4);
    assert_eq!(holdouts, Vec::<String>::new());
    assert_eq!(
        stages(&stats),
        [
            ("LOOP1_params_6", 29),
            ("NGRAM_CPS_IMPL2_params_1_1_100", 13)
        ]
    );
    // The first machine found with six steps, and the classic Σ(2) champion.
    assert_eq!(stats.max_steps.unwrap().1.to_string(), "0RB1RZ_1LA1RB");
    assert_eq!(stats.max_ones.unwrap().1.to_string(), "1RB1LB_1LA1RZ");
}

#[test]
fn three_states() {
    let (stats, holdouts) = run(3, 2);
    check_totals(&stats, 5_417, 1_772, 21, 6);
    assert_eq!(holdouts, Vec::<String>::new());
    assert_eq!(
        stages(&stats),
        [
            ("LOOP1_params_21", 3_492),
            ("NGRAM_CPS_IMPL2_params_1_1_100", 80),
            ("NGRAM_CPS_IMPL2_params_2_2_200", 35),
            ("NGRAM_CPS_IMPL2_params_3_3_400", 20),
            ("NGRAM_CPS_IMPL1_params_2_2_2_1600", 18),
        ]
    );
    assert_eq!(
        stats.max_steps.unwrap().1.to_string(),
        "1RB1RZ_1LB0RC_1LC1LA"
    );
}

#[test]
fn two_states_three_symbols() {
    let (stats, holdouts) = run(2, 3);
    check_totals(&stats, 6_019, 1_816, 38, 9);
    assert_eq!(holdouts, Vec::<String>::new());
    assert_eq!(
        stages(&stats),
        [
            ("LOOP1_params_38", 3_861),
            ("NGRAM_CPS_IMPL2_params_1_1_400", 288),
            ("NGRAM_CPS_IMPL2_params_2_2_800", 36),
            ("NGRAM_CPS_IMPL2_params_3_3_400", 8),
            ("REPWL_params_2_3_320_400", 10),
        ]
    );
}

#[test]
fn the_standard_pipeline_also_settles_small_sizes() {
    for (states, nodes, s, sigma) in [(2, 61, 6, 4), (3, 5_417, 21, 6)] {
        let mut e = Enumerator::with_pipeline(states, 2, turing_bb::Pipeline::standard()).unwrap();
        let stats = e.run_to_completion();
        assert_eq!(stats.nodes, nodes);
        assert_eq!(stats.holdouts, 0);
        assert_eq!(stats.max_steps.map(|(v, _)| v), Some(s));
        assert_eq!(stats.max_ones.map(|(v, _)| v), Some(sigma));
    }
}

#[test]
fn shards_add_up_to_the_full_enumeration() {
    for count in [1, 2, 3, 4, 7] {
        let mut total = Stats::default();
        for index in 0..count {
            let mut e = Enumerator::new(3, 2)
                .unwrap()
                .with_shard(index, count)
                .unwrap();
            total.merge(e.run_to_completion());
        }
        let (full, _) = run(3, 2);
        assert_eq!(total.nodes, full.nodes, "{count} shards");
        assert_eq!(total.halting, full.halting);
        assert_eq!(total.by_stage, full.by_stage);
        assert_eq!(total.max_steps.map(|r| r.0), Some(21));
        assert_eq!(total.max_ones.map(|r| r.0), Some(6));
    }
}

#[test]
fn the_records_halt_as_recorded() {
    for (states, symbols) in [(1, 2), (2, 2), (3, 2), (2, 3)] {
        let (stats, _) = run(states, symbols);
        let (s, tm) = stats.max_steps.unwrap();
        assert_eq!(
            common::halts_within(&tm, 1_000).map(|r| r.0),
            Some(s),
            "{tm}"
        );
        let (sigma, tm) = stats.max_ones.unwrap();
        assert_eq!(
            common::halts_within(&tm, 1_000).map(|r| r.1),
            Some(sigma),
            "{tm}"
        );
    }
}

#[test]
#[ignore = "about a second in release mode; run with --release -- --include-ignored"]
fn four_states() {
    let (stats, holdouts) = run(4, 2);
    check_totals(&stats, 858_909, 249_693, 107, 13);
    assert_eq!(holdouts, Vec::<String>::new());
    assert_eq!(
        stages(&stats),
        [
            ("LOOP1_params_107", 588_373),
            ("NGRAM_CPS_IMPL2_params_1_1_100", 11_644),
            ("NGRAM_CPS_IMPL2_params_2_2_200", 4_759),
            ("NGRAM_CPS_IMPL2_params_3_3_400", 1_731),
            ("NGRAM_CPS_IMPL1_params_2_2_2_1600", 2_296),
            ("NGRAM_CPS_IMPL1_params_2_3_3_1600", 161),
            ("NGRAM_CPS_IMPL1_params_4_2_2_600", 174),
            ("NGRAM_CPS_IMPL1_params_4_3_3_1600", 29),
            ("NGRAM_CPS_IMPL1_params_6_2_2_3200", 14),
            ("NGRAM_CPS_IMPL1_params_6_3_3_3200", 10),
            ("NGRAM_CPS_IMPL1_params_8_2_2_1600", 8),
            ("NGRAM_CPS_IMPL1_params_8_3_3_1600", 3),
            ("NGRAM_CPS_LRU_params_2_2_10000", 8),
            ("NGRAM_CPS_IMPL1_params_10_4_4_10000", 4),
            ("REPWL_params_4_3_320_10000", 2),
        ]
    );
    let families: Vec<(DeciderId, u64)> = stats.by_decider.clone();
    assert_eq!(
        families,
        [
            (DeciderId::Loops, 588_373),
            (DeciderId::NGramCps, 18_134),
            (DeciderId::NGramCpsHistory, 2_699),
            (DeciderId::NGramCpsLru, 8),
            (DeciderId::RepWl, 2),
        ]
    );
    let (_, champion) = stats.max_steps.unwrap();
    assert_eq!(champion.to_string(), "1RB1LB_1LA0LC_1RZ1LD_1RD0RA");
    assert_eq!(common::halts_within(&champion, 1_000), Some((107, 13)));
}

#[test]
#[ignore = "a few seconds in release mode; run with --release -- --include-ignored"]
fn four_states_in_four_shards() {
    let mut total = Stats::default();
    for index in 0..4 {
        let mut e = Enumerator::new(4, 2).unwrap().with_shard(index, 4).unwrap();
        total.merge(e.run_to_completion());
    }
    let (full, _) = run(4, 2);
    assert_eq!(total.nodes, full.nodes);
    assert_eq!(total.halting, full.halting);
    assert_eq!(total.holdouts, full.holdouts);
    assert_eq!(total.by_decider, full.by_decider);
    assert_eq!(total.by_stage, full.by_stage);
    assert_eq!(total.max_steps.map(|r| r.0), Some(107));
    assert_eq!(total.max_ones.map(|r| r.0), Some(13));
}

mod common;
