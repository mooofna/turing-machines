//! State diagrams: labels, layout, routing and SVG output.

use proptest::prelude::*;
use turing_core::{Dir, MachineSpec, StateSpec, TransitionSpec};
use turing_render::layout::NODE_RADIUS;
use turing_render::{Edge, Graph, LayoutKind, Node, layout, route, svg::to_svg};

/// A spec from `(from, read, to, write, moves)` rows; read and write list one glyph per tape.
fn spec(
    states: &[(&str, bool)],
    start: &str,
    rows: &[(&str, &str, &str, &str, &str)],
) -> MachineSpec {
    let mut spec = MachineSpec::new();
    spec.states = states
        .iter()
        .map(|&(name, is_final)| StateSpec {
            name: name.into(),
            is_final,
        })
        .collect();
    spec.start = start.into();
    spec.tapes = rows
        .first()
        .map_or(1, |r| u8::try_from(r.1.chars().count()).unwrap());
    for &(from, read, to, write, moves) in rows {
        spec.transitions.push(TransitionSpec {
            from: from.into(),
            read: read.chars().collect(),
            to: to.into(),
            write: write.chars().collect(),
            dir: moves.chars().map(|c| Dir::from_char(c).unwrap()).collect(),
            ..TransitionSpec::default()
        });
    }
    spec
}

/// Binary increment, least significant bit first.
fn increment() -> MachineSpec {
    spec(
        &[("q0", false), ("q1", true)],
        "q0",
        &[
            ("q0", "1", "q0", "0", "R"),
            ("q0", "0", "q1", "1", "R"),
            ("q0", "␣", "q1", "1", "R"),
        ],
    )
}

fn describe(graph: &Graph) -> String {
    let mut out = String::new();
    for (i, n) in graph.nodes.iter().enumerate() {
        out.push_str(&format!(
            "node {i} {}{}{}\n",
            n.name,
            if n.is_start { " start" } else { "" },
            if n.is_final { " final" } else { "" }
        ));
    }
    for e in &graph.edges {
        out.push_str(&format!(
            "edge {} -> {} {:?}\n",
            e.from, e.to, e.transitions
        ));
        for l in &e.labels {
            out.push_str(&format!("  {l}\n"));
        }
    }
    out
}

#[test]
fn single_tape_labels() {
    insta::assert_snapshot!(describe(&Graph::from_spec(&increment())));
}

#[test]
fn multi_tape_labels_and_grouping() {
    // Copies the first tape to the second, then walks back; transitions are given out of
    // order and one names an unknown state.
    let spec = spec(
        &[("q0", false), ("q1", false), ("qf", true)],
        "q0",
        &[
            ("q1", "a␣", "q1", "a␣", "LN"),
            ("q0", "a␣", "q0", "aa", "RR"),
            ("q0", "␣␣", "q1", "␣␣", "LN"),
            ("q0", "b␣", "q0", "bb", "RR"),
            ("q1", "␣␣", "qf", "␣␣", "RL"),
            ("q1", "b␣", "nowhere", "b␣", "LN"),
        ],
    );
    insta::assert_snapshot!(describe(&Graph::from_spec(&spec)));
}

#[test]
fn invisible_blanks_are_shown_as_open_boxes() {
    let mut spec = spec(&[("q", false)], "q", &[("q", " ", "q", "1", "R")]);
    spec.blank = ' ';
    assert_eq!(Graph::from_spec(&spec).edges[0].labels, ["␣ ↦ 1, R"]);
    // Visible blanks keep their glyph, as in busy beaver machines.
    let mut spec = spec_with_blank_zero();
    spec.blank = '0';
    assert_eq!(Graph::from_spec(&spec).edges[0].labels, ["0 ↦ 1, R"]);
}

fn spec_with_blank_zero() -> MachineSpec {
    spec(&[("A", false)], "A", &[("A", "0", "A", "1", "R")])
}

#[test]
fn svg_of_a_small_machine() {
    let graph = Graph::from_spec(&increment());
    let l = layout(&graph, LayoutKind::Auto);
    let routes = route(&graph, &l.x, &l.y);
    insta::assert_snapshot!(to_svg(&graph, &l, &routes, "#111111", "#ffffff"));
}

#[test]
fn svg_of_a_busy_beaver() {
    // The BB(3) champion 1RB1RZ_0RC1RB_1LC1LA with an explicit halting state.
    let graph = Graph::from_spec(&spec(
        &[("A", false), ("B", false), ("C", false), ("H", true)],
        "A",
        &[
            ("A", "0", "B", "1", "R"),
            ("A", "1", "H", "1", "R"),
            ("B", "0", "C", "0", "R"),
            ("B", "1", "B", "1", "R"),
            ("C", "0", "C", "1", "L"),
            ("C", "1", "A", "1", "L"),
        ],
    ));
    let l = layout(&graph, LayoutKind::Auto);
    let routes = route(&graph, &l.x, &l.y);
    insta::assert_snapshot!(to_svg(&graph, &l, &routes, "#111111", "#ffffff"));
}

/// The first and the last point of SVG path data.
fn ends(path: &str) -> ((f64, f64), (f64, f64)) {
    let numbers: Vec<f64> = path
        .split_whitespace()
        .filter_map(|t| t.parse().ok())
        .collect();
    assert!(numbers.len() >= 4 && numbers.len() % 2 == 0, "{path}");
    let n = numbers.len();
    ((numbers[0], numbers[1]), (numbers[n - 2], numbers[n - 1]))
}

fn arbitrary_graph() -> impl Strategy<Value = (Graph, Vec<f64>, Vec<f64>)> {
    (1usize..9).prop_flat_map(|n| {
        (
            prop::collection::btree_set((0..n, 0..n), 0..(n * n).min(20)),
            prop::collection::vec(prop_oneof![Just(0.0), -400.0..400.0f64], n),
            prop::collection::vec(prop_oneof![Just(0.0), -400.0..400.0f64], n),
            0..n,
        )
            .prop_map(move |(pairs, x, y, start)| {
                let nodes = (0..n)
                    .map(|i| Node {
                        name: format!("q{i}"),
                        is_start: i == start,
                        is_final: false,
                    })
                    .collect();
                let edges = pairs
                    .into_iter()
                    .map(|(from, to)| Edge {
                        from,
                        to,
                        labels: vec!["a ↦ b, R".into(); 1 + (from + to) % 3],
                        transitions: vec![0],
                    })
                    .collect();
                (Graph { nodes, edges }, x, y)
            })
    })
}

proptest! {
    #[test]
    fn routes_run_from_rim_to_rim((graph, x, y) in arbitrary_graph()) {
        let routes = route(&graph, &x, &y);
        prop_assert_eq!(routes.len(), graph.edges.len());
        for r in &routes {
            let e = &graph.edges[r.edge];
            prop_assert!(!r.path.contains("NaN") && !r.path.contains("inf"), "{}", r.path);
            prop_assert!(r.tip_angle.is_finite());
            prop_assert!(r.label.0.is_finite() && r.label.1.is_finite());
            prop_assert!(["start", "middle", "end"].contains(&r.label_anchor));
            let (start, end) = ends(&r.path);
            let rim = |p: (f64, f64), i: usize| ((p.0 - x[i]).hypot(p.1 - y[i]) - NODE_RADIUS).abs();
            prop_assert!(rim(start, e.from) < 0.01, "start {:?} of {}", start, r.path);
            prop_assert!(rim(end, e.to) < 0.01, "end {:?} of {}", end, r.path);
            prop_assert!((end.0 - r.tip.0).abs() < 0.006 && (end.1 - r.tip.1).abs() < 0.006);
        }
    }

    #[test]
    fn layouts_are_finite_and_deterministic((graph, _x, _y) in arbitrary_graph()) {
        for kind in [LayoutKind::Auto, LayoutKind::Layered, LayoutKind::Circular] {
            let l = layout(&graph, kind);
            prop_assert_eq!(l.x.len(), graph.nodes.len());
            prop_assert!(l.x.iter().chain(&l.y).all(|v| v.is_finite()));
            prop_assert!(l.width.is_finite() && l.height.is_finite());
            prop_assert_eq!(&l, &layout(&graph, kind));
        }
    }
}
