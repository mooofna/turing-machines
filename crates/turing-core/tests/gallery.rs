//! Golden tests for the gallery in `machines/`: every expectation in `gallery.json`, and
//! exhaustive checks of the course and classic machines on small inputs.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use turing_core::dsl;
use turing_core::standard::spec_from_standard;
use turing_core::{ChainSim, Kind, Machine, MachineSpec, Outcome, Simulator, TapeView, decode};

fn machines_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../machines")
}

fn gallery_json() -> Value {
    let text = fs::read_to_string(machines_dir().join("gallery.json")).expect("gallery.json");
    serde_json::from_str(&text).expect("valid JSON")
}

/// The spec of a gallery entry, from its `.tm` file or its standard string.
fn spec_of(entry: &Value) -> MachineSpec {
    let id = entry["id"].as_str().expect("an id");
    if let Some(file) = entry["file"].as_str() {
        let source = fs::read_to_string(machines_dir().join(file)).expect("the .tm file");
        let parsed = dsl::parse(&source);
        assert!(
            parsed.diagnostics.is_empty(),
            "{id}: {:#?}",
            parsed.diagnostics
        );
        parsed.spec.expect("a spec")
    } else {
        let text = entry["standard"]
            .as_str()
            .expect("a file or a standard string");
        spec_from_standard(text).unwrap_or_else(|e| panic!("{id}: {e}"))
    }
}

/// Loads a `.tm` file of the gallery by its path relative to `machines/`.
fn load(file: &str) -> Machine {
    let source = fs::read_to_string(machines_dir().join(file)).expect("the .tm file");
    dsl::parse(&source)
        .spec
        .expect("no errors")
        .compile()
        .expect("compiles")
}

/// The first `count` outputs of an enumerator within `budget` steps.
fn emitted(machine: &Machine, count: usize, budget: u64) -> Vec<String> {
    let mut sim = Simulator::new(machine, &[]);
    let mut out = Vec::new();
    if machine.is_final(sim.state()) {
        out.push(decode(machine, sim.tape(0)).word);
    }
    while out.len() < count && sim.steps() < budget {
        if sim.run(1).is_halted() {
            break;
        }
        if machine.is_final(sim.state()) {
            out.push(decode(machine, sim.tape(0)).word);
        }
    }
    out
}

/// What a bounded run ended with.
struct Run {
    outcome: Outcome,
    ones: u64,
    word: String,
    value: Option<u128>,
    accepted: bool,
}

/// Runs `machine` on `input` for at most `budget` steps. Single-tape machines run on both
/// simulators, which must agree.
fn run(machine: &Machine, input: &str, budget: u64) -> Run {
    let input = machine.encode_input(input).expect("a valid input");
    let mut sim = Simulator::new(machine, &input);
    let outcome = sim.run(budget);
    let tape = sim.tape(0);
    if machine.tapes() == 1 {
        let mut chain = ChainSim::new(machine, &input).expect("one tape");
        let chained = chain.run(budget);
        // The chain simulator may prove non-termination before the budget runs out.
        if !matches!(chained, Outcome::NeverHalts { .. }) {
            assert_eq!(chained, outcome, "{}", machine.name());
            assert_eq!(chain.tape().count_non_blank(), tape.count_non_blank());
        }
    }
    let output = decode(machine, tape);
    let accepted = match outcome {
        Outcome::Halted { state, .. } => machine.is_final(state),
        _ => false,
    };
    Run {
        outcome,
        ones: tape.count_non_blank(),
        word: output.word,
        value: output.value,
        accepted,
    }
}

#[test]
fn the_gallery_is_well_formed() {
    let gallery = gallery_json();
    assert_eq!(gallery["version"], 1);
    let groups: Vec<&str> = gallery["groups"]
        .as_array()
        .expect("groups")
        .iter()
        .map(|g| g["id"].as_str().expect("a group id"))
        .collect();
    let mut ids = BTreeSet::new();
    let mut used_groups = BTreeSet::new();
    let mut files = BTreeSet::new();
    for entry in gallery["machines"].as_array().expect("machines") {
        let id = entry["id"].as_str().expect("an id");
        assert!(ids.insert(id), "duplicate id {id}");
        assert!(
            id.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "{id}"
        );
        let group = entry["group"].as_str().expect("a group");
        assert!(groups.contains(&group), "{id}: unknown group {group}");
        used_groups.insert(group);
        assert_ne!(
            entry["file"].is_string(),
            entry["standard"].is_string(),
            "{id}: exactly one of file and standard"
        );
        if let Some(file) = entry["file"].as_str() {
            files.insert(file.to_string());
        }
        for field in ["title", "provenance", "source", "description"] {
            assert!(entry[field].is_string(), "{id}: {field}");
        }
        assert!(entry["budget"].as_u64().is_some_and(|b| b > 0), "{id}");
        assert!(!entry["examples"].as_array().expect("examples").is_empty());
    }
    assert_eq!(used_groups.len(), groups.len(), "every group has machines");

    // Every machine file is listed.
    for dir in ["course", "classic"] {
        for file in fs::read_dir(machines_dir().join(dir)).expect("a directory") {
            let name = file.expect("an entry").file_name();
            let path = format!("{dir}/{}", name.to_string_lossy());
            assert!(files.contains(&path), "{path} is not in gallery.json");
        }
    }
}

#[test]
fn every_example_meets_its_expectations() {
    let gallery = gallery_json();
    for entry in gallery["machines"].as_array().expect("machines") {
        let id = entry["id"].as_str().expect("an id");
        let spec = spec_of(entry);
        let machine = spec.compile().unwrap_or_else(|d| panic!("{id}: {d:#?}"));
        let budget = entry["budget"].as_u64().expect("a budget");
        for example in entry["examples"].as_array().expect("examples") {
            let input = example["input"].as_str().expect("an input");
            let expect = example["expect"].as_object().expect("expectations");
            let context = format!("{id} on {input:?}");
            if let Some(words) = expect.get("emitted") {
                let words: Vec<&str> = words
                    .as_array()
                    .expect("an array")
                    .iter()
                    .map(|w| w.as_str().expect("a word"))
                    .collect();
                assert_eq!(machine.kind(), Kind::Enumerator, "{context}");
                assert_eq!(emitted(&machine, words.len(), budget), words, "{context}");
                continue;
            }
            let run = run(&machine, input, budget);
            for (key, value) in expect {
                match key.as_str() {
                    "halts" => assert_eq!(
                        run.outcome.is_halted(),
                        value.as_bool().expect("a bool"),
                        "{context}"
                    ),
                    "steps" => assert_eq!(run.outcome.steps(), value.as_u64().expect("a number")),
                    "ones" => assert_eq!(run.ones, value.as_u64().expect("a number"), "{context}"),
                    "output" => assert_eq!(run.word, value.as_str().expect("a word"), "{context}"),
                    "value" => assert_eq!(
                        run.value.map(|v| v.to_string()).as_deref(),
                        Some(value.as_str().expect("a decimal string")),
                        "{context}"
                    ),
                    "accepted" => {
                        assert!(run.outcome.is_halted(), "{context}");
                        assert_eq!(run.accepted, value.as_bool().expect("a bool"), "{context}");
                    }
                    other => panic!("{context}: unknown expectation {other}"),
                }
            }
        }
    }
}

/// `x` repeated `n` times.
fn xs(n: usize) -> String {
    "x".repeat(n)
}

#[test]
fn unary_multiplication_multiplies() {
    let machine = load("course/unary-multiplication.tm");
    for m in 0..=8 {
        for n in 0..=8 {
            let input = format!("#{}#{}#", xs(m), xs(n));
            let run = run(&machine, &input, 1_000_000);
            assert!(run.accepted, "{input}");
            assert_eq!(run.word, xs(m * n), "{input}");
            assert_eq!(run.value, Some(u128::try_from(m * n).expect("small")));
        }
    }
}

#[test]
fn doubling_takes_quadratic_time_on_one_tape_and_linear_time_on_two() {
    let one = load("course/doubling.tm");
    let two = load("course/doubling-two-tapes.tm");
    for n in 0..=20_u64 {
        let input = xs(usize::try_from(n).expect("small"));
        let (a, b) = (run(&one, &input, 1_000_000), run(&two, &input, 1_000_000));
        for run in [&a, &b] {
            assert!(run.accepted, "n = {n}");
            assert_eq!(run.word, xs(2 * input.len()), "n = {n}");
        }
        assert_eq!(a.outcome.steps(), 2 * (n + 1) * (n + 1), "n = {n}");
        assert_eq!(b.outcome.steps(), 2 * n + 1, "n = {n}");
    }
}

#[test]
fn mod_two_computes_the_parity() {
    let machine = load("course/mod-two.tm");
    for n in 0..=20 {
        let run = run(&machine, &xs(n), 1_000);
        assert!(run.accepted, "n = {n}");
        assert_eq!(run.word, xs(n % 2), "n = {n}");
    }
}

#[test]
fn binary_increment_adds_one() {
    let machine = load("course/binary-increment.tm");
    for n in 0..=200_u32 {
        // Least significant bit first; zero is the empty word.
        let input: String = format!("{n:b}").chars().rev().collect();
        let input = if n == 0 { String::new() } else { input };
        let run = run(&machine, &input, 1_000);
        assert_eq!(run.value, Some(u128::from(n) + 1), "n = {n}");
    }
}

#[test]
fn enumerators_output_their_languages_in_order() {
    let threes = load("course/multiples-of-three.tm");
    let expected: Vec<String> = (0..10).map(|n| xs(3 * n)).collect();
    assert_eq!(emitted(&threes, 10, 1_000_000), expected);

    let anbn = load("course/an-bn.tm");
    let expected: Vec<String> = (0..10)
        .map(|n| format!("{}{}", "a".repeat(n), "b".repeat(n)))
        .collect();
    assert_eq!(emitted(&anbn, 10, 1_000_000), expected);
}

/// Every word over `alphabet` of length at most `max`.
fn words(alphabet: &[char], max: usize) -> Vec<String> {
    let mut all = vec![String::new()];
    let mut last = vec![String::new()];
    for _ in 0..max {
        last = last
            .iter()
            .flat_map(|w| alphabet.iter().map(move |&c| format!("{w}{c}")))
            .collect();
        all.extend(last.iter().cloned());
    }
    all
}

#[test]
fn palindromes_are_accepted_and_nothing_else() {
    let machine = load("classic/palindromes.tm");
    for word in words(&['a', 'b'], 8) {
        let reversed: String = word.chars().rev().collect();
        let run = run(&machine, &word, 100_000);
        assert_eq!(run.accepted, word == reversed, "{word:?}");
    }
}

#[test]
fn anbncn_is_accepted_and_nothing_else() {
    let machine = load("classic/anbncn.tm");
    for word in words(&['a', 'b', 'c'], 7) {
        let n = word.len() / 3;
        let expected = word == format!("{}{}{}", "a".repeat(n), "b".repeat(n), "c".repeat(n));
        let run = run(&machine, &word, 100_000);
        assert_eq!(run.accepted, expected, "{word:?}");
    }
}

#[test]
fn binary_addition_adds() {
    let machine = load("classic/binary-addition.tm");
    for a in 0..=20_u32 {
        for b in 0..=20_u32 {
            let input = format!("{a:b}+{b:b}");
            let run = run(&machine, &input, 100_000);
            assert!(run.accepted, "{input}");
            assert_eq!(run.word, format!("{:b}", a + b), "{input}");
            assert_eq!(run.value, Some(u128::from(a + b)), "{input}");
        }
    }
}
