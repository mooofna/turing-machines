# turing-cli

`tm`, the command-line tool of the turing-machines workspace. It runs and traces Turing machines
written in the notation of the TheoLog course at TU Dresden or in the bbchallenge standard format,
explains mistakes in descriptions with the lines they refer to, formats and converts descriptions,
draws space-time and state diagrams, enumerates all machines of a size to find the busy beaver,
and explores the cryptid Antihydra.

```sh
cargo install --path crates/turing-cli   # from the repository root
tm help
```

Every example below runs from the repository root and is checked by the tests of this crate, so
the outputs are exact. Wall-clock times are left out because the tests set `TM_NO_TIMING=1`.

## Running machines

The exercise that started the project, TheoLog 2024, sheet 1, task 3: read two numbers, write them
in unary as `#xxx#xxxx#`, multiply them with the machine A_mul and read the result back.

```console
$ tm run machines/course/unary-multiplication.tm --unary 3 4
12

```

`--unary` and `--binary` print only the result. `--format text` gives the whole report:

```console
$ tm run machines/course/unary-multiplication.tm --unary 3 4 --format text
machine   A_mul
          transducer, 1 tape, 13 states, 4 symbols, 24 transitions
input     #xxx#xxxx#
outcome   halted after 276 steps in state q12
output    xxxxxxxxxxxx (12)
tape      q12 xxxxxxxxxxxx
non-blank 12

```

Single-tape machines run with chain steps, which jump over whole runs of equal symbols. The
5-state busy beaver takes 47,176,870 steps and finishes in milliseconds:

```console
$ tm run --standard 1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA
machine   1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA
          beaver, 1 tape, 5 states and the halting state Z, 2 symbols, 10 transitions
outcome   halted after 47,176,870 steps in state Z
tape      1 Z 01001001001001001001001001001001001001001001001001001001001…
non-blank 4,098

```

Enumerators print a word whenever they enter a final state:

```console
$ tm run machines/course/an-bn.tm --max-words 4 --format value

ab
aabb
aaabbb

```

## Tracing a run

`tm trace` writes the configurations `u q v` of the course, one step per line:

```console
$ tm trace machines/course/binary-increment.tm --input 1011
 0    q0 1011
 1 ⊢  0 q0 011
 2 ⊢  01 q1 11

halted after 2 steps in state q1, output 0111 (14)

```

## Checking descriptions

`tm check` reports every problem at once, with the source lines, a suggestion where there is one,
and the place a duplicate was first defined. The exit code is 1 if there is an error.

```console
$ tm check typo.tm
? 1
error[E004]: δ(q0, x) is defined twice
  ╭▸ typo.tm:7:1
  │
6 │ δ(q0, x) = (q0, X, R)
  │ ───────────────────── first defined here
7 │ δ(q0, x) = (q1, X, L)
  │ ━━━━━━━━━━━━━━━━━━━━━
  │
  ╰ help: a deterministic machine has at most one transition per state and symbol

error[E003]: the state `qq1` is not declared in Q
  ╭▸ typo.tm:8:13
  │
8 │ δ(q0, ␣) = (qq1, ␣, L)
  │             ━━━
  │
  ╰ help: did you mean `q1`?

error[E008]: `Q` is not a direction
  ╭▸ typo.tm:9:20
  │
9 │ δ(q1, X) = (q1, X, Q)
  │                    ━
  │
  ╰ help: use L, R or N

error: `typo.tm` is invalid: 3 errors

```

`tm fmt` writes the canonical form (`--write` in place, `--check` for CI, `--ascii` for plain
keyboards), and `tm convert` moves between the description language, the standard format and
JSON:

```console
$ tm convert --standard 1RB1RZ_1LB0RC_1LC1LA --to dsl
kind beaver

Q = {A, B, C, Z}
Γ = {0, 1}
blank = 0
F = {Z}

δ(A, 0) = (B, 1, R)
δ(A, 1) = (Z, 1, R)
δ(B, 0) = (B, 1, L)
δ(B, 1) = (C, 0, R)
δ(C, 0) = (C, 1, L)
δ(C, 1) = (A, 1, L)

```

## Busy beavers

`tm enumerate` walks the tree normal form of the Coq-BB5 proof and sorts every machine into
halting, proved non-halting by one of the deciders, or holdout:

```console
$ tm enumerate --states 3
Busy beaver search over 3 states and 2 symbols, in tree normal form

stage                                         resolved   remaining
nodes of the tree normal form                                5,417
halting                                          1,772       3,645
non-halting: cyclers and translated cyclers      3,492         153
non-halting: n-gram closed position sets           135          18
non-halting: n-gram CPS with history                18           0
non-halting: n-gram CPS with LRU                     0           0
non-halting: repeated word lists                     0           0
holdouts                                                         0

S(3) = 21   1RB1RZ_1LB0RC_1LC1LA
Σ(3) = 6    1RB1LC_1LA1RB_1LB1RZ

```

`tm enumerate --states 4` does the same for 858,909 nodes in about half a second (release build)
and finds S(4) = 107 and Σ(4) = 13 with no holdouts. `tm decide` asks the same deciders about a
single machine:

```console
$ tm decide 1RB0LB_1LA0RB
machine   1RB0LB_1LA0RB
verdict   never halts (decider: cyclers and translated cyclers)

```

## Antihydra

`tm antihydra` checks the rule of the 6-state cryptid on the machine itself and then iterates the
rule with big integers. Whether it ever halts is an open problem, and the output says so.

```console
$ tm antihydra --check-rows 6
machine   1RB1RA_0LC1LE_1LD1LC_1LA0LB_1LF1RE_---0RA

Rule applications on the machine, A(a, b) = 0^∞ 1^a 0 1^b E> 0^∞

  n   a    b   at step   steps   predicted   agrees
  0   0    4        11      11          11   yes
  1   2    8        58      47          47   yes
  2   4   14       169     111         111   yes
  3   6   23       419     250         250   yes
  4   5   36       919     500         500   yes
  5   7   56     2,128   1,209       1,209   yes

check     6 of 6 rule applications on the machine agree with the rule
rule      100,000 iterations from h = 8, c = 0
          h even 50,159 times (c + 2), odd 49,841 times (c - 1)
          balance c = 50,477, smallest 2
          h has 58,500 bits

The balance did not become negative in 100,000 iterations.
Whether it ever does is an open problem, so nobody knows whether Antihydra halts.
The question is close to Mahler's 3/2 problem.

```

## Pictures

```console,ignore
$ tm render --standard 1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA --spacetime bb5.png
$ tm render machines/course/unary-multiplication.tm --diagram mul.svg --theme dark
```

The space-time diagram uses the colours of the website: ink `#111111` and accent `#215caf` on
white, or `--theme dark` with `#f1f1f1` and `#7ba7f0` on `#0e0e0e`.

## Conventions

* **Input.** `--input WORD` puts a word on the first tape; `_` stands for the blank unless the
  machine uses `_` itself. Without an input option the machine's own example input is used.
  `--unary 3 4` writes the course encoding `#xxx#xxxx#` with the machine's tally symbol (a single
  number without `#` if the machine does not use `#`), and `--binary 11` writes a number in the bit
  order of the machine's output.
* **Counts.** Options that take a number of steps accept `1000`, `1_000_000`, `1e9` and `10^9`.
* **Colours.** On by default on terminals; `--color never`, `--color always` and `NO_COLOR` change
  that. Diagnostics go to standard error, results to standard output.
* **Exit codes.** 0 for success, 1 for an invalid machine or a failed check (such as
  `tm fmt --check`), 2 for usage and I/O errors.
