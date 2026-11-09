# P12 Übergabe-Fragment: Logbuch

## Start-Check (vor erster Änderung)
- [V] Toolchain: rustc / cargo 1.89.0 geprüft.
- [V] Workspace-Status: HEAD ist `65e1d68a5` (restructure/main).
- [V] Vorhandene Fremdfehler außerhalb BESITZ:
  - `contextra-graph`: E0428 (duplicate definition `contract`), E0432 (`GraphIndexExt`), E0433 (`contextra_ports` / `contextra_types` fehlen in CSR/PPR).
  - `contextra-crypto`: E0658/E0599 (cfg serde unexpect, unlinked types).
  - `contextra-vector`: E0592 (`remove`, `rebuild_sync` duplicate definitions), E0599 (`zeroize`, `try_new_with_rng_and_kernel`), E0609 (missing field `kernel` on `HnswColdCore`).
  - Diese Fehler verhindern derzeit vollständiges `cargo check -p contextra` bzw. `cargo test -p contextra`, da `contextra` über `contextra-db` -> `contextra-engine` von diesen Crates transitiv abhängt.
  - Dokumentiert in `OPEN_ISSUES.md`.

## Inventar der Lizenzbezüge
- [V] `crates/contextra/Cargo.toml`:
  - `contextra-license = { workspace = true, optional = true }`
  - `license = ["dep:contextra-license"]`
- [V] `crates/contextra/src/builder.rs`:
  - `#[cfg(feature = "license")] use contextra_license::SignedLicenseGate;`
  - `contextra_ports::license::{FeatureRing, LicenseGate, OpenFastGate}`
  - Methode `with_license_gate(gate)` (wird zu deprecated No-op).
  - Methode `with_signed_license(...)` (wird entfernt).
- [V] `crates/contextra/src/boot.rs`:
  - `contextra_ports::license::{FeatureRing, LicenseGate}`
  - `license_gate.check_ring(requested_ring)` in Phase B8.
- [V] `crates/contextra/src/performance_profile.rs`:
  - `FeatureRing`, `LicenseError`, `LicenseGate`
  - `enforce_license` (wird deprecated No-op).
- [V] `crates/contextra/src/collection_profile.rs`:
  - `validate_with_license` (wird deprecated No-op).
- [V] `crates/contextra/tests/`:
  - `license_gate_enforcement.rs` -> nach `archive/tests-legacy/contextra/`
  - `license_gate_always_enforced.rs` -> nach `archive/tests-legacy/contextra/`
  - `deletion_chain_e2e.rs` -> nach `archive/tests-legacy/contextra/`
  - `wiring_matrix.rs` -> Test `open_without_gate_never_grants_sovereign` anpassen.

## Commit: WP-11 G1 Gate Hygiene (contextra-testkit Entfernung)
- **Dateien**: `crates/contextra/Cargo.toml`, `docs/handoff/fragments/P12/LOG.md`, `docs/handoff/fragments/P12/OPEN_ISSUES.md`
- **Prüfbefehl**: `python3 tools/gate_dependencies.py --json`
- **Ergebnis**: grün für `contextra` (Verletzung "[G1-VIOLATION] Klasse-C-Crate contextra hängt von contextra-testkit (dev) ab" vollständig beseitigt).
