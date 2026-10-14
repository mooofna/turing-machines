//! Drawing the edges between placed states.
//!
//! Every edge leaves its source on the rim of the source circle and ends with its arrow tip on
//! the rim of the target circle. An edge is a straight line unless that would be ambiguous or
//! cross another state:
//!
//! * two states joined in both directions get two arcs bent to opposite sides, each to the
//!   left of its direction of travel;
//! * an edge whose straight line passes close to another state becomes a quadratic arc bent
//!   just far enough to clear it;
//! * a self-loop is a teardrop on the side of its state with the most free room, above the
//!   state when there is room there, so that loops in a diagram look alike.
//!
//! Labels sit next to the apex of their edge on the outside of the bend and extend away from
//! it. Routing is cheap enough to run on every animation frame while states are dragged.

use std::f64::consts::{PI, TAU};
use std::fmt::Write as _;
use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::graph::Graph;
use crate::layout::{NODE_RADIUS, START_ARROW_LENGTH};

/// The font size of edge labels in diagram units.
pub const LABEL_FONT_SIZE: f64 = 12.0;

/// The distance between the lines of a label block.
pub const LABEL_LINE_HEIGHT: f64 = 15.0;

/// The estimated advance of an ASCII label character, 0.6 em of a monospace font.
pub const LABEL_CHAR_WIDTH: f64 = 0.6 * LABEL_FONT_SIZE;

/// The estimated advance of any other label character, such as `↦` or `␣`, a full em: a
/// monospace font may lack these glyphs and borrow wider ones from a fallback font.
pub const LABEL_WIDE_CHAR_WIDTH: f64 = LABEL_FONT_SIZE;

/// The free space between an edge and its label.
pub const LABEL_GAP: f64 = 4.0;

/// The free space an edge keeps around states it passes.
const CLEARANCE: f64 = 10.0;

/// Below this length two points count as the same.
const EPS: f64 = 1e-9;

/// The drawing of one edge.
///
/// The label block of an edge has one line per transition. `label.0` is the x coordinate at
/// which `label_anchor` aligns every line and `label.1` is the top of the block, so line `i`
/// (from 0) is centred vertically on `label.1 + (i + 0.5) * LABEL_LINE_HEIGHT`; see
/// [`EdgeRoute::label_line_center`].
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeRoute {
    /// Index into [`Graph::edges`].
    pub edge: usize,
    /// SVG path data from the source rim to the target rim, arrow tip at the end.
    pub path: String,
    /// The direction of the arrow tip in radians, for drawing the arrowhead.
    pub tip_angle: f64,
    /// The tip position.
    pub tip: (f64, f64),
    /// The anchor of the label block.
    pub label: (f64, f64),
    /// `start`, `middle` or `end`, as in SVG `text-anchor`.
    pub label_anchor: &'static str,
}

impl EdgeRoute {
    /// The y coordinate of the vertical centre of label line `line`.
    pub fn label_line_center(&self, line: usize) -> f64 {
        #[expect(
            clippy::cast_precision_loss,
            reason = "label blocks have a handful of lines"
        )]
        let line = line as f64;
        self.label.1 + (line + 0.5) * LABEL_LINE_HEIGHT
    }
}

/// The estimated width and height of a label block with these lines.
///
/// ```
/// use turing_render::route::{
///     LABEL_CHAR_WIDTH, LABEL_LINE_HEIGHT, LABEL_WIDE_CHAR_WIDTH, label_size,
/// };
/// let (w, h) = label_size(&["a ↦ b, R".to_string(), "␣ ↦ ␣, L".to_string()]);
/// assert_eq!(w, 5.0 * LABEL_CHAR_WIDTH + 3.0 * LABEL_WIDE_CHAR_WIDTH);
/// assert_eq!(h, 2.0 * LABEL_LINE_HEIGHT);
/// ```
pub fn label_size(lines: &[String]) -> (f64, f64) {
    let width = |line: &String| -> f64 {
        line.chars()
            .map(|c| {
                if c.is_ascii() {
                    LABEL_CHAR_WIDTH
                } else {
                    LABEL_WIDE_CHAR_WIDTH
                }
            })
            .sum()
    };
    #[expect(clippy::cast_precision_loss, reason = "a handful of lines")]
    let height = lines.len() as f64 * LABEL_LINE_HEIGHT;
    (lines.iter().map(width).fold(0.0, f64::max), height)
}

/// Routes every edge of `graph` for node centres `x` and `y`.
///
/// The routes come in the order of [`Graph::edges`]. Edges that name a node outside the
/// graph are left out. Missing or non-finite coordinates count as 0.
pub fn route(graph: &Graph, x: &[f64], y: &[f64]) -> Vec<EdgeRoute> {
    geometry(graph, x, y)
        .iter()
        .map(|g| {
            let mut path = String::with_capacity(64);
            g.curve.write_path(&mut path);
            EdgeRoute {
                edge: g.edge,
                path,
                tip_angle: g.tip_angle,
                tip: (g.tip.x, g.tip.y),
                label: (g.label.x, g.label.y),
                label_anchor: g.anchor.as_str(),
            }
        })
        .collect()
}

/// A point or a direction in diagram units.
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub(crate) struct V {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

impl V {
    pub(crate) const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// The unit vector at `angle`.
    fn polar(angle: f64) -> Self {
        Self::new(angle.cos(), angle.sin())
    }

    fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    fn len(self) -> f64 {
        self.x.hypot(self.y)
    }

    fn angle(self) -> f64 {
        self.y.atan2(self.x)
    }

    /// The unit vector in this direction, or `fallback` for a zero vector.
    fn unit_or(self, fallback: Self) -> Self {
        let len = self.len();
        if len > EPS { self / len } else { fallback }
    }

    /// The direction 90 degrees to the left of `self` on screen, where y points down.
    fn left(self) -> Self {
        Self::new(self.y, -self.x)
    }

    fn dist(self, other: Self) -> f64 {
        (self - other).len()
    }
}

impl Add for V {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self::new(self.x + o.x, self.y + o.y)
    }
}

impl Sub for V {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Self::new(self.x - o.x, self.y - o.y)
    }
}

impl Neg for V {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl Mul<f64> for V {
    type Output = Self;
    fn mul(self, k: f64) -> Self {
        Self::new(self.x * k, self.y * k)
    }
}

impl Div<f64> for V {
    type Output = Self;
    fn div(self, k: f64) -> Self {
        Self::new(self.x / k, self.y / k)
    }
}

/// The shape of an edge.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Curve {
    /// A straight line.
    Line(V, V),
    /// A quadratic Bézier curve.
    Quad(V, V, V),
    /// A cubic Bézier curve, used for self-loops.
    Cubic(V, V, V, V),
}

impl Curve {
    /// The point at parameter `t` in 0 to 1.
    pub(crate) fn at(self, t: f64) -> V {
        let rest = 1.0 - t;
        match self {
            Self::Line(p0, p1) => p0 * rest + p1 * t,
            Self::Quad(p0, c, p1) => p0 * (rest * rest) + c * (2.0 * rest * t) + p1 * (t * t),
            Self::Cubic(p0, c1, c2, p1) => {
                p0 * (rest * rest * rest)
                    + c1 * (3.0 * rest * rest * t)
                    + c2 * (3.0 * rest * t * t)
                    + p1 * (t * t * t)
            }
        }
    }

    /// The derivative at parameter `t`.
    fn tangent(self, t: f64) -> V {
        let rest = 1.0 - t;
        match self {
            Self::Line(p0, p1) => p1 - p0,
            Self::Quad(p0, c, p1) => (c - p0) * (2.0 * rest) + (p1 - c) * (2.0 * t),
            Self::Cubic(p0, c1, c2, p1) => {
                (c1 - p0) * (3.0 * rest * rest)
                    + (c2 - c1) * (6.0 * rest * t)
                    + (p1 - c2) * (3.0 * t * t)
            }
        }
    }

    fn start(self) -> V {
        match self {
            Self::Line(a, _) | Self::Quad(a, _, _) | Self::Cubic(a, _, _, _) => a,
        }
    }

    fn end(self) -> V {
        match self {
            Self::Line(_, b) | Self::Quad(_, _, b) | Self::Cubic(_, _, _, b) => b,
        }
    }

    /// Appends SVG path data.
    fn write_path(self, out: &mut String) {
        let points: &[V] = match &self {
            Self::Line(a, b) => &[*a, *b],
            Self::Quad(a, c, b) => &[*a, *c, *b],
            Self::Cubic(a, c1, c2, b) => &[*a, *c1, *c2, *b],
        };
        let command = match self {
            Self::Line(..) => 'L',
            Self::Quad(..) => 'Q',
            Self::Cubic(..) => 'C',
        };
        out.push('M');
        for (i, p) in points.iter().enumerate() {
            if i == 1 {
                out.push(' ');
                out.push(command);
            }
            out.push(' ');
            write_num(out, p.x);
            out.push(' ');
            write_num(out, p.y);
        }
    }
}

/// Writes `v` rounded to two decimals without trailing zeros, and never `-0`.
pub(crate) fn write_num(out: &mut String, v: f64) {
    let rounded = (v * 100.0).round() / 100.0;
    if rounded == 0.0 || !rounded.is_finite() {
        out.push('0');
    } else {
        // Writing to a String cannot fail.
        let _ = write!(out, "{rounded}");
    }
}

/// How a label block is aligned at its anchor.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Anchor {
    /// The block starts at the anchor and extends to the right.
    Start,
    /// The block is centred on the anchor.
    Middle,
    /// The block ends at the anchor.
    End,
}

impl Anchor {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Middle => "middle",
            Self::End => "end",
        }
    }

    /// The left edge of a block of width `w` anchored at `x`.
    fn left(self, x: f64, w: f64) -> f64 {
        match self {
            Self::Start => x,
            Self::Middle => x - w / 2.0,
            Self::End => x - w,
        }
    }
}

/// An axis-aligned rectangle.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Rect {
    pub(crate) x0: f64,
    pub(crate) y0: f64,
    pub(crate) x1: f64,
    pub(crate) y1: f64,
}

impl Rect {
    fn overlaps(self, o: Self) -> bool {
        self.x0 < o.x1 && o.x0 < self.x1 && self.y0 < o.y1 && o.y0 < self.y1
    }

    /// The smallest rectangle containing `a` and `b`.
    fn spanning(a: V, b: V) -> Self {
        Self {
            x0: a.x.min(b.x),
            y0: a.y.min(b.y),
            x1: a.x.max(b.x),
            y1: a.y.max(b.y),
        }
    }

    /// The rectangle with `d` added on every side.
    fn grown(self, d: f64) -> Self {
        Self {
            x0: self.x0 - d,
            y0: self.y0 - d,
            x1: self.x1 + d,
            y1: self.y1 + d,
        }
    }

    /// Whether the rectangles overlap or touch.
    fn touches(self, o: Self) -> bool {
        self.x0 <= o.x1 && o.x0 <= self.x1 && self.y0 <= o.y1 && o.y0 <= self.y1
    }

    /// The distance from `p` to the rectangle, 0 inside.
    fn dist(self, p: V) -> f64 {
        let dx = (self.x0 - p.x).max(p.x - self.x1).max(0.0);
        let dy = (self.y0 - p.y).max(p.y - self.y1).max(0.0);
        dx.hypot(dy)
    }
}

/// The geometry of one routed edge.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Geometry {
    pub(crate) edge: usize,
    pub(crate) curve: Curve,
    pub(crate) tip: V,
    pub(crate) tip_angle: f64,
    pub(crate) label: V,
    pub(crate) anchor: Anchor,
    pub(crate) label_box: Rect,
}

/// Segments per curve in collision tests.
const SAMPLES: usize = 16;

/// A curve sampled as a polyline, for collision tests.
#[derive(Clone, Debug)]
struct Trace {
    /// The edge, or `usize::MAX` for the start arrow.
    edge: usize,
    points: [V; SAMPLES + 1],
    /// The number of points used: 2 for a line.
    len: usize,
    bounds: Rect,
}

impl Trace {
    fn new(edge: usize, curve: Curve) -> Self {
        let mut points = [V::default(); SAMPLES + 1];
        let len = if matches!(curve, Curve::Line(..)) {
            points[0] = curve.start();
            points[1] = curve.end();
            2
        } else {
            #[expect(clippy::cast_precision_loss, reason = "a handful of samples")]
            for (k, p) in points.iter_mut().enumerate() {
                *p = curve.at(k as f64 / SAMPLES as f64);
            }
            SAMPLES + 1
        };
        let bounds = points[..len].iter().fold(
            Rect {
                x0: f64::INFINITY,
                y0: f64::INFINITY,
                x1: f64::NEG_INFINITY,
                y1: f64::NEG_INFINITY,
            },
            |r, p| Rect {
                x0: r.x0.min(p.x),
                y0: r.y0.min(p.y),
                x1: r.x1.max(p.x),
                y1: r.y1.max(p.y),
            },
        );
        Self {
            edge,
            points,
            len,
            bounds,
        }
    }

    fn segments(&self) -> impl Iterator<Item = (V, V)> + '_ {
        self.points[..self.len].windows(2).map(|w| (w[0], w[1]))
    }

    fn hits_rect(&self, r: Rect) -> bool {
        self.bounds.touches(r) && self.segments().any(|(a, b)| segment_hits_rect(a, b, r))
    }

    fn crosses(&self, other: &Self) -> bool {
        self.bounds.touches(other.bounds)
            && self.segments().any(|(a, b)| {
                let ab = Rect::spanning(a, b);
                ab.touches(other.bounds)
                    && other.segments().any(|(c, d)| {
                        ab.touches(Rect::spanning(c, d)) && segments_cross(a, b, c, d)
                    })
            })
    }
}

/// Whether the segment from `a` to `b` meets the rectangle (Liang and Barsky clipping).
fn segment_hits_rect(from: V, to: V, rect: Rect) -> bool {
    if !Rect::spanning(from, to).touches(rect) {
        return false;
    }
    let delta = to - from;
    let (mut enter, mut leave) = (0.0_f64, 1.0_f64);
    for (towards, room) in [
        (-delta.x, from.x - rect.x0),
        (delta.x, rect.x1 - from.x),
        (-delta.y, from.y - rect.y0),
        (delta.y, rect.y1 - from.y),
    ] {
        if towards == 0.0 {
            if room < 0.0 {
                return false;
            }
        } else {
            let at = room / towards;
            if towards < 0.0 {
                enter = enter.max(at);
            } else {
                leave = leave.min(at);
            }
            if enter > leave {
                return false;
            }
        }
    }
    true
}

/// Whether two segments properly cross.
fn segments_cross(a: V, b: V, c: V, d: V) -> bool {
    let orient = |p: V, q: V, r: V| (q - p).x * (r - p).y - (q - p).y * (r - p).x;
    let (d1, d2) = (orient(c, d, a), orient(c, d, b));
    let (d3, d4) = (orient(a, b, c), orient(a, b, d));
    d1 * d2 < 0.0 && d3 * d4 < 0.0
}

/// What is already drawn while loops and labels are being placed.
struct Scene<'a> {
    pos: &'a [V],
    traces: Vec<Trace>,
    boxes: Vec<Rect>,
}

impl Scene<'_> {
    /// How badly a label box of edge `own` collides with states, labels and other edges.
    ///
    /// Other edges must keep a small distance, so that a label is never read as belonging
    /// to a neighbouring edge, and other labels a character's width to the side, so that two
    /// labels never read as one line.
    ///
    /// Counting stops once the cost reaches `limit`, the best cost found so far.
    fn box_cost(&self, b: Rect, own: usize, limit: usize) -> usize {
        const KEEP: f64 = 4.0;
        let wide = b.grown(KEEP);
        let apart = Rect {
            x0: b.x0 - LABEL_CHAR_WIDTH,
            x1: b.x1 + LABEL_CHAR_WIDTH,
            ..wide
        };
        let mut cost = 10
            * self
                .pos
                .iter()
                .filter(|&&c| b.dist(c) < NODE_RADIUS + KEEP)
                .count();
        for o in &self.boxes {
            if cost >= limit {
                return cost;
            }
            if o.overlaps(apart) {
                cost += 6;
            }
        }
        for t in &self.traces {
            if cost >= limit {
                return cost;
            }
            if t.edge != own && t.hits_rect(wide) {
                cost += 3;
            }
        }
        cost
    }
}

/// Which way the label of a routed edge may go.
#[derive(Copy, Clone, Debug)]
enum Shape {
    /// A straight line: either side.
    Line,
    /// An arc bent to the left of its direction of travel (`outer` = 1) or to the right
    /// (`outer` = -1). The label prefers the outside of the bend.
    Arc { outer: f64 },
    /// A self-loop in direction `out`: its label goes outwards.
    Loop { out: V },
}

/// An edge whose curve is fixed and whose label is still to be placed.
struct Routed {
    edge: usize,
    curve: Curve,
    tip_angle: f64,
    shape: Shape,
    size: (f64, f64),
}

impl Routed {
    /// Where the label may go, best first: points on the curve and the side to put it on.
    fn label_spots(&self) -> Vec<(V, V)> {
        let at = |t: f64| self.curve.at(t);
        let normal = |t: f64| self.curve.tangent(t).unit_or(V::new(1.0, 0.0)).left();
        match self.shape {
            Shape::Line => {
                let left = normal(0.5);
                // Above a horizontal edge, right of a vertical one.
                let up = if left.y < -EPS || (left.y.abs() <= EPS && left.x > 0.0) {
                    left
                } else {
                    -left
                };
                [
                    (0.5, up),
                    (0.5, -up),
                    (0.36, up),
                    (0.64, up),
                    (0.36, -up),
                    (0.64, -up),
                    (0.24, up),
                    (0.76, up),
                    (0.24, -up),
                    (0.76, -up),
                ]
                .into_iter()
                .map(|(t, side)| (at(t), side))
                .collect()
            }
            Shape::Arc { outer } => {
                let mut spots: Vec<(V, V)> = [0.5, 0.38, 0.62, 0.28, 0.72]
                    .into_iter()
                    .map(|t| (at(t), normal(t) * outer))
                    .collect();
                // Inside the bend, next to the partner arc if there is one, so last.
                spots.push((at(0.5), normal(0.5) * -outer));
                spots
            }
            Shape::Loop { out } => vec![(at(0.5), out)],
        }
    }
}

/// Routes all edges and places their labels.
pub(crate) fn geometry(graph: &Graph, xs: &[f64], ys: &[f64]) -> Vec<Geometry> {
    let n = graph.nodes.len();
    let coord = |v: &[f64], i: usize| v.get(i).copied().filter(|c| c.is_finite()).unwrap_or(0.0);
    let pos: Vec<V> = (0..n).map(|i| V::new(coord(xs, i), coord(ys, i))).collect();

    let mut pairs: Vec<(usize, usize)> = graph.edges.iter().map(|e| (e.from, e.to)).collect();
    pairs.sort_unstable();
    let joined = |a: usize, b: usize| pairs.binary_search(&(a, b)).is_ok();

    // Edges between two states first: self-loops avoid where they meet the rims.
    let mut routed: Vec<Option<Routed>> = Vec::with_capacity(graph.edges.len());
    let mut rim_angles: Vec<Vec<f64>> = vec![Vec::new(); n];
    let mut scene = Scene {
        pos: &pos,
        traces: Vec::with_capacity(graph.edges.len() + 1),
        boxes: Vec::with_capacity(graph.edges.len()),
    };
    for (i, e) in graph.edges.iter().enumerate() {
        if e.from >= n || e.to >= n || e.from == e.to {
            routed.push(None);
            continue;
        }
        let r = between(i, e.from, e.to, &pos, joined(e.to, e.from), &e.labels);
        rim_angles[e.from].push((r.curve.start() - pos[e.from]).angle());
        rim_angles[e.to].push((r.curve.end() - pos[e.to]).angle());
        scene.traces.push(Trace::new(i, r.curve));
        routed.push(Some(r));
    }
    for (i, node) in graph.nodes.iter().enumerate() {
        if node.is_start {
            rim_angles[i].push(PI);
            let tip = pos[i] - V::new(NODE_RADIUS, 0.0);
            let tail = tip - V::new(START_ARROW_LENGTH, 0.0);
            scene
                .traces
                .push(Trace::new(usize::MAX, Curve::Line(tail, tip)));
        }
    }

    // Self-loops, each with its label, where they collide least.
    let mut placed: Vec<Option<Geometry>> = vec![None; routed.len()];
    for (i, e) in graph.edges.iter().enumerate() {
        if e.from < n && e.from == e.to {
            let (r, g) = self_loop(&scene, i, e.from, &rim_angles[e.from], &e.labels);
            scene.traces.push(Trace::new(i, r.curve));
            scene.boxes.push(g.label_box);
            routed[i] = Some(r);
            placed[i] = Some(g);
        }
    }

    // Labels of arcs, which have fewer good spots, then of straight edges.
    for arcs in [true, false] {
        for (slot, r) in placed.iter_mut().zip(&routed) {
            let Some(r) = r else { continue };
            if slot.is_some() || matches!(r.shape, Shape::Arc { .. }) != arcs {
                continue;
            }
            let mut best: Option<(usize, Geometry)> = None;
            for (spot, side) in r.label_spots() {
                let g = labelled(r, spot, side);
                let limit = best.as_ref().map_or(usize::MAX, |(c, _)| *c);
                let cost = scene.box_cost(g.label_box, r.edge, limit);
                if cost < limit {
                    best = Some((cost, g));
                }
                if cost == 0 {
                    break;
                }
            }
            if let Some((_, g)) = best {
                scene.boxes.push(g.label_box);
                *slot = Some(g);
            }
        }
    }
    placed.into_iter().flatten().collect()
}

/// The number of pairs of edges that cross and of labels that collide with something, a
/// measure of how cluttered a drawing is.
pub(crate) fn clutter(graph: &Graph, xs: &[f64], ys: &[f64]) -> usize {
    let geo = geometry(graph, xs, ys);
    let traces: Vec<Trace> = geo.iter().map(|g| Trace::new(g.edge, g.curve)).collect();
    let mut count = 0;
    for (i, a) in traces.iter().enumerate() {
        count += traces[i + 1..].iter().filter(|b| a.crosses(b)).count();
    }
    let n = graph.nodes.len();
    let coord = |v: &[f64], i: usize| v.get(i).copied().filter(|c| c.is_finite()).unwrap_or(0.0);
    let pos: Vec<V> = (0..n).map(|i| V::new(coord(xs, i), coord(ys, i))).collect();
    for (i, g) in geo.iter().enumerate() {
        let b = g.label_box;
        let hit = pos.iter().any(|&c| b.dist(c) < NODE_RADIUS + 2.0)
            || geo
                .iter()
                .enumerate()
                .any(|(j, o)| j != i && o.label_box.overlaps(b))
            || traces.iter().any(|t| t.edge != g.edge && t.hits_rect(b));
        count += usize::from(hit);
    }
    count
}

/// The geometry of `r` with its label beside `spot` on side `side`.
fn labelled(r: &Routed, spot: V, side: V) -> Geometry {
    let (w, h) = r.size;
    let (label, anchor) = place_label(spot, side, w, h);
    let x0 = anchor.left(label.x, w);
    Geometry {
        edge: r.edge,
        curve: r.curve,
        tip: r.curve.end(),
        tip_angle: r.tip_angle,
        label,
        anchor,
        label_box: Rect {
            x0,
            y0: label.y,
            x1: x0 + w,
            y1: label.y + h,
        },
    }
}

/// The anchor and alignment of a `w` × `h` block that lies beyond `apex` in direction `d`
/// (a unit vector), at least [`LABEL_GAP`] away from the line through `apex` normal to `d`.
///
/// The block grows away from the edge: centred above or below a nearly horizontal edge,
/// left- or right-aligned beside a nearly vertical one, and from its nearest corner on
/// diagonal ones. In the last two cases the position does not depend on the width, so a
/// label that renders wider than estimated still keeps clear of its edge.
fn place_label(apex: V, dir: V, width: f64, height: f64) -> (V, Anchor) {
    const NEARLY: f64 = 0.2;
    if dir.x.abs() <= NEARLY {
        // Push the centre out until the nearest corner is LABEL_GAP away.
        let reach = LABEL_GAP + width / 2.0 * dir.x.abs() + height / 2.0 * dir.y.abs();
        let centre = apex + dir * reach;
        return (V::new(centre.x, centre.y - height / 2.0), Anchor::Middle);
    }
    let anchor = if dir.x > 0.0 {
        Anchor::Start
    } else {
        Anchor::End
    };
    if dir.y.abs() <= NEARLY {
        // Both corners on the near side must be LABEL_GAP away.
        let x = apex.x + (LABEL_GAP * dir.x * dir.x + dir.y.abs() * height / 2.0) / dir.x;
        let middle = apex.y + dir.y * LABEL_GAP;
        (V::new(x, middle - height / 2.0), anchor)
    } else {
        let corner = apex + dir * LABEL_GAP;
        let top = if dir.y < 0.0 {
            corner.y - height
        } else {
            corner.y
        };
        (V::new(corner.x, top), anchor)
    }
}

/// Routes an edge between two different states.
fn between(
    edge: usize,
    from: usize,
    to: usize,
    pos: &[V],
    both_ways: bool,
    labels: &[String],
) -> Routed {
    let (source, target) = (pos[from], pos[to]);
    let len = source.dist(target);
    let along = (target - source).unit_or(V::new(1.0, 0.0));
    let offset = bend(from, to, pos, len, along, both_ways);
    let size = label_size(labels);
    if offset == 0.0 {
        Routed {
            edge,
            curve: Curve::Line(source + along * NODE_RADIUS, target - along * NODE_RADIUS),
            tip_angle: along.angle(),
            shape: Shape::Line,
            size,
        }
    } else {
        let control = (source + target) / 2.0 + along.left() * offset;
        let p0 = source + (control - source).unit_or(along) * NODE_RADIUS;
        let p1 = target + (control - target).unit_or(-along) * NODE_RADIUS;
        Routed {
            edge,
            curve: Curve::Quad(p0, control, p1),
            tip_angle: (target - control).angle(),
            shape: Shape::Arc {
                outer: offset.signum(),
            },
            size,
        }
    }
}

/// The signed offset of the control point of the edge from `from` to `to`, measured to the
/// left of the direction of travel, or 0 for a straight line.
fn bend(from: usize, to: usize, pos: &[V], len: f64, along: V, both_ways: bool) -> f64 {
    let left = along.left();
    let need = NODE_RADIUS + CLEARANCE;
    // On a quadratic arc with control offset h the point at distance s along the chord lies
    // k(t) h to the side, k(t) = 2t(1 - t), t = s / len. A state at (s, q) in chord
    // coordinates is cleared when |q - k(t) h| >= need, so it forbids an open interval of h.
    let mut blocked: Vec<(f64, f64)> = Vec::new();
    for (k, &other) in pos.iter().enumerate() {
        if k == from || k == to {
            continue;
        }
        let rel = other - pos[from];
        let distance = rel.dot(along);
        if distance <= 0.0 || distance >= len {
            continue;
        }
        let t = distance / len;
        let rise = 2.0 * t * (1.0 - t);
        let across = rel.dot(left);
        blocked.push(((across - need) / rise, (across + need) / rise));
    }
    let clear_line = !blocked.iter().any(|&(lo, hi)| lo < 0.0 && 0.0 < hi);
    if !both_ways && clear_line {
        return 0.0;
    }
    let base = if both_ways {
        (0.3 * len).clamp(28.0, 64.0)
    } else {
        24.0
    };
    let cap = (1.5 * len).max(160.0);
    let up = first_free(&blocked, base, 1.0);
    if both_ways {
        return verified(from, to, pos, along, &[up, up + 24.0, up + 48.0], base);
    }
    let down = first_free(&blocked, -base, -1.0);
    let mut candidates = if down.abs() + EPS < up.abs() {
        [down, up]
    } else {
        [up, down]
    };
    for h in &mut candidates {
        if h.abs() > cap {
            *h = cap.copysign(*h);
        }
    }
    verified(from, to, pos, along, &candidates, base)
}

/// The first `h` from `start` in direction `dir` (+1 or -1) outside every open interval.
fn first_free(blocked: &[(f64, f64)], start: f64, dir: f64) -> f64 {
    let mut h = start;
    loop {
        let mut moved = false;
        for &(lo, hi) in blocked {
            if lo < h && h < hi {
                h = if dir > 0.0 { hi } else { lo };
                moved = true;
            }
        }
        if !moved {
            return h;
        }
    }
}

/// The first candidate whose arc keeps clear of all other states, or the one that comes
/// closest. The interval test above looks at each state only straight across the chord, so
/// steep arcs are checked again by sampling.
fn verified(from: usize, to: usize, pos: &[V], u: V, candidates: &[f64], base: f64) -> f64 {
    let (a, b) = (pos[from], pos[to]);
    let left = u.left();
    let mid = (a + b) / 2.0;
    let mut best = (
        f64::NEG_INFINITY,
        candidates.first().copied().unwrap_or(base),
    );
    for &h in candidates {
        let curve = Curve::Quad(a, mid + left * h, b);
        let mut clearance = f64::INFINITY;
        for k in 1..16 {
            let p = curve.at(f64::from(k) / 16.0);
            for (j, &o) in pos.iter().enumerate() {
                if j != from && j != to {
                    clearance = clearance.min(p.dist(o) - NODE_RADIUS);
                }
            }
        }
        if clearance >= CLEARANCE * 0.5 {
            return h;
        }
        if clearance > best.0 {
            best = (clearance, h);
        }
    }
    best.1
}

/// The self-loop of state `node` in direction `phi`.
fn loop_curve(c: V, phi: f64) -> Curve {
    // The loop leaves and enters the rim SPREAD radians either side of `phi`; its control
    // points lie REACH beyond the rim, FAN radians either side.
    const SPREAD: f64 = 0.38;
    const FAN: f64 = 0.55;
    const REACH: f64 = 56.0;
    Curve::Cubic(
        c + V::polar(phi - SPREAD) * NODE_RADIUS,
        c + V::polar(phi - FAN) * (NODE_RADIUS + REACH),
        c + V::polar(phi + FAN) * (NODE_RADIUS + REACH),
        c + V::polar(phi + SPREAD) * NODE_RADIUS,
    )
}

/// Routes the self-loop of state `node`, whose rim is already used at `taken` angles, with
/// its label.
///
/// Straight up is the first choice, then straight down, right and left, as long as no edge
/// leaves the state within 72 degrees of that direction; then the middles of the gaps
/// between the edges, largest gap first. The first of these whose loop and label collide
/// least with what is drawn wins.
fn self_loop(
    scene: &Scene<'_>,
    edge: usize,
    node: usize,
    taken: &[f64],
    labels: &[String],
) -> (Routed, Geometry) {
    const ENOUGH: f64 = 1.25;
    const CRAMPED: f64 = 0.75;
    let c = scene.pos[node];
    let room = |phi: f64| {
        taken
            .iter()
            .map(|&a| angle_between(phi, a))
            .fold(PI, f64::min)
    };
    let mut directions: Vec<f64> = [-PI / 2.0, PI / 2.0, 0.0, PI]
        .into_iter()
        .filter(|&phi| room(phi) >= ENOUGH)
        .collect();
    let mut sorted: Vec<f64> = taken.iter().map(|a| a.rem_euclid(TAU)).collect();
    sorted.sort_by(f64::total_cmp);
    let mut gaps: Vec<(f64, f64)> = sorted
        .iter()
        .enumerate()
        .map(|(i, &a)| {
            let next = sorted.get(i + 1).copied().unwrap_or(sorted[0] + TAU);
            (next - a, f64::midpoint(a, next))
        })
        .collect();
    gaps.sort_by(|x, y| y.0.total_cmp(&x.0));
    directions.extend(gaps.iter().map(|g| g.1));
    if directions.is_empty() {
        directions.push(-PI / 2.0);
    }

    let mut best: Option<(usize, Routed, Geometry)> = None;
    for phi in directions {
        let out = V::polar(phi);
        let curve = loop_curve(c, phi);
        let r = Routed {
            edge,
            curve,
            tip_angle: curve.tangent(1.0).angle(),
            shape: Shape::Loop { out },
            size: label_size(labels),
        };
        let g = labelled(&r, curve.at(0.5), out);
        let trace = Trace::new(edge, curve);
        let states = scene
            .pos
            .iter()
            .enumerate()
            .filter(|&(k, &o)| {
                k != node && trace.points.iter().any(|&p| p.dist(o) < NODE_RADIUS + 3.0)
            })
            .count();
        let crossings = scene
            .traces
            .iter()
            .filter(|t| t.edge != edge && t.crosses(&trace))
            .count();
        let cramped = if room(phi) < CRAMPED { 20 } else { 0 };
        let cost =
            scene.box_cost(g.label_box, edge, usize::MAX) + 10 * states + 3 * crossings + cramped;
        if best.as_ref().is_none_or(|(c, _, _)| cost < *c) {
            let done = cost == 0;
            best = Some((cost, r, g));
            if done {
                break;
            }
        }
    }
    let (_, r, g) = best.unwrap_or_else(|| {
        let curve = loop_curve(c, -PI / 2.0);
        let r = Routed {
            edge,
            curve,
            tip_angle: curve.tangent(1.0).angle(),
            shape: Shape::Loop {
                out: V::new(0.0, -1.0),
            },
            size: label_size(labels),
        };
        let g = labelled(&r, curve.at(0.5), V::new(0.0, -1.0));
        (0, r, g)
    });
    (r, g)
}

/// The angle between two directions, 0 to π.
fn angle_between(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(TAU);
    d.min(TAU - d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Edge, Node};

    fn node(name: &str) -> Node {
        Node {
            name: name.into(),
            is_start: false,
            is_final: false,
        }
    }

    fn edge(from: usize, to: usize) -> Edge {
        Edge {
            from,
            to,
            labels: vec!["a ↦ b, R".into()],
            transitions: vec![0],
        }
    }

    fn graph(n: usize, edges: &[(usize, usize)]) -> Graph {
        Graph {
            nodes: (0..n).map(|i| node(&format!("q{i}"))).collect(),
            edges: edges.iter().map(|&(a, b)| edge(a, b)).collect(),
        }
    }

    #[test]
    fn numbers_are_short_and_never_negative_zero() {
        let mut out = String::new();
        for v in [0.0, -0.0, -0.001, 1.0, 12.345, -7.5, 1e-9, f64::NAN] {
            write_num(&mut out, v);
            out.push(' ');
        }
        assert_eq!(out, "0 0 0 1 12.35 -7.5 0 0 ");
    }

    #[test]
    fn a_free_edge_is_straight_from_rim_to_rim() {
        let g = graph(2, &[(0, 1)]);
        let r = route(&g, &[0.0, 140.0], &[0.0, 0.0]);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].path, "M 22 0 L 118 0");
        assert_eq!(r[0].tip, (118.0, 0.0));
        assert_eq!(r[0].tip_angle, 0.0);
        assert_eq!(r[0].label_anchor, "middle");
        // Above the edge.
        assert!(r[0].label.1 + LABEL_LINE_HEIGHT <= -LABEL_GAP + 1e-9);
    }

    #[test]
    fn opposite_edges_bend_to_opposite_sides() {
        let g = graph(2, &[(0, 1), (1, 0)]);
        let geo = geometry(&g, &[0.0, 140.0], &[0.0, 0.0]);
        let (Curve::Quad(_, c0, _), Curve::Quad(_, c1, _)) = (geo[0].curve, geo[1].curve) else {
            panic!("both edges should be arcs");
        };
        // Left of travel: the rightward edge bends up, the leftward edge bends down.
        assert!(c0.y < 0.0 && c1.y > 0.0);
        assert!(geo[0].label_box.y1 < 0.0 && geo[1].label_box.y0 > 0.0);
    }

    #[test]
    fn an_edge_bends_around_a_state_in_its_way() {
        let g = graph(3, &[(0, 2)]);
        let geo = geometry(&g, &[0.0, 140.0, 280.0], &[0.0, 0.0, 0.0]);
        let curve = geo[0].curve;
        assert!(matches!(curve, Curve::Quad(..)));
        let middle = V::new(140.0, 0.0);
        for k in 0..=32 {
            let p = curve.at(f64::from(k) / 32.0);
            assert!(p.dist(middle) >= NODE_RADIUS + CLEARANCE * 0.5, "{p:?}");
        }
    }

    #[test]
    fn a_loop_goes_up_when_there_is_room() {
        let g = graph(2, &[(0, 0), (0, 1)]);
        let geo = geometry(&g, &[0.0, 140.0], &[0.0, 0.0]);
        let apex = geo[0].curve.at(0.5);
        assert!(apex.y < -NODE_RADIUS && apex.x.abs() < 1e-6, "{apex:?}");
        assert_eq!(geo[0].anchor, Anchor::Middle);
    }

    #[test]
    fn a_loop_avoids_crowded_sides() {
        // Edges up, left and right of state 0, and the start arrow from the left.
        let mut g = graph(4, &[(0, 0), (0, 1), (0, 2), (0, 3)]);
        g.nodes[0].is_start = true;
        let geo = geometry(&g, &[0.0, 0.0, 140.0, -140.0], &[0.0, -140.0, 0.0, 0.0]);
        let apex = geo[0].curve.at(0.5);
        assert!(apex.y > NODE_RADIUS && apex.x.abs() < 1e-6, "{apex:?}");
        assert_eq!(geo[0].anchor, Anchor::Middle);
        assert!(geo[0].label_box.y0 > apex.y);
    }

    #[test]
    fn labels_beside_vertical_edges_grow_away() {
        let g = graph(2, &[(0, 1)]);
        let r = route(&g, &[0.0, 0.0], &[0.0, 140.0]);
        // Travelling down, the label goes right and is left-aligned.
        assert_eq!(r[0].label_anchor, "start");
        assert!(r[0].label.0 >= LABEL_GAP - 1e-9);
    }

    #[test]
    fn coincident_states_give_finite_routes() {
        let g = graph(2, &[(0, 1), (1, 0), (0, 0)]);
        for r in route(&g, &[5.0, 5.0], &[5.0, 5.0]) {
            assert!(!r.path.contains("NaN") && r.tip_angle.is_finite());
            assert!(r.label.0.is_finite() && r.label.1.is_finite());
        }
    }

    #[test]
    fn bad_indices_and_coordinates_are_tolerated() {
        let g = graph(2, &[(0, 1), (0, 5)]);
        let r = route(&g, &[f64::NAN], &[]);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].edge, 0);
    }
}
