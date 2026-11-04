Schnittstellen (Traits) zwischen den Schichten: Speicher, Index, Dateisystem, Zeit, Zufall, Löschschicht.

## Status

Ring 0. Ist-Zustand: Produktionsreife für Release 1.0.0; Altports `graph`, `checkpoint`, `license`, `attention`, `plugin`, `embedding`, `reranker`, `query_rewriter` entfernt; alle Löschports (`ErasureLayer`, `ReadGate`, `RecordSealer`, `FileKeyring`, `LedgerSink`, `ErasureObserver`, `PreGate`, `ReportSigner`) nach SPEC 8.2 und 3.1 vorhanden.

## Abhängigkeiten

Erlaubt laut SPEC 5.3: `types`.

Das Crate ist Teil der Architekturprüfung (`contextra-arch-tests`). Abhängigkeiten werden nicht von Hand gepflegt, sondern gegen die Matrix geprüft (AR-1, AR-11).
