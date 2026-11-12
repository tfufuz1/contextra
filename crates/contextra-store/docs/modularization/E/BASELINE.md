# BASELINE — contextra-store (Modul LSM-Schreibpfad / Gruppe E)

## 1. Ausgangszustand
- Commit-Basis: `922ee269471eb76eb4be9370c0eb0e03b0e2a3ea` (origin/main)
- Toolchain: `rustc 1.89.0`
- Feature-Satz FS: `test-hooks,fault-injection,legacy-wal-key,memory-only-storage,workspace`

## 2. Baseline-Befunde vor Arbeitsbeginn
- **`Cargo.toml` & `src/lib.rs`**: In `Cargo.toml` deklarierte `features.workspace` die Pseudo-Dependencies `dep:contextra-types`, obwohl diese nicht-optional waren. In `src/lib.rs` fehlten `pub mod budget;` und `pub mod stats;`. Dies wurde als Baseline-Reparatur vorbereitet.
- **`io_seam.rs`**: `Vfs::read` verlor beim Einbau der Default-Methoden die Umsetzung von `ErrorKind::NotFound` auf `StoreError::NotFound(..)`. Dies wurde als Übergabe an Agent G dokumentiert und minimal repariert.
- **`golden_corpus.rs`**: `tests/golden/.wal_integrity_key` hat 59 Bytes statt 32 Bytes; `golden_corpus` scheiterte bereits in der Baseline. Als FINDING dokumentiert.
- **`tests/commit_cancel_chain_consistency.rs` & `tests/commit_paths_equivalence.rs`**: Reopen nach Drop scheiterte an `DirLock` ("Datenverzeichnis bereits in Benutzung"), da `storage.close().await` vor Drop fehlte. Beide Tests wurden in eigenem Besitz repariert und sind nun grün.
