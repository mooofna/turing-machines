//! Types that cross the WebAssembly boundary, with generated TypeScript definitions.
//!
//! Characters cross as one-character strings, positions as UTF-16 code unit offsets (what
//! JavaScript strings and `CodeMirror` use) and step counts as `number`, which is exact up to
//! 2^53. Values that can grow beyond that cross as decimal strings.

use serde::{Deserialize, Serialize};
use tsify::Tsify;

/// What a machine is used for.
#[derive(Tsify, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum KindDto {
    /// Accepts or rejects its input.
    Acceptor,
    /// Computes a function.
    #[default]
    Transducer,
    /// Prints a language.
    Enumerator,
    /// Busy beaver candidate on the blank tape.
    Beaver,
}

/// A head move.
#[derive(Tsify, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirDto {
    /// Left.
    L,
    /// Right.
    R,
    /// No move.
    N,
}

/// How the output of a transducer is read as a number.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum DecoderDto {
    /// No number.
    #[default]
    Word,
    /// Count of a tally symbol.
    Unary {
        /// The tally glyph.
        symbol: String,
    },
    /// Binary digits.
    Binary {
        /// `lsb` or `msb` first.
        order: BitOrderDto,
    },
}

/// Bit order of [`DecoderDto::Binary`].
#[derive(Tsify, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BitOrderDto {
    /// Least significant bit first.
    Lsb,
    /// Most significant bit first.
    Msb,
}

/// A source range in UTF-16 code units.
#[derive(Tsify, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpanDto {
    /// First code unit.
    pub from: u32,
    /// One past the last code unit.
    pub to: u32,
}

/// A state.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StateDto {
    /// Name.
    pub name: String,
    /// Member of F.
    pub is_final: bool,
}

/// One transition δ(from, read) = (to, write, dir), one entry per tape in `read`, `write`,
/// `dir`.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TransitionDto {
    /// Current state.
    pub from: String,
    /// Read glyphs.
    pub read: Vec<String>,
    /// Next state.
    pub to: String,
    /// Written glyphs.
    pub write: Vec<String>,
    /// Moves.
    pub dir: Vec<DirDto>,
    /// End-of-line comment.
    #[tsify(optional)]
    pub comment: Option<String>,
    /// Comment lines above.
    #[serde(default)]
    pub leading: Vec<String>,
    /// Source range, if parsed from text.
    #[tsify(optional)]
    pub span: Option<SpanDto>,
}

/// Which parts were declared explicitly.
#[derive(Tsify, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
#[expect(
    clippy::struct_excessive_bools,
    reason = "mirrors turing_core::Declared"
)]
pub struct DeclaredDto {
    /// Q.
    pub states: bool,
    /// Σ.
    pub input_alphabet: bool,
    /// Γ.
    pub tape_alphabet: bool,
    /// q0.
    pub start: bool,
    /// Blank.
    pub blank: bool,
}

/// Comments not attached to a transition.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct CommentsDto {
    /// Before the first declaration.
    pub header: Vec<String>,
    /// After the last transition.
    pub trailing: Vec<String>,
}

/// The editable machine. This is the single source of truth of the playground.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MachineSpecDto {
    /// Name, possibly empty.
    pub name: String,
    /// Use.
    pub kind: KindDto,
    /// 1 to 3.
    pub tapes: u8,
    /// Blank glyph.
    pub blank: String,
    /// Σ.
    pub input_alphabet: Vec<String>,
    /// Γ, blank included.
    pub tape_alphabet: Vec<String>,
    /// Q.
    pub states: Vec<StateDto>,
    /// q0.
    pub start: String,
    /// δ in source order.
    pub transitions: Vec<TransitionDto>,
    /// Default input.
    pub input: String,
    /// Output decoder.
    pub output: DecoderDto,
    /// Explicit declarations.
    pub declared: DeclaredDto,
    /// Free comments.
    pub comments: CommentsDto,
}

/// Error or warning.
#[derive(Tsify, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SeverityDto {
    /// Error.
    Error,
    /// Warning.
    Warning,
}

/// A related location of a diagnostic.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RelatedDto {
    /// UTF-16 start.
    pub from: u32,
    /// UTF-16 end.
    pub to: u32,
    /// Explanation.
    pub note: String,
}

/// A diagnostic with UTF-16 positions and 1-based line and column.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticDto {
    /// Severity.
    pub severity: SeverityDto,
    /// Stable code, for example `E004`.
    pub code: String,
    /// Message.
    pub message: String,
    /// Suggestion.
    #[tsify(optional)]
    pub help: Option<String>,
    /// UTF-16 start, if the diagnostic points into source text.
    #[tsify(optional)]
    pub from: Option<u32>,
    /// UTF-16 end.
    #[tsify(optional)]
    pub to: Option<u32>,
    /// 1-based line of `from`.
    #[tsify(optional)]
    pub line: Option<u32>,
    /// 1-based column of `from`, in UTF-16 code units.
    #[tsify(optional)]
    pub column: Option<u32>,
    /// Related locations.
    pub related: Vec<RelatedDto>,
}

/// The result of parsing a description or decoding a shared link.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ParseResultDto {
    /// The machine, if there were no errors.
    pub spec: Option<MachineSpecDto>,
    /// All diagnostics.
    pub diagnostics: Vec<DiagnosticDto>,
    /// The decoded description, set by `decodeShare` whenever the link could be unpacked, so
    /// that a shared machine with errors can still be opened in the text editor. Diagnostic
    /// positions refer to this text.
    #[tsify(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// A list of diagnostics.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticsDto {
    /// The diagnostics.
    pub diagnostics: Vec<DiagnosticDto>,
}

/// A placed state of the diagram.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphNodeDto {
    /// State name.
    pub name: String,
    /// q0.
    pub is_start: bool,
    /// In F.
    pub is_final: bool,
    /// Centre x.
    pub x: f64,
    /// Centre y.
    pub y: f64,
}

/// An edge of the diagram, all transitions between two states.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GraphEdgeDto {
    /// Source node index.
    pub from: usize,
    /// Target node index.
    pub to: usize,
    /// One label per transition, `a ↦ b, R`.
    pub labels: Vec<String>,
    /// Spec indices of the transitions.
    pub transitions: Vec<usize>,
}

/// A laid-out diagram.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GraphLayoutDto {
    /// States with positions.
    pub nodes: Vec<GraphNodeDto>,
    /// Edges.
    pub edges: Vec<GraphEdgeDto>,
    /// Bounding box width.
    pub width: f64,
    /// Bounding box height.
    pub height: f64,
    /// Node radius.
    pub radius: f64,
}

/// The drawing of one edge.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EdgeRouteDto {
    /// Edge index.
    pub edge: usize,
    /// SVG path data.
    pub path: String,
    /// Arrow tip x.
    pub tip_x: f64,
    /// Arrow tip y.
    pub tip_y: f64,
    /// Arrow direction in radians.
    pub tip_angle: f64,
    /// Label anchor x.
    pub label_x: f64,
    /// Label anchor y.
    pub label_y: f64,
    /// `start`, `middle` or `end`.
    pub label_anchor: String,
}

/// All edge drawings.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EdgeRoutesDto {
    /// One per edge.
    pub routes: Vec<EdgeRouteDto>,
}

/// The visible part of one tape.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TapeWindowDto {
    /// Head position.
    pub head: f64,
    /// Position of `cells[0]`.
    pub start: f64,
    /// Glyphs from `start` on.
    pub cells: Vec<String>,
}

/// Running, halted, or proved never to halt.
#[derive(Tsify, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RunStatusDto {
    /// Can step.
    Running,
    /// No transition defined.
    Halted,
    /// Proved non-halting by the simulator.
    NeverHalts,
}

/// Accept or reject, for acceptors that halted.
#[derive(Tsify, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VerdictKindDto {
    /// Halted in F.
    Accepted,
    /// Halted outside F.
    Rejected,
}

/// The output of a transducer.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OutputDto {
    /// The word.
    pub word: String,
    /// Its decoded value as a decimal string.
    #[tsify(optional)]
    pub value: Option<String>,
}

/// Everything the playground shows about the current configuration.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDto {
    /// Steps since the start.
    pub step: f64,
    /// Current state name.
    pub state: String,
    /// Status.
    pub status: RunStatusDto,
    /// For halted acceptors.
    #[tsify(optional)]
    pub verdict: Option<VerdictKindDto>,
    /// Why the machine never halts, if proved.
    #[tsify(optional)]
    pub reason: Option<String>,
    /// One window per tape.
    pub tapes: Vec<TapeWindowDto>,
    /// Smallest and largest head position of tape 1 so far.
    pub extent: [f64; 2],
    /// The configuration in course notation, one line per tape.
    pub configuration: String,
    /// Spec index of the transition the next step applies.
    #[tsify(optional)]
    pub next_transition: Option<usize>,
    /// Spec index of the transition the last step applied.
    #[tsify(optional)]
    pub last_transition: Option<usize>,
    /// Non-blank cells on tape 1.
    pub ones: f64,
    /// Output for halted transducers.
    #[tsify(optional)]
    pub output: Option<OutputDto>,
    /// Words printed by an enumerator so far (the last 50).
    pub emitted: Vec<String>,
    /// How many words an enumerator printed in total.
    pub emitted_count: f64,
}

/// Where a run stops early.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct BreakpointsDto {
    /// Stop before applying any of these transitions (spec indices).
    pub transitions: Vec<usize>,
    /// Stop after entering any of these states.
    pub states: Vec<String>,
    /// Stop after an enumerator prints a word.
    #[serde(default)]
    pub on_output: bool,
}

/// Why a run stopped.
#[derive(Tsify, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum StopReasonDto {
    /// The step budget was used up.
    Budget,
    /// The machine halted.
    Halted,
    /// A breakpoint was hit.
    Breakpoint,
    /// The machine was proved never to halt.
    NeverHalts,
}

/// The result of [`crate::Session::run`].
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RunResultDto {
    /// The configuration afterwards.
    pub snapshot: SnapshotDto,
    /// Why it stopped.
    pub reason: StopReasonDto,
    /// Steps made by this call.
    pub steps_done: f64,
}

/// Row mapping of a space-time diagram.
#[derive(Tsify, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
pub enum SpaceTimeModeDto {
    /// One row per step from `firstStep`.
    Exact {
        /// First step.
        first_step: f64,
        /// Pixel width of a cell, or fit automatically.
        #[tsify(optional)]
        cell_px: Option<u32>,
    },
    /// Evenly sampled rows over the whole run.
    Overview {
        /// Step budget.
        max_steps: f64,
    },
}

/// What to render.
#[derive(Tsify, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct SpaceTimeRequestDto {
    /// Pixels.
    pub width: u32,
    /// Pixels.
    pub height: u32,
    /// Row mapping.
    pub mode: SpaceTimeModeDto,
}

/// Colours as CSS hex strings.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PaletteDto {
    /// Background, for example `#ffffff`.
    pub background: String,
    /// Ink, for example `#111111`.
    pub ink: String,
    /// Head, for example `#215caf`.
    pub accent: String,
}

/// Rendering progress.
#[derive(Tsify, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SpaceTimeProgressDto {
    /// Rows finished.
    pub rows_done: u32,
    /// Steps simulated.
    pub steps: f64,
    /// Complete.
    pub done: bool,
    /// The machine halted within the rendered range.
    pub halted: bool,
}

/// The cell under a pixel.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CellRefDto {
    /// Step of the row.
    pub step: f64,
    /// First tape position of the column.
    pub from: f64,
    /// Last tape position of the column.
    pub to: f64,
    /// Glyph, if the column is a single cell.
    #[tsify(optional)]
    pub glyph: Option<String>,
}

/// Count of machines resolved by one decider.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeciderCountDto {
    /// Stable key, for example `ngram-cps`.
    pub key: String,
    /// English label.
    pub label: String,
    /// Machines proved non-halting.
    pub non_halting: f64,
}

/// A record value with its machine.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RecordDto {
    /// S or Σ.
    pub value: f64,
    /// Standard format.
    pub machine: String,
}

/// Enumeration progress.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EnumProgressDto {
    /// Nodes visited.
    pub nodes: f64,
    /// Halting nodes.
    pub halting: f64,
    /// Per decider, in pipeline order.
    pub deciders: Vec<DeciderCountDto>,
    /// Unresolved nodes.
    pub holdouts: f64,
    /// Largest S so far.
    #[tsify(optional)]
    pub max_steps: Option<RecordDto>,
    /// Largest Σ so far.
    #[tsify(optional)]
    pub max_ones: Option<RecordDto>,
    /// Finished.
    pub done: bool,
}

/// A list of machines in standard format.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MachineListDto {
    /// Standard format strings.
    pub machines: Vec<String>,
}

/// The answer of the decider pipeline for one machine.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum VerdictDto {
    /// Reaches an undefined transition.
    Halts {
        /// Steps, busy beaver convention (the halting step counted).
        steps: f64,
        /// Non-blank cells after halting.
        ones: f64,
    },
    /// Proved non-halting.
    NonHalting {
        /// Decider key.
        decider: String,
        /// Decider label.
        label: String,
    },
    /// No decider could tell.
    Unknown,
    /// The machine cannot be decided here (too large, several tapes, `N` moves).
    Unsupported {
        /// Why.
        reason: String,
    },
}

/// One Antihydra rule application observed on the machine.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AntihydraRowDto {
    /// Iteration.
    pub iteration: u32,
    /// a.
    pub a: f64,
    /// b.
    pub b: f64,
    /// Machine steps at A(a, b).
    pub steps: f64,
    /// Steps since the previous row.
    pub delta: f64,
    /// Steps the rule predicts.
    pub predicted: f64,
    /// Everything matches.
    pub agrees: bool,
}

/// New rows of the machine check.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AntihydraRowsDto {
    /// Rows found by this call.
    pub rows: Vec<AntihydraRowDto>,
    /// The check has finished.
    pub done: bool,
}

/// A sample of the rule walk.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WalkSampleDto {
    /// Iteration.
    pub iteration: f64,
    /// Balance c.
    pub balance: f64,
    /// Bit length of h.
    pub bits: f64,
}

/// Summary of the rule walk.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WalkSummaryDto {
    /// Iterations.
    pub iterations: f64,
    /// c became negative.
    pub halted: bool,
    /// Iterations with h even.
    pub evens: f64,
    /// Iterations with h odd.
    pub odds: f64,
    /// Current c.
    pub balance: f64,
    /// Smallest c after iteration 0.
    pub min_balance: f64,
    /// Bit length of h.
    pub bits: f64,
    /// Samples for plotting.
    pub samples: Vec<WalkSampleDto>,
}

/// `base ↑↑ height`.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TowerDto {
    /// Base.
    pub base: u32,
    /// Height.
    pub height: u32,
    /// Decimal digits, if short enough.
    #[tsify(optional)]
    pub exact: Option<String>,
    /// Number of decimal digits as a decimal string, if known.
    #[tsify(optional)]
    pub digits: Option<String>,
}
