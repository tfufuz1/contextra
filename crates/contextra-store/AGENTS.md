# AGENTS.md — contextra-store
> Ring 1 · stable · Quelle: capabilities.toml · Spec: K.26 / III.3 / L.2

## 1. Zweck
Persistenzschicht des gesamten Systems auf Basis einer LSM-Tree-Speicher-Engine mit Write-Ahead-Log, MemTable, SSTables, transaktionalem Manifest und Hintergrund-Compaction.
Bietet Crash-Consistency, MVCC-Snapshot-Isolation und HMAC-abgesicherte WAL-Integrität.
Implementiert die Schnittstelle `StorageEngine` aus `contextra-core` für produktive Persistenz.

## 2. Modul-Karte

| Datei / Verzeichnis | Verantwortung |
|---|---|
| `lib.rs` | Crate-Einstiegspunkt mit `#![forbid(unsafe_code)]` und Re-Exports aller Kernstrukturen |
| `lsm/` | `LsmStorage`-Orchestrator, Transaktions-Commit/Recovery, Scan, Observer (`WalObserver`, STO-018) und Maintenance-Ops |
| `wal/` | Write-Ahead-Log: V3 HMAC-Chaining, CRC32-Verifikation, Flusher-Actor, Append-Only I/O und Open/Heal-Recovery |
| `memtable.rs` | In-Memory Skip-List mit Sequenznummern und Tombstone-Markierungen |
| `sstable/` | On-Disk SSTable-Segmente: Block-Builder/Reader, Bloom-Filter, Index, CRC32 und Block-Cache (`quick_cache` S3-FIFO) |
| `compaction/` | `CompactionEngine`: Tiered/Leveled Compaction, MVCC-Retention, Merge-Operatoren und `CostBasedAdaptivePlanner` (STO-021) |
| `manifest/` | Append-Only Manifest (`MFMN`), Rebuild valider/toter SSTables (L.2) und Schutz vor SSTable-Wiederauferstehung |
| `engine/` | StorageEngine-Implementierungen (z. B. `MemoryOnlyStorageEngine` für Tests/In-Memory-Betrieb) |
| `kv/` | KV-Segment-Persistenz und `KvDeleteMode`-Steuerung |
| `system_pressure.rs` | Dynamisches Memory- und Disk-Pressure-Monitoring für Throttling und Flushes |
| `tenant_codec.rs` | Multi-Tenant Key-Präfix-Codierung und Isolation |
| `util.rs` | Atomic-Rename und Schlüssel-Initialisierung (`load_or_create_integrity_key`) |

## 3. Invarianten

- **fsync Error Propagation (INV-DURABILITY-RING)**: JEDER `sync_all()` / `sync_data()`-Aufruf MUSS Fehler mit `?` propagieren.
  *Prüfbefehl*: `cargo xtask check-result-dropped-io`
- **last_committed_tx Single Load**: In `get_at_seq()` und `scan_prefix_at()` wird `last_committed_tx` genau einmal zu Beginn geladen.
  *Test*: `cargo test -p contextra-store --test mvcc_tests`
- **TOMBSTONE_BIT-Disziplin (ADR-041)**: Bit 63 (`seq & !TOMBSTONE_BIT`) MUSS vor allen Sequenznummern-Vergleichen maskiert werden.
  *Test*: `cargo test -p contextra-store --lib`
- **Flush-before-Visible (ADR-043)**: `last_committed_tx` MUSS vor dem Einfügen neuer SSTables in die sichtbare Liste aktualisiert werden.
  *Test*: `cargo test -p contextra-store --lib`
- **Atomic Rename Pattern**: Temporäre Dateischreibung (`.tmp`), `fsync`, atomarer Rename auf Zielpfad und `fsync` des Parent-Directories.
  *Test*: `cargo test -p contextra-store --lib`
- **WAL Tail Truncation (INV-WAL-TRUNCATION-1)**: Manifest speichert High-Water-Mark HMAC zum Schutz gegen unerlaubtes WAL-Kürzen.
  *Test*: `cargo test -p contextra-store --test recovery_tests`
- **TTL-Compaction (INV-TTL-1)**: Compaction vergleicht TTL-Ablaufzeiten gegen Schnappschuss-Grenzen, berechnet jedoch niemals Ablaufzeiten neu.
  *Test*: `cargo test -p contextra-store --lib`
- **SSTable-Wiederauferstehungsschutz (L.2)**: Recovery prüft gefundene `.sst`-Dateien gegen die Manifest-Menge `dead_set` und verhindert Re-Import gelöschter/ersetzter Daten.
  *Test*: `cargo test -p contextra-store --test recovery_tests`

## 4. Verboten / Anti-Patterns

- **Ungeprüftes Ignorieren von I/O-Ergebnissen**: `let _ = file.sync_all()` ist verboten, da Durability-Garantien verschluckt werden.
  *Richtig*: `file.sync_all().await.map_err(|e| ContextraError::Storage(format!("fsync: {e}")))?;`
- **Synchrone Disk-I/O auf Tokio-Akteuren**: `std::fs::File` darf im async-Pfad NUR innerhalb von `tokio::task::spawn_blocking` verwendet werden.
- **Direkter Import von `contextra_store::lsm::guard` von extern**: Snapshot-Pinning im Store ist `pub(crate)`.
  *Richtig*: Nutze `contextra_checkpoint::CheckpointGuard` für die öffentliche Checkpoint-API.
- **Falsche Crate-Bezeichnung**: Das Krypto-Crate heißt `contextra-crypto` (Cargo Package Name `contextra-crypto`).
- **Loom-Tests ohne cfg-Guard**: Concurrency-Tests unter Loom erfordern `#![allow(unexpected_cfgs)]` und Entkopplung im Nicht-Loom-Build.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- **Sperrenreihenfolge (Ring 1)**:
  1. `write_lock` (`tokio::sync::Mutex`) — WAL & MemTable Atomizität
  2. `memtable` (`Arc<parking_lot::RwLock>`) — In-Memory Skip-List
  3. `sstables` (`Arc<parking_lot::RwLock>`) — SSTable-Segmentliste
  4. `snapshot_registry` (`parking_lot::Mutex`) — MVCC Snapshot Pins
- **Async-Regel**: Halte KEINE `parking_lot`-Guards über `.await`-Grenzen.
- **Thread-Safety**: WAL-Flusher und Compaction-Worker laufen als abgetrennte Tasks/Threads und kommunizieren über entkoppelte Kanäle/MPSC.

## 6. Verifikation

```bash
cargo test -p contextra-store --locked
cargo xtask check-result-dropped-io
```

## 7. Bekannte Lücken / SOLL

- **WASM Merge-Operatoren im Compaction-Pfad (C.4.3.2)**: Trait `MergeOperator` ist vorhanden, WASM-basierte Ausführung während Compaction-Merge ist als Erweiterung vorbereitet (🟡 TEIL).
- **SSTable-Rang-Feld (S-01)**: `rank` in Manifest-Einträgen ist seit der Umstellung auf kontinuierliche Compaction deprecated.
