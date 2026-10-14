# The machine

Turing introduced his machine in a paper on computable numbers [@turing1936] as a model of a person who calculates with pencil and paper by following a fixed table of instructions, looking at one square of the paper at a time. He argued that every procedure that can be carried out mechanically can be carried out by such a machine, and used the model to show that Hilbert's decision problem for first-order logic has no algorithmic solution. Church reached the same conclusion independently with a different formalism [@church1936a] [@church1936b]. This chapter states the definition in the notation of the TU Dresden course Theoretische Informatik und Logik [@theolog2024], which the site uses throughout.

## States, tape and transitions

A machine has a finite set of states and a tape that extends without bound in both directions. The tape is divided into cells that each hold one symbol, and all but finitely many cells hold the blank symbol ␣. In each step the machine reads the symbol under its head and, depending on this symbol and its current state, writes a symbol, moves the head one cell to the left or to the right, and changes its state.

Formally, a Turing machine is a tuple $M = (Q, \Sigma, \Gamma, \delta, q_0, F)$ of a finite set of states, an input alphabet, a tape alphabet containing the blank, a partial transition function, a start state and a set of final states. An entry $\delta(q, a) = (q', b, D)$ means: in state $q$, reading $a$, write $b$, move in direction $D \in \{L, R\}$ and continue in state $q'$.

The machine in Fig. 1.1, from the original 2024 course project, adds one to a binary number written with the least significant bit first, so that the carry travels in the direction in which the head moves. The transition $\delta(q_0, 1) = (q_0, 0, R)$ passes the carry on, and $\delta(q_0, 0) = \delta(q_0, \text{␣}) = (q_1, 1, R)$ absorbs it. No transition is defined in $q_1$, the only final state.

<<figure:fig1.1 | Fig. 1.1. A two-state machine that adds one to a binary number written least significant bit first. Enter a binary number and step through the run one transition at a time.>>

## Configurations and steps

A configuration describes the complete situation of a machine at one moment. It is written as a word $u\,q\,v$: the tape contains $uv$ surrounded by blanks, the machine is in state $q$, and the head is on the first symbol of $v$, or on a blank if $v$ is empty. On input $w$ the machine starts in the configuration $q_0\,w$. If configuration $C'$ results from $C$ by one transition, we write $C \vdash C'$, and $C \vdash^* C'$ if $C'$ is reached from $C$ in any finite number of steps, including none.

For the increment machine on input $11$, which is the number three with the least significant bit first, the run is

$$q_0\,11 \;\vdash\; 0\,q_0\,1 \;\vdash\; 00\,q_0 \;\vdash\; 001\,q_1 ,$$

and the result $001$ is the number four.

## Halting, acceptance and computed functions

A machine halts when it reaches a configuration in which no transition is defined for the current state and the symbol under the head. If it halts in a final state, it accepts its input. The language of $M$ is the set $L(M)$ of input words that $M$ accepts. A machine can also be read as computing a partial function: if $M$ halts on $w$ in a final state, the word left on the tape is the result $f_M(w)$; otherwise $f_M(w)$ is undefined. If $M$ never halts on $w$, its run is infinite, and neither acceptance nor a result is defined. The gallery contains machines of each kind: acceptors, which recognise languages, transducers, which compute functions, and enumerators, which use an additional output convention to list the words of a language one after another.

## Unary numbers

The simplest way to write numbers on the tape is unary encoding: the number $n$ is written as $x^n$, a block of $n$ copies of the symbol $x$, so that zero is the empty word. Several numbers are separated by a marker symbol such as #. Unary encoding is wasteful, since the length of the input grows with the number itself, but it reduces arithmetic to moving and copying symbols.

The first exercise sheet of the course asked for a machine $A_{\mathrm{mul}}$ that multiplies two numbers in unary, and for a program that reads two decimal numbers, encodes them in unary, runs the machine and converts the result back to decimal [@theolog2024]. That exercise is the reason this project exists, and Fig. 1.2 carries out the same task in the browser. $A_{\mathrm{mul}}$ multiplies by repeated addition, appending a copy of the second factor to the output for each $x$ of the first factor.

<<figure:fig1.2 | Fig. 1.2. The machine A_mul from the course multiplies two numbers in unary. Enter two decimal numbers to see them encoded, multiplied step by step and decoded again.>>

## Several tapes

A multi-tape machine has $k$ tapes with one head each; every transition reads and writes on all tapes at once, and a head may also stay in place. Additional tapes make many algorithms faster but do not change what can be computed: a single-tape machine can simulate $k$ tapes, and $t$ steps of the multi-tape machine cost it at most a constant multiple of $t^2$ steps, provided $t$ is at least the length of the input.

Task 4 of the same sheet asked for a machine $M_{*2}$ computing $x^n \mapsto x^{2n}$, once with several tapes and once with one [@theolog2024]; lecture 2 of the course uses $M_{*2}$ in the proof, presented in chapter 4, that the busy beaver function is not computable. Fig. 1.3 compares the two versions. The two-tape machine can copy the input to its second tape while reading it once, and its number of steps grows linearly in $n$. The single-tape machine walks back and forth across a growing block once for every input symbol, and its number of steps grows quadratically.

<<figure:fig1.3 | Fig. 1.3. The doubling machine M*2 on one tape and on two tapes, with the number of steps for n = 0 to 20 measured in your browser. Select a value of n to watch both runs side by side.>>

<details>
<summary>Formal definition</summary>

A deterministic single-tape Turing machine is a tuple $M = (Q, \Sigma, \Gamma, \delta, q_0, F)$ where

- $Q$ is a finite set of states,
- $\Sigma$ is the input alphabet,
- $\Gamma$ is the tape alphabet, with $\Sigma \subseteq \Gamma$ and a blank symbol $\text{␣} \in \Gamma \setminus \Sigma$,
- $\delta \colon Q \times \Gamma \rightharpoonup Q \times \Gamma \times \{L, R\}$ is a partial function, the transition function,
- $q_0 \in Q$ is the start state, and
- $F \subseteq Q$ is the set of final states.

We assume $Q \cap \Gamma = \emptyset$.

**Configurations.** A configuration is a word $u\,q\,v$ with $u, v \in \Gamma^*$ and $q \in Q$. Words that differ only in blanks at the left end of $u$ or at the right end of $v$ denote the same configuration, and $u\,q$ with empty $v$ is identified with $u\,q\,\text{␣}$. The start configuration on input $w \in \Sigma^*$ is $q_0\,w$.

**Steps.** For $q \in Q$, symbols $a, b, c \in \Gamma$ and words $u, v \in \Gamma^*$:

- if $\delta(q, a) = (q', b, R)$, then $u\,q\,a\,v \vdash u\,b\,q'\,v$;
- if $\delta(q, a) = (q', b, L)$, then $u\,c\,q\,a\,v \vdash u\,q'\,c\,b\,v$ and $q\,a\,v \vdash q'\,\text{␣}\,b\,v$.

The relation $\vdash^*$ is the reflexive and transitive closure of $\vdash$.

**Halting and acceptance.** A configuration $u\,q\,a\,v$ is halting if $\delta(q, a)$ is undefined. $M$ halts on $w$ if $q_0\,w \vdash^* C$ for a halting configuration $C$; since $M$ is deterministic, this $C$ is unique. $M$ accepts $w$ if it halts on $w$ in a configuration whose state belongs to $F$, and

$$L(M) = \{\, w \in \Sigma^* \mid M \text{ accepts } w \,\}.$$

**Computed function.** The partial function $f_M \colon \Sigma^* \rightharpoonup \Gamma^*$ is defined by $f_M(w) = y$ if $q_0\,w \vdash^* u\,q\,v$ for a halting configuration with $q \in F$, where $y$ is the word $uv$ with leading and trailing blanks removed. If no such configuration is reached, $f_M(w)$ is undefined. A partial function is Turing-computable if it equals $f_M$ for some machine $M$.

**Transitions without movement.** Some presentations also allow a third direction $N$, in which the head stays in place, and several of the 2024 course solutions used it. This does not change what can be computed: a transition $\delta(q, a) = (q', b, N)$ can be replaced by $\delta(q, a) = (p, b, R)$ together with $\delta(p, c) = (q', c, L)$ for every $c \in \Gamma$, where $p$ is a new state that is not final. After these two steps the machine is in the configuration that the single $N$ step would have produced.

</details>

<details>
<summary>Multi-tape machines</summary>

A $k$-tape Turing machine is a tuple $M = (Q, \Sigma, \Gamma, \delta, q_0, F)$ as above, except that the transition function has the form

$$\delta \colon Q \times \Gamma^k \rightharpoonup Q \times \Gamma^k \times \{L, R, N\}^k .$$

An entry $\delta(q, a_1, \dots, a_k) = (q', b_1, \dots, b_k, D_1, \dots, D_k)$ means: in state $q$, if head $i$ reads $a_i$ for every $i$, then head $i$ writes $b_i$ and moves in direction $D_i$, where $N$ means that it stays in place, and the machine continues in state $q'$. The input is written on the first tape, all other tapes are blank, and the result of a computation is read from the first tape. A configuration consists of the state and one pair $(u_i, v_i)$ for each tape $i$.

**Simulation on one tape.** Let $M$ be a $k$-tape machine. A single-tape machine $M'$ uses the tape alphabet $\Gamma \cup (\Gamma \times \{0, 1\})^k$. A cell of $M'$ holds, for each tape of $M$, the symbol in the corresponding cell together with a bit that marks whether the head of that tape is there. $M'$ first rewrites its input into this track format. To simulate one step of $M$, it sweeps over the used part of its tape and collects the $k$ symbols under the marked heads in its state, which is possible because there are only finitely many combinations, and then sweeps back to write the new symbols and move the marks.

If $M$ runs for $t$ steps on an input of length $m$, every tape of $M$ uses at most $m + t + 1$ cells, so one simulated step costs $O(m + t)$ steps of $M'$ and the whole simulation costs $O(t\,(m + t))$ steps, which is $O(t^2)$ when $t \ge m$. $M'$ accepts the same language as $M$, and if it converts the first track back into ordinary symbols at the end, it computes the same function.

</details>
