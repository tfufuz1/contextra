# Open Issues (P14: Evaluations-Harness)

## Fremdfehler (Stand: Arbeitsbeginn / Vorarbeit)
Folgende Crates außerhalb BESITZ kompilieren vor Beginn der P14-Arbeiten nicht (verifiziert am HEAD 65e1d68a):

1. **`contextra-crypto`**:
   - `crates/contextra-crypto/src/secret.rs:94:7`: `error: unexpected 'cfg' condition value: 'serde'` (`-D unexpected-cfgs`).
   - Verhindert das Bauen aller abhängigen Crates (`contextra-store`, `contextra-db`, `contextra-engine`, `contextra`).
2. **`contextra-graph`**:
   - `crates/contextra-graph/src/csr/graph_index/mutations.rs`: Unaufgelöste Symbole `contextra_ports` und `contextra_types` (über 150 Compilerfehler `error[E0433]`, `error[E0428]`).
3. **`contextra-vector`**:
   - `crates/contextra-vector/src/hnsw/types.rs`: `error[E0592]: duplicate definitions with name 'remove'` und `'rebuild_sync'`.
   - `error[E0599]: no method named 'zeroize' found for mutable reference '&mut [f32]'`.
4. **`contextra-infer-candle`**:
   - `crates/contextra-infer-candle/src/lib.rs`: Unaufgelöstes Modul `thiserror` (`error[E0433]`).
5. **`contextra` (Fassade) & `contextra-db`**:
   - Kompilieren nicht aufgrund der obigen Fehler in `contextra-graph`, `contextra-crypto` und `contextra-vector`.

## Status contextra-bench
- Die autarke Evaluierungs-Logik (Metriken, Datensätze, Adapter-Skizzen) wird in `benchmarks/contextra-bench` autark bzw. isoliert bereitgestellt und mit Unit-Tests validiert.
- Ein vollständiger Lauf der Vergleichsmatrix und des Benchmarks auf dem Mini-Korpus ist blockiert, weil die Kern-Crates (`contextra-crypto`, `contextra-graph`, `contextra-vector`) nicht kompilieren.
- Sobald die Fremdfehler durch die jeweils zuständigen Arbeitspakete behoben sind, kann der vollständige Durchstich über die Fassade ausgeführt werden.
