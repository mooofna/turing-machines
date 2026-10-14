# turing-bb

Enumeration of all n-state Turing machines in tree normal form, deciders that prove machines
non-halting, and tools for the cryptid Antihydra.

* `Enumerator` walks the tree normal form exactly as the Coq-BB5 proof does and classifies every
  node with a `Pipeline` of deciders.
* `deciders` contains `Loops` (cyclers and translated cyclers), `NGramCps` (n-gram closed
  position sets, with history and LRU augmentations) and `RepWl` (repeated word lists).
* `cryptids::antihydra` checks Antihydra's high-level rules on the machine and iterates the rule
  itself with big integers.
* `uparrow` and `count` compute Knuth's up-arrows and the number of machines with n states.

The deciders are not formally verified. The tests compare their results node by node with the
enumeration verified in Coq by the bbchallenge collaboration.
