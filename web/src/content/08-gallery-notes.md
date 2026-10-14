# Gallery notes

One or two sentences per gallery machine, keyed by the machine's id.

## binary-increment

Adds one to a binary number written with the least significant bit first, so that the carry moves in the same direction as the head. It is the first example in the lecture of the course [@theolog2024-lecture1] and the first machine of the 2024 simulator, and Fig. 1.1 steps through it.

## multiples-of-three

An enumerator for the multiples of three in unary, from task 2a of the first exercise sheet [@theolog2024]. Each time the machine enters its output state, the tape holds the next word of the list, starting with the empty word.

## an-bn

An enumerator for the language $\{a^n b^n \mid n \in \mathbb{N}\}$, from task 2b of the first exercise sheet [@theolog2024]. It lists the words in order of increasing $n$, obtaining each from the previous one by adding one $a$ and one $b$.

## unary-multiplication

The machine $A_{\mathrm{mul}}$ from task 3 of the first exercise sheet, which multiplies two numbers in unary by repeated addition [@theolog2024]. This exercise is the origin of the project; Fig. 1.2 runs the machine with decimal input and output.

## doubling

The machine $M_{*2}$ on a single tape, computing $x^n \mapsto x^{2n}$ (task 4b of the first exercise sheet) [@theolog2024]. It walks back and forth across the growing block for every input symbol, so its number of steps grows quadratically in $n$.

## doubling-two-tapes

The machine $M_{*2}$ with two tapes (task 4a of the first exercise sheet) [@theolog2024]. With the second tape it copies the input in a single pass, so its number of steps grows linearly in $n$; Fig. 1.3 compares both versions.

## mod-two

The machine $A_{\mathrm{mod2}}$ from task E of the third exercise sheet, computing $x^n \mapsto x^{n \bmod 2}$ in unary [@theolog2024]. It erases the input from the right, keeps the parity of the number of erased symbols in its state, and writes a single $x$ at the end if that number is odd.

## palindromes

An acceptor for palindromes, the words that read the same from left to right and from right to left. It compares the first and the last remaining symbol, erases both and repeats, so its number of steps grows quadratically with the length of the input.

## anbncn

An acceptor for the language $\{a^n b^n c^n \mid n \ge 0\}$, which is not context-free. In each pass over the input it marks one $a$, one $b$ and one $c$, and it accepts when all symbols are marked at the same time.

## binary-addition

A transducer that adds two binary numbers, written with the most significant bit first and separated by +, and leaves their sum on the tape. It adds the bits of the second number to the first one at a time, from the right, and lets carries run to the left.

## bb1

The busy beaver champion for one state: it writes a 1, moves and halts [@wiki-bb1m]. Steps: {{fact:bb1-steps}}; ones: {{fact:bb1-ones}}.

## bb2

The two-state champion `1RB1LB_1LA1RZ`, which attains both $S(2)$ and $\Sigma(2)$ [@wiki-bb2]. Steps: {{fact:bb2-steps}}; ones: {{fact:bb2-ones}}.

## bb3

The three-state machine `1RB1RZ_1LB0RC_1LC1LA`, which attains $S(3)$ = {{fact:bb3-steps}}. No three-state machine attains both maxima: this one leaves fewer than $\Sigma(3)$ = {{fact:bb3-ones}} ones, and the machines that leave $\Sigma(3)$ ones halt earlier [@linrado1965] [@wiki-bb3].

## bb4

The four-state champion `1RB1LB_1LA0LC_1RZ1LD_1RD0RA`, which attains both $S(4)$ and $\Sigma(4)$ [@brady1983] [@wiki-bb4]. Steps: {{fact:bb4-steps}}; ones: {{fact:bb4-ones}}.

## bb5

The five-state champion `1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA`, found by Marxen and Buntrock [@marxenbuntrock1990] and proved optimal by the bbchallenge collaboration [@bbchallenge2025]. Steps: {{fact:bb5-steps}}; ones: {{fact:bb5-ones}}.

## bb2x3

The champion for two states and three symbols, `1RB2LB1RZ_2LA2RB1LB`. It halts after {{fact:bb23-steps}} steps, the largest number possible for this class [@lafittepapazian2007] [@wiki-bb23].

## bb2x4

The champion for two states and four symbols, `1RB2LA1RA1RA_1LB1LA3RB1RZ`. It halts after {{fact:bb24-steps}} steps, the largest number possible for this class [@coqbb5] [@wiki-bb24].

## bb6

The current champion for six states, `1RB1RA_1RC1RZ_1LD0RF_1RA0LE_0LD1RC_1RA0RE` [@wiki-bb6]. It is known to halt, and its running time establishes the bound {{fact:bb6-lower-bound}} [@wiki-bb6-champion]. That is far more than can be simulated step by step, so the gallery shows the beginning of its run.

## antihydra

A six-state cryptid whose halting problem is a Collatz-like question about the map $h \mapsto \lfloor 3h/2 \rfloor$ started at 8 [@wiki-antihydra]. Whether it halts is open; chapter 6 explains the rule.

## hydra

A cryptid with two states and five symbols that follows the same map as Antihydra from a different starting value, with the roles of even and odd values exchanged [@wiki-hydra]. Whether it halts is open.

## bigfoot

A cryptid with three states and three symbols whose behaviour follows a Collatz-like rule [@wiki-bigfoot]. Whether it halts is open.

## wolfram-2-3

Wolfram's machine with two states and three symbols, `1RB2LA1LA_2LA2RB0RA`, which has no halting transition [@wolfram2002]. Smith proved it universal in an extended sense that requires an infinite, non-periodic initial tape [@smith2020]; whether this counts as universality has been debated [@pratt2007fom], and chapter 2 discusses the caveat.
