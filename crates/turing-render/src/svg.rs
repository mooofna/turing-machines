//! A complete state diagram as SVG, for the command-line tool and the README.
//!
//! The drawing follows the course: states are circles with their name in the middle, final
//! states have a second, inner circle, an arrow points into the start state from the left,
//! and every edge carries its transitions as a stack of `a ↦ b, R` lines. Strokes are
//! hairlines in the ink colour, arrowheads small filled triangles, and labels have a halo in
//! the background colour so that they stay legible where they cross an edge.

use std::fmt::Write as _;

use crate::graph::Graph;
use crate::layout::{Layout, NODE_RADIUS, START_ARROW_LENGTH};
use crate::route::{EdgeRoute, LABEL_FONT_SIZE, write_num};

/// The stroke width of circles and edges.
pub const STROKE_WIDTH: f64 = 1.5;

/// The length of an arrowhead from its base to its tip.
pub const ARROW_LENGTH: f64 = 9.0;

/// Half the width of the base of an arrowhead.
pub const ARROW_HALF_WIDTH: f64 = 3.5;

/// The distance between the two circles of a final state.
pub const FINAL_INSET: f64 = 4.0;

/// The largest font size of state names; longer names are set smaller.
pub const STATE_FONT_SIZE: f64 = 14.0;

/// The font stack for state names.
pub const SANS_FONTS: &str = "'Schibsted Grotesk', 'Helvetica Neue', Helvetica, Arial, sans-serif";

/// The font stack for edge labels.
pub const MONO_FONTS: &str = "'IBM Plex Mono', ui-monospace, 'SF Mono', Menlo, Consolas, monospace";

/// The width of the background halo behind label text.
const HALO_WIDTH: f64 = 4.0;

/// The baseline of a line of text lies this many ems below its vertical centre.
const BASELINE_SHIFT: f64 = 0.35;

/// Writes a standalone SVG with course-style states (circles, final states doubled, an arrow
/// into the start state) in the given ink and accent colours.
///
/// `ink` and `background` are CSS colours such as `#111111`. `layout` gives the state
/// centres and the canvas size, `routes` the edges, as computed by [`crate::layout()`] and
/// [`crate::route()`].
///
/// ```
/// use turing_render::{Edge, Graph, LayoutKind, Node, layout, route, svg::to_svg};
/// let graph = Graph {
///     nodes: vec![Node { name: "q0".into(), is_start: true, is_final: true }],
///     edges: vec![Edge { from: 0, to: 0, labels: vec!["a ↦ a, R".into()], transitions: vec![0] }],
/// };
/// let l = layout(&graph, LayoutKind::Auto);
/// let svg = to_svg(&graph, &l, &route(&graph, &l.x, &l.y), "#111111", "#ffffff");
/// assert!(svg.starts_with("<svg") && svg.ends_with("</svg>\n"));
/// assert!(svg.contains(">q0</text>") && svg.contains(">a ↦ a, R</text>"));
/// ```
pub fn to_svg(
    graph: &Graph,
    layout: &Layout,
    routes: &[EdgeRoute],
    ink: &str,
    background: &str,
) -> String {
    let mut svg = Svg {
        graph,
        layout,
        routes,
        ink: escape(ink),
        background: escape(background),
        out: String::with_capacity(1024 + 256 * (graph.nodes.len() + routes.len())),
    };
    svg.open();
    svg.edges();
    svg.states();
    svg.arrowheads();
    svg.labels();
    svg.names();
    svg.out.push_str("</svg>\n");
    svg.out
}

/// The parts of a diagram, written back to front.
struct Svg<'a> {
    graph: &'a Graph,
    layout: &'a Layout,
    routes: &'a [EdgeRoute],
    ink: String,
    background: String,
    out: String,
}

impl Svg<'_> {
    /// The centre of state `i`.
    fn at(&self, i: usize) -> (f64, f64) {
        (
            self.layout.x.get(i).copied().unwrap_or(0.0),
            self.layout.y.get(i).copied().unwrap_or(0.0),
        )
    }

    /// The `svg` element and the background.
    fn open(&mut self) {
        let mut w = String::new();
        let mut h = String::new();
        write_num(&mut w, self.layout.width);
        write_num(&mut h, self.layout.height);
        let background = &self.background;
        line(
            &mut self.out,
            format_args!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" \
                 viewBox=\"0 0 {w} {h}\" role=\"img\" aria-label=\"State diagram\">"
            ),
        );
        line(
            &mut self.out,
            format_args!("<rect width=\"100%\" height=\"100%\" fill=\"{background}\"/>"),
        );
    }

    /// Edge lines and the shaft of the start arrow, under the states.
    fn edges(&mut self) {
        let ink = &self.ink;
        line(
            &mut self.out,
            format_args!("<g fill=\"none\" stroke=\"{ink}\" stroke-width=\"{STROKE_WIDTH}\">"),
        );
        for r in self.routes {
            line(&mut self.out, format_args!("<path d=\"{}\"/>", r.path));
        }
        for (i, node) in self.graph.nodes.iter().enumerate() {
            if node.is_start {
                let (x, y) = self.at(i);
                self.out.push_str("<path d=\"M ");
                write_point(&mut self.out, x - NODE_RADIUS - START_ARROW_LENGTH, y);
                self.out.push_str(" L ");
                write_point(&mut self.out, x - NODE_RADIUS, y);
                self.out.push_str("\"/>\n");
            }
        }
        self.out.push_str("</g>\n");
    }

    /// Circles filled with the background, so that edges end at the rims.
    fn states(&mut self) {
        let (ink, background) = (&self.ink, &self.background);
        line(
            &mut self.out,
            format_args!(
                "<g fill=\"{background}\" stroke=\"{ink}\" stroke-width=\"{STROKE_WIDTH}\">"
            ),
        );
        for (i, node) in self.graph.nodes.iter().enumerate() {
            let (x, y) = self.at(i);
            circle(&mut self.out, x, y, NODE_RADIUS);
            if node.is_final {
                circle(&mut self.out, x, y, NODE_RADIUS - FINAL_INSET);
            }
        }
        self.out.push_str("</g>\n");
    }

    /// Arrowheads over the rims.
    fn arrowheads(&mut self) {
        let ink = &self.ink;
        line(&mut self.out, format_args!("<g fill=\"{ink}\">"));
        for r in self.routes {
            arrowhead(&mut self.out, r.tip, r.tip_angle);
        }
        for (i, node) in self.graph.nodes.iter().enumerate() {
            if node.is_start {
                let (x, y) = self.at(i);
                arrowhead(&mut self.out, (x - NODE_RADIUS, y), 0.0);
            }
        }
        self.out.push_str("</g>\n");
    }

    /// Edge labels with a halo in the background colour.
    fn labels(&mut self) {
        let (ink, background) = (&self.ink, &self.background);
        line(
            &mut self.out,
            format_args!(
                "<g font-family=\"{MONO_FONTS}\" font-size=\"{LABEL_FONT_SIZE}\" \
                 fill=\"{ink}\" stroke=\"{background}\" stroke-width=\"{HALO_WIDTH}\" \
                 stroke-linejoin=\"round\" paint-order=\"stroke\">"
            ),
        );
        for r in self.routes {
            let Some(edge) = self.graph.edges.get(r.edge) else {
                continue;
            };
            for (i, text) in edge.labels.iter().enumerate() {
                let baseline = r.label_line_center(i) + BASELINE_SHIFT * LABEL_FONT_SIZE;
                self.out.push_str("<text x=\"");
                write_num(&mut self.out, r.label.0);
                self.out.push_str("\" y=\"");
                write_num(&mut self.out, baseline);
                self.out.push('"');
                if r.label_anchor == "start" {
                    line(&mut self.out, format_args!(">{}</text>", escape(text)));
                } else {
                    line(
                        &mut self.out,
                        format_args!(
                            " text-anchor=\"{}\">{}</text>",
                            r.label_anchor,
                            escape(text)
                        ),
                    );
                }
            }
        }
        self.out.push_str("</g>\n");
    }

    /// State names, centred in their circles.
    fn names(&mut self) {
        let ink = &self.ink;
        line(
            &mut self.out,
            format_args!("<g font-family=\"{SANS_FONTS}\" fill=\"{ink}\" text-anchor=\"middle\">"),
        );
        for (i, node) in self.graph.nodes.iter().enumerate() {
            let (x, y) = self.at(i);
            let size = name_size(&node.name, node.is_final);
            self.out.push_str("<text x=\"");
            write_num(&mut self.out, x);
            self.out.push_str("\" y=\"");
            write_num(&mut self.out, y + BASELINE_SHIFT * size);
            self.out.push_str("\" font-size=\"");
            write_num(&mut self.out, size);
            line(
                &mut self.out,
                format_args!("\">{}</text>", escape(&node.name)),
            );
        }
        self.out.push_str("</g>\n");
    }
}

/// The font size at which `name` fits into its circle.
fn name_size(name: &str, is_final: bool) -> f64 {
    let room = 2.0 * (NODE_RADIUS - if is_final { FINAL_INSET } else { 0.0 }) - 8.0;
    #[expect(clippy::cast_precision_loss, reason = "names are short")]
    let chars = name.chars().count().max(1) as f64;
    (room / (0.6 * chars)).clamp(8.0, STATE_FONT_SIZE)
}

/// Appends formatted text and a newline.
fn line(out: &mut String, args: std::fmt::Arguments<'_>) {
    // Writing to a String cannot fail.
    let _ = out.write_fmt(args);
    out.push('\n');
}

fn write_point(out: &mut String, x: f64, y: f64) {
    write_num(out, x);
    out.push(' ');
    write_num(out, y);
}

fn circle(out: &mut String, x: f64, y: f64, r: f64) {
    out.push_str("<circle cx=\"");
    write_num(out, x);
    out.push_str("\" cy=\"");
    write_num(out, y);
    out.push_str("\" r=\"");
    write_num(out, r);
    out.push_str("\"/>\n");
}

/// A filled triangle with its tip at `tip`, pointing in direction `angle`.
fn arrowhead(out: &mut String, tip: (f64, f64), angle: f64) {
    let (dx, dy) = (angle.cos(), angle.sin());
    let base = (tip.0 - dx * ARROW_LENGTH, tip.1 - dy * ARROW_LENGTH);
    let (nx, ny) = (-dy * ARROW_HALF_WIDTH, dx * ARROW_HALF_WIDTH);
    out.push_str("<path d=\"M ");
    write_point(out, tip.0, tip.1);
    out.push_str(" L ");
    write_point(out, base.0 + nx, base.1 + ny);
    out.push_str(" L ");
    write_point(out, base.0 - nx, base.1 - ny);
    out.push_str(" Z\"/>\n");
}

/// Escapes text for XML content and attribute values.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_markup() {
        assert_eq!(
            escape("a<b & \"c\" 'd'>"),
            "a&lt;b &amp; &quot;c&quot; &apos;d&apos;&gt;"
        );
    }

    #[test]
    fn long_names_get_smaller() {
        assert_eq!(name_size("q0", false), STATE_FONT_SIZE);
        assert!(name_size("accept", false) < STATE_FONT_SIZE);
        assert!(name_size("accept", true) < name_size("accept", false));
        assert_eq!(name_size("a_very_long_state_name", false), 8.0);
    }
}
