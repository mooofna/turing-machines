//! Space-time diagrams.
//!
//! A space-time diagram draws a run of a single-tape machine as an image: time runs
//! downwards, the tape runs across, and the head is marked in the accent colour.
//!
//! Rendering fills an index buffer with one byte per pixel (see [`index`]) and colours it
//! separately with [`SpaceTime::rgba`], so that a change of theme does not run the machine
//! again. Rendering is incremental: [`SpaceTime::advance`] simulates a bounded number of steps
//! per call, so a web worker can spread a long run over many short slices. Splitting the work
//! differently never changes the image.
//!
//! # Modes
//!
//! * [`SpaceTimeMode::Exact`] shows one configuration per row, from a given step on, until
//!   the machine halts or the image is full. Cells are square, `cell_px` pixels wide and
//!   high, so a row is `cell_px` pixel rows high. Without an explicit size, the largest size
//!   up to [`MAX_AUTO_CELL_PX`] is chosen at which every written cell and every cell the head
//!   visits lie within the width and, if the machine halts within the image, every row up
//!   to the halt fits; a longer run shows at least [`MIN_AUTO_ROWS`] rows where the image is
//!   high enough. If the tape does not fit even at one pixel per cell, rows are one pixel
//!   high and columns become bins shaded by the share of written cells, as in the overview.
//!   The window of cells is centred on what is drawn.
//! * [`SpaceTimeMode::Overview`] shows a whole run. A first pass runs the machine until it
//!   halts or reaches the step budget, which gives the number of steps T and every position
//!   the head reaches. A second pass samples the configuration at evenly spaced steps from 0
//!   to T, one sample per row, the last row showing step T. The tape across the head's extent
//!   is divided into one bin per pixel column, shaded by the share of written cells in it.
//!   Each row also tints the blank cells the head visited since the previous sample. Short
//!   runs on narrow tapes are drawn with square cells like the exact mode.
//!
//! Both passes use the chain simulator of `turing-core`, which jumps over whole runs of
//! equal cells and stops exactly at any step, so sampling a run of millions of steps costs
//! little more than its macro steps. Runs into proved endless loops (a stay loop or a walk
//! into blank tape) are carried on for the full budget.
//!
//! In the index buffer and in [`SpaceTime::rgba`], a written cell is shaded by its symbol,
//! the highest symbol darkest, and a bin by the share of written cells in it.

use std::ops::{Range, RangeInclusive};
use std::sync::Arc;

use turing_core::{ChainSim, Engine, Machine, NonHaltProof, Outcome, RunTape, Symbol, TapeView};

use crate::palette::{Palette, Rgb};

/// Index values in the index buffer.
pub mod index {
    /// A blank cell or an empty bin.
    pub const BACKGROUND: u8 = 0;
    /// The lightest shade of ink; shades run up to [`INK_MAX`].
    pub const INK_MIN: u8 = 1;
    /// The darkest shade of ink.
    pub const INK_MAX: u8 = 64;
    /// The cells the head visited during a sampled row.
    pub const HEAD_RANGE: u8 = 254;
    /// The head.
    pub const HEAD: u8 = 255;
}

use index::{BACKGROUND, HEAD, HEAD_RANGE, INK_MAX, INK_MIN};

/// The largest cell size chosen automatically, in pixels.
pub const MAX_AUTO_CELL_PX: u32 = 8;

/// The fewest rows an automatically sized exact diagram of a run longer than the image
/// shows, if the image is high enough.
pub const MIN_AUTO_ROWS: usize = 32;

/// The largest image side in pixels.
pub const MAX_SIDE: u32 = 4096;

/// How strongly [`index::HEAD_RANGE`] is tinted with the accent over the background.
pub const HEAD_RANGE_TINT: f64 = 0.35;

/// How rows map to steps.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpaceTimeMode {
    /// Row r shows the configuration after step `first_step + r`. Cells are `cell_px` pixels
    /// wide, or chosen to fit the head's extent if `None`.
    ///
    /// Cells are square, so each row is `cell_px` pixel rows high; see the
    /// [module documentation](self).
    Exact {
        /// The step of the first row.
        first_step: u64,
        /// Pixel width of a cell.
        cell_px: Option<u32>,
    },
    /// The run until it halts or reaches `max_steps` is sampled at evenly spaced steps, one
    /// sample per row, and columns are bins over the head's whole extent.
    Overview {
        /// The step budget.
        max_steps: u64,
    },
}

/// What to render.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SpaceTimeRequest {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Row mapping.
    pub mode: SpaceTimeMode,
}

/// Progress of an incremental rendering.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct SpaceTimeProgress {
    /// Rows finished.
    pub rows_done: u32,
    /// Machine steps simulated so far.
    ///
    /// This is the furthest step of the run reached so far. In the overview it is the length
    /// T of the run once the first pass is complete; the second pass does not count again.
    pub steps: u64,
    /// Whether the image is complete.
    pub done: bool,
    /// Whether the machine halted within the rendered range.
    pub halted: bool,
}

/// The tape cell (or bin) and step under a pixel.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CellRef {
    /// The step of the row.
    pub step: u64,
    /// The first tape position of the column.
    pub from: i64,
    /// The last tape position of the column.
    pub to: i64,
    /// The symbol, if the column is a single cell.
    pub symbol: Option<Symbol>,
}

/// An incrementally rendered space-time diagram of a single-tape machine.
///
/// ```
/// use turing_core::{Dir, Machine};
/// use turing_render::{Palette, SpaceTime, SpaceTimeMode, SpaceTimeRequest};
///
/// // Writes 1 and moves right forever: the head draws a diagonal.
/// let m = Machine::builder("walk").transition("q", '␣', "q", '1', Dir::R).build().unwrap();
/// let request = SpaceTimeRequest {
///     width: 16,
///     height: 16,
///     mode: SpaceTimeMode::Exact { first_step: 0, cell_px: Some(1) },
/// };
/// let mut st = SpaceTime::new(&m, &[], request).unwrap();
/// while !st.advance(5).done {}
/// assert_eq!(st.progress().rows_done, 16);
/// assert_eq!(st.rgba(&Palette::LIGHT).len(), 16 * 16 * 4);
/// ```
#[derive(Debug)]
pub struct SpaceTime {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) index: Vec<u8>,
    pub(crate) progress: SpaceTimeProgress,
    /// What was asked for.
    mode: SpaceTimeMode,
    /// |Γ|, for shading symbols.
    symbols: usize,
    /// The machine on its input, before the first step.
    start: Chain,
    /// The simulator of the current pass.
    sim: Chain,
    /// Where the rendering stands.
    phase: Phase,
    /// How rows and columns map to steps and cells, once known.
    grid: Option<Grid>,
    /// When columns are single cells, the symbol of every drawn cell, row by row, for
    /// [`SpaceTime::locate`].
    cells: Vec<u8>,
    /// The written part of the input.
    input_extent: Option<(i64, i64)>,
}

/// Rendering needs a single-tape machine and a non-empty image.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SpaceTimeError {
    /// More than one tape.
    #[error("space-time diagrams show single-tape machines only")]
    MultiTape,
    /// Zero width or height, or more than 4096 × 4096 pixels.
    #[error("the image must be 1 to 4096 pixels wide and high")]
    Size,
    /// The input contains a symbol outside the machine's tape alphabet.
    #[error("the input contains a symbol outside the tape alphabet")]
    Input,
}

/// The simulator used for both passes.
type Chain = ChainSim<Arc<Machine>>;

/// Where an incremental rendering stands.
#[derive(Clone, Debug)]
enum Phase {
    /// Exact mode: running to the first row.
    Seek,
    /// Exact mode: recording the head position of every row that may be drawn.
    MeasureRows {
        /// The simulator at the first row, kept for drawing.
        first: Box<Chain>,
        /// The written cells at the first row.
        written: Option<(i64, i64)>,
        /// The head position of each row so far.
        heads: Vec<i64>,
        /// Whether the machine halted at the last row.
        halted: bool,
    },
    /// Overview: running to the end to learn the length and the extent of the run.
    MeasureRun,
    /// Drawing row `next`.
    Draw {
        /// The next row to draw.
        next: u32,
        /// The cells the head visited since the previous row.
        swept: Option<(i64, i64)>,
    },
    /// Finished.
    Done,
}

/// How pixels map to steps and tape cells.
#[derive(Clone, Debug)]
struct Grid {
    columns: Columns,
    /// Pixel rows per row.
    row_px: u32,
    /// The number of rows to draw.
    rows: u32,
    /// The step of row 0.
    first: u64,
    /// Steps between rows.
    stride: u64,
    /// The last step shown; later rows are clamped to it.
    last: u64,
    /// Whether rows tint the cells visited since the previous row.
    sweeps: bool,
}

impl Grid {
    /// The step shown in row `row`.
    fn step(&self, row: u32) -> u64 {
        self.first
            .saturating_add(self.stride.saturating_mul(u64::from(row)))
            .min(self.last)
    }
}

/// How pixel columns map to tape cells.
#[derive(Clone, Debug)]
enum Columns {
    /// One cell per `px` pixel columns: `count` cells from position `first`, starting at
    /// pixel column `x0`.
    Cells {
        first: i64,
        count: u32,
        px: u32,
        x0: u32,
    },
    /// One bin per pixel column over `cells` cells from position `first`, more cells than
    /// pixel columns. Taking cell k as the interval from k to k + 1, bin x of `width` covers
    /// the interval from `x * cells / width` to `(x + 1) * cells / width`, so bins are equally
    /// wide and cells on their borders count in part. Bins of a whole number of cells would
    /// alternate in size and show a periodic tape as stripes.
    Bins { first: i64, cells: u64, width: u32 },
}

impl Columns {
    /// Single cells, `px` wide, in a window of the whole width centred on `around`.
    fn cells(width: u32, px: u32, around: (i64, i64)) -> Self {
        let count = width / px;
        let used = span(around);
        let first = if used <= u64::from(count) {
            let pad = (u64::from(count) - used) / 2;
            around.0.saturating_sub(signed(pad))
        } else {
            around.0
        };
        Self::Cells {
            first,
            count,
            px,
            x0: (width - count * px) / 2,
        }
    }

    /// One bin per pixel column over the cells `extent`, which has more cells than `width`.
    fn bins(width: u32, extent: (i64, i64)) -> Self {
        Self::Bins {
            first: extent.0,
            cells: span(extent),
            width,
        }
    }

    /// The tape positions covered.
    fn range(&self) -> RangeInclusive<i64> {
        match self {
            Self::Cells { first, count, .. } => {
                *first..=first.saturating_add(i64::from(*count) - 1)
            }
            Self::Bins { first, cells, .. } => *first..=first.saturating_add(signed(*cells) - 1),
        }
    }
}

/// The number of cells in an inclusive range.
fn span(range: (i64, i64)) -> u64 {
    range.1.abs_diff(range.0).saturating_add(1)
}

/// A step count or length as a signed distance, saturating.
fn signed(n: u64) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

/// The smallest range containing `b` and `a`, if there is an `a`.
fn union(a: Option<(i64, i64)>, b: (i64, i64)) -> (i64, i64) {
    a.map_or(b, |a| (a.0.min(b.0), a.1.max(b.1)))
}

/// Widens `range` to include `a` and `b`.
fn widen(range: &mut Option<(i64, i64)>, a: i64, b: i64) {
    *range = Some(union(*range, (a.min(b), a.max(b))));
}

/// Runs `sim` for at most `fuel` steps, on through proved endless loops. If `swept` is
/// given, it is widened by the head position after every step. Returns whether the machine
/// has halted.
fn run(sim: &mut Chain, fuel: u64, swept: Option<&mut Option<(i64, i64)>>) -> bool {
    let Some(swept) = swept else {
        return Engine::advance(sim, fuel).is_halted();
    };
    let before = sim.steps();
    // A macro step moves the head monotonically from the cell after its first step to the
    // cell after its last, so these two bound everything it visits.
    let mut head = sim.tape().head();
    let outcome = sim.run_until(fuel, |event| {
        let delta = event.action.dir[0].delta();
        let end = head.saturating_add(signed(event.repeats).saturating_mul(delta));
        widen(swept, head.saturating_add(delta), end);
        head = end;
        false
    });
    let Outcome::NeverHalts { proof, .. } = outcome else {
        return outcome.is_halted();
    };
    let rest = fuel - (sim.steps() - before);
    if rest == 0 {
        return false;
    }
    let delta = match proof {
        NonHaltProof::ChainToInfinity { dir, .. } => dir.delta(),
        NonHaltProof::StayLoop { .. } => 0,
    };
    let at = sim.tape().head();
    let halted = Engine::advance(sim, rest).is_halted();
    widen(swept, at.saturating_add(delta), sim.tape().head());
    halted
}

/// Calls `f(lo, hi, symbol)` for the head cell and every run of the tape, on each side of
/// the head from the head outwards.
fn for_each_run(tape: &RunTape, mut f: impl FnMut(i64, i64, Symbol)) {
    let head = tape.head();
    let mut near = head;
    for run in tape.left().iter().rev() {
        let far = near.saturating_sub(signed(run.len));
        f(far, near - 1, run.symbol);
        near = far;
    }
    f(head, head, tape.read());
    let mut near = head;
    for run in tape.right().iter().rev() {
        let far = near.saturating_add(signed(run.len));
        f(near + 1, far, run.symbol);
        near = far;
    }
}

impl SpaceTime {
    /// Prepares a diagram of `machine` started on `input`. Nothing is simulated yet.
    ///
    /// # Errors
    ///
    /// See [`SpaceTimeError`].
    pub fn new(
        machine: &Machine,
        input: &[Symbol],
        request: SpaceTimeRequest,
    ) -> Result<Self, SpaceTimeError> {
        let SpaceTimeRequest {
            width,
            height,
            mode,
        } = request;
        if !(1..=MAX_SIDE).contains(&width) || !(1..=MAX_SIDE).contains(&height) {
            return Err(SpaceTimeError::Size);
        }
        if machine.tapes() != 1 {
            return Err(SpaceTimeError::MultiTape);
        }
        if input.iter().any(|s| s.index() >= machine.num_symbols()) {
            return Err(SpaceTimeError::Input);
        }
        let start = ChainSim::new(Arc::new(machine.clone()), input)
            .map_err(|_| SpaceTimeError::MultiTape)?;
        let input_extent = start.tape().non_blank().map(|r| (*r.start(), *r.end()));
        let phase = match mode {
            SpaceTimeMode::Exact { .. } => Phase::Seek,
            SpaceTimeMode::Overview { .. } => Phase::MeasureRun,
        };
        Ok(Self {
            width,
            height,
            index: vec![BACKGROUND; width as usize * height as usize],
            progress: SpaceTimeProgress::default(),
            mode,
            symbols: machine.num_symbols(),
            sim: start.clone(),
            start,
            phase,
            grid: None,
            cells: Vec::new(),
            input_extent,
        })
    }

    /// Simulates at most `max_steps` more machine steps (macro steps count by their length)
    /// and fills finished rows.
    pub fn advance(&mut self, max_steps: u64) -> SpaceTimeProgress {
        let mut budget = max_steps;
        loop {
            let progressed = match self.phase {
                Phase::Seek => self.seek(&mut budget),
                Phase::MeasureRows { .. } => self.measure_rows(&mut budget),
                Phase::MeasureRun => self.measure_run(&mut budget),
                Phase::Draw { .. } => self.draw(&mut budget),
                Phase::Done => false,
            };
            if !progressed {
                break;
            }
        }
        self.progress.done = matches!(self.phase, Phase::Done);
        self.progress
    }

    /// Image width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Image height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The index buffer, row-major, one byte per pixel; see [`index`].
    pub fn index_buffer(&self) -> &[u8] {
        &self.index
    }

    /// The image as RGBA bytes.
    ///
    /// Background pixels take the background colour, ink shades are mixed from the
    /// background towards the ink ([`index::INK_MAX`] is the ink itself, lower shades are
    /// lighter but never the background), the head takes the accent and the visited range
    /// [`HEAD_RANGE_TINT`] of the accent over the background.
    pub fn rgba(&self, palette: &Palette) -> Vec<u8> {
        let lut = colour_table(palette);
        let mut out = Vec::with_capacity(self.index.len() * 4);
        for &i in &self.index {
            let Rgb(r, g, b) = lut[usize::from(i)];
            out.extend_from_slice(&[r, g, b, 255]);
        }
        out
    }

    /// The cell and step under pixel (x, y), if inside the rendered part.
    pub fn locate(&self, x: u32, y: u32) -> Option<CellRef> {
        let grid = self.grid.as_ref()?;
        let row = y / grid.row_px;
        if row >= self.progress.rows_done || x >= self.width {
            return None;
        }
        let step = grid.step(row);
        match &grid.columns {
            Columns::Cells {
                first,
                count,
                px,
                x0,
            } => {
                let k = x.checked_sub(*x0)? / px;
                if k >= *count {
                    return None;
                }
                let pos = first.saturating_add(i64::from(k));
                let symbol = self
                    .cells
                    .get(row as usize * *count as usize + k as usize)
                    .map(|&s| Symbol::new(s));
                Some(CellRef {
                    step,
                    from: pos,
                    to: pos,
                    symbol,
                })
            }
            Columns::Bins {
                first,
                cells,
                width,
            } => {
                let bins = Bins::new(*cells, *width);
                let (lo, hi) = bins.cells_of(x);
                Some(CellRef {
                    step,
                    from: first.saturating_add(signed(lo)),
                    to: first.saturating_add(signed(hi)),
                    symbol: None,
                })
            }
        }
    }

    /// The progress so far.
    pub fn progress(&self) -> SpaceTimeProgress {
        self.progress
    }

    /// The tape positions shown from the left to the right edge, once the first pass has
    /// fixed them.
    pub fn tape_range(&self) -> Option<RangeInclusive<i64>> {
        Some(self.grid.as_ref()?.columns.range())
    }

    /// The step shown in row `row` and the pixel rows it covers, once the first pass has
    /// fixed them; `None` for rows that will not be drawn.
    pub fn row(&self, row: u32) -> Option<(u64, Range<u32>)> {
        let grid = self.grid.as_ref()?;
        (row < grid.rows).then(|| (grid.step(row), row * grid.row_px..(row + 1) * grid.row_px))
    }

    /// Notes the furthest step reached.
    fn reached(&mut self) {
        self.progress.steps = self.progress.steps.max(self.sim.steps());
    }

    /// Exact mode: runs to the first row.
    fn seek(&mut self, budget: &mut u64) -> bool {
        let SpaceTimeMode::Exact { first_step, .. } = self.mode else {
            unreachable!("only the exact mode seeks");
        };
        let before = self.sim.steps();
        if before < first_step {
            let halted = run(&mut self.sim, (*budget).min(first_step - before), None);
            *budget -= self.sim.steps() - before;
            self.reached();
            if self.sim.steps() < first_step {
                if halted {
                    // The machine halted before the first row: there is nothing to draw.
                    self.progress.halted = true;
                    self.phase = Phase::Done;
                    return true;
                }
                return self.sim.steps() > before;
            }
        }
        let written = self.sim.tape().non_blank().map(|r| (*r.start(), *r.end()));
        self.phase = Phase::MeasureRows {
            heads: vec![self.sim.tape().head()],
            first: Box::new(self.sim.clone()),
            written,
            halted: false,
        };
        true
    }

    /// The cell size of the exact mode if it was given, clamped to the image.
    fn given_cell_px(&self) -> Option<u32> {
        match self.mode {
            SpaceTimeMode::Exact {
                cell_px: Some(px), ..
            } => Some(px.clamp(1, self.width.min(self.height))),
            _ => None,
        }
    }

    /// Exact mode: records the head of every row that may be drawn, then fixes the grid.
    fn measure_rows(&mut self, budget: &mut u64) -> bool {
        let limit = (self.height / self.given_cell_px().unwrap_or(1)) as usize;
        let Phase::MeasureRows { heads, halted, .. } = &mut self.phase else {
            unreachable!("called in the measuring phase");
        };
        let mut moved = false;
        while heads.len() < limit && !*halted && *budget > 0 {
            let before = self.sim.steps();
            // Reports a halt after the step as well, so the last row is known at once.
            *halted = run(&mut self.sim, 1, None);
            if self.sim.steps() > before {
                *budget -= 1;
                heads.push(self.sim.tape().head());
                moved = true;
            } else {
                *halted = true;
            }
        }
        if heads.len() < limit && !*halted {
            self.reached();
            return moved;
        }
        let Phase::MeasureRows {
            first,
            written,
            heads,
            halted,
        } = std::mem::replace(&mut self.phase, Phase::Done)
        else {
            unreachable!("checked above");
        };
        self.progress.halted = halted;
        self.reached();
        self.sim = *first;
        self.grid = Some(self.exact_grid(&heads, written, halted));
        self.start_drawing();
        true
    }

    /// The grid of the exact mode for the head positions of the rows that exist.
    fn exact_grid(&self, heads: &[i64], written: Option<(i64, i64)>, halted: bool) -> Grid {
        let SpaceTimeMode::Exact { first_step, .. } = self.mode else {
            unreachable!("only the exact mode measures rows");
        };
        // The cells the head visits in the first `rows` rows, and those and the written ones.
        let visited = |rows: usize| -> (i64, i64) {
            let heads = &heads[..rows.clamp(1, heads.len())];
            let lo = heads.iter().copied().min().unwrap_or(0);
            let hi = heads.iter().copied().max().unwrap_or(0);
            (lo, hi)
        };
        let extent = |rows: usize| union(written, visited(rows));
        let grid = |columns, row_px: u32| Grid {
            columns,
            row_px,
            rows: u32::try_from(heads.len())
                .unwrap_or(u32::MAX)
                .min(self.height / row_px),
            first: first_step,
            stride: 1,
            last: u64::MAX,
            sweeps: false,
        };
        if let Some(px) = self.given_cell_px() {
            let rows = (self.height / px) as usize;
            let all = extent(rows);
            let around = if span(all) <= u64::from(self.width / px) {
                all
            } else {
                visited(rows)
            };
            return grid(Columns::cells(self.width, px, around), px);
        }
        // A run that ends within the image is shown whole. A longer one is cut anyway, so it
        // gets the largest cells that still show a fair number of rows.
        let available = heads.len();
        let wanted = if halted {
            available
        } else {
            available.min(MIN_AUTO_ROWS)
        };
        for px in (1..=MAX_AUTO_CELL_PX.min(self.height)).rev() {
            let rows = (self.height / px) as usize;
            let all = extent(rows);
            if rows >= wanted && span(all) <= u64::from(self.width / px) {
                return grid(Columns::cells(self.width, px, all), px);
            }
        }
        grid(Columns::bins(self.width, extent(self.height as usize)), 1)
    }

    /// Overview: runs to the end, then fixes the grid.
    fn measure_run(&mut self, budget: &mut u64) -> bool {
        let SpaceTimeMode::Overview { max_steps } = self.mode else {
            unreachable!("only the overview measures the run");
        };
        let before = self.sim.steps();
        // With no fuel this still reports whether the machine has halted.
        let halted = run(&mut self.sim, (*budget).min(max_steps - before), None);
        *budget -= self.sim.steps() - before;
        self.reached();
        if !halted && self.sim.steps() < max_steps {
            return self.sim.steps() > before;
        }
        let total = self.sim.steps();
        let reach = self.sim.extent();
        let extent = union(self.input_extent, (*reach.start(), *reach.end()));
        self.progress.halted = halted;
        self.grid = Some(self.overview_grid(total, extent));
        self.sim = self.start.clone();
        self.start_drawing();
        true
    }

    /// The grid of the overview of a run of `total` steps over the cells `extent`.
    fn overview_grid(&self, total: u64, extent: (i64, i64)) -> Grid {
        let samples = u64::from(self.height);
        let (stride, rows) = if samples < 2 {
            (total.max(1), 1)
        } else {
            let stride = total.div_ceil(samples - 1).max(1);
            let rows = u32::try_from(total.div_ceil(stride) + 1).unwrap_or(self.height);
            (stride, rows)
        };
        let (columns, row_px) = if span(extent) <= u64::from(self.width) {
            let cells = u32::try_from(span(extent)).unwrap_or(self.width);
            let px = (self.width / cells).clamp(1, MAX_AUTO_CELL_PX);
            let row_px = (self.height / rows).clamp(1, px);
            (Columns::cells(self.width, px, extent), row_px)
        } else {
            (Columns::bins(self.width, extent), 1)
        };
        Grid {
            columns,
            row_px,
            rows,
            first: 0,
            stride,
            last: total,
            sweeps: true,
        }
    }

    /// Enters the drawing phase at row 0 with the simulator at or before the first row.
    fn start_drawing(&mut self) {
        if let Some(grid) = &self.grid
            && let Columns::Cells { count, .. } = grid.columns
        {
            self.cells = vec![0; grid.rows as usize * count as usize];
        }
        self.phase = Phase::Draw {
            next: 0,
            swept: None,
        };
    }

    /// Draws rows as far as the budget reaches.
    fn draw(&mut self, budget: &mut u64) -> bool {
        let Some(grid) = self.grid.take() else {
            unreachable!("the grid is fixed before drawing");
        };
        let Phase::Draw {
            mut next,
            mut swept,
        } = self.phase
        else {
            unreachable!("called in the drawing phase");
        };
        let mut moved = false;
        let mut stuck = false;
        while next < grid.rows {
            let target = grid.step(next);
            let now = self.sim.steps();
            if now < target {
                if *budget == 0 {
                    break;
                }
                let track = grid.sweeps.then_some(&mut swept);
                let halted = run(&mut self.sim, (*budget).min(target - now), track);
                *budget -= self.sim.steps() - now;
                moved |= self.sim.steps() > now;
                if self.sim.steps() < target {
                    // A halt before the target cannot happen, as the first pass got there;
                    // stop rather than wait for it.
                    stuck = halted;
                    if stuck {
                        break;
                    }
                    continue;
                }
            }
            self.render_row(&grid, next, swept.take());
            next += 1;
            self.progress.rows_done = next;
            moved = true;
        }
        let finished = next >= grid.rows || stuck;
        self.grid = Some(grid);
        self.phase = if finished {
            Phase::Done
        } else {
            Phase::Draw { next, swept }
        };
        moved || finished
    }

    /// Fills row `row` of the image from the current configuration.
    fn render_row(&mut self, grid: &Grid, row: u32, swept: Option<(i64, i64)>) {
        let width = self.width as usize;
        let y0 = (row * grid.row_px) as usize;
        let y1 = (((row + 1) * grid.row_px) as usize).min(self.height as usize);
        let values = match &grid.columns {
            Columns::Cells { first, count, .. } => self.cell_values(row, *first, *count, swept),
            Columns::Bins {
                first,
                cells,
                width,
            } => bin_values(self.sim.tape(), *first, Bins::new(*cells, *width), swept),
        };
        for y in y0..y1 {
            let line = &mut self.index[y * width..(y + 1) * width];
            match &grid.columns {
                Columns::Cells { px, x0, .. } => {
                    let (px, x0) = (*px as usize, *x0 as usize);
                    for (k, &v) in values.iter().enumerate() {
                        line[x0 + k * px..x0 + (k + 1) * px].fill(v);
                    }
                }
                Columns::Bins { .. } => line.copy_from_slice(&values),
            }
        }
    }

    /// The index of each of `count` single cells from `first` in row `row`, which are also
    /// recorded for [`SpaceTime::locate`].
    fn cell_values(
        &mut self,
        row: u32,
        first: i64,
        count: u32,
        swept: Option<(i64, i64)>,
    ) -> Vec<u8> {
        let tape = self.sim.tape();
        let symbols = tape.window(first, count as usize);
        let mut values: Vec<u8> = symbols
            .iter()
            .map(|&s| symbol_shade(self.symbols, s))
            .collect();
        let row_start = row as usize * count as usize;
        if let Some(slots) = self.cells.get_mut(row_start..row_start + symbols.len()) {
            for (slot, s) in slots.iter_mut().zip(&symbols) {
                *slot = s.get();
            }
        }
        // The column of a position, possibly outside the window.
        let column = |pos: i64| pos.saturating_sub(first);
        if let Some((lo, hi)) = swept {
            let (a, b) = (column(lo).max(0), column(hi).min(i64::from(count) - 1));
            if let (Ok(a), Ok(b)) = (usize::try_from(a), usize::try_from(b)) {
                mark_swept(values.get_mut(a..=b).unwrap_or_default());
            }
        }
        if let Ok(k) = usize::try_from(column(tape.head()))
            && let Some(v) = values.get_mut(k)
        {
            *v = HEAD;
        }
        values
    }
}

/// Bins of equal width over a number of cells; see [`Columns::Bins`].
///
/// Positions are measured in units of 1 / `width` cell, so that every bin border is a whole
/// number: bin x runs from `x * cells` to `(x + 1) * cells`.
#[derive(Copy, Clone, Debug)]
struct Bins {
    cells: u128,
    width: u128,
}

impl Bins {
    fn new(cells: u64, width: u32) -> Self {
        Self {
            cells: u128::from(cells),
            width: u128::from(width),
        }
    }

    /// The bin that contains the middle of the cell at offset `k`.
    fn of(self, k: u64) -> usize {
        let middle = (2 * u128::from(k) + 1) * self.width;
        usize::try_from(middle / (2 * self.cells)).unwrap_or(usize::MAX)
    }

    /// The first and last cell offsets that bin `x` covers, at least in part.
    fn cells_of(self, x: u32) -> (u64, u64) {
        let x = u128::from(x);
        let lo = x * self.cells / self.width;
        let hi = ((x + 1) * self.cells).div_ceil(self.width) - 1;
        (
            u64::try_from(lo).unwrap_or(u64::MAX),
            u64::try_from(hi).unwrap_or(u64::MAX),
        )
    }

    /// The bins that the cells at offsets `lo` to `hi` overlap.
    fn overlapping(self, lo: u64, hi: u64) -> (usize, usize) {
        let first = u128::from(lo) * self.width / self.cells;
        let last = ((u128::from(hi) + 1) * self.width - 1) / self.cells;
        (
            usize::try_from(first).unwrap_or(usize::MAX),
            usize::try_from(last).unwrap_or(usize::MAX),
        )
    }
}

/// The index of each bin of a row: the share of written cells, the visited range and the
/// head.
fn bin_values(tape: &RunTape, first: i64, bins: Bins, swept: Option<(i64, i64)>) -> Vec<u8> {
    let count = usize::try_from(bins.width).unwrap_or(0);
    let cells = u64::try_from(bins.cells).unwrap_or(u64::MAX);
    // Written length per bin, in units of 1 / width cell.
    let mut written = vec![0u128; count];
    let mut cursor = 0usize;
    for_each_run(tape, |lo, hi, symbol| {
        if symbol.is_blank() {
            return;
        }
        let lo = u64::try_from(lo.saturating_sub(first)).unwrap_or(0);
        let Ok(hi) = u64::try_from(hi.saturating_sub(first)) else {
            return;
        };
        if lo < cells && lo <= hi {
            add_to_bins(bins, &mut written, &mut cursor, lo, hi.min(cells - 1));
        }
    });
    let mut values: Vec<u8> = written
        .iter()
        .map(|&w| {
            if w == 0 {
                BACKGROUND
            } else {
                #[expect(clippy::cast_precision_loss, reason = "a share")]
                let share = w as f64 / bins.cells as f64;
                ink_shade(share)
            }
        })
        .collect();
    let offset = |pos: i64| {
        u64::try_from(pos.checked_sub(first)?)
            .ok()
            .filter(|&k| k < cells)
    };
    let last = first.saturating_add(signed(cells) - 1);
    if let Some((lo, hi)) = swept
        && let (Some(lo), Some(hi)) = (offset(lo.max(first)), offset(hi.min(last)))
        && lo <= hi
    {
        let (a, b) = bins.overlapping(lo, hi);
        mark_swept(values.get_mut(a..=b.min(count - 1)).unwrap_or_default());
    }
    if let Some(k) = offset(tape.head())
        && let Some(v) = values.get_mut(bins.of(k))
    {
        *v = HEAD;
    }
    values
}

/// Tints the blank cells among `values` as visited.
fn mark_swept(values: &mut [u8]) {
    for v in values {
        if *v == BACKGROUND {
            *v = HEAD_RANGE;
        }
    }
}

/// Adds the cells at offsets `lo` to `hi` to the bins they overlap, in parts. `cursor` is a
/// bin of the previous call; runs arrive in order of position on each side of the head, so
/// moving it costs constant time per run on average and no division is needed.
fn add_to_bins(bins: Bins, written: &mut [u128], cursor: &mut usize, lo: u64, hi: u64) {
    // The run as an interval in units of 1 / width cell.
    let start = u128::from(lo) * bins.width;
    let end = (u128::from(hi) + 1) * bins.width;
    let border = |x: usize| x as u128 * bins.cells;
    while *cursor > 0 && border(*cursor) > start {
        *cursor -= 1;
    }
    while *cursor + 1 < written.len() && border(*cursor + 1) <= start {
        *cursor += 1;
    }
    let mut x = *cursor;
    while x < written.len() && border(x) < end {
        let overlap = end.min(border(x + 1)) - start.max(border(x));
        written[x] += overlap;
        x += 1;
    }
}

/// The ink shade of `symbol` in an alphabet of `symbols` symbols: the highest symbol is the
/// darkest.
fn symbol_shade(symbols: usize, symbol: Symbol) -> u8 {
    if symbol.is_blank() {
        return BACKGROUND;
    }
    let top = symbols.saturating_sub(1).max(1);
    #[expect(clippy::cast_precision_loss, reason = "at most 64 symbols")]
    let share = symbol.index().min(top) as f64 / top as f64;
    ink_shade(share)
}

/// The ink index for a share of 0 to 1 of the darkest shade.
fn ink_shade(share: f64) -> u8 {
    let range = f64::from(INK_MAX - INK_MIN);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "clamped to the ink range"
    )]
    let index = (f64::from(INK_MIN) + (share.clamp(0.0, 1.0) * range).round()) as u8;
    index
}

/// The colour of every index.
fn colour_table(palette: &Palette) -> [Rgb; 256] {
    let mut lut = [palette.ink; 256];
    lut[usize::from(BACKGROUND)] = palette.background;
    for i in INK_MIN..=INK_MAX {
        lut[usize::from(i)] = palette
            .background
            .mix(palette.ink, f64::from(i) / f64::from(INK_MAX));
    }
    lut[usize::from(HEAD_RANGE)] = palette.background.mix(palette.accent, HEAD_RANGE_TINT);
    lut[usize::from(HEAD)] = palette.accent;
    lut
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn shades_span_the_ink_range() {
        assert_eq!(ink_shade(0.0), INK_MIN);
        assert_eq!(ink_shade(1.0), INK_MAX);
        assert_eq!(ink_shade(7.0), INK_MAX);
        assert_eq!(symbol_shade(2, Symbol::new(1)), INK_MAX);
        assert_eq!(symbol_shade(3, Symbol::new(2)), INK_MAX);
        assert_eq!(symbol_shade(3, Symbol::BLANK), BACKGROUND);
        assert!(symbol_shade(3, Symbol::new(1)) < INK_MAX);
    }

    #[test]
    fn colours_mix_from_the_background() {
        let palette = Palette {
            background: Rgb(250, 250, 250),
            ink: Rgb(10, 10, 10),
            accent: Rgb(0, 100, 200),
        };
        let lut = colour_table(&palette);
        assert_eq!(lut[usize::from(BACKGROUND)], palette.background);
        assert_eq!(lut[usize::from(INK_MAX)], palette.ink);
        assert_ne!(lut[usize::from(INK_MIN)], palette.background);
        assert!(lut[usize::from(INK_MIN)].0 > lut[usize::from(INK_MIN + 1)].0);
        assert_eq!(lut[usize::from(HEAD)], palette.accent);
        assert_eq!(lut[usize::from(HEAD_RANGE)], Rgb(163, 198, 233));
    }

    #[test]
    fn bins_cover_cells_evenly() {
        let bins = Bins::new(19, 10);
        assert_eq!(bins.cells_of(0), (0, 1));
        assert_eq!(bins.cells_of(1), (1, 3));
        assert_eq!(bins.cells_of(9), (17, 18));
        assert_eq!(bins.of(0), 0);
        assert_eq!(bins.of(9), 5);
        assert_eq!(bins.of(18), 9);
        assert_eq!(bins.overlapping(1, 8), (0, 4));
    }

    proptest! {
        #[test]
        fn binned_runs_keep_their_length(
            width in 1u32..300,
            extra in 1u64..10_000,
            runs in prop::collection::vec((0u64..20_000, 0u64..500), 1..20),
        ) {
            let cells = u64::from(width) + extra;
            let bins = Bins::new(cells, width);
            let mut written = vec![0u128; width as usize];
            let mut cursor = 0;
            let mut total = 0u128;
            for (lo, len) in runs {
                let lo = lo % cells;
                let hi = (lo + len).min(cells - 1);
                add_to_bins(bins, &mut written, &mut cursor, lo, hi);
                total += u128::from(hi - lo + 1) * u128::from(width);
                let (a, b) = bins.overlapping(lo, hi);
                prop_assert!(a <= b && b < width as usize);
                prop_assert!(bins.cells_of(u32::try_from(a).unwrap()).0 <= lo);
                prop_assert!(bins.cells_of(u32::try_from(b).unwrap()).1 >= hi);
            }
            prop_assert_eq!(written.iter().sum::<u128>(), total);
        }
    }
}
