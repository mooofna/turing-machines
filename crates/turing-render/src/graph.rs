//! The state diagram as a graph with labelled edges.

use std::collections::{BTreeMap, HashMap};

use turing_core::{Dir, MachineSpec, TransitionSpec};

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
    ///
    /// There is one node per state in spec order and one edge per ordered pair of states
    /// with at least one transition between them. Labels follow [`transition_label`].
    ///
    /// ```
    /// use turing_core::{Dir, MachineSpec, StateSpec, TransitionSpec};
    /// use turing_render::Graph;
    ///
    /// let mut spec = MachineSpec::new();
    /// spec.states = vec![
    ///     StateSpec { name: "q0".into(), is_final: false },
    ///     StateSpec { name: "q1".into(), is_final: true },
    /// ];
    /// spec.start = "q0".into();
    /// for (read, to, write) in [('1', "q0", '0'), ('0', "q1", '1'), ('␣', "q1", '1')] {
    ///     spec.transitions.push(TransitionSpec {
    ///         from: "q0".into(),
    ///         read: vec![read],
    ///         to: to.into(),
    ///         write: vec![write],
    ///         dir: vec![Dir::R],
    ///         ..TransitionSpec::default()
    ///     });
    /// }
    /// let graph = Graph::from_spec(&spec);
    /// assert!(graph.nodes[0].is_start && graph.nodes[1].is_final);
    /// assert_eq!(graph.edges.len(), 2);
    /// assert_eq!(graph.edges[0].labels, ["1 ↦ 0, R"]);
    /// assert_eq!(graph.edges[1].labels, ["0 ↦ 1, R", "␣ ↦ 1, R"]);
    /// assert_eq!(graph.edges[1].transitions, [1, 2]);
    /// ```
    pub fn from_spec(spec: &MachineSpec) -> Self {
        let nodes = spec
            .states
            .iter()
            .map(|s| Node {
                name: s.name.clone(),
                is_start: s.name == spec.start,
                is_final: s.is_final,
            })
            .collect();

        // The first state of a name wins, as in `MachineSpec::state_index`.
        let mut index: HashMap<&str, usize> = HashMap::with_capacity(spec.states.len());
        for (i, s) in spec.states.iter().enumerate() {
            index.entry(s.name.as_str()).or_insert(i);
        }

        let mut pairs: BTreeMap<(usize, usize), Edge> = BTreeMap::new();
        for (i, t) in spec.transitions.iter().enumerate() {
            let (Some(&from), Some(&to)) = (index.get(t.from.as_str()), index.get(t.to.as_str()))
            else {
                continue;
            };
            let edge = pairs.entry((from, to)).or_insert_with(|| Edge {
                from,
                to,
                labels: Vec::new(),
                transitions: Vec::new(),
            });
            edge.labels.push(transition_label(spec, t));
            edge.transitions.push(i);
        }

        Self {
            nodes,
            edges: pairs.into_values().collect(),
        }
    }
}

/// The label of one transition in course notation.
///
/// A single-tape transition reads `a ↦ b, R`; a transition of a k-tape machine reads
/// `(a, b) ↦ (c, d), (R, L)`. Glyphs are taken from the spec. A blank glyph that would be
/// invisible (a space or another whitespace or control character) is shown as `␣`.
///
/// ```
/// use turing_core::{Dir, MachineSpec, TransitionSpec};
/// use turing_render::graph::transition_label;
///
/// let mut spec = MachineSpec::new();
/// spec.tapes = 2;
/// let t = TransitionSpec {
///     from: "q".into(),
///     read: vec!['a', '␣'],
///     to: "q".into(),
///     write: vec!['b', 'a'],
///     dir: vec![Dir::R, Dir::L],
///     ..TransitionSpec::default()
/// };
/// assert_eq!(transition_label(&spec, &t), "(a, ␣) ↦ (b, a), (R, L)");
/// ```
pub fn transition_label(spec: &MachineSpec, transition: &TransitionSpec) -> String {
    let shown = |c: char| {
        if c == spec.blank && (c.is_whitespace() || c.is_control()) {
            '\u{2423}'
        } else {
            c
        }
    };
    let mut out = String::new();
    push_tuple(&mut out, transition.read.iter().map(|&c| shown(c)));
    out.push_str(" \u{21a6} ");
    push_tuple(&mut out, transition.write.iter().map(|&c| shown(c)));
    out.push_str(", ");
    push_tuple(&mut out, transition.dir.iter().map(|&d: &Dir| d.as_char()));
    out
}

/// Writes one item as is and several items as `(a, b, c)`.
fn push_tuple(out: &mut String, items: impl ExactSizeIterator<Item = char>) {
    let parenthesise = items.len() != 1;
    if parenthesise {
        out.push('(');
    }
    for (i, c) in items.enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push(c);
    }
    if parenthesise {
        out.push(')');
    }
}
