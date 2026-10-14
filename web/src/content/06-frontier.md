# The frontier

For six states the value of $S$ is unknown. This chapter describes what is known about six-state machines, the Collatz-like problems that appear among them, and a result showing that beyond a certain number of states the value of $S$ cannot be proved from the usual axioms of mathematics.

## Six states

The record for six states has been raised many times [@wiki-bb6] [@michel-survey]. The current champion, discovered by mxdys ({{fact:bb6-champion-date}}), establishes the bound {{fact:bb6-lower-bound}} [@wiki-bb6] [@wiki-bb6-champion]. The bound follows from a mathematical analysis of the machine, since its running time is far beyond step-by-step simulation, and its halting has also been proved in Coq [@wiki-bb6-champion]. On {{fact:bb6-holdouts-date}}, {{fact:bb6-holdouts}} six-state machines, counted up to equivalence, remained whose behaviour from the blank tape had not been determined [@wiki-bb6].

## Up-arrow notation

Numbers of this size are written in Knuth's up-arrow notation [@knuth1976]. A single arrow denotes exponentiation, $a \uparrow b = a^b$, and each additional arrow iterates the operation with one arrow fewer. Thus $a \uparrow\uparrow b$ is a tower of $b$ copies of $a$, evaluated from the top, as in $2 \uparrow\uparrow 3 = 2^{2^{2}}$. The number $2 \uparrow\uparrow 4$ is {{live:tower-4: value of 2↑↑4}}, and $2 \uparrow\uparrow 5 = 2^{2 \uparrow\uparrow 4}$ has {{live:tower-5-digits: number of decimal digits of 2↑↑5}} decimal digits. The number of decimal digits of $2 \uparrow\uparrow 6$ itself has about as many digits as $2 \uparrow\uparrow 5$, so $2 \uparrow\uparrow 6$ cannot be written out in full.

<<figure:fig6.1 | Fig. 6.1. Towers of twos, 2↑↑k, computed exactly in your browser. Choose k to see the value, and for k = 5 the number of its decimal digits and the digits themselves.>>

## Cryptids

A cryptid is a machine whose behaviour from the blank tape can be described by a fairly simple mathematical rule, where deciding whether the machine halts amounts to an open problem about that rule; the definition is informal [@wiki-cryptids]. Most known cryptids follow Collatz-like rules, in which a number is transformed repeatedly by a formula that depends on its remainder modulo a fixed integer. If a cryptid with $n$ states exists, determining $S(n)$ requires settling its problem, since with $S(n)$ known one could run the cryptid for $S(n)$ steps and see whether it halts. The same holds for machines constructed to halt exactly when a known conjecture fails, which the details below list.

## Antihydra

Antihydra is the six-state machine `1RB1RA_0LC1LE_1LD1LC_1LA0LB_1LF1RE_---0RA`, whose irregular behaviour was first reported in the bbchallenge community ({{fact:antihydra-discovery}}) [@wiki-antihydra]. Its run can be described by a value $b$ and a counter $a$: starting from $a = 0$ and $b = 4$, each round replaces $b$ by $\lfloor 3b/2 \rfloor + 2$. If $b$ was even, $a$ increases by 2; if $b$ was odd, $a$ decreases by 1. The machine halts when $b$ is odd while $a = 0$, that is, as soon as the odd values of $b$ outnumber twice the even ones. Whether this ever happens is open.

<<figure:fig6.2 | Fig. 6.2. Antihydra checked against its rule and then iterated. The page first runs the machine itself and confirms the first {{live:antihydra-checked: number of rule applications confirmed by direct simulation of the machine}} rule applications, then applies the rule {{live:antihydra-iterations}} times and plots the counter a.>>

Within {{live:antihydra-iterations}} iterations the counter has not reached the halting condition; after the last one its value is {{live:antihydra-balance: value of the counter a after the last iteration}}. The computation says nothing about later iterations. The values of $b$ are closely tied to the powers of $3/2$ [@wiki-antihydra] [@odlyzkowilf1991], and the question about their parities resembles Mahler's open question whether some real $\xi > 0$ makes the fractional part of $\xi (3/2)^n$ less than $1/2$ for every $n$ [@mahler1968]. If the parities behaved like fair coin flips, the counter would perform a random walk with steps $+2$ and $-1$ that drifts upward and almost certainly never returns to zero from a large value. No proof is known that the parities behave this way, so the heuristic does not settle the question.

## Independence from set theory

Most of mathematics can be formalised in Zermelo-Fraenkel set theory (ZF). By Gödel's second incompleteness theorem, ZF cannot prove its own consistency if it is consistent. A machine that searches through all proofs from the axioms of ZF and halts when it finds a contradiction therefore runs forever if ZF is consistent, but ZF cannot prove that it does. The first explicit machine whose behaviour ZFC cannot settle was given by Yedidia and Aaronson [@yedidiaaaronson2016], and later machines of the proof-searching kind have far fewer states. The smallest established one has {{fact:zf-record-established}} states [@wade2025] [@wiki-logical-independence]. A machine with {{fact:zf-tm-254}} states was announced on {{fact:zf-tm-254-date}} with a Coq formalisation and is awaiting review [@ketchersid2026zf254].

If ZF is consistent, it follows that ZF proves no upper bound for $S(n)$, and in particular does not determine $S(n)$, for any $n \ge$ {{fact:zf-record-established}}. Since the proof of $S(5)$ uses only methods that can be formalised in ZF, the threshold from which on ZF determines no value of $S$ lies between six and {{fact:zf-record-established}} [@wiki-logical-independence].

## More symbols

The busy beaver functions are also studied for machines with $n$ states and $m$ symbols; $S(n, m)$ denotes the largest running time [@aaronson2020]. The values $S(2, 3)$ = {{fact:bb23-steps}} [@lafittepapazian2007] [@wiki-bb23] and $S(2, 4)$ = {{fact:bb24-steps}} [@wiki-bb24] are known, and the Coq development of chapter 5 verifies both [@coqbb5]. For two states with five symbols, the cryptid Hydra follows a rule of the same kind as Antihydra, with the roles of even and odd values exchanged; Antihydra was named after it [@wiki-hydra] [@wiki-antihydra]. For three states with three symbols, the cryptid Bigfoot follows a Collatz-like rule of a different form [@wiki-bigfoot]. The values $S(2, 5)$ and $S(3, 3)$ therefore depend on open problems.

## The state of knowledge

The values of $S$ and $\Sigma$ are known for machines with at most five states and two symbols, and $S$ is known for two states with three and with four symbols. For six states, the best known lower bound is {{fact:bb6-lower-bound}}, and no upper bound is known. Determining $S(6)$ requires deciding whether Antihydra halts, a question of the same kind as open questions about the powers of $3/2$. From {{fact:zf-record-established}} states on, ZF determines no value of $S$, assuming that ZF is consistent. For the numbers of states in between, it is not known which values ZF can prove.

<details>
<summary>Notes on the six-state numbers</summary>

The bound above is the one stated on the bbchallenge wiki. The paper on the five-state busy beaver states the weaker bound {{fact:bb6-lower-bound-conservative}} for the same machine [@bbchallenge2025]. The holdout count above identifies equivalent machines; counted without this identification, the list contained {{fact:bb6-holdouts-no-equivalence}} machines on the same date [@wiki-bb6]. Both counts change as machines are resolved.

</details>

<details>
<summary>Up-arrow notation, formally</summary>

For integers $a \ge 1$, $b \ge 1$ and $k \ge 2$,

$$a \uparrow b = a^b, \qquad a \uparrow^{k} 1 = a, \qquad a \uparrow^{k} (b + 1) = a \uparrow^{k-1} \bigl(a \uparrow^{k} b\bigr),$$

where $\uparrow^2$ is written $\uparrow\uparrow$ and $\uparrow^3$ is written $\uparrow\uparrow\uparrow$ [@knuth1976]. For example, $2 \uparrow\uparrow\uparrow 3 = 2 \uparrow\uparrow 4$, and $2 \uparrow\uparrow\uparrow 4 = 2 \uparrow\uparrow (2 \uparrow\uparrow 4)$ is a tower of twos whose height is $2 \uparrow\uparrow 4$; in the same way, $2 \uparrow\uparrow\uparrow 5$ is a tower of twos whose height is $2 \uparrow\uparrow\uparrow 4$. Chains of arrows are evaluated from the right: $a \uparrow\uparrow b \uparrow\uparrow c$ means $a \uparrow\uparrow (b \uparrow\uparrow c)$, a tower of copies of $a$ whose height is itself a tower. The bound from the wiki quoted above is a chain of this kind.

</details>

<details>
<summary>Machines for known conjectures</summary>

Cryptids are found by searching through small machines. Other machines are constructed so that they halt exactly when a known conjecture fails. A machine with {{fact:goldbach-tm-states}} states halts if and only if Goldbach's conjecture is false [@leng-goldbach]. Stérin and Woods built a machine with {{fact:erdos-tm-states}} states that halts if and only if a conjecture of Erdős about the base-3 digits of powers of two is false [@sterinwoods2024]. For the Riemann hypothesis, the corresponding machine has {{fact:riemann-tm-states}} states [@aaronson2020]. Knowing $S$ for these numbers of states would settle each of these conjectures, in the same way as for cryptids.

</details>

<details>
<summary>Antihydra's rule in detail</summary>

Write $A(a, b)$ for the configuration $1^a\,0\,1^b\,E$ in the notation of chapter 1: the tape holds a block of $a$ ones, a zero and a block of $b$ ones, and the machine is in state $E$ on the blank cell to the right of the second block. Then, for all $a \ge 0$ and $k \ge 1$,

$$A(a,\ 2k) \vdash^* A(a + 2,\ 3k + 2), \qquad A(a + 1,\ 2k + 1) \vdash^* A(a,\ 3k + 3),$$

and from $A(0,\ 2k + 1)$ the machine reaches an undefined transition after finitely many further steps and halts. Started on the blank tape, the machine reaches $A(0, 4)$ [@wiki-antihydra]. Both rules replace $b$ by $\lfloor 3b/2 \rfloor + 2$, which gives the description in the text.

The shifted values $h = b + 4$ follow the simpler rule $h \mapsto \lfloor 3h/2 \rfloor$, starting from $h = 8$, and have the same parities as $b$. Writing $h_i$ for the value after $i$ rounds, there is a real constant $K$ with $h_i = \lfloor K (3/2)^i \rfloor + 1$ for all $i$ [@wiki-antihydra] [@odlyzkowilf1991]. Whether Antihydra halts is therefore a question about the parities of $\lfloor K (3/2)^i \rfloor$. Mahler's question concerns the fractional parts of $\xi (3/2)^n$ [@mahler1968]. The two questions are related, but they are different questions.

</details>

<details>
<summary>The random-walk heuristic</summary>

Suppose the parities of the successive values of $b$ were independent and each even with probability $1/2$. Then the counter performs a random walk with steps $+2$ and $-1$, each with probability $1/2$, and the machine halts when the walk first reaches $-1$. Let $P(n)$ be the probability that the walk, started at $n \ge 0$, ever reaches $-1$. Then $P(-1) = 1$ and

$$P(n) = \tfrac12 P(n - 1) + \tfrac12 P(n + 2) \qquad (n \ge 0).$$

Trying $P(n) = r^{n+1}$ gives $r^3 - 2r + 1 = 0$, that is $(r - 1)(r^2 + r - 1) = 0$. Since the walk drifts upward, $P(n)$ tends to zero, and the bounded solution with this property is

$$P(n) = \left(\frac{\sqrt5 - 1}{2}\right)^{n+1},$$

as also derived on the wiki [@wiki-antihydra]. The rule has been {{fact:antihydra-simulated}} [@wiki-antihydra]; for a walk at that height, the probability of ever reaching $-1$ is {{fact:antihydra-halting-probability}}. The calculation rests on the assumption of independent fair coin flips, which has not been proved for Antihydra.

</details>

<details>
<summary>Why ZF cannot determine S(n) for large n</summary>

Let $Z$ be a machine with $m$ states such that ZF proves the statement "$Z$ halts if and only if ZF is inconsistent". The proof-searching constructions are designed to have this property. For $n \ge m$, adding $n - m$ unreachable states to $Z$ gives a machine $Z_n$ with $n$ states that halts if and only if $Z$ halts, and ZF proves this as well.

Suppose ZF is consistent and proves $S(n) \le k$ for some number $k$. Since ZF is consistent, $Z_n$ never halts. In particular, $Z_n$ has not halted after $k$ steps; this is a statement about a finite computation, it is true, and ZF proves it by carrying out the computation. Together with $S(n) \le k$, ZF then proves that $Z_n$ never halts, hence that $Z$ never halts, hence that ZF is consistent. This contradicts Gödel's second incompleteness theorem. Therefore, if ZF is consistent, ZF proves no statement of the form $S(n) \le k$ with $n \ge m$, and in particular no statement of the form $S(n) = k$.

A machine can be proved non-halting in ZFC exactly when it can be proved non-halting in ZF, by Shoenfield's absoluteness theorem, so adding the axiom of choice changes nothing here [@wiki-logical-independence]. Peano arithmetic is weaker than ZF, and the corresponding record for it is a machine with {{fact:pa-tm-states}} states [@house2026pa] [@wiki-logical-independence].

</details>

<details>
<summary>History of the record</summary>

Yedidia and Aaronson gave the first explicit machine of this type for set theory, with {{fact:zf-tm-7910}} states, whose non-halting cannot be proved in ZFC if ZFC is consistent; their argument relies on an unpublished result of Harvey Friedman [@yedidiaaaronson2016] [@wiki-logical-independence]. Later machines search directly for a contradiction in ZF and have far fewer states. They are produced by compilers from higher-level programs, and not every entry in the following history has been formally verified [@wiki-logical-independence].

{{fact:zf-history}}

</details>
