Reciprocal Rank Fusion für die Hybridsuche.

## Status

Ring 2. Ist-Zustand: vorhanden (3.440 Zeilen in 3.0.0), Ziel deutlich kleiner; Module `dibud`, `drift`, `explain`, `fusion/*` außer RRF sind zu entfernen. Soll-Zustand nach [`docs/SPEC.md`](../../docs/SPEC.md), Abschnitt 5.1 und 8.

## Abhängigkeiten

Erlaubt laut SPEC 5.3: `types`, `ports`.

Das Crate ist Teil der Architekturprüfung (`contextra-arch-tests`). Abhängigkeiten werden nicht von Hand gepflegt, sondern gegen die Matrix geprüft (AR-1, AR-11).
