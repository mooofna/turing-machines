#![doc = include_str!("../README.md")]
// wasm-bindgen exports are `pub` by necessity; the generated glue uses them.
#![allow(unreachable_pub, reason = "wasm-bindgen exports")]

mod convert;
mod cryptids;
pub mod dto;
mod editor;
mod enumerate;
mod read;
mod session;
mod share;
mod spacetime;
mod spans;
#[cfg(test)]
mod testing;

use std::fmt;

use serde::Serialize;
use tsify::{Ts, Tsify};
use turing_bb::cryptids::antihydra::{MachineCheck, RuleWalk};
use wasm_bindgen::prelude::*;

use crate::convert::count;
use crate::dto::{
    AntihydraRowsDto, BreakpointsDto, CellRefDto, DiagnosticsDto, EdgeRoutesDto, EnumProgressDto,
    GraphLayoutDto, MachineListDto, MachineSpecDto, PaletteDto, ParseResultDto, RunResultDto,
    SnapshotDto, SpaceTimeProgressDto, SpaceTimeRequestDto, TowerDto, VerdictDto, WalkSummaryDto,
};

/// An error at the boundary, thrown to JavaScript as an `Error` with this message.
///
/// The inner modules return it instead of [`JsError`], so they can be tested natively, where
/// creating a JavaScript value is impossible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Error(String);

impl Error {
    /// An error with `message`, one plain English sentence.
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<Error> for JsError {
    fn from(error: Error) -> Self {
        JsError::new(&error.0)
    }
}

/// A value that crosses the boundary as its TypeScript type.
type Out<T> = Result<Ts<T>, JsError>;

/// Serialises a DTO for JavaScript.
fn out<T: Tsify + Serialize>(value: &T) -> Out<T> {
    Ts::from_rust(value).map_err(|e| JsError::new(&e.to_string()))
}

/// Reads a spec from JavaScript inside the function, so a malformed value becomes an ordinary
/// error instead of a leak at the ABI boundary.
fn spec_arg(spec: &Ts<MachineSpecDto>) -> Result<MachineSpecDto, JsError> {
    Ok(read::spec(&spec.js_value())?)
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
/// Diagnostics carry UTF-16 offsets into `source` and 1-based lines and UTF-16 columns.
///
/// # Errors
///
/// Only on internal serialisation failures; problems in the source are diagnostics.
#[wasm_bindgen(js_name = parseSource)]
pub fn parse_source(source: &str) -> Out<ParseResultDto> {
    out(&editor::parse_source(source))
}

/// Writes a spec in canonical form; `ascii` replaces δ, Σ, Γ and ␣.
///
/// # Errors
///
/// Fails if `spec` is not a valid `MachineSpecDto`, including glyphs that are not exactly one
/// character.
#[wasm_bindgen(js_name = formatSpec)]
#[expect(
    clippy::needless_pass_by_value,
    reason = "wasm-bindgen passes owned values"
)]
pub fn format_spec(spec: Ts<MachineSpecDto>, ascii: bool) -> Result<String, JsError> {
    Ok(editor::format_spec(&spec_arg(&spec)?, ascii)?)
}

/// Checks a spec and returns every error and warning.
///
/// Spans are taken over from the transition spans of `spec` unchanged (UTF-16, without line
/// and column). Glyphs that are not exactly one character are `E102` errors.
///
/// # Errors
///
/// Fails if `spec` is not a valid `MachineSpecDto`.
#[wasm_bindgen(js_name = checkSpec)]
#[expect(
    clippy::needless_pass_by_value,
    reason = "wasm-bindgen passes owned values"
)]
pub fn check_spec(spec: Ts<MachineSpecDto>) -> Out<DiagnosticsDto> {
    out(&editor::check_spec(&spec_arg(&spec)?))
}

/// Parses the bbchallenge standard format, for example `1RB1LB_1LA1RZ`.
///
/// # Errors
///
/// Only on internal serialisation failures; syntax errors are `E101` diagnostics.
#[wasm_bindgen(js_name = fromStandard)]
pub fn from_standard(text: &str) -> Out<ParseResultDto> {
    out(&editor::from_standard(text))
}

/// Writes a spec in the standard format, or `undefined` if it cannot be expressed there.
///
/// # Errors
///
/// Fails if `spec` is not a valid `MachineSpecDto`.
#[wasm_bindgen(js_name = toStandard)]
#[expect(
    clippy::needless_pass_by_value,
    reason = "wasm-bindgen passes owned values"
)]
pub fn to_standard(spec: Ts<MachineSpecDto>) -> Result<Option<String>, JsError> {
    Ok(editor::to_standard(&spec_arg(&spec)?)?)
}

/// Encodes a spec for a share link: `1.` followed by base64url of the deflated description.
///
/// # Errors
///
/// Fails if `spec` is not a valid `MachineSpecDto` or its description exceeds 64 KB.
#[wasm_bindgen(js_name = encodeShare)]
#[expect(
    clippy::needless_pass_by_value,
    reason = "wasm-bindgen passes owned values"
)]
pub fn encode_share(spec: Ts<MachineSpecDto>) -> Result<String, JsError> {
    Ok(share::encode(&spec_arg(&spec)?)?)
}

/// Decodes a share link payload produced by [`encode_share`].
///
/// # Errors
///
/// Only on internal serialisation failures; invalid payloads are `E100` diagnostics.
#[wasm_bindgen(js_name = decodeShare)]
pub fn decode_share(payload: &str) -> Out<ParseResultDto> {
    out(&share::decode(payload))
}

/// Places the states of a spec. `kind` is `auto`, `layered` or `circular`.
///
/// # Errors
///
/// Fails if `spec` is not a valid `MachineSpecDto`, for an unknown `kind` and for diagrams of
/// more than 4096 states or 65536 transitions.
#[wasm_bindgen(js_name = graphLayout)]
#[expect(
    clippy::needless_pass_by_value,
    reason = "wasm-bindgen passes owned values"
)]
pub fn graph_layout(spec: Ts<MachineSpecDto>, kind: &str) -> Out<GraphLayoutDto> {
    out(&editor::graph_layout(&spec_arg(&spec)?, kind)?)
}

/// Routes the edges of a spec for node centres `xs` and `ys`.
///
/// # Errors
///
/// Fails if `spec` is not a valid `MachineSpecDto` or there is not exactly one finite position
/// per state.
#[wasm_bindgen(js_name = graphRoute)]
#[expect(
    clippy::needless_pass_by_value,
    reason = "wasm-bindgen passes owned values"
)]
pub fn graph_route(spec: Ts<MachineSpecDto>, xs: &[f64], ys: &[f64]) -> Out<EdgeRoutesDto> {
    out(&editor::graph_route(&spec_arg(&spec)?, xs, ys)?)
}

/// Knuth's `base ↑↑ height`, exact up to `max_digits` digits (at most 100000).
///
/// # Errors
///
/// Only on internal serialisation failures.
#[wasm_bindgen(js_name = upArrow)]
pub fn up_arrow(base: u32, height: u32, max_digits: u32) -> Out<TowerDto> {
    out(&editor::up_arrow(base, height, max_digits))
}

/// The number of machines with `states` states and `symbols` symbols in Radó's count, as a
/// decimal string, or `undefined` if it exceeds 2^128.
#[wasm_bindgen(js_name = machineCount)]
pub fn machine_count(states: u32, symbols: u32) -> Option<String> {
    turing_bb::count::machines(states, symbols).map(|n| n.to_string())
}

/// Runs the decider pipeline on a single-tape machine started on the blank tape.
///
/// A halting verdict counts a final transition into a state without transitions (`1RZ`) as a
/// step, as in the busy beaver convention, and counts the ones after its write; a machine that
/// halts on an undefined transition reports the steps it made. Specs with errors and machines
/// with several tapes, `N` moves, an input, more than 6 states with transitions or more than 8
/// symbols are `unsupported`.
///
/// # Errors
///
/// Fails if `spec` is not a valid `MachineSpecDto`.
#[wasm_bindgen]
#[expect(
    clippy::needless_pass_by_value,
    reason = "wasm-bindgen passes owned values"
)]
pub fn decide(spec: Ts<MachineSpecDto>) -> Out<VerdictDto> {
    out(&enumerate::decide(&spec_arg(&spec)?))
}

// Engines (worker, advanced in bounded chunks).

/// A run of a machine that can step, seek in both directions and run with breakpoints.
///
/// Step counts are JavaScript numbers: fractions are cut off, `NaN` and negative counts count
/// as 0, and a session never goes beyond step 2^53 - 1. Windows show at most 4096 cells on
/// either side of a head.
#[wasm_bindgen]
#[derive(Debug)]
pub struct Session {
    inner: session::Session,
}

#[wasm_bindgen]
impl Session {
    /// Opens `spec` with `input` on the first tape.
    ///
    /// # Errors
    ///
    /// Fails if the spec has errors or the input contains symbols outside Σ.
    #[wasm_bindgen(constructor)]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "wasm-bindgen passes owned values"
    )]
    pub fn new(spec: Ts<MachineSpecDto>, input_word: &str) -> Result<Session, JsError> {
        Ok(Self {
            inner: session::Session::new(&spec_arg(&spec)?, input_word)?,
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
        self.inner.step(convert::count(count));
        out(&self.inner.snapshot(radius))
    }

    /// Goes back up to `count` steps.
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn back(&mut self, count: f64, radius: u32) -> Out<SnapshotDto> {
        self.inner.back(convert::count(count));
        out(&self.inner.snapshot(radius))
    }

    /// Moves to step `step` (or the halting step, if earlier).
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn seek(&mut self, step: f64, radius: u32) -> Out<SnapshotDto> {
        self.inner.seek(count(step));
        out(&self.inner.snapshot(radius))
    }

    /// Runs at most `max_steps` steps, stopping at breakpoints.
    ///
    /// # Errors
    ///
    /// Fails if `breakpoints` is not a valid `BreakpointsDto`.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "wasm-bindgen passes owned values"
    )]
    pub fn run(
        &mut self,
        max_steps: f64,
        breakpoints: Ts<BreakpointsDto>,
        radius: u32,
    ) -> Out<RunResultDto> {
        let breakpoints = read::breakpoints(&breakpoints.js_value())?;
        out(&self.inner.run(count(max_steps), &breakpoints, radius))
    }
}

/// An incrementally rendered space-time diagram.
#[wasm_bindgen(js_name = SpaceTimeJob)]
#[derive(Debug)]
pub struct SpaceTimeJob {
    inner: spacetime::Job,
}

#[wasm_bindgen(js_class = SpaceTimeJob)]
impl SpaceTimeJob {
    /// Prepares a diagram of `spec` on `input`.
    ///
    /// # Errors
    ///
    /// Fails for invalid specs, machines with several tapes and invalid sizes.
    #[wasm_bindgen(constructor)]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "wasm-bindgen passes owned values"
    )]
    pub fn new(
        spec: Ts<MachineSpecDto>,
        input_word: &str,
        request: Ts<SpaceTimeRequestDto>,
    ) -> Result<SpaceTimeJob, JsError> {
        Ok(Self {
            inner: spacetime::Job::new(
                &spec_arg(&spec)?,
                input_word,
                &read::spacetime_request(&request.js_value())?,
            )?,
        })
    }

    /// Simulates at most `max_steps` more steps.
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn advance(&mut self, max_steps: f64) -> Out<SpaceTimeProgressDto> {
        out(&self.inner.advance(count(max_steps)))
    }

    /// The image as RGBA bytes in the given colours.
    ///
    /// # Errors
    ///
    /// Fails if `palette` is not a valid `PaletteDto` or a colour is not `#rgb` or `#rrggbb`.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "wasm-bindgen passes owned values"
    )]
    pub fn rgba(&self, palette: Ts<PaletteDto>) -> Result<Vec<u8>, JsError> {
        let palette = spacetime::palette(&read::palette(&palette.js_value())?)?;
        Ok(self.inner.rgba(&palette))
    }

    /// The cell under pixel (x, y), or `undefined` outside the rendered part.
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn locate(&self, x: u32, y: u32) -> Result<Option<Ts<CellRefDto>>, JsError> {
        self.inner.locate(x, y).map(|cell| out(&cell)).transpose()
    }

    /// Width in pixels.
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.inner.width()
    }

    /// Height in pixels.
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.inner.height()
    }
}

/// An enumeration of all machines of one size in tree normal form.
#[wasm_bindgen(js_name = EnumerationJob)]
#[derive(Debug)]
pub struct EnumerationJob {
    inner: enumerate::Job,
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
        Ok(Self {
            inner: enumerate::Job::new(states, symbols, shard_index, shard_count)?,
        })
    }

    /// Visits at most `max_nodes` more nodes.
    ///
    /// The progress lists every decider in pipeline order, also those that resolved nothing
    /// yet; a decider used by several stages of the pipeline is listed once with the sum.
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn advance(&mut self, max_nodes: f64) -> Out<EnumProgressDto> {
        out(&self.inner.advance(count(max_nodes)))
    }

    /// Up to `limit` machines resolved by the decider with key `decider` (none for an unknown
    /// key).
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn examples(&self, decider: &str, limit: u32) -> Out<MachineListDto> {
        out(&self.inner.examples(decider, limit))
    }

    /// Up to `limit` unresolved machines.
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn holdouts(&self, limit: u32) -> Out<MachineListDto> {
        out(&self.inner.holdouts(limit))
    }
}

/// Checks Antihydra's rules on the machine itself.
#[wasm_bindgen(js_name = AntihydraCheckJob)]
#[derive(Debug, Default)]
pub struct AntihydraCheckJob {
    inner: MachineCheck,
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
        out(&cryptids::check(&mut self.inner, count(max_macro_steps)))
    }
}

/// Iterates Antihydra's rule with big integers.
#[wasm_bindgen(js_name = AntihydraWalkJob)]
#[derive(Debug, Default)]
pub struct AntihydraWalkJob {
    inner: RuleWalk,
}

#[wasm_bindgen(js_class = AntihydraWalkJob)]
impl AntihydraWalkJob {
    /// Starts at h = 8, c = 0.
    #[wasm_bindgen(constructor)]
    pub fn new() -> AntihydraWalkJob {
        Self::default()
    }

    /// Runs `iterations` more iterations and returns a summary with at most `samples`
    /// samples (at most 10000).
    ///
    /// # Errors
    ///
    /// Only on internal serialisation failures.
    pub fn advance(&mut self, iterations: f64, samples: u32) -> Out<WalkSummaryDto> {
        out(&cryptids::walk(&mut self.inner, count(iterations), samples))
    }
}

/// Inputs from JavaScript that are well-formed DTOs but describe nonsense, and malformed values,
/// must give errors or diagnostics, never panics: with `panic = "abort"` a panic would take the
/// whole module down.
#[cfg(test)]
mod robustness {
    use crate::dto::{
        BreakpointsDto, DirDto, MachineSpecDto, PaletteDto, SpaceTimeModeDto, SpaceTimeRequestDto,
        StateDto, TransitionDto,
    };
    use crate::testing::{self, t};
    use crate::{editor, enumerate, session, share, spacetime};

    fn mutations() -> Vec<(&'static str, MachineSpecDto)> {
        let base = testing::increment();
        let mut out = Vec::new();
        let mut add = |name: &'static str, f: &dyn Fn(&mut MachineSpecDto)| {
            let mut spec = base.clone();
            f(&mut spec);
            out.push((name, spec));
        };
        add("unchanged", &|_| {});
        add("no tapes", &|s| s.tapes = 0);
        add("two tapes, one entry each", &|s| s.tapes = 2);
        add("255 tapes", &|s| s.tapes = 255);
        add("empty entries", &|s| {
            s.transitions[0].read.clear();
            s.transitions[0].write.clear();
            s.transitions[0].dir.clear();
        });
        add("three reads", &|s| {
            s.transitions[0].read = vec!["1".into(); 3];
        });
        add("no states", &|s| {
            s.states.clear();
            s.transitions.clear();
            s.start = String::new();
        });
        add("unknown start", &|s| s.start = "nowhere".into());
        add("declared states missing", &|s| {
            s.declared.states = true;
            s.states.truncate(1);
        });
        add("declared alphabet missing", &|s| {
            s.declared.tape_alphabet = true;
            s.tape_alphabet = vec!["␣".into()];
        });
        add("blank in input", &|s| s.input_alphabet.push("␣".into()));
        add("duplicate transitions", &|s| {
            let first = s.transitions[0].clone();
            s.transitions.push(first);
        });
        add("100 symbols", &|s| {
            s.tape_alphabet = (0..100)
                .map(|i| char::from_u32(0x100 + i).unwrap_or('?').to_string())
                .collect();
        });
        add("5000 states", &|s| {
            s.states = (0..5000)
                .map(|i| StateDto {
                    name: format!("s{i}"),
                    is_final: false,
                })
                .collect();
        });
        add("big table", &|s| {
            s.tapes = 3;
            s.tape_alphabet = (0..64)
                .map(|i| char::from_u32(0x100 + i).unwrap_or('?').to_string())
                .collect();
            s.states = (0..4000)
                .map(|i| StateDto {
                    name: format!("s{i}"),
                    is_final: false,
                })
                .collect();
            s.transitions = vec![TransitionDto {
                from: "s0".into(),
                read: vec!["␣".into(); 3],
                to: "s1".into(),
                write: vec!["␣".into(); 3],
                dir: vec![DirDto::R; 3],
                comment: None,
                leading: Vec::new(),
                span: None,
            }];
        });
        add("huge name and comments", &|s| {
            s.name = "n".repeat(1 << 20);
            s.comments.header = vec!["c".repeat(1 << 16); 4];
        });
        add("spans out of range", &|s| {
            for tr in &mut s.transitions {
                tr.span = Some(crate::dto::SpanDto {
                    from: u32::MAX,
                    to: 0,
                });
            }
        });
        add("many transitions", &|s| {
            s.transitions = (0..70_000)
                .map(|i| t(&format!("s{i}"), "1", "q0", "1", DirDto::R))
                .collect();
        });
        add("stay loop", &|s| {
            s.transitions = vec![t("q0", "1", "q0", "1", DirDto::N)];
        });
        out
    }

    #[test]
    fn nonsense_specs_give_errors() {
        for (name, spec) in mutations() {
            let _ = editor::format_spec(&spec, false);
            let _ = editor::format_spec(&spec, true);
            let _ = editor::check_spec(&spec);
            let _ = editor::to_standard(&spec);
            let _ = editor::graph_layout(&spec, "auto");
            let n = spec.states.len();
            let _ = editor::graph_route(&spec, &vec![0.0; n], &vec![0.0; n]);
            let _ = share::encode(&spec);
            let _ = enumerate::decide(&spec);
            for input in ["", "1101", "x", "␣1␣"] {
                if let Ok(mut s) = session::Session::new(&spec, input) {
                    s.step(10);
                    s.back(5);
                    let _ = s.run(1_000, &BreakpointsDto::default(), 5);
                    let _ = s.snapshot(u32::MAX);
                    s.seek(u64::MAX);
                    s.seek(0);
                }
            }
            let request = SpaceTimeRequestDto {
                width: 16,
                height: 16,
                mode: SpaceTimeModeDto::Overview { max_steps: 100.0 },
            };
            if let Ok(mut job) = spacetime::Job::new(&spec, "", &request) {
                job.advance(u64::MAX);
                let _ = job.locate(3, 3);
            }
            eprintln!("{name}: ok");
        }
    }

    #[test]
    fn huge_and_garbage_text() {
        let garbage: String = (0..200_000u32)
            .map(|i| char::from_u32(i % 0x3000 + 1).unwrap_or('δ'))
            .collect();
        let _ = editor::parse_source(&garbage);
        let _ = editor::parse_source(&"δ(q0, ␣) = (q1, x, R)\n".repeat(20_000));
        let _ = editor::from_standard(&garbage);
        let _ = editor::from_standard(&"1RB1LB_".repeat(100_000));
        let _ = share::decode(&garbage);
        let _ = share::decode(&format!("1.{garbage}"));
        let _ = editor::up_arrow(0, u32::MAX, u32::MAX);
        let _ = editor::up_arrow(1, u32::MAX, u32::MAX);
        let _ = editor::up_arrow(u32::MAX, u32::MAX, u32::MAX);
        let _ = editor::up_arrow(10, 3, u32::MAX);
        let _ = editor::up_arrow(2, 0, 0);
        let mut s = session::Session::new(&testing::increment(), &"1".repeat(1_000_000))
            .expect("a long input is fine");
        s.step(u64::MAX);
        assert_eq!(s.snapshot(10).step, 1_000_001.0);
    }

    #[test]
    fn malformed_values_do_not_deserialize() {
        let values = [
            serde_json::json!(null),
            serde_json::json!(42),
            serde_json::json!("spec"),
            serde_json::json!({}),
            serde_json::json!({"tapes": -1}),
            serde_json::json!({"name": "x", "kind": "robot"}),
            serde_json::json!([1, 2, 3]),
        ];
        for v in &values {
            assert!(
                serde_json::from_value::<MachineSpecDto>(v.clone()).is_err(),
                "{v}"
            );
            assert!(
                serde_json::from_value::<SpaceTimeRequestDto>(v.clone()).is_err(),
                "{v}"
            );
            assert!(
                serde_json::from_value::<PaletteDto>(v.clone()).is_err(),
                "{v}"
            );
        }
        let mut spec = serde_json::to_value(testing::increment()).unwrap();
        spec["tapes"] = serde_json::json!(300);
        assert!(serde_json::from_value::<MachineSpecDto>(spec.clone()).is_err());
        spec["tapes"] = serde_json::json!(1);
        spec["transitions"][0]["dir"] = serde_json::json!(["X"]);
        assert!(serde_json::from_value::<MachineSpecDto>(spec).is_err());
        let breakpoints: BreakpointsDto =
            serde_json::from_value(serde_json::json!({"transitions": [1e9], "states": []}))
                .unwrap_or_default();
        let mut s = session::Session::new(&testing::increment(), "1").unwrap();
        let _ = s.run(10, &breakpoints, 0);
    }
}
