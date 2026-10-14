# Contributing

Thanks for your interest. Issues, questions, new gallery machines and pull requests are welcome.

## Setup

```bash
rustup show                                             # installs the toolchain from rust-toolchain.toml
cargo install wasm-bindgen-cli --version 0.2.129 --locked
cd web && npm ci                                        # Node 24
pre-commit install                                      # optional: format, gitleaks and the style check on every commit
```

## Checks

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --release -p turing-bb -- --include-ignored   # full enumeration of 4-state machines
python3 scripts/check_style.py
cd web && npm run build && npm run check && npm test && npx playwright test
```

## Conventions

* Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, ...); release-please builds the changelog from them.
* No `unsafe` code. Public items are documented; behaviour that is easy to get wrong has a doctest or a property test.
* A new decider comes with a soundness argument in its documentation and must keep the node-by-node agreement with the Coq-verified enumeration.
* Gallery machines live in `machines/` with their provenance and expected results; the golden tests run every example.
* Prose uses "to" for ranges and commas for lists; the style check rejects em dashes, en dashes and middle dots. Numbers in texts are computed live or come with a source.
