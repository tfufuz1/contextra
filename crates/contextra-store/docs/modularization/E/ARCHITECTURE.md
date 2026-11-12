# ARCHITECTURE — LSM-Schreibpfad (Komponente E)

## 1. Übersicht & Datenpfad
Der Schreibpfad in `contextra-store` steuert alle zustandsverändernden Operationen (`put`, `put_if_absent`, `delete`, `delete_many`, `delete_prefix`, `commit`, `rollback`) auf Transaktionsebene.

```text
Client / Transaktion
      │ (put / delete / put_if_absent)
      ▼
   TxBuffer (Staging im RAM mit DocId-Ableitung)
      │
      ▼ commit()
┌─────────────────────────────────────────────────────────────┐
│ 1. Backpressure & Memory-Budget Check                       │
│ 2. IntentLockGuard erzeugen                                 │
│ 3. commit_mutex akquirieren                                 │
│ 4. ReadSet & Staged Ops aus TxBuffer entnehmen              │
│ 5. SSI-Validierung gegen Anti-Dependencies                  │
│ 6. prepare_commit_ops (SeqNo-Vergabe, WAL- & Mem-Vektoren)  │
└─────────────────────────────────────────────────────────────┘
      │
      ├──> Single-Commit (window == 0 oder MemoryOnly)
      │      ├── WAL prepare_batch
      │      ├── wal.truncate_lock akquirieren (ADR-0XX-P0A Handoff)
      │      ├── commit_mutex freigeben
      │      ├── WAL Append (fsync)
      │      ├── state.read() -> MemTable Update
      │      ├── advance_visibility (last_committed_tx Release)
      │      └── Observer benachrichtigen
      │
      └──> Group-Commit Pipeline (window > 0)
             ├── Leader registriert PendingCommitQueue
             ├── Followers enqueuen GroupCommitRequest & warten auf oneshot
             ├── Leader wartet window oder bis Queue voll (Notify)
             ├── Leader nimmt Queue unter commit_mutex
             ├── WAL prepare_batch für gesamten Batch
             ├── wal.truncate_lock akquirieren (Lock-Handoff)
             ├── commit_mutex freigeben
             ├── WAL Append für gesamten Batch
             ├── state.read() -> MemTable Updates für alle Tx
             ├── advance_visibility für alle Tx
             ├── Observer benachrichtigen
             └── Follower über oneshot-Kanäle aufwecken
```

## 2. Lock-Hierarchie & Sperrenregister
Gemäß SPEC §3.5, SPEC §5.4, MV-08 und Invariante `INV-LOCK-ORDER-1`:

| Rang | Konstante | Primitiv | Feld / Instanz | Haltedauer | Zweck |
|---|---|---|---|---|---|
| 100 | `RANK_DOCUMENT_INTENT` | `std::sync::Mutex` | `LsmStorage::intent_locks` | < 10 µs | TOCTOU-Schutz bei `put_if_absent` |
| 200 | `RANK_COMMIT_MUTEX` | `tokio::sync::Mutex` | `LsmStorage::commit_mutex` | < 50 ms | Sequenzvergabe & Batch-Vorbereitung |
| 300 | `RANK_FLUSH_MUTEX` | `tokio::sync::Mutex` | `LsmStorage::flush_mutex` | < 500 ms | Serialisiert MemTable-Flushes |
| 400 | `RANK_STATE_RWLOCK` | `tokio::sync::RwLock` | `LsmStorage::state` | < 50 µs | MemTable-Zeiger & immutable MemTables |
| 500 | `RANK_WAL_TRUNCATE` | `tokio::sync::Mutex` | `Wal::truncate_lock` | < 10 ms | WAL-Truncate & Append-Serialisierung |
| 600 | `RANK_SSTABLES_RWLOCK` | `tokio::sync::RwLock` | `LsmStorage::sstables` | < 100 µs | SSTable-Reader-Katalog |
| 700 | `RANK_MEMTABLE_SHARDS` | `parking_lot::RwLock` | `MemTable::shards` | < 5 µs | BTreeMap-Shards im RAM |
| 800 | `RANK_MANIFEST_MUTEX` | `tokio::sync::Mutex` | `Manifest::file` | < 20 ms | Manifest-Dateizugriff |
| 900 | `RANK_OBSERVER_REGISTRY` | `parking_lot::RwLock` | `ObserverRegistry::observers` | < 5 µs | Registrierung von WAL-Beobachtern |
| 950 | `RANK_BLOCK_CACHE` | `parking_lot::RwLock` | `SieveCacheBackend::state` | < 2 µs | LRU/SIEVE Cache Evictions |

## 3. Lock-Handoff-Invariante (ADR-0XX-P0A)
Beim Übergang von der Batch-Vorbereitung zur physischen Persistenz wird `wal.truncate_lock` akquiriert, **bevor** `commit_mutex` freigegeben wird. Dadurch wird garantiert, dass keine fremde Transaktion oder Truncate-Operation zwischen Sequenznummernvergabe und WAL-Append interleavt.

## 4. Observer-Architektur
- **Fail-Open**: Tritt bei der Ausführung eines Observers eine Verzögerung über `max_observer_latency` oder ein Panic auf, schlägt der Circuit-Breaker an. Die Transaktion wird niemals blockiert oder abgebrochen.
- **Worker-Thread**: Jeder Observer wird über einen eigenen synchronen Bounded-Kanal (256 Kapazität) bedient, um Tokio-Runtime-Worker-Threads freizuhalten.
