# FINDINGS: Befunde außerhalb des primären Auftrags D

## 1. Cargo.toml Defekt in `crates/contextra-store`
- **Datei**: `crates/contextra-store/Cargo.toml:17`
- **Beobachtung**:
  ```toml
  workspace = ["dep:contextra-types", "dep:contextra-ports"]
  ```
  steht in Konflikt mit den nicht-optionalen Abhängigkeiten:
  ```toml
  contextra-types = { workspace = true }
  contextra-ports = { workspace = true }
  ```
- **Auswirkung**: Cargo verweigert das Parsen des Manifests mit:
  `feature "workspace" includes "dep:contextra-types", but "contextra-types" is not an optional dependency`
- **Reproduktion**: `cargo check -p contextra-store`
- **Lösungsvorschlag**: In `Cargo.toml` die Dependencies als `optional = true` deklarieren, wie in commit `15ef71077`.

## 2. Fehlende ContextraError-Varianten in Tests fremder Module
- **Dateien**: `src/lsm/tests/commit_tests.rs`, `src/sstable/tests.rs`
- **Beobachtung**: Tests versuchen `ContextraError::InvalidInput` und `ContextraError::ChecksumMismatch` abzugleichen, während `StoreError` verwendet wird.
- **Zuständigkeit**: Agent B (SSTable) und Agent E (LSM-Schreibpfad).
