//! The enumeration driver and its statistics.

use crate::compact::{CompactAction, CompactTm, MAX_STATES, MAX_SYMBOLS, Next};
use crate::pipeline::{DeciderId, Pipeline, Verdict};
use crate::tnf::Node;

/// The depth at which [`Enumerator::with_shard`] splits the tree between shards.
pub const SHARD_DEPTH: u8 = 3;

/// How many example machines are kept per decider.
pub const EXAMPLES_PER_DECIDER: usize = 8;

/// How many holdouts are kept as machines; [`Stats::holdouts`] counts all of them.
pub const MAX_STORED_HOLDOUTS: usize = 1 << 16;

/// Counts gathered during an enumeration.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Stats {
    /// Nodes of the tree normal form visited.
    pub nodes: u64,
    /// Nodes that reach an undefined transition (internal nodes of the tree).
    pub halting: u64,
    /// Nodes proved non-halting, per decider, in pipeline order: one entry for each of
    /// [`DeciderId::ALL`], whether the pipeline uses it or not.
    pub by_decider: Vec<(DeciderId, u64)>,
    /// Nodes no decider could resolve.
    pub holdouts: u64,
    /// The largest S found so far with the complete machine (its last undefined transition
    /// filled with a halting transition writing 1), in busy beaver convention.
    pub max_steps: Option<(u64, CompactTm)>,
    /// The largest Σ found so far with the complete machine.
    pub max_ones: Option<(u64, CompactTm)>,
    /// The Σ of the machine in [`Stats::max_steps`]. Among machines with the same S, the
    /// record is the one that leaves the most ones.
    pub max_steps_ones: u64,
    /// The S of the machine in [`Stats::max_ones`]. Among machines with the same Σ, the
    /// record is the one that runs longest.
    pub max_ones_steps: u64,
    /// Nodes proved non-halting per stage of the pipeline, named by [`crate::Decider::name`],
    /// in pipeline order.
    pub by_stage: Vec<(String, u64)>,
}

impl Stats {
    /// Nodes proved non-halting by any decider.
    pub fn non_halting(&self) -> u64 {
        self.by_decider.iter().map(|&(_, n)| n).sum()
    }

    /// Nodes proved non-halting by decider `id`.
    pub fn resolved_by(&self, id: DeciderId) -> u64 {
        self.by_decider
            .iter()
            .find(|&&(d, _)| d == id)
            .map_or(0, |&(_, n)| n)
    }

    /// Adds the counts of another shard of the same enumeration and keeps the larger records.
    ///
    /// Records are compared by [`record_beats`], which does not depend on the order in which
    /// machines are visited, so merging the shards of an enumeration in any order gives the
    /// records of the unsharded enumeration.
    pub fn merge(&mut self, other: &Self) {
        self.nodes += other.nodes;
        self.halting += other.halting;
        self.holdouts += other.holdouts;
        for &(id, n) in &other.by_decider {
            match self.by_decider.iter_mut().find(|(d, _)| *d == id) {
                Some((_, m)) => *m += n,
                None => self.by_decider.push((id, n)),
            }
        }
        for (name, n) in &other.by_stage {
            match self.by_stage.iter_mut().find(|(s, _)| s == name) {
                Some((_, m)) => *m += n,
                None => self.by_stage.push((name.clone(), *n)),
            }
        }
        if let Some((s, tm)) = other.max_steps
            && record_beats(
                (s, other.max_steps_ones, &tm),
                self.max_steps,
                self.max_steps_ones,
            )
        {
            self.max_steps = other.max_steps;
            self.max_steps_ones = other.max_steps_ones;
        }
        if let Some((sigma, tm)) = other.max_ones
            && record_beats(
                (sigma, other.max_ones_steps, &tm),
                self.max_ones,
                self.max_ones_steps,
            )
        {
            self.max_ones = other.max_ones;
            self.max_ones_steps = other.max_ones_steps;
        }
    }
}

/// Whether a candidate record `(value, tie, machine)` beats the current record `current` with
/// tie value `current_tie`.
///
/// A larger value wins; on equal values the larger tie value wins (the number of ones for S,
/// the number of steps for Σ); on equal tie values the machine with the smaller transition table
/// wins. This is a total order on records, so the result does not depend on the order of the
/// enumeration or on how it is split into shards.
pub(crate) fn record_beats(
    (value, tie, machine): (u64, u64, &CompactTm),
    current: Option<(u64, CompactTm)>,
    current_tie: u64,
) -> bool {
    current.is_none_or(|(best, best_machine)| {
        (value, tie, std::cmp::Reverse(machine.table))
            > (best, current_tie, std::cmp::Reverse(best_machine.table))
    })
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

/// One node visited by [`Enumerator::advance_with`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Visit<'a> {
    /// The partial machine.
    pub tm: &'a CompactTm,
    /// Its depth in the tree, 0 for the root.
    pub depth: u8,
    /// What the pipeline found.
    pub verdict: Verdict,
    /// The name of the stage that answered, `None` for a holdout.
    pub stage: Option<&'a str>,
}

/// Depth-first enumeration of the tree normal form, classifying each node with a pipeline.
///
/// The nodes are visited in exactly the order of Coq-BB5's enumeration, see [`crate::tnf`].
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
    pub(crate) stack: Vec<Node>,
    pub(crate) stats: Stats,
    pub(crate) holdouts: Vec<CompactTm>,
    pub(crate) examples: Vec<(DeciderId, Vec<CompactTm>)>,
    pub(crate) pipeline: Pipeline,
    pub(crate) shard: (u32, u32),
    pub(crate) states: u8,
    pub(crate) symbols: u8,
    /// The names of the pipeline's stages.
    pub(crate) stage_names: Vec<String>,
    /// How many nodes at [`SHARD_DEPTH`] have been met so far, by every shard alike.
    pub(crate) split_index: u64,
}

impl Enumerator {
    /// Enumerates machines with `states` states and `symbols` symbols with
    /// [`Pipeline::for_size`]: the exact pipeline of Coq-BB5 for BB(2), BB(3), BB(4) and
    /// BB(2,3), and [`Pipeline::standard`] otherwise.
    ///
    /// # Errors
    ///
    /// See [`EnumError`].
    pub fn new(states: u8, symbols: u8) -> Result<Self, EnumError> {
        Self::check_size(states, symbols)?;
        Self::with_pipeline(states, symbols, Pipeline::for_size(states, symbols))
    }

    /// Like [`Enumerator::new`] with a custom pipeline.
    ///
    /// # Errors
    ///
    /// See [`EnumError`].
    pub fn with_pipeline(states: u8, symbols: u8, pipeline: Pipeline) -> Result<Self, EnumError> {
        Self::check_size(states, symbols)?;
        let stage_names = pipeline.stage_names();
        let mut e = Self {
            stack: Vec::new(),
            stats: Stats::default(),
            holdouts: Vec::new(),
            examples: Vec::new(),
            pipeline,
            shard: (0, 1),
            states,
            symbols,
            stage_names,
            split_index: 0,
        };
        e.reset();
        Ok(e)
    }

    fn check_size(states: u8, symbols: u8) -> Result<(), EnumError> {
        let ok = (1..=MAX_STATES).contains(&usize::from(states))
            && (2..=MAX_SYMBOLS).contains(&usize::from(symbols));
        if ok {
            Ok(())
        } else {
            Err(EnumError::Size {
                max_states: MAX_STATES,
                max_symbols: MAX_SYMBOLS,
            })
        }
    }

    /// Starts over from the root.
    fn reset(&mut self) {
        self.stack.clear();
        self.stack.push(Node::root(self.states, self.symbols));
        self.stats = Stats {
            by_decider: DeciderId::ALL.iter().map(|&id| (id, 0)).collect(),
            by_stage: self.stage_names.iter().map(|s| (s.clone(), 0)).collect(),
            ..Stats::default()
        };
        self.holdouts.clear();
        self.examples = DeciderId::ALL.iter().map(|&id| (id, Vec::new())).collect();
        self.split_index = 0;
    }

    /// Restricts the enumeration to shard `index` of `count`, for parallel workers. The sums of
    /// the shards' statistics equal the full enumeration.
    ///
    /// The nodes at depth [`SHARD_DEPTH`] are dealt out in turn: shard `w` explores the
    /// subtrees of the nodes whose position among them is `w` modulo `count`. Every shard
    /// classifies the few nodes above that depth to find them, but only shard 0 counts them.
    /// The enumeration starts over from the root.
    ///
    /// ```
    /// use turing_bb::{Enumerator, Stats};
    /// let mut total = Stats::default();
    /// for w in 0..3 {
    ///     let mut e = Enumerator::new(3, 2).unwrap().with_shard(w, 3).unwrap();
    ///     total.merge(e.run_to_completion());
    /// }
    /// assert_eq!(total.nodes, 5_417);
    /// ```
    ///
    /// # Errors
    ///
    /// See [`EnumError`].
    pub fn with_shard(mut self, index: u32, count: u32) -> Result<Self, EnumError> {
        if index >= count {
            return Err(EnumError::Shard { index, count });
        }
        self.shard = (index, count);
        self.reset();
        Ok(self)
    }

    /// Visits at most `max_nodes` more nodes.
    pub fn advance(&mut self, max_nodes: u64) -> &Stats {
        self.advance_with(max_nodes, |_| {})
    }

    /// Like [`Enumerator::advance`], and calls `visit` for every node this shard counts, in
    /// the order of the enumeration.
    ///
    /// ```
    /// use turing_bb::Enumerator;
    /// let mut first = Vec::new();
    /// Enumerator::new(2, 2).unwrap().advance_with(3, |v| first.push(v.tm.to_string()));
    /// assert_eq!(first, ["------_------", "0RA---_------", "1RA---_------"]);
    /// ```
    pub fn advance_with(&mut self, max_nodes: u64, mut visit: impl FnMut(Visit<'_>)) -> &Stats {
        let mut budget = max_nodes;
        while budget > 0 {
            let Some(node) = self.stack.pop() else {
                break;
            };
            if node.depth == SHARD_DEPTH {
                let index = self.split_index;
                self.split_index += 1;
                if index % u64::from(self.shard.1) != u64::from(self.shard.0) {
                    continue;
                }
            }
            budget -= 1;
            let (verdict, stage) = self.pipeline.decide_with_stage(&node.tm);
            if let Verdict::Halts { at, .. } = verdict {
                node.push_children(at, &mut self.stack);
            }
            if node.depth >= SHARD_DEPTH || self.shard.0 == 0 {
                self.record(&node.tm, verdict, stage);
                visit(Visit {
                    tm: &node.tm,
                    depth: node.depth,
                    verdict,
                    stage: stage.map(|i| self.stage_names[i].as_str()),
                });
            }
        }
        &self.stats
    }

    /// Adds one classified node to the statistics.
    fn record(&mut self, tm: &CompactTm, verdict: Verdict, stage: Option<usize>) {
        let stats = &mut self.stats;
        stats.nodes += 1;
        match verdict {
            Verdict::Halts { steps, ones, at } => {
                stats.halting += 1;
                let mut complete = *tm;
                complete.set(
                    at.0,
                    at.1,
                    Some(CompactAction {
                        write: 1,
                        dir: turing_core::Dir::R,
                        next: Next::Halt,
                    }),
                );
                let s = steps + 1;
                let sigma = ones + u64::from(at.1 == 0);
                if record_beats((s, sigma, &complete), stats.max_steps, stats.max_steps_ones) {
                    stats.max_steps = Some((s, complete));
                    stats.max_steps_ones = sigma;
                }
                if record_beats((sigma, s, &complete), stats.max_ones, stats.max_ones_steps) {
                    stats.max_ones = Some((sigma, complete));
                    stats.max_ones_steps = s;
                }
            }
            Verdict::NonHalting(id) => {
                if let Some((_, n)) = stats.by_decider.iter_mut().find(|(d, _)| *d == id) {
                    *n += 1;
                }
                if let Some((_, n)) = stage.and_then(|i| stats.by_stage.get_mut(i)) {
                    *n += 1;
                }
                if let Some((_, list)) = self.examples.iter_mut().find(|(d, _)| *d == id)
                    && list.len() < EXAMPLES_PER_DECIDER
                {
                    list.push(*tm);
                }
            }
            Verdict::Unknown => {
                stats.holdouts += 1;
                if self.holdouts.len() < MAX_STORED_HOLDOUTS {
                    self.holdouts.push(*tm);
                }
            }
        }
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

    /// Machines no decider resolved (at most [`MAX_STORED_HOLDOUTS`] of them).
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

    /// The pipeline that classifies the nodes.
    pub fn pipeline(&self) -> &Pipeline {
        &self.pipeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_sizes_and_shards() {
        assert!(Enumerator::new(0, 2).is_err());
        assert!(Enumerator::new(7, 2).is_err());
        assert!(Enumerator::new(2, 1).is_err());
        assert!(Enumerator::new(2, 9).is_err());
        let e = Enumerator::new(2, 2).unwrap();
        assert_eq!(
            e.with_shard(2, 2).unwrap_err(),
            EnumError::Shard { index: 2, count: 2 }
        );
    }

    #[test]
    fn advance_respects_the_budget() {
        let mut e = Enumerator::new(3, 2).unwrap();
        assert_eq!(e.advance(10).nodes, 10);
        assert_eq!(e.advance(0).nodes, 10);
        assert!(!e.is_done());
        assert_eq!(e.run_to_completion().nodes, 5_417);
        assert!(e.is_done());
    }

    #[test]
    fn keeps_examples() {
        let mut e = Enumerator::new(3, 2).unwrap();
        e.run_to_completion();
        assert_eq!(e.examples(DeciderId::Loops).len(), EXAMPLES_PER_DECIDER);
        assert_eq!(e.examples(DeciderId::RepWl).len(), 0);
        assert_eq!(
            e.examples(DeciderId::Loops)[0].to_string(),
            "0RA---_------_------"
        );
    }
}
