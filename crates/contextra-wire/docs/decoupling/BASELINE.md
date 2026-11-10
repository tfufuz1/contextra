# Baseline contextra-wire

Erhebungszeitpunkt: 2026-11-10
Commit: Vorbereitung Auftrag wire

## 1. Test-Status
- `cargo test -p contextra-wire --locked`:
  - `src/lib.rs`: 26 Tests (alle OK)
  - `tests/adapter_tests.rs`: 8 Tests (alle OK)
  - `tests/ipc_tests.rs`: 66 Tests (alle OK)
  - `tests/schema_drift.rs`: 2 Tests (alle OK)
  - `tests/verification_robustness.rs`: 8 Tests (alle OK)
  - Gesamt: 110 Tests (100% bestanden, 0 flaky, 0 rot)
  - Testlaufzeit: ~0.8s

## 2. Bau-Status
- `cargo check -p contextra-wire --no-default-features`: OK
- `cargo check -p contextra-wire`: OK
- `cargo clippy -p contextra-wire --all-targets -- -D warnings`: OK

## 3. Abhängigkeiten & Schichtung
- Kern-Closure (`cargo tree -p contextra-wire -e normal --no-default-features`): Leer (0 interne Abhängigkeiten).
- Externe Abhängigkeiten: `flatbuffers`, `bytes`, `serde`, `serde_json`, `thiserror`. Optional: `tracing`.
