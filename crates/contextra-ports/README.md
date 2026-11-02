Schnittstellen (Traits) zwischen den Schichten: Speicher, Index, Dateisystem, Zeit, Zufall, Löschschicht.

## Status

Ring 0. Ist-Zustand: vorhanden; Altports `graph`, `checkpoint`, `license`, `attention`, `plugin` entfernt; `embedding`, `reranker`, `query_rewriter` noch vorhanden; Löschports fehlen. Soll-Zustand nach [`docs/SPEC.md`](../../docs/SPEC.md), Abschnitt 5.1 und 8.

## Abhängigkeiten

Erlaubt laut SPEC 5.3: `types`.

Das Crate ist Teil der Architekturprüfung (`contextra-arch-tests`). Abhängigkeiten werden nicht von Hand gepflegt, sondern gegen die Matrix geprüft (AR-1, AR-11).
