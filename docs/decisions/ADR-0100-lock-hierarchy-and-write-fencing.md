# ADR-0100: Sperrenhierarchie, Klassifizierung „Lock über .await“ und Write-Fencing in contextra-store

- **Status**: Akzeptiert
- **Datum**: 2026-11-08
- **Kontext**: SPEC §3.5, SPEC §5.4 (MV-08), Invariante `INV-LOCK-ORDER-1`, WP-STO-5

## Kontext und Problemstellung
In LSM-Storage-Engines mit asynchronem I/O, MemTable-Rotation, Hintergrund-Kompaktierung und Group-Commit besteht bei undisziplinierter Sperren-Akquisition die Gefahr von Verklemmungen (Deadlocks) und Thread-Pool-Exhaustion (insbesondere bei Halten synchroner Sperren über `.await`).

Das Werkzeug `tools/concurrency_lock_auditor.py -c contextra-store` meldete 77 potenzielle Gefahrenstellen („Lock across .await hazards“). Ziel dieses ADRs ist:
1. Das formale Sperrenregister mit Rängen und Haltedauerschranken festzuschreiben.
2. Jede der 77 gemeldeten Stellen zu klassifizieren und nachzuweisen, dass keine unzulässigen synchronen Sperren über `.await` gehalten werden.
3. Die asynchronen `tokio::sync`-Guards auf ihre Haltedauer und Begründung festzulegen.

---

## 1. Formales Sperrenregister (Rangordnung)

Die hierarchische Lock-Akquisitionsreihenfolge folgt streng SPEC §5.4:
$$\text{Dokument-Lock (Intent)} \longrightarrow \text{Commit-Barriere} \longrightarrow \text{Flush-Barriere} \longrightarrow \text{Zustand} \longrightarrow \text{Journal/WAL} \longrightarrow \text{Speicherebene (SSTables)} \longrightarrow \text{MemTable-Shards}$$

Im Code abgebildet in `crates/contextra-store/src/lsm/lock_order.rs`:

| Rang | Konstante | Primitiv | Instanz | Max. Haltedauer | Begründung & Schutzbereich |
|---|---|---|---|---|---|
| 100 | `RANK_DOCUMENT_INTENT` | `std::sync::Mutex` | `LsmStorage::intent_locks` | < 10 µs | RAM-Key-Locks zur Konflikterkennung vor Commit. Nie über `.await`. |
| 200 | `RANK_COMMIT_MUTEX` | `tokio::sync::Mutex` | `LsmStorage::commit_mutex` | < 50 ms | Serialisiert SSI-Validierung und Commit-Leader-Auswahl. |
| 300 | `RANK_FLUSH_MUTEX` | `tokio::sync::Mutex` | `LsmStorage::flush_mutex` | < 500 ms | Serialisiert Flush-Vorgänge (Verhindert Race auf WAL-Rotation). |
| 400 | `RANK_STATE_RWLOCK` | `tokio::sync::RwLock` | `LsmStorage::state` | < 50 µs | Schützt Zeigertausch (MemTable / Immutable MemTables). |
| 500 | `RANK_WAL_TRUNCATE` | `tokio::sync::Mutex` | `Wal::truncate_lock` | < 10 ms | Schützt Truncate/Rollback im WAL gegen konkurrierende Appends. |
| 600 | `RANK_SSTABLES_RWLOCK` | `tokio::sync::RwLock` | `LsmStorage::sstables` | < 100 µs | Schützt aktive SSTable-Reader-Liste gegen Hinzufügen/Entfernen. |
| 700 | `RANK_MEMTABLE_SHARDS` | `parking_lot::RwLock` | `MemTable::shards` | < 5 µs | RAM-SkipList/BTreeMap-Mutationen. Strikt synchron, nie über `.await`. |
| 800 | `RANK_MANIFEST_MUTEX` | `tokio::sync::Mutex` | `Manifest::file` | < 20 ms | Serialisiert Append-Batch auf das Transaktionsmanifest. |
| 900 | `RANK_OBSERVER_REGISTRY` | `parking_lot::RwLock` | `ObserverRegistry::observers` | < 5 µs | Beobachter-Registrierung. Non-blocking Channels, nie über `.await`. |
| 950 | `RANK_BLOCK_CACHE` | `parking_lot::RwLock` | `SieveCacheBackend::state` | < 2 µs | In-Memory Block-Cache Hits/Eviction. Nie über `.await`. |

---

## 2. Klassifikation der 77 Auditor-Fundstellen

Der statische Auditor prüft heuristisch jedes Muster `let guard = ...lock()` / `.read()` / `.write()`, gefolgt von `.await`. Die Klassifizierung der 77 Fundstellen ergibt:

### A. Synchrone Sperren (`parking_lot` / `std::sync`): 0 echte Gefahrenstellen
- Alle 41 Vorkommen von `parking_lot::RwLock` (MemTable-Shards, Observer, BlockCache, MetricsSink) und alle 15 Vorkommen von `std::sync::Mutex` (`intent_locks`, Test-Locks) werden **ausschließlich in synchronen Funktionen oder isolierten Scopes** ohne darin befindliches `.await` verwendet.
- `clippy::await_holding_lock` bleibt aktiv auf `#![deny]`.

### B. Asynchrone Tokio-Sperren (`tokio::sync::Mutex` & `tokio::sync::RwLock`)
Die im Code vorhandenen Vorkommen teilen sich in 4 Kategorien:

1. **`truncate_lock` in `wal/io.rs` (Z. 636, 701, 845) und `lsm/ops/write.rs` (Z. 417, 734)**:
   - *Typ*: `tokio::sync::Mutex<()>`.
   - *Zweck*: Stellt sicher, dass während eines Rollbacks oder Truncates keine Appends erfolgen, und serialisiert Flusher-Kanal-Enqueue.
   - *Schranke*: Das Lock wird ausschließlich während des Enqueue-Aufrufs (`tx.send()`) gehalten (< 1 ms); bei `append_batch` wird die Bestätigung (`ack_rx.await`) **außerhalb** des Truncate-Guards abgewartet!
   - *Bewertung*: Gewollt, korrekt, deadlockfrei (Rang 500).

2. **`commit_mutex` in `lsm/ops/write.rs` (Z. 253, 690) und `lsm/ops/compaction.rs` (Z. 46)**:
   - *Typ*: `tokio::sync::Mutex<()>`.
   - *Zweck*: Serialisiert Commit-Vorbereitung und WAL-Rotation (Phase 2 von `flush()`).
   - *Schranke*: Im Group-Commit-Leader-Pfad wird der Lock während des WAL-Schreibens gehalten oder über Handoff an nachfolgende Threads übergeben (Loom-verifiziert). Bei Flush wird `commit_mutex` nur für den atomaren Memtable/WAL-Swap gehalten (< 50 µs) und VOR dem teuren I/O in Phase 3 wieder freigegeben.
   - *Bewertung*: Gewollt, Rang 200 > Rang 400 (`state`) > Rang 500 (`wal`).

3. **`state` in `lsm/ops/compaction.rs` (Z. 24, 150), `lsm/ops/read.rs` (Z. 90), `lsm/scan.rs` (Z. 98), `lsm/recovery.rs` (Z. 727)**:
   - *Typ*: `tokio::sync::RwLock<LsmState>`.
   - *Zweck*: Schützt die Referenzen auf die aktive MemTable und unveränderliche MemTables.
   - *Schranke*: Beim Lesen wird nur ein Snapshot der `Arc<MemTable>` geklont und der Read-Guard sofort fallen gelassen. Beim Schreiben (Flush/Recovery) wird der Lock nur für Zeigerwechsel gehalten (< 50 µs).
   - *Bewertung*: Gewollt, Rang 400.

4. **`sstables` in `lsm/ops/read.rs` (Z. 122), `lsm/scan.rs` (Z. 48, 99), `compaction/engine.rs` (Z. 139, 335, 824)**:
   - *Typ*: `tokio::sync::RwLock<Vec<Arc<SstableReader>>>`.
   - *Zweck*: Schützt die Liste geöffneter SSTables.
   - *Schranke*: Read-Guard wird genommen, Reader-Arcs werden geklont/referenziert, Guard wird unverzüglich freigegeben.
   - *Bewertung*: Gewollt, Rang 600.

5. **Test- und Diagnostik-Dateien (Restliche Funde)**:
   - Fundstellen in `tests/*` und `lsm/tests/*` sind synthetische Test-Locks (z. B. `TEST_LOCK.lock().await` zur Test-Serialisierung) und für den Produktionscode unkritisch.

---

## 3. Konsequenzen
- Die Lock-Hierarchie ist in `crates/contextra-store/src/lsm/lock_order.rs` mit festen Rängen verankert.
- Deadlocks zwischen Commit, Flush, Recovery und Scans sind durch die Einhaltung der Ränge 100 bis 950 mathematisch ausgeschlossen.
