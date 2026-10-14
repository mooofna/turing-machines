# Benchmarks

The simulators of `turing-core` are measured with [divan](https://github.com/nvzqz/divan) in
`crates/turing-core/benches/sim.rs`. Each benchmark builds a busy beaver champion and runs it
from the blank tape to its halt; the throughput counter is the number of machine steps.

```sh
cargo bench -p turing-core --bench sim
```

## Results

Measured on 14 October 2026 on an Apple M1 Pro (6 performance and 2 efficiency cores,
16 GB) with macOS 26.5 and Rust 1.99.0, in the `bench` profile (release with fat LTO and one
codegen unit). Other builds were running at the same time, with a load average of about 12,
so the medians are somewhat pessimistic.

| Simulator | Machine | Steps | Time to halt, fastest | Time to halt, median | Steps per second, median |
|---|---|---:|---:|---:|---:|
| `Simulator` (dense) | BB(5) `1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA` | 47,176,870 | 95.4 ms | 103.3 ms | 457 M |
| `Simulator` (dense) | BB(2,4) `1RB2LA1RA1RA_1LB1LA3RB1RZ` | 3,932,964 | 7.99 ms | 8.10 ms | 486 M |
| `ChainSim` | BB(5) | 47,176,870 | 0.323 ms | 0.324 ms | |
| `ChainSim` | BB(2,4) | 3,932,964 | 0.073 ms | 0.073 ms | |

`ChainSim` covers the BB(5) run in 67,346 macro steps (700 steps per macro step on average) and
the BB(2,4) run in 12,234. Stepping one step back at the end of the BB(5) run with a
`Timeline` of `ChainSim` checkpoints takes 49 µs (median): it restores the nearest checkpoint
and replays at most one checkpoint interval.

Targets from the project plan: at least 200 M steps per second for the dense simulator and
well under 20 ms for the chain simulator to halt BB(5). Both are met.

## The dense hot loop

A step of the dense simulator reads the cell under the head, looks up δ for the state and the
symbol, writes, moves and enters the next state. The speed is bounded by the chain of
dependent operations from one step to the next, not by the number of instructions:

1. The first version kept the head and the visited range in the `Tape` and looked up the
   `Option<Action>` table at `state × stride + read`. Every write of a cell could alias the
   tape's fields as far as the compiler knew, so the head was stored and reloaded on every
   step, and the multiplication sat on the chain from one state to the next. BB(5) ran at
   about 330 M steps per second.
2. Running bursts of steps on local variables and a plain slice of cells removed the reloads,
   but the compiler turned the move of the head into a conditional select. The address of
   the next cell then waited for the table entry, and the speed dropped to about 175 M steps
   per second. Giving each move its own exit at the edge of the allocation keeps the move a
   branch, which the processor predicts and runs ahead of the table load.
3. Single-tape machines now carry a second, packed copy of δ: one `u64` per entry holding the
   move, the written symbol, the next state and that state's first table index. The lookup
   for the next step is a single load at `row + read`, with no multiplication on the chain.
   BB(5) runs at about 460 M to 495 M steps per second.

Steps at the edge of the allocated tape (where the tape has to grow) and steps that could
start a provable endless loop leave the burst and go through one careful step with every
check, so the result is the same as with a plain step-by-step loop. The differential tests
in `crates/turing-core/tests/differential.rs` compare both simulators with a naive reference
on random machines.

## The chain simulator

`ChainSim` keeps the tape as two stacks of maximal runs. A transition δ(q, a) = (q, b, D)
that stays in its state and moves is applied to the whole run of a's ahead of the head in one
macro step; a chain that would exceed the remaining fuel is shortened, so a run stops exactly
at any requested step. The busy beaver champions spend most of their time sweeping over long
runs of ones in such states, which is why BB(5) needs only 67,346 macro steps.
