# Re-deriving BB(2), BB(3) and BB(4)

The crate `turing-bb` enumerates every Turing machine with n states and two symbols (and with
two states and three symbols) in tree normal form, decides for each one whether it halts from
the blank tape, and reads off the busy beaver values S(n) (the largest number of steps) and
Σ(n) (the largest number of ones left on the tape). The enumeration, the deciders and their
parameters are those of Coq-BB5, the Coq proof of BB(5) = 47,176,870 by mxdys and the
bbchallenge collaboration [1, 2]. The Rust deciders are not formally verified; instead, their
results are compared with the enumerations that Coq-BB5 verified, node by node, and they agree
on every node. The values themselves are classical: Σ(2) = 4 is due to Radó, S(2) = 6 and both
values for three states to Lin (published with Radó in 1965), BB(4) = 107 to Brady [5], and
BB(2,3) = 38 to Lafitte and Papazian.

| Size | Nodes | Halting | Non-halting | Holdouts | S | Σ | Time |
|---|---:|---:|---:|---:|---:|---:|---:|
| 1 state | 3 | 1 | 2 | 0 | 1 | 1 | under 0.1 ms |
| 2 states | 61 | 19 | 42 | 0 | 6 | 4 | 0.02 ms |
| 3 states | 5,417 | 1,772 | 3,645 | 0 | 21 | 6 | 1.8 ms |
| 4 states | 858,909 | 249,693 | 609,216 | 0 | 107 | 13 | 0.54 s |
| 2 states, 3 symbols | 6,019 | 1,816 | 4,203 | 0 | 38 | 9 | 3.4 ms |

Times are medians of `cargo bench -p turing-bb` on an Apple M1 Pro (native, release build, one
thread); see `docs/benchmarks.md`.

## Tree normal form

Enumerating all (4(n + 1))^(2n) machines is wasteful: most of them differ only in transitions
they never use. The tree normal form (TNF) [3] enumerates *partial* machines instead, in which
some transitions are undefined, and defines a transition only when the machine actually needs
it.

* The root is the machine with every transition undefined.
* Each node is classified. If, started on the blank tape, it reaches an undefined transition
  δ(q, a) after t steps, it is a *halting node*: completing δ(q, a) with a transition into the
  halting state gives a machine that halts after t + 1 steps, with as many non-blank cells as
  the node had plus one if a was blank (the halting transition writes 1). If the node has
  further undefined transitions, it gets one child per way to define δ(q, a): every symbol to
  write, both directions and every next state from A up to the first state that no transition
  enters yet. If δ(q, a) was the last undefined transition, the node has no children, since
  its completions never halt.
* If the node is proved never to halt, it is a leaf: every machine that agrees with it on its
  defined transitions runs identically and never halts either.
* At the root only moves to the right are used: a machine whose first move is to the left is
  the mirror image of one whose first move is to the right.

The restriction of next states makes the states appear in the order in which they are first
entered, so machines that differ by a renaming of states B, C, ... are enumerated once. The
enumeration is exhaustive for halting machines. Take any machine M that halts from the blank
tape, mirrored if its first move is to the left (which changes neither its step count nor its
ones) and with its states renamed in order of first use. The partial machine P made of exactly
the transitions M uses before its halting transition, with that transition left undefined, is
a node of the tree: every ancestor of P is a restriction of P, follows the same run until it
meets an undefined transition, meets one no later than M halts, and is therefore a halting
node that gets expanded towards P. P halts after one step fewer than M, and its completion with
`1RZ` leaves at least as many ones as M, whatever M's halting transition writes. Conversely,
every value read off a halting node is attained by a machine, the node completed with `1RZ`.
So if every node is classified (no holdouts), S(n) and Σ(n) are the maxima over the halting
nodes.

This is exactly the enumeration of Coq-BB5 (`TNF.v`, `BB4_TNF_Enumeration.v`), including the
order in which nodes are visited (depth first, children by next state, then direction L before
R, then symbol written) and two details that fix the node counts:

* Coq-BB5 keeps both `0RB` and `1RB` (and `0RA`, `1RA`) as first transitions, unlike the
  smaller variant TNF-1RB used elsewhere [3].
* The root is counted as a node, and the L children of the root are never created.

The counts above, 61, 5,417 and 858,909, are those of the Coq-BB5 READMEs.

### Step bounds

The first stage of every pipeline simulates each node for N steps, and Coq-BB5 sets N to the
busy beaver value itself: 6, 21, 107 and 38. This does not presuppose the result. A node that
reached its undefined transition only after N or more steps would not be recognised as halting,
and no sound decider could prove it non-halting, so it would remain a holdout. An enumeration
without holdouts therefore certifies that every halting node halts within N steps, and the
maximum is then computed from the actual step counts. The same pipelines are used here so that
the results can be compared with Coq-BB5 node by node; a larger N would decide more nodes in the
first stage and fewer later, with the same S(n) and Σ(n).

## Deciders

A decider receives a partial machine and answers *halts* (with the step count, the number of
non-blank cells and the undefined transition reached), *never halts*, or *unknown*. A pipeline
applies deciders in order until one answers. Every *never halts* answer must be backed by a
proof; the arguments below are the ones Coq-BB5 formalises.

### Loops (`loop1_decider`)

The machine is simulated for N steps. If it meets an undefined transition at a time t < N, it
halts. Otherwise the decider looks, once, at the history c(0), ..., c(N) of configurations,
each reduced to the *state-symbol pair* σ(t) (state and symbol under the head), the head
position x(t) and whether the head is on the leftmost or rightmost cell visited so far. It
searches for a period p with 1 ≤ p ≤ N/2 such that σ(N) = σ(N − p) = σ(N − 2p) and, walking
backwards, σ(N − j) = σ(N − p − j) for j = 0, 1, ..., J for some J ≥ p at which, with
a = N − p − J, b = a + p and d = x(N) − x(N − p),

* d = 0 and x(a) = x(b): a *cycler*, which returns to the same configuration forever;
* d > 0, the head is on the rightmost visited cell at time a, and x(a) < x(b): a *translated
  cycler*, which repeats the same behaviour further and further to the right;
* d < 0: the mirror image.

*Soundness.* The move at time t depends only on σ(t), so σ(a + i) = σ(b + i) for 0 ≤ i ≤ J
gives x(b + i) − x(a + i) = d. One shows by induction on t that σ(t + p) = σ(t) and
x(t + p) = x(t) + d for all t ≥ a; then every state-symbol pair after time a already occurred
with a defined transition, and the machine never halts. For the induction step, take the cell
y = x(t + p) and its last visit u before t + p. If u ≥ b, the cell x(t) = y − d was last
visited at u − p and written by the same transition. Otherwise y was not visited during
[b, t + p), hence x(t) was not visited during [a, t). For a cycler this contradicts
x(t − p) = x(t). For a translated cycler moving right, either x(t) ≤ x(a): then x(t) < x(a), because x(t) was
not visited at time a, so every position during [a, t) is right of x(t) (the head moves one cell
at a time), among them x(t − p), and x(t) = x(t − p) + d > x(t) is a contradiction; or
x(t) > x(a), the rightmost cell visited up to time a, and then neither x(t) nor y was ever
visited before, so both are blank. The requirement J ≥ p guarantees
t − p ≥ a in this argument. The full argument is in the rustdoc of `deciders::loops`.

### n-gram closed position sets (`NGramCPS_decider`)

Introduced by Nathan Fenner [4]. The decider over-approximates the set of reachable
configurations by their local views: a set M of *local contexts* (state, symbol under the head,
the `len_l` cells to its left and the `len_r` cells to its right), a set L of left n-grams and a
set R of right n-grams. Initially they contain the blank context in state A and the blank
n-grams. Processing a context with a transition to the right adds the left n-gram that moves
away from the head to L, and adds to M the context after the move for every symbol x that can
appear at the far end of the new right window, that is every x for which the right n-gram
formed by the remaining cells and x is in R. Moves to the left are symmetric. The sets grow
until a full pass changes nothing.

*Soundness.* Call a configuration covered if its local context is in M and every window of
`len_l` (or `len_r`) consecutive cells beyond the context, read away from the head, is in L
(or R). The blank configuration is covered, and closure under processing makes the successor
of a covered configuration covered. Hence every reachable configuration is covered, and if no
context in M has an undefined transition, the machine never halts.

*Augmentations.* The same decider runs on an augmented machine whose symbols are pairs
(symbol, history), where the history of a cell records the state-symbol pairs of the
transitions that wrote it: the last k of them (`History(k)`, Coq-BB5's `NGramCPS_decider_impl1`)
or a list of distinct pairs, most recently used first (`Lru`, `NGramCPS_LRU_decider`). The
augmented machine makes the same moves and state changes as the original, so it halts if and
only if the original does, while its n-grams carry more information. Augmented symbols are
interned as small integers.

*Faithfulness.* The decider has a gas parameter that bounds the work. To decide exactly the same
machines as Coq-BB5 with the same parameters, the Rust port reproduces its search order and gas
accounting: passes over all known contexts, most recent first, newly found contexts processed
next, one unit of gas per processed context and per completed pass.

### Repeated word lists (`RepWL_ES_decider`)

The tape on each side of the head is cut into words of `len` cells, counted outwards from the
head, and runs of equal words are compressed: k copies of a word w are kept as wᵏ while
k < m and become wᵐ⁺ ("at least m copies") from m copies on. An abstract configuration is the
word list behind the head, the word list in front of it, the state and the direction the head
faces. An abstract step pops the first word in front (a run wᵏ⁺ may leave either wᵏ⁻¹ or wᵏ⁺
behind, so steps branch), simulates the machine on that word alone for at most T steps until
the head leaves it, and pushes the updated word on the side where the head left, turning
around if it left backwards. A pushed word joins a run of the same word and saturates at wᵐ⁺.

*Soundness.* Each abstract configuration stands for all concrete configurations whose tape
matches it, and every concrete configuration reaches, in at least one step, one that matches
a successor of its abstraction. If the set of abstract configurations reachable from the blank
one is explored completely within the gas bound, without an undefined transition and without a
word simulation exceeding T steps, it is a closed set containing the start and no halting
configuration. Because the reachable set is explored completely, the answer does not depend
on the search order.

## Pipelines

The pipelines are those of Coq-BB5's `BB*_Deciders_Pipeline.v`. Parameters of n-gram CPS are
(history length, left length, right length, gas); of RepWL (word length, m, T, gas).

| Stage | BB(2) | BB(3) | BB(4) | BB(2,3) |
|---|---|---|---|---|
| Loops | N = 6 | N = 21 | N = 107 | N = 38 |
| n-gram CPS, plain | (1, 1, 100) | (1, 1, 100), (2, 2, 200), (3, 3, 400) | (1, 1, 100), (2, 2, 200), (3, 3, 400) | (1, 1, 400), (2, 2, 800), (3, 3, 400) |
| n-gram CPS, history | | (2, 2, 2, 1600) | (2, 2, 2, 1600), (2, 3, 3, 1600), (4, 2, 2, 600), (4, 3, 3, 1600), (6, 2, 2, 3200), (6, 3, 3, 3200), (8, 2, 2, 1600), (8, 3, 3, 1600) | |
| n-gram CPS, LRU | | | (2, 2, 10000) | |
| n-gram CPS, history | | | (10, 4, 4, 10000) | |
| RepWL | | | (4, 3, 320, 10000) | (2, 3, 320, 400) |

`Pipeline::coq(states, symbols)` returns these pipelines, `Pipeline::standard()` is the BB(4)
pipeline, and `Enumerator::new` uses the Coq-BB5 pipeline where one exists and the standard
one otherwise (for one state, for instance). The BB(5) proof uses a larger pipeline with
further deciders and hand-checked certificates for individual machines; BB(4) needs only
these three techniques. The first stage of the BB(4) pipeline is a single Loops stage with
107 steps; Coq-BB5 defines but does not use a second Loops stage with 4,100 steps.

## Results

Machines proved non-halting per stage, from `cargo run --release -p turing-bb --example funnel`
(arguments: states, symbols). Every count equals the corresponding count in the Coq-BB5 READMEs.

**BB(2)**: 61 nodes, 19 halting.

| Stage | Non-halting |
|---|---:|
| Loops, N = 6 | 29 |
| n-gram CPS (1, 1, 100) | 13 |

**BB(3)**: 5,417 nodes, 1,772 halting.

| Stage | Non-halting |
|---|---:|
| Loops, N = 21 | 3,492 |
| n-gram CPS (1, 1, 100) | 80 |
| n-gram CPS (2, 2, 200) | 35 |
| n-gram CPS (3, 3, 400) | 20 |
| n-gram CPS with history (2, 2, 2, 1600) | 18 |

**BB(4)**: 858,909 nodes, 249,693 halting.

| Stage | Non-halting | Remaining after the stage |
|---|---:|---:|
| Loops, N = 107 | 588,373 | 20,843 |
| n-gram CPS (1, 1, 100) | 11,644 | 9,199 |
| n-gram CPS (2, 2, 200) | 4,759 | 4,440 |
| n-gram CPS (3, 3, 400) | 1,731 | 2,709 |
| n-gram CPS with history (2, 2, 2, 1600) | 2,296 | 413 |
| n-gram CPS with history (2, 3, 3, 1600) | 161 | 252 |
| n-gram CPS with history (4, 2, 2, 600) | 174 | 78 |
| n-gram CPS with history (4, 3, 3, 1600) | 29 | 49 |
| n-gram CPS with history (6, 2, 2, 3200) | 14 | 35 |
| n-gram CPS with history (6, 3, 3, 3200) | 10 | 25 |
| n-gram CPS with history (8, 2, 2, 1600) | 8 | 17 |
| n-gram CPS with history (8, 3, 3, 1600) | 3 | 14 |
| n-gram CPS with LRU (2, 2, 10000) | 8 | 6 |
| n-gram CPS with history (10, 4, 4, 10000) | 4 | 2 |
| RepWL (4, 3, 320, 10000) | 2 | 0 |

"Remaining" counts the nodes that are neither halting nor resolved by this or an earlier
stage. The two machines that only repeated word lists resolve are `1RB1LA_1LA0RC_1LD1RC_---0LA`
and `1RB0RB_1LC1RB_---0LD_1RA1LD`, as in Coq-BB5.

**BB(2,3)**: 6,019 nodes, 1,816 halting.

| Stage | Non-halting |
|---|---:|
| Loops, N = 38 | 3,861 |
| n-gram CPS (1, 1, 400) | 288 |
| n-gram CPS (2, 2, 800) | 36 |
| n-gram CPS (3, 3, 400) | 8 |
| RepWL (2, 3, 320, 400) | 10 |

The records, as complete machines with the halting transition written as `1RZ`. Among the
machines with the largest S the record is the one that leaves the most ones, and among those
with the largest Σ the one that runs longest; remaining ties go to the smaller transition table.
This order does not depend on the enumeration order, so the four workers of the website, which
each enumerate a part of the tree, report the same machines as a single run:

| Size | S | Machine | Σ | Machine |
|---|---:|---|---:|---|
| 1 state | 1 | `1RZ---` | 1 | `1RZ---` |
| 2 states | 6 | `1RB1LB_1LA1RZ` | 4 | `1RB1LB_1LA1RZ` |
| 3 states | 21 | `1RB1RZ_1LB0RC_1LC1LA` | 6 | `1RB1RZ_0RC1RB_1LC1LA` |
| 4 states | 107 | `1RB1LB_1LA0LC_1RZ1LD_1RD0RA` | 13 | `1RB1LB_1LA0LC_1RZ1LD_1RD0RA` |
| 2 states, 3 symbols | 38 | `1RB2LB1RZ_2LA2RB1LB` | 9 | `1RB2LB1RZ_2LA2RB1LB` |

## Comparison with Coq-BB5

Coq-BB5 extracts its enumerations as CSV files that list every node in enumeration order with
its status and the identifier of the decider that classified it, for example
`0RB---_1LA0LA,nonhalt,NGRAM_CPS_IMPL2_params_1_1_100`. The tests compare the Rust enumeration
with these files row by row: the same machine, the same status and the same decider with the
same parameters.

* BB(2), BB(3) and BB(2,3): the CSV files are part of the repository
  (`crates/turing-bb/tests/data`, with their SHA-256 hashes) and `tests/coq_csv.rs` checks all
  61, 5,417 and 6,019 rows on every test run.
* BB(4): the 45 MB file is downloaded separately; `tests/coq_bb4.rs` checks all 858,909 rows
  when the environment variable `COQ_BB4_CSV` points to it (about one second in release mode).

All rows agree. In addition, `tests/soundness.rs` runs every decider with every parameter set
above on every halting node for up to four states (and Loops with every smaller step budget),
and none of them claims that a halting node never halts; every node proved non-halting for up
to three states runs for 100,000 steps without halting; and `tests/random_machines.rs` checks
every verdict on random machines against plain simulation.

The deciders here are ports of programs whose correctness Coq-BB5 proves; the ports themselves
are tested, not proved. The agreement with the verified enumeration on every node is strong
evidence that they compute the same functions on these inputs, and the busy beaver values are
read from the halting nodes, whose step counts are exact by simulation.

## References

1. The bbchallenge Collaboration, J. Blanchard, D. Briggs, K. Deka, N. Fenner, Y. Forster,
   G. Georgiev, M. L. House, R. Hunter, Iijil, M. Kądziołka, P. Kropitz, S. Ligocki, mxdys,
   M. Naściszewski, savask, T. Stérin, C. Xu, J. Yuen, T. Zimmermann. *Determination of the
   fifth Busy Beaver value.* arXiv:2509.12337, 2025.
2. mxdys. *Coq-BB5 release v1.0.0.* Zenodo, 2025, doi:10.5281/zenodo.17061968.
   <https://github.com/ccz181078/Coq-BB5>
3. bbchallenge wiki, *Tree Normal Form*. <https://wiki.bbchallenge.org/wiki/Tree_Normal_Form>
4. N. Fenner. *bb-simple-n-gram-cps*. <https://github.com/Nathan-Fenner/bb-simple-n-gram-cps>
5. A. H. Brady. *The determination of the value of Rado's noncomputable function Σ(k) for
   four-state Turing machines.* Mathematics of Computation 40(162), 647 to 665, 1983.
