# Offene Punkte und Fremdefehler (P13)

## Fremdefehler (außerhalb BESITZ)

### 1. `contextra-crypto` (lib) Kompilierungsfehler
- **Betroffene Pfade:** `crates/contextra-crypto/src/secret.rs`, `src/kv_shredding.rs`
- **Fehler:**
  - `unexpected cfg condition value: serde` in `src/secret.rs:94` und `src/secret.rs:105`.
  - Veralteter Typ `contextra_types::error::HnswDeletionError::VerificationFailed`, der in `contextra_types` nicht mehr existiert.
- **Auswirkung:** Blockiert Kompilierung von `contextra-db` Tests und Binaries, da `contextra-db` über `contextra-engine` transitiv an `contextra-crypto` hängt.
- **Maßnahme:** Nicht in P13 anrühren (Regel 5); wird in Paket P12 bzw. Upstream-Crypto behoben.

### 2. `contextra-vector` (lib) Kompilierungsfehler
- **Betroffene Pfade:** `crates/contextra-vector/src/hnsw/types.rs:334`
- **Fehler:**
  - `no function or associated item named try_new_with_rng_and_kernel found for struct GenericHnswIndex`.
- **Auswirkung:** Blockiert Kompilierung von `contextra-engine` und Dev-Dependencies von `contextra-db`.
- **Maßnahme:** Nicht in P13 anrühren (Regel 5); wird im zuständigen Vector-Paket behoben.

### 3. `contextra-graph` (lib) Fehlende Ports/Types Bindeglieder
- **Betroffene Pfade:** `crates/contextra-graph/src/csr/...`
- **Fehler:**
  - `use of unresolved module or unlinked crate contextra_types` und `contextra_ports` in diversen Modulen ohne gesetzte Compat-Features.
- **Auswirkung:** Blockiert transitive Kompilierung bei `--all-targets`.
- **Maßnahme:** Nicht in P13 anrühren (Regel 5).
