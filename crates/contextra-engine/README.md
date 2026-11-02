Orchestrierung: Mandanten, Collections, Transaktionen, Suche, Hintergrundtasks.

## Status

Ring 3. Ist-Zustand: vorhanden (28.001 Zeilen); kompiliert in der Prüfung 4.2.0 nicht erreichbar; enthält Fremdmodule (`chunker`, `extraction`, `export`, `import`, `temporal_filter`, `decay_controller`); Modul `contextra_impl` heißt `instance`. Soll-Zustand nach [`docs/SPEC.md`](../../docs/SPEC.md), Abschnitt 5.1 und 8.

## Abhängigkeiten

Erlaubt laut SPEC 5.3: `types`, `ports`, `sys` (nur Prozesshärtung, D-88), `durable-fs`, `crypto`, `mvcc`, `store`, `text`, `vector`, `rank`, `erasure`.

Das Crate ist Teil der Architekturprüfung (`contextra-arch-tests`). Abhängigkeiten werden nicht von Hand gepflegt, sondern gegen die Matrix geprüft (AR-1, AR-11).
