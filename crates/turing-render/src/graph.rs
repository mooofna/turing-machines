//! The state diagram as a graph with labelled edges.

use turing_core::MachineSpec;

/// A state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    /// The state name.
    pub name: String,
    /// Whether it is q0.
    pub is_start: bool,
    /// Whether it belongs to F.
    pub is_final: bool,
}

/// All transitions from one state to another, drawn as one edge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    /// Index of the source node.
    pub from: usize,
    /// Index of the target node.
    pub to: usize,
    /// One label per transition, for example `x ↦ X, R`, in spec order.
    pub labels: Vec<String>,
    /// The indices of these transitions in the spec.
    pub transitions: Vec<usize>,
}

/// A state diagram.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Graph {
    /// The states, in spec order.
    pub nodes: Vec<Node>,
    /// The edges, by source and then target.
    pub edges: Vec<Edge>,
}

impl Graph {
    /// The diagram of a spec. Transitions naming unknown states are skipped.
    pub fn from_spec(spec: &MachineSpec) -> Self {
        let _ = spec;
        todo!("A4")
    }
}
