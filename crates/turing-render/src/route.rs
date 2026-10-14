//! Drawing the edges between placed states.

use crate::graph::Graph;

/// The drawing of one edge.
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeRoute {
    /// Index into [`Graph::edges`].
    pub edge: usize,
    /// SVG path data from the source rim to the target rim, arrow tip at the end.
    pub path: String,
    /// The direction of the arrow tip in radians, for drawing the arrowhead.
    pub tip_angle: f64,
    /// The tip position.
    pub tip: (f64, f64),
    /// The anchor of the label block.
    pub label: (f64, f64),
    /// `start`, `middle` or `end`, as in SVG `text-anchor`.
    pub label_anchor: &'static str,
}

/// Routes every edge of `graph` for node centres `x` and `y`.
pub fn route(graph: &Graph, x: &[f64], y: &[f64]) -> Vec<EdgeRoute> {
    let _ = (graph, x, y);
    todo!("A4")
}
