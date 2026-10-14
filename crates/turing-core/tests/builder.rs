//! The machine builder as seen by other crates.

use std::collections::BTreeSet;

use proptest::prelude::*;
use turing_core::{
    BuildError, BuildErrors, Dir, Kind, MAX_STATES, MAX_SYMBOLS, MAX_TAPES, Machine,
    MachineBuilder, State, Symbol,
};

#[test]
fn every_error_has_a_message() {
    let cases = [
        (
            BuildError::DuplicateState { name: "q".into() },
            "the state `q` is declared twice",
        ),
        (
            BuildError::UnknownState {
                name: "q".into(),
                index: Some(3),
            },
            "the state `q` is not declared in Q",
        ),
        (
            BuildError::UnknownSymbol {
                glyph: 'x',
                index: None,
            },
            "the symbol `x` is not in the tape alphabet",
        ),
        (
            BuildError::DuplicateTransition {
                state: "q".into(),
                read: "x".into(),
                index: 1,
                first: 0,
            },
            "δ(q, x) is defined twice",
        ),
        (
            BuildError::WrongArity {
                tapes: 2,
                found: 1,
                index: 0,
            },
            "the transition has 1 entries, but the machine has 2 tapes",
        ),
        (BuildError::MissingStart, "the machine has no states"),
        (
            BuildError::BlankInInputAlphabet { blank: '␣' },
            "the blank `␣` cannot be part of the input alphabet",
        ),
        (
            BuildError::InputNotInTape { glyph: 'y' },
            "the input symbol `y` is not in the tape alphabet",
        ),
        (
            BuildError::TapeCount { max: MAX_TAPES },
            "a machine has 1 to 3 tapes",
        ),
        (
            BuildError::TooManyStates { max: MAX_STATES },
            "a machine has at most 4096 states",
        ),
        (
            BuildError::TooManySymbols { max: MAX_SYMBOLS },
            "a tape alphabet has at most 64 symbols",
        ),
        (
            BuildError::TableTooLarge { max: 1 << 22 },
            "the transition table would have more than 4194304 entries",
        ),
    ];
    for (error, message) in cases {
        assert_eq!(error.to_string(), message);
    }
}

#[test]
fn build_errors_are_standard_errors() {
    let errors = Machine::builder("empty").build().unwrap_err();
    let boxed: Box<dyn std::error::Error> = Box::new(errors.clone());
    assert_eq!(boxed.to_string(), "the machine has no states");
    assert_eq!(errors, BuildErrors(vec![BuildError::MissingStart]));
}

#[test]
fn the_course_example_builds() {
    // Unary multiplication from the TheoLog exercise sheet, first transitions only.
    let m = Machine::builder("A_mul")
        .kind(Kind::Transducer)
        .declare_states(["q0", "q1", "q2"])
        .input_alphabet(['x', '#'])
        .tape_alphabet(['x', 'X', '#', '␣'])
        .start("q0")
        .final_state("q2")
        .default_input("#xxx#xx#")
        .transition("q0", '#', "q1", '#', Dir::R)
        .transition("q1", 'x', "q1", 'x', Dir::R)
        .transition("q1", 'X', "q1", 'X', Dir::R)
        .build()
        .unwrap();
    assert_eq!(m.alphabet().glyphs(), ['␣', 'x', 'X', '#']);
    assert_eq!(m.alphabet().input_glyphs().collect::<String>(), "x#");
    assert_eq!(m.encode_input(m.default_input()).unwrap().len(), 8);
    assert_eq!(m.encode_input("#X#").unwrap_err().position, 1);
    assert!(m.is_final(m.state_by_name("q2").unwrap()));
    assert_eq!(m.num_transitions(), 3);
}

/// A random single-tape transition list over states `q0..q4` and glyphs `a`, `b`, `c`.
fn transitions() -> impl Strategy<Value = Vec<(u8, u8, u8, u8, u8)>> {
    prop::collection::vec((0u8..5, 0u8..4, 0u8..5, 0u8..4, 0u8..3), 0..30)
}

fn glyph(i: u8) -> char {
    ['␣', 'a', 'b', 'c'][usize::from(i)]
}

fn dir(i: u8) -> Dir {
    [Dir::L, Dir::R, Dir::N][usize::from(i)]
}

fn builder(list: &[(u8, u8, u8, u8, u8)]) -> MachineBuilder {
    list.iter().fold(
        Machine::builder("random").state("q0"),
        |b, &(from, read, to, write, d)| {
            b.transition(
                &format!("q{from}"),
                glyph(read),
                &format!("q{to}"),
                glyph(write),
                dir(d),
            )
        },
    )
}

proptest! {
    #[test]
    fn built_tables_hold_exactly_the_first_definitions(list in transitions()) {
        let mut seen = BTreeSet::new();
        let mut duplicates = Vec::new();
        for (index, &(from, read, ..)) in list.iter().enumerate() {
            if !seen.insert((from, read)) {
                duplicates.push(index);
            }
        }
        match builder(&list).build() {
            Ok(m) => {
                prop_assert!(duplicates.is_empty());
                prop_assert_eq!(m.num_transitions(), list.len());
                for (index, &(from, read, to, write, d)) in list.iter().enumerate() {
                    let q = m.state_by_name(&format!("q{from}")).unwrap();
                    let a = m.alphabet().symbol(glyph(read)).unwrap();
                    let action = m.action1(q, a).unwrap();
                    prop_assert_eq!(&m.state(action.next).name, &format!("q{to}"));
                    prop_assert_eq!(m.alphabet().glyph(action.write[0]), glyph(write));
                    prop_assert_eq!(action.dir[0], dir(d));
                    prop_assert_eq!(m.origin(q, &[a]), Some(index));
                }
                let listed: Vec<(State, Symbol)> =
                    m.transitions().map(|t| (t.from, t.read[0])).collect();
                let mut sorted = listed.clone();
                sorted.sort();
                prop_assert_eq!(listed, sorted);
            }
            Err(errors) => {
                let reported: Vec<usize> = errors
                    .0
                    .iter()
                    .map(|e| match e {
                        BuildError::DuplicateTransition { index, .. } => *index,
                        other => panic!("unexpected error {other:?}"),
                    })
                    .collect();
                prop_assert_eq!(reported, duplicates);
            }
        }
    }

    #[test]
    fn building_is_deterministic(list in transitions()) {
        prop_assert_eq!(builder(&list).build(), builder(&list).build());
    }
}
