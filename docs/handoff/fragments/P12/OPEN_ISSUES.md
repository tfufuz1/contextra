# Offene Punkte & Fremdfehler (Paket P12)

## Fremdfehler (außerhalb BESITZ)

### CI: Format Check (cargo fmt --check)
- **Fehler**: Workspace-weite Formatierungsfehler in `crates/contextra-store` und `crates/contextra-vector`.
- **Status**: Fremdfehler. `cargo fmt --check -p contextra` ist vollständig grün. Änderungen an fremden Crates sind durch Global Rule 2 verboten.

### CI: Coverage Gate / proof-tests / Compile Check (cargo check / cargo test)
- **Fehler 1**: `error[E0609]: no field 'kernel' on type 'HnswColdCore'` in `crates/contextra-vector/src/hnsw/core_rebuild.rs:97, 144, 156`.
- **Fehler 2**: `error[E0599]: no function or associated item named 'try_new_with_rng_and_kernel' found for struct 'GenericHnswIndex'` in `crates/contextra-vector/src/hnsw/types.rs:334`.
- **Fehler 3**: `error[E0592]: duplicate definitions with name 'now_unix_ms'` in `crates/contextra-crypto`.
- **Fehler 4**: `error: package(s) 'xtask' not found in workspace` / `manifest path 'xtask/Cargo.toml' does not exist`. Betrifft Workflows `check`, `FlatBuffers Schema Drift Gate`, `Capability Marker Drift Gate`, `Check Duplicate Symbols Cross File Gate`, `Ring & Safety Checks`.
- **Status**: Alle Fehler liegen in fremden Crates (`contextra-vector`, `contextra-crypto`, `xtask`). Die Fassade `contextra` kompiliert fehlerfrei (`cargo check -p contextra` grün, sobald Abhängigkeiten bauen).

### CI: Dependency Policy Gate / cargo deny check
- **Fehler**: Sicherheitswarnungen in transitive Abhängigkeiten fremder Crates:
  - `lopdf` (RUSTSEC-2024-0435 / RUSTSEC-2024-0436) - stack overflow in nested PDF
  - `lru` (RUSTSEC-2026-0001) - panic safety
  - `tokio` (RUSTSEC-2025-0009) - AsyncFd I/O safety
  - `zip` (RUSTSEC-2025-0010 / RUSTSEC-2025-0011 / RUSTSEC-2025-0012)
- **Status**: Fremdefehler in fremden Crates bzw. workspace-weiten Abhängigkeiten. `crates/contextra` verwendet keine dieser Crates direkt.

### CI: Production Unwrap Counter Gate
- **Fehler**: Gate scheitert, da workspace-weit 1333 `unwrap()` / `expect()` gezählt werden (Schwellenwert 1197).
- **Status**: Fremdefehler. In `crates/contextra/src/` befinden sich 0 `unwrap()` und 0 `expect()`.

### CI: Gate G1 (tools/gate_dependencies.py)
- **Fehler**: 40 Verstöße in fremden Crates (`contextra-agent`, `contextra-crypto`, `contextra-engine`, `contextra-vector` etc.).
- **Status**: Fremdfehler. Für `contextra` werden 0 Verstöße gemeldet.

## Unklares / [H]-Punkte
- Keine offenen Unklarheiten bezüglich der Fassade `contextra`.
