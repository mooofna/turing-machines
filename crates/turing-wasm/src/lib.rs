#![doc = include_str!("../README.md")]
// wasm-bindgen exports are `pub` by necessity; the generated glue uses them.
#![allow(unreachable_pub, reason = "wasm-bindgen exports")]

pub mod dto;
mod stub;

use serde::Serialize;
use serde::de::DeserializeOwned;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

use crate::dto::{
    AntihydraRowsDto, BreakpointsDto, CellRefDto, DiagnosticsDto, EdgeRoutesDto, EnumProgressDto,
    GraphLayoutDto, MachineListDto, MachineSpecDto, PaletteDto, ParseResultDto, RunResultDto,
    SnapshotDto, SpaceTimeProgressDto, SpaceTimeRequestDto, TowerDto, VerdictDto, WalkSummaryDto,
};

/// A value that crosses the boundary as its TypeScript type.
type Out<T> = Result<Ts<T>, JsError>;

/// Serialises a DTO for JavaScript.
fn out<T: Tsify + Serialize>(value: &T) -> Out<T> {
    Ts::from_rust(value).map_err(|e| JsError::new(&e.to_string()))
}

/// Deserialises a DTO from JavaScript inside the function, so a malformed value becomes an
/// ordinary error instead of a leak at the ABI boundary.
fn input<T>(value: &Ts<T>) -> Result<T, JsError>
where
    T: Tsify + DeserializeOwned,
    <T as Tsify>::JsType: Clone,
{
    value.to_rust().map_err(|e| JsError::new(&e.to_string()))
}

/// Installs a panic hook that reports Rust panics on the browser console.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// The crate version, for cache diagnostics.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").into()
}

// Editor services (main thread, synchronous).

/// Parses a description in the course notation.
///
/// # Errors
///
/// Only on internal serialisation failures; problems in the source are diagnostics.
#[wasm_bindgen(js_name = parseSource)]
pub fn parse_source(source: &str) -> Out<ParseResultDto> {
    out(&stub::parse_source(source))
}

/// Writes a spec in canonical form; `ascii` replaces δ, Σ, Γ and ␣.
///
/// # Errors
///
/// Fails if `spec` is not a valid `MachineSpecDto`.
#[wasm_bindgen(js_name = formatSpec)]
pub fn format_spec(spec: Ts<MachineSpecDto>, ascii: bool) -> Result<String, JsError> {
    Ok(stub::format_spec(&input(&spec)?, ascii))
}

/// Checks a spec and returns every error and warning.
///
/// # Errors
///
/// Fails if `spec` is not a valid `MachineSpecDto`.
#[wasm_bindgen(js_name = checkSpec)]
pub fn check_spec(spec: Ts<MachineSpecDto>) -> Out<DiagnosticsDto> {
    let _ = input(&spec)?;
    out(&DiagnosticsDto {
        diagnostics: Vec::new(),
    })
}

/// Parses the bbchallenge standard format, for example `1RB1LB_1LA1RZ`.
///
/// # Errors
///
/// Only on internal serialisation failures; syntax errors are diagnostics.
#[wasm_bindgen(js_name = fromStandard)]
pub fn from_standard(text: &str) -> Out<ParseResultDto> {
    out(&stub::from_standard(text))
}

/// Writes a spec in the standard format, or `undefined` if it cannot be expressed there.
///
/// # Errors
///
/// Fails if `spec` is not a valid `MachineSpecDto`.
#[wasm_bindgen(js_name = toStandard)]
pub fn to_standard(spec: Ts<MachineSpecDto>) -> Result<Option<String>, JsError> {
    let _ = input(&spec)?;
    Ok(None)
}

/// Encodes a spec for a share link: `1.` followed by base64url of the deflated description.
///
/// # Errors
///
/// Fails if `spec` is not a valid `MachineSpecDto`.
#[wasm_bindgen(js_name = encodeShare)]
pub fn encode_share(spec: Ts<MachineSpecDto>) -> Result<String, JsError> {
    let _ = input(&spec)?;
    Ok("1.stub".into())
}

/// Decodes a share link payload produced by [`encode_share`].
///
/// # Errors
///
/// Only on internal serialisation failures; invalid payloads are diagnostics.
#[wasm_bindgen(js_name = decodeShare)]
pub fn decode_share(payload: &str) -> Out<ParseResultDto> {
    out(&stub::parse_source(payload))
}

/// Places the states of a spec. `kind` is `auto`, `layered` or `circular`.
///
/// # Errors
///
/// Fails if `spec` is not a valid `MachineSpecDto`.
#[wasm_bindgen(js_name = graphLayout)]
pub fn graph_layout(spec: Ts<MachineSpecDto>, kind: &str) -> Out<GraphLayoutDto> {
    out(&stub::graph_layout(&input(&spec)?, kind))
}

/// Routes the edges of a spec for node centres `xs` and `ys`.
///
/// # Errors
///
/// Fails if `spec` is not a valid `MachineSpecDto`.
#[wasm_bindgen(js_name = graphRoute)]
pub fn graph_route(spec: Ts<MachineSpecDto>, xs: &[f64], ys: &[f64]) -> Out<EdgeRoutesDto> {
    out(&stub::graph_route(&input(&spec)?, xs, ys))
}

/// Knuth's `base ↑↑ height`, exact up to `max_digits` digits.
///
/// # Errors
///
/// Only on internal serialisation failures.
#[wasm_bindgen(js_name = upArrow)]
pub fn up_arrow(base: u32, height: u32, max_digits: u32) -> Out<TowerDto> {
    let _ = max_digits;
    out(&TowerDto {
        base,
        height,
        exact: None,
        digits: None,
    })
}

/// The number of machines with `states` states and `symbols` symbols in Radó's count, as a
/// decimal string, or `undefined` if it exceeds 2^128.
#[wasm_bindgen(js_name = machineCount)]
pub fn machine_count(states: u32, symbols: u32) -> Option<String> {
    turing_bb::count::machines(states, symbols).map(|n| n.to_string())
}

/// Runs the decider pipeline on a single-tape machine started on the blank tape.
///
/// # Errors
///
/// Fails if `spec` is not a valid `MachineSpecDto`.
#[wasm_bindgen]
pub fn decide(spec: Ts<MachineSpecDto>) -> Out<VerdictDto> {
    let _ = input(&spec)?;
    out(&VerdictDto::Unknown)
}

// Engines (worker, advanced in bounded chunks).

/// A run of a machine that can step, seek in both directions and run with breakpoints.
#[wasm_bindgen]
pub struct Session {
    inner: stub::Session,
}

#[wasm_bindgen]
impl Session {
    /// Opens `spec` with `input` on the first tape.
    ///
    /// # Errors
    ///
    /// Fails if the spec has errors or the input contains symbols outside Σ.
    #[wasm_bindgen(constructor)]
    pub fn new(spec: Ts<MachineSpecDto>, input_word: &str) -> Result<Session, JsError> {
        Ok(Self {
            inner: stub::Session::new(input(&spec)?, input_word),
        })
    }

    /// The current configuration with `radius` cells on either side of each head.
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn snapshot(&self, radius: u32) -> Out<SnapshotDto> {
        out(&self.inner.snapshot(radius))
    }

    /// Makes up to `count` steps.
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn step(&mut self, count: f64, radius: u32) -> Out<SnapshotDto> {
        let target = self.inner.step + count;
        out(&self.inner.seek(target, radius))
    }

    /// Goes back up to `count` steps.
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn back(&mut self, count: f64, radius: u32) -> Out<SnapshotDto> {
        let target = (self.inner.step - count).max(0.0);
        out(&self.inner.seek(target, radius))
    }

    /// Moves to step `step` (or the halting step, if earlier).
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn seek(&mut self, step: f64, radius: u32) -> Out<SnapshotDto> {
        out(&self.inner.seek(step, radius))
    }

    /// Runs at most `max_steps` steps, stopping at breakpoints.
    ///
    /// # Errors
    ///
    /// Fails if `breakpoints` is not a valid `BreakpointsDto`.
    pub fn run(
        &mut self,
        max_steps: f64,
        breakpoints: Ts<BreakpointsDto>,
        radius: u32,
    ) -> Out<RunResultDto> {
        let _ = input(&breakpoints)?;
        out(&self.inner.run(max_steps, radius))
    }
}

/// An incrementally rendered space-time diagram.
#[wasm_bindgen(js_name = SpaceTimeJob)]
pub struct SpaceTimeJob {
    inner: stub::SpaceTime,
}

#[wasm_bindgen(js_class = SpaceTimeJob)]
impl SpaceTimeJob {
    /// Prepares a diagram of `spec` on `input`.
    ///
    /// # Errors
    ///
    /// Fails for invalid specs, machines with several tapes and invalid sizes.
    #[wasm_bindgen(constructor)]
    pub fn new(
        spec: Ts<MachineSpecDto>,
        input_word: &str,
        request: Ts<SpaceTimeRequestDto>,
    ) -> Result<SpaceTimeJob, JsError> {
        let _ = (input(&spec)?, input_word);
        let request = input(&request)?;
        Ok(Self {
            inner: stub::SpaceTime::new(request.width, request.height),
        })
    }

    /// Simulates at most `max_steps` more steps.
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn advance(&mut self, max_steps: f64) -> Out<SpaceTimeProgressDto> {
        let _ = max_steps;
        out(&self.inner.advance())
    }

    /// The image as RGBA bytes in the given colours.
    ///
    /// # Errors
    ///
    /// Fails if `palette` is not a valid `PaletteDto`.
    pub fn rgba(&self, palette: Ts<PaletteDto>) -> Result<Vec<u8>, JsError> {
        Ok(self.inner.rgba(&input(&palette)?))
    }

    /// The cell under pixel (x, y), or `undefined` outside the rendered part.
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn locate(&self, x: u32, y: u32) -> Result<Option<Ts<CellRefDto>>, JsError> {
        let _ = (x, y);
        Ok(None)
    }

    /// Width in pixels.
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.inner.width
    }

    /// Height in pixels.
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.inner.height
    }
}

/// An enumeration of all machines of one size in tree normal form.
#[wasm_bindgen(js_name = EnumerationJob)]
pub struct EnumerationJob {
    nodes: f64,
    target: f64,
}

#[wasm_bindgen(js_class = EnumerationJob)]
impl EnumerationJob {
    /// Enumerates `states`-state, `symbols`-symbol machines, shard `shard_index` of
    /// `shard_count`.
    ///
    /// # Errors
    ///
    /// Fails for unsupported sizes or shards.
    #[wasm_bindgen(constructor)]
    pub fn new(
        states: u8,
        symbols: u8,
        shard_index: u32,
        shard_count: u32,
    ) -> Result<EnumerationJob, JsError> {
        let _ = (symbols, shard_index, shard_count);
        let target = match states {
            1 => 3.0,
            2 => 61.0,
            3 => 5_417.0,
            _ => 858_909.0,
        };
        Ok(Self { nodes: 0.0, target })
    }

    /// Visits at most `max_nodes` more nodes.
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn advance(&mut self, max_nodes: f64) -> Out<EnumProgressDto> {
        self.nodes = (self.nodes + max_nodes).min(self.target);
        out(&stub::enum_progress(self.nodes, self.target))
    }

    /// Up to `limit` machines resolved by the decider with key `decider`.
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn examples(&self, decider: &str, limit: u32) -> Out<MachineListDto> {
        let _ = (decider, limit);
        out(&MachineListDto {
            machines: Vec::new(),
        })
    }

    /// Up to `limit` unresolved machines.
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn holdouts(&self, limit: u32) -> Out<MachineListDto> {
        let _ = limit;
        out(&MachineListDto {
            machines: Vec::new(),
        })
    }
}

/// Checks Antihydra's rules on the machine itself.
#[wasm_bindgen(js_name = AntihydraCheckJob)]
#[derive(Default)]
pub struct AntihydraCheckJob {
    _private: (),
}

#[wasm_bindgen(js_class = AntihydraCheckJob)]
impl AntihydraCheckJob {
    /// Starts the machine on the blank tape.
    #[wasm_bindgen(constructor)]
    pub fn new() -> AntihydraCheckJob {
        Self::default()
    }

    /// Runs at most `max_macro_steps` macro steps.
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn advance(&mut self, max_macro_steps: f64) -> Out<AntihydraRowsDto> {
        let _ = max_macro_steps;
        out(&AntihydraRowsDto {
            rows: Vec::new(),
            done: true,
        })
    }
}

/// Iterates Antihydra's rule with big integers.
#[wasm_bindgen(js_name = AntihydraWalkJob)]
#[derive(Default)]
pub struct AntihydraWalkJob {
    iterations: f64,
}

#[wasm_bindgen(js_class = AntihydraWalkJob)]
impl AntihydraWalkJob {
    /// Starts at h = 8, c = 0.
    #[wasm_bindgen(constructor)]
    pub fn new() -> AntihydraWalkJob {
        Self::default()
    }

    /// Runs `iterations` more iterations and returns a summary with at most `samples`
    /// samples.
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn advance(&mut self, iterations: f64, samples: u32) -> Out<WalkSummaryDto> {
        let _ = samples;
        self.iterations += iterations;
        out(&WalkSummaryDto {
            iterations: self.iterations,
            halted: false,
            evens: 0.0,
            odds: 0.0,
            balance: 0.0,
            min_balance: 0.0,
            bits: 0.0,
            samples: Vec::new(),
        })
    }
}
