# P09: OPEN ISSUES & FREMDFEHLER

## Foreign Failures (Fremdfehler außerhalb BESITZ)

1. **Gate Stack / Compile Errors in Foreign Crates**:
   - `contextra-graph`: Unresolved imports `contextra_ports` and `contextra_types` in `src/graph_index.rs`.
   - `contextra-vector`: Duplicate definitions of `remove` and `rebuild_sync`, missing `zeroize` in scope, unknown field `kernel`.
   - `contextra-crypto`: Compilation fails in multiple CI workflows (`proof-tests`, `Fast Unit Tests`, `Coverage Gate (llvm-cov)`, `Retrieval Quality Regression Gate`) with:
     - 250 clippy errors in `Compile & Security Check Gate` (doc list item without indentation, casting truncation/sign loss, indexing/slicing errors in `chain.rs`, `creation.rs`, `types.rs`, `kdf.rs`, `kv_cipher.rs`, `kv_segment/store.rs`, `wal_crypto.rs`).
     - Compiler errors in `kv_segment` when building under `--all-targets` or coverage instrumentation.
   - `contextra-text`: 55 clippy errors in `Clippy Check` (disallowed types `HashMap`, `HashSet` under note P28, slice panics, truncation).
   - `contextra-rank`: Clippy error `casting f64 to f32 may truncate the value` in `src/fusion/rrf.rs:89`.

2. **Missing `xtask` Package in Workspace**:
   - Workflows `Check Duplicate Symbols Cross File Gate`, `Capability Marker Drift Gate`, `FlatBuffers Schema Drift Gate`, `Ring & Safety Checks`, `Phantom Commit Protection Gate`, `Cargo Mutants Runner` failed with:
     `error: package(s) xtask not found in workspace /home/runner/work/contextra/contextra` or `manifest path xtask/Cargo.toml does not exist`.
   - Root workspace `Cargo.toml` does not contain `xtask` as a workspace member.
   - `Cargo Mutants Runner` additionally fails pushing `docs/mutation_score_history.jsonl` because the pathspec did not match any files (exit code 128) and xtask mutation recorder failed.

3. **Supply-chain / Vulnerability Bans in Root (`deny.toml` / `Cargo.lock`)**:
   - `cargo deny` failed on RUSTSEC-2026-0424 (`wasmtime 36.0.19`), RUSTSEC-2026-0367, RUSTSEC-2026-0368, RUSTSEC-2026-0369, RUSTSEC-2026-0429 (`zip 0.6.6`, `zip 4.6.1`, `zip 8.6.0`).
   - Root `deny.toml` bans / advisory policies are outside P09 BESITZ.

4. **Production Unwrap Counter Gate**:
   - CI check reports 1333 unwrap/expect occurrences across `crates/`, exceeding `.github/unwrap_baseline.txt` (1197).
   - Verified that P09 added 0 unwrap/expect in production code within BESITZ (`checkpoint`, `testkit`, `sys`, `wire`, `durable-fs`). The baseline delta is entirely in pre-existing or parallel crates outside P09 BESITZ.

5. **Format Check Gate**:
   - `cargo fmt --all -- --check` fails in root due to unformatted files in `crates/contextra-store/` and `crates/contextra-vector/`. Files in P09 BESITZ are fully formatted.
