# Tests

Abgeleitet aus SPEC 15.1, D-44, D-46, T-TRACE-1, P7, P10.

- **Rot vor Grün.** Der Test schlägt zuerst fehl und zeigt die Abwesenheit der Funktion (D-44).
- **Rückverfolgbarkeit.** Jeder Test trägt `#[ctx_test(id = "T-…")]`, jede Benchmark `#[ctx_bench(id = "BM-…")]`. `contextra-arch-tests` prüft, dass jede ID existiert und abgedeckt ist (T-TRACE-1).
- **Determinismus.** Tests nutzen `ManualClock`, den deterministischen `Rng` und die In-Memory-Ports aus `contextra-testkit` (P7).
- **Fehlerinjektion.** Crash- und Fehlertests nutzen `FaultVfs` und decken jeden `write`, `fsync`, `rename` und `unlink` der Saga und des Commit-Pfads ab (SPEC 15.1).
- **Testfeature.** Das Feature `testing` ersetzt nur den Argon2-Schritt und ist nie im Release aktiv (D-46, T-CFG-2).
- **Eigenschaftstests** prüfen gegen das `ReferenceModel` (naives In-Memory-Orakel).
- **Nebenläufigkeit** mit `loom` für Commit-Barriere, Gate und Purge-Barriere (T-CONC-1).
- Deaktivierte Tests brauchen ein Ablaufdatum in `governance/exceptions.toml`.
