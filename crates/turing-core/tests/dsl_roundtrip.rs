//! The description language: parsing, inference, comments, formatting and round trips.

use std::fs;
use std::path::PathBuf;

use proptest::prelude::*;
use proptest::sample::Index;
use turing_core::dsl::{self, FormatStyle};
use turing_core::{
    BitOrder, Declared, Decoder, Dir, Kind, Machine, MachineSpec, SpecComments, StateSpec,
    TransitionSpec,
};

const STYLES: [FormatStyle; 2] = [FormatStyle::Unicode, FormatStyle::Ascii];

/// The spec without source spans, which formatting cannot keep.
fn without_spans(mut spec: MachineSpec) -> MachineSpec {
    for t in &mut spec.transitions {
        t.span = None;
    }
    spec
}

fn parse_ok(source: &str) -> MachineSpec {
    let parsed = dsl::parse(source);
    assert!(
        parsed.diagnostics.is_empty(),
        "{source}\n{:#?}",
        parsed.diagnostics
    );
    parsed.spec.expect("a spec")
}

/// Whether two machines behave the same: equal apart from the order of their transitions.
fn assert_same_machine(a: &Machine, b: &Machine) {
    assert_eq!(a.name(), b.name());
    assert_eq!(a.kind(), b.kind());
    assert_eq!(a.tapes(), b.tapes());
    assert_eq!(a.alphabet(), b.alphabet());
    assert_eq!(a.states(), b.states());
    assert_eq!(a.start(), b.start());
    assert_eq!(a.decoder(), b.decoder());
    assert_eq!(a.default_input(), b.default_input());
    assert!(a.transitions().eq(b.transitions()));
}

/// Formats `spec` in both styles and checks that parsing gives it back.
fn assert_round_trip(spec: &MachineSpec) {
    let expected = without_spans(spec.clone());
    for style in STYLES {
        let text = dsl::format(spec, style);
        let parsed = dsl::parse(&text);
        let errors: Vec<_> = parsed.diagnostics.iter().filter(|d| d.is_error()).collect();
        assert!(errors.is_empty(), "{text}\n{errors:#?}");
        let again = without_spans(parsed.spec.expect("a spec"));
        assert_eq!(again, expected, "{text}");
        assert_eq!(dsl::format(&again, style), text, "formatting is idempotent");
    }
}

fn transition(from: &str, read: &[char], to: &str, write: &[char], dir: &[Dir]) -> TransitionSpec {
    TransitionSpec {
        from: from.into(),
        read: read.to_vec(),
        to: to.into(),
        write: write.to_vec(),
        dir: dir.to_vec(),
        ..TransitionSpec::default()
    }
}

fn gallery_sources() -> Vec<(String, String)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../machines");
    let mut sources = Vec::new();
    for dir in ["course", "classic"] {
        for entry in fs::read_dir(root.join(dir)).expect("a directory") {
            let path = entry.expect("an entry").path();
            let text = fs::read_to_string(&path).expect("a file");
            sources.push((path.display().to_string(), text));
        }
    }
    sources.sort();
    sources
}

#[test]
fn gallery_machines_round_trip() {
    for (path, source) in gallery_sources() {
        let spec = parse_ok(&source);
        assert_round_trip(&spec);
        let machine = spec.compile().expect(&path);
        // The canonical spec of the machine describes the same machine.
        let canonical = MachineSpec::from(&machine);
        assert_same_machine(&canonical.compile().expect(&path), &machine);
        assert_round_trip(&canonical);
    }
}

#[test]
fn the_canonical_layout() {
    let source = "\
// A machine with every kind of line.
machine \"layout\"
kind acceptor
tapes = 2
input \"ab\"

δ(q0, (a, ␣)) = (q0, (a, a), (R, R)) // copy
δ(q0, (b, ␣)) = (q0, (b, b), (R, R))
// at the end
δ(q0, (␣, ␣)) = (accept, (␣, ␣), (L, L)) // done
δ(accept, ('*', ␣)) = (accept, (␣, '''), (N, N))
F = {accept}
// the end
";
    let spec = parse_ok(source);
    insta::assert_snapshot!(dsl::format(&spec, FormatStyle::Unicode));
    insta::assert_snapshot!("ascii", dsl::format(&spec, FormatStyle::Ascii));
}

#[test]
fn read_sets_and_star_expand_with_the_first_tape_outermost() {
    let spec = parse_ok("tapes = 2\nδ(q, ({a, b}, {c, d})) = (p, (*, x), (R, L))");
    let pairs: Vec<(String, String)> = spec
        .transitions
        .iter()
        .map(|t| (t.read.iter().collect(), t.write.iter().collect()))
        .collect();
    let expected = [("ac", "ax"), ("ad", "ax"), ("bc", "bx"), ("bd", "bx")];
    assert_eq!(pairs, expected.map(|(r, w)| (r.to_string(), w.to_string())));
    assert!(spec.transitions.iter().all(|t| t.dir == [Dir::R, Dir::L]));
}

#[test]
fn underscore_and_the_blank_sign_mean_the_blank() {
    let spec = parse_ok("blank = 0\nδ(A, _) = (A, ␣, R)\nδ(A, '_') = (A, '␣', L)");
    assert_eq!(spec.transitions[0].read, ['0']);
    assert_eq!(spec.transitions[0].write, ['0']);
    assert_eq!(spec.transitions[1].read, ['_']);
    assert_eq!(spec.transitions[1].write, ['␣']);
    assert_eq!(spec.tape_alphabet, ['0', '_', '␣']);
    assert_round_trip(&spec);

    let spec = parse_ok("blank = '_'\nδ(A, _) = (A, x, R)");
    assert_eq!(spec.blank, '_');
    assert_eq!(spec.transitions[0].read, ['_']);
    assert_round_trip(&spec);
}

#[test]
fn ascii_keywords_are_synonyms() {
    let ascii = parse_ok("Sigma = {a}\nGamma = {a, _}\ndelta(q0, a) = (q1, _, R)");
    let unicode = parse_ok("Σ = {a}\nΓ = {a, ␣}\nδ(q0, a) = (q1, ␣, R)");
    assert_eq!(without_spans(ascii), without_spans(unicode));
}

#[test]
fn undeclared_sets_are_inferred() {
    let spec = parse_ok(
        "input \"ab\"\nstart = s\nF = {f}\nδ(q0, x) = (q1, X, R)\nδ(s, ␣) = (q0, y, L)\nδ(q1, x) = (f, x, R)",
    );
    let names: Vec<&str> = spec.states.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["s", "q0", "q1", "f"]);
    assert!(spec.states[3].is_final);
    assert_eq!(spec.start, "s");
    assert_eq!(spec.tape_alphabet, ['␣', 'x', 'X', 'y', 'a', 'b']);
    assert_eq!(spec.input_alphabet, ['x', 'X', 'y', 'a', 'b']);
    assert_eq!(
        spec.declared,
        Declared {
            start: true,
            ..Declared::default()
        }
    );

    // A declared Γ without the blank gains it; Σ is then Γ without the blank.
    let spec = parse_ok("Γ = {a, b}\nδ(q0, a) = (q0, b, R)");
    assert_eq!(spec.tape_alphabet, ['a', 'b', '␣']);
    assert_eq!(spec.input_alphabet, ['a', 'b']);
    assert_eq!(spec.start, "q0");
}

#[test]
fn comments_are_attached_where_they_belong() {
    let source = "\
// header 1

// header 2
machine \"m\" // about the name
// before Q
Q = {q0, q1}

// leading of the first transition
δ(q0, a) = (q1, a, R) // trailing
//
//    indented
δ(q1, {a, b}) = (q1, *, R)
// last words
";
    let spec = parse_ok(source);
    assert_eq!(
        spec.comments,
        SpecComments {
            header: ["header 1", "header 2", "about the name", "before Q"]
                .map(String::from)
                .to_vec(),
            trailing: vec!["last words".into()],
        }
    );
    let t = &spec.transitions;
    assert_eq!(t[0].leading, ["leading of the first transition"]);
    assert_eq!(t[0].comment.as_deref(), Some("trailing"));
    assert_eq!(t[1].leading, ["", "   indented"]);
    assert!(t[2].leading.is_empty() && t[2].comment.is_none());
    assert_round_trip(&spec);
}

#[test]
fn comments_above_a_first_transition() {
    // Directly above: leading. Separated by a blank line: header.
    let spec = parse_ok("// a\n\n// b\nδ(q0, a) = (q0, a, R)");
    assert_eq!(spec.comments.header, ["a"]);
    assert_eq!(spec.transitions[0].leading, ["b"]);
    assert_round_trip(&spec);

    let spec = parse_ok("// a\n\nδ(q0, a) = (q0, a, R)");
    assert_eq!(spec.comments.header, ["a"]);
    assert!(spec.transitions[0].leading.is_empty());
    assert_round_trip(&spec);
}

#[test]
fn strings_keep_escapes() {
    let spec = parse_ok(
        r#"machine "say \"hi\" \\ there\n"
input "a\tb"
δ(q0, a) = (q0, b, R)"#,
    );
    assert_eq!(spec.name, "say \"hi\" \\ there\n");
    assert_eq!(spec.input, "a\tb");
    assert_round_trip(&spec);
}

#[test]
fn windows_line_endings() {
    let spec = parse_ok("Q = {q0}\r\nδ(q0, a) = (q0, a, R) // c\r\n");
    assert_eq!(spec.transitions[0].comment.as_deref(), Some("c"));
}

#[test]
fn the_table_limit_matches_the_builder() {
    let spec = |states: usize| {
        let mut spec = MachineSpec::new();
        spec.tapes = 2;
        spec.declared.states = true;
        spec.declared.tape_alphabet = true;
        spec.states = (0..states)
            .map(|i| StateSpec {
                name: format!("q{i}"),
                is_final: false,
            })
            .collect();
        spec.tape_alphabet = ('A'..='Z')
            .chain('a'..='z')
            .chain('0'..='9')
            .chain(['#', '␣'])
            .collect();
        assert_eq!(spec.tape_alphabet.len(), 64);
        spec
    };
    // 1024 × 64² = 2²² entries is the largest table.
    let largest = spec(1024);
    assert!(!largest.check().iter().any(|d| d.is_error()));
    assert!(largest.compile().is_ok());
    let too_large = spec(1025);
    let codes: Vec<_> = too_large
        .check()
        .into_iter()
        .filter(|d| d.is_error())
        .map(|d| d.code)
        .collect();
    assert_eq!(codes, ["E009"]);
    assert!(too_large.compile().is_err());
}

#[test]
fn compile_keeps_the_transition_order_for_origins() {
    let spec = parse_ok("δ(q1, b) = (q0, b, L)\nδ(q0, a) = (q1, a, R)\nδ(q0, b) = (q0, a, R)");
    let machine = spec.compile().expect("compiles");
    for (i, t) in spec.transitions.iter().enumerate() {
        let q = machine.state_by_name(&t.from).expect("a state");
        let a = machine.alphabet().symbol(t.read[0]).expect("a symbol");
        assert_eq!(machine.origin(q, &[a]), Some(i));
    }
}

#[test]
fn the_canonical_spec_of_a_machine() {
    let spec = parse_ok("δ(q1, b) = (q0, b, L)\nδ(q0, a) = (q1, a, R)\nδ(q0, ␣) = (q0, a, R)");
    let canonical = MachineSpec::from(&spec.compile().expect("compiles"));
    let order: Vec<(&str, char)> = canonical
        .transitions
        .iter()
        .map(|t| (t.from.as_str(), t.read[0]))
        .collect();
    assert_eq!(order, [("q1", 'b'), ("q0", '␣'), ("q0", 'a')]);
    assert_eq!(
        canonical.declared,
        Declared {
            states: true,
            input_alphabet: true,
            tape_alphabet: true,
            start: true,
            blank: false,
        }
    );
    assert_eq!(canonical.tape_alphabet, ['␣', 'b', 'a']);
}

// Property tests.

/// Glyphs that exercise quoting: punctuation of the grammar, quotes, spaces, letters, digits.
const GLYPHS: [char; 22] = [
    'x', 'X', '#', '0', '1', 'a', 'Ä', '+', '*', '\'', '"', ' ', '/', ';', '_', '␣', '(', ',', '=',
    '{', 'δ', '\t',
];

/// State names, including words that are keywords at the start of a line.
const NAMES: [&str; 12] = [
    "q0", "q1", "q_2", "q'", "qacc", "A", "F", "Q", "start", "delta", "L", "Ω",
];

fn comment() -> impl Strategy<Value = String> {
    "[ a-zA-Z0-9,.;:()δ#*/'\"_-]{0,24}".prop_map(|s| s.trim_end().to_string())
}

fn text() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 \"\\\\\n\tδ␣_]{0,12}"
}

/// The raw choices behind one transition.
type RawTransition = (
    Index,
    [Index; 3],
    Index,
    [Option<Index>; 3],
    [u8; 3],
    Option<String>,
    Vec<String>,
);

fn raw_transition() -> impl Strategy<Value = RawTransition> {
    (
        any::<Index>(),
        any::<[Index; 3]>(),
        any::<Index>(),
        any::<[Option<Index>; 3]>(),
        [0..3_u8, 0..3_u8, 0..3_u8],
        prop::option::of(comment()),
        prop::collection::vec(comment(), 0..3),
    )
}

/// Valid specs in the form the parser produces: undeclared sets are inferred.
fn valid_spec() -> impl Strategy<Value = MachineSpec> {
    let alphabet = (
        any::<[bool; 5]>(),
        any::<Index>(),
        prop::sample::subsequence(GLYPHS.to_vec(), 0..5).prop_shuffle(),
        any::<Index>(),
        any::<u8>(),
        prop::collection::vec(any::<Index>(), 0..4),
    );
    let states = (
        prop::sample::subsequence(NAMES.to_vec(), 1..6).prop_shuffle(),
        any::<u8>(),
        any::<Index>(),
        prop_oneof![4 => Just(1_u8), 1 => Just(2_u8), 1 => Just(3_u8)],
        prop::collection::vec(raw_transition(), 0..10),
    );
    let extras = (
        text(),
        0..4_u8,
        (0..3_u8, any::<Index>(), any::<bool>()),
        prop::collection::vec(comment(), 0..3),
        prop::collection::vec(comment(), 0..3),
    );
    (alphabet, states, extras).prop_map(build_spec)
}

#[expect(clippy::needless_pass_by_value, reason = "the shape proptest produces")]
fn build_spec(
    (alphabet, states, extras): (
        ([bool; 5], Index, Vec<char>, Index, u8, Vec<Index>),
        (Vec<&str>, u8, Index, u8, Vec<RawTransition>),
        (String, u8, (u8, Index, bool), Vec<String>, Vec<String>),
    ),
) -> MachineSpec {
    let (flags, blank_choice, gamma, blank_position, sigma_mask, input) = alphabet;
    let (names, final_mask, start, tapes, raw) = states;
    let (name, kind, (output, unary, lsb), header, trailing) = extras;
    let declared = Declared {
        states: flags[0],
        input_alphabet: flags[1],
        tape_alphabet: flags[2],
        start: flags[3],
        blank: flags[4],
    };
    let blank = if declared.blank {
        *blank_choice.get(&GLYPHS)
    } else {
        '␣'
    };
    let gamma: Vec<char> = gamma.into_iter().filter(|&g| g != blank).collect();
    let symbols: Vec<char> = std::iter::once(blank)
        .chain(gamma.iter().copied())
        .collect();
    let sigma: Vec<char> = gamma
        .iter()
        .enumerate()
        .filter(|&(i, _)| sigma_mask & (1 << i) != 0)
        .map(|(_, &g)| g)
        .collect();
    let tapes_usize = usize::from(tapes);

    let mut transitions: Vec<TransitionSpec> = Vec::new();
    for (from, read, to, write, dir, comment, leading) in raw {
        let read: Vec<char> = read[..tapes_usize]
            .iter()
            .map(|i| *i.get(&symbols))
            .collect();
        let from = *from.get(&names);
        if transitions.iter().any(|t| t.from == from && t.read == read) {
            continue;
        }
        let write = write[..tapes_usize]
            .iter()
            .zip(&read)
            .map(|(w, &r)| w.map_or(r, |i| *i.get(&symbols)))
            .collect();
        let dir = dir[..tapes_usize]
            .iter()
            .map(|&d| [Dir::L, Dir::R, Dir::N][usize::from(d)])
            .collect();
        transitions.push(TransitionSpec {
            from: from.into(),
            read,
            to: (*to.get(&names)).into(),
            write,
            dir,
            comment,
            leading,
            span: None,
        });
    }

    let start = if declared.start {
        *start.get(&names)
    } else {
        ""
    };
    let mut states: Vec<StateSpec> = names
        .iter()
        .enumerate()
        .map(|(i, &name)| StateSpec {
            name: name.into(),
            is_final: final_mask & (1 << i) != 0,
        })
        .collect();
    if !declared.states {
        let mentioned = |name: &str| {
            name == start || transitions.iter().any(|t| t.from == name || t.to == name)
        };
        states.retain(|s| s.is_final || mentioned(&s.name));
        if states.is_empty() {
            states.push(StateSpec {
                name: names[0].into(),
                is_final: true,
            });
        }
    }

    // The input word uses glyphs that are valid in Σ (or the blank).
    let input_glyphs: Vec<char> = if declared.input_alphabet {
        std::iter::once(blank)
            .chain(sigma.iter().copied())
            .collect()
    } else {
        symbols.clone()
    };
    let input: String = input.iter().map(|i| *i.get(&input_glyphs)).collect();

    let mut tape_alphabet = Vec::new();
    if declared.tape_alphabet {
        tape_alphabet = gamma.clone();
        let at = blank_position.index(tape_alphabet.len() + 1);
        tape_alphabet.insert(at, blank);
    }
    let mut spec = MachineSpec {
        name,
        kind: [
            Kind::Transducer,
            Kind::Acceptor,
            Kind::Enumerator,
            Kind::Beaver,
        ][usize::from(kind)],
        tapes,
        blank,
        input_alphabet: if declared.input_alphabet {
            sigma
        } else {
            Vec::new()
        },
        tape_alphabet,
        states,
        start: start.into(),
        transitions,
        input,
        output: match output {
            0 => Decoder::Word,
            1 => Decoder::Unary {
                symbol: *unary.get(&GLYPHS),
            },
            _ => Decoder::Binary {
                order: if lsb { BitOrder::Lsb } else { BitOrder::Msb },
            },
        },
        declared,
        comments: SpecComments { header, trailing },
    };
    spec.infer();
    spec
}

/// Text made of the language's tokens, so that the parser sees nearly valid input.
fn token_soup() -> impl Strategy<Value = String> {
    let pieces = prop::sample::select(vec![
        "δ",
        "delta",
        "(",
        ")",
        "{",
        "}",
        ",",
        " = ",
        "=",
        "*",
        "'",
        "'x'",
        "\"",
        "\"s\"",
        "//",
        "// c",
        "\n",
        "\r\n",
        " ",
        "q0",
        "q1",
        "x",
        "_",
        "␣",
        "#",
        "L",
        "R",
        "N",
        "S",
        "Q",
        "Σ",
        "Γ",
        "F",
        "start",
        "blank",
        "tapes",
        "machine",
        "kind",
        "acceptor",
        "input",
        "output",
        "unary",
        "binary",
        "lsb",
        "12",
        "0",
        "3",
        "99999999999999999999999",
        "δ(q0, x) = (q1, y, R)\n",
        "δ(q, ({a, b}, *)) = (p, (*, *), (L, R))\n",
        "tapes = 2\n",
    ]);
    prop::collection::vec(pieces, 0..40).prop_map(|p| p.concat())
}

/// Every span lies within `source` and on character boundaries.
fn assert_spans_valid(source: &str) {
    let parsed = dsl::parse(source);
    for d in &parsed.diagnostics {
        let spans = d.span.into_iter().chain(d.related.iter().map(|r| r.0));
        for span in spans {
            let (start, end) = (span.start as usize, span.end as usize);
            assert!(start <= end && end <= source.len(), "{d:?}");
            assert!(
                source.is_char_boundary(start) && source.is_char_boundary(end),
                "{d:?}"
            );
        }
    }
    assert_eq!(
        parsed.spec.is_some(),
        !parsed.diagnostics.iter().any(|d| d.is_error())
    );
}

/// Specs that may be invalid in every way, to compare the checks with the builder.
fn any_spec() -> impl Strategy<Value = MachineSpec> {
    let names = prop::sample::select(vec!["q0", "q1", "q2", "", "_", "q 1", "1q", "A"]);
    let glyph = prop::sample::select(vec!['a', 'b', '␣', '0', '\n']);
    let raw = (
        names.clone(),
        prop::collection::vec(glyph.clone(), 0..4),
        names.clone(),
        prop::collection::vec(glyph.clone(), 0..4),
        prop::collection::vec(prop::sample::select(vec![Dir::L, Dir::R, Dir::N]), 0..4),
    );
    (
        any::<[bool; 5]>(),
        0..5_u8,
        glyph.clone(),
        prop::collection::vec(glyph.clone(), 0..4),
        prop::collection::vec(glyph, 0..4),
        prop::collection::vec((names.clone(), any::<bool>()), 0..4),
        names,
        prop::collection::vec(raw, 0..6),
        "[ab␣0]{0,3}",
    )
        .prop_map(
            |(flags, tapes, blank, sigma, gamma, states, start, raw, input)| {
                let mut spec = MachineSpec {
                    tapes,
                    blank,
                    input_alphabet: sigma,
                    tape_alphabet: gamma,
                    states: states
                        .into_iter()
                        .map(|(name, is_final)| StateSpec {
                            name: name.into(),
                            is_final,
                        })
                        .collect(),
                    start: start.into(),
                    input,
                    declared: Declared {
                        states: flags[0],
                        input_alphabet: flags[1],
                        tape_alphabet: flags[2],
                        start: flags[3],
                        blank: flags[4],
                    },
                    ..MachineSpec::new()
                };
                spec.transitions = raw
                    .into_iter()
                    .map(|(from, read, to, write, dir)| transition(from, &read, to, &write, &dir))
                    .collect();
                spec
            },
        )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn formatting_round_trips(spec in valid_spec()) {
        assert_round_trip(&spec);
    }

    #[test]
    fn valid_specs_compile_and_survive_canonicalisation(spec in valid_spec()) {
        let machine = spec.compile().expect("a valid spec compiles");
        let canonical = MachineSpec::from(&machine);
        assert_same_machine(&canonical.compile().expect("compiles"), &machine);
        assert_round_trip(&canonical);
    }

    #[test]
    fn check_and_compile_agree(spec in any_spec()) {
        let has_errors = spec.check().iter().any(|d| d.is_error());
        let compiled = spec.compile();
        prop_assert_eq!(has_errors, compiled.is_err(), "{:?}", compiled.err());
    }

    #[test]
    fn parsing_never_panics_on_any_text(source in "\\PC*") {
        assert_spans_valid(&source);
    }

    #[test]
    fn parsing_never_panics_on_token_soup(source in token_soup()) {
        assert_spans_valid(&source);
    }

    #[test]
    fn parsing_never_panics_on_damaged_machines(
        cut in any::<Index>(),
        insert in any::<Index>(),
        piece in "[(){},='*\"/\n δ_␣x]{0,3}",
    ) {
        for (_, source) in gallery_sources() {
            let mut damaged = source.clone();
            let mut at = cut.index(source.len() + 1);
            while !damaged.is_char_boundary(at) {
                at -= 1;
            }
            damaged.truncate(at);
            let mut at = insert.index(source.len() + 1);
            while !source.is_char_boundary(at) {
                at -= 1;
            }
            damaged.push_str(&piece);
            damaged.push_str(&source[at..]);
            assert_spans_valid(&damaged);
        }
    }
}

/// Hostile input must not make parsing slow: every check uses sets and maps, and "did you
/// mean" searches are rationed. Each of these texts parses in well under a second; a quadratic
/// check would take minutes.
#[test]
fn large_inputs_parse_quickly() {
    let states: Vec<String> = (0..5_000).map(|i| format!("q{i}")).collect();
    let mut text = format!("Q = {{{}}}\n", states.join(", "));
    for i in 0..20_000 {
        text.push_str(&format!("δ(q{i}, x) = (p{i}, x, R)\n"));
    }
    let codes: std::collections::BTreeSet<_> = dsl::parse(&text)
        .diagnostics
        .iter()
        .map(|d| d.code)
        .collect();
    assert_eq!(codes, ["E003", "E009", "W001"].into());

    let glyphs: String = ('\u{4e00}'..'\u{9000}').take(20_000).collect();
    let mut text = String::new();
    for (i, glyph) in glyphs.chars().enumerate() {
        text.push_str(&format!("δ(q{i}, {glyph}) = (q{}, {glyph}, R)\n", i + 1));
    }
    let codes: std::collections::BTreeSet<_> = dsl::parse(&text)
        .diagnostics
        .iter()
        .map(|d| d.code)
        .collect();
    assert_eq!(codes, ["E009"].into());

    let finals: Vec<String> = (0..20_000).map(|i| format!("f{i}")).collect();
    let text = format!(
        "Q = {{{}}}\nF = {{{}}}\n",
        states.join(", "),
        finals.join(", ")
    );
    assert!(
        dsl::parse(&text)
            .diagnostics
            .iter()
            .any(|d| d.code == "E003")
    );

    let set: Vec<String> = (0..100)
        .map(|i| format!("'{}'", char::from_u32(0x100 + i).unwrap()))
        .collect();
    let set = format!("{{{}}}", set.join(", "));
    let text = format!("tapes = 3\nδ(q, ({set}, {set}, {set})) = (q, (*, *, *), (R, R, R))\n");
    let codes: Vec<_> = dsl::parse(&text)
        .diagnostics
        .iter()
        .map(|d| d.code)
        .collect();
    assert_eq!(codes, ["E009"]);
}
