# FINDINGS — contextra-store (Modul LSM-Schreibpfad / Gruppe E)

## 1. Diskrepanz in Lock-Ordnung Doku
- **Datei:Zeile**: `src/lsm/mod.rs:35-45` vs `src/lsm/lock_order.rs:10-40` vs Crate-`AGENTS.md §6`.
- **Beobachtung**: `src/lsm/mod.rs` dokumentiert 5 Stufen (`commit_mutex` → `pending_commit_queue` → `state` → `wal` → `wal.truncate_lock`). `lock_order.rs` dokumentiert 10 Ränge (100 bis 950) inklusive Document Intent, Flush-Mutex, SSTables, MemTable-Shards, Manifest, Observer-Registry und Block-Cache.
- **Vorschlag**: In `lock_order.rs` wird die vollständige 10-Stufen-Hierarchie formal als Master-Referenz dokumentiert und mit Invariante `INV-LOCK-ORDER-1` hinterlegt.

## 2. Golden-Corpus `.wal_integrity_key` Längenfehler
- **Datei:Zeile**: `tests/golden/.wal_integrity_key`
- **Beobachtung**: Datei hat Länge 59 Bytes anstelle von 32 Bytes (in Commit 4ec7ae460 wurde ein Escaped-String statt roher 32 Bytes persistiert), weshalb `golden_corpus` in der Baseline mit `WAL integrity key has unexpected length: 59 (expected 32)` fehlschlägt.
- **Vorschlag**: An Agent G übergeben (Golden-Dateien sind für Agent E read-only).

## 3. Baseline-Fehler in `Cargo.toml` und `src/lib.rs`
- **Beobachtung**: `workspace = ["dep:contextra-types", "dep:contextra-ports"]` in `Cargo.toml` kollidierte mit nicht-optionalen Dependencies. In `src/lib.rs` fehlten `pub mod budget;` und `pub mod stats;`.

## 4. Pre-Existing Loom Build Failure
- **Datei:Zeile**: `src/wal/hmac.rs:283` und `src/compat/license.rs:3`
- **Beobachtung**: `src/lib.rs` blendet `mod lsm` unter `#[cfg(loom)]` aus (`#[cfg(not(loom))] pub mod lsm;`), aber `wal/hmac.rs` ruft `crate::lsm::recovery::next_temp_file_counter()` auf und `compat/license.rs` nutzt `crate::lsm::config`. Daher schlägt der Crate-Build unter `--cfg loom` bereits in der Baseline fehl.
- **Vorschlag**: An Agent A (WAL) und Agent H (compat) übergeben.
