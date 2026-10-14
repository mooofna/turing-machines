//! Deciders and the pipeline that applies them in order.

use crate::compact::CompactTm;
use crate::deciders::{Loops, NGramCps, RepWl};

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

    /// The decider with the given [`key`](DeciderId::key).
    ///
    /// ```
    /// use turing_bb::DeciderId;
    /// assert_eq!(DeciderId::from_key("repwl"), Some(DeciderId::RepWl));
    /// assert_eq!(DeciderId::from_key("magic"), None);
    /// ```
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|id| id.key() == key)
    }
}

/// What a decider found out about a machine started on the blank tape.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Verdict {
    /// The machine reaches the undefined transition δ(`at.0`, `at.1`) after `steps` steps with
    /// `ones` non-blank cells. The halting step itself is not counted here.
    ///
    /// A transition to the halting state `Z` counts like an undefined transition: `steps` is
    /// the number of steps before it is taken.
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

    /// The decider with its parameters, in Coq-BB5's naming where it has one, for example
    /// `NGRAM_CPS_IMPL2_params_1_1_100`.
    fn name(&self) -> String {
        self.id().key().to_owned()
    }

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
            .entries(self.stages.iter().map(|d| d.name()))
            .finish()
    }
}

/// The n-gram CPS stages of Coq-BB5's BB(4) pipeline after the plain ones, in order:
/// `(history, len_l, len_r, gas)`, with `None` for the LRU stage.
const BB4_AUGMENTED: [(Option<usize>, usize, usize, u64); 10] = [
    (Some(2), 2, 2, 1600),
    (Some(2), 3, 3, 1600),
    (Some(4), 2, 2, 600),
    (Some(4), 3, 3, 1600),
    (Some(6), 2, 2, 3200),
    (Some(6), 3, 3, 3200),
    (Some(8), 2, 2, 1600),
    (Some(8), 3, 3, 1600),
    (None, 2, 2, 10_000),
    (Some(10), 4, 4, 10_000),
];

impl Pipeline {
    /// The pipeline with the deciders and parameters of the Coq-BB5 proof (see
    /// `docs/deciders.md`).
    ///
    /// This is the pipeline of Coq-BB5's proof of BB(4) = 107 (`BB4_Deciders_Pipeline.v`):
    /// [`Loops`] with 107 steps, plain n-gram CPS with radius 1, 2 and 3, eight n-gram CPS
    /// stages with history, one with an LRU list, one more with history and repeated word
    /// lists, fifteen stages in all. It is a subset of the BB(5) pipeline and decides every
    /// machine with at most four states and two symbols.
    ///
    /// ```
    /// use turing_bb::Pipeline;
    /// assert_eq!(Pipeline::standard().stage_names().len(), 15);
    /// ```
    pub fn standard() -> Self {
        let mut stages: Vec<Box<dyn Decider>> = vec![
            Box::new(Loops::new(107)),
            Box::new(NGramCps::plain(1, 1, 100)),
            Box::new(NGramCps::plain(2, 2, 200)),
            Box::new(NGramCps::plain(3, 3, 400)),
        ];
        for (history, len_l, len_r, gas) in BB4_AUGMENTED {
            stages.push(match history {
                Some(h) => Box::new(NGramCps::history(h, len_l, len_r, gas)),
                None => Box::new(NGramCps::lru(len_l, len_r, gas)),
            });
        }
        stages.push(Box::new(RepWl::new(4, 3, 320, 10_000)));
        Self { stages }
    }

    /// The exact pipeline Coq-BB5 uses for machines with `states` states and `symbols`
    /// symbols, if it has one: BB(2), BB(3), BB(4) and BB(2,3).
    ///
    /// The step count of [`Loops`] is the busy beaver value itself in each case (6, 21, 107
    /// and 38). This does not presuppose the value: a machine that halted later would be
    /// recognised by no stage and remain a holdout, so an enumeration without holdouts
    /// certifies that no machine halts later.
    ///
    /// ```
    /// use turing_bb::Pipeline;
    /// let bb2 = Pipeline::coq(2, 2).unwrap();
    /// assert_eq!(bb2.stage_names(), ["LOOP1_params_6", "NGRAM_CPS_IMPL2_params_1_1_100"]);
    /// assert!(Pipeline::coq(5, 2).is_none());
    /// ```
    pub fn coq(states: u8, symbols: u8) -> Option<Self> {
        let list: Vec<Box<dyn Decider>> = match (states, symbols) {
            (2, 2) => vec![
                Box::new(Loops::new(6)),
                Box::new(NGramCps::plain(1, 1, 100)),
            ],
            (3, 2) => vec![
                Box::new(Loops::new(21)),
                Box::new(NGramCps::plain(1, 1, 100)),
                Box::new(NGramCps::plain(2, 2, 200)),
                Box::new(NGramCps::plain(3, 3, 400)),
                Box::new(NGramCps::history(2, 2, 2, 1600)),
            ],
            (4, 2) => return Some(Self::standard()),
            (2, 3) => vec![
                Box::new(Loops::new(38)),
                Box::new(NGramCps::plain(1, 1, 400)),
                Box::new(NGramCps::plain(2, 2, 800)),
                Box::new(NGramCps::plain(3, 3, 400)),
                Box::new(RepWl::new(2, 3, 320, 400)),
            ],
            _ => return None,
        };
        Some(Self { stages: list })
    }

    /// The pipeline [`crate::Enumerator::new`] uses: [`Pipeline::coq`] where Coq-BB5 has a
    /// pipeline for this size, otherwise [`Pipeline::standard`].
    pub fn for_size(states: u8, symbols: u8) -> Self {
        Self::coq(states, symbols).unwrap_or_else(Self::standard)
    }

    /// A pipeline with the given stages.
    pub fn new(stages: Vec<Box<dyn Decider>>) -> Self {
        Self { stages }
    }

    /// The [`Decider::name`] of every stage, in order.
    pub fn stage_names(&self) -> Vec<String> {
        self.stages.iter().map(|d| d.name()).collect()
    }

    /// The [`Decider::id`] of every stage, in order.
    pub fn stage_ids(&self) -> Vec<DeciderId> {
        self.stages.iter().map(|d| d.id()).collect()
    }

    /// Applies the stages in order and returns the first answer that is not
    /// [`Verdict::Unknown`].
    pub fn decide(&mut self, tm: &CompactTm) -> Verdict {
        self.decide_with_stage(tm).0
    }

    /// Like [`Pipeline::decide`], and also returns the index of the stage that answered.
    ///
    /// ```
    /// use turing_bb::{CompactTm, Pipeline, Verdict};
    /// let mut p = Pipeline::coq(2, 2).unwrap();
    /// let tm: CompactTm = "0RB---_1LA0LA".parse().unwrap();
    /// let (verdict, stage) = p.decide_with_stage(&tm);
    /// assert!(matches!(verdict, Verdict::NonHalting(_)));
    /// assert_eq!(stage, Some(1));
    /// ```
    pub fn decide_with_stage(&mut self, tm: &CompactTm) -> (Verdict, Option<usize>) {
        for (i, stage) in self.stages.iter_mut().enumerate() {
            let verdict = stage.decide(tm);
            if verdict != Verdict::Unknown {
                return (verdict, Some(i));
            }
        }
        (Verdict::Unknown, None)
    }
}
