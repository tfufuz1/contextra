# FINDINGS STORE-MOD-B

Dieser Bericht dokumentiert Beobachtungen, Anomalien und Baseline-Defekte außerhalb des direkten Eigentums von Agent B.

## 1. Baseline-Defekt: `Cargo.toml` Abhängigkeiten vs. Features
- **Datei:** `crates/contextra-store/Cargo.toml`
- **Beobachtung:** Das Feature `workspace = ["dep:contextra-types", "dep:contextra-ports"]` erfordert, dass die entsprechenden Abhängigkeiten optional sind (`optional = true`). Da `contextra-types` und `contextra-ports` non-optional waren, schlug `cargo check` und `cargo metadata` fehl.
- **Lösungsvorschlag:** `contextra-types = { workspace = true, optional = true }` und `contextra-ports = { workspace = true, optional = true }` beibehalten.

## 2. Baseline-Defekt: Fehlende Module in `src/lib.rs`
- **Datei:** `crates/contextra-store/src/lib.rs`
- **Beobachtung:** Module `budget` und `stats` existieren physisch auf Platte, wurden aber in `lib.rs` nicht deklariert. Dies führt zu `could not find budget/stats in the crate root` in Compaction und LSM.
- **Lösungsvorschlag:** Standardisierter Fix `WP-B.0a` (`pub mod budget; pub mod stats;`) unmittelbar nach `pub use error::{StoreError, StoreResult};`.

## 3. Golden Corpus WAL Integrity Key Länge
- **Datei:** `crates/contextra-store/tests/golden/.wal_integrity_key`
- **Beobachtung:** Die Datei hat eine Länge von 59 Bytes. `Wal::open` validiert HMAC-Schlüssel strikt auf 32 Bytes Länge (`Storage("WAL integrity key has unexpected length: 59 (expected 32)")`). Dies blockiert `tests/golden_corpus.rs` im WAL-Teil, während der SSTable-Teil fehlerfrei durchläuft.
- **Lösungsvorschlag:** Agent A / Agent I sollte die Korpus-Datei `.wal_integrity_key` mit einem 32-Byte Key regenerieren und `MANIFEST.sha256` anpassen.

## 4. Feature `sieve-cache`
- **Datei:** `crates/contextra-store/Cargo.toml`
- **Beobachtung:** Das Feature `sieve-cache` ist in `Cargo.toml` deklariert, wird jedoch im gesamten Produktionscode von `contextra-store` nicht über `#[cfg(feature = "sieve-cache")]` abgefragt. Es existiert eine `SieveCacheBackend` Implementierung in `block_cache.rs`, die unabhängig vom Feature kompiliert wird.
- **Lösungsvorschlag:** Feature als Platzhalter belassen oder in Abstimmung mit Agent G bereinigen.

## 5. Merged Main Defekt: Fehlende Methoden in `Wal` nach STORE-MOD-A
- **Dateien:** `crates/contextra-store/src/wal/` vs `crates/contextra-store/src/lsm/`
- **Beobachtung:** Nach Merge von `origin/main` (`c245110b6` / PR #6035 STORE-MOD-A) fehlen auf `wal::core::Wal` die Methoden `append_batch_locked` (referenziert in `lsm/group_commit.rs:24` und `lsm/ops/write.rs:423`) sowie `append_batch` in Tests, und in `wal/io/append.rs:83` fehlt eine Typannotation bei `Arc::clone(km)`.
- **Zuständigkeit:** Agent A (WAL-Subsystem) und Agent E (LSM-Schreibpfad). Die SSTable-Komponenten (Agent B) sind hiervon unabhängig; alle SSTable-spezifischen Integrations- und Differenztests sind nachweislich grün.
