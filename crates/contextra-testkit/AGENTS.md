 Es liefert Mocks, In-Memory-Referenzimplementierungen, Fault- und Crash-VFS, Referenzmodelle für Erasure und Konformitätssuiten für Storage, VFS, Clock, Graph, Vector, Text, Metrics und Erasure-Layer.

Harte Regeln:
- Das Crate steht ausschließlich in `[dev-dependencies]`. Ein Produktions-Dependency darauf ist ein Fehler.
- Alle Doubles sind deterministisch. Keine Systemzeit, kein Zufall ohne Seed.
- Konformitätssuiten sind die Vertragsprüfung für jede Port-Implementierung. Ändert sich ein Port, ändert sich die Suite im selben Schritt.
- Das Übergangsfeature `erasure-conformance` wird laut Doku in Phase 2 entfernt. Bis dahin keine neuen Abhängigkeiten darauf.
- Die In-Memory-Implementierungen müssen die echte Semantik abbilden. Abweichungen sind dokumentiert und getestet (`in_memory_doubles.rs`).

Verifikation: `cargo test -p contextra-testkit --locked`, `cargo test -p contextra-testkit --test loom_erasure` mit `--cfg loom`, `cargo test -p contextra-testkit --test conformance_selftest`.

Optimierungsfokus: Schnelle In-Memory-Doubles, geringe Testlaufzeit durch gezielte Seeds und reduzierte Fuzzing-Dauer in Standardläufen.
