//! End-to-end runs of known machines on both simulators.

use turing_core::config_text::configuration;
use turing_core::{
    ChainSim, Decoder, Dir, Engine, Kind, Machine, Outcome, Simulator, Status, TapeView, Timeline,
    decode,
};

/// Builds a machine from the bbchallenge standard format (`1RB1LC_...`) with the builder.
///
/// States are `A`, `B`, ... in order, symbols are the digits with blank `0`, and `Z` is the
/// halting state. The real parser lives in `turing_core::standard`; this one keeps the tests of
/// the simulators independent of it.
fn standard(text: &str) -> Machine {
    let rows: Vec<&[u8]> = text.split('_').map(str::as_bytes).collect();
    let symbols = rows[0].len() / 3;
    let digit = |i: usize| {
        u32::try_from(i)
            .ok()
            .and_then(|i| char::from_digit(i, 10))
            .expect("at most 10 symbols")
    };
    let name = |i: usize| {
        let offset = u8::try_from(i).expect("at most 26 states");
        char::from(b'A' + offset).to_string()
    };
    let mut builder = Machine::builder(text)
        .kind(Kind::Beaver)
        .blank('0')
        .tape_alphabet((0..symbols).map(digit));
    for i in 0..rows.len() {
        builder = builder.state(&name(i));
    }
    for (i, row) in rows.iter().enumerate() {
        for (read, cell) in row.chunks(3).enumerate() {
            if cell == b"---" {
                continue;
            }
            let dir = Dir::from_char(char::from(cell[1])).expect("a move L or R");
            let next = char::from(cell[2]).to_string();
            builder = builder.transition(&name(i), digit(read), &next, char::from(cell[0]), dir);
        }
    }
    if text.contains('Z') {
        builder = builder.final_state("Z");
    }
    builder.build().expect("a valid machine")
}

/// The steps and the number of non-blank cells after running both simulators to the halt.
fn run_both(m: &Machine, fuel: u64) -> (u64, u64) {
    let mut dense = Simulator::new(m, &[]);
    let mut chain = ChainSim::new(m, &[]).expect("a single-tape machine");
    let Outcome::Halted { steps, state } = dense.run(fuel) else {
        panic!("{} did not halt", m.name());
    };
    assert_eq!(chain.run(fuel), Outcome::Halted { steps, state });
    assert_eq!(m.state(state).name, "Z");
    let ones = dense.tape(0).count_non_blank();
    assert_eq!(chain.tape().count_non_blank(), ones);
    assert_eq!(chain.tape().non_blank(), dense.tape(0).non_blank());
    assert_eq!(chain.tape().head(), dense.tape(0).head());
    assert_eq!(chain.extent(), dense.tape(0).touched());
    (steps, ones)
}

#[test]
fn small_busy_beaver_champions() {
    let champions = [
        ("1RZ---", 1, 1),
        ("1RB1LB_1LA1RZ", 6, 4),
        ("1RB1RZ_1LB0RC_1LC1LA", 21, 5),
        ("1RB1LB_1LA0LC_1RZ1LD_1RD0RA", 107, 13),
        ("1RB2LB1RZ_2LA2RB1LB", 38, 9),
    ];
    for (text, steps, ones) in champions {
        assert_eq!(run_both(&standard(text), 1_000), (steps, ones), "{text}");
    }
}

#[test]
fn two_state_four_symbol_champion() {
    let m = standard("1RB2LA1RA1RA_1LB1LA3RB1RZ");
    assert_eq!(run_both(&m, 10_000_000), (3_932_964, 2050));
}

#[test]
fn five_state_champion() {
    let m = standard("1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA");
    assert_eq!(run_both(&m, 100_000_000), (47_176_870, 4098));
}

#[test]
fn the_five_state_champion_is_never_overshot() {
    let m = standard("1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA");
    let mut chain = ChainSim::new(&m, &[]).unwrap();
    assert_eq!(
        chain.run(47_176_869),
        Outcome::OutOfFuel { steps: 47_176_869 }
    );
    let mut dense = Simulator::new(&m, &[]);
    let mut timeline = Timeline::new(ChainSim::new(&m, &[]).unwrap());
    for target in [47_176_869, 1_000_000, 30_000_000, 47_176_870, 12_345] {
        let outcome = timeline.seek(target);
        if target < dense.steps() {
            dense = Simulator::new(&m, &[]);
        }
        assert_eq!(dense.advance(target - dense.steps()), outcome);
        let current = timeline.current();
        assert_eq!(current.state(), dense.state());
        assert_eq!(current.tape().head(), dense.tape(0).head());
        let extent = dense.tape(0).non_blank().unwrap();
        assert_eq!(current.tape().non_blank(), Some(extent.clone()));
        let len = usize::try_from(extent.end() - extent.start() + 1).unwrap();
        assert_eq!(
            current.tape().window(*extent.start(), len),
            dense.tape(0).window(*extent.start(), len)
        );
    }
}

/// Copies a word over {a, b} to the second tape and rewinds both heads.
fn copy_machine() -> Machine {
    let mut b = Machine::builder("copy").tapes(2).input_alphabet(['a', 'b']);
    for x in ['a', 'b'] {
        b = b
            .transition_multi("copy", &[x, '␣'], "copy", &[x, x], &[Dir::R, Dir::R])
            .transition_multi("back", &[x, x], "back", &[x, x], &[Dir::L, Dir::L]);
    }
    b.transition_multi("copy", &['␣', '␣'], "back", &['␣', '␣'], &[Dir::L, Dir::L])
        .transition_multi("back", &['␣', '␣'], "done", &['␣', '␣'], &[Dir::R, Dir::R])
        .final_state("done")
        .build()
        .expect("a valid machine")
}

#[test]
fn two_tape_copy() {
    let m = copy_machine();
    for word in [
        "",
        "a",
        "ab",
        "abba",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbba",
    ] {
        let input = m.encode_input(word).unwrap();
        let mut sim = Simulator::new(&m, &input);
        let n = u64::try_from(input.len()).unwrap();
        let done = m.state_by_name("done").unwrap();
        assert_eq!(
            sim.run(1_000),
            Outcome::Halted {
                steps: 2 * n + 2,
                state: done
            }
        );
        assert!(m.is_final(sim.state()));
        for tape in sim.tapes() {
            assert_eq!(tape.head(), 0);
            assert_eq!(tape.window(0, input.len()), input);
            assert_eq!(decode(&m, tape).word, word);
        }
        assert!(ChainSim::new(&m, &input).is_err());
    }
}

#[test]
fn two_tape_doubling_takes_linear_time() {
    // x^n to x^2n: copy to tape 2, rewind it, append it to tape 1.
    let m = Machine::builder("double")
        .tapes(2)
        .decoder(Decoder::Unary { symbol: 'x' })
        .transition_multi("copy", &['x', '␣'], "copy", &['x', 'x'], &[Dir::R, Dir::R])
        .transition_multi(
            "copy",
            &['␣', '␣'],
            "rewind",
            &['␣', '␣'],
            &[Dir::N, Dir::L],
        )
        .transition_multi(
            "rewind",
            &['␣', 'x'],
            "rewind",
            &['␣', 'x'],
            &[Dir::N, Dir::L],
        )
        .transition_multi(
            "rewind",
            &['␣', '␣'],
            "append",
            &['␣', '␣'],
            &[Dir::N, Dir::R],
        )
        .transition_multi(
            "append",
            &['␣', 'x'],
            "append",
            &['x', 'x'],
            &[Dir::R, Dir::R],
        )
        .transition_multi(
            "append",
            &['␣', '␣'],
            "done",
            &['␣', '␣'],
            &[Dir::N, Dir::N],
        )
        .build()
        .unwrap();
    for n in 0..=20 {
        let input = m.encode_input(&"x".repeat(n)).unwrap();
        let mut sim = Simulator::new(&m, &input);
        let outcome = sim.run(10_000);
        assert_eq!(outcome.steps(), 3 * u64::try_from(n).unwrap() + 3);
        assert!(outcome.is_halted());
        let out = decode(&m, sim.tape(0));
        assert_eq!(out.word, "x".repeat(2 * n));
        assert_eq!(out.value, Some(2 * u128::try_from(n).unwrap()));
    }
}

#[test]
fn configurations_follow_a_run() {
    // Binary increment, least significant bit first.
    let m = Machine::builder("increment")
        .transition("carry", '1', "carry", '0', Dir::R)
        .transition("carry", '0', "done", '1', Dir::L)
        .transition("carry", '␣', "done", '1', Dir::L)
        .final_state("done")
        .build()
        .unwrap();
    let input = m.encode_input("1101").unwrap();
    let mut sim = Simulator::new(&m, &input);
    let mut trace = vec![configuration(&m, sim.state(), sim.tape(0), 40)];
    while sim.step() == Status::Running {
        trace.push(configuration(&m, sim.state(), sim.tape(0), 40));
    }
    assert_eq!(
        trace,
        ["carry 1101", "0 carry 101", "00 carry 01", "0 done 011"]
    );

    let mut chain = ChainSim::new(&m, &input).unwrap();
    chain.run(100);
    assert_eq!(
        configuration(&m, chain.state(), chain.tape(), 40),
        "0 done 011"
    );
}
