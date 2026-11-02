LSM-Speicher: WAL, Memtable, SSTables, Manifest, Kompaktion, Purge-Primitive.

## Status

Ring 2. Ist-Zustand: vorhanden (26.996 Zeilen, 124 Testdateien in 3.0.0); kompiliert nicht (Lizenz-Import in `lsm/config.rs`, `unsafe`-Aufrufe in `wal/replay.rs`); Feature-Altlasten; `std::fs` an 169 Stellen. Soll-Zustand nach [`docs/SPEC.md`](../../docs/SPEC.md), Abschnitt 5.1 und 8.

## Abhängigkeiten

Erlaubt laut SPEC 5.3: `types`, `ports`, `sys`, `crypto`, `mvcc`.

Das Crate ist Teil der Architekturprüfung (`contextra-arch-tests`). Abhängigkeiten werden nicht von Hand gepflegt, sondern gegen die Matrix geprüft (AR-1, AR-11).
