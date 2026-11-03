HNSW-Vektorindex mit Arena, löschbaren Slots und verschlüsseltem Snapshot.

## Status

Ring 2. Ist-Zustand: vorhanden (13.624 Zeilen in 3.0.0); kompiliert nicht; 46 % des Umfangs liegen außerhalb des Soll (`diskann`, `quantize`, `quantize_rabitq`, `partial_rebuild`, `acorn`); `std::fs` und `buffer_removed` an 51 Stellen. Soll-Zustand nach [`docs/SPEC.md`](../../docs/SPEC.md), Abschnitt 5.1 und 8.

## Abhängigkeiten

Erlaubt laut SPEC 5.3: `types`, `ports`, `simd`, `crypto`.

Das Crate ist Teil der Architekturprüfung (`contextra-arch-tests`). Abhängigkeiten werden nicht von Hand gepflegt, sondern gegen die Matrix geprüft (AR-1, AR-11).
