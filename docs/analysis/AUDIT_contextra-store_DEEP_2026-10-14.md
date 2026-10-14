# AUDIT-Bericht: contextra-store (Ring 1) LSM, Compaction & Persistence DEEP AUDIT

**Spezialist:** S1 (Spezialist für LSM-Storage, Compaction und Tombstone-Semantik)
**Crate:** `contextra-store`
**Datum:** 2026-10-14
**VERIFIED-BY-SESSION:** S1-PASS-20261014

---

## 1. Scope und Modulabdeckung

Scope: `crates/contextra-store` (Ring 1):
- `lsm/` (`engine.rs`, `ops/`, `recovery.rs`, `flush.rs`, `commit.rs`, `group_commit.rs`, `fence.rs`, `purge/`)
- `sstable/` (`builder.rs`, `reader.rs`, `block_search.rs`, `block_cache.rs`, `bloom.rs`, `io.rs`, `stream.rs`, `orphan_gc.rs`)
- `memtable.rs`
- `compaction/` (`engine.rs`, `adaptive.rs`, `config.rs`, `retention.rs`, `purge_filter.rs`, `merge_operator.rs`)
- `manifest/` (`core.rs`, `entry.rs`, `recovery.rs`, `rollover.rs`)
- `erasure_ledger/` (`mod.rs`, `entry.rs`, `replay_filter.rs`, `coverage.rs`)
- `kv/` (`segment.rs`, `delete_mode.rs`)
- `util.rs`

---

## 2. Inventar der pub-Items und Verdrahtungsstatus

| Modul | Symbol / Typ | Sichtbarkeit | Verdrahtung / Status |
|---|---|---|---|
| `lsm` | `LsmStorage` | `pub struct` | Produktiver Einstiegspunkt (`contextra-engine`, `contextra-db`). Fully wired. |
| `lsm` | `LsmConfig` | `pub struct` | Konfigurations-Struktur für LSM Engine und Subsysteme. |
| `memtable` | `MemTable` | `pub struct` | In-Memory MVCC SkipList/BTreeMap Index. |
| `sstable` | `SstableBuilder` | `pub struct` | SSTable-Erzeugung (`flush.rs`, `compaction/engine.rs`). |
| `sstable` | `SstableReader` | `pub struct` | On-Disk SSTable Reader mit Block-Cache & Bloom-Filtern. |
| `sstable` | `BlockCache` | `pub struct` | Sharded Block Cache mit S3-FIFO / SIEVE Backend und Cache-Reports. |
| `compaction` | `CompactionEngine` | `pub struct` | Background / Ad-Hoc Compaction Engine. |
| `compaction` | `CostBasedAdaptivePlanner` | `pub struct` | **Verdrahtet im Default-Pfad** (`enable_adaptive_compaction = true`). |
| `compaction` | `CompactionConfig` | `pub struct` | Compaction-Parametersteuerung. |
| `compaction` | `PurgeReport` | `pub struct` | Purge-Report inkl. `obsolete_files` für sichere Entsorgung. |
| `manifest` | `Manifest` | `pub struct` | Transaktionales Append-Only Manifest (`MFMN`). |
| `erasure_ledger` | `ErasureLedgerHandle` | `pub struct` | Thread-sicherer Handle für ErasureLedger & WAL-Replay-Filter. |
| `kv` | `KvDeleteMode` | `pub enum` | Steuerung der KV-Shredding und Deletion-Modi. |

---

## 3. Beantwortung der 8 fachlichen Prüffragen

1. **Leserreihenfolge über Memtable, unveränderliche Memtables und Level-Dateien:**
   > **KORREKT.** Lookups in `get_at_seq` / `scan_prefix_at` prüfen zuerst die aktive `MemTable`, danach die `immutable_memtables` (neueste zuerst) und zuletzt die SSTable-Liste (neueste `max_seq` zuerst). Höhere Sequenznummern maskieren konsistent ältere Versionen.

2. **Snapshot-Retention und Version-GC:**
   > **KORREKT.** Die Compaction berechnet die untere Grenze `min_snapshot_seq = floor.floor()`. Alle Versionen mit $\text{raw\_seq} > \text{min\_snapshot\_seq}$ bleiben erhalten. Nur Versionen $\le \text{min\_snapshot\_seq}$ werden bis auf die jeweils neueste Version unterhalb der Schwelle konsolidiert.

3. **Tombstone-Semantik und Wiederauferstehungsschutz:**
   > **FORMAL VERIFIZIERT.** `SeqBitTombstone::is_tombstone(seq, flags)` (Bit 63 / Flag 0x01) wird durchgehend in `memtable.rs`, `sstable/reader.rs` und `compaction/engine.rs` angewendet. Tombstones werden **ausschließlich** während einer vollständigen Compaction (`is_full_compaction == true`) und nur dann gelöscht, wenn `raw_seq < min_snapshot_seq` gilt. Dadurch ist ein Wiederaufleben älterer Daten physisch ausgeschlossen.

4. **Fehlerunterscheidung im Lesepfad:**
   > **KORREKT.** I/O-Fehler, Prüfsummenfehler (`checksum_mismatch`), Entschlüsselungsfehler und Block-Korruptionen brechen mit `Err(ContextraError)` ab und werden direkt propagiert. Nicht vorhandene Schlüssel liefern `Ok(None)`.

5. **Atomare Manifest-Änderungen:**
   > **KORREKT.** Manifest-Rollover nutzt temporäre Dateien (`MANIFEST.new.<pid>.<cnt>`), `sync_all`, atomaren Rename und Parent-Directory `fsync`. Abbruch vor dem Rename belässt die bisherige Manifest-Datei intakt. Unvollständige Frames am EOF werden beim Neustart über Tail-Truncation verifiziert.

6. **Längen- und Prüfsummenvalidierung vor Dekodierung:**
   > **KORREKT.** Block-Trailer, Offset-Längen und Bloom-Längen werden vor Slice-Zugriffen validiert. Bloom-Filter und Index-Blöcke verifizieren CRC32 vor der Iteration; Datenblöcke verifizieren CRC32 und AES-GCM-SIV AEAD Nonces vor der Pufferdekodierung.

7. **Sequenznummern Über- und Unterlauf:**
   > **KORREKT.** `seq_no = u64::MAX` wird bei `WalEntry::try_new` explizit abgelehnt. Bit 63 ist reserviert für Tombstones. Alle Vergleiche maskieren Bit 63 via `seq & !TOMBSTONE_BIT`.

8. **TODO-Kommentare & Stubs:**
   > **ÜBERPRÜFT.** Keine verbleibenden Platzhalter-Returns in der produktiven Speicherlogik. Alle offenen Wiring-Hinweise beziehen sich auf die äußere Facaden-Anbindung.

---

## 4. Kostenmodelle & Adaptive Compaction

### Amplification Cost Models
- **Write Amplification (WA):** $WA = \frac{\text{Bytes physisch geschrieben}}{\text{Bytes logisch geschrieben}}$. Bei Standard-STCS gilt $WA \approx O(L \cdot T)$ mit Tier-Verhältnis $T=4.0$. Unter `CompactionStrategy::WriteOptimizedSTCS` wird WA minimiert.
- **Read Amplification (RA):** $RA = \text{SSTables per Lookup}$. Ohne Compaction wächst $RA = O(N_{\text{sst}})$. Bei Lese-dominanten Workloads schaltet `CostBasedAdaptivePlanner` auf `CompactionStrategy::ReadOptimizedAggressive` um und verschmilzt Segmente zu $1$ SSTable ($RA \to 1$).
- **Space Amplification (SA):** $SA = \frac{\text{Gesamtdiskplatz}}{\text{Nutzdatenplatz}}$. Compaction entfernt überschriebene/obsolete Versionen und Tombstones unterhalb der Snapshot-Grenze und reduziert $SA \to 1.0$.

### Verdrahtungsnachweis
In `CompactionConfig::default()` ist `enable_adaptive_compaction: true` gesetzt. Der Test `cost_based_adaptive_planner_production.rs` weist nach, dass Lese-Operationen über `LsmStorage::get` die `WorkloadMetrics` erhöhen, woraufhin `CostBasedAdaptivePlanner` im Produktionspfad `ReadOptimizedAggressive` wählt und 4 SSTables atomar zu 1 SSTable kompaktiert.

---

## 5. Invarianten-Nachweis

- **P2 (Zero-Panic):** Kein `.unwrap()` / `.expect()` in produktiven I/O- oder Retention-Pfaden. All I/O errors pass through `?`.
- **P28 (Determinismus):** Compaction-Schwellenwerte und Rate-Limiting basieren auf `seqno`, Operationszählern und dem injizierten `Clock`-Port (`self.clock.monotonic_nanos()`), nicht auf Wall-Clock `Instant`.
- **INV-WAL-TRUNCATION-1:** High-Water-Mark HMAC im Manifest verhindert unbemerktes WAL-Kürzen.
- **Snapshot Isolation:** Lookups lesen konsistent gegen `last_committed_tx` und `snapshot_seq`.

---

## 6. Testergebnisse

```bash
cargo test -p contextra-store --locked
```
**Status:** ALL TESTS PASSING (Unit, Integration, MVCC, Recovery, Compaction, BlockCache & ErasureLedger).
