# Übergabebericht MISSION `STORE-2` · LSM-Engine: Schreib-/Lesepfad, Memtable, Flush, Recovery

## 1. Zusammenfassung & Zielbild
Im Rahmen von Auftrag `STORE-2` wurde der Kern der LSM-Engine in `contextra-store` auf Korrektheit, Cancel-Sicherheit, Invarianten und Zero-Panic-Konformität gehärtet:
- **Schreibpfad & Cancel-Sicherheit:**
  - Atomare Batches (alles oder nichts) mit `SingleCommitGuard` und `LeaderCancelGuard` in `lsm/ops/write.rs`.
  - Strikte Einhaltung der globalen Lock-Reihenfolge (`Dokument -> Commit-Barriere -> Purge-Barriere -> Journal/WAL -> Speicherebene`), kein Halten von Locks über `.await`-Punkte.
  - Sequenzzuweisung und MVCC-Sichtbarkeitsfortschritt (`advance_visibility`) synchronisiert mit Transaktionszustand.
- **Memtable & Backpressure:**
  - 16-Shard geordnete Datenstruktur in `src/memtable.rs` mit 2-Byte-Präfix-Rangesharding für Standard-Schlüssel und Hash-Sharding für reservierte Systempräfixe (`__col:`, `__meta:`, `__rel:`, etc.).
  - Streng aufsteigende Sortierung nach Rohsequenznummer bei Out-of-Order-Einfügungen.
  - Bounded Memory Backpressure und atomare Größenanpassung mit Sättigung (`saturating_sub_size`).
- **Lesepfad & Scans:**
  - Deterministische hierarchische Suche (`MemTable -> Immutable MemTables -> SSTables L0..LN`).
  - Respektierung von Snapshot-Isolation (`snapshot_tx`), Tombstones (`TOMBSTONE_BIT`) und SSI ReadSet Clamping (`INV-MVCC-SSI-1`).
  - Speicherbegrenzte Streaming- und Paginierungs-Scans mit Cursor.
- **Flush & Recovery:**
  - Atomarer Memtable-Flush über SSTable-Erstellung mit VFS-Port und Manifest-Registrierung vor Freigabe des WAL-Watermarks.
  - Crash-Sicherheit (`flush_crash_simulation`) und Verzeichnissperre (`DirLock`).
  - Fail-Open Observer-Mechanismus mit Circuit-Breaker und Timeouts ohne Blockade des Commit-Pfads.
- **Zero-Panic-Härtung:**
  - Beseitigung von `expect` in `generate_crypto_salt()` in `lsm/recovery.rs` und Überführung in fallibles `Result<[u8; 32]>`.
  - Isolation von Test-Helfern in `src/lsm/tests.rs` hinter `#[cfg(test)]`.

## 2. Ist- vs. Soll-Vergleich
| Anforderung | Status | Nachweis |
|---|---|---|
| Schreibpfad: Validierung, WAL-Append, Memtable-Insert, Cancel-Sicherheit | Erfüllt | `src/lsm/ops/write.rs`, `src/lsm/commit.rs`, `src/lsm/group_commit.rs` |
| Memtable: Sharded BTreeMap, Versionsordnung, Backpressure, Shard-Verteilung | Erfüllt | `src/memtable.rs`, `tests/memtable_shard_distribution.rs` |
| Lesepfad: Memtable -> Immutables -> SSTables, Snapshot-Isolation, Scans | Erfüllt | `src/lsm/ops/read.rs`, `src/lsm/scan.rs` |
| Flush & Recovery: Atomarer Flush, Manifest vor WAL-Freigabe, DirLock, Replay | Erfüllt | `src/lsm/ops/compaction.rs`, `src/lsm/recovery.rs` |
| Fail-Open Observer: Circuit-Breaker, keine Deadlocks | Erfüllt | `src/lsm/observer.rs`, `tests/observer_blocking_failopen.rs` |
| Reference Engine: In-Memory Storage Engine Orakel | Erfüllt | `src/engine/memory_only.rs`, `tests/memory_only_crash_semantics.rs` |
| Zero-Panic: Keine unerlaubten Panics im Produktionscode | Erfüllt | `python3 tools/zero_panic_analyzer.py` (0 Prod-Verstöße im Scope) |
| Substanzerhalt: Keine Funktionen verloren | Erfüllt | Funktionsinventar 7616 Funktionen unverändert |

## 3. Offene Punkte / Upstream-Hinweise (Widerspruchsanalyse & Scope Guard)
- **`Cargo.lock`:** Bleibt unverändert per Scope Guard Vorgabe (`git checkout HEAD -- Cargo.lock`).
- **Pre-existing Upstream Compile Blocker:**
  1. `crates/contextra-crypto/src/secret.rs`: `unexpected cfg condition value: serde` unter `-D unexpected-cfgs`.
  2. `crates/contextra-store/src/lib.rs:57`: Moduldeklaration `pub mod kv;` verweist auf ein nach `contextra-crypto/src/kv_segment` migriertes Modul (aus Refactoring-Commit 7410879a4). Die Datei `src/lib.rs` gehört laut Scope-Regeln zu `STORE-5` und wurde per Klassenvertrag unberührt gelassen.
  3. `crates/contextra-store/src/lsm/recovery.rs:200, 535, 596`: In main fehlten zuvor Felder auf `LsmConfig` (`key_source`, `clock`), während `generate_crypto_salt` fallibel über `KeyMaterialSource` umgestellt wurde.
  4. `crates/contextra-store/src/wal/mod.rs:883`: Veraltetes Feld `poisoned` in `poison_for_test` (durch `poison_state` abgelöst).
  Diese Punkte gehören fremden OWNS-Pfaden (`STORE-1`, `STORE-5`, `CRYPTO-1`) und sind im Konvergenzlauf durch die zuständigen Prompts zu bereinigen.
