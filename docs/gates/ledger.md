# Harness Gate: Ledger (`cargo xtask ledger`)

## Summary

`cargo xtask ledger` manages the hard-gate ledger for worktree verification and CI proof aggregation.

## Subcommands & Standard Flags

- `cargo xtask ledger tree-hash`: Computes the worktree tree-hash using a temporary git index file (`GIT_INDEX_FILE`). Captures untracked files and modifications while respecting `.gitignore`.
- `cargo xtask ledger write --results-dir <dir> --out <file>`: Aggregates individual gate JSON reports into a single ledger JSON document containing `tree_hash`, `head`, `generated_at`, `gates[]`, and `overall_status`.
- `cargo xtask ledger verify --ledger <file>`: Validates that the ledger `tree_hash` matches the current worktree tree-hash and that `overall_status` is `PASS`.

Standard flags supported: `--root <dir>`, `--base <rev>`, `--head <rev>`, `--json`.

## Invariants & Compliance

- **FAIL-CLOSED**: Missing or invalid gate inputs result in a `FAIL` / `ERROR` state.
- **Isolationsregel**: Worktree hashes capture untracked files to guarantee that green status applies strictly to the exact file contents submitted.
