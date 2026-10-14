//! Deciders and the pipeline that applies them in order.

use crate::compact::CompactTm;

/// The deciders, in the order of the standard pipeline.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DeciderId {
    /// Simulation that finds halting, cyclers and translated cyclers.
    Loops,
    /// n-gram closed position set on the plain machine.
    NGramCps,
    /// n-gram closed position set on the machine augmented with a history of transitions.
    NGramCpsHistory,
    /// n-gram closed position set on the machine augmented with a least-recently-used list.
    NGramCpsLru,
    /// Repeated word lists.
    RepWl,
}

impl DeciderId {
    /// Every decider, in pipeline order.
    pub const ALL: [Self; 5] = [
        Self::Loops,
        Self::NGramCps,
        Self::NGramCpsHistory,
        Self::NGramCpsLru,
        Self::RepWl,
    ];

    /// A short English name for tables.
    pub fn label(self) -> &'static str {
        match self {
            Self::Loops => "Cyclers and translated cyclers",
            Self::NGramCps => "n-gram closed position sets",
            Self::NGramCpsHistory => "n-gram CPS with history",
            Self::NGramCpsLru => "n-gram CPS with LRU",
            Self::RepWl => "Repeated word lists",
        }
    }

    /// A stable identifier for data exchange, for example `ngram-cps`.
    pub fn key(self) -> &'static str {
        match self {
            Self::Loops => "loops",
            Self::NGramCps => "ngram-cps",
            Self::NGramCpsHistory => "ngram-cps-history",
            Self::NGramCpsLru => "ngram-cps-lru",
            Self::RepWl => "repwl",
        }
    }
}

/// What a decider found out about a machine started on the blank tape.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Verdict {
    /// The machine reaches the undefined transition δ(`at.0`, `at.1`) after `steps` steps with
    /// `ones` non-blank cells. The halting step itself is not counted here.
    Halts {
        /// Steps before reaching the undefined transition.
        steps: u64,
        /// Non-blank cells at that moment.
        ones: u64,
        /// State and symbol of the undefined transition.
        at: (u8, u8),
    },
    /// The decider proved that the machine never halts.
    NonHalting(DeciderId),
    /// The decider could not tell.
    Unknown,
}

/// A procedure that tries to decide whether a machine halts from the blank tape.
pub trait Decider: Send {
    /// Which decider this is.
    fn id(&self) -> DeciderId;

    /// Returns [`Verdict::NonHalting`] only with a proof; [`Verdict::Halts`] only after
    /// actually reaching an undefined transition.
    fn decide(&mut self, tm: &CompactTm) -> Verdict;
}

/// Deciders applied in order until one gives an answer.
pub struct Pipeline {
    pub(crate) stages: Vec<Box<dyn Decider>>,
}

impl core::fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list()
            .entries(self.stages.iter().map(|d| d.id()))
            .finish()
    }
}

impl Pipeline {
    /// The pipeline with the deciders and parameters of the Coq-BB5 proof (see
    /// `docs/deciders.md`).
    pub fn standard() -> Self {
        todo!("A3")
    }

    /// A pipeline with the given stages.
    pub fn new(stages: Vec<Box<dyn Decider>>) -> Self {
        Self { stages }
    }

    /// Applies the stages in order and returns the first answer that is not
    /// [`Verdict::Unknown`].
    pub fn decide(&mut self, tm: &CompactTm) -> Verdict {
        for stage in &mut self.stages {
            let verdict = stage.decide(tm);
            if verdict != Verdict::Unknown {
                return verdict;
            }
        }
        Verdict::Unknown
    }
}
