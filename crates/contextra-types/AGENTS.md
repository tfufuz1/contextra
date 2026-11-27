Das Crate enthält DocId (u64, mit Feature `docid-128` u128), TxId, TenantId, Token-Budgets, Filter, Tags und den zentralen Fehlerbaum ContextraError.

Deine Stärken: Newtype-Design, typsichere IDs, Wire-Kompatibilität über serde und bincode, semver-bewusste API-Evolution, Property-Tests mit proptest.

Harte Regeln:
- Keine Systemzeit, kein Thread-RNG, keine I/O. `#![forbid(unsafe_code)]` bleibt bestehen. Keine internen Abhängigkeiten.
- Jede Änderung an Serialisierungsformaten ist ein Kompatibilitätsereignis. Vorher Round-Trip-Tests und Altdaten-Fixtures prüfen.
- Beide Varianten von `docid-128` müssen kompilieren und getestet werden: `cargo test -p contextra-types` und `cargo test -p contextra-types --features docid-128`.
- Fehler bleiben typisiert, keine String-Fehler.

Vor jeder Änderung: Auswirkungen auf Downstream-Crates (ports, mvcc, store, vector, text, graph, engine, db, contextra, mcp) abschätzen und mit `cargo check --workspace` verifizieren.

Optimierungsfokus: Kleine Typen, `#[must_use]` dort wo sinnvoll, keine Heap-Allokation in Hot-Path-Typen, `const fn` wo möglich.
