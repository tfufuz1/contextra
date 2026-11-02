Vektor-Distanzfunktionen (L2, Dot, Cosine) mit SIMD-Kernen und Skalar-Referenz.

## Status

Ring 1. Ist-Zustand: vorhanden, 13/13 Lib-Tests in 3.0.0; u8-VNNI-Pfad zu entfernen. Soll-Zustand nach [`docs/SPEC.md`](../../docs/SPEC.md), Abschnitt 5.1 und 8.

## Abhängigkeiten

Erlaubt laut SPEC 5.3: `types`.

Das Crate ist Teil der Architekturprüfung (`contextra-arch-tests`). Abhängigkeiten werden nicht von Hand gepflegt, sondern gegen die Matrix geprüft (AR-1, AR-11).
