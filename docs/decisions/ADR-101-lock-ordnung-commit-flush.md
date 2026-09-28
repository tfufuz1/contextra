# ADR-101: Lock-Ordnung und Entkopplung von Commit und Flush

## Status
Proposed

## Kontext
Im Commit-Pfad von `contextra-store` war der Flush-Vorgang (`storage.flush().await?`) synchron integriert. Dies führte zu verlängerten Haltezeiten von `commit_mutex` und konnte synchrone I/O-Fehler beim Commit erzeugen, obwohl die WAL-Persistenz bereits garantiert war. Zudem gab es Inkonsistenzen bei der Veröffentlichung der MVCC-Sichtbarkeit und der WAL-HMAC-Kette vor erfolgreichem Write.

## Entscheidung
1. **Entkopplung von Flush**: `flush()` wird vollständig vom synchronen Commit-Pfad entkoppelt. Committer rufen stattdessen nach der MemTable-Aktualisierung `request_flush()` auf. Ein dedizierter Hintergrund-Worker verarbeitet Flush-Signale asynchron. Flush-Fehler beeinträchtigen nicht mehr bereits dauerhaft geschriebene Commits, sondern setzen den Systemzustand `StorageHealth::FlushFailing`.
2. **Strikte Lock-Hierarchie**:
   - `commit_mutex` (`tokio::sync::Mutex<()>`)
   - `pending_commit_queue` (`tokio::sync::Mutex<Option<PendingCommitQueue>>`)
   - `state` (`tokio::sync::RwLock<LsmState>`)
   - `wal` (`RwLock<Arc<Wal>>`)
   - `wal.truncate_lock` (`tokio::sync::Mutex<()>`)
   - Leaf-Locks: `intent_locks` (`std::Mutex`) und `tx_buffer` (kein `.await` unter Leaf-Locks)
3. **Veröffentlichungsreihenfolge**: MemTable-Updates werden angewendet und `last_applied_seq` aktualisiert, bevor `advance_visibility(tx_id)` aufgerufen wird.
4. **WAL-HMAC-Kette**: Die HMAC-Kette wird im Arbeitsspeicher erst nach physisch erfolgreichem Append im Flusher-Actor aktualisiert.

## Konsequenzen
- Signifikant reduzierte Commit-Latenz und Eliminerung von I/O-Sperren während Flushes.
- Deterministischeres Verhalten bei Abbruch und Recovery.
