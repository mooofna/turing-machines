# Notes for the reviewer

## Files

- `00-home.md`: title, lede, hero caption, one sentence per chapter, playground and gallery.
- `01-machine.md` to `06-frontier.md`: the six chapters. Visible prose is roughly 670 to 900 words per chapter (counted without math and placeholders); definitions and proofs are in `<details>` blocks.
- `07-about.md`: origin, repository, privacy, sources, licence, citation, acknowledgements.
- `08-gallery-notes.md`: one or two sentences per gallery machine, keyed by id.

All files were checked for em dashes, en dashes, middle dots, exclamation marks, question marks and the banned words; none remain. "Runs forever" occurs only where non-halting is proved (constructed loops, cyclers, the conditional statement about the ZF machine).

## Placeholder and citation ids

After the fact-check message, placeholders and citations were switched to the ids in `research/facts.json` and `research/sources.json`. Mapping from the brief: `bb5-proof-date` to `bb5-announcement-date`, `bb5-enumerated` to `bb5-enumerated-machines`, `bb5-sporadic` to `bb5-sporadic-machines`, `tnf-nodes-bbN` to `bbN-tnf-count`, `bb6-bound` to `bb6-lower-bound` (its display string already contains "S(6) > Σ(6) > ...", so the text says "establishes the bound {{fact:bb6-lower-bound}}"), `antihydra-date` to `antihydra-discovery`, `zf-states` to `zf-record-established` (432), `pa-states` to `pa-tm-states`, `machines-5-states` to `count-bb5-rado` (chapter 2) and `count-bb5-aaronson` (chapter 5). Sources: `lin-rado1965` to `linrado1965`, `coq-bb5` to `coqbb5`, `bb6-wiki` to `wiki-bb6`, `zf-wiki` to `wiki-logical-independence`, `smith2007` to `smith2020`, `church1936` to `church1936a` and `church1936b`, and so on.

Ids used in the text that are not yet in the ledger and need to be added:

- `{{fact:bb6-champion-date}}`: 25 June 2025 (from the note of `champion-bb6`).
- `{{fact:bb6-holdouts-date}}`: 10 October 2026 (from the note of `bb6-holdouts`).
- `{{fact:zf-tm-254-date}}`: 13 October 2026 (from the note of `zf-tm-254`). The sentence follows the coordinator's wording: announced with a Coq formalisation, awaiting review.
- `{{fact:zf-history}}`: a table, to be rendered from the `zf-tm-*` facts (states, date, discoverer, verification status). It sits inside the details block "History of the record" in chapter 6.
- `[@theolog2024]` (the exercise sheets) and `[@bbchallenge-format]` (the standard text format) are not in `sources.json`.
- `{{bibliography}}` in `07-about.md` marks where the bibliography is rendered.

Live values introduced in addition to the brief (format `{{live:ID: description}}`): `tower-4`, `tower-5-digits`, `antihydra-checked`, `antihydra-balance`. From the brief: `hero-steps`, `hero-ones`, `hero-time`, `antihydra-iterations`.

## Literal numbers that remain

Only structural numbers: numbers of states and symbols in definitions, rule constants in the Antihydra and Hydra rules ($b = 4$, $h = 8$, $h = 3$, steps $+2$ and $-1$), the worked example of the increment machine (input 11 is three, output 001 is four), "at most five states" in the Rice example, "between six and" (derived from the proof of $S(5)$), the figure parameters $n = 0$ to 20 (fig. 1.3) and $k = 5$ (fig. 6.1), the years 2024 and 2026 describing the project itself, the licence version and the BibTeX year. Machine strings in the text format are written literally; they match the ledger entries `champion-bb2`, `champion-bb3`, `champion-bb4`, `champion-bb5`, `champion-bb23`, `champion-bb24`, `champion-bb6`, `antihydra-machine` and `wolfram23-machine`, and can be replaced by `{{fact:champion-...}}` if preferred.

## Content decisions to check

1. **Directions.** Single-tape machines use $D \in \{L, R\}$ as the brief asks; multi-tape machines use $\{L, R, N\}^k$. Several 2024 course solutions used a stay move; the formal-definition block shows how to replace it. If the course definition itself allows $N$ on one tape, adjust the wording.
2. **Halting problem notation.** $P_{\mathrm{halt}} = \{\mathrm{enc}(M)\#\#w \mid M \text{ halts on } w\}$. The exercise sheets use `##` between encodings; please check whether the lecture writes `enc(M)##w` or `enc(M)##enc(w)`.
3. **Non-computability proof (chapter 4).** I did not have the lecture slides. The visible proof idea and the full proof use $M_{*2}$ as described on exercise sheet 1 (lecture 2, slide 27), proved directly for $S$ (via "writing $S(2n)$ ones takes at least $S(2n)$ steps") and then for $\Sigma$, with a strict-monotonicity lemma and an explicit input/output convention ("normal form": head on the blank cell left of the block). With $n = c + k + 1$ the contradiction is $S(2n-1) \ge S(2n)$. Please compare with the lecture's version; the text only claims that the argument "follows the proof in lecture 2".
4. **Busy beaver model.** $n$ states plus a separate halting state $H$; every entry defined; the transition into $H$ counts as a step; `---` is treated as a halting transition when reached. This matches the bbchallenge convention ($S(1) = 1$).
5. **Attributions (per fact check).** $\Sigma(2)$: Radó; $S(2)$: Lin, published with Radó; $n = 3$: Lin, published with Radó; $n = 4$: Brady, independently Machlin and Stout. Two descriptions are plausible but not in the ledger: Lin's "program plus separate arguments for unclassified machines", and Marxen and Buntrock's "simulator that accelerates repetitive behaviour".
6. **Three states.** The text states that no three-state machine attains both $S(3)$ and $\Sigma(3)$. I confirmed this by brute force over all three-state machines with explicit halting transitions (machines with 21 steps leave 4 or 5 ones; machines with 6 ones halt after 11 to 14 steps), consistent with ledger entries `champion-bb3` and `champion-sigma3`. Fig. 4.1 is described as showing machines that attain $S(n)$.
7. **Counting conventions.** Chapter 2 and Fig. 2.2 use Radó's $(4(n+1))^{2n}$ and name the alternative $(4n+1)^{2n}$; chapter 5 uses the paper's count and says so.
8. **Deciders.** Each has an intuitive sentence and a "Precisely:" sentence. The translated-cycler criterion is proved in a details block. The text says the proof combines cyclers and translated cyclers in one decider (Loops) and uses further methods (FAR, WFAR) for a small number of machines, without numbers.
9. **ZF independence.** The details give a proof that, if ZF is consistent, ZF proves no bound $S(n) \le k$ for $n \ge m$, assuming a machine $Z$ with $m$ states such that ZF proves "$Z$ halts iff ZF is inconsistent". This needs only consistency; the wiki states the general result for consistent and arithmetically sound theories. Yedidia and Aaronson's machine is described separately (ZFC, relies on an unpublished result of Friedman), since it is not a proof-searching machine.
10. **Antihydra.** The rule is stated in the wiki's $(a, b)$ form, with $A(a, b)$ written as the course configuration $1^a\,0\,1^b\,E$ (head on the blank to the right of the second block) and the restriction $k \ge 1$. The visible text gives the equivalent description with $b \mapsto \lfloor 3b/2 \rfloor + 2$; details give the $h$ form, $h_i = \lfloor K(3/2)^i \rfloor + 1$ (Odlyzko and Wilf, via the wiki) and the relation to Mahler's problem, described as "related", never as equivalent. The random-walk probability $((\sqrt5 - 1)/2)^{n+1}$ is derived in a details block.
11. **Hydra.** Described as the same map $h \mapsto \lfloor 3h/2 \rfloor$ from 3 with even and odd exchanged (ledger `hydra-rules`). I checked that Hydra's parameter in the wiki's $C(a, b)$ form equals $3h - 6$ for the first terms, so the parities agree.
12. **Figure behaviour assumed in the text** (please check against the implementation): Fig. 1.1 uses the 2024 `add1` machine ($\delta(q_0, 1) = (q_0, 0, R)$, $\delta(q_0, 0) = \delta(q_0, ␣) = (q_1, 1, R)$); $A_{\mathrm{mul}}$ works by repeated addition with `#` separators; the two-tape $M_{*2}$ copies in a single pass (linear) and the one-tape version walks back and forth (quadratic); Fig. 4.1 draws ones dark and the head in blue and compresses the five-state diagram in time; Fig. 5.2 shows "the evidence the decider found"; Fig. 6.2 first confirms rule applications against the machine.
13. **Gallery.** The notes for `palindromes`, `anbncn`, `binary-addition` and `mod-two` describe the usual algorithms and should be checked against the actual machines. The 2024 enumerators for task 2a and 2b do not output the empty word ($n = 0$); the notes avoid claiming that it is listed.
14. **About page.** The crate descriptions follow the brief only. The BibTeX URL `https://mooofna.github.io/turing-machines/` is a guess at the deployed address. The acknowledgement is written in the first person.
15. **Rendering.** The math uses `\text{␣}` for the blank, `\rightharpoonup` for partial functions, `\tfrac`, `\bmod` and `\square`; please check that the MathML renderer supports them. Figure captions contain plain-text formulas (A_mul, M*2, 2↑↑k) because they may not pass through the math renderer.
