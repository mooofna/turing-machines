# Programs as data

A Turing machine is a finite object. Its states, its alphabets and its transition table can be written down as a single word over a fixed alphabet, and that word can be the input of another machine. Turing used this observation to construct one machine that can carry out the computation of every other machine [@turing1936]. The same observation is the starting point of chapter 3, which asks what one machine can find out about another.

## Machines as words

An encoding assigns to every machine $M$ a word $\mathrm{enc}(M)$ over a fixed alphabet. It must be possible to reconstruct $M$ from $\mathrm{enc}(M)$ and to decide whether a given word encodes a machine at all. The course uses the alphabet $\{0, 1, \#\}$: states, symbols and directions are numbered and written in binary, numbers are separated by $\#$, and the parts of the machine by $\#\#$ [@theolog2024-lecture4]. Input words are encoded in the same way, and a machine $M$ together with an input word $w$ is written $\mathrm{enc}(M)\#\#\mathrm{enc}(w)$.

<details>
<summary>The encoding of the course</summary>

Number the states $q_0, \dots, q_n$ with $q_0$ the start state, the tape symbols $\sigma_0, \dots, \sigma_m$ and the three directions $L$, $R$ and $N$, and write $\mathrm{bin}(i)$ for the binary representation of the number $i$. The course defines [@theolog2024-lecture4]

$$\mathrm{enc}(M) = \mathrm{enc}(Q)\#\#\mathrm{enc}(\Sigma)\#\#\mathrm{enc}(\Gamma)\#\#\mathrm{enc}(\delta)\#\#\mathrm{enc}(F),$$

where

- $\mathrm{enc}(Q) = \mathrm{bin}(0)\#\cdots\#\mathrm{bin}(n)$ lists the numbers of the states, and $\mathrm{enc}(\Sigma)$, $\mathrm{enc}(\Gamma)$ and $\mathrm{enc}(F)$ list the numbers of the input symbols, the tape symbols and the final states in the same way;
- $\mathrm{enc}(\delta)$ lists the codes of all transitions, each followed by $\#$.

The code of a transition $\delta(q_i, \sigma_j) = (q_k, \sigma_l, D)$ consists of five numbers,

$$\mathrm{bin}(i)\#\mathrm{bin}(j)\#\mathrm{bin}(k)\#\mathrm{bin}(l)\#\mathrm{bin}(D),$$

where $D$ stands for the number of the direction.

A word $w = a_1 \cdots a_\ell$ is encoded as $\mathrm{enc}(w) = \mathrm{bin}(a_1)\#\cdots\#\mathrm{bin}(a_\ell)$, where each symbol is replaced by its number. Every number is a non-empty word over $\{0, 1\}$, so the separators can be recognised and the word can be read back unambiguously. Different numberings and different orders of the transitions give different words for the same machine. This does no harm: all that is needed is that every word in the image of $\mathrm{enc}$ describes exactly one machine.

</details>

Since every machine is described by a finite word, there are only countably many machines. The languages over a non-empty alphabet, on the other hand, form an uncountable set, by the same diagonal argument that shows that the real numbers are uncountable [@theolog2024-lecture2]. All but countably many languages are therefore accepted by no Turing machine. This counting argument shows that such languages exist without naming one; chapter 3 gives an explicit example.

## Universal machines

A universal machine $U$ receives $\mathrm{enc}(M)\#\#\mathrm{enc}(w)$ as input and simulates $M$ on $w$ step by step. It halts exactly when $M$ halts on $w$ and accepts exactly when $M$ accepts $w$, so that

$$L(U) = \{\, \mathrm{enc}(M)\#\#\mathrm{enc}(w) \mid w \in L(M) \,\}.$$

Turing's paper contains the first construction of such a machine [@turing1936]. The universal machine of the course has four tapes: one for its input, one for its own work, one for the current state of $M$ and one for the tape of $M$ [@theolog2024-lecture4]. A universal machine stores the description of the simulated machine on its tape and consults it in every step, in the same way that a stored-program computer keeps its program in memory. The simulator on this site is a program of this kind as well: it reads a machine description and executes it.

<details>
<summary>Small universal machines</summary>

Turing's universal machine was large, and later work looked for universal machines with as few states and symbols as possible. Under the standard conventions, with a finite input on an otherwise blank tape, the smallest known universal machines have the following numbers of states and symbols: {{fact:utm-smallest-pairs}} [@nearywoods2011survey] [@wiki-utm]. What counts as universal depends on conventions that are easy to overlook, in particular how the input is placed on the tape and how the end of a computation is recognised.

In *A New Kind of Science* [@wolfram2002], Wolfram presented a machine with two states and three symbols, `1RB2LA1LA_2LA2RB0RA` in the text format below, and suggested that it might be universal. It has no halting transition. Wolfram later offered a prize for a proof, and Smith gave one [@wolfram2007blog] [@smith2020]. His construction starts the machine on a tape that is not blank outside a finite region: the tape is filled with an infinite, non-periodic pattern, produced by a simple procedure that is itself not universal. Since the machine never halts, the result of a simulated computation also has to be recognised by other means than halting. Whether universality in this extended sense should count as universality in the usual sense has been debated [@pratt2007fom], and the survey cited above does not list the machine among the universal machines for the standard definitions [@nearywoods2011survey].

</details>

## A text format for machines

For machines with few states, the busy beaver community uses a compact text format [@bbchallenge-format]. States are named by the letters A, B, C, and so on, and A is the start state. Symbols are digits, and 0 is the blank. The table is written one state after another, separated by underscores. For each state there is one entry of three characters per symbol, in the order 0, 1, and so on: the symbol to write, the direction L or R, and the next state. The entry `---` marks an undefined transition, at which the machine halts. A next state beyond the machine's states, usually Z or H, means that the machine halts after this transition; this site writes Z.

As an example, `1RB1LB_1LA1RZ` describes a machine with two states. In state A it writes 1 in both cases and continues in state B, moving right after reading 0 and left after reading 1. In state B, after reading 0, it writes 1, moves left and continues in A; after reading 1, it writes 1, moves right and halts. This machine is a two-state busy beaver champion, which chapter 4 returns to. Machines with more symbols use the same format with more entries per state.

<<figure:fig2.1 | Fig. 2.1. One machine in three linked views: a state diagram, a transition table and a line in the standard text format. Edit the text or the table, and the other views follow.>>

## Counting machines

With $n$ states and two symbols, the transition table has $2n$ entries. Each entry chooses a symbol to write (two options), a direction (two options) and a next state, which is one of the $n$ states or the halting state ($n + 1$ options). There are therefore

$$\bigl(4(n+1)\bigr)^{2n}$$

machines with $n$ states and two symbols [@rado1962]. For five states this is {{fact:count-bb5-rado}}. This is Radó's way of counting, in which a transition into the halting state is counted once for each symbol it writes and each direction it moves. Aaronson's survey and the paper on the five-state busy beaver treat all transitions into the halting state as a single option, which gives $4n + 1$ options per entry, $(4n+1)^{2n}$ machines in total, and {{fact:count-bb5-aaronson}} for five states [@aaronson2020] [@bbchallenge2025].

For the busy beaver problem, most of these machines are equivalent to others: renaming the states other than the start state, exchanging left and right, or changing a transition that is never used when the machine starts on the blank tape does not change the number of steps or the number of ones written. Chapter 5 shows how the search for the five-state busy beaver avoids these duplicates.

<<figure:fig2.2 | Fig. 2.2. The number of machines with $n$ states and two symbols, computed exactly in your browser with Radó's formula $\bigl(4(n+1)\bigr)^{2n}$. Choose $n$ to see the number written out in full.>>
