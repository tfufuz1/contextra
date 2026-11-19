# contextra-types — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §§3, 5, 11, 14 und `docs/crates/contextra-types.md`. **Ausgangslage:** 5.281 LOC mit 32 Enums, alten Query-/Graph-/Ranking-Typen und mehreren Produktions-Panikmustern, obwohl der Sollzustand ein kleines kanonisches Datenmodell ist.

1. Inventarisiere öffentliche Typen, ordne jeden Begriff genau einem Besitzer zu und entferne doppelte IDs zugunsten von `contextra-ids`.
2. Reduziere die Kernoberfläche auf `ContextraError`, `EraseTarget`, `SagaState`, Retention-Attestation/-Code, `Document`/`SearchResult` und `ContextraConfig` samt erforderlichen Encryption/Guard/DataClass/Erasure-Feldern.
3. Mache alle Persistenz-DTOs versioniert und serde-stabil; kein I/O, keine Krypto, keine Indexpolicy und keine Logik höherer Schichten.
4. Ergänze die kanonischen Fehlervarianten (`Erased`, `Restricted`, Auditpflicht, ClassViolation, Crypto, SnapshotRevoked) ohne Klartext-/Schlüssel-Leaks und ersetze die bestehenden Panic-Pfade.
5. Leere Features bis auf ausdrücklich spezifizierte Kompatibilität, migriere Nutzer schrittweise und entferne alte Re-Exports nach Public-API-Diff.

**Abnahme:** Serde-/Bincode-Golden-Tests, Fehler-Redaction-Tests, Typbesitzer-/Dependency-Gate und Zero-Panic-Gate.
