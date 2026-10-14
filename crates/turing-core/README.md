# turing-core

The model of a deterministic Turing machine with one to three tapes, two simulators and a small
description language that follows the notation of the course Theoretische Informatik und Logik at TU Dresden.

* `Machine` is a validated machine with a dense transition table. Build one with
  `MachineBuilder`, from a `MachineSpec`, from the description language (`dsl::parse`) or from the
  bbchallenge standard text format (`standard::parse_standard`).
* `Simulator` executes one step at a time on dense two-way infinite tapes.
* `ChainSim` executes single-tape machines on a run-length encoded tape and jumps over whole runs
  when a transition loops in place. It stops exactly at any requested step.
* `Timeline` keeps checkpoints of either simulator so that a run can be replayed backwards.

The crate is `no_std` with `alloc` and contains no unsafe code.
