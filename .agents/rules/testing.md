# Governance Rule: Deterministic Testing & Reference Models (`rules/testing.md`)

> **Status 30.10.2026 (Reduktion):** Diese Regel gilt weiter, soweit sie nicht auf entfernte Crates oder entfernte Routinen verweist. Solche Stellen sind veraltet. Maßgeblich: `AGENTS.md` und `docs/SPEC.md` (Fassung 4.0).

## Gilt für
- `crates/*/tests/`
- `crates/contextra-testkit`

## Pflichtregeln
1. **Verwendung des `ReferenceModel`**: Komplexe MVCC- und KV-Operationen müssen gegen das deterministische `ReferenceModel` in `contextra-testkit` verifiziert werden (Quelle: `crates/contextra-testkit/src/reference_model.rs`).
2. **Deterministic Time / Sequence Counters**: Tests dürfen keine echten Systemuhren (`Instant::now()`, `SystemTime::now()`) für logische Invarianten oder Compaction-Schwellen nutzen, sondern logische Sequence Numbers (`seqno`) (Quelle: `crates/contextra-store/src/compaction/adaptive.rs`).
3. **Isolated Temp Repos**: Tests dürfen niemals im Arbeitsverzeichnis des Haupt-Repositories schreiben, sondern müssen isolierte `tempfile::tempdir()` Verzeichnisse verwenden.

## Häufige Fehler
- Nutzung von `std::thread::sleep` anstelle von deterministischen Barrier-/Notification-Primitiven.
- Abhängigkeit von Pfaden außerhalb des Temp-Verzeichnisses.

## Verweise
- `crates/contextra-testkit/src/reference_model.rs`
- `crates/contextra-store/src/compaction/adaptive.rs`
