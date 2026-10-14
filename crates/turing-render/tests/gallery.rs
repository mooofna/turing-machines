//! Every machine of the gallery gets a sound state diagram and, if it has one tape, a
//! complete space-time overview.

use std::path::PathBuf;

use turing_core::{MachineSpec, dsl, standard};
use turing_render::layout::NODE_RADIUS;
use turing_render::spacetime::index::HEAD;
use turing_render::{
    Graph, LayoutKind, SpaceTime, SpaceTimeMode, SpaceTimeRequest, layout, route, svg::to_svg,
};

/// The id and spec of every gallery machine, read from `machines/gallery.json` without a
/// JSON parser: each entry has an `"id"` line followed by a `"file"` or `"standard"` line.
fn gallery() -> Vec<(String, MachineSpec)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../machines");
    let json = std::fs::read_to_string(root.join("gallery.json")).expect("the gallery index");
    let value = |line: &str, key: &str| {
        line.strip_prefix(&format!("\"{key}\": \""))
            .and_then(|rest| rest.split('"').next())
            .map(String::from)
    };
    let mut id = String::new();
    let mut out = Vec::new();
    for line in json.lines().map(str::trim) {
        if let Some(v) = value(line, "id") {
            id = v;
        } else if let Some(file) = value(line, "file") {
            let source = std::fs::read_to_string(root.join(&file)).expect("a gallery file");
            let spec = dsl::parse(&source).spec.expect("a valid description");
            out.push((id.clone(), spec));
        } else if let Some(text) = value(line, "standard") {
            let spec = standard::spec_from_standard(&text).expect("a valid standard text");
            out.push((id.clone(), spec));
        }
    }
    out
}

#[test]
fn the_gallery_is_found() {
    let machines = gallery();
    assert!(machines.len() >= 20, "{}", machines.len());
    assert!(machines.iter().any(|(id, _)| id == "bb5"));
    assert!(machines.iter().any(|(id, _)| id == "unary-multiplication"));
}

#[test]
fn every_diagram_is_sound() {
    for (id, spec) in gallery() {
        let graph = Graph::from_spec(&spec);
        assert_eq!(graph.nodes.len(), spec.states.len(), "{id}");
        let labels: usize = graph.edges.iter().map(|e| e.labels.len()).sum();
        assert_eq!(labels, spec.transitions.len(), "{id}");
        let l = layout(&graph, LayoutKind::Auto);
        assert!(l.width.is_finite() && l.height.is_finite(), "{id}");
        let routes = route(&graph, &l.x, &l.y);
        assert_eq!(routes.len(), graph.edges.len(), "{id}");
        for r in &routes {
            let e = &graph.edges[r.edge];
            let rim = ((r.tip.0 - l.x[e.to]).hypot(r.tip.1 - l.y[e.to]) - NODE_RADIUS).abs();
            assert!(rim < 1e-6, "{id}: tip off the rim");
            assert!(!r.path.contains("NaN"), "{id}");
            // Labels stay on the canvas.
            assert!(r.label.0 >= 0.0 && r.label.0 <= l.width, "{id}");
            assert!(r.label.1 >= 0.0 && r.label.1 <= l.height, "{id}");
        }
        let svg = to_svg(&graph, &l, &routes, "#111111", "#ffffff");
        assert_eq!(
            svg.matches("<circle").count(),
            graph.nodes.len() + spec.states.iter().filter(|s| s.is_final).count(),
            "{id}"
        );
        assert_eq!(
            svg.matches("<text").count(),
            graph.nodes.len() + labels,
            "{id}"
        );
    }
}

#[test]
fn every_single_tape_machine_has_an_overview() {
    for (id, spec) in gallery() {
        let machine = spec.compile().expect("a gallery machine compiles");
        if machine.tapes() != 1 {
            continue;
        }
        let input = machine
            .encode_input(machine.default_input())
            .expect("the default input fits the machine");
        let request = SpaceTimeRequest {
            width: 320,
            height: 200,
            mode: SpaceTimeMode::Overview {
                max_steps: 2_000_000,
            },
        };
        let mut st = SpaceTime::new(&machine, &input, request).expect("a valid request");
        let p = st.advance(u64::MAX);
        assert!(p.done && p.rows_done >= 1, "{id}: {p:?}");
        assert!(st.index_buffer().contains(&HEAD), "{id}: no head drawn");
    }
}
