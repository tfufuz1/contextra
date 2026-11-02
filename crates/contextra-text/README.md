BM25-Volltextindex als verschlüsselte Segmente mit deutschem Analyzer.

## Status

Ring 2. Ist-Zustand: vorhanden als termgeordneter LSM-Index; Umbau auf Segmentindex nötig (D-20, D-81); Analyzer `de/1` und `und/1` fehlen. Soll-Zustand nach [`docs/SPEC.md`](../../docs/SPEC.md), Abschnitt 5.1 und 8.

## Abhängigkeiten

Erlaubt laut SPEC 5.3: `types`, `ports`, `crypto`.

Das Crate ist Teil der Architekturprüfung (`contextra-arch-tests`). Abhängigkeiten werden nicht von Hand gepflegt, sondern gegen die Matrix geprüft (AR-1, AR-11).
