## What and why

## Checks

- [ ] `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass
- [ ] Changes to deciders: `cargo test --release -p turing-bb -- --include-ignored` passes
- [ ] Website changes: `npm run check`, `npm test` and `npx playwright test` pass in `web/`
- [ ] `python3 scripts/check_style.py` passes
