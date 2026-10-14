# The busy beaver

Radó considered all Turing machines with $n$ states and two symbols that halt when started on a blank tape, and asked for the largest number of steps such a machine can make and the largest number of ones it can leave on the tape [@rado1962]. The answers define two functions, $S(n)$ and $\Sigma(n)$. For every $n$ their values are specific natural numbers, yet no algorithm computes them, and only a few values are known.

## Definition

The machines considered here have $n$ states, named $A, B, C, \dots$, the tape alphabet $\{0, 1\}$ with $0$ as the blank, and in addition a halting state $H$ that is not counted among the $n$ states. Every entry of the transition table is defined, and the machine halts when a transition leads into $H$. A machine is started in state $A$ on a tape that contains only zeros. If it halts, its running time is the number of transitions it performs, including the final transition into $H$ [@wiki-bb-functions], and its score is the number of ones on the tape at that moment. Then

- $S(n)$ is the largest running time of a machine with $n$ states that halts, and
- $\Sigma(n)$ is the largest score of a machine with $n$ states that halts.

Both are maxima over a finite, non-empty set: there are finitely many machines with $n$ states, and some of them halt. Following Aaronson's survey and the bbchallenge collaboration, $S(n)$ is also written $\mathrm{BB}(n)$ [@aaronson2020]. Here $\Sigma$ denotes Radó's function and not an input alphabet. In the text format of chapter 2, a transition into $H$ is written with the letter Z, and an undefined entry `---` is treated as a halting transition when it is reached.

## Why S cannot be computed

The argument follows the proof in lecture 2 of the course, which uses the doubling machine $M_{*2}$ from chapter 1 [@theolog2024]. Suppose some machine $M_S$ with $k$ states computed $S$ in unary, turning a block of $m$ ones into a block of $S(m)$ ones, and let $c$ be the number of states of $M_{*2}$. For each $n$, chain three machines: one with $n$ states that writes $n$ ones on the blank tape, then $M_{*2}$, which doubles them, then $M_S$. The combined machine has $n + c + k$ states and halts with $S(2n)$ ones on the tape. It writes at most one symbol per step, so it runs for at least $S(2n)$ steps, and therefore $S(n + c + k) \ge S(2n)$. On the other hand, $S$ is strictly increasing, and for $n > c + k$ we have $n + c + k < 2n$, hence $S(n + c + k) < S(2n)$. The assumption leads to a contradiction, so $S$ is not computable. The same argument, without the step from ones to steps, shows that $\Sigma$ is not computable either.

Radó proved more: $\Sigma(n)$ exceeds $f(n)$ for all sufficiently large $n$, for every computable function $f$ [@rado1962]. Since a machine makes at least as many steps as it leaves ones on the tape, the same holds for $S$. The function $S$ is also directly tied to the halting problem. If $S(n)$ were known, one could decide for every machine with $n$ states whether it halts on the blank tape, by running it for $S(n)$ steps: a machine that has not halted by then never halts.

## Known values

| $n$ | $S(n)$ | $\Sigma(n)$ | Determined by |
|---|---|---|---|
| 1 | {{fact:bb1-steps}} | {{fact:bb1-ones}} | Radó [@rado1962] |
| 2 | {{fact:bb2-steps}} | {{fact:bb2-ones}} | $\Sigma$: Radó [@rado1962]; $S$: Lin [@lin1963], published with Radó [@linrado1965] |
| 3 | {{fact:bb3-steps}} | {{fact:bb3-ones}} | Lin [@lin1963], published with Radó [@linrado1965] |
| 4 | {{fact:bb4-steps}} | {{fact:bb4-ones}} | Brady [@brady1983]; independently Machlin and Stout [@machlinstout1990] |
| 5 | {{fact:bb5-steps}} | {{fact:bb5-ones}} | The bbchallenge collaboration [@bbchallenge2025] [@coqbb5] |

For six states only lower bounds are known; chapter 6 describes them. For three states, the two maxima are attained by different machines: the machines that run longest leave fewer than $\Sigma(3)$ ones, and the machines that leave $\Sigma(3)$ ones halt earlier [@linrado1965]. For the other values in the table, a single machine attains both.

## Reading a space-time diagram

Fig. 4.1 runs, for each $n$ from one to five, a machine that attains $S(n)$, and draws its run as a space-time diagram. Each row of the diagram shows the tape at one moment. Time runs downward and the tape runs horizontally, so the first row is the blank tape and the last row is the tape when the machine halts. Cells that hold a 1 are drawn dark, and the cell under the head is marked in blue. A head that sweeps steadily in one direction draws a diagonal line, and a pattern that repeats while it moves along the tape produces regular stripes. The five-state champion runs for {{fact:bb5-steps}} steps, far too many to draw one row per step, so its diagram is compressed in time.

<<figure:fig4.1 | Fig. 4.1. The champions for one to five states, each simulated in your browser and drawn as a space-time diagram, with time running downward and the head marked in blue. Choose a champion to run it.>>

## History

Radó's question led to a sequence of computer searches. In his dissertation, Lin determined $S(2)$ and both values for three states, using a program to classify the machines and separate arguments for those that the program could not classify; the results were published with Radó [@lin1963] [@linrado1965]. Brady completed the case of four states [@brady1983], and Machlin and Stout confirmed the result with a different method [@machlinstout1990]. For five states, Marxen and Buntrock found the machine that is now known to be the champion, using a simulator that accelerates repetitive behaviour [@marxenbuntrock1990]. Whether any five-state machine runs longer remained open until the bbchallenge collaboration announced a formally verified proof on {{fact:bb5-announcement-date}} [@bbchallenge-announcement-2024] [@bbchallenge2025] [@coqbb5]. Chapter 5 describes how.

<<figure:fig4.2 | Fig. 4.2. Milestones in the history of the busy beaver functions, from Radó's definition to the verified value for five states, each with its source.>>

<details>
<summary>Formal definition</summary>

In the notation of chapter 1, a busy beaver machine with $n$ states is a machine

$$M = (Q \cup \{H\},\ \{1\},\ \{0, 1\},\ \delta,\ A,\ \{H\})$$

where $Q$ is a set of $n$ states with $A \in Q$ and $H \notin Q$, the blank is $0$, and $\delta \colon (Q \times \{0, 1\}) \to (Q \cup \{H\}) \times \{0, 1\} \times \{L, R\}$ is defined on all of $Q \times \{0, 1\}$ and undefined in $H$. The machine is run on the empty input, so its start configuration is $A$. Since $\delta$ is total on $Q \times \{0, 1\}$, the only halting configurations are those in state $H$. If the run reaches a configuration $u\,H\,v$ after exactly $t$ steps, then the running time of $M$ is $t(M) = t$, and its score $\sigma(M)$ is the number of ones in $uv$. The transition into $H$ is a step of $\vdash$, so it is counted in $t(M)$. Then

$$S(n) = \max\{\, t(M) \mid M \text{ has } n \text{ states and halts} \,\}, \qquad \Sigma(n) = \max\{\, \sigma(M) \mid M \text{ has } n \text{ states and halts} \,\}.$$

A machine in the text format whose run reaches an undefined entry after $t$ steps corresponds to the completed machine in which this entry is a transition into $H$. That machine halts after $t + 1$ steps, and the score is largest if the final transition writes a 1.

</details>

<details>
<summary>Proof that S and Σ are not computable</summary>

**Conventions.** Numbers are written in unary as blocks of ones. A machine computes a function $f \colon \mathbb{N} \to \mathbb{N}$ in normal form if, started with the head on the blank cell immediately to the left of a block $1^m$ on an otherwise blank tape, it halts with the head on the blank cell immediately to the left of a block $1^{f(m)}$ on an otherwise blank tape. Every computable function can be computed in normal form by a machine with tape alphabet $\{0, 1\}$; this is a standard construction. Entries of the transition table that are never used can be filled arbitrarily, so all machines below have a table that is defined everywhere, as the definition above requires.

**Composition.** For machines $M_1$ and $M_2$, the machine $M_1 ; M_2$ has the states of both (renamed apart if necessary) and the start state of $M_1$. Every transition of $M_1$ into $H$ is redirected to the start state of $M_2$, with the same written symbol and direction. Then $M_1 ; M_2$ first behaves like $M_1$ and continues like $M_2$ from the configuration in which $M_1$ would have halted. Its number of states is the sum of the numbers of states of $M_1$ and $M_2$.

**Lemma.** For every $m \ge 1$, $S(m + 1) \ge S(m) + 1$ and $\Sigma(m + 1) \ge \Sigma(m) + 1$.

*Proof.* Let $M$ have $m$ states and halt. Its last transition is the entry for some pair $(q, a)$ and leads into $H$; since the machine halts immediately afterwards, this entry is used exactly once in the run. Add a new state $p$, redirect the entry for $(q, a)$ to $p$ with the same written symbol and direction, and set $\delta(p, 1) = (p, 1, R)$ and $\delta(p, 0) = (H, 1, R)$. The new machine has $m + 1$ states and behaves like $M$ until $M$ would halt. It then moves right across a block of ones, which ends because the tape holds finitely many ones, writes a 1 on the first 0 and halts. It makes at least one more step than $M$ and leaves exactly one more 1. Applying this to a machine with running time $S(m)$ and to a machine with score $\Sigma(m)$ gives both inequalities. $\square$

**Theorem.** $S$ is not computable.

*Proof.* Suppose $M_S$, with $k$ states, computes $m \mapsto S(m)$ in normal form, and let $M_{*2}$, with $c$ states, compute $m \mapsto 2m$ in normal form. (Task 4 of the first exercise sheet asks for a machine computing $x^n \mapsto x^{2n}$ over the alphabet $\{x, \text{␣}\}$. Read $x$ as 1 and ␣ as 0; after a final adjustment of the head position, such a machine computes $m \mapsto 2m$ in normal form.) For $n \ge 1$, let $W_n$ have the states $w_1, \dots, w_n$, with start state $w_1$, and the transitions $\delta(w_i, 0) = (w_{i+1}, 1, L)$ for $i < n$ and $\delta(w_n, 0) = (H, 1, L)$; the entries for reading 1 are never used and can be set arbitrarily. Started on the blank tape, $W_n$ writes $1^n$ from right to left and halts with the head on the blank cell immediately to the left of the block.

The machine $N_n = W_n ; M_{*2} ; M_S$ has $n + c + k$ states. Started on the blank tape, it halts with exactly $S(2n)$ ones on the tape. The tape is blank at the start and every step changes at most one cell, so $N_n$ makes at least $S(2n)$ steps, and $S(n + c + k) \ge S(2n)$. Choose $n = c + k + 1$. Then $n + c + k = 2n - 1$, and the lemma gives $S(2n - 1) < S(2n)$, a contradiction. $\square$

**Theorem.** $\Sigma$ is not computable.

*Proof.* Replace $M_S$ by a machine $M_\Sigma$ that computes $m \mapsto \Sigma(m)$ in normal form. Then $N_n$ halts with $\Sigma(2n)$ ones on the tape, so $\Sigma(2n - 1) \ge \Sigma(2n)$ for $n = c + k + 1$, which again contradicts the lemma. $\square$

</details>
