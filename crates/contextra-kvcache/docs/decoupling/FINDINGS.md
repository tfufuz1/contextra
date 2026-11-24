# Findings: Decoupling contextra-kvcache

## WP-0 Baseline Findings
- `cargo check -p contextra-kvcache --no-default-features`:
  Fails as expected with 18 unresolved imports due to ungated usage of `contextra_types`, `contextra_ports`, `contextra_crypto`, and items in `compat` gated under `ports-compat`.
- `cargo check -p contextra-kvcache --all-targets`:
  Passes without errors on default features (`default = ["std", "ports-compat", "crypto-compat", "kv-encryption"]`).
- `cargo test -p contextra-kvcache`:
  40 unit tests pass in `lib`, all integration test suites pass (1 test ignored: `generate_all_golden` as expected).
- External workspace users:
  `grep -rEn "contextra_kvcache" crates benchmarks --include=*.rs` shows NO external crates in the workspace directly depend on `contextra-kvcache`.
- In `contextra-vector/src/diskann/build.rs`, a syntax error currently prevents `cargo check --workspace --all-targets` from passing, which is an external issue outside the scope of `crates/contextra-kvcache/**`.
- Two `CURRENT_KV_KEY_DERIVATION_VERSION` constants exist in `contextra-crypto` and `contextra-kvcache`. Per instructions, these are kept separate and not unified.
