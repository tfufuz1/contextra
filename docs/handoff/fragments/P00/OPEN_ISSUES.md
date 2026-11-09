# OPEN_ISSUES (Welle 0 - P00)

## Fremdfehler (außerhalb BESITZ)

1. **`contextra-crypto`**: Unerwartete Bedingung `#[cfg(feature = "serde")]` in `crates/contextra-crypto/src/secret.rs:94` und `:105`.
   - *Fehlermeldung:* `error: unexpected 'cfg' condition value: 'serde'`
   - *Auswirkung:* Blockiert das Kompilieren von `contextra-crypto` sowie abhängiger Crates (`contextra-store`, `contextra-kvcache`, `contextra-bench`).
   - *Status:* Nicht in BESITZ von P00. Betrifft `contextra-crypto` (unzugeordnet in P01-P14; gehört zu Sicherheits-/Krypto-Pfaden).

2. **`contextra-graph`**: Mehrfachdefinition des Moduls `contract` in `crates/contextra-graph/src/lib.rs:37` und `:42`.
   - *Fehlermeldung:* `error[E0428]: the name 'contract' is defined multiple times`
   - *Auswirkung:* Blockiert `contextra-graph` und alle darauf aufbauenden Crates (`contextra-engine`, `contextra-db`, `contextra`, `contextra-mcp`, `contextra-py`, `contextra-agent`, `contextra-cognition`).
   - *Zuständigkeit:* Auftrag P06 (graph).

3. **`contextra-vector`**: Doppeldefinition von Methoden in `crates/contextra-vector/src/hnsw/deletion.rs` (`remove`) und `crates/contextra-vector/src/hnsw/types.rs` (`rebuild_sync`), fehlendes `zeroize` und fehlendes Feld `kernel` auf `HnswColdCore`.
   - *Fehlermeldung:* `error[E0592]: duplicate definitions with name 'remove'`
   - *Zuständigkeit:* Auftrag P03 (vector).

4. **`contextra-infer-candle`**: Fehlendes Modul/Crate `thiserror` in `crates/contextra-infer-candle/src/error.rs`.
   - *Fehlermeldung:* `error[E0433]: failed to resolve: use of unresolved module or unlinked crate 'thiserror'`
   - *Zuständigkeit:* unzugeordnet (Crate-Satellit für Candle-Inferenz).
