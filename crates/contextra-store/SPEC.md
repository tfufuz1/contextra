# SPEC — `contextra-store`

**Normativitätsklasse:** Klasse B (Storage Bundle)
**Ring:** Ring 1
**Status:** In Entwicklung (Welle 3 / Paket B8 / M11)
**Normative Referenz:** `docs/SPEC.md` v4.1 (§0, §2, §3, §4, §5, §14.1, §15)

---

## 1. Zweck & Systemgrenzen
`contextra-store` implementiert das persistente Key-Value-Speichertriebwerk der Contextra-Architektur auf Basis eines Log-Structured Merge-Trees (LSM). Das Speichertriebwerk bietet atomare Schreibtransaktionen, Snapshot-Isolation über Sequenznummern, Append-Only Write-Ahead-Logging (WAL) mit HMAC-Kette und mehrstufige Hintergrundkompaktierung.

- **Klasse B (Storage Bundle):** `contextra-store` ist Teil des zusammenhängenden Speicher-Bundles (`types`, `ports`, `mvcc`, `crypto`, `sys`, `store`) und wird als Bundle exportiert/eingebunden.
- **Ring 1:** Abhängigkeiten zeigen ausschließlich auf gleiche oder niedrigere Ringe (`contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-crypto`, `contextra-sys`).
- **Speichersicherheit:** `#![forbid(unsafe_code)]` ohne Ausnahme im gesamten Crate (`lib.rs:25`).

---

## 2. Modulkarte & Verantwortlichkeiten

| Modul / Pfad | Verantwortung & Schnittstellen | Ist / Soll |
|---|---|---|
| `wal/` (`mod.rs`, `encode.rs`, `flusher.rs`, `hmac.rs`, `io.rs`, `replay.rs`) | Append-Only Write-Ahead-Log, HMAC-SHA256-Kette, asynchroner Flusher-Actor, Open-Heal bei Torn Writes, Replay | **Ist:** Aktiv; `flusher` handhabt Disk-Sync; Tail-Truncation bei Torn Writes. |
| `lsm/` (`engine.rs`, `commit.rs`, `flush.rs`, `group_commit.rs`, `guard.rs`, `recovery.rs`, `scan.rs`, `ops/*`, `validate.rs`, `observer.rs`) | Hauptorchestrator `LsmStorage`, Commit-Pfade, Flush-Koordination, Group-Commit Leader/Follower, Transaktionslebenszyklus | **Ist:** Aktiv; orchestriert MemTable, WAL und SSTables; implementiert `StorageEngine`. |
| `memtable.rs` | Nebenläufige, geschardete In-Memory BTreeMap mit Sequenznummern und Tombstones | **Ist:** Aktiv; Lock-freies / geschardetes BTreeMap-Design mit binärer Suche nach Sequenznummern. |
| `sstable/` (`builder.rs`, `reader.rs`, `block_cache.rs`, `bloom.rs`, `stream.rs`, `block_search.rs`, `reader_ext.rs`, `io.rs`) | Sortierte Segmente auf Festplatte (Format V0–V4), Block-Cache (LRU / quick_cache), Bloom-Filter (Block- und Datei-Ebene) | **Ist:** Aktiv; V4-Format ist Standard; Trailer mit Index-Offset und Bloom-Filter. |
| `manifest/` (`core.rs`, `entry.rs`, `recovery.rs`, `rollover.rs`) | Transaktionales Manifest für SSTable-Generationen und Versionierung (Magic `MFMN`) | **Ist:** Aktiv; atomare Manifest-Fortschreibung mit `sync_all` und Verzeichnis-Sync. |
| `compaction/` (`engine.rs`, `adaptive.rs`, `config.rs`, `merge_operator.rs`, `retention.rs`) | Leveled und Adaptive Compaction-Engine, `MergeOperator`-Ausführung, Tombstone-Bereinigung, TTL-Prüfung | **Ist:** Aktiv; Bereinigung von überdeckten Versionen und abgelaufenen Schlüsseln. |
| `scanner/` (`mod.rs`, `memtable.rs`, `sstable.rs`, `wal.rs`, `purge.rs`) | Port-Implementierungen `LayerScanner` und `Purger` für Residual-Scans und bereichsweise Purge-Kompaktierung | **Ist:** Übergangsweise im Store; **Soll:** Auslagerung nach `contextra-erasure-adapters` (SPEC §6). |
| `fsync.rs` | Zentrale Flush- und Sync-Abstraktion (Datei- und Verzeichnis-Sync) | **Ist:** Aktiv; fail-closed Fehlerpropagation (INV-FSYNC-1). |
| `tenant_codec.rs` | Mandanten-Präfixierung (`TenantId`) und logische Schlüssel-Isolation | **Ist:** Aktiv; trennt Keyspace mandantenrein (INV-TENANT-1). |
| `system_pressure.rs` | Überwachung von Systemressourcen (Blocking Thread Pool, Warteschlangen) | **Ist:** Aktiv; meldet Speicherdruck an den Compaction-Planer. |
| `engine/` (`memory_only.rs`) | In-Memory-Double (`InMemoryStorageEngine`) für schnelle Tests ohne Dateisystem | **Ist:** Aktiv; implementiert `StorageEngine` rein im RAM. |
| `kv/` (`mod.rs`, `delete_mode.rs`, `segment.rs`) | Historische KV-Segment- und Löschmodus-Relikte | **Ist:** Altlast / Deprecated Debt DEBT-01; **Soll:** Vollständiger Rückbau. |

---

## 3. Datenpfad

Der Datenfluss für Schreib- und Leseoperationen verläuft deterministisch und strikt geschichtet:

```
[ Client / Transaktion ]
        │
        ▼ (put / delete mit TxId)
[ TxBuffer (contextra-mvcc) ]
        │
        ▼ (commit(tx_id))
[ LSM Commit Orchestrator (lsm/ops/write.rs / lsm/group_commit.rs) ]
        │
        ├─────────────────────────────────────────┐
        ▼ (1. WAL-Phase)                          ▼ (2. MemTable-Phase)
[ WAL Flusher Actor (wal/flusher.rs) ]    [ MemTable (memtable.rs) ]
   - HMAC-Kette berechnen (wal/hmac.rs)       - Sharded BTreeMap
   - Payload schreiben (wal/io.rs)            - Multi-Version Index
   - file.sync_all() (INV-FSYNC-1)            - Tombstone (Bit 63)
        │                                         │
        └─────────────────────────────────────────┘
                                │
                                ▼ (MemTable voll / Flush-Schwelle)
                  [ SSTable Builder (sstable/builder.rs) ]
                     - Data Blocks (4 KiB)
                     - Block-Level Bloom Filter
                     - SSTable Trailer & Index
                     - fsync(file) & fsync(parent_dir)
                                │
                                ▼ (Commit im Manifest)
                  [ Manifest (manifest/core.rs) ]
                     - ManifestEntry::AddSstable
                     - fsync(manifest)
                                │
                                ▼ (Hintergrund)
                  [ Compaction Engine (compaction/engine.rs) ]
                     - Leveled Merge / Adaptive Compaction
                     - MergeOperator
                     - Physische Entfernung überschatteter Versionen
```

---

## 4. Invariantentabelle

| INV-ID | Aussage | Durchsetzungsort (Code) | Testnachweis (`tests/`) |
|---|---|---|---|
| **INV-STORE-1** | **Durability & Fsync:** Dateisystem-Syncs dürfen nie ignoriert werden (`let _ =`). Nach `fsync`-Fehlern wird das Handle permanent vergiftet und der Fehler fail-closed propagiert. | `crates/contextra-store/src/fsync.rs:18–35`, `src/wal/flusher.rs:363, 465` | `tests/fsync_syscall_verification.rs`, `tests/wal_robustness.rs`, `tests/flush_durability.rs` |
| **INV-STORE-2** | **Zweiphasiger WAL-Flush & Open-Heal:** WAL schreibt erst Payload und aktualisiert danach HMAC. Torn Writes am Dateiende werden beim Replay erkannt und getrimmt. | `crates/contextra-store/src/wal/flusher.rs:340–390`, `src/wal/replay.rs:180–230` | `tests/wal_open_heal.rs`, `tests/wal_torn_write_fault_injection.rs`, `tests/wal_replay_parity.rs` |
| **INV-STORE-3** | **MemTable Out-of-Order Sequenzen:** Versionen pro Schlüssel sind streng nach `seq_no` sortiert (`binary_search_by_key`). Höhere Sequenzen überschatten ältere; vertauschte Schreibreihenfolgen korrumpieren nicht. | `crates/contextra-store/src/memtable.rs:130–170` | `tests/memtable_shard_distribution.rs`, `tests/campaign_fuzzregress_memtable_concurrent.rs` |
| **INV-STORE-4** | **Tombstone-Disziplin:** Löschmarkierungen setzen Bit 63 (`TOMBSTONE_BIT = 1 << 63`). Bei allen Sequenz- und Sichtbarkeitsvergleichen ist Bit 63 vorher zu maskieren (`seq & !TOMBSTONE_BIT`). | `crates/contextra-store/src/sstable/reader.rs:490–510`, `src/compaction/adaptive.rs:120–140` | `tests/tombstone_scan.rs`, `tests/adaptive_compaction_tombstone.rs` |
| **INV-STORE-5** | **Single-Load Snapshot Isolation:** `last_committed_tx` wird bei Leseoperationen (`get_at_seq`, `scan_prefix_at`) exakt einmal zu Beginn geladen. Flush-before-Visible erzwingt Persistenz vor Sichtbarkeit. | `crates/contextra-store/src/lsm/scan.rs:40–80`, `src/lsm/flush.rs:90–120` | `tests/ssi_write_skew_integration.rs`, `tests/ssi_unbounded_snapshot_seq.rs`, `tests/loom_commit_flush_visibility.rs` |
| **INV-TENANT-1** | **Mandanten-Isolation:** Jeder gespeicherte Schlüssel enthält ein typisiertes `TenantId`-Präfix. Mandantenübergreifende Scans oder Datenzugriffe sind auf Store-Ebene ausgeschlossen. | `crates/contextra-store/src/tenant_codec.rs:25–65` | `tests/tenant_scoped_storage.rs` |
| **INV-FORMAT-1** | **Header- & Magic-Validierung:** Alle persistenten Dateien besitzen deterministische Magic-Bytes und Versionsnummern. Unbekannte Versionen werden fail-closed abgewiesen. | `src/wal/encode.rs:57–65`, `src/sstable/builder.rs:470`, `src/manifest/core.rs:21` | `tests/sstable_format_v2_compat.rs`, `tests/sstable_format_v3_to_v4_compat.rs`, `tests/manifest_corruption.rs` |

---

## 5. Dateiformate mit Magic & Version

### 5.1 Write-Ahead-Log (WAL)
- **V2 Format (Legacy):** Magic `MFW2` (`[0x4D, 0x46, 0x57, 0x32]`), batch-verschlüsselt.
- **V3 Format (Standard):** Magic `MFW3` (`[0x4D, 0x46, 0x57, 0x33]`).
  - Frame-Struktur: `length: u32` (Little Endian) + `CRC32C: u32` + `HMAC-SHA256: [u8; 32]` + `Payload`.
  - HMAC-Kette: $H_N = \text{HMAC}(K, H_{N-1} \parallel \text{seq} \parallel \text{tx\_id} \parallel \text{payload})$ mit $H_0 = [0u8; 32]$.
  - Open-Heal: Bei unvollständigen Schreibvorgängen am Dateiende schneidet der Recovery-Prozess die Datei auf den letzten verifizierten Frame-Offset zurück (`wal/replay.rs:215`).

### 5.2 Sorted String Table (SSTable)
- **Header / Magic:**
  - Standard (Format V1–V4): Magic `MFSX` (`0x5853_464D` Little Endian im Trailer, `sstable/block_cache.rs:352`).
  - Legacy (Format V0): Magic `MFST` (`0x4D46_5354` Little Endian im Trailer).
- **Trailer-Struktur (Format V4):**
  - Befindet sich an den letzten 54 Bytes der Datei:
    - Bytes 0..8: `min_tx_id: u64`
    - Bytes 8..16: `max_tx_id: u64`
    - Bytes 16..24: `min_seq: u64`
    - Bytes 24..32: `max_seq: u64`
    - Bytes 32..40: `index_offset: u64`
    - Bytes 40..48: `bloom_offset: u64`
    - Bytes 48..50: `format_version: u16` (Wert: 4)
    - Bytes 50..54: `magic: u32` (`MFSX`)
- **Block-Format (4 KiB Blöcke):**
  - Sequenz von Einträgen `(key, val, seq, tx_id)` gefolgt von Block-Trailer mit Eintrag-Offsets und lokalem Block-Bloom-Filter.

### 5.3 Manifest
- **Magic:** `MFMN` (`b"MFMN"`, 4 Bytes, `manifest/core.rs:21`).
- **Header:** Magic + `version: u16` (Standard: 1).
- **Log-Einträge:** Längen-präfigierte binäre Records (`ManifestEntry::AddSstable`, `ManifestEntry::RemoveSstable`, `ManifestEntry::Rollover`).

---

## 6. Ports & Schnittstellen

### Implementierte Ports (`contextra-ports`)
- `StorageEngine`: Hauptvertrag für KV-Operationen (`get`, `put`, `delete`, `commit`, `rollback`, `scan_prefix`). Implementiert von `LsmStorage` (`lsm/engine.rs`) und `InMemoryStorageEngine` (`engine/memory_only.rs`).
- `Purger`: Schnittstelle zur physischen Bereinigung von gelöschten Scopes (`compact_range`). Implementiert in `scanner/purge.rs`.
- `LayerScanner`: Schnittstelle zur forensischen Residuenmessung. Implementiert in `scanner/mod.rs` (`MemtableScanner`, `SstableScanner`, `WalScanner`).
- `BlockCache`: Cache-Abstraktion für SSTable-Blöcke (`sstable/block_cache.rs`).
- `MergeOperator`: Schnittstelle für atomare Lese-Modifiziere-Schreibe-Zusammenführungen während Flush und Compaction (`compaction/merge_operator.rs`).

### Konsumierte Ports
- `Clock` (`contextra-ports::clock`): Deterministische Zeitquelle für TTL, Timeouts und Retention.
- `Vfs` (`contextra-ports::vfs`): Dateisystemabstraktion.
  - **Ist-Zustand:** Dateisystem-I/O nutzt derzeit `tokio::fs` für asynchrone Metadaten/Lifecycle und `std::fs` innerhalb von `spawn_blocking` für Block-I/O.
  - **Soll-Zustand (WP 4.4):** Vollständige Umstellung auf den `Vfs`-Port für autarke Dateisystem-Isolation.

---

## 7. Lock-Register & Poison-Matrix

### 7.1 Sperrenordnung (Lock Order)
Zur Vermeidung von Deadlocks gilt die strikte hierarchische Sperrenreihenfolge (P06 / ADR-0XX-P0A):

$$\text{commit\_mutex} \longrightarrow \text{wal.truncate\_lock} \longrightarrow \text{memtable\_shard\_lock} \longrightarrow \text{sstable\_manifest\_lock}$$

- **Lock-Handoff Invariante:** Im Group-Commit wird `truncate_lock` der WAL akquiriert, **bevor** `commit_mutex` freigegeben wird (`lsm/mod.rs` und ADR-0XX-P0A). Dies stellt sicher, dass die physische Schreibreihenfolge auf Platte exakt der logischen HMAC-Kettenreihenfolge entspricht.
- **Async/Sync-Trennung:** Niemals `parking_lot`-Guards über `.await`-Punkte halten (`clippy::await_holding_lock`).

### 7.2 Poison-Matrix & Fail-Closed-Verhalten (P01)

| Auslösendes Ereignis | Betroffene Komponente | Zustandsübergang | Auswirkung auf Folgetransaktionen | Wiederherstellungsmethode |
|---|---|---|---|---|
| `fsync()`-Fehler beim WAL-Append | `Wal` / Flusher Actor | Handle wird `poisoned = true` | Alle nachfolgenden Schreibvorgänge schlagen mit `ContextraError::Storage` fehl. | `wal.recover_from_poison().await` nach Tail-Truncation oder Neustart mit Open-Heal. |
| Torn Write / Teil-Frame am WAL-Ende | `Wal` / Replay | `Recovering` | Replay trimmt unvollständige Bytes bis zum letzten validen Frame. | Automatisch beim Replay; nach erfolgreichem Schnitt wieder `Healthy`. |
| Prüfsummen- oder HMAC-Fehler mitten in der WAL | `Wal` / Replay | `Corrupted` | Replay bricht fail-closed ab; Storage verweigert den Start. | Manueller Eingriff / Restore aus Snapshot. |
| Plattenüberlauf (`ENOSPC`) | LSM Storage | `ReadOnly` | Mutationen werden abgewiesen; Leseoperationen bleiben möglich. | Freigabe von Speicherplatz, anschließend `reclaim()`. |
| Mutex-Poisoning (Panic in Thread) | Synchronisations-Locks | `Poisoned` | Kein Panic; Kapselung und Rückgabe als `ContextraError::Storage`. | Neustart des Crate-Subsystems. |

---

## 8. Garantien und Nicht-Garantien

### Garantien (G)
- **G-1 (Crash-Konsistenz):** Ein erfolgreicher Transaktions-Commit (`commit(tx_id).await == Ok(())`) ist nach einem Prozessabsturz (`SIGKILL`) oder Stromausfall dauerhaft in der WAL persistiert.
- **G-2 (Snapshot-Isolation):** Transaktionsleser sehen ausschließlich Datenstände mit `commit_seq <= snapshot_seq`. Unvollständige oder laufende Transaktionen sind unsichtbar.
- **G-3 (Bit-Rot-Erkennung):** Alle Datenblöcke in SSTables und WAL-Frames sind durch CRC32C-Prüfsummen und HMAC-Hashes geschützt. Bit-Rot wird beim Lesen deterministisch erkannt und mit `ContextraError::Storage` gemeldet.
- **G-4 (Open-Heal bei Torn Writes):** Unvollständig geschriebene Bytes am Dateiende der WAL, die durch plötzlichen Stromausfall entstehen, werden beim Replay erkannt und ohne Datenverlust an bestätigten Transaktionen abgeschnitten.
- **G-5 (Verzeichnis-Sperre / DirLock):** Gleichzeitiges Öffnen desselben Datenverzeichnisses durch zwei Prozesse wird über eine exklusive Dateisperre (`LOCK`, 0600) abgewiesen (`util.rs:24`).

### Nicht-Garantien (Nicht-Ziele / Non-Guarantees)
- **Keine automatische Bit-Rot-Korrektur:** Bit-Rot wird zuverlässig erkannt, aber nicht repariert (keine Reed-Solomon- oder Paritätskorrektur im Triebwerk).
- **Keine Manipulationssicherheit bei lokalem Keyfile:** Wird `.wal_integrity_key` unverschlüsselt neben der WAL-Datei im Verzeichnis abgelegt, schützt HMAC vor Bit-Rot, jedoch **nicht** vor einem Angreifer mit Schreibrechten auf das Dateisystem (siehe SECURITY NOTE in `wal/hmac.rs`). Kryptographische Fälschungssicherheit erfordert die Injektion eines externen `KeyManager`.
- **Keine Windows-Verzeichnissynchronisation:** Auf Windows-Betriebssystemen meldet `fsync_dir` den Fehler `ContextraError::Io` mit `ErrorKind::Unsupported` (E20; `fsync.rs:10–29`), da Windows-Dateisysteme kein Verzeichnis-Fsync unterstützen.
- **Kein Schutz vor SSD-Wear-Leveling-Resten:** Das physische Überschreiben von Flash-Speicherblöcken auf Firmware-Ebene kann durch das Triebwerk nicht sichergestellt werden.
- **Keine Rechts- oder Löschsagas:** `contextra-store` führt keine Deletion-Proofs oder juristischen Löschketten; diese liegen ausschließlich in `contextra-erasure`.

---

## 9. Feature-Matrix

| Feature | Zweck | Standard | Abhängigkeiten |
|---|---|---|---|
| `default` | Minimaler Produktions-Build für LSM-Storage | Ja | keine |
| `inspect` | Stellt Schnappschuss-APIs (`sstables_snapshot`, `memtables_snapshot`) für Scanner bereit | Nein | keine |
| `block-cache-v2` | Aktiviert optimierten Block-Cache basierend auf `quick_cache` | Nein | `quick_cache` |
| `memory-only-storage` | Ermöglicht flüchtigen Betrieb rein im Arbeitsspeicher | Nein | keine |
| `docid-128` | Unterstützt 128-Bit-Dokument-IDs | Nein | `contextra-types/docid-128` |
| `legacy-wal-key` | Abwärtskompatibilität für Migration alter WAL-Integritätsschlüssel | Nein | keine |
| `fault-injection` | Ermöglicht simulierte I/O- und Crash-Injektionen in Tests | Nein | keine |
| `test-hooks` | Interne Hooks für deterministische Unit- und Integrationstests | Nein | keine |

---

## 10. Ist- und Soll-Zustand (Differenzanalyse)

| Bereich | Ist-Zustand (Code) | Soll-Zustand (SPEC v4.1) | Geplante Maßnahme |
|---|---|---|---|
| **Dateisystem-Port** | Direkte Datei-I/O über `tokio::fs` und `std::fs::File` in `spawn_blocking`. | Vollständige Abstraktion über `contextra-ports::vfs::Vfs` (M4). | WP 4.4 / D4 Umstellung. |
| **Port-Implementierung** | `LayerScanner` und `Purger` befinden sich direkt in `scanner/`. | Speicherkerne bieten nur Basis-APIs (`compact`, `reclaim`); Adapter liegen in `contextra-erasure-adapters`. | Welle 4 / WP 3.2. |
| **KV-Löschrelikte** | `src/kv/segment.rs` und `src/kv/delete_mode.rs` enthalten veraltete Löschmodus-Relikte. | Vollständige Neutralisierung ohne Shredding-Begriffe. | Deprecated Debt DEBT-01 Abbau. |
| **Intra-Doc-Links** | Wenige unaufgelöste Rustdoc-Referenzen auf `MetricsSink`, `SsiValidator`, `SstableBuilder`. | Fehlerfreier Bau unter `RUSTDOCFLAGS="-D warnings"`. | Behebung in Paket B8. |
