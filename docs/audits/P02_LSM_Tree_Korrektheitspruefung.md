# Review: P02 — LSM-Tree Korrektheitsprüfung

## 1. Zusammenfassung

Die tiefgehende Sicherheits- und Korrektheitsanalyse der LSM-Tree Storage-Engine (`crates/contextra-store`) offenbart eine robuste Grundarchitektur mit atomaren Manifest-Rollovers, sharded In-Memory-MemTables und MVCC-fähiger Isolation. Es wurden jedoch zwei kritische bzw. schwerwiegende Korrektheitsmängel identifiziert: Erstens ignoriert die Hintergrund-Kompaktion (`CompactionEngine`) aktive Lese-Snapshots in `TxBuffer`, da sie ausschließlich `SnapshotRegistry::min_active_seqno()` abfragt. Dadurch werden sichtbare Versionen und Tombstones während laufender Leseabfragen vorzeitig gelöscht (CRITICAL MVCC Isolation Violation). Zweitens erlaubt die Binärsuche in SSTable-Datenblöcken (`block_search.rs`) bei mehreren Versionen desselben Keys beliebige Trefferpunkte ohne Rücklaufgarantie zur neuesten Version (HIGH Stale Read). Ein dritter Befund betrifft die Asynchronität zwischen `DeletionProof`-Ausstellung und WAL-`fsync()` bei `drop_collection` (HIGH Durability Gap).

## 2. Geprüfte Dateien

- `crates/contextra-store/src/memtable.rs`: Sharded BTreeMap MemTable mit Range-/Hash-Sharding und MVCC-Versionierung.
- `crates/contextra-store/src/sstable/builder.rs`: SSTable- und BlockBuilder-Implementierung (Format v3/v4) mit Per-Block-Bloom-Filtern.
- `crates/contextra-store/src/sstable/reader.rs`: SSTableReader für Point-Lookups und Iterator-Erzeugung.
- `crates/contextra-store/src/sstable/block_search.rs`: Binärsuche innerhalb decodierter SSTable-Blöcke.
- `crates/contextra-store/src/lsm/engine.rs`: Hauptstruktur `LsmStorage` und Systemkoordination.
- `crates/contextra-store/src/lsm/guard.rs`: Thread-sicherer Zustand `LsmState` für aktive und immutable MemTables.
- `crates/contextra-store/src/lsm/ops/read.rs`: MVCC Point-Lookups (`get_at_seq_tracked`) und Range-/Prefix-Scans.
- `crates/contextra-store/src/lsm/ops/write.rs`: Commit-Pipeline, Group Commit und Transaction-Sequence-Nummern.
- `crates/contextra-store/src/lsm/ops/maintenance.rs`: Checkpoint-Pinning und Hilfsoperationen.
- `crates/contextra-store/src/compaction/engine.rs`: Size-Tiered & Adaptive Compaction Engine mit Multi-Way-Merge.
- `crates/contextra-store/src/compaction/retention.rs`: Tombstone-Retention-Regeln für MVCC-Snapshots.
- `crates/contextra-store/src/manifest/core.rs`: Append-only Manifest zum Nachverfolgen aktiver SSTables.
- `crates/contextra-store/src/manifest/rollover.rs`: Crash-sicherer atomarer Rollover des Manifests.
- `crates/contextra-store/src/kv/delete_mode.rs`: Steuerung von `TombstoneOnly` vs. `CryptoShred`.
- `crates/contextra-engine/src/contextra_impl/collections.rs`: `drop_collection` Implementierung und `DeletionProof`-Erzeugung.

## 3. Extrahierte Invarianten (Phase A)

- **I-1 (Internal Key & Multi-Version Ordering):** Einträge für denselben User-Key müssen im LSM-Tree strikt nach absteigender Sequenznummer (`raw_seq`) geordnet sein, sodass neuere Versionen ältere Versionen bei Abfragen verdecken.
- **I-2 (MVCC Snapshot Isolation & Non-Destructive GC):** Versionen und Tombstones mit `seq_no >= min_active_snapshot` (über alle aktiven Reader in `TxBuffer` und gepinnten Checkpoints in `SnapshotRegistry`) dürfen NIEMALS von der Kompaktion gelöscht werden.
- **I-3 (Tombstone Retention in Partial Compaction):** Tombstones dürfen nur während einer vollständigen Kompaktion (`is_full_compaction == true`) entfernt werden, sofern kein aktiver Snapshot sie referenziert UND keine älteren SSTables außerhalb der Kompaktionsrunde existieren.
- **I-4 (Atomic State Transition during Flush/Compaction):** Der Übergang von MemTable zu SSTable bzw. die Ersetzung alter SSTables durch kompaktiertes SSTable im Manifest und `LsmState` muss für Reader atomar sichtbar sein, ohne Daten zu verpassen oder zu duplizieren.
- **I-5 (DeletionProof Durability & Non-Lying Attestation):** Ein ausgestellter `DeletionProof` darf erst an den Aufrufer zurückgegeben werden, nachdem die zugehörigen Lösch-Tombstones dauerhaft auf Disk reihum mittels `fsync()` persistiert wurden.

## 4. Befunde

### [CRITICAL] F-01 — Kompaktion ignoriert aktive MVCC-Reader in TxBuffer und löscht sichtbare Daten

- **Ort:** `crates/contextra-store/src/compaction/engine.rs:207`
- **Invariante betroffen:** I-2
- **Beleg:**
```rust
let min_snapshot_seq = self.snapshot_registry.min_active_seqno();
let output_path = self.generate_sst_path(data_path)?;
self.merge_sstables_with_cancel(
    &input_ssts,
    &output_path,
    min_snapshot_seq,
    is_full_compaction,
    cancel_token,
)
.await?;
```
- **Angriffs-/Fehlerszenario:**
  1. Ein langlaufender Lese-Task (z. B. ein Graph-Traversal oder Vektor-Scan) startet bei Sequenznummer 100. Die Sequenznummer wird in `tx_buffer` über `register_read` registriert. Es findet kein Aufruf von `snapshot_registry.pin(100)` statt (da `SnapshotRegistry` nur für explizite Checkpoints genutzt wird).
  2. In der DB existiert ein Put bei Sequenznummer 50 und ein Tombstone bei Sequenznummer 90 für den Schlüssel `K`.
  3. Die Hintergrund-Kompaktion `CompactionEngine::maybe_compact` wird ausgelöst. Sie liest `min_snapshot_seq = self.snapshot_registry.min_active_seqno()`. Da keine Checkpoints gepinnt sind, ergibt dies `u64::MAX`.
  4. Die Kompaktion führt ein Full Merge durch. Da `raw_seq (90) < min_snapshot_seq (u64::MAX)`, stuft die Kompaktion den Tombstone und den darunter liegenden Put (seq 50) als veraltet ein und löscht beide (GC).
  5. Der aktive Reader versucht im weiteren Verlauf seiner Transaktion, den Schlüssel `K` bei Snapshot-Sequenz 80 zu lesen (wobei der Tombstone bei seq 90 für ihn unsichtbar sein sollte und der Put bei seq 50 sichtbar sein müsste).
  6. Die komprimierte SSTable enthält den Schlüssel `K` nicht mehr.
- **Auswirkung:** Stillschweigender Datenverlust und Verletzung der MVCC-Snapshot-Isolation für aktive Lese-Transaktionen während paralleler Hintergrund-Kompaktion.
- **Empfehlung:** `CompactionEngine` muss bei der Ermittlung der untersten aktiven Sequenznummer (`min_snapshot_seq`) das Minimum aus `snapshot_registry.min_active_seqno()` UND `tx_buffer.min_read_snapshot()` bilden.

---

### [HIGH] F-02 — Binärsuche in SSTable-Blöcken liefert beliebige Version bei Mehrfachversionen desselben Keys

- **Ort:** `crates/contextra-store/src/sstable/block_search.rs:18`
- **Invariante betroffen:** I-1
- **Beleg:**
```rust
pub fn search_block(block_bytes: &[u8], target_key: &[u8]) -> Option<usize> {
    let (entries, offsets) = parse_block_trailer(block_bytes)?;
    offsets
        .binary_search_by(|&offset| {
            let key = parse_entry_key(block_bytes, offset as usize)?;
            Some(key.cmp(target_key))
        })
        .ok()
}
```
- **Angriffs-/Fehlerszenario:**
  1. Zwei Versionen desselben User-Keys `K` werden im selben Flush-Zyklus in eine SSTable geschrieben: `(K, v1, seq=10)` gefolgt von `(K, v2, seq=20)`.
  2. Bei der Ausführung von `search_block` mit `target_key = K` nutzt Standard-`binary_search_by` einen nicht-deterministischen Bisektionspfad. Wenn beide Elemente den Schlüssel `K` besitzen, gibt `binary_search_by` den Index eines beliebigen Treffers zurück (z. B. Index von `seq=10`).
  3. `SstableReader::get_at` übernimmt diesen Index direkt ohne Zurückspulen zum ersten Vorkommen des Schlüssels im Block.
  4. Die Leseoperation wählt `seq=10` (`v1`) aus, obwohl `v2` (`seq=20`) vorhanden ist und zur Anfrage-Sequenz passt.
- **Auswirkung:** Lesen veralteter Daten (Stale Read) trotz korrekter Sequenznummer.
- **Empfehlung:** Bei einem erfolgreichen Treffer in `binary_search_by` muss die Suchlogik nach links bis zum ersten Auftreten von `target_key` im Offset-Array zurückspulen, um sicherzustellen, dass die Versionen im Block stets in absteigender Sequenzreihenfolge ausgewertet werden.

---

### [HIGH] F-03 — DeletionProof wird vor vollständigem WAL-fsync ausgestellt

- **Ort:** `crates/contextra-engine/src/contextra_impl/collections.rs:376-386`
- **Invariante betroffen:** I-5
- **Beleg:**
```rust
let layer_proofs = vec![LayerCleanupProof::new_after_verified_empty(
    DeletionLayer::LsmMemtable,
    remaining_col_data.len() + remaining_txt_data.len(),
)]
.into_iter()
.collect::<Result<Vec<_>>>()
.map_err(|e| {
    contextra_types::ContextraError::Internal(format!(
        "CRITICAL: Collection '{name}' was tombstoned and committed at tx {}, but DeletionProof generation failed: {e}. Data is permanently deleted.",
        tx
    ))
})?;
```
- **Angriffs-/Fehlerszenario:**
  1. Aufrufer führt `drop_collection` aus. `delete_prefix` schreibt Tombstones in den MemTable und in den WAL-Group-Commit-Puffer.
  2. Der Group-Commit-Leaderreihenfolge folgend werden die Writes im OS-Page-Cache abgelegt, aber das synchrone Disk-`fsync()` steht im konfigurierte Flusher-Intervall (z. B. 500 µs Window) noch aus.
  3. `drop_collection` erzeugt den kryptografischen `DeletionProof` und sendet ihn an den Aufrufer zurück.
  4. Unmittelbar nach der Antwort kommt es zu einem abrupten Stromausfall / Kernel-Crash, bevor das OS-Buffer-Flushing abgeschlossen ist.
  5. Beim Systemneustart wird die WAL bis zum letzten fsync-bestätigten Zustand replayed. Die Tombstones für die gelöschte Collection fehlen in der WAL.
  6. Die Collection und ihre Dokumente tauchen bei der Recovery wieder auf, obwohl der Client einen gültigen, kryptografisch signierten `DeletionProof` besitzt.
- **Auswirkung:** Falsches Testat der logischen Löschung unter Re-Emergenz gelöschter Daten nach Crash.
- **Empfehlung:** `drop_collection` muss vor Erzeugung und Rückgabe des `DeletionProof` ein explizites `storage.wal.sync_all()` ausführen und dessen erfolgreichen Abschluss abwarten.

---

### Kompaktions-Failure-Matrix

| Crash-Zeitpunkt | Zustand auf Disk bei Recovery | Verlorene Writes? | Duplizierte Daten? | Tombstone-Resurrection? |
| :--- | :--- | :--- | :--- | :--- |
| **1. Während Merge (SST.tmp schreiben)** | Teilweise `sst-compact-xxx.tmp` Datei existiert. Manifest unverändert. | Nein | Nein (tmp-Datei wird verworfen) | Nein |
| **2. Nach SST.tmp Sync, VOR Manifest-Batch-Append** | `sst-compact-xxx.sst` auf Disk. Manifest enthält nur alte SSTables. | Nein | Nein (neue SST wird bei Recovery als untracked ignoriert/bereinigt) | Nein |
| **3. Mitten im Manifest-Batch-Append (Torn Frame)** | Manifest hat unvollständigen Frame am Dateiende. | Nein | Nein (Manifest-Recovery führt Tail-Truncation durch) | Nein |
| **4. Nach Manifest-Batch-Append, VOR Deletion alter SSTables** | Manifest verweist auf neue SST. Alte SSTs noch auf Disk. | Nein | Nein (reconstruct_valid_sstables filtert alte SSTs aus) | Nein |
| **5. Nach Deletion alter SSTables, VOR Manifest-Rollover** | Neue SST aktiv, alte SSTs gelöscht. Manifest enthält alte Historie. | Nein | Nein | Nein |

## 5. Optimierungspotenzial (Phase D)

1. **Pre-Allocation im MemTable Scan:** In `memtable.rs` erzeugen `scan_prefix_into` und `scan_range_into` temporäre BTreeMap-Einträge. Eine initiale Kapazitätsreservierung oder Vermeidung von Heap-Allokationen bei der Schlüsselprüfung steigert den Durchsatz bei intensiven Scan-Mustern.
2. **Read-Ahead Buffer bei SSTable-Merge:** Die `CompactionEngine` liest SSTables blockweise über den `BlockCache`. Ein dedizierter Read-Ahead-Stream-Buffer (64 KB Sequential-Read) während des Merges reduziert OS-Syscall-Overhead bei Multi-Gigabyte-Kompaktionen.
3. **Zero-Copy Key References im Compaction Heap:** In `compaction/engine.rs` klont `HeapItem` bei jedem Heap-Schritt `item.key` (`Bytes::clone`). Durch Nutzung leichtgewichtiger Slices oder Referenzen entfallen atomare Reference-Count-Inkremente bei Millionen Merge-Iterationen.

## 6. Offene Fragen / nicht verifizierbar ohne Laufzeit-Tests

- **SSTable v3/v4 Stream Multi-Version Interleaving:** Es sollte mittels Property-basierten Fuzz-Tests überprüft werden, ob extrem verschachtelte Lese- und Kompaktionsmuster bei gemischten Format-Versionen (v3 Bloom-Filter vs. v4 Adaptive Bloom-Filter) in Grenzfällen zu Abweichungen führen.

## 7. Jules-Task-Karten

```yaml
id: JULES-P02-01
title: Include active read snapshots from TxBuffer in CompactionEngine min_snapshot_seq calculation
severity: CRITICAL
files_to_touch:
  - crates/contextra-store/src/compaction/engine.rs
  - crates/contextra-store/src/lsm/engine.rs
context: >
  CompactionEngine computes min_snapshot_seq strictly from SnapshotRegistry::min_active_seqno(),
  ignoring long-running active MVCC read transactions registered in TxBuffer.
  This allows background compaction to purge tombstones and older visible versions
  under active read queries, violating MVCC Snapshot Isolation.
acceptance_criteria:
  - CompactionEngine resolves min_snapshot_seq as the minimum of SnapshotRegistry::min_active_seqno() and TxBuffer::min_read_snapshot().
  - Active read transactions in TxBuffer protect historical versions and tombstones from premature GC during compaction.
test_to_add: >
  Integration test in compaction/tests/advanced.rs where a long-running TxBuffer read query at seq N
  executes concurrently with full compaction, verifying no data visible to seq N is purged.
non_goals: >
  Do not alter SnapshotRegistry API or change check-pointing logic outside of min_snapshot_seq calculation.
```

```yaml
id: JULES-P02-02
title: Enforce left-rewind binary search for duplicate keys in SSTable block search
severity: HIGH
files_to_touch:
  - crates/contextra-store/src/sstable/block_search.rs
  - crates/contextra-store/src/sstable/reader.rs
context: >
  search_block uses slice::binary_search_by which lands on an arbitrary match when multiple
  versions of the same user key exist in an SSTable block.
  Point-lookups can return older versions instead of the latest visible version.
acceptance_criteria:
  - Successful binary search match rewinds to the first occurrence of the target user key in the block offset array.
  - SstableReader::get_at evaluates block entries in descending sequence order.
test_to_add: >
  Unit test in sstable/block_search.rs with multiple versions of the same user key in one block,
  asserting search_block always returns the offset of the first (newest) version.
non_goals: >
  Do not change the SSTable on-disk block format v3 or v4 layout.
```

```yaml
id: JULES-P02-03
title: Force WAL fsync before issuing collection-scoped DeletionProof in drop_collection
severity: HIGH
files_to_touch:
  - crates/contextra-engine/src/contextra_impl/collections.rs
context: >
  drop_collection issues a DeletionProof immediately after writing tombstones to MemTable and WAL queue.
  If a crash occurs before the background flusher executes fsync(), the tombstones are lost on recovery,
  causing deleted collection data to reappear despite a valid DeletionProof being issued.
acceptance_criteria:
  - drop_collection explicitly calls and awaits WAL sync_all prior to generating the DeletionProof.
  - Crash recovery after receiving a DeletionProof strictly guarantees collection tombstones persist on disk.
test_to_add: >
  Integration test in tests/deletion_proof_integration.rs simulating a crash/restart immediately post-drop
  and verifying zero collection keys reappear upon recovery.
non_goals: >
  Do not modify DeletionProof cryptography or signature structures in contextra-crypto.
```
