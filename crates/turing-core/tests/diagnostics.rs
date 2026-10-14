//! Every diagnostic code, rendered against its source text and compared with a snapshot.

use std::fmt::Write as _;
use std::path::PathBuf;

use turing_core::dsl;
use turing_core::{Diagnostic, Dir, MachineSpec, Span, StateSpec, TransitionSpec};

/// Writes one source line with a marker under `span`, in the style of rustc.
fn render_span(out: &mut String, source: &str, span: Span, marker: &str, note: &str) {
    let (start, end) = (span.start as usize, span.end as usize);
    let line_start = source[..start].rfind('\n').map_or(0, |i| i + 1);
    let line_end = source[start..]
        .find('\n')
        .map_or(source.len(), |i| start + i);
    let line = source[..start].matches('\n').count() + 1;
    let column = source[line_start..start].chars().count();
    let width = source[start..end.min(line_end)].chars().count().max(1);
    let text = source[line_start..line_end].trim_end();
    writeln!(out, "  --> {line}:{}", column + 1).unwrap();
    writeln!(out, "{line:>3} | {text}").unwrap();
    let marks = marker.repeat(width);
    let gap = " ".repeat(column);
    let note = if note.is_empty() {
        String::new()
    } else {
        format!(" {note}")
    };
    writeln!(out, "    | {gap}{marks}{note}").unwrap();
}

/// All diagnostics of `source`, in order, with their spans drawn into the text.
fn render(source: &str, diagnostics: &[Diagnostic]) -> String {
    let mut out = String::new();
    for d in diagnostics {
        writeln!(out, "{d}").unwrap();
        if let Some(span) = d.span {
            render_span(&mut out, source, span, "^", "");
        }
        for (span, note) in &d.related {
            render_span(&mut out, source, *span, "-", note);
        }
        if let Some(help) = &d.help {
            writeln!(out, "    = help: {help}").unwrap();
        }
        out.push('\n');
    }
    out
}

/// Parses `source` and renders its diagnostics, which must have the codes `codes`.
fn check(source: &str, codes: &[&str]) -> String {
    let parsed = dsl::parse(source);
    let found: Vec<&str> = parsed.diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(found, codes, "{source}");
    assert_eq!(
        parsed.spec.is_some(),
        !parsed.diagnostics.iter().any(Diagnostic::is_error)
    );
    render(source, &parsed.diagnostics)
}

macro_rules! snapshot {
    ($name:ident, $source:expr, $codes:expr) => {
        #[test]
        fn $name() {
            insta::assert_snapshot!(check($source, &$codes));
        }
    };
}

snapshot!(e001_missing_comma, "δ(q0, x) = (q1, x R)", ["E001"]);
snapshot!(
    e001_unknown_declaration,
    "strat = q0\nδ(q0, x) = (q1, x, R)",
    ["E001"]
);
snapshot!(e001_not_delta, "d(q0, x) = (q1, x, R)", ["E001"]);
snapshot!(e001_symbol_too_long, "δ(q0, xy) = (q1, x, R)", ["E001"]);
snapshot!(e001_star_read, "δ(q0, *) = (q1, x, R)", ["E001"]);
snapshot!(e001_blank_as_state, "δ(_, x) = (q1, x, R)", ["E001"]);
snapshot!(
    e001_unknown_kind,
    "kind decider\nδ(q0, x) = (q1, x, R)",
    ["E001"]
);
snapshot!(e001_extra_tokens, "Q = {q0} q1", ["E001"]);
snapshot!(e001_bad_quote, "δ(q0, 'ab') = (q1, x, R)", ["E001"]);
snapshot!(
    e002_unterminated_string,
    "machine \"A_mul\ninput \"x\"\nδ(q0, x) = (q1, x, R)",
    ["E002"]
);
snapshot!(
    e003_unknown_state,
    "Q = {q0, q1}\nδ(q0, x) = (q1, x, R)\nδ(q1, x) = (q2, x, R)",
    ["E003"]
);
snapshot!(
    e003_unknown_symbol,
    "Γ = {x, ␣}\nδ(q0, X) = (q0, x, R)",
    ["E003"]
);
snapshot!(
    e003_unknown_final_state,
    "Q = {q0, q1}\nF = {qf}\nδ(q0, x) = (q1, x, R)",
    ["E003"]
);
snapshot!(
    e003_unknown_start_state,
    "Q = {q0, q1}\nstart = s\nδ(q0, x) = (q1, x, R)",
    ["E003"]
);
snapshot!(
    e003_input_outside_sigma,
    "Σ = {x}\ninput \"xyx\"\nδ(q0, x) = (q0, x, R)",
    ["E003"]
);
snapshot!(
    e004_duplicate_transition,
    "δ(q0, x) = (q1, x, R)\nδ(q0, x) = (q2, x, L)",
    ["E004"]
);
snapshot!(
    e004_duplicate_in_a_set,
    "δ(q0, {x, y, x}) = (q1, *, R)",
    ["E004"]
);
snapshot!(
    e005_no_states,
    "// nothing here\nmachine \"empty\"",
    ["E005"]
);
snapshot!(
    e006_blank_in_sigma,
    "Σ = {x, ␣}\nδ(q0, x) = (q0, x, R)",
    ["E006"]
);
snapshot!(
    e007_sigma_not_in_gamma,
    "Σ = {x, y}\nΓ = {x, ␣}\nδ(q0, x) = (q0, x, R)",
    ["E007"]
);
snapshot!(e008_invalid_direction, "δ(q0, x) = (q1, x, S)", ["E008"]);
snapshot!(e008_lower_case_direction, "δ(q0, x) = (q1, x, r)", ["E008"]);
snapshot!(
    e009_too_many_tapes,
    "tapes = 4\nδ(q0, x) = (q0, x, R)",
    ["E009"]
);
snapshot!(
    e009_huge_tape_count,
    "tapes = 300\nδ(q0, x) = (q0, x, R)",
    ["E009"]
);
snapshot!(
    e010_tuple_without_tapes,
    "δ(q0, (x, y)) = (q1, (x, y), (R, R))",
    ["E010"]
);
snapshot!(
    e010_entries_disagree,
    "tapes = 2\nδ(q0, (x, y)) = (q1, x, R)",
    ["E010"]
);
snapshot!(
    e011_declared_twice,
    "Q = {q0}\nQ = {q0, q1}\nδ(q0, x) = (q0, x, R)",
    ["E011"]
);
snapshot!(
    e011_listed_twice,
    "Q = {q0, q1, q0}\nF = {q1, q1}\nδ(q0, x) = (q1, x, R)",
    ["E011", "E011"]
);
snapshot!(
    w001_unreachable_state,
    "δ(q0, x) = (q1, x, R)\nδ(q2, x) = (q1, x, R)",
    ["W001"]
);
snapshot!(
    w001_unreachable_declared_state,
    "Q = {q0, q1, q9}\nδ(q0, x) = (q1, x, R)",
    ["W001"]
);
snapshot!(
    w002_acceptor_without_final_states,
    "kind acceptor\nδ(q0, a) = (q1, a, R)",
    ["W002"]
);
snapshot!(
    w002_enumerator_without_final_states,
    "kind enumerator\nδ(q0, ␣) = (q0, a, R)",
    ["W002"]
);
snapshot!(
    w003_unused_symbol,
    "Γ = {x, y, ␣}\nδ(q0, x) = (q0, x, R)",
    ["W003"]
);

/// Every line is checked, so all errors appear at once and in order.
#[test]
fn all_errors_at_once() {
    let source = "\
machine \"broken
kind decider
Q = {q0, q1}
Σ = {x, ␣}
δ(q0, x) = (q1, x, S)
δ(q0, x) = (q2, x, R)
δ(q0, x) = (q1, y, R)
δ(q1, x) = (q0, x, R)
δ(q1, x) = (q0, x, L)
";
    insta::assert_snapshot!(check(
        source,
        &["E002", "E001", "E006", "E008", "E003", "E004", "E004"]
    ));
}

/// Warnings are dropped when lines could not be read, because they may be wrong then.
#[test]
fn warnings_wait_for_a_complete_text() {
    let parsed = dsl::parse("Γ = {x, y, ␣}\nδ(q0, x) = (q0, x, R)\nδ(q0, y) = (q0, y, R");
    let codes: Vec<&str> = parsed.diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(codes, ["E001"]);
}

/// E012 can only arise from specs built in code, for example by an editor.
#[test]
fn e012_names_the_language_cannot_write() {
    let mut spec = MachineSpec::new();
    spec.declared.states = true;
    spec.states = ["q 1", "", "q0"]
        .map(|name| StateSpec {
            name: name.into(),
            is_final: false,
        })
        .to_vec();
    spec.transitions.push(TransitionSpec {
        from: "q0".into(),
        read: vec!['x'],
        to: "q0".into(),
        write: vec!['x'],
        dir: vec![Dir::R],
        ..TransitionSpec::default()
    });
    let diagnostics: Vec<_> = spec
        .check()
        .into_iter()
        .filter(Diagnostic::is_error)
        .collect();
    insta::assert_snapshot!(render("", &diagnostics));
    assert!(spec.compile().is_err());
}

/// Checks on a spec without source text fall back to the spans stored in its transitions.
#[test]
fn checks_without_source_use_transition_spans() {
    let parsed = dsl::parse("Q = {q0}\nδ(q0, x) = (q0, x, R)");
    let mut spec = parsed.spec.expect("valid");
    spec.transitions[0].to = "q1".into();
    let diagnostics = spec.check();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "E003");
    assert_eq!(diagnostics[0].span, spec.transitions[0].span);
}

#[test]
fn every_code_is_documented() {
    let docs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/dsl.md");
    let docs = std::fs::read_to_string(docs).expect("docs/dsl.md");
    let errors = (1..=12).map(|i| format!("E{i:03}"));
    let warnings = (1..=3).map(|i| format!("W{i:03}"));
    for code in errors.chain(warnings) {
        assert!(
            docs.contains(&format!("### {code}")),
            "{code} is not documented"
        );
    }
}
