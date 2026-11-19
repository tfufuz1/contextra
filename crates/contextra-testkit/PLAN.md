# contextra-testkit — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §12 und `docs/crates/contextra-testkit.md`. **Ausgangslage:** Das Testkit hat einen zu breiten Satz historischer Index-/MVCC-Doubles und ein nichtleeres Default-Feature.

1. Setze `default = []` und beschränke Produktionsabhängigkeiten auf die normativen Port-/Typverträge; kein Produktcrate darf Testkit normal dependieren.
2. Liefere `ManualClock`, `FaultVfs`, deterministische Layer-/Barrier-/Key-Slot-Doubles und eine `Scanner<Armed>`-Positivkontrolle mit Canary; bewaffnen muss explizit sein.
3. Implementiere wiederverwendbare Erasure-Conformance-Suite: Gate, idempotentes Purge, Witness, Crash/Recovery und keine Residuen nach erfolgreichem Testfall.
4. Füge einen Layering-Gate-Helfer hinzu, der Cargo-Graph und Default-Features gegen R1–R3 prüft; Testdoubles dürfen nur testbar, nicht produktiv exportiert sein.
5. Entferne nicht verwendete historische Doppelsysteme, halte die Doubles deterministisch und dokumentiere ihre bewussten Modellgrenzen.

**Abnahme:** Positivkontrolle findet absichtlich platzierten Canary, Conformance-Suite gegen mindestens einen Fake-Layer, Testkit-nicht-im-Release-Graph und Feature-Audit.
