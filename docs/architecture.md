# Architecture

The repository is a Cargo workspace of five crates and a static web site. All computation is
done in Rust; the web site compiles the Rust code to WebAssembly and runs it in the visitor's
browser.

```text
                      turing-core
           (model, simulators, language)
              ▲            ▲          ▲
              │            │          │
         turing-bb    turing-render   │
       (enumeration,  (space-time,    │
        deciders,      layout, SVG)   │
        Antihydra)         ▲          │
              ▲            │          │
              ├────────────┼──────────┤
              │            │          │
         turing-cli    turing-wasm ───┘
          (`tm`)      (bindings, DTOs)
                           │
                           ▼
                web/ (SvelteKit, static)
```

## turing-core

`no_std` with `alloc`, no I/O, no unsafe code. It defines the machine model and everything that
runs a machine.

* **Model.** `Symbol` and `State` are newtypes over small integers; `Dir` is `L`, `R` or `N`.
  A `Machine` is validated once and stores δ as a dense table indexed by state and the tuple of
  read symbols, so a step is one index computation and one load. Machines have one to three
  tapes. `MachineBuilder` builds a machine from names and glyphs and reports every error at
  once.
* **Interchange.** `MachineSpec` is the editable form: names in their original order, which
  sets were declared, comments and source spans. The description language (`dsl`) parses into
  a spec and formats a spec back; `MachineSpec::compile` produces a `Machine` and keeps the
  transition order, so `Machine::origin` maps every table entry back to the line it came from.
  The bbchallenge standard format (`standard`) is supported in both directions.
* **Simulators.** `Simulator` runs any machine on dense two-way infinite tapes. Its single-tape
  hot loop keeps head, state and step count in locals and reads a packed copy of δ that stores
  the next state's row offset; it reaches about 490 million steps per second on an Apple M1
  Pro. `ChainSim` runs single-tape machines on a run-length encoded tape and executes a
  transition that loops in place over a whole run of equal symbols in one macro step; the fifth
  busy beaver champion's 47,176,870 steps take 67,346 macro steps. Both simulators stop exactly
  at any requested step and recognise two simple kinds of endless loops. `Timeline`
  checkpoints either simulator so that a run can be moved backwards.
* **Testing.** Property tests compare both simulators with a deliberately naive reference
  simulator on random machines, and the gallery tests run every documented example.

## turing-bb

Busy beaver search, faithful to the Coq proof of BB(5) by the bbchallenge collaboration.
`Enumerator` walks the tree normal form depth first and classifies each node with a pipeline of
deciders (`Loops`, `NGramCps` with history and LRU augmentations, `RepWl`), with the parameters
of Coq-BB5. The enumeration advances in bounded chunks and can be split into shards, so the web
site can run it in workers with a progress display. `cryptids::antihydra` checks Antihydra's
high-level rules on the machine itself and iterates the abstract rule with big integers. See
[`deciders.md`](deciders.md).

## turing-render

Pictures without a graphics dependency. `SpaceTime` renders a run into an index buffer (one
byte per pixel: background, ink shades, head) in bounded slices and colours it separately, so a
change of theme does not re-run the machine. Long runs are sampled exactly with `ChainSim`.
`layout` and `route` place states and draw edges for the state diagram (layered or circular,
curved edges around obstacles, self-loops where there is room, labels placed by scoring
collisions); `svg` writes a complete diagram.

## turing-cli

The `tm` command: `run`, `trace`, `check`, `fmt`, `convert`, `enumerate`, `decide`, `render`
and `antihydra`. Diagnostics are printed with source snippets. The tests run every command
against snapshots with `trycmd`, including the examples in the crate's README.

## turing-wasm

The boundary between Rust and the web site. Every type that crosses it is a DTO with a
generated TypeScript definition (`tsify`), so the TypeScript code is checked against the Rust
code at compile time. Values are deserialised inside the exported functions, never in the ABI
glue, so malformed input from JavaScript becomes an ordinary error and cannot leak memory.
Source positions are converted from UTF-8 bytes to UTF-16 code units, which is what JavaScript
strings and CodeMirror use.

The bindings form two groups:

* **Editor services**, synchronous, on the main thread: parse, format, check, standard format,
  share links, diagram layout and edge routing. Routing is fast enough to run on every frame
  while a state is dragged.
* **Engines**, in Web Workers: `Session` (step, seek in both directions, run with
  breakpoints), `SpaceTimeJob`, `EnumerationJob` and the Antihydra jobs. Each advances in
  chunks that the worker sizes to about 16 ms, so long computations report progress about ten
  times a second and can be cancelled between chunks.

## web

A SvelteKit site, prerendered to static HTML and served by GitHub Pages.

* **Workers.** `lib/engine/protocol.ts` defines the messages between the page and two workers:
  `engine` for interactive work (playground sessions, space-time diagrams) and `batch` for long
  jobs (enumeration, Antihydra), so a running enumeration never delays a step in the
  playground. Every request has an id; long jobs stream responses and end with a final one.
* **Playground.** The editable `MachineSpec` is the single source of truth. The transition
  table, the state diagram and the text editor all edit it; Rust parses, validates, formats and
  lays it out. While the text has errors, the other views keep the last valid machine.
* **Content.** The chapters are Markdown files rendered at build time: mathematics becomes
  MathML, `{{fact:id}}` placeholders become values from a ledger of facts with sources, and
  citations are numbered per page. Nothing of this ships to the browser.
* **Design.** The design tokens are those of the author's homepage. The site sets no cookies,
  loads nothing from other origins, and a content security policy restricts scripts, workers
  and connections to the site itself.

## Checks

`cargo fmt`, `cargo clippy` with the pedantic lints and `-D warnings`, `cargo test` (unit,
property, golden and snapshot tests), `cargo doc` with `-D warnings`, a `no_std` build of
turing-core, `cargo deny`, the house style check, and on the web side `svelte-check`, Prettier,
Vitest, and Playwright with axe in both colour schemes on desktop and phone widths. A separate
workflow compares the 4-state enumeration with the Coq-verified one every week.
