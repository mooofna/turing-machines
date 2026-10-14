# What cannot be computed

Some questions about a Turing machine can be answered by running it. Whether a machine halts on a given input within a fixed number of steps is such a question: simulate it for that many steps and look. Whether it halts at all is a different kind of question. This chapter shows that no machine answers it correctly for all machines and inputs. Rice's theorem then extends the result to every non-trivial question about the language that a machine accepts.

## Decidable and semi-decidable languages

A language $L \subseteq \Sigma^*$ is decidable if there is a machine that halts on every input and accepts exactly the words in $L$. It is semi-decidable if there is a machine $M$ with $L(M) = L$; on inputs outside $L$, such a machine may run forever. Every decidable language is semi-decidable. A language is decidable exactly when both the language and its complement are semi-decidable: given machines for both, one can run them alternately, one step at a time, until one of them accepts.

## The halting problem

The halting problem is the language

$$P_{\mathrm{halt}} = \{\, \mathrm{enc}(M)\#\#w \mid M \text{ halts on input } w \,\}.$$

It is semi-decidable: a universal machine, modified so that it accepts as soon as the simulated machine halts, accepts exactly $P_{\mathrm{halt}}$. It is not decidable [@turing1936]. By the previous section, the complement of $P_{\mathrm{halt}}$ is then not even semi-decidable. There is no procedure that confirms non-halting in every case in which it holds.

The proof follows the diagonal argument of the course [@theolog2024]. List all machines as $M_1, M_2, \dots$ and consider the infinite table whose entry in row $i$ and column $j$ records whether $M_i$ halts on the input $\mathrm{enc}(M_j)$. If some machine decided $P_{\mathrm{halt}}$, one could build a machine $D$ that, on input $\mathrm{enc}(M_j)$, determines the diagonal entry for $M_j$ and does the opposite: it runs forever if $M_j$ halts on $\mathrm{enc}(M_j)$, and halts otherwise. $D$ occurs in the list, say as $M_d$, and its behaviour on $\mathrm{enc}(M_d)$ would have to differ from itself. The details below give the full proof.

## Simulation with a step budget

Fig. 3.1 fills a table of this kind for a few small machines and inputs by simulation. Each machine runs for at most a given number of steps, the budget. A cell shows the step at which the machine halted, or that it is still running when the budget is used up. Raising the budget changes some cells from "still running" to "halted at step $t$", and never the other way round.

No budget completes the table. If a machine does not halt on an input, its cell shows "still running" for every budget, and the simulation cannot distinguish it from a machine that halts later than the budget allows. For a particular cell, non-halting can sometimes be proved by another argument, for example by observing that the machine returns to a configuration it was in before; chapter 5 is about arguments of this kind. For a fixed finite table the correct entries exist, and a list of them is trivially a procedure that gives them. The theorem excludes a single procedure that gives the correct answer for every machine and every input.

For the same reason, there is no computable function $b$ such that every machine $M$ that halts on $w$ does so within $b(\mathrm{enc}(M)\#\#w)$ steps: simulating for that many steps would decide $P_{\mathrm{halt}}$. Chapter 4 studies the smallest bound of this kind for machines with a given number of states, started on the blank tape.

<<figure:fig3.1 | Fig. 3.1. Small machines (rows) run on several inputs (columns), each for at most the chosen number of steps. Raise the step budget and watch some cells change from still running to halted.>>

## Rice's theorem

The halting problem is one instance of a general pattern. Rice's theorem [@rice1953] states that every non-trivial property of the language accepted by a machine is undecidable. A property is non-trivial if some semi-decidable languages have it and others do not. For example, it is undecidable whether $L(M)$ is empty and whether $L(M)$ is finite. The third exercise sheet of the course asks whether it is decidable that a machine accepts only palindromes; by Rice's theorem it is not [@theolog2024].

The theorem only concerns properties of the accepted language. Properties of the description itself, such as having at most five states, can be decided by inspecting the encoding.

<details>
<summary>Proof that the halting problem is undecidable</summary>

Suppose a machine $H$ decides $P_{\mathrm{halt}}$: it halts on every input, and it accepts exactly the words $\mathrm{enc}(M)\#\#w$ for which $M$ halts on $w$. Construct a machine $D$ that works as follows on input $x$.

1. If $x$ is not the encoding of a machine, halt.
2. Otherwise $x = \mathrm{enc}(M)$ for some machine $M$. Write $\mathrm{enc}(M)\#\#\mathrm{enc}(M)$ on the tape and run $H$ on it.
3. If $H$ accepts, enter a state $p$ with $\delta(p, a) = (p, a, R)$ for every symbol $a$, so that the machine moves to the right forever. If $H$ rejects, halt.

Every step can be carried out by a Turing machine, and step 2 terminates because $H$ halts on every input. Now consider $D$ on its own encoding. By construction, $D$ halts on $\mathrm{enc}(D)$ if and only if $H$ rejects $\mathrm{enc}(D)\#\#\mathrm{enc}(D)$. By the assumption on $H$, this happens if and only if $D$ does not halt on $\mathrm{enc}(D)$. This is a contradiction, so $H$ does not exist.

In the table picture, row $i$ lists the behaviour of $M_i$ on the inputs $\mathrm{enc}(M_1), \mathrm{enc}(M_2), \dots$. The machine $D$ is constructed to differ from row $i$ in column $i$, for every $i$. Its row therefore differs from every row of the table, although $D$ is a machine and has a row.

Since $P_{\mathrm{halt}}$ is semi-decidable but not decidable, its complement is not semi-decidable.

</details>

<details>
<summary>Rice's theorem</summary>

**Theorem** [@rice1953]. Let $E$ be a set of semi-decidable languages over $\Sigma$ that is neither empty nor the set of all semi-decidable languages. Then the language

$$L_E = \{\, \mathrm{enc}(M) \mid L(M) \in E \,\}$$

is undecidable.

**Proof.** Assume first that $\emptyset \notin E$. Since $E$ is not empty, there is a machine $M_E$ with $L(M_E) \in E$. For a machine $M$ and a word $w$, let $M'$ be the machine that, on input $x$, first simulates $M$ on $w$; if this simulation halts, $M'$ runs $M_E$ on $x$ and accepts if $M_E$ accepts. If $M$ halts on $w$, then $L(M') = L(M_E) \in E$. If $M$ does not halt on $w$, then $L(M') = \emptyset \notin E$. The map that sends $\mathrm{enc}(M)\#\#w$ to $\mathrm{enc}(M')$, and every word that is not of this form to the encoding of a fixed machine that accepts nothing, is computable. It is therefore a reduction $P_{\mathrm{halt}} \le_m L_E$, and a decision procedure for $L_E$ would decide $P_{\mathrm{halt}}$.

If $\emptyset \in E$, let $E'$ be the set of semi-decidable languages that are not in $E$. Then $E'$ is neither empty nor the set of all semi-decidable languages, and $\emptyset \notin E'$, so $L_{E'}$ is undecidable by the first case. Within the decidable set of all machine encodings, $L_{E'}$ is the complement of $L_E$, so $L_E$ is undecidable as well.

</details>
