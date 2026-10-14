# turing-bb

Enumeration of all n-state Turing machines in tree normal form, deciders that prove machines
non-halting, and tools for the cryptid Antihydra.

* `Enumerator` walks the tree normal form exactly as the Coq-BB5 proof does and classifies every
  node with a `Pipeline` of deciders. It runs incrementally (`advance`), can be split into
  shards for parallel workers, and keeps statistics, examples and holdouts.
* `deciders` contains `Loops` (cyclers and translated cyclers), `NGramCps` (n-gram closed
  position sets, with history and LRU augmentations) and `RepWl` (repeated word lists), ported
  from Coq-BB5 with its parameters. `Pipeline::coq` gives Coq-BB5's exact pipelines for BB(2),
  BB(3), BB(4) and BB(2,3).
* `cryptids::antihydra` checks Antihydra's high-level rules on the machine and iterates the rule
  itself with big integers.
* `uparrow` and `count` compute Knuth's up-arrows and the number of machines with n states.

```rust
use turing_bb::Enumerator;

let mut e = Enumerator::new(3, 2).unwrap();
let stats = e.run_to_completion();
assert_eq!(stats.nodes, 5_417);
assert_eq!(stats.holdouts, 0);
assert_eq!(stats.max_steps.map(|(s, _)| s), Some(21)); // S(3)
assert_eq!(stats.max_ones.map(|(s, _)| s), Some(6)); // Σ(3)
```

For four states the enumeration visits 858,909 nodes in about half a second (release build)
and finds S(4) = 107 and Σ(4) = 13 without holdouts.

The deciders are not formally verified. The tests compare their results node by node with the
enumerations verified in Coq by the bbchallenge collaboration: every node, its status and the
decider (with its parameters) agree for BB(2), BB(3), BB(2,3) and BB(4). `docs/deciders.md`
explains the tree normal form, each decider and its soundness argument.
