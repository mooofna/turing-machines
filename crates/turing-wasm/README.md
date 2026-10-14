# turing-wasm

WebAssembly bindings for the web front end in `web/`. Every type that crosses the boundary is a
DTO in `dto.rs` with a generated TypeScript definition, so the TypeScript side is checked
against the Rust side at compile time.

The bindings come in two groups. Editor services (`parse_source`, `format_spec`, `graph_layout`,
...) are synchronous and run on the main thread. Engines (`Session`, `SpaceTimeJob`,
`EnumerationJob`, the Antihydra jobs) run in a Web Worker and advance in bounded chunks, so a
long computation never blocks the page and can be cancelled between chunks.

## Rules at the boundary

* Glyphs are strings of exactly one character; anything else is an error that names the field.
* Positions are UTF-16 code units, as in JavaScript strings. Text parsed here (`parseSource`,
  `fromStandard`, `decodeShare`) gets its byte spans converted, with 1-based lines and columns;
  spans of a spec that comes from JavaScript are UTF-16 already and pass through unchanged.
* Counts are JavaScript numbers, exact up to 2^53. Arguments are cut to whole numbers, `NaN`
  and negative values count as 0, larger values as 2^53 - 1; a session never goes beyond step
  2^53 - 1. Unbounded values (tower digits, decoded outputs) cross as decimal strings.
* Malformed arguments throw a JavaScript `Error`; nothing reachable from JavaScript panics.
  Problems in a machine are diagnostics, with the codes of `docs/dsl.md` and three more:
  `E100` for a share link that cannot be decoded, `E101` for text that is not a machine in the
  standard format and `E102` for a glyph in a spec that is not exactly one character.

## Modules

| Module | Contents |
|---|---|
| `dto` | The DTOs and their TypeScript types |
| `read` | Reading the input DTOs from JavaScript values |
| `convert` | DTOs to the workspace types and back, numbers at the 2^53 limit |
| `spans` | UTF-8 byte offsets to UTF-16 offsets, lines and columns |
| `editor` | Parsing, formatting, checking, the standard format, the state diagram, towers |
| `share` | Share link payloads: `1.` + base64url(deflate(description)) |
| `session` | Stepping, exact seeking in both directions, runs with breakpoints |
| `spacetime` | Space-time diagrams |
| `enumerate` | Enumeration in tree normal form and the decider pipeline |
| `cryptids` | Antihydra's rules on the machine and the rule walk |

Build with `npm run wasm` in `web/`, which calls `cargo build --profile wasm-release`,
`wasm-bindgen --target web` and `wasm-opt`. `npm test` in `web/` runs the integration tests of
`web/src/lib/engine/wasm.test.ts` against the built module in Node.
