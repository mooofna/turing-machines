//! Placing the states of a diagram.

use crate::graph::Graph;

/// Which layout to compute.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum LayoutKind {
    /// Layered for sparse graphs (edges ≤ 2.5 × nodes), circular otherwise.
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

/// Computes a layout.
pub fn layout(graph: &Graph, kind: LayoutKind) -> Layout {
    let _ = (graph, kind);
    todo!("A4")
}
