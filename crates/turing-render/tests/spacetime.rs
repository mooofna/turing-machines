//! Space-time diagrams: pixels, sampling, colours, lookup and incremental rendering.

use proptest::prelude::*;
use turing_core::{Dir, Kind, Machine, MachineBuilder, Symbol};
use turing_render::spacetime::index::{BACKGROUND, HEAD, HEAD_RANGE, INK_MAX, INK_MIN};
use turing_render::{
    CellRef, Palette, Rgb, SpaceTime, SpaceTimeError, SpaceTimeMode, SpaceTimeProgress,
    SpaceTimeRequest,
};

/// A busy beaver in the standard text format, `Z` halting.
fn beaver(name: &str, table: &str) -> Machine {
    let mut b = MachineBuilder::new(name)
        .kind(Kind::Beaver)
        .blank('0')
        .tape_alphabet(['0', '1']);
    for (q, row) in table.split('_').enumerate() {
        let from = char::from(b'A' + u8::try_from(q).expect("at most 26 states")).to_string();
        let cells: Vec<char> = row.chars().collect();
        for (read, t) in cells.chunks(3).enumerate() {
            let dir = Dir::from_char(t[1]).expect("a move L or R");
            let read = char::from(b'0' + u8::try_from(read).expect("at most 10 symbols"));
            b = b.transition(&from, read, &t[2].to_string(), t[0], dir);
        }
    }
    b.final_state("Z").build().expect("a valid busy beaver")
}

fn bb3() -> Machine {
    beaver("bb3", "1RB1RZ_1LB0RC_1LC1LA")
}

fn bb4() -> Machine {
    beaver("bb4", "1RB1LB_1LA0LC_1RZ1LD_1RD0RA")
}

fn bb5() -> Machine {
    beaver("bb5", "1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA")
}

/// Writes 1 and moves right forever.
fn walk() -> Machine {
    Machine::builder("walk")
        .transition("q", '␣', "q", '1', Dir::R)
        .build()
        .expect("a valid machine")
}

/// Moves right forever without writing.
fn drift() -> Machine {
    Machine::builder("drift")
        .transition("q", '␣', "q", '␣', Dir::R)
        .build()
        .expect("a valid machine")
}

/// Stays on its cell forever.
fn stay() -> Machine {
    Machine::builder("stay")
        .transition("q", '␣', "q", '␣', Dir::N)
        .build()
        .expect("a valid machine")
}

/// Writes 1, 2, 1, 2, ... moving right, then halts after four cells.
fn two_symbols() -> Machine {
    Machine::builder("two")
        .transition("a", '␣', "b", '1', Dir::R)
        .transition("b", '␣', "c", '2', Dir::R)
        .transition("c", '␣', "d", '1', Dir::R)
        .transition("d", '␣', "e", '2', Dir::R)
        .build()
        .expect("a valid machine")
}

fn exact(width: u32, height: u32, first_step: u64, cell_px: Option<u32>) -> SpaceTimeRequest {
    SpaceTimeRequest {
        width,
        height,
        mode: SpaceTimeMode::Exact {
            first_step,
            cell_px,
        },
    }
}

fn overview(width: u32, height: u32, max_steps: u64) -> SpaceTimeRequest {
    SpaceTimeRequest {
        width,
        height,
        mode: SpaceTimeMode::Overview { max_steps },
    }
}

/// Renders completely in one call.
fn render(machine: &Machine, input: &[Symbol], request: SpaceTimeRequest) -> SpaceTime {
    let mut st = SpaceTime::new(machine, input, request).expect("a valid request");
    let p = st.advance(u64::MAX);
    assert!(p.done, "{p:?}");
    st
}

fn pixel(st: &SpaceTime, x: u32, y: u32) -> u8 {
    st.index_buffer()[(y * st.width() + x) as usize]
}

/// The index buffer as text, one character per pixel.
fn picture(st: &SpaceTime) -> String {
    let mut out = String::new();
    for y in 0..st.height() {
        for x in 0..st.width() {
            out.push(match pixel(st, x, y) {
                BACKGROUND => '.',
                HEAD => 'H',
                HEAD_RANGE => '-',
                INK_MAX => '#',
                _ => '+',
            });
        }
        out.push('\n');
    }
    out
}

#[test]
fn a_walk_to_the_right_is_a_diagonal() {
    let st = render(&walk(), &[], exact(6, 4, 0, Some(1)));
    // Four cells, centred in six columns.
    assert_eq!(st.tape_range(), Some(-1..=4));
    assert_eq!(
        picture(&st),
        ".H....\n\
         .#H...\n\
         .##H..\n\
         .###H.\n"
    );
    assert_eq!(
        st.progress(),
        SpaceTimeProgress {
            rows_done: 4,
            steps: 3,
            done: true,
            halted: false
        }
    );
}

#[test]
fn a_run_that_halts_is_shown_whole_with_square_cells() {
    // Five configurations over five cells: 8 pixels per cell fill 40 by 40.
    let st = render(&two_symbols(), &[], exact(40, 48, 0, None));
    assert_eq!(st.tape_range(), Some(0..=4));
    assert_eq!(st.row(4), Some((4, 32..40)));
    assert_eq!(st.row(5), None);
    assert_eq!(pixel(&st, 39, 39), HEAD);
    assert_eq!(pixel(&st, 31, 39), INK_MAX);
    assert_eq!(pixel(&st, 0, 40), BACKGROUND);
    // Square cells that would cut the run short are smaller.
    let st = render(&two_symbols(), &[], exact(40, 20, 0, None));
    assert_eq!(st.row(4), Some((4, 16..20)));
}

#[test]
fn a_longer_run_gets_large_cells_and_enough_rows() {
    // The stay loop never ends; one cell wide, 32 rows of 8 pixels fill the height.
    let st = render(&stay(), &[], exact(64, 256, 0, None));
    assert_eq!(st.row(31), Some((31, 248..256)));
    // A small image shows more rows at a smaller size instead of a single large row.
    let st = render(&walk(), &[], exact(12, 12, 0, None));
    assert_eq!(st.row(11), Some((11, 11..12)));
}

#[test]
fn a_later_first_step_scrolls_the_window() {
    let st = render(&walk(), &[], exact(4, 2, 10, Some(1)));
    // The cells written so far and visited, 0 to 11, do not fit, so the window is centred
    // on the head.
    assert_eq!(st.tape_range(), Some(9..=12));
    assert_eq!(picture(&st), "#H..\n##H.\n");
    assert_eq!(st.locate(0, 1).unwrap().step, 11);
}

#[test]
fn the_window_centres_a_narrow_run() {
    let st = render(&two_symbols(), &[], exact(10, 5, 0, Some(1)));
    // Five rows (steps 0 to 4), cells 0 to 4, centred in ten columns.
    assert_eq!(st.tape_range(), Some(-2..=7));
    assert_eq!(st.progress().rows_done, 5);
    assert!(st.progress().halted);
}

#[test]
fn higher_symbols_are_darker() {
    let st = render(&two_symbols(), &[], exact(4, 5, 0, Some(1)));
    // The last row: 1 2 1 2, the head beyond.
    let last: Vec<u8> = (0..4).map(|x| pixel(&st, x, 4)).collect();
    let light = INK_MIN + (INK_MAX - INK_MIN).div_ceil(2);
    assert_eq!(last, [light, INK_MAX, light, INK_MAX]);
}

#[test]
fn halting_before_the_first_row_draws_nothing() {
    let st = render(&bb3(), &[], exact(10, 10, 100, None));
    assert_eq!(
        st.progress(),
        SpaceTimeProgress {
            rows_done: 0,
            steps: 21,
            done: true,
            halted: true
        }
    );
    assert!(st.index_buffer().iter().all(|&i| i == BACKGROUND));
    assert_eq!(st.locate(0, 0), None);
}

#[test]
fn bb4_exact_shows_every_step() {
    let st = render(&bb4(), &[], exact(200, 1000, 0, None));
    let p = st.progress();
    assert_eq!((p.rows_done, p.steps, p.halted), (108, 107, true));
    let (step, pixels) = st.row(107).unwrap();
    assert_eq!(step, 107);
    let px = pixels.end - pixels.start;
    assert!(px >= 1 && 108 * px <= 1000);
    let count = |y: u32, value: u8| (0..200).filter(|&x| pixel(&st, x, y) == value).count();
    // Exactly one head per row and 13 ones at the end.
    for r in 0..108 {
        assert_eq!(count(r * px, HEAD), px as usize, "row {r}");
    }
    let y = 107 * px;
    let head = st.locate((0..200).find(|&x| pixel(&st, x, y) == HEAD).unwrap(), y);
    let under_head = usize::from(head.unwrap().symbol != Some(Symbol::BLANK));
    assert_eq!(count(y, INK_MAX) / px as usize + under_head, 13);
}

#[test]
fn overviews_of_small_champions_finish_and_halt() {
    for (machine, steps) in [(bb3(), 21), (bb4(), 107)] {
        let st = render(&machine, &[], overview(300, 200, 1_000_000));
        let p = st.progress();
        assert_eq!((p.steps, p.halted, p.done), (steps, true, true));
        // One sample per step, the last one at the halt.
        assert_eq!(p.rows_done, u32::try_from(steps).unwrap() + 1);
        assert_eq!(st.row(p.rows_done - 1).unwrap().0, steps);
    }
}

#[test]
fn overview_samples_evenly_and_ends_at_the_last_step() {
    let st = render(&walk(), &[], overview(50, 11, 1000));
    let p = st.progress();
    assert_eq!((p.steps, p.halted, p.rows_done), (1000, false, 11));
    let steps: Vec<u64> = (0..11).map(|r| st.row(r).unwrap().0).collect();
    assert_eq!(steps, (0..=10).map(|r| r * 100).collect::<Vec<_>>());
    // 1001 cells in 50 bins: the head is in the last bin of the last row.
    assert_eq!(st.tape_range(), Some(0..=1000));
    assert_eq!(pixel(&st, 49, 10), HEAD);
    // Half the tape is written halfway.
    let written = (0..50).filter(|&x| pixel(&st, x, 5) != BACKGROUND).count();
    assert!((24..=26).contains(&written), "{written}");
}

#[test]
fn overview_tints_blank_cells_the_head_passed() {
    let st = render(&drift(), &[], overview(10, 3, 18));
    // Rows at steps 0, 9 and 18 over 19 cells in 10 bins.
    // Each bin is 1.9 cells wide. The cells after each step since the previous sample are
    // tinted where they are blank: cells 1 to 8 overlap bins 0 to 4, cells 10 to 17 bins 5
    // to 9; each head is in the bin that holds the middle of its cell.
    assert_eq!(
        picture(&st),
        "H.........\n\
         -----H....\n\
         .....----H\n"
    );
}

#[test]
fn stay_loops_and_walks_run_for_the_whole_budget() {
    let st = render(&stay(), &[], overview(4, 4, 1000));
    assert_eq!(st.progress().steps, 1000);
    assert!(!st.progress().halted);
    let st = render(&walk(), &[], exact(4, 4, 1_000_000_000_000, Some(1)));
    assert_eq!(st.progress().steps, 1_000_000_000_003);
    assert_eq!(pixel(&st, 0, 0), HEAD);
}

#[test]
fn bb5_overview_samples_the_whole_run() {
    let st = render(&bb5(), &[], overview(800, 600, 100_000_000));
    let p = st.progress();
    assert_eq!(
        (p.steps, p.halted, p.rows_done, p.done),
        (47_176_870, true, 600, true)
    );
    assert_eq!(st.row(599).unwrap().0, 47_176_870);
    let range = st.tape_range().unwrap();
    assert!(range.end() - range.start() > 800, "{range:?}");
    // Every row has exactly one head pixel.
    for y in 0..600 {
        assert_eq!((0..800).filter(|&x| pixel(&st, x, y) == HEAD).count(), 1);
    }
    let CellRef {
        step,
        from,
        to,
        symbol,
    } = st.locate(400, 599).unwrap();
    assert_eq!((step, symbol), (47_176_870, None));
    assert!(from < to && range.contains(&from) && range.contains(&to));
}

#[test]
fn locate_finds_steps_cells_and_symbols() {
    let st = render(&walk(), &[], exact(8, 8, 0, Some(1)));
    let one = Symbol::new(1);
    assert_eq!(
        st.locate(3, 5),
        Some(CellRef {
            step: 5,
            from: 3,
            to: 3,
            symbol: Some(one)
        })
    );
    // The symbol under the head.
    assert_eq!(st.locate(5, 5).unwrap().symbol, Some(Symbol::BLANK));
    assert_eq!(st.locate(8, 0), None);
    assert_eq!(st.locate(0, 8), None);

    // Rows not yet drawn are not located.
    let mut st = SpaceTime::new(&walk(), &[], exact(8, 8, 0, Some(1))).unwrap();
    assert_eq!(st.locate(0, 0), None);
    st.advance(7);
    assert_eq!(st.progress().rows_done, 1);
    assert!(st.locate(0, 0).is_some() && st.locate(0, 1).is_none());
}

#[test]
fn colours_follow_the_palette() {
    let st = render(&drift(), &[], overview(10, 3, 18));
    let palette = Palette {
        background: Rgb(255, 255, 255),
        ink: Rgb(0, 0, 0),
        accent: Rgb(0, 0, 255),
    };
    let rgba = st.rgba(&palette);
    let at = |x: u32, y: u32| {
        let i = 4 * (y * 10 + x) as usize;
        (Rgb(rgba[i], rgba[i + 1], rgba[i + 2]), rgba[i + 3])
    };
    assert_eq!(at(0, 0), (Rgb(0, 0, 255), 255));
    assert_eq!(at(1, 0), (Rgb(255, 255, 255), 255));
    assert_eq!(at(0, 1), (Rgb(166, 166, 255), 255));

    let st = render(&two_symbols(), &[], exact(4, 5, 0, Some(1)));
    let rgba = st.rgba(&palette);
    // Symbol 2 is the ink itself, symbol 1 about half way.
    assert_eq!(&rgba[4 * (4 * 4 + 1)..4 * (4 * 4 + 1) + 4], &[0, 0, 0, 255]);
    let half = rgba[4 * (4 * 4)];
    assert!((120..=135).contains(&half), "{half}");
}

#[test]
fn bad_requests_are_rejected() {
    let m = walk();
    for (w, h) in [(0, 10), (10, 0), (4097, 10), (10, 4097)] {
        assert_eq!(
            SpaceTime::new(&m, &[], overview(w, h, 10)).unwrap_err(),
            SpaceTimeError::Size
        );
    }
    let two_tapes = Machine::builder("copy")
        .tapes(2)
        .transition_multi("q", &['a', '␣'], "q", &['a', 'a'], &[Dir::R, Dir::R])
        .build()
        .unwrap();
    assert_eq!(
        SpaceTime::new(&two_tapes, &[], overview(10, 10, 10)).unwrap_err(),
        SpaceTimeError::MultiTape
    );
    assert_eq!(
        SpaceTime::new(&m, &[Symbol::new(9)], overview(10, 10, 10)).unwrap_err(),
        SpaceTimeError::Input
    );
}

#[test]
fn input_is_part_of_the_picture() {
    // The input 1 1 1 lies right of the start; the walk overwrites it with 1.
    let m = walk();
    let one = m.alphabet().symbol('1').unwrap();
    let st = render(&m, &[one, one, one], exact(6, 1, 0, Some(1)));
    assert_eq!(picture(&st), ".H##..\n");
}

fn machines() -> Vec<Machine> {
    vec![bb3(), bb4(), walk(), drift(), stay(), two_symbols()]
}

fn requests() -> Vec<SpaceTimeRequest> {
    vec![
        exact(40, 30, 0, None),
        exact(13, 17, 5, Some(2)),
        exact(64, 128, 0, None),
        overview(37, 23, 500),
        overview(3, 200, 10_000),
        overview(16, 1, 300),
    ]
}

proptest! {
    #[test]
    fn chunked_rendering_matches_one_pass(
        machine in 0..6usize,
        request in 0..6usize,
        chunks in prop::collection::vec(0u64..40, 1..12),
    ) {
        let (m, r) = (&machines()[machine], requests()[request]);
        let whole = render(m, &[], r);
        let mut parts = SpaceTime::new(m, &[], r).unwrap();
        let mut progress = parts.progress();
        let mut calls = 0;
        while !progress.done {
            let next = parts.advance(chunks[calls % chunks.len()].max(u64::from(calls > 10_000)));
            prop_assert!(next.rows_done >= progress.rows_done);
            prop_assert!(next.steps >= progress.steps);
            progress = next;
            calls += 1;
        }
        prop_assert_eq!(parts.progress(), whole.progress());
        prop_assert_eq!(parts.index_buffer(), whole.index_buffer());
        prop_assert_eq!(parts.rgba(&Palette::DARK), whole.rgba(&Palette::DARK));
        for (x, y) in [(0, 0), (5, 3), (12, 16), (2, 199)] {
            prop_assert_eq!(parts.locate(x, y), whole.locate(x, y));
        }
    }
}
