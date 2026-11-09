# Testabdeckung · contextra-engine (AP P10)

## Zielwert
- Ziel-Linienabdeckung laut Spezifikation: **≥ 65 %**

## Status & Messung
- Crate-weiter Messlauf mit `cargo llvm-cov` wird durch ungelöste Fremdkompilierfehler in externen Workspace-Crates blockiert (`contextra-crypto`, `contextra-vector`, `contextra-graph`).
- Vorhandene Test-Suite in `crates/contextra-engine/`:
  - 41 Integrationstestdateien unter `tests/`
  - Umfangreiche Modultests unter `src/collection/tests/`, `src/background_workers/tests.rs`, `src/chunker/tests.rs`
- Neu hinzugefügte Tests in AP P10:
  - `tests/e2e_hybrid_no_inference.rs`: End-to-End-Suche über alle Signale (Vektor + BM25 + Graph) mit RRF-Fusion, Deletion in allen 3 Indizes, Signal-Resilienz.
  - `tests/tx_second_step_rollback.rs`: Transaktionsabbruch im 2. Indexschritt mit kompensierendem Rollback (keine Teilschreibung sichtbar).
