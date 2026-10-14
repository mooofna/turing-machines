//! Benchmarks of diagram routing and space-time rendering. Run with
//! `cargo bench -p turing-render`.

use divan::Bencher;
use turing_core::{Dir, Kind, Machine, MachineBuilder};
use turing_render::{
    Edge, Graph, LayoutKind, Node, SpaceTime, SpaceTimeMode, SpaceTimeRequest, layout, route,
};

fn main() {
    divan::main();
}

/// The BB(5) champion `1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA`.
fn bb5() -> Machine {
    let table = [
        ("A", '0', "B", '1', Dir::R),
        ("A", '1', "C", '1', Dir::L),
        ("B", '0', "C", '1', Dir::R),
        ("B", '1', "B", '1', Dir::R),
        ("C", '0', "D", '1', Dir::R),
        ("C", '1', "E", '0', Dir::L),
        ("D", '0', "A", '1', Dir::L),
        ("D", '1', "D", '1', Dir::L),
        ("E", '0', "Z", '1', Dir::R),
        ("E", '1', "A", '0', Dir::L),
    ];
    table
        .iter()
        .fold(
            MachineBuilder::new("bb5")
                .kind(Kind::Beaver)
                .blank('0')
                .tape_alphabet(['0', '1']),
            |b, &(from, read, to, write, dir)| b.transition(from, read, to, write, dir),
        )
        .final_state("Z")
        .build()
        .expect("the champion is a valid machine")
}

/// A pseudo-random graph with `n` states and `m` edges, two transitions each.
fn graph(n: usize, m: usize) -> Graph {
    let mut seed = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        usize::try_from(seed % 1_000_003).unwrap_or(0)
    };
    let mut pairs = std::collections::BTreeSet::new();
    while pairs.len() < m {
        pairs.insert((next() % n, next() % n));
    }
    Graph {
        nodes: (0..n)
            .map(|i| Node {
                name: format!("q{i}"),
                is_start: i == 0,
                is_final: i + 1 == n,
            })
            .collect(),
        edges: pairs
            .into_iter()
            .map(|(from, to)| Edge {
                from,
                to,
                labels: vec!["a ↦ b, R".into(), "x ↦ y, L".into()],
                transitions: vec![0, 1],
            })
            .collect(),
    }
}

/// Routing runs on every animation frame while a state is dragged.
#[divan::bench(args = [(12, 30), (30, 45), (30, 75)])]
fn route_edges(bencher: Bencher<'_, '_>, (n, m): (usize, usize)) {
    let g = graph(n, m);
    let l = layout(&g, LayoutKind::Layered);
    bencher.bench(|| route(divan::black_box(&g), &l.x, &l.y));
}

#[divan::bench(args = [(12, 30), (30, 75)])]
fn layout_auto(bencher: Bencher<'_, '_>, (n, m): (usize, usize)) {
    let g = graph(n, m);
    bencher.bench(|| layout(divan::black_box(&g), LayoutKind::Auto));
}

/// The overview on the home page: 47,176,870 steps sampled into 600 rows.
#[divan::bench(sample_count = 20)]
fn bb5_overview_800x600(bencher: Bencher<'_, '_>) {
    let m = bb5();
    let request = SpaceTimeRequest {
        width: 800,
        height: 600,
        mode: SpaceTimeMode::Overview {
            max_steps: 100_000_000,
        },
    };
    bencher.bench(|| {
        let mut st = SpaceTime::new(&m, &[], request).expect("a valid request");
        st.advance(u64::MAX)
    });
}

/// The first steps of BB(5), one row per step.
#[divan::bench]
fn bb5_exact_800x600(bencher: Bencher<'_, '_>) {
    let m = bb5();
    let request = SpaceTimeRequest {
        width: 800,
        height: 600,
        mode: SpaceTimeMode::Exact {
            first_step: 0,
            cell_px: None,
        },
    };
    bencher.bench(|| {
        let mut st = SpaceTime::new(&m, &[], request).expect("a valid request");
        st.advance(u64::MAX)
    });
}
