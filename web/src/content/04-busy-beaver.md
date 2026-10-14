# The busy beaver

Radó considered all Turing machines with $n$ states and two symbols that halt when started on a blank tape, and asked for the largest number of steps such a machine can make and the largest number of ones it can leave on the tape [@rado1962]. The answers define two functions, $S(n)$ and $\Sigma(n)$. For every $n$ their values are specific natural numbers, yet no algorithm computes them, and only a few values are known.

## Definition

The machines considered here have $n$ states, named $A, B, C, \dots$, the tape alphabet $\{0, 1\}$ with $0$ as the blank, and in addition a halting state $H$ that is not counted among the $n$ states. Every entry of the transition table is defined, and the machine halts when a transition leads into $H$. A machine is started in state $A$ on a tape that contains only zeros. If it halts, its running time is the number of transitions it performs, including the final transition into $H$ [@wiki-bb-functions], and its score is the number of ones on the tape at that moment. Then

- $S(n)$ is the largest running time of a machine with $n$ states that halts, and
- $\Sigma(n)$ is the largest score of a machine with $n$ states that halts.

Both are maxima over a finite, non-empty set: there are finitely many machines with $n$ states, and some of them halt. Following Aaronson's survey and the bbchallenge collaboration, $S(n)$ is also written $\mathrm{BB}(n)$ [@aaronson2020]. Here $\Sigma$ denotes Radó's function and not an input alphabet. In the text format of chapter 2, a transition into $H$ is written with the letter Z, and an undefined entry `---` is treated as a halting transition when it is reached.

Lecture 2 of the course defines both functions for machines with the tape alphabet $\{x, \text{␣}\}$ that halt when no transition is defined, and remarks that the exact values depend on such details of the definition [@theolog2024-lecture2]. Its example for $\Sigma(2) = 4$ is the machine `1RB1LB_1LA1RZ` of chapter 2, with $x$ for 1 and no transition where this site has the halting transition `1RZ`. Without that transition the machine makes one step fewer and leaves the same ones. This site follows the convention of the bbchallenge collaboration, which counts the final transition, and all values below use it.

<details>
<summary>Formal definition</summary>

In the notation of chapter 1, a busy beaver machine with $n$ states is a machine

$$M = (Q \cup \{H\},\ \{1\},\ \{0, 1\},\ \delta,\ A,\ \{H\})$$

where $Q$ is a set of $n$ states with $A \in Q$ and $H \notin Q$, the blank is $0$, and $\delta \colon Q \times \{0, 1\} \to (Q \cup \{H\}) \times \{0, 1\} \times \{L, R\}$ is defined on all of $Q \times \{0, 1\}$ and undefined in $H$. The machine is run on the empty input, so its start configuration is $A$. Since $\delta$ is total on $Q \times \{0, 1\}$, the only halting configurations are those in state $H$. If the run reaches a configuration $u\,H\,v$ after exactly $t$ steps, then the running time of $M$ is $t(M) = t$, and its score $\sigma(M)$ is the number of ones in $uv$. The transition into $H$ is a step of $\vdash$, so it is counted in $t(M)$. Then

$$S(n) = \max_M t(M), \qquad \Sigma(n) = \max_M \sigma(M),$$

where $M$ ranges over the machines with $n$ states that halt.

A machine in the text format whose run reaches an undefined entry after $t$ steps corresponds to the completed machine in which this entry is a transition into $H$. That machine halts after $t + 1$ steps, and the score is largest if the final transition writes a 1.

</details>

## Why Σ and S cannot be computed

The proof in lecture 2 of the course uses machines as subroutines [@theolog2024-lecture2]. Suppose some machine $M_\Sigma$ computed $\Sigma$, turning a block of $m$ ones into a block of $\Sigma(m)$ ones. Let $M_{+1}$ add one to a block and let $M_{*2}$, the doubling machine of chapter 1, double it. If $k$ is the number of states of these three machines together, a machine $I_k$ with $k$ states can write $k$ ones on the blank tape. Running $I_k$, $M_{*2}$, $M_\Sigma$ and $M_{+1}$ one after another gives a machine with $2k$ states that halts with $\Sigma(2k) + 1$ ones on the tape. By the definition of $\Sigma$, no machine with $2k$ states leaves more than $\Sigma(2k)$ ones, so $M_\Sigma$ cannot exist.

The function $S$ is not computable either. If it were, one could compute $\Sigma(n)$ by running every machine with $n$ states for $S(n)$ steps: a machine that has not halted by then never halts, and the largest number of ones left by the others is $\Sigma(n)$. In the same way, knowing $S(n)$ would decide for every machine with $n$ states whether it halts on the blank tape. Conversely, lecture 4 of the course uses $\Sigma$ for a second proof that the halting problem is undecidable: a decision procedure for halting would compute $\Sigma(n)$ by deciding which machines with $n$ states halt on the blank tape and running those to the end [@theolog2024-lecture4].

Radó proved more: $\Sigma(n)$ exceeds $f(n)$ for all sufficiently large $n$, for every computable function $f$ [@rado1962]. A machine writes at most one symbol per step, so it makes at least as many steps as it leaves ones on the tape, and the same holds for $S$.

<details>
<summary>Proof that Σ and S are not computable</summary>

**Conventions.** Numbers are written in unary as blocks of ones. A machine computes a function $f \colon \mathbb{N} \to \mathbb{N}$ in normal form if, started with the head on the blank cell immediately to the left of a block $1^m$ on an otherwise blank tape, it halts with the head on the blank cell immediately to the left of a block $1^{f(m)}$ on an otherwise blank tape. Every computable function can be computed in normal form by a machine with tape alphabet $\{0, 1\}$; this is a standard construction, which converts numbers from binary to unary and reduces the tape alphabet to two symbols, as the lecture notes [@theolog2024-lecture2]. Entries of the transition table that are never used can be filled arbitrarily, so all machines below have a table that is defined everywhere, as the definition above requires.

**Composition.** For machines $M_1$ and $M_2$, the machine $M_1 ; M_2$ has the states of both (renamed apart if necessary) and the start state of $M_1$. Every transition of $M_1$ into $H$ is redirected to the start state of $M_2$, with the same written symbol and direction. Then $M_1 ; M_2$ first behaves like $M_1$ and continues like $M_2$ from the configuration in which $M_1$ would have halted. Its number of states is the sum of the numbers of states of $M_1$ and $M_2$. The lecture, whose machines have no separate halting state, merges the end state of each machine with the start state of the next [@theolog2024-lecture2]; here the transitions into $H$ are redirected instead.

**Theorem.** $\Sigma$ is not computable.

*Proof.* Suppose $M_\Sigma$ computes $m \mapsto \Sigma(m)$ in normal form. Let $M_{+1}$ compute $m \mapsto m + 1$ and $M_{*2}$ compute $m \mapsto 2m$, both in normal form. (Task 4 of the first exercise sheet asks for a machine computing $x^n \mapsto x^{2n}$ over the alphabet $\{x, \text{␣}\}$. Read $x$ as 1 and ␣ as 0; after a final adjustment of the head position, such a machine computes $m \mapsto 2m$ in normal form.) Let $k$ be the total number of states of $M_\Sigma$, $M_{+1}$ and $M_{*2}$.

Let $I_k$ have the states $w_1, \dots, w_k$, with start state $w_1$, and the transitions $\delta(w_i, 0) = (w_{i+1}, 1, L)$ for $i < k$ and $\delta(w_k, 0) = (H, 1, L)$; the entries for reading 1 are never used and can be set arbitrarily. Started on the blank tape, $I_k$ writes $1^k$ from right to left and halts with the head on the blank cell immediately to the left of the block.

The machine $N = I_k ; M_{*2} ; M_\Sigma ; M_{+1}$ has $k + k = 2k$ states. Started on the blank tape, $I_k$ writes $1^k$, $M_{*2}$ turns it into $1^{2k}$, $M_\Sigma$ into $1^{\Sigma(2k)}$ and $M_{+1}$ into $1^{\Sigma(2k) + 1}$, and $N$ halts. So $N$ is a machine with $2k$ states that halts with score $\Sigma(2k) + 1$, and the definition of $\Sigma$ gives

$$\Sigma(2k) \ge \Sigma(2k) + 1 ,$$

a contradiction [@theolog2024-lecture2]. $\square$

**Theorem.** $S$ is not computable.

*Proof.* Suppose $S$ were computable. Then $\Sigma$ would be computable: given $n$, list the finitely many machines with $n$ states, run each of them on the blank tape for $S(n)$ steps, and output the largest score among those that have halted. By the definition of $S(n)$, every machine with $n$ states that halts does so within $S(n)$ steps, so this procedure sees every halting machine and outputs $\Sigma(n)$. This contradicts the previous theorem. $\square$

</details>

## Known values

| $n$ | $S(n)$ | $\Sigma(n)$ | Determined by |
|---|---|---|---|
| 1 | {{fact:bb1-steps}} | {{fact:bb1-ones}} | Radó [@rado1962] |
| 2 | {{fact:bb2-steps}} | {{fact:bb2-ones}} | Radó [@rado1962] for $\Sigma$ and Lin [@lin1963] [@linrado1965] for $S$ |
| 3 | {{fact:bb3-steps}} | {{fact:bb3-ones}} | Lin [@lin1963] [@linrado1965] |
| 4 | {{fact:bb4-steps}} | {{fact:bb4-ones}} | Brady [@brady1983] and independently Machlin and Stout [@machlinstout1990] |
| 5 | {{fact:bb5-steps}} | {{fact:bb5-ones}} | The bbchallenge collaboration [@bbchallenge2025] [@coqbb5] |

For six states only lower bounds are known; chapter 6 describes them. For three states, the two maxima are attained by different machines: the machines that run longest leave fewer than $\Sigma(3)$ ones, and the machines that leave $\Sigma(3)$ ones halt earlier [@linrado1965] [@wiki-bb3]. For the other values in the table, a single machine attains both.

## Space-time diagrams

Fig. 4.1 runs, for each $n$ from one to five, a machine that attains $S(n)$, and draws its run as a space-time diagram. Each row of the diagram shows the tape at one moment. Time runs downward and the tape runs horizontally, so the first row is the blank tape and the last row is the tape when the machine halts. Cells that hold a 1 are filled, and the cell under the head is marked in blue. A head that sweeps steadily in one direction draws a diagonal line, and a pattern that repeats while it moves along the tape produces regular stripes. The five-state champion runs for {{fact:bb5-steps}} steps, far too many to draw one row per step, so its diagram is compressed in time.

<<figure:fig4.1 | Fig. 4.1. The champions for one to five states, each simulated in your browser and drawn as a space-time diagram, with time running downward and the head marked in blue. Choose a champion to run it.>>

## History

Radó's question led to a sequence of computer searches. In his dissertation, Lin determined $S(2)$ and both values for three states with the help of a computer program; the results were published with Radó [@lin1963] [@linrado1965]. Brady completed the case of four states [@brady1983], and Machlin and Stout gave an independent proof [@machlinstout1990]. For five states, Marxen and Buntrock found the machine that is now known to be the champion in a computer search [@marxenbuntrock1990]. Whether any five-state machine runs longer remained open until the bbchallenge collaboration announced a formally verified proof on {{fact:bb5-announcement-date}} [@bbchallenge-announcement-2024] [@bbchallenge2025] [@coqbb5]. Chapter 5 describes how.

<<figure:fig4.2 | Fig. 4.2. Milestones in the history of the busy beaver functions, from Radó's definition to the verified value for five states, each with its source.>>
