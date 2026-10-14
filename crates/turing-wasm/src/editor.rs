//! Editor services: parsing, formatting, checking, the standard format and the state diagram.
//!
//! These run synchronously on the main thread, so every function here is linear (or close to
//! it) in its input and bounded for large inputs.

use turing_core::dsl::{self, FormatStyle};
use turing_core::standard::{self, StandardError};
use turing_render::{Graph, LayoutKind};

use crate::Error;
use crate::convert;
use crate::dto::{
    DiagnosticDto, DiagnosticsDto, EdgeRouteDto, EdgeRoutesDto, GraphEdgeDto, GraphLayoutDto,
    GraphNodeDto, MachineSpecDto, ParseResultDto, SeverityDto, TowerDto,
};
use crate::spans::{LineIndex, Positions};

/// The diagnostic code for text that is not a machine in the standard format.
pub(crate) const CODE_STANDARD: &str = "E101";

/// The largest diagram the layout accepts, in states.
pub(crate) const MAX_DIAGRAM_STATES: usize = turing_core::MAX_STATES;

/// The largest diagram the layout accepts, in transitions.
pub(crate) const MAX_DIAGRAM_TRANSITIONS: usize = 1 << 16;

/// The largest number of digits `upArrow` writes out.
pub(crate) const MAX_TOWER_DIGITS: u32 = 100_000;

/// Parses a description; diagnostics get UTF-16 positions, lines and columns.
pub(crate) fn parse_source(source: &str) -> ParseResultDto {
    let parsed = dsl::parse(source);
    let positions = Positions::source(source);
    ParseResultDto {
        spec: parsed.spec.map(|spec| convert::spec_dto(&spec, &positions)),
        diagnostics: parsed
            .diagnostics
            .iter()
            .map(|d| convert::diagnostic_dto(d, &positions))
            .collect(),
        source: None,
    }
}

/// Writes `spec` in canonical form.
///
/// # Errors
///
/// Fails if a glyph is not one character.
pub(crate) fn format_spec(dto: &MachineSpecDto, ascii: bool) -> Result<String, Error> {
    let spec = convert::spec(dto).map_err(|p| convert::first_error(&p))?;
    let style = if ascii {
        FormatStyle::Ascii
    } else {
        FormatStyle::Unicode
    };
    Ok(dsl::format(&spec, style))
}

/// Every error and warning of `spec`. Malformed glyphs are reported as `E102` errors instead of
/// being thrown, so the table editor can show them like any other problem. Spans are UTF-16
/// offsets taken over from the spec unchanged.
pub(crate) fn check_spec(dto: &MachineSpecDto) -> DiagnosticsDto {
    let diagnostics = match convert::spec(dto) {
        Ok(spec) => spec
            .check()
            .iter()
            .map(|d| convert::diagnostic_dto(d, &Positions::Utf16))
            .collect(),
        Err(problems) => problems
            .iter()
            .map(convert::Invalid::to_diagnostic)
            .collect(),
    };
    DiagnosticsDto { diagnostics }
}

/// Parses the standard format. Surrounding white space is ignored; errors are `E101`
/// diagnostics with positions in `text`.
pub(crate) fn from_standard(text: &str) -> ParseResultDto {
    let trimmed = text.trim();
    let offset = text.len() - text.trim_start().len();
    match standard::spec_from_standard(trimmed) {
        Ok(spec) => ParseResultDto {
            spec: Some(convert::spec_dto(&spec, &Positions::Utf16)),
            diagnostics: Vec::new(),
            source: None,
        },
        Err(error) => ParseResultDto {
            spec: None,
            diagnostics: vec![standard_diagnostic(text, offset, &error)],
            source: None,
        },
    }
}

fn standard_diagnostic(text: &str, offset: usize, error: &StandardError) -> DiagnosticDto {
    let mut diagnostic = DiagnosticDto {
        severity: SeverityDto::Error,
        code: CODE_STANDARD.into(),
        message: error.to_string(),
        help: Some("A machine in the standard format looks like `1RB1LB_1LA1RZ`.".into()),
        from: None,
        to: None,
        line: None,
        column: None,
        related: Vec::new(),
    };
    if let StandardError::Syntax { at, message } = error {
        let start = offset.saturating_add(*at).min(text.len());
        let end = text[start..]
            .chars()
            .next()
            .map_or(start, |c| start + c.len_utf8());
        let index = LineIndex::new(text);
        let (start, end) = (to_u32(start), to_u32(end));
        let (line, column) = index.line_column(start);
        diagnostic.message = format!("invalid standard format: {message}");
        diagnostic.from = Some(index.utf16(start));
        diagnostic.to = Some(index.utf16(end));
        diagnostic.line = Some(line);
        diagnostic.column = Some(column);
    }
    diagnostic
}

fn to_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// The standard format of `spec`, or `None` if the spec has errors or the format cannot express
/// the machine.
///
/// # Errors
///
/// Fails if a glyph is not one character.
pub(crate) fn to_standard(dto: &MachineSpecDto) -> Result<Option<String>, Error> {
    let spec = convert::spec(dto).map_err(|p| convert::first_error(&p))?;
    Ok(spec
        .compile()
        .ok()
        .and_then(|machine| standard::to_standard(&machine).ok()))
}

/// The diagram of a spec, after checking that it is small enough to lay out.
fn diagram(dto: &MachineSpecDto) -> Result<Graph, Error> {
    if dto.states.len() > MAX_DIAGRAM_STATES {
        return Err(Error::new(format!(
            "the diagram shows at most {MAX_DIAGRAM_STATES} states"
        )));
    }
    if dto.transitions.len() > MAX_DIAGRAM_TRANSITIONS {
        return Err(Error::new(format!(
            "the diagram shows at most {MAX_DIAGRAM_TRANSITIONS} transitions"
        )));
    }
    let spec = convert::spec(dto).map_err(|p| convert::first_error(&p))?;
    Ok(Graph::from_spec(&spec))
}

/// Places the states of `spec`.
///
/// # Errors
///
/// Fails for an unknown `kind`, a malformed glyph or a diagram beyond the limits.
pub(crate) fn graph_layout(dto: &MachineSpecDto, kind: &str) -> Result<GraphLayoutDto, Error> {
    let kind = match kind {
        "auto" => LayoutKind::Auto,
        "layered" => LayoutKind::Layered,
        "circular" => LayoutKind::Circular,
        other => {
            return Err(Error::new(format!(
                "unknown layout `{}`, expected `auto`, `layered` or `circular`",
                convert::shorten(other, 16)
            )));
        }
    };
    let graph = diagram(dto)?;
    let layout = turing_render::layout(&graph, kind);
    let nodes = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| GraphNodeDto {
            name: node.name.clone(),
            is_start: node.is_start,
            is_final: node.is_final,
            x: layout.x.get(i).copied().unwrap_or(0.0),
            y: layout.y.get(i).copied().unwrap_or(0.0),
        })
        .collect();
    let edges = graph
        .edges
        .into_iter()
        .map(|edge| GraphEdgeDto {
            from: edge.from,
            to: edge.to,
            labels: edge.labels,
            transitions: edge.transitions,
        })
        .collect();
    Ok(GraphLayoutDto {
        nodes,
        edges,
        width: layout.width,
        height: layout.height,
        radius: turing_render::layout::NODE_RADIUS,
    })
}

/// Routes the edges of `spec` for the node centres `xs` and `ys`.
///
/// # Errors
///
/// Fails if there is not exactly one finite coordinate pair per state, for a malformed glyph
/// or a diagram beyond the limits.
pub(crate) fn graph_route(
    dto: &MachineSpecDto,
    xs: &[f64],
    ys: &[f64],
) -> Result<EdgeRoutesDto, Error> {
    let graph = diagram(dto)?;
    let n = graph.nodes.len();
    if xs.len() != n || ys.len() != n {
        return Err(Error::new(format!(
            "expected {n} node positions, got {} x and {} y coordinates",
            xs.len(),
            ys.len()
        )));
    }
    if !xs.iter().chain(ys).all(|v| v.is_finite()) {
        return Err(Error::new("node positions must be finite numbers"));
    }
    let routes = turing_render::route(&graph, xs, ys)
        .into_iter()
        .map(|r| EdgeRouteDto {
            edge: r.edge,
            path: r.path,
            tip_x: r.tip.0,
            tip_y: r.tip.1,
            tip_angle: r.tip_angle,
            label_x: r.label.0,
            label_y: r.label.1,
            label_anchor: r.label_anchor.into(),
        })
        .collect();
    Ok(EdgeRoutesDto { routes })
}

/// `base ↑↑ height`, written out if it has at most `max_digits` digits; `max_digits` is capped
/// at [`MAX_TOWER_DIGITS`] so that a call stays fast.
pub(crate) fn up_arrow(base: u32, height: u32, max_digits: u32) -> TowerDto {
    let max = usize::try_from(max_digits.min(MAX_TOWER_DIGITS)).unwrap_or(0);
    let tower = turing_bb::uparrow::tetration(base, height, max);
    TowerDto {
        base: tower.base,
        height: tower.height,
        exact: tower.exact,
        digits: tower.digits.map(|d| d.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{DirDto, SpanDto};
    use crate::testing;

    const SOURCE: &str = "// Binary increment, least significant bit first\n\
machine \"increment\"\n\
kind transducer\n\
input \"1101\"\n\
Q = {q0, q1}\n\
Σ = {0, 1}\n\
Γ = {0, 1, ␣}\n\
start = q0\n\
F = {q1}\n\
\n\
δ(q0, 1) = (q0, 0, R) // carry\n\
δ(q0, 0) = (q1, 1, R)\n\
δ(q0, ␣) = (q1, 1, R)\n";

    /// The UTF-16 offset of byte offset `byte` of `source`, as JavaScript counts.
    fn js(source: &str, byte: usize) -> u32 {
        u32::try_from(source[..byte].encode_utf16().count()).unwrap()
    }

    /// The text of a UTF-16 range of `source`.
    fn slice16(source: &str, from: u32, to: u32) -> String {
        let units: Vec<u16> = source.encode_utf16().collect();
        String::from_utf16(&units[from as usize..to as usize]).unwrap()
    }

    fn without_spans(mut spec: MachineSpecDto) -> MachineSpecDto {
        for t in &mut spec.transitions {
            t.span = None;
        }
        spec
    }

    #[test]
    fn parses_course_notation_with_utf16_spans() {
        let result = parse_source(SOURCE);
        assert_eq!(result.diagnostics, []);
        let spec = result.spec.expect("valid source");
        assert_eq!(spec.blank, "␣");
        assert_eq!(spec.input_alphabet, ["0", "1"]);
        assert_eq!(spec.transitions.len(), 3);
        assert_eq!(spec.transitions[2].read, ["␣"]);
        assert_eq!(spec.transitions[0].comment.as_deref(), Some("carry"));
        for (t, line) in spec.transitions.iter().zip([
            "δ(q0, 1) = (q0, 0, R)",
            "δ(q0, 0) = (q1, 1, R)",
            "δ(q0, ␣) = (q1, 1, R)",
        ]) {
            let span = t.span.expect("parsed transitions have spans");
            assert_eq!(span.from, js(SOURCE, SOURCE.find(line).unwrap()));
            assert!(slice16(SOURCE, span.from, span.to).starts_with("δ(q0"));
        }
    }

    #[test]
    fn format_round_trips() {
        let spec = parse_source(SOURCE).spec.expect("valid source");
        for ascii in [false, true] {
            let text = format_spec(&spec, ascii).unwrap();
            assert_eq!(text.contains('δ'), !ascii, "{text}");
            let again = parse_source(&text).spec.expect("formatted text parses");
            assert_eq!(without_spans(again), without_spans(spec.clone()));
        }
        let unicode = format_spec(&spec, false).unwrap();
        assert_eq!(
            format_spec(&parse_source(&unicode).spec.unwrap(), false).unwrap(),
            unicode
        );
    }

    #[test]
    fn diagnostics_have_consistent_utf16_positions() {
        let source = "Q = {q0, q1}\nΣ = {x}\nδ(q0, ␣) = (q9, x, R)\nδ(q0, ␣) = (q1, x, R)\n";
        let result = parse_source(source);
        assert!(result.spec.is_none());
        let line_starts: Vec<u32> = std::iter::once(0)
            .chain(source.match_indices('\n').map(|(i, _)| js(source, i + 1)))
            .collect();
        let errors: Vec<&DiagnosticDto> = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == SeverityDto::Error)
            .collect();
        assert_ne!(errors.len(), 0);
        for d in &result.diagnostics {
            let (Some(from), Some(to), Some(line), Some(column)) = (d.from, d.to, d.line, d.column)
            else {
                continue;
            };
            assert!(from <= to);
            assert_eq!(line_starts[line as usize - 1] + column - 1, from, "{d:?}");
        }
        // The unknown state is reported in the third line.
        let unknown = errors
            .iter()
            .find(|d| d.message.contains("q9"))
            .expect("an error about q9");
        assert_eq!(unknown.line, Some(3));
        let (from, to) = (unknown.from.unwrap(), unknown.to.unwrap());
        assert!(slice16(source, from, to).contains("q9"));
    }

    #[test]
    fn check_reports_malformed_glyphs() {
        let mut spec = testing::increment();
        spec.transitions[1].write = vec!["11".into()];
        spec.transitions[1].span = Some(SpanDto { from: 7, to: 9 });
        let result = check_spec(&spec);
        assert_eq!(result.diagnostics.len(), 1);
        let d = &result.diagnostics[0];
        assert_eq!(d.code, convert::CODE_GLYPH);
        assert_eq!((d.from, d.to), (Some(7), Some(9)));
        assert!(format_spec(&spec, false).is_err());
        assert!(to_standard(&spec).is_err());
    }

    #[test]
    fn check_passes_spans_through() {
        let mut spec = testing::increment();
        spec.declared.states = true;
        spec.states.truncate(1);
        spec.transitions[1].span = Some(SpanDto { from: 100, to: 121 });
        let result = check_spec(&spec);
        let error = result
            .diagnostics
            .iter()
            .find(|d| d.severity == SeverityDto::Error)
            .expect("q1 is not declared");
        if let Some(from) = error.from {
            assert!((100..=121).contains(&from) || error.related.iter().any(|r| r.from >= 100));
        }
        assert_eq!(error.line, None);
    }

    #[test]
    fn standard_format_round_trips() {
        for text in [testing::BB4, testing::BB5, "1RB1LB_1LA1RZ", "1RB---_1LA---"] {
            let result = from_standard(text);
            assert_eq!(result.diagnostics, []);
            let spec = result.spec.expect("valid standard format");
            assert_eq!(spec.kind, crate::dto::KindDto::Beaver);
            assert_eq!(to_standard(&spec).unwrap().as_deref(), Some(text));
        }
        let padded = from_standard(&format!("  {}\n", testing::BB4));
        assert!(padded.spec.is_some());
    }

    #[test]
    fn standard_format_errors_point_into_the_text() {
        let text = "\n  1RB1LB_1LA1R?";
        let result = from_standard(text);
        assert!(result.spec.is_none());
        let d = &result.diagnostics[0];
        assert_eq!(d.code, CODE_STANDARD);
        if let (Some(from), Some(line), Some(column)) = (d.from, d.line, d.column) {
            assert_eq!(line, 2);
            assert_eq!(from, column - 1 + 1);
            assert!(from as usize <= text.len());
        }
        assert_eq!(from_standard("").spec, None);
    }

    #[test]
    fn to_standard_of_course_machines() {
        // A machine with an N move cannot be written in the standard format.
        let mut spec = testing::increment();
        spec.transitions[0].dir = vec![DirDto::N];
        assert_eq!(to_standard(&spec).unwrap(), None);
        // A spec with errors neither.
        let mut broken = testing::increment();
        broken.tapes = 0;
        assert_eq!(to_standard(&broken).unwrap(), None);
    }

    #[test]
    fn layout_and_routes() {
        let spec = testing::increment();
        for kind in ["auto", "layered", "circular"] {
            let layout = graph_layout(&spec, kind).unwrap();
            assert_eq!(layout.nodes.len(), 2);
            assert!(layout.nodes[0].is_start);
            assert!(layout.nodes[1].is_final);
            assert_eq!(layout.edges.len(), 2);
            assert!(layout.width > 0.0 && layout.height > 0.0);
            let xs: Vec<f64> = layout.nodes.iter().map(|n| n.x).collect();
            let ys: Vec<f64> = layout.nodes.iter().map(|n| n.y).collect();
            let routes = graph_route(&spec, &xs, &ys).unwrap();
            assert_eq!(routes.routes.len(), layout.edges.len());
            assert!(routes.routes.iter().all(|r| r.path.starts_with('M')));
        }
        assert!(graph_layout(&spec, "spiral").is_err());
        assert!(graph_route(&spec, &[0.0], &[0.0, 1.0]).is_err());
        assert!(graph_route(&spec, &[0.0, f64::NAN], &[0.0, 1.0]).is_err());
        assert!(graph_route(&spec, &[], &[]).is_err());
    }

    #[test]
    fn towers() {
        assert_eq!(up_arrow(2, 4, 100).exact.as_deref(), Some("65536"));
        let t = up_arrow(2, 5, 100);
        assert_eq!(t.exact, None);
        assert_eq!(t.digits.as_deref(), Some("19729"));
        let t = up_arrow(2, 5, u32::MAX);
        assert_eq!(t.exact.map(|e| e.len()), Some(19_729));
        let t = up_arrow(3, 4, u32::MAX);
        assert_eq!(t.exact, None);
    }
}
