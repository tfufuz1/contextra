Öffentliche API und Verdrahtung. Enthält keine Logik.

## Status

Ring 4. Ist-Zustand: vorhanden (1.558 Zeilen), widerspricht SPEC 7 in mehreren Punkten (`builder(dimension)`, `EncryptionMode`, `LicenseGate`, `TextEmbeddingEngine`, Profile, Rewriter). Soll-Zustand nach [`docs/SPEC.md`](../../docs/SPEC.md), Abschnitt 5.1 und 8.

## Abhängigkeiten

Erlaubt laut SPEC 5.3: `types`, `durable-fs`, `engine` (D-88).

Das Crate ist Teil der Architekturprüfung (`contextra-arch-tests`). Abhängigkeiten werden nicht von Hand gepflegt, sondern gegen die Matrix geprüft (AR-1, AR-11).
