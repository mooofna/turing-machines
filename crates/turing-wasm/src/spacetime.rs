//! Space-time diagrams, rendered incrementally in the worker.

use std::fmt;

use turing_core::Alphabet;
use turing_render::{Palette, Rgb, SpaceTime, SpaceTimeMode, SpaceTimeRequest};

use crate::Error;
use crate::convert::{self, number};
use crate::dto::{
    CellRefDto, MachineSpecDto, PaletteDto, SpaceTimeModeDto, SpaceTimeProgressDto,
    SpaceTimeRequestDto,
};

/// A diagram being rendered, with the alphabet to name the cells under the pointer.
pub(crate) struct Job {
    diagram: SpaceTime,
    alphabet: Alphabet,
}

impl fmt::Debug for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpaceTimeJob")
            .field("width", &self.diagram.width())
            .field("height", &self.diagram.height())
            .field("progress", &self.diagram.progress())
            .finish_non_exhaustive()
    }
}

/// The request of a DTO. Step numbers beyond 2^53 - 1 are capped there.
pub(crate) fn request(dto: &SpaceTimeRequestDto) -> SpaceTimeRequest {
    SpaceTimeRequest {
        width: dto.width,
        height: dto.height,
        mode: match dto.mode {
            SpaceTimeModeDto::Exact {
                first_step,
                cell_px,
            } => SpaceTimeMode::Exact {
                first_step: convert::count(first_step),
                cell_px,
            },
            SpaceTimeModeDto::Overview { max_steps } => SpaceTimeMode::Overview {
                max_steps: convert::count(max_steps),
            },
        },
    }
}

/// The palette of a DTO.
///
/// # Errors
///
/// Names the first colour that is not `#rgb` or `#rrggbb`.
pub(crate) fn palette(dto: &PaletteDto) -> Result<Palette, Error> {
    let parse = |name: &str, text: &str| {
        Rgb::from_hex(text).ok_or_else(|| {
            Error::new(format!(
                "the {name} colour `{}` is not of the form #rgb or #rrggbb",
                convert::shorten(text, 16)
            ))
        })
    };
    Ok(Palette {
        background: parse("background", &dto.background)?,
        ink: parse("ink", &dto.ink)?,
        accent: parse("accent", &dto.accent)?,
    })
}

impl Job {
    /// Prepares a diagram of `spec` on `input`.
    ///
    /// # Errors
    ///
    /// Fails for specs with errors, inputs outside Σ, several tapes and invalid sizes.
    pub(crate) fn new(
        spec: &MachineSpecDto,
        input: &str,
        request: &SpaceTimeRequestDto,
    ) -> Result<Self, Error> {
        let spec = convert::spec(spec).map_err(|p| convert::first_error(&p))?;
        let machine = spec.compile().map_err(|d| convert::first_diagnostic(&d))?;
        let word = machine
            .encode_input(input)
            .map_err(|e| Error::new(e.to_string()))?;
        let diagram = SpaceTime::new(&machine, &word, self::request(request))
            .map_err(|e| Error::new(e.to_string()))?;
        Ok(Self {
            diagram,
            alphabet: machine.alphabet().clone(),
        })
    }

    /// Simulates at most `max_steps` more steps.
    pub(crate) fn advance(&mut self, max_steps: u64) -> SpaceTimeProgressDto {
        let p = self.diagram.advance(max_steps);
        SpaceTimeProgressDto {
            rows_done: p.rows_done,
            steps: number(p.steps),
            done: p.done,
            halted: p.halted,
        }
    }

    /// The image in `palette`.
    pub(crate) fn rgba(&self, palette: &Palette) -> Vec<u8> {
        self.diagram.rgba(palette)
    }

    /// The cell under pixel (`x`, `y`).
    pub(crate) fn locate(&self, x: u32, y: u32) -> Option<CellRefDto> {
        if x >= self.diagram.width() || y >= self.diagram.height() {
            return None;
        }
        self.diagram.locate(x, y).map(|cell| CellRefDto {
            step: number(cell.step),
            from: convert::signed(cell.from),
            to: convert::signed(cell.to),
            glyph: cell.symbol.map(|s| self.alphabet.glyph(s).to_string()),
        })
    }

    pub(crate) fn width(&self) -> u32 {
        self.diagram.width()
    }

    pub(crate) fn height(&self) -> u32 {
        self.diagram.height()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_cap_steps() {
        let dto = SpaceTimeRequestDto {
            width: 10,
            height: 20,
            mode: SpaceTimeModeDto::Overview {
                max_steps: f64::INFINITY,
            },
        };
        assert_eq!(
            request(&dto).mode,
            SpaceTimeMode::Overview {
                max_steps: convert::MAX_SAFE
            }
        );
        let dto = SpaceTimeRequestDto {
            width: 10,
            height: 20,
            mode: SpaceTimeModeDto::Exact {
                first_step: -3.0,
                cell_px: Some(4),
            },
        };
        assert_eq!(
            request(&dto).mode,
            SpaceTimeMode::Exact {
                first_step: 0,
                cell_px: Some(4)
            }
        );
    }
}

#[cfg(test)]
mod diagrams {
    use super::*;
    use crate::testing;

    fn palette_dto() -> PaletteDto {
        PaletteDto {
            background: "#ffffff".into(),
            ink: "#111".into(),
            accent: "#215caf".into(),
        }
    }

    #[test]
    fn bb4_overview_completes() {
        let request = SpaceTimeRequestDto {
            width: 120,
            height: 80,
            mode: SpaceTimeModeDto::Overview { max_steps: 1e6 },
        };
        let mut job = Job::new(&testing::standard(testing::BB4), "", &request).unwrap();
        let progress = loop {
            let p = job.advance(10);
            if p.done {
                break p;
            }
        };
        assert!(progress.halted);
        assert_eq!(progress.rows_done, 80);
        let palette = palette(&palette_dto()).unwrap();
        let pixels = job.rgba(&palette);
        assert_eq!(pixels.len(), 120 * 80 * 4);
        assert!(job.locate(0, 0).is_some());
        assert_eq!(job.locate(120, 0), None);
        assert_eq!(job.locate(u32::MAX, u32::MAX), None);
    }

    #[test]
    fn exact_rows_name_their_cells() {
        let request = SpaceTimeRequestDto {
            width: 64,
            height: 50,
            mode: SpaceTimeModeDto::Exact {
                first_step: 0.0,
                cell_px: Some(4),
            },
        };
        let mut job = Job::new(&testing::standard(testing::BB4), "", &request).unwrap();
        while !job.advance(1_000).done {}
        let cell = job.locate(32, 10).expect("inside the image");
        assert_eq!(cell.step, 10.0);
        assert_eq!(cell.from, cell.to);
        assert!(matches!(cell.glyph.as_deref(), Some("0" | "1")));
    }

    #[test]
    fn bad_requests_are_errors() {
        let spec = testing::standard(testing::BB4);
        let mut request = SpaceTimeRequestDto {
            width: 0,
            height: 10,
            mode: SpaceTimeModeDto::Overview { max_steps: 100.0 },
        };
        assert!(Job::new(&spec, "", &request).is_err());
        request.width = 100_000;
        assert!(Job::new(&spec, "", &request).is_err());
        request.width = 10;
        assert!(Job::new(&spec, "7", &request).is_err());
        let mut colours = palette_dto();
        colours.ink = "black".into();
        let error = palette(&colours).unwrap_err();
        assert_eq!(
            error.to_string(),
            "the ink colour `black` is not of the form #rgb or #rrggbb"
        );
    }
}
