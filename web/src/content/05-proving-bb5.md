# Proving BB(5)

The statement that $S(5)$ equals {{fact:bb5-steps}} consists of two claims. The first is that some machine with five states halts after exactly that many steps when started on the blank tape; this is checked by running the machine. The second is that every machine with five states either halts within that many steps or never halts. This claim concerns all {{fact:count-bb5-aaronson}} machines with five states (counted, as in the paper, with a single halting option per entry; see chapter 2), and for each machine that does not halt it requires a proof that it does not. Chapter 3 shows that no single method decides halting for machines of every size, so the work combines methods that each cover a large class of machines with individual proofs for the machines that remain. The bbchallenge collaboration completed this work and checked it in the proof assistant Coq, since renamed the Rocq Prover [@coqbb5]; the paper [@bbchallenge2025] describes the methods. Following the collaboration, this chapter writes $\mathrm{BB}(5)$ for $S(5)$.

## Tree normal form

Most of the machines counted in chapter 2 are redundant. A transition that is never used from the blank tape does not affect the run, and renaming states or exchanging left and right does not change the running time. Tree normal form, which goes back to Brady's dissertation [@brady1964thesis] [@wiki-tnf], enumerates one representative of each class of equivalent machines.

The enumeration starts from the machine in which every transition is undefined. Each machine in the tree is simulated from the blank tape. If it reaches an undefined transition, its run up to that point is the same for every way of filling in the rest of the table. One child makes this transition a halting transition, which gives a halting machine whose running time is known exactly. The other children define the transition in every admissible way and are simulated in turn. To avoid duplicates, states are named in the order in which the run first reaches them, and the first move goes to the right. A machine that provably never reaches an undefined transition is a leaf: no way of completing its table makes it halt. For five states, the enumeration in the proof contains {{fact:bb5-enumerated-machines}} machines [@bbchallenge2025].

## Deciders

A decider is a program that receives a machine and either proves that it never halts or gives up. It must never claim non-halting for a machine that halts, but it may give up on machines it cannot handle. The proof uses several deciders, each aimed at one kind of behaviour.

- **Cyclers.** The machine eventually repeats itself exactly. Precisely: the same configuration, consisting of state, head position and tape contents, occurs at two different times, so the run between these times repeats forever.
- **Translated cyclers.** The machine repeats a pattern while travelling along the tape. Precisely: at two times the head visits a cell that has never been visited before, in the same state and on the same side of the visited region, and the tape contents behind the head agree at both times over the largest distance the head moves back between the two times; the transitions between the two times then repeat forever, shifted along the tape.
- **n-gram closed position sets.** The decider replaces the full tape by local information about it. Precisely: it computes a set of words of length $n$ that may occur on the tape, together with a set of local situations at the head (the state and the nearby symbols), such that the start configuration is described, the sets are closed under the step relation, and no described situation is about to halt; every reachable configuration is then described by these sets, so the machine does not halt.
- **Repeated word lists.** The decider describes the tape as repeated blocks. Precisely: the tape is written as a list of words of a fixed length with repetition counts, every count above a threshold $T$ is replaced by "at least $T$", and the decider explores the abstract configurations reachable in this representation; if the exploration ends with a closed set that contains no halting configuration, the machine does not halt.

The cycler and translated cycler checks are combined in one decider in the proof. Further methods, among them reductions to finite automata, resolve a small number of additional machines [@bbchallenge2025].

## Machines with individual proofs

Some machines are resolved by none of the general deciders. The paper calls them sporadic machines; there are {{fact:bb5-sporadic-machines}} of them, and each has its own proof of non-halting, formalised in Coq [@bbchallenge2025] [@coqbb5]. The same development also proves the values for two, three and four states [@coqbb5]. The details below describe what the formal verification covers.

## The enumeration in your browser

Fig. 5.1 runs the enumeration in tree normal form for two, three and four states in your browser, using this site's own deciders. The table lists how many machines halted, how many each decider resolved and how many remain unresolved. For comparison it shows the number of machines in the formally verified enumeration: {{fact:bb2-tnf-count}} for two states, {{fact:bb3-tnf-count}} for three states and {{fact:bb4-tnf-count}} for four states.

The deciders on this site are not formally verified. Their results are checked against the Coq-verified enumeration in the project's tests, using the open data of the Coq-BB5 project [@coqbb5].

<<figure:fig5.1 | Fig. 5.1. Enumeration of all machines with two, three or four states in tree normal form, run in your browser. Choose the number of states to start; the table shows how many machines each decider resolved, next to the formally verified count.>>

<<figure:fig5.2 | Fig. 5.2. For each decider, one machine that it resolves, with its space-time diagram and the evidence the decider found. Choose a decider to see its example.>>

<details>
<summary>Tree normal form in detail</summary>

Fix $n$. A node of the tree is a partial transition table $T \colon Q \times \{0, 1\} \rightharpoonup \{0, 1\} \times \{L, R\} \times Q$ with $|Q| = n$. The root is the empty table. A node is processed as follows.

1. Simulate $T$ from the blank tape in state $A$.
2. If the run reaches a pair $(q, a)$ for which $T$ is undefined, after $t$ steps, then every completion of $T$ in which $(q, a)$ leads into $H$ halts after exactly $t + 1$ steps. This value is recorded, with the score obtained by writing a 1 in the final step. If $T$ has at least one other undefined entry, the children of $T$ are the tables $T \cup \{(q, a) \mapsto (b, D, q')\}$ for $b \in \{0, 1\}$, $D \in \{L, R\}$ and $q'$ ranging over the states already reached in the run and the first state not yet reached. If $(q, a)$ is the first transition of the run, only $D = R$ is used. If $(q, a)$ is the last undefined entry, there are no further children, since a completed table without a transition into $H$ never halts.
3. If a decider proves that the run of $T$ never reaches an undefined entry, then no completion of $T$ halts, and the node is a leaf.
4. Otherwise the node is unresolved.

Every machine with $n$ states that halts on the blank tape uses only the entries its run reaches. After renaming its states in the order of first use, and mirroring it if its first move goes to the left, it agrees with some node of the tree on these entries, and its running time is one of the recorded values. If no node remains unresolved, $S(n)$ is therefore the largest recorded value, and $\Sigma(n)$ the largest recorded score.

</details>

<details>
<summary>Translated cyclers in detail</summary>

Consider a run in which, at times $t_1 < t_2$, the head is on a cell to the right of every cell visited before, in the same state $q$ both times. Let $p_1$ and $p_2$ be the head positions at these times, and let $d \ge 0$ be the largest distance by which the head is left of $p_1$ at any time between $t_1$ and $t_2$, that is, $d = \max\{p_1 - p(s) \mid t_1 \le s \le t_2\}$, where $p(s)$ is the head position at time $s$. Suppose that the tape contents on the cells $p_1 - d, \dots, p_1$ at time $t_1$ equal the contents on the cells $p_2 - d, \dots, p_2$ at time $t_2$. All cells to the right of the head are blank at both times. Between $t_1$ and $t_2$ the machine only reads cells in $[p_1 - d, \infty)$, so from time $t_2$ it reads the same symbols, shifted by $p_2 - p_1$, and repeats the same transitions. The argument applies again from the new record, so the machine never halts. The mirror-image condition handles machines that travel to the left. A cycler is the case in which an entire configuration recurs without any shift.

</details>

<details>
<summary>What the formal proof checks</summary>

Formal verification means that the proof checker of Coq has checked every step of the argument: the enumeration, the correctness of each decider and of its results, and the individual proofs of the sporadic machines, all relative to definitions of Turing machines and of $S$ written in Coq [@coqbb5] [@bbchallenge2025]. What remains to be trusted is the proof checker itself and the agreement between these definitions and the mathematical ones. The deciders on this site are ordinary programs without such a proof; this is why their results are compared with the verified enumeration.

</details>
