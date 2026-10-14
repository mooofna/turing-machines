//! Placing the states of a diagram.
//!
//! Both layouts are deterministic: the same graph always gives the same coordinates.
//!
//! * The layered layout puts the start state in the first column and every other state in
//!   the column of its breadth-first distance from the start, so that a run reads from left
//!   to right. States that cannot be reached form a last column. Within a column, states are
//!   ordered by four alternating barycentre sweeps, which pull neighbours to the same height
//!   and so reduce crossings, and the column is centred vertically.
//! * The circular layout spaces the states evenly on a circle in spec order, the start state
//!   on the far left, with a radius that leaves the same room between neighbours for any
//!   number of states. It suits dense machines such as busy beavers, whose layered drawing
//!   would be a tangle of long edges.
//!
//! The column and row spacing grows with the widest and tallest label block, so labels of
//! multi-tape machines fit between their states.

use std::f64::consts::{PI, TAU};

use crate::graph::Graph;
use crate::route::{self, Curve, label_size};

/// Which layout to compute.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum LayoutKind {
    /// Layered for sparse graphs (edges ≤ 2.5 × nodes), circular otherwise.
    ///
    /// A sparse graph is still drawn on a circle if its layered drawing is clearly more
    /// cluttered, counting pairs of crossing edges and labels that collide with states,
    /// labels or edges. This catches machines such as busy beavers, whose transitions jump
    /// back and forth between all states, while machines that run in stages stay layered.
    #[default]
    Auto,
    /// Breadth-first layers from the start state, left to right, ordered by barycentres.
    Layered,
    /// Evenly on a circle, in spec order, start at the left.
    Circular,
}

/// Node centres in diagram units (one unit is about one CSS pixel at scale 1).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Layout {
    /// x of each node centre.
    pub x: Vec<f64>,
    /// y of each node centre.
    pub y: Vec<f64>,
    /// The bounding box width, margins and labels included.
    pub width: f64,
    /// The bounding box height, margins and labels included.
    pub height: f64,
}

/// The node radius in diagram units.
pub const NODE_RADIUS: f64 = 22.0;

/// The smallest distance between the columns of the layered layout.
pub const LAYER_GAP: f64 = 140.0;

/// The smallest distance between the rows of a column in the layered layout.
pub const ROW_GAP: f64 = 90.0;

/// The smallest distance between neighbours on the circle of the circular layout.
pub const CIRCLE_GAP: f64 = 150.0;

/// The empty border around the drawing.
pub const MARGIN: f64 = 16.0;

/// The length of the arrow that points into the start state from the left.
pub const START_ARROW_LENGTH: f64 = 28.0;

/// How much more cluttered than the circular drawing a layered drawing may be before
/// [`LayoutKind::Auto`] prefers the circle.
const AUTO_TOLERANCE: usize = 2;

/// The number of barycentre sweeps of the layered layout.
const SWEEPS: usize = 4;

/// Computes a layout.
///
/// The coordinates are shifted so that the drawing, including self-loops, labels and the
/// start arrow, starts [`MARGIN`] from the top left corner, and `width` and `height` add the
/// same margin on the far sides.
///
/// ```
/// use turing_render::{Edge, Graph, LayoutKind, Node, layout};
/// let node = |name: &str, is_start| Node { name: name.into(), is_start, is_final: false };
/// let edge = |from, to| Edge { from, to, labels: vec!["1 ↦ 0, R".into()], transitions: vec![0] };
/// let graph = Graph {
///     nodes: vec![node("A", true), node("B", false), node("C", false)],
///     edges: vec![edge(0, 1), edge(1, 2)],
/// };
/// let l = layout(&graph, LayoutKind::Layered);
/// assert!(l.x[0] < l.x[1] && l.x[1] < l.x[2]);
/// assert_eq!(l.y[0], l.y[1]);
/// ```
pub fn layout(graph: &Graph, kind: LayoutKind) -> Layout {
    let n = graph.nodes.len();
    if n == 0 {
        return Layout {
            x: Vec::new(),
            y: Vec::new(),
            width: 2.0 * MARGIN,
            height: 2.0 * MARGIN,
        };
    }
    let (x, y) = match kind {
        LayoutKind::Layered => layered(graph),
        LayoutKind::Circular => circular(graph),
        LayoutKind::Auto => {
            #[expect(clippy::cast_precision_loss, reason = "small counts")]
            let sparse = graph.edges.len() as f64 <= 2.5 * n as f64;
            let ring = circular(graph);
            if sparse {
                let layers = layered(graph);
                let tangled = route::clutter(graph, &layers.0, &layers.1);
                if tangled > route::clutter(graph, &ring.0, &ring.1) + AUTO_TOLERANCE {
                    ring
                } else {
                    layers
                }
            } else {
                ring
            }
        }
    };
    fit(graph, x, y)
}

/// The widest and tallest label block of the graph.
fn largest_label(graph: &Graph) -> (f64, f64) {
    graph
        .edges
        .iter()
        .map(|e| label_size(&e.labels))
        .fold((0.0, 0.0), |(w, h), (ew, eh)| {
            (f64::max(w, ew), f64::max(h, eh))
        })
}

/// The index of the start state, or 0.
fn start_index(graph: &Graph) -> usize {
    graph.nodes.iter().position(|v| v.is_start).unwrap_or(0)
}

/// Breadth-first columns, barycentre order, centred columns.
fn layered(graph: &Graph) -> (Vec<f64>, Vec<f64>) {
    let n = graph.nodes.len();
    let mut succ: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for e in &graph.edges {
        if e.from < n && e.to < n && e.from != e.to {
            succ[e.from].push(e.to);
            adj[e.from].push(e.to);
            adj[e.to].push(e.from);
        }
    }

    // Columns in order of discovery.
    let start = start_index(graph);
    let mut layer = vec![usize::MAX; n];
    let mut layers: Vec<Vec<usize>> = vec![vec![start]];
    layer[start] = 0;
    let mut frontier = vec![start];
    while !frontier.is_empty() {
        let mut next = Vec::new();
        for &v in &frontier {
            for &w in &succ[v] {
                if layer[w] == usize::MAX {
                    layer[w] = layers.len();
                    next.push(w);
                }
            }
        }
        if !next.is_empty() {
            layers.push(next.clone());
        }
        frontier = next;
    }
    let unreachable: Vec<usize> = (0..n).filter(|&v| layer[v] == usize::MAX).collect();
    if !unreachable.is_empty() {
        for &v in &unreachable {
            layer[v] = layers.len();
        }
        layers.push(unreachable);
    }

    // Positions relative to the centre of the column.
    let mut slot = vec![0.0; n];
    let centre = |slot: &mut [f64], column: &[usize]| {
        #[expect(clippy::cast_precision_loss, reason = "small counts")]
        let middle = (column.len() as f64 - 1.0) / 2.0;
        for (i, &v) in column.iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "small counts")]
            let i = i as f64;
            slot[v] = i - middle;
        }
    };
    for column in &layers {
        centre(&mut slot, column);
    }

    let count = layers.len();
    for sweep in 0..SWEEPS {
        let down = sweep % 2 == 0;
        let order: Vec<usize> = if down {
            (1..count).collect()
        } else {
            (0..count.saturating_sub(1)).rev().collect()
        };
        for l in order {
            let reference = if down { l - 1 } else { l + 1 };
            let bary: Vec<(usize, f64)> = layers[l]
                .iter()
                .map(|&v| {
                    let mean = |filter: &dyn Fn(usize) -> bool| -> Option<f64> {
                        let (sum, k) = adj[v]
                            .iter()
                            .filter(|&&w| filter(layer[w]))
                            .fold((0.0, 0.0), |(s, k), &w| (s + slot[w], k + 1.0));
                        (k > 0.0).then(|| sum / k)
                    };
                    let b = mean(&|lw| lw == reference)
                        .or_else(|| mean(&|lw| lw != l))
                        .unwrap_or(slot[v]);
                    (v, b)
                })
                .collect();
            let mut sorted = bary;
            sorted.sort_by(|a, b| a.1.total_cmp(&b.1).then(slot[a.0].total_cmp(&slot[b.0])));
            layers[l] = sorted.into_iter().map(|(v, _)| v).collect();
            centre(&mut slot, &layers[l]);
        }
    }

    let (label_w, label_h) = largest_label(graph);
    let gap = (label_w + 2.0 * NODE_RADIUS + 24.0).clamp(LAYER_GAP, 2.0 * LAYER_GAP + 40.0);
    let row = (label_h + 2.0 * NODE_RADIUS + 16.0).clamp(ROW_GAP, 2.0 * ROW_GAP + 20.0);
    #[expect(clippy::cast_precision_loss, reason = "small counts")]
    let x = (0..n).map(|v| layer[v] as f64 * gap).collect();
    let y = (0..n).map(|v| slot[v] * row).collect();
    (x, y)
}

/// Evenly on a circle in spec order, the start state at the left.
fn circular(graph: &Graph) -> (Vec<f64>, Vec<f64>) {
    let n = graph.nodes.len();
    if n == 1 {
        return (vec![0.0], vec![0.0]);
    }
    let (label_w, _) = largest_label(graph);
    let chord = (label_w + 2.0 * NODE_RADIUS + 24.0).clamp(CIRCLE_GAP, 2.0 * CIRCLE_GAP + 20.0);
    #[expect(clippy::cast_precision_loss, reason = "small counts")]
    let nf = n as f64;
    let radius = chord / (2.0 * (PI / nf).sin());
    let start = start_index(graph);
    (0..n)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "small counts")]
            let k = ((i + n - start) % n) as f64;
            let angle = PI + TAU * k / nf;
            (radius * angle.cos(), radius * angle.sin())
        })
        .unzip()
}

/// Shifts the drawing to the margin and measures it.
fn fit(graph: &Graph, mut x: Vec<f64>, mut y: Vec<f64>) -> Layout {
    let mut min = (f64::INFINITY, f64::INFINITY);
    let mut max = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    let mut cover = |px: f64, py: f64| {
        min = (min.0.min(px), min.1.min(py));
        max = (max.0.max(px), max.1.max(py));
    };
    for (i, node) in graph.nodes.iter().enumerate() {
        cover(x[i] - NODE_RADIUS, y[i] - NODE_RADIUS);
        cover(x[i] + NODE_RADIUS, y[i] + NODE_RADIUS);
        if node.is_start {
            cover(x[i] - NODE_RADIUS - START_ARROW_LENGTH, y[i]);
        }
    }
    for g in route::geometry(graph, &x, &y) {
        let curve: Curve = g.curve;
        for k in 0..=16 {
            let p = curve.at(f64::from(k) / 16.0);
            cover(p.x, p.y);
        }
        cover(g.label_box.x0, g.label_box.y0);
        cover(g.label_box.x1, g.label_box.y1);
    }
    let dx = MARGIN - min.0;
    let dy = MARGIN - min.1;
    for v in &mut x {
        *v += dx;
    }
    for v in &mut y {
        *v += dy;
    }
    Layout {
        x,
        y,
        width: max.0 - min.0 + 2.0 * MARGIN,
        height: max.1 - min.1 + 2.0 * MARGIN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Edge, Node};

    fn graph(n: usize, start: usize, edges: &[(usize, usize)]) -> Graph {
        Graph {
            nodes: (0..n)
                .map(|i| Node {
                    name: format!("q{i}"),
                    is_start: i == start,
                    is_final: false,
                })
                .collect(),
            edges: edges
                .iter()
                .map(|&(from, to)| Edge {
                    from,
                    to,
                    labels: vec!["a ↦ b, R".into()],
                    transitions: vec![0],
                })
                .collect(),
        }
    }

    #[test]
    fn empty_graph_is_just_margins() {
        let l = layout(&Graph::default(), LayoutKind::Auto);
        assert_eq!(l.x.len(), 0);
        assert_eq!((l.width, l.height), (2.0 * MARGIN, 2.0 * MARGIN));
    }

    #[test]
    fn layers_follow_breadth_first_distance() {
        // 1 is the start; 0 is reachable in two steps; 3 is unreachable.
        let g = graph(4, 1, &[(1, 2), (2, 0), (0, 1), (3, 3)]);
        let l = layout(&g, LayoutKind::Layered);
        assert!(l.x[1] < l.x[2] && l.x[2] < l.x[0] && l.x[0] < l.x[3]);
        assert!((l.x[2] - l.x[1] - LAYER_GAP).abs() < 1e-9);
    }

    #[test]
    fn columns_are_centred_and_spaced() {
        let g = graph(4, 0, &[(0, 1), (0, 2), (0, 3)]);
        let l = layout(&g, LayoutKind::Layered);
        let mut ys = [l.y[1], l.y[2], l.y[3]];
        ys.sort_by(f64::total_cmp);
        assert!((ys[1] - ys[0] - ROW_GAP).abs() < 1e-9);
        assert!((ys[2] - ys[1] - ROW_GAP).abs() < 1e-9);
        assert!((ys[1] - l.y[0]).abs() < 1e-9);
    }

    #[test]
    fn barycentres_untangle_crossings() {
        // 0 -> {1, 2}; 1 -> 4 and 2 -> 3, with 3 discovered before 4: the sweeps must swap
        // 3 and 4 so that the edges do not cross.
        let g = graph(5, 0, &[(0, 2), (0, 1), (1, 4), (2, 3)]);
        let l = layout(&g, LayoutKind::Layered);
        assert_eq!(l.y[1] < l.y[2], l.y[4] < l.y[3]);
    }

    #[test]
    fn circle_starts_at_the_left_and_runs_clockwise() {
        let g = graph(4, 2, &[]);
        let l = layout(&g, LayoutKind::Circular);
        let min_x = l.x.iter().copied().fold(f64::INFINITY, f64::min);
        assert_eq!(l.x[2], min_x);
        // Next in spec order after the start (wrapping): 3 at the top, 0 at the right.
        assert!(l.y[3] < l.y[2] && l.x[0] > l.x[3]);
        let r = (l.x[0] - l.x[2]) / 2.0;
        let chord = r * 2.0 * (PI / 4.0).sin();
        assert!((chord - CIRCLE_GAP).abs() < 1e-6);
    }

    #[test]
    fn auto_switches_on_density() {
        let sparse = graph(2, 0, &[(0, 1), (1, 0)]);
        let dense = graph(2, 0, &[(0, 0), (0, 1), (1, 0), (1, 1)]);
        let dense_bigger = graph(
            3,
            0,
            &[
                (0, 0),
                (0, 1),
                (0, 2),
                (1, 0),
                (1, 1),
                (1, 2),
                (2, 0),
                (2, 1),
            ],
        );
        assert_eq!(
            layout(&sparse, LayoutKind::Auto),
            layout(&sparse, LayoutKind::Layered)
        );
        assert_eq!(
            layout(&dense, LayoutKind::Auto),
            layout(&dense, LayoutKind::Layered)
        );
        assert_eq!(
            layout(&dense_bigger, LayoutKind::Auto),
            layout(&dense_bigger, LayoutKind::Circular)
        );
    }

    #[test]
    fn everything_fits_inside_the_box() {
        let g = graph(
            5,
            0,
            &[(0, 0), (0, 1), (1, 2), (2, 1), (2, 3), (3, 0), (4, 4)],
        );
        for kind in [LayoutKind::Layered, LayoutKind::Circular] {
            let l = layout(&g, kind);
            for g in route::geometry(&g, &l.x, &l.y) {
                for k in 0..=16 {
                    let p = g.curve.at(f64::from(k) / 16.0);
                    assert!(p.x >= MARGIN - 1e-6 && p.x <= l.width - MARGIN + 1e-6);
                    assert!(p.y >= MARGIN - 1e-6 && p.y <= l.height - MARGIN + 1e-6);
                }
                assert!(g.label_box.x0 >= MARGIN - 1e-6 && g.label_box.y0 >= MARGIN - 1e-6);
                assert!(g.label_box.x1 <= l.width - MARGIN + 1e-6);
                assert!(g.label_box.y1 <= l.height - MARGIN + 1e-6);
            }
            // The start arrow.
            assert!(l.x[0] - NODE_RADIUS - START_ARROW_LENGTH >= MARGIN - 1e-6);
        }
    }

    #[test]
    fn layout_is_deterministic() {
        let g = graph(
            6,
            0,
            &[
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 4),
                (4, 5),
                (5, 0),
                (2, 5),
                (1, 4),
            ],
        );
        assert_eq!(layout(&g, LayoutKind::Auto), layout(&g, LayoutKind::Auto));
    }
}
