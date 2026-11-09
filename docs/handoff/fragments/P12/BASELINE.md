# P12 Baseline (WP-100)

## Ausführung: `cargo test -p contextra`
- **Befehl**: `cargo test -p contextra`
- **Ergebnis**: Durch pre-existierende Fremdfehler in transitiven Workspace-Abhängigkeiten blockiert (`contextra-vector`, `contextra-graph`, `contextra-crypto`).
  - `contextra-graph`: E0428 (duplicate definition `contract`), E0432 (`GraphIndexExt`), E0433 (`contextra_ports` / `contextra_types` fehlen in CSR/PPR).
  - `contextra-crypto`: unexpected `cfg` condition value `serde`.
  - `contextra-vector`: E0592 (`remove`, `rebuild_sync` duplicate definitions), E0599 (`zeroize`, `try_new_with_rng_and_kernel`), E0609 (missing field `kernel` on `HnswColdCore`).
- **Status Crate-intern**:
  - `crates/contextra` selbst ist frei von Lizenz-Gates (`SignedLicenseGate` entfernt, `with_license_gate` deprecated no-op).
  - Archivierung der 3 Testdateien nach `archive/tests-legacy/contextra/` erfolgreich abgeschlossen.
  - Dokumentiert in `OPEN_ISSUES.md`.
