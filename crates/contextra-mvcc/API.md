# API-Dokumentation: `contextra-mvcc`

Diese Datei beschreibt die öffentliche Schnittstelle des `contextra-mvcc`-Crates.

## Übersicht der Module und Kern-Typen

### Root (`contextra_mvcc`)
Re-exportiert die Kernkomponenten und Domänentypen für Transaktionssteuerung und Snapshot-Isolation.

#### Typen & Re-Exports
- `ContextraError`, `Result`: Zentrales Fehlersystem (`thiserror`).
- `DocId`, `TenantId`, `TxId`, `TOMBSTONE_BIT`: Identifikatoren und Maskierungs-Bits.
- `SeqBitTombstone`, `TombstoneSemanticsCheck`: Hilfskonstrukte für Tombstone-Semantik.

### 1. `seq_log` — Sequenz-Log und Versionierung
- `SequenceLog`: In-Memory-Verfolgung monotoner Versionen und Dokumentenlebenszyklen.
  - `new() -> Self`
  - `record_insert(&mut self, doc_id: DocId, seq: u64)`: Registriert Einfügung oder Update.
  - `record_delete(&mut self, doc_id: DocId, seq: u64)`: Markiert Dokument als gelöscht.
  - `is_visible(&self, doc_id: DocId, as_of: u64) -> bool`: O(1) Prüfung nach MVCC-Sichtbarkeitsgesetz.
  - `compact(&mut self, min_active_seq: u64) -> usize`: Bereinigt überschriebene Versionen unterhalb des aktiven Horizonts.
- `SeqLogEntry`: Einzelner Versionseintrag (`doc_id`, `insert_seq`, `delete_seq`).
  - `is_visible(&self, as_of: u64) -> bool`

### 2. `snapshot` — Snapshot-Registry & Pin-Management
- `SnapshotRegistry`: Thread-sichere Registrierung aktiver Schnappschüsse.
  - `new() -> Self`
  - `register(&self, seq_no: u64) -> SnapshotGuard`: Registriert Schnappschuss mit RAII-Guard.
  - `try_register(&self, seq_no: u64) -> Result<SnapshotGuard>`: Registrierung mit Kapazitätsprüfung.
  - `min_active_seqno(&self) -> u64`: Atomar ausgelesene Untergrenze aktiver Leser (`u64::MAX` falls keine aktiv).
  - `oldest_active_snapshot(&self) -> Option<u64>`: Untergrenze als `Option`.
- `SnapshotGuard`: RAII-Guard; de-registriert den Schnappschuss beim Verlassen des Gültigkeitsbereichs (`Drop`).

### 3. `tx_buffer` — Transaktions-Staging
- `TxBuffer<T>`: Sharded Staging-Puffer für uncommitted Transaktionen.
  - `new() -> Self`
  - `with_config(config: TxBufferConfig) -> Self`
  - `begin(&self, tx: TxId)`: Initialisiert Transaktions-Puffer.
  - `stage(&self, tx: TxId, op: IndexOp<T>) -> Result<()>`: Fügt Operation hinzu (begrenzt durch Ops & Byte-Budget).
  - `stage_kv(&self, tx: TxId, op: IndexOp<(Vec<u8>, Vec<u8>)>) -> Result<()>`: KV-spezifisches Staging.
  - `drain(&self, tx: TxId) -> Vec<IndexOp<T>>`: Entnimmt Operationen zur Commit-Ausführung.
  - `discard(&self, tx: TxId)`: Verwirft alle Operationen (Rollback).
  - `is_key_staged_for_tx(&self, tx_id: TxId, key: &[u8]) -> bool`: Read-your-own-writes Abfrage.
  - `min_read_snapshot(&self) -> Option<u64>`: Niedrigster aktiver Lese-Snapshot über alle offenen Transaktionen.
- `IndexOp<T>`: `Insert { doc_id, data }`, `Delete { doc_id, data }`.

### 4. `ssi` — Serializable Snapshot Isolation (SSI)
- `SequenceLogSsiValidator`: Validiert Transaktions-ReadSets gegen Commit-Historie.
  - `new() -> Self`
  - `validate(&self, tx_id: TxId, read_set: &ReadSet) -> Result<()>`: Prüft auf Write-Skew / rw-Konflikte.
  - `validate_and_record(&self, tx_id: TxId, read_set: &ReadSet, write_keys: impl IntoIterator<Item = &[u8]>, commit_seq: u64) -> Result<()>`: Atomare Validierung und Registrierung.
  - `record_commit_key(&self, key: &[u8], commit_seq: u64)`: Manuelle Commit-Registrierung.
  - `prune_through(&self, seq: u64)`: Pruning alter Commit-Historie.
- `ReadSet`: Verfolgt gelesene Schlüssel und Präfixe einer Transaktion.
  - `record_read(&mut self, key: impl Into<Vec<u8>>, snapshot_seq: u64)`
  - `record_range_prefix(&mut self, prefix: impl Into<Vec<u8>>, snapshot_seq: u64)`
  - `min_snapshot_seq(&self) -> Option<u64>`
