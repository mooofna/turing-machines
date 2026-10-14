//! The enumeration driver and its statistics.

use crate::compact::CompactTm;
use crate::pipeline::{DeciderId, Pipeline};

/// Counts gathered during an enumeration.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Stats {
    /// Nodes of the tree normal form visited.
    pub nodes: u64,
    /// Nodes that reach an undefined transition (internal nodes of the tree).
    pub halting: u64,
    /// Nodes proved non-halting, per decider, in pipeline order.
    pub by_decider: Vec<(DeciderId, u64)>,
    /// Nodes no decider could resolve.
    pub holdouts: u64,
    /// The largest S found so far with the complete machine (its last undefined transition
    /// filled with a halting transition writing 1), in busy beaver convention.
    pub max_steps: Option<(u64, CompactTm)>,
    /// The largest Σ found so far with the complete machine.
    pub max_ones: Option<(u64, CompactTm)>,
}

/// Why an enumeration cannot start.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum EnumError {
    /// Unsupported size.
    #[error("enumeration supports 1 to {max_states} states and 2 to {max_symbols} symbols")]
    Size {
        /// Largest state count.
        max_states: usize,
        /// Largest symbol count.
        max_symbols: usize,
    },
    /// Invalid shard.
    #[error("shard {index} of {count} does not exist")]
    Shard {
        /// Index.
        index: u32,
        /// Count.
        count: u32,
    },
}

/// Depth-first enumeration of the tree normal form, classifying each node with a pipeline.
///
/// ```
/// use turing_bb::Enumerator;
/// let mut e = Enumerator::new(2, 2).unwrap();
/// let stats = e.run_to_completion();
/// assert_eq!(stats.nodes, 61);
/// assert_eq!(stats.max_steps.map(|(s, _)| s), Some(6));
/// assert_eq!(stats.holdouts, 0);
/// ```
#[derive(Debug)]
pub struct Enumerator {
    pub(crate) stack: Vec<crate::tnf::Node>,
    pub(crate) stats: Stats,
    pub(crate) holdouts: Vec<CompactTm>,
    pub(crate) examples: Vec<(DeciderId, Vec<CompactTm>)>,
    pub(crate) pipeline: Pipeline,
    pub(crate) shard: (u32, u32),
}

impl Enumerator {
    /// Enumerates machines with `states` states and `symbols` symbols with
    /// [`Pipeline::standard`].
    ///
    /// # Errors
    ///
    /// See [`EnumError`].
    pub fn new(states: u8, symbols: u8) -> Result<Self, EnumError> {
        Self::with_pipeline(states, symbols, Pipeline::standard())
    }

    /// Like [`Enumerator::new`] with a custom pipeline.
    ///
    /// # Errors
    ///
    /// See [`EnumError`].
    pub fn with_pipeline(states: u8, symbols: u8, pipeline: Pipeline) -> Result<Self, EnumError> {
        let _ = (states, symbols, pipeline);
        todo!("A3")
    }

    /// Restricts the enumeration to shard `index` of `count`, for parallel workers. The sums of
    /// the shards' statistics equal the full enumeration.
    ///
    /// # Errors
    ///
    /// See [`EnumError`].
    pub fn with_shard(self, index: u32, count: u32) -> Result<Self, EnumError> {
        let _ = (index, count);
        todo!("A3")
    }

    /// Visits at most `max_nodes` more nodes.
    pub fn advance(&mut self, max_nodes: u64) -> &Stats {
        let _ = max_nodes;
        todo!("A3")
    }

    /// Runs until every node has been visited.
    pub fn run_to_completion(&mut self) -> &Stats {
        while !self.is_done() {
            self.advance(1 << 16);
        }
        &self.stats
    }

    /// Whether every node has been visited.
    pub fn is_done(&self) -> bool {
        self.stack.is_empty()
    }

    /// The statistics so far.
    pub fn stats(&self) -> &Stats {
        &self.stats
    }

    /// Machines no decider resolved.
    pub fn holdouts(&self) -> &[CompactTm] {
        &self.holdouts
    }

    /// The first few machines resolved by decider `id`.
    pub fn examples(&self, id: DeciderId) -> &[CompactTm] {
        self.examples
            .iter()
            .find(|(d, _)| *d == id)
            .map_or(&[], |(_, v)| v.as_slice())
    }
}
