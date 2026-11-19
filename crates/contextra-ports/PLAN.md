# contextra-ports — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §§4.3, 7, 10 und `docs/crates/contextra-ports.md`. **Ausgangslage:** Das vermeintliche Port-Crate hat 30+ historische Traits, E2-Typabhängigkeit und Produktions-Panikmuster; es muss auf die normativen Verträge schrumpfen.

1. Entferne doppelte Clock/RNG/VFS/Index/Inference/Plugin-/Observability-Verträge; diese gehören ihren jeweiligen E0/E1-Besitzern oder sind nicht im Scope.
2. Definiere allein `CommitBarrier`, `PurgeBarrier`, `EffaceableKeySlot`/`KeyAnchor`, `Purgeable`, `EmbeddingEngine` sowie die spezifizierten Erasure-/Witness-Datentypen als object-sichere, dokumentierte Ports.
3. Implementiere Barrieren mit expliziter Sperr- und Speicherordnung: shared commit vs. exclusive gate, Purge nur nach Gate; keine await-holding-lock-Pfade.
4. Verwende primitive/ID-Typen aus E0, nicht `contextra-types`, wenn dies eine E2→E0-Rückkante erzeugt; behebe damit Ringmetadaten und Cargo-Abhängigkeiten.
5. Ersetze Produktions-Panics in allen verbleibenden Port-Helfern durch `PortError` und liefere Conformance-Tests als Dev-/Testkit-Verbraucher.

**Abnahme:** object-safety-Compile-Tests, Barrier-Concurrency-/Fence-Tests, Contract-Conformance und graphfreier E0-Dependency-Audit.
