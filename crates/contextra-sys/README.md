Einzige `unsafe`-Stelle für Betriebssystemaufrufe: `fsync`, `mlock`, Prozesshärtung.

## Status

Ring 1. Ist-Zustand: vorhanden, 30/30 Lib-Tests in 3.0.0; `acl_win32.rs` und `mmap.rs` zu entfernen; `mmap_readonly` ist `unsafe fn` und bricht den Build der Aufrufer. Soll-Zustand nach [`docs/SPEC.md`](../../docs/SPEC.md), Abschnitt 5.1 und 8.

## Abhängigkeiten

Erlaubt laut SPEC 5.3: `types`.

Das Crate ist Teil der Architekturprüfung (`contextra-arch-tests`). Abhängigkeiten werden nicht von Hand gepflegt, sondern gegen die Matrix geprüft (AR-1, AR-11).
