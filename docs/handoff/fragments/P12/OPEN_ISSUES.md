# P12 Offene Punkte & Fremdfehler

## Fremdfehler (außerhalb BESITZ)

1. **`contextra-graph` Kompilierungsfehler:**
   - Datei: `crates/contextra-graph/src/lib.rs:37, 42`
   - Fehler: `E0428: the name contract is defined multiple times`
   - Sowie fehlende Imports / `E0433` (unresolved `contextra_types`, `contextra_ports`) in `csr/graph_index/`, `csr/graph_write/`, etc.
   - Status: Fremdfehler, gemeldet für den Integrator.

2. **`contextra-vector` Kompilierungsfehler:**
   - Datei: `crates/contextra-vector/src/hnsw/deletion.rs:310`, `types.rs:597`, etc.
   - Fehler: `E0592: duplicate definitions with name remove / rebuild_sync`, `E0599: no method named zeroize`, `E0609: no field kernel on type HnswColdCore`, `E0599: try_new_with_rng_and_kernel`.
   - Status: Fremdfehler, gemeldet für den Integrator.

3. **`contextra-crypto` Kompilierungsfehler:**
   - Datei: `crates/contextra-crypto/src/secret.rs:94, 105`
   - Fehler: unexpected `cfg` condition value `serde`.
   - Status: Fremdfehler, gemeldet für den Integrator.
