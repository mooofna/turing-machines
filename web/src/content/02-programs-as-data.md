# Programs as data

A Turing machine is a finite object. Its states, its alphabets and its transition table can be written down as a single word over a fixed alphabet, and that word can be the input of another machine. Turing used this observation to construct one machine that can carry out the computation of every other machine [@turing1936]. The same observation is the starting point of chapter 3, which asks what one machine can find out about another.

## Machines as words

An encoding assigns to every machine $M$ a word $\mathrm{enc}(M)$ over a fixed alphabet, for instance $\{0, 1\}$. It must be possible to reconstruct $M$ from $\mathrm{enc}(M)$ and to decide whether a given word encodes a machine at all. A typical encoding numbers the states and symbols and writes each entry of the transition table as a short block of digits; the details below give one such encoding. A machine together with an input word $w$ is written $\mathrm{enc}(M)\#\#w$, following the notation of the course [@theolog2024].

Since every machine is described by a finite word, there are only countably many machines. The languages over a non-empty alphabet, on the other hand, form an uncountable set, by the same diagonal argument that shows that the real numbers are uncountable. All but countably many languages are therefore accepted by no Turing machine. This counting argument shows that such languages exist without naming one; chapter 3 gives an explicit example.

## Universal machines

A universal machine $U$ receives $\mathrm{enc}(M)\#\#w$ as input and simulates $M$ on $w$ step by step. It halts exactly when $M$ halts on $w$ and accepts exactly when $M$ accepts $w$, so that

$$L(U) = \{\, \mathrm{enc}(M)\#\#w \mid w \in L(M) \,\}.$$

Turing's paper contains the first construction of such a machine [@turing1936]. A universal machine stores the description of the simulated machine on its tape and consults it in every step, in the same way that a stored-program computer keeps its program in memory. The simulator on this site is a program of this kind as well: it reads a machine description and executes it.

## A text format for machines

For machines with few states, the busy beaver community uses a compact text format [@bbchallenge-format]. States are named by the letters A, B, C, and so on, and A is the start state. Symbols are digits, and 0 is the blank. The table is written one state after another, separated by underscores. For each state there is one entry of three characters per symbol, in the order 0, 1, and so on: the symbol to write, the direction L or R, and the next state. The entry `---` is undefined, and the machine halts when it reaches it. A next state that is not one of the machine's states, conventionally Z, also means that the machine halts after this transition.

As an example, `1RB1LB_1LA1RZ` describes a machine with two states. In state A it writes 1 in both cases and continues in state B, moving right after reading 0 and left after reading 1. In state B, after reading 0, it writes 1, moves left and continues in A; after reading 1, it writes 1, moves right and halts. This machine is a two-state busy beaver champion, which chapter 4 returns to. Machines with more symbols use the same format with more entries per state.

<<figure:fig2.1 | Fig. 2.1. One machine in three linked views: a state diagram, a transition table and a line in the standard text format. Edit the text or the table, and the other views follow.>>

## Counting machines

With $n$ states and two symbols, the transition table has $2n$ entries. Each entry chooses a symbol to write (two options), a direction (two options) and a next state, which is one of the $n$ states or the halting state ($n + 1$ options). There are therefore

$$\bigl(4(n+1)\bigr)^{2n}$$

machines with $n$ states and two symbols [@rado1962]. For five states this is {{fact:count-bb5-rado}}. This is Radó's way of counting, in which a transition into the halting state is counted once for each symbol it writes and each direction it moves. Aaronson's survey and the paper on the five-state busy beaver treat all transitions into the halting state as a single option, which gives $4n + 1$ options per entry, $(4n+1)^{2n}$ machines in total, and {{fact:count-bb5-aaronson}} for five states [@aaronson2020] [@bbchallenge2025]. For the busy beaver problem, most of these machines are equivalent to others: renaming the states other than the start state, exchanging left and right, or changing a transition that is never used when the machine starts on the blank tape does not change the number of steps or the number of ones written. Chapter 5 shows how the search for the five-state busy beaver avoids these duplicates.

<<figure:fig2.2 | Fig. 2.2. The number of machines with n states and two symbols, computed exactly in your browser with Radó's formula (4(n+1))^(2n). Choose n to see the number written out in full.>>

<details>
<summary>An encoding over {0, 1}</summary>

Number the states $q_1, \dots, q_s$ with $q_1$ the start state, the tape symbols $a_1, \dots, a_r$ with $a_1 = \text{␣}$, and the directions $D_1 = L$ and $D_2 = R$. A transition $\delta(q_i, a_j) = (q_k, a_l, D_m)$ is encoded as

$$0^i\,1\,0^j\,1\,0^k\,1\,0^l\,1\,0^m .$$

The word $\mathrm{enc}(M)$ consists of the codes of all transitions, separated by $11$, followed by $111$ and the numbers of the final states, each written as a block of zeros, separated by single ones. The input alphabet can be recorded in the same way. Every block of zeros is non-empty, so the separators can be recognised and the machine can be read back unambiguously. Different orders of the transitions give different words for the same machine; this does no harm, since all that is needed is that every word in the image of $\mathrm{enc}$ describes exactly one machine.

</details>

<details>
<summary>Small universal machines</summary>

Turing's universal machine was large, and later work looked for universal machines with as few states and symbols as possible. Under the standard conventions, with a finite input on an otherwise blank tape, the survey by Neary and Woods lists the {{fact:utm-smallest-standard}} [@nearywoods2011survey]. What counts as universal depends on conventions that are easy to overlook, in particular how the input is placed on the tape and how the end of a computation is recognised.

In *A New Kind of Science* [@wolfram2002], Wolfram presented a machine with two states and three symbols, `1RB2LA1LA_2LA2RB0RA` in the text format above, and suggested that it might be universal. It has no halting transition. Wolfram later offered a prize for a proof, and Smith gave one [@wolfram2007blog] [@smith2020]. His construction starts the machine on a tape that is not blank outside a finite region: the tape is filled with an infinite, non-periodic pattern, produced by a simple procedure that is itself not universal. Since the machine never halts, the result of a simulated computation also has to be recognised by other means than halting. Whether universality in this extended sense should count as universality in the usual sense has been debated [@pratt2007fom], and the survey cited above does not list the machine among the universal machines for the standard definitions [@nearywoods2011survey].

</details>
