//! The bbchallenge standard format: reading, writing and golden runs of well-known machines.
//!
//! The step counts and scores below are the published values (bbchallenge wiki); the states
//! after a million steps were computed independently with a plain reference simulator.

use proptest::prelude::*;
use turing_core::dsl::{self, FormatStyle};
use turing_core::standard::{StandardError, parse_standard, spec_from_standard, to_standard};
use turing_core::{ChainSim, Kind, Machine, Outcome, Simulator, TapeView};

/// Halting champions: the machine, its number of steps and its number of non-blank cells.
const HALTING: [(&str, u64, u64); 14] = [
    ("1RZ---", 1, 1),
    ("1RB1LB_1LA1RZ", 6, 4),
    ("1RB1RZ_1LB0RC_1LC1LA", 21, 5),
    // The five Σ(3) champions.
    ("1RB1RZ_0RC1RB_1LC1LA", 14, 6),
    ("1RB1RC_1LC1RZ_1RA0LB", 13, 6),
    ("1RB1LC_1LA1RB_1LB1RZ", 13, 6),
    ("1RB1RA_1LC1RZ_1RA1LB", 12, 6),
    ("1RB1LC_1RC1RZ_1LA0LB", 11, 6),
    ("1RB1LB_1LA0LC_1RZ1LD_1RD0RA", 107, 13),
    ("1RB0RC_1LA1RA_1RZ1RD_1LD0LB", 96, 13),
    ("1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA", 47_176_870, 4098),
    ("1RB1RA_1LC1LB_1RA1LD_1RA1LE_1RZ0LC", 11_798_826, 4098),
    ("1RB2LB1RZ_2LA2RB1LB", 38, 9),
    ("1RB2LA1RA1RA_1LB1LA3RB1RZ", 3_932_964, 2050),
];

/// Machines that run on: the state, the number of non-blank cells and the head position after
/// exactly 1,000,000 steps.
const RUNNING: [(&str, &str, u64, i64); 5] = [
    ("1RB1RA_1RC1RZ_1LD0RF_1RA0LE_0LD1RC_1RA0RE", "D", 1312, 276),
    ("1RB1RA_0LC1LE_1LD1LC_1LA0LB_1LF1RE_---0RA", "A", 1381, 604),
    ("1RB3RB---3LA1RA_2LA3RA4LB0LB0LA", "B", 895, -862),
    ("1RB2RA1LC_2LC1RB2RB_---2LA1LA", "A", 1156, 724),
    ("1RB2LA1LA_2LA2RB0RA", "A", 1376, 600),
];

#[test]
fn champions_halt_with_their_scores_on_both_simulators() {
    for (text, steps, ones) in HALTING {
        let m = parse_standard(text).unwrap();
        let mut chain = ChainSim::new(&m, &[]).unwrap();
        let Outcome::Halted { steps: s, state } = chain.run(u64::MAX) else {
            panic!("{text} does not halt");
        };
        assert_eq!((s, chain.tape().count_non_blank()), (steps, ones), "{text}");
        assert_eq!(m.state(state).name, "Z");
        assert!(m.is_final(state));
        if steps < 5_000_000 {
            let mut sim = Simulator::new(&m, &[]);
            assert_eq!(sim.run(u64::MAX), Outcome::Halted { steps, state });
            assert_eq!(sim.tape(0).count_non_blank(), ones);
        }
    }
}

#[test]
fn running_machines_match_an_independent_simulation() {
    for (text, state, ones, head) in RUNNING {
        let m = parse_standard(text).unwrap();
        let mut sim = Simulator::new(&m, &[]);
        assert_eq!(sim.run(1_000_000), Outcome::OutOfFuel { steps: 1_000_000 });
        assert_eq!(m.state(sim.state()).name, state, "{text}");
        assert_eq!(sim.tape(0).count_non_blank(), ones, "{text}");
        assert_eq!(sim.tape(0).head(), head, "{text}");

        let mut chain = ChainSim::new(&m, &[]).unwrap();
        assert_eq!(
            chain.run(1_000_000),
            Outcome::OutOfFuel { steps: 1_000_000 }
        );
        assert_eq!(chain.state(), sim.state(), "{text}");
        assert_eq!(chain.tape().head(), head, "{text}");
    }
}

#[test]
fn wolframs_machine_after_a_thousand_steps() {
    let m = parse_standard("1RB2LA1LA_2LA2RB0RA").unwrap();
    assert_eq!(
        m.num_transitions(),
        6,
        "δ is total, so the machine never halts"
    );
    let mut sim = Simulator::new(&m, &[]);
    sim.run(1_000);
    assert_eq!(m.state(sim.state()).name, "B");
    assert_eq!(sim.tape(0).count_non_blank(), 43);
    assert_eq!(sim.tape(0).head(), -10);
}

#[test]
fn writing_reproduces_the_text() {
    let texts = HALTING
        .iter()
        .map(|h| h.0)
        .chain(RUNNING.iter().map(|r| r.0));
    for text in texts {
        let m = parse_standard(text).unwrap();
        assert_eq!(to_standard(&m).unwrap(), text);
        assert_eq!(parse_standard(&to_standard(&m).unwrap()).unwrap(), m);
    }
}

#[test]
fn the_spec_is_a_beaver_over_digits() {
    let spec = spec_from_standard("1RB2LB1RZ_2LA2RB1LB").unwrap();
    assert_eq!(spec.kind, Kind::Beaver);
    assert_eq!(spec.blank, '0');
    assert_eq!(spec.tape_alphabet, ['0', '1', '2']);
    assert_eq!(spec.input_alphabet, ['1', '2']);
    assert_eq!(spec.start, "A");
    let names: Vec<_> = spec
        .states
        .iter()
        .map(|s| (s.name.as_str(), s.is_final))
        .collect();
    assert_eq!(names, [("A", false), ("B", false), ("Z", true)]);
    assert!(spec.check().is_empty());
    insta::assert_snapshot!(dsl::format(&spec, FormatStyle::Unicode));
}

#[test]
fn specs_survive_the_description_language() {
    for text in HALTING
        .iter()
        .map(|h| h.0)
        .chain(RUNNING.iter().map(|r| r.0))
    {
        let spec = spec_from_standard(text).unwrap();
        for style in [FormatStyle::Unicode, FormatStyle::Ascii] {
            let parsed = dsl::parse(&dsl::format(&spec, style));
            assert!(parsed.diagnostics.is_empty(), "{text}");
            let mut again = parsed.spec.unwrap();
            for t in &mut again.transitions {
                t.span = None;
            }
            assert_eq!(again, spec, "{text}");
        }
        assert_eq!(spec.compile().unwrap(), parse_standard(text).unwrap());
    }
}

#[test]
fn every_letter_beyond_the_last_state_halts_in_z() {
    let m = parse_standard("1RB1RH_1LA1RZ").unwrap();
    assert_eq!(m.num_states(), 3);
    assert_eq!(to_standard(&m).unwrap(), "1RB1RZ_1LA1RZ");
    // Without a halting transition there is no state Z.
    let m = parse_standard("1RB---_1LA---").unwrap();
    assert_eq!(m.num_states(), 2);
    assert_eq!(parse_standard("  1RZ---\n").unwrap().num_states(), 2);
}

fn syntax_at(text: &str) -> usize {
    match spec_from_standard(text) {
        Err(StandardError::Syntax { at, .. }) => at,
        other => panic!("{text:?}: {other:?}"),
    }
}

#[test]
fn syntax_errors_point_at_the_byte() {
    assert_eq!(syntax_at(""), 0);
    assert_eq!(syntax_at("   "), 3);
    assert_eq!(syntax_at("1RB_"), 4);
    assert_eq!(syntax_at("_1RB"), 0);
    assert_eq!(syntax_at("1RB1L"), 5);
    assert_eq!(syntax_at("xRB"), 0);
    assert_eq!(syntax_at("1NB"), 1);
    assert_eq!(syntax_at("1Rb"), 2);
    assert_eq!(syntax_at("1R-"), 2);
    assert_eq!(syntax_at(" 1RB 1RA"), 4);
    assert_eq!(syntax_at("1RB1RA_1LA2RB"), 10);
    assert_eq!(syntax_at("1RBδRA"), 3);
}

#[test]
fn rows_must_have_equal_length() {
    assert_eq!(spec_from_standard("1RB1LA_1LA"), Err(StandardError::Ragged));
}

#[test]
fn letters_and_digits_limit_the_size() {
    let states = vec!["0RA"; 27].join("_");
    assert!(matches!(
        spec_from_standard(&states),
        Err(StandardError::Unsupported { .. })
    ));
    assert!(spec_from_standard(&vec!["0RA"; 26].join("_")).is_ok());
    let symbols = "1RA".repeat(11);
    assert!(matches!(
        spec_from_standard(&symbols),
        Err(StandardError::Unsupported { .. })
    ));
}

fn machine(source: &str) -> Machine {
    let parsed = dsl::parse(source);
    assert!(
        !parsed.diagnostics.iter().any(|d| d.is_error()),
        "{:?}",
        parsed.diagnostics
    );
    parsed.spec.unwrap().compile().unwrap()
}

fn unsupported(source: &str) -> String {
    match to_standard(&machine(source)) {
        Err(StandardError::Unsupported { reason }) => reason,
        other => panic!("{other:?}"),
    }
}

#[test]
fn writing_relabels_states_with_the_start_first() {
    let m = machine(
        "blank = 0\nstart = go\nF = {stop}\n\
         δ(back, 0) = (go, 1, L)\n\
         δ(go, 0) = (back, 1, R)\n\
         δ(go, 1) = (stop, 0, R)",
    );
    assert_eq!(to_standard(&m).unwrap(), "1RB0RZ_1LA---");
    // A state that halts without being final keeps a letter of its own.
    let m = machine("blank = 0\nδ(A, 0) = (B, 1, R)\nδ(A, 1) = (C, 1, R)");
    assert_eq!(to_standard(&m).unwrap(), "1RB1RC_------_------");
}

#[test]
fn writing_rejects_what_the_format_cannot_express() {
    let two_tapes = "tapes = 2\nblank = 0\nδ(A, (0, 0)) = (A, (1, 1), (R, R))";
    assert!(unsupported(two_tapes).contains("2 tapes"));
    assert!(unsupported("blank = 0\nδ(A, 0) = (A, 1, N)").contains("moves N"));
    assert!(unsupported("δ(A, x) = (A, x, R)").contains("digits"));
    let two_halts = "blank = 0\nF = {B, C}\nδ(A, 0) = (B, 1, R)\nδ(A, 1) = (C, 1, R)";
    assert!(unsupported(two_halts).contains("both halt"));
    assert!(unsupported("blank = 0\nΓ = {0, 2}\nδ(A, 0) = (A, 2, R)").contains("digits"));
    let many: String = (0..26)
        .map(|i| format!("δ(q{i}, 0) = (q{}, 1, R)\n", i + 1))
        .collect();
    let many = format!("blank = 0\nF = {{q26}}\n{many}");
    assert!(unsupported(&many).contains("at most 25"));
}

/// A text in the standard format with `states` states and `symbols` symbols.
fn standard_text() -> impl Strategy<Value = String> {
    (1..=6_usize, 1..=4_usize).prop_flat_map(|(states, symbols)| {
        let entry = prop_oneof![
            1 => Just("---".to_string()),
            4 => (0..symbols, prop::bool::ANY, 0..=states + 1).prop_map(move |(w, r, next)| {
                let next = if next >= states { 25 } else { next };
                format!(
                    "{w}{}{}",
                    if r { 'R' } else { 'L' },
                    char::from(b'A' + u8::try_from(next).unwrap_or(25))
                )
            }),
        ];
        prop::collection::vec(prop::collection::vec(entry, symbols), states).prop_map(|rows| {
            rows.iter()
                .map(|r| r.concat())
                .collect::<Vec<_>>()
                .join("_")
        })
    })
}

proptest! {
    #[test]
    fn writing_inverts_reading(text in standard_text()) {
        let machine = parse_standard(&text).unwrap();
        prop_assert_eq!(to_standard(&machine).unwrap(), text);
    }

    #[test]
    fn halting_letters_are_all_z(text in standard_text(), letter in b'A'..=b'Z') {
        // Rename the halting letter; every letter beyond the last state halts.
        let states = text.split('_').count();
        let letter = char::from(letter);
        if usize::from(u8::try_from(letter).unwrap_or(b'Z') - b'A') >= states {
            let renamed = text.replace('Z', &letter.to_string());
            let machine = parse_standard(&renamed).unwrap();
            prop_assert_eq!(to_standard(&machine).unwrap(), text);
        }
    }
}
