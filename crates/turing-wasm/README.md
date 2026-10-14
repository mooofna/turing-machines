# turing-wasm

WebAssembly bindings for the web front end in `web/`. Every type that crosses the boundary is a
DTO in `dto.rs` with a generated TypeScript definition, so the TypeScript side is checked
against the Rust side at compile time.

The bindings come in two groups. Editor services (`parse_source`, `format_spec`, `graph_layout`,
...) are synchronous and run on the main thread. Engines (`Session`, `SpaceTimeJob`,
`EnumerationJob`, the Antihydra jobs) run in a Web Worker and advance in bounded chunks, so a
long computation never blocks the page and can be cancelled between chunks.

Build with `npm run wasm` in `web/`, which calls `cargo build --profile wasm-release`,
`wasm-bindgen --target web` and `wasm-opt`.
