//! Helpers for the unit tests: specs written as DTOs, without the parser.

use crate::dto::{
    CommentsDto, DeclaredDto, DecoderDto, DirDto, KindDto, MachineSpecDto, StateDto, TransitionDto,
};

/// The BB(5) champion of Marxen and Buntrock.
pub(crate) const BB5: &str = "1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA";

/// The BB(4) champion of Brady.
pub(crate) const BB4: &str = "1RB1LB_1LA0LC_1RZ1LD_1RD0RA";

/// A single-tape transition `δ(from, read) = (to, write, dir)`.
pub(crate) fn t(from: &str, read: &str, to: &str, write: &str, dir: DirDto) -> TransitionDto {
    multi(from, &[read], to, &[write], &[dir])
}

/// A transition with one entry per tape.
pub(crate) fn multi(
    from: &str,
    read: &[&str],
    to: &str,
    write: &[&str],
    dir: &[DirDto],
) -> TransitionDto {
    TransitionDto {
        from: from.into(),
        read: read.iter().map(|&s| s.into()).collect(),
        to: to.into(),
        write: write.iter().map(|&s| s.into()).collect(),
        dir: dir.to_vec(),
        comment: None,
        leading: Vec::new(),
        span: None,
    }
}

/// A spec with blank `␣`, everything inferred from the transitions, and `finals` in F.
pub(crate) fn spec(
    kind: KindDto,
    tapes: u8,
    transitions: Vec<TransitionDto>,
    finals: &[&str],
) -> MachineSpecDto {
    let mut states: Vec<StateDto> = Vec::new();
    let mut glyphs: Vec<String> = vec!["␣".into()];
    for tr in &transitions {
        for name in [&tr.from, &tr.to] {
            if !states.iter().any(|s| &s.name == name) {
                states.push(StateDto {
                    name: name.clone(),
                    is_final: finals.contains(&name.as_str()),
                });
            }
        }
        for g in tr.read.iter().chain(&tr.write) {
            if !glyphs.contains(g) {
                glyphs.push(g.clone());
            }
        }
    }
    let start = states.first().map(|s| s.name.clone()).unwrap_or_default();
    MachineSpecDto {
        name: String::new(),
        kind,
        tapes,
        blank: "␣".into(),
        input_alphabet: glyphs[1..].to_vec(),
        tape_alphabet: glyphs,
        states,
        start,
        transitions,
        input: String::new(),
        output: DecoderDto::Word,
        declared: DeclaredDto::default(),
        comments: CommentsDto::default(),
    }
}

/// Binary increment, least significant bit first, decoded as a binary number.
pub(crate) fn increment() -> MachineSpecDto {
    let mut s = spec(
        KindDto::Transducer,
        1,
        vec![
            t("q0", "1", "q0", "0", DirDto::R),
            t("q0", "0", "q1", "1", DirDto::R),
            t("q0", "␣", "q1", "1", DirDto::R),
        ],
        &["q1"],
    );
    s.input_alphabet = vec!["0".into(), "1".into()];
    s.output = DecoderDto::Binary {
        order: crate::dto::BitOrderDto::Lsb,
    };
    s
}

/// An enumerator printing a, aa, aaa, ...: it prints each time it enters `p` at the left
/// end of the word, then appends an `a` at the right end.
pub(crate) fn tally_enumerator() -> MachineSpecDto {
    spec(
        KindDto::Enumerator,
        1,
        vec![
            t("q0", "␣", "back", "a", DirDto::L),
            t("back", "a", "back", "a", DirDto::L),
            t("back", "␣", "p", "␣", DirDto::R),
            t("p", "a", "fwd", "a", DirDto::R),
            t("fwd", "a", "fwd", "a", DirDto::R),
            t("fwd", "␣", "back", "a", DirDto::L),
        ],
        &["p"],
    )
}

/// A machine in the standard format as a spec, through the core.
pub(crate) fn standard(text: &str) -> MachineSpecDto {
    let spec = turing_core::standard::spec_from_standard(text).expect("valid standard format");
    crate::convert::spec_dto(&spec, &crate::spans::Positions::Utf16)
}
