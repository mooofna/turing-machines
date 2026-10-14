//! `tm render`: space-time diagrams as PNG and state diagrams as SVG.

use std::io::Write;
use std::path::{Path, PathBuf};

use turing_render::{
    Graph, LayoutKind, Palette, Rgb, SpaceTime, SpaceTimeMode, SpaceTimeRequest, layout, route,
    svg::to_svg,
};

use crate::cmd::parse_count;
use crate::error::{Error, Result};
use crate::input::{self, InputArgs};
use crate::load::{self, MachineArg};
use crate::text;

const AFTER_HELP: &str = "\
A space-time diagram shows the tape across and time downwards. With --overview, the whole run up
to MAX_STEPS is sampled at evenly spaced steps and every column is a bin of cells, shaded by how
many are not blank; with --exact, every row is one step and every column one cell. The head is
drawn in the accent colour.

Examples:
  tm render --standard 1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA --spacetime bb5.png
  tm render --standard 1RB1LB_1LA1RZ --spacetime bb2.png --exact 0 --width 200 --height 100
  tm render machines/course/unary-multiplication.tm --diagram mul.svg --theme dark";

/// Arguments of `tm render`.
#[derive(clap::Args, Debug)]
#[command(after_help = AFTER_HELP)]
#[command(group(clap::ArgGroup::new("outputs").args(["spacetime", "diagram"]).required(true).multiple(true)))]
pub(crate) struct Args {
    #[command(flatten)]
    machine: MachineArg,

    #[command(flatten)]
    input: InputArgs,

    /// Write a space-time diagram of the run as PNG to this file
    #[arg(long, value_name = "OUT.png")]
    spacetime: Option<PathBuf>,

    /// Write the state diagram as SVG to this file
    #[arg(long, value_name = "OUT.svg")]
    diagram: Option<PathBuf>,

    /// The width of the space-time diagram in pixels
    #[arg(long, value_name = "PIXELS", default_value_t = 800)]
    width: u32,

    /// The height of the space-time diagram in pixels
    #[arg(long, value_name = "PIXELS", default_value_t = 800)]
    height: u32,

    /// Sample the run up to this many steps, one row per sample [default: 1e8]
    #[arg(long, value_name = "MAX_STEPS", value_parser = parse_count, conflicts_with = "exact")]
    overview: Option<u64>,

    /// Draw one row per step, starting at this step
    #[arg(long, value_name = "FIRST_STEP", value_parser = parse_count)]
    exact: Option<u64>,

    /// The colours: the light or the dark theme of the website
    #[arg(long, value_enum, value_name = "THEME", default_value_t = Theme::Light)]
    theme: Theme,
}

/// The colour themes of the website.
#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
enum Theme {
    /// Dark ink on white
    Light,
    /// Light ink on near black
    Dark,
}

impl Theme {
    fn palette(self) -> Palette {
        match self {
            Self::Light => Palette {
                background: Rgb(0xff, 0xff, 0xff),
                ink: Rgb(0x11, 0x11, 0x11),
                accent: Rgb(0x21, 0x5c, 0xaf),
            },
            Self::Dark => Palette {
                background: Rgb(0x0e, 0x0e, 0x0e),
                ink: Rgb(0xf1, 0xf1, 0xf1),
                accent: Rgb(0x7b, 0xa7, 0xf0),
            },
        }
    }

    /// Ink and background for SVG.
    fn svg_colours(self) -> (&'static str, &'static str) {
        match self {
            Self::Light => ("#111111", "#ffffff"),
            Self::Dark => ("#f1f1f1", "#0e0e0e"),
        }
    }
}

/// The default step budget of an overview.
const DEFAULT_OVERVIEW_STEPS: u64 = 100_000_000;

/// How many machine steps to simulate between checks of the progress.
const CHUNK_STEPS: u64 = 1 << 22;

fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    std::fs::write(path, bytes).map_err(|source| Error::File {
        action: "write",
        path: path.display().to_string(),
        source,
    })
}

fn encode_png(
    width: u32,
    height: u32,
    rgba: &[u8],
) -> std::result::Result<Vec<u8>, png::EncodingError> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(rgba)?;
        writer.finish()?;
    }
    Ok(bytes)
}

/// Runs `tm render`.
pub(crate) fn exec(args: &Args) -> Result<()> {
    let mut out = anstream::stdout().lock();
    let mut err = anstream::stderr().lock();
    let loaded = load::load(&args.machine, &mut err)?;

    if let Some(path) = &args.spacetime {
        let input = input::prepare(&loaded.machine, &args.input)?;
        let mode = match args.exact {
            Some(first_step) => SpaceTimeMode::Exact {
                first_step,
                cell_px: None,
            },
            None => SpaceTimeMode::Overview {
                max_steps: args.overview.unwrap_or(DEFAULT_OVERVIEW_STEPS),
            },
        };
        let request = SpaceTimeRequest {
            width: args.width,
            height: args.height,
            mode,
        };
        let mut diagram = SpaceTime::new(&loaded.machine, &input.symbols, request)
            .map_err(|e| Error::usage(e.to_string()))?;
        let mut progress = diagram.progress();
        while !progress.done {
            let next = diagram.advance(CHUNK_STEPS);
            if next == progress {
                // No progress at all: the renderer has nothing more to do.
                break;
            }
            progress = next;
        }
        let rgba = diagram.rgba(&args.theme.palette());
        let png = encode_png(diagram.width(), diagram.height(), &rgba)
            .map_err(|e| Error::failed(format!("cannot encode the PNG: {e}")))?;
        write_file(path, &png)?;
        let end = if progress.halted {
            "the machine halted"
        } else {
            "the machine had not halted by the last row"
        };
        writeln!(
            out,
            "wrote {} ({} × {} pixels, {} simulated, {end})",
            path.display(),
            diagram.width(),
            diagram.height(),
            text::count(progress.steps, "step", "steps"),
        )?;
    }

    if let Some(path) = &args.diagram {
        let graph = Graph::from_spec(&loaded.spec);
        let placed = layout(&graph, LayoutKind::Auto);
        let routes = route(&graph, &placed.x, &placed.y);
        let (ink, background) = args.theme.svg_colours();
        let svg = to_svg(&graph, &placed, &routes, ink, background);
        write_file(path, svg.as_bytes())?;
        writeln!(
            out,
            "wrote {} ({}, {})",
            path.display(),
            text::count(graph.nodes.len() as u64, "state", "states"),
            text::count(graph.edges.len() as u64, "edge", "edges"),
        )?;
    }
    Ok(())
}
