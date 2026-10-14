//! Space-time diagrams.

use turing_core::{Machine, Symbol};

use crate::palette::Palette;

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

/// How rows map to steps.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpaceTimeMode {
    /// Row r shows the configuration after step `first_step + r`. Cells are `cell_px` pixels
    /// wide, or chosen to fit the head's extent if `None`.
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
#[derive(Debug)]
pub struct SpaceTime {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) index: Vec<u8>,
    pub(crate) progress: SpaceTimeProgress,
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
        let _ = (machine, input, request);
        todo!("A4")
    }

    /// Simulates at most `max_steps` more machine steps (macro steps count by their length)
    /// and fills finished rows.
    pub fn advance(&mut self, max_steps: u64) -> SpaceTimeProgress {
        let _ = max_steps;
        todo!("A4")
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
    pub fn rgba(&self, palette: &Palette) -> Vec<u8> {
        let _ = palette;
        todo!("A4")
    }

    /// The cell and step under pixel (x, y), if inside the rendered part.
    pub fn locate(&self, x: u32, y: u32) -> Option<CellRef> {
        let _ = (x, y);
        todo!("A4")
    }

    /// The progress so far.
    pub fn progress(&self) -> SpaceTimeProgress {
        self.progress
    }
}
