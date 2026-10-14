//! The individual deciders.
//!
//! Each decider is a port of the corresponding decider of the Coq-BB5 proof (bbchallenge
//! collaboration, arXiv:2509.12337) with the same parameters and, where it matters for the
//! result, the same search order. Each module documents the argument that makes its
//! non-halting answers sound. `docs/deciders.md` explains them together with the tree normal
//! form.
//!
//! * [`Loops`] ([`loops`]): halting, cyclers and translated cyclers by simulation.
//! * [`NGramCps`] ([`ngram_cps`]): n-gram closed position sets, on the plain machine or on
//!   the machine augmented with a history ([`augment::History`]) or a least-recently-used
//!   list ([`augment::Lru`]) of the updates of each cell.
//! * [`RepWl`] ([`repwl`]): repeated word lists.

pub mod augment;
pub mod loops;
pub mod ngram_cps;
pub mod repwl;

pub use loops::Loops;
pub use ngram_cps::{MAX_NGRAM, NGramCps};
pub use repwl::{MAX_WORD, RepWl};
