# ADR-105: SSTable Format Version 2 für MVCC Multi-Version Retention

## Status
Proposed

## Kontext
Bisherige SSTables (Format Version 1) schrieben beim Flush pro Key nur die jeweils aktuellste Version. Dadurch gingen gepinnte Snapshot-Versionen und Tombstones oberhalb des Snapshot-Floors verloren.

## Entscheidung
1. **Format Version 2**: Die SSTable-Formatversion wird im Trailer auf `2` angehoben. SSTables im Format Version 2 speichern mehrere Versionen pro Key geordnet nach (`key ASC`, `seq DESC`).
2. **Abwärtskompatibilität**: Reader unterstützen weiterhin Format Version 0, 1 und 2. Für v1 wird bei `get_at` die gegebene Einzelversion geprüft.
3. **Gemeinsame Retention-Regel**: Flush und Compaction-Merge nutzen `retain_key_versions`: Alle Versionen mit `raw_seq > floor_seq` werden behalten, ebenso wie die erste (neueste) Version mit `raw_seq <= floor_seq`.
4. **Point Lookups**: `SstableReader::get_at(key, max_seq, max_tx)` durchsucht Blöcke und gibt die neueste zulässige Version zurück, die `max_seq` und `max_tx` erfüllt.

## Konsequenzen
- Erhalt der MVCC-Historie über Flushes hinweg für gepinnte Snapshots.
- Keine Migration bestehender SSTables erforderlich; v1-Dateien bleiben nahtlos lesbar.
