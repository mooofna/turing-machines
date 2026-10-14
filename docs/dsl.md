# The description language

Machines in this project are written in a small text language that follows the notation of the
course Theoretische Informatik und Logik at TU Dresden. A transition reads exactly as on the
exercise sheets:

```text
δ(q0, #) = (q1, #, R)
```

This document is the reference for the language: its grammar, what a description means, the
shorthand it offers, every diagnostic it reports, the bbchallenge standard format that the
project reads and writes next to it, and a worked example. The implementation lives in
`crates/turing-core/src/dsl/` and `crates/turing-core/src/spec.rs`.

## Contents

1. [A first example](#a-first-example)
2. [Grammar](#grammar)
3. [Semantics](#semantics)
4. [Declarations and inference](#declarations-and-inference)
5. [Shorthand](#shorthand)
6. [Comments and formatting](#comments-and-formatting)
7. [Diagnostics](#diagnostics)
8. [The standard format](#the-standard-format)
9. [Worked example: A_mul](#worked-example-a_mul)

## A first example

```text
// Binary increment, least significant bit first.
machine "Binary increment"
input "1101"
output binary lsb

Q = {q0, q1}
Σ = {0, 1}
Γ = {0, 1, ␣}
start = q0
F = {q1}

δ(q0, 1) = (q0, 0, R)  // 1 + 1 = 10: write 0 and carry on
δ(q0, 0) = (q1, 1, R)  // the carry stops here
δ(q0, ␣) = (q1, 1, R)  // the carry becomes a new most significant bit
```

Every line holds one item: a declaration or a transition. Only the transitions are required;
the description `δ(q0, x) = (q1, x, R)` on its own is a complete machine.

## Grammar

The grammar in EBNF. Each item stands on its own line; blank lines are ignored, and `//` starts a
comment that runs to the end of the line (`#` is a common tape symbol, so it cannot start
comments). Spaces and tabs between tokens are ignored; line breaks end items.

```ebnf
description = { line } ;
line        = [ item ] [ comment ] newline ;
comment     = "//" { any character except newline } ;

item        = "machine" STRING
            | "kind" ( "acceptor" | "transducer" | "enumerator" | "beaver" )
            | "input" STRING
            | "output" ( "word" | "unary" SYMBOL | "binary" ( "lsb" | "msb" ) )
            | "tapes" "=" NUMBER
            | "Q" "=" state_set
            | ( "Σ" | "Sigma" ) "=" symbol_set
            | ( "Γ" | "Gamma" ) "=" symbol_set
            | "blank" "=" SYMBOL
            | "start" "=" STATE
            | "F" "=" state_set
            | transition ;

state_set   = "{" [ STATE { "," STATE } ] "}" ;
symbol_set  = "{" [ SYMBOL { "," SYMBOL } ] "}" ;

transition  = ( "δ" | "delta" ) "(" STATE "," read ")" "=" "(" STATE "," write "," moves ")" ;
read        = read_entry | "(" read_entry { "," read_entry } ")" ;
read_entry  = SYMBOL | "{" SYMBOL { "," SYMBOL } "}" ;
write       = write_entry | "(" write_entry { "," write_entry } ")" ;
write_entry = SYMBOL | "*" ;
moves       = DIR | "(" DIR { "," DIR } ")" ;

DIR         = "L" | "R" | "N" ;
STATE       = ( letter | "_" ) { letter | digit | "_" | "'" } ;   (* but not a lone "_" *)
SYMBOL      = "'" character "'"
            | any single character except whitespace and ( ) { } , = ' * ; " ;
STRING      = '"' { character | '\"' | '\\' | '\n' | '\t' } '"' ;
NUMBER      = digit { digit } ;
```

Notes on the tokens:

* A **letter** is any Unicode letter, so `q0`, `q_Ausgabe`, `qa'` and `Ω` are state names.
  Keywords are only keywords at the start of a line: a state may be called `start`, `F` or `L`.
* A **symbol** is a single character. Unquoted, `x`, `#`, `0`, `+` and `Ä` are symbols. A
  character from the excluded list is written in single quotes: `'*'`, `','`, `' '`, `'''` (the
  quote itself) or `'"'`. Inside quotes every character stands for itself.
* `_` and `␣` (U+2423) unquoted both mean **the blank**, whatever glyph the blank has. Quoted,
  `'_'` and `'␣'` are ordinary glyphs.
* A lone `/` is a symbol; two slashes start a comment.
* Strings know the escapes `\"`, `\\`, `\n` and `\t`; any other backslash stands for itself.
  A string ends on its line.
* Line endings may be `\n` or `\r\n`. Spans in diagnostics are UTF-8 byte offsets.

The order of items is free; the formatter writes them in a fixed order (see
[Comments and formatting](#comments-and-formatting)).

## Semantics

A description defines a deterministic Turing machine M = (Q, Σ, Γ, δ, q0, F) with one to three
tapes.

* **Tape.** Every tape is infinite in both directions and blank outside the written part. The
  input is written on the first tape from position 0 on, the other tapes start blank, and every
  head starts on position 0. The blank glyph may appear inside an input word.
* **Steps.** In state q with symbols a1, ..., ak under the heads, the machine applies
  δ(q, (a1, ..., ak)) = (p, (b1, ..., bk), (D1, ..., Dk)): it writes every bi, moves every head
  by Di and enters p. `L` moves one cell left, `R` one cell right.
* **N moves.** `N` keeps a head in place. The course allows only `L` and `R`; `N` is an extension
  for convenience, and every machine of the gallery does without it.
* **Halting.** The machine halts when δ is undefined for the current state and symbols. Halting
  is not tied to F: a machine may halt in any state, and a final state may have transitions.
* **Final states** are used according to the kind of the machine.

| `kind` | Meaning |
|---|---|
| `transducer` (default) | Computes a function. When it halts, the output is the content of the first tape from its leftmost to its rightmost non-blank cell, with inner blanks written as the blank glyph. |
| `acceptor` | Accepts its input if it halts in a final state and rejects it if it halts in any other state. |
| `enumerator` | Prints a language. It starts on the blank tape; every step into a final state outputs the current word of the first tape. If the start state is final, the initial (empty) tape is output first. |
| `beaver` | Starts on the blank tape; only the number of steps and of non-blank cells matter. Busy beaver machines enter an explicit halting state with a real transition (`1RZ`), so that the step count matches Radó's definition. |

**Decoders.** `output` says how a transducer's output word is read as a number:

* `output word` (default): no number.
* `output unary x`: the number of occurrences of `x`.
* `output binary msb` or `output binary lsb`: a binary number written with the glyphs `0` and
  `1`, most or least significant bit first. The empty word is 0.

**Default input.** `input "..."` stores the word that a front end puts on the tape when the
machine is opened.

**Several tapes.** `tapes = k` with k from 1 to 3 gives every transition one entry per tape,
written as tuples:

```text
tapes = 2
δ(q1, (x, ␣)) = (q1, (x, x), (R, R))
```

The output is read from the first tape. Accelerated simulation, the space-time diagram, the
standard format and the busy beaver tools work on single-tape machines only.

## Declarations and inference

Every declaration is optional. A set that is declared is enforced: names outside it are errors.
A set that is not declared is inferred:

| Set | Inferred as |
|---|---|
| Q | The declared start state, then every state in order of first mention in the transitions, then the states of F that are not mentioned elsewhere. |
| q0 | The first state of Q. |
| Γ | The blank, a declared Σ, every glyph in the transitions in order of use, and (if Σ is not declared either) the glyphs of the input word. |
| Σ | Γ without the blank. |
| blank | `␣`. |
| tapes | 1. |

A declared Γ always contains the blank; if the declaration leaves it out, it is added at the end.
Inferring the sets of a spec twice changes nothing (`MachineSpec::infer`).

## Shorthand

Two kinds of shorthand keep tables short. The parser expands both into one transition per
combination, so the machine (and the formatted text) lists every transition explicitly.

**Read sets.** A set of symbols in a read position stands for one transition per symbol:

```text
δ(q2, {a, b}) = (q2, *, R)
```

is the same as

```text
δ(q2, a) = (q2, a, R)
δ(q2, b) = (q2, b, R)
```

With several tapes, every read entry may be a set, and the transitions are the cartesian
product, with the first tape varying slowest:

```text
tapes = 2
δ(q, ({a, b}, {c, d})) = (p, (*, x), (R, L))
```

gives δ(q, (a, c)), δ(q, (a, d)), δ(q, (b, c)) and δ(q, (b, d)), in this order.

**Star.** `*` in a write position writes back the symbol read on that tape. It may be used with
or without read sets.

## Comments and formatting

Comments are kept through parsing and formatting:

* Comment lines before the first item belong to the **header**. If the first item is a
  transition, the comment block directly above it (no blank line in between) is its **leading**
  comment instead.
* Comment lines directly above a later transition are its **leading** comment.
* A comment at the end of a transition line is the transition's **trailing** comment. With read
  sets, comments go to the first expanded transition.
* Comment lines after the last item are the **trailing** comments of the description.
* Comments that stand before a declaration, or at the end of a declaration line, have no place of
  their own; they are kept with the header.

`dsl::format` writes a description in canonical form, in this order:

1. the header comments, then `machine`, `kind`, `tapes`, `input` and `output`, each only if it
   differs from the default;
2. a blank line and the declared sets: `Q`, `Σ`, `Γ`, `blank` (if declared or not `␣`),
   `start`, and `F` whenever some state is final;
3. a blank line and the transitions in their order, read sets and `*` expanded. A transition
   with leading comments starts a new group after a blank line. Within a group, the `=` signs
   and the trailing comments are aligned;
4. a blank line and the trailing comments.

Symbols are quoted only where the parser would read them differently. `FormatStyle::Ascii`
writes `delta`, `Sigma`, `Gamma` and `_` instead of `δ`, `Σ`, `Γ` and `␣`. For every description
that parses without errors, parsing the formatted text gives the same machine, comments
included; only the source spans differ.

## Diagnostics

`dsl::parse` reads the whole text and reports every problem, not only the first: after a syntax
error it skips to the end of the line and continues with the next. A diagnostic has a stable
code, a message, a span (UTF-8 byte offsets), optionally a help text and related spans. Errors
`E001` to `E099` prevent the machine from being built; warnings `W001` to `W099` point at
something that is probably unintended. When some line could not be read, warnings (and `E005`)
are held back, because they may be caused by the missing line.

`MachineSpec::check` runs the same checks on a spec that did not come from text, for example
one edited in a table; its diagnostics point at whole transitions.

### E001

**Unexpected token.** The text does not fit the grammar. The message says what was expected.

```text
δ(q0, x) = (q1, x R)
error[E001]: expected `,`, found `R`
```

Typical help texts: "did you mean `start`?" for a misspelt keyword, "a transition starts with
`δ` or `delta`" for `d(q0, x) = ...`, "a symbol is a single character" for `δ(q0, xy)`, and
"`*` writes back the symbol read" for a `*` in a read position.

### E002

**Unterminated string.** A string literal has no closing `"` on its line.

```text
machine "A_mul
error[E002]: the string is not terminated
```

### E003

**Unknown state or symbol.** A transition, `start` or `F` names a state outside a declared Q, a
transition uses a glyph outside a declared Γ, or the input word holds a glyph outside Σ. A help
text suggests the closest name within two edits.

```text
Q = {q0, q1}
δ(q1, x) = (q2, x, R)
error[E003]: the state `q2` is not declared in Q
help: did you mean `q0`?
```

### E004

**Duplicate transition.** δ(q, a) is defined twice, which a deterministic machine does not allow.
A related span points at the first definition.

```text
δ(q0, x) = (q1, x, R)
δ(q0, x) = (q2, x, L)
error[E004]: δ(q0, x) is defined twice
```

### E005

**No states.** The description defines no state at all, so there is no start state.

```text
// nothing here
error[E005]: the machine has no states
```

### E006

**Blank in Σ.** The input alphabet must not contain the blank.

```text
Σ = {x, ␣}
error[E006]: the blank `␣` cannot be part of the input alphabet Σ
```

### E007

**Σ ⊄ Γ.** Every input symbol must belong to the tape alphabet.

```text
Σ = {x, y}
Γ = {x, ␣}
error[E007]: the input symbol `y` is not in the tape alphabet Γ
```

### E008

**Invalid direction.** A move is something other than `L`, `R` or `N`.

```text
δ(q0, x) = (q1, x, S)
error[E008]: `S` is not a direction
help: use L, R or N
```

### E009

**Too large.** The machine exceeds a limit: 1 to 3 tapes, at most 4096 states, at most 64
symbols, a transition table |Q| × |Γ|^k of at most 2²² entries, or more than 2¹⁸ transitions after
expanding read sets.

```text
tapes = 4
error[E009]: a machine has 1 to 3 tapes, not 4
```

### E010

**Wrong number of tape entries.** A transition does not have one read symbol, one written symbol
and one move per tape.

```text
δ(q0, (x, y)) = (q1, (x, y), (R, R))
error[E010]: the transition reads 2 symbols, but the machine has 1 tape
help: declare `tapes = 2` for a machine with 2 tapes
```

### E011

**Duplicate declaration.** A declaration appears twice, or a set lists an element twice.

```text
Q = {q0, q1, q0}
error[E011]: the state `q0` is listed twice in Q
```

### E012

**Name the language cannot write.** A state name that is not a valid STATE token (for example
`q 1` or the empty name), or a line break as a symbol. Text never produces this error; it guards
specs built by other means, such as an editor, so that they can always be formatted.

```text
error[E012]: `q 1` is not a valid state name
```

### W001

**Unreachable state.** No chain of transitions leads from the start state to this state.

```text
Q = {q0, q1, q9}
δ(q0, x) = (q1, x, R)
warning[W001]: the state `q9` cannot be reached from the start state `q0`
```

### W002

**No final state.** An acceptor without final states accepts no word, and an enumerator without
final states outputs nothing.

```text
kind acceptor
δ(q0, a) = (q1, a, R)
warning[W002]: the acceptor has no final state, so it accepts no word
```

### W003

**Unused symbol.** A symbol of a declared Γ or Σ is never read or written by any transition.

```text
Γ = {x, y, ␣}
δ(q0, x) = (q0, x, R)
warning[W003]: the symbol `y` is never read or written
```

## The standard format

The busy beaver community writes machines in the bbchallenge standard text format, for example
the BB(5) champion:

```text
1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA
```

States are the letters `A`, `B`, ... in order, separated by `_`; `A` is the start state. Each
state lists one transition per read symbol `0`, `1`, ...: the written digit, the move `L` or `R`,
and the next state. `---` marks an undefined transition. A next-state letter beyond the last
state (usually `Z`) is the halting state: the transition is executed and counted, then the
machine halts.

`standard::parse_standard` and `standard::spec_from_standard` read this format into a `beaver`
machine with blank `0` and Γ = {0, ..., k - 1}, where k is the number of transitions per state.
All next-state letters beyond the last state map to a single final state `Z` without
transitions, added only if some transition enters it. Surrounding whitespace is ignored; any
other deviation is a syntax error with a byte offset. The spec declares Q (so the states keep
their letter order), Γ and the blank. In the description language the BB(2) champion
`1RB1LB_1LA1RZ` reads:

```text
kind beaver

Q = {A, B, Z}
Γ = {0, 1}
blank = 0
F = {Z}

δ(A, 0) = (B, 1, R)
δ(A, 1) = (B, 1, L)
δ(B, 0) = (A, 1, L)
δ(B, 1) = (Z, 1, R)
```

`standard::to_standard` writes a machine back. It requires:

* one tape, and only the moves `L` and `R`;
* at most 10 symbols, which must be exactly the digits `0` to `k - 1` with blank `0` (the format
  writes glyphs, so a machine over `x` and `#` is rejected rather than silently renamed);
* at most 26 states, or 25 plus the halting state;
* at most one halting state: a final state, other than the start state, that has no transitions
  but is entered by one. It is written as `Z`. Two such states cannot be told apart in the
  format. A state that is not final keeps a letter of its own even if it has no transitions,
  which halts the machine just the same.

State names are not kept: the start state becomes `A` and the other states follow in their
order. For every text that `parse_standard` accepts, `to_standard` gives back the same text with
every halting letter written as `Z`.

## Worked example: A_mul

Exercise sheet 1, task 3 of the 2024 course asks for a machine A_mul that multiplies two
numbers in unary. The input `#x^m#x^n#` should become x^(m × n). The full machine is
`machines/course/unary-multiplication.tm`; this section explains it.

**Idea.** For every x of the first factor, copy the second factor once behind the third `#`.
Marks keep track of the progress: an x of the first factor that has been dealt with becomes `X`,
and so does each x of the second factor while it is being copied.

**Header and sets.**

```text
machine "A_mul"
input "#xxx#xx#"
output unary x

Q = {q0, q1, q2, q3, q4, q5, q6, q7, q8, q9, q10, q11, q12}
Σ = {x, #}
Γ = {x, X, #, ␣}
start = q0
F = {q12}
```

`output unary x` makes the front ends show the result as a number: `#xxx#xx#` gives 6.

**Outer loop over the first factor.** q1 looks at the next x of the first factor. It marks it
and runs right to the second factor (q2); if there is none left, it starts the cleanup.

```text
δ(q0, #) = (q1, #, R)  // skip the first separator
δ(q1, x) = (q2, X, R)
δ(q1, #) = (q10, ␣, L)
δ(q2, x) = (q2, x, R)  // skip the rest of the first factor
δ(q2, #) = (q3, #, R)
```

**Inner loop over the second factor.** q3 marks the next x of the second factor and appends one
x to the product (q4, q5), then walks back to the last mark (q6, q7). When the second factor is
used up, q8 restores it to x and q9 returns to the last mark of the first factor.

```text
δ(q3, x) = (q4, X, R)
δ(q3, #) = (q8, #, L)
δ(q4, x) = (q4, x, R)
δ(q4, #) = (q5, #, R)
δ(q5, x) = (q5, x, R)
δ(q5, ␣) = (q6, x, L)
δ(q6, x) = (q6, x, L)
δ(q6, #) = (q7, #, L)
δ(q7, x) = (q7, x, L)
δ(q7, X) = (q3, X, R)
δ(q8, X) = (q8, x, L)
δ(q8, #) = (q9, #, L)
δ(q9, x) = (q9, x, L)
δ(q9, X) = (q1, X, R)
```

**Cleanup.** The tape is now `#X^m␣x^n#x^(m × n)`. q10 erases the marks and the first `#` going
left, q11 erases everything up to and including the third `#` going right, and the machine halts
in q12 on the first x of the product. A read set handles the blanks and the x of the second
factor at once:

```text
δ(q10, X) = (q10, ␣, L)
δ(q10, #) = (q11, ␣, R)
δ(q11, {x, ␣}) = (q11, ␣, R)
δ(q11, #) = (q12, ␣, R)
```

The tape is infinite in both directions, so the product can stay where it was built; the 2024
version of this machine instead shifted it left and stopped with `N` moves. The golden tests run
A_mul for all m and n from 0 to 8; `#xxx#xx#` takes 106 steps.

**A run in course notation.** The configurations u q v of the first steps on `#x#x#`, where q
stands left of the symbol under the head:

```text
q0 #x#x#
# q1 x#x#
#X q2 #x#
#X# q3 x#
#X#X q4 #
#X#X# q5 ␣
#X#X q6 #x
```
