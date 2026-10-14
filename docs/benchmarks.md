# Benchmarks

## turing-bb

Measured with `cargo bench -p turing-bb` (divan, release profile with fat LTO) on an Apple M1
Pro with 16 GB of memory, macOS 26.5, Rust 1.99.0, one thread, on 14 October 2026. Medians.

| Benchmark | Median | What it measures |
|---|---:|---|
| `enumerate_bb2` | 17.4 µs | 61 nodes of the 2-state tree normal form with the Coq-BB5 BB(2) pipeline |
| `enumerate_bb3` | 1.76 ms | 5,417 nodes, BB(3) pipeline |
| `enumerate_bb2x3` | 3.40 ms | 6,019 nodes of the 2-state 3-symbol tree, BB(2,3) pipeline |
| `enumerate_bb4` | 536 ms | 858,909 nodes, BB(4) pipeline (15 stages) |
| `loops_on_the_bb4_champion` | 408 ns | Loops with 107 steps on the node of the BB(4) champion |
| `pipeline_on_a_repwl_machine` | 951 µs | the BB(4) pipeline on `1RB1LA_1LA0RC_1LD1RC_---0LA`, which only RepWL resolves |
| `antihydra_walk_100k` | 76.4 ms | 100,000 iterations of Antihydra's rule with big integers |
| `two_up_up_five` | 730 µs | 2↑↑5 = 2^65536 as a decimal string (19,729 digits) |

Running Antihydra itself with the chain simulator up to 2^53 steps (41 rule applications, the
last one A(20, 86956818) at step 4,536,893,241,720,843) takes about 9.4 s
(`cargo test --release -p turing-bb --test antihydra -- --include-ignored`).

The same BB(4) enumeration compiled to `wasm32-unknown-unknown` (release, fat LTO, no
wasm-opt) and run in Node.js 25.2 takes 0.85 to 1.05 s on the same machine, the BB(3)
enumeration 7.5 ms.
