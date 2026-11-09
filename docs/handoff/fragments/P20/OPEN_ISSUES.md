# OPEN_ISSUES (Welle 2 - P20 - WP-14)

## Fremdfehler (außerhalb BESITZ)

1. **`contextra-crypto`**: Unerwartetes cfg-Feature `serde`.
   - *Fehlermeldung:* `error: unexpected 'cfg' condition value: 'serde'`
   - *Auswirkung:* Blockiert direkte und indirekte Kompilierung von `contextra-crypto`.

2. **`contextra-graph`**: Mehrfachdefinition des Moduls `contract` in `crates/contextra-graph/src/lib.rs:37` und `:42` sowie unaufgelöste `contextra_types` / `contextra_ports` Typen.
   - *Fehlermeldung:* `error[E0428]: the name 'contract' is defined multiple times`
   - *Auswirkung:* Blockiert `contextra-graph` und damit alle nachgelagerten Crates inklusive `contextra-engine`, `contextra-db`, `contextra`, `contextra-mcp`.
   - *Zuständigkeit:* Auftrag P06 (graph).

3. **`contextra-vector`**: Doppeldefinition von Methoden in `crates/contextra-vector/src/hnsw/deletion.rs` (`remove`) und `crates/contextra-vector/src/hnsw/types.rs` (`rebuild_sync`).
   - *Fehlermeldung:* `error[E0592]: duplicate definitions with name 'remove'`
   - *Zuständigkeit:* Auftrag P03 (vector).
