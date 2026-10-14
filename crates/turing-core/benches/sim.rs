//! Benchmarks of the simulators. Run with `cargo bench -p turing-core`.
//!
//! Each benchmark runs a busy beaver champion from the blank tape to its halt. The throughput
//! counter is the number of machine steps, so divan reports steps per second.

use divan::counter::ItemsCount;
use divan::{Bencher, black_box};
use turing_core::{ChainSim, Dir, Kind, Machine, Outcome, Simulator, Timeline};

fn main() {
    divan::main();
}

/// The five-state champion: 47,176,870 steps, 4,098 ones.
const BB5: (&str, u64) = ("1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA", 47_176_870);

/// The two-state four-symbol champion: 3,932,964 steps.
const BB2X4: (&str, u64) = ("1RB2LA1RA1RA_1LB1LA3RB1RZ", 3_932_964);

/// Builds a machine from the standard format; see `tests/sim.rs`.
fn standard(text: &str) -> Machine {
    let rows: Vec<&[u8]> = text.split('_').map(str::as_bytes).collect();
    let digit = |i: usize| {
        u32::try_from(i)
            .ok()
            .and_then(|i| char::from_digit(i, 10))
            .expect("at most 10 symbols")
    };
    let name = |i: usize| char::from(b'A' + u8::try_from(i).expect("few states")).to_string();
    let mut builder = Machine::builder(text)
        .kind(Kind::Beaver)
        .blank('0')
        .tape_alphabet((0..rows[0].len() / 3).map(digit));
    for i in 0..rows.len() {
        builder = builder.state(&name(i));
    }
    for (i, row) in rows.iter().enumerate() {
        for (read, cell) in row.chunks(3).enumerate() {
            let dir = Dir::from_char(char::from(cell[1])).expect("a move");
            let next = char::from(cell[2]).to_string();
            builder = builder.transition(&name(i), digit(read), &next, char::from(cell[0]), dir);
        }
    }
    builder.build().expect("a valid machine")
}

#[divan::bench(args = [BB5, BB2X4], sample_count = 10)]
fn dense_to_halt(bencher: Bencher<'_, '_>, (text, steps): (&str, u64)) {
    let m = standard(text);
    bencher.counter(ItemsCount::new(steps)).bench(|| {
        let mut sim = Simulator::new(&m, &[]);
        let outcome = sim.run(u64::MAX);
        assert_eq!(outcome.steps(), steps);
        black_box(outcome)
    });
}

#[divan::bench(args = [BB5, BB2X4], sample_count = 20)]
fn chain_to_halt(bencher: Bencher<'_, '_>, (text, steps): (&str, u64)) {
    let m = standard(text);
    bencher.counter(ItemsCount::new(steps)).bench(|| {
        let mut sim = ChainSim::new(&m, &[]).expect("a single-tape machine");
        let outcome = sim.run(u64::MAX);
        assert_eq!(outcome.steps(), steps);
        black_box(outcome)
    });
}

/// One step back from the end of the BB(5) run: restoring a checkpoint and replaying at most
/// one interval.
#[divan::bench(sample_count = 20)]
fn timeline_step_back(bencher: Bencher<'_, '_>) {
    let m = standard(BB5.0);
    let mut timeline = Timeline::new(ChainSim::new(&m, &[]).expect("a single-tape machine"));
    assert!(matches!(timeline.seek(u64::MAX), Outcome::Halted { .. }));
    bencher.bench_local(|| {
        let outcome = timeline.seek(BB5.1 - 1);
        timeline.seek(BB5.1);
        black_box(outcome)
    });
}
