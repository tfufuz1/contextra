# Fehlerinventar: `contextra-vector` (S-36)

Erhebung aller im Quelltext (`crates/contextra-vector/src/`) vorkommenden Fehlerkonstruktionen und Abbildung auf `VectorError` und `contextra_types::ContextraError`.

## 1. Varianten-Zuordnungstabelle

| Lokale `VectorError`-Variante | Ziel `ContextraError`-Variante | Typischer Display-String (bitgleich) |
|---|---|---|
| `InvalidInput(String)` | `InvalidInput(String)` | `Invalid input: {msg}` |
| `Index(String)` | `Index(String)` | `Index error: {msg}` |
| `Storage(String)` | `Storage(String)` | `Storage error: {msg}` |
| `DimensionMismatch { expected, actual }` | `EmbeddingDimensionMismatch { expected, got: actual }` | `Embedding dimension mismatch: expected {expected}, got {actual}` |
| `ConnectivityDegraded { deleted_ratio }` | `HnswConnectivityDegraded { deleted_ratio }` | `HNSW graph connectivity degraded: {deleted_ratio:.1}% deleted nodes` |
| `GraphRepairFailed(String)` | `GraphRepairFailed(HnswDeletionError)` | `graph repair failed: {msg}` |
| `NotFound(String)` | `NotFound(String)` | `Not found: {msg}` |
| `Conflict(String)` | `Conflict(String)` | `Conflict: {msg}` |
| `Concurrency(String)` | `Index(String)` | `Index error: {msg}` |
| `Io(String)` | `Io(std::io::Error)` | `I/O error: {msg}` |
| `Unsupported(String)` | `CapabilityUnsupported { capability: "vector", reason }` | `Capability unsupported: vector - {reason}` |
| `Internal(String)` | `Internal(String)` | `Internal error: {msg}` |

## 2. Inventarisierte Vorkommen im Code

1. `ContextraError::InvalidInput` / `invalid_input`:
   - `hnsw/config.rs`: Validierung von Parametern wie M, ef_construction, Dimension.
   - `hnsw/adaptive_ef.rs`: Validierung von Parametern für adaptives EF.
   - `hnsw/core_search.rs`, `hnsw/vector_index_impl.rs`: Prüfung von Eingabevektordimensionen und Parametern.
   - `diskann/types.rs`: Validierung von Pfaden und Konfigurationsparametern.
   -> Wird vollständig auf `VectorError::InvalidInput` abgebildet.
2. `ContextraError::Index`:
   - `hnsw/core_rebuild.rs`: Rebuild Backoff Cooldown, Quantizer missing, Corrupt node index.
   - `hnsw/core_search.rs`: Fehlende Nachbarknoten, Quantizer not trained.
   - `hnsw/arena.rs`: Slot-Kapazitätsüberschreitung.
   -> Wird auf `VectorError::Index` abgebildet.
3. `ContextraError::Storage`:
   - `persistence/header.rs`: Magic mismatch, Version unzulässig, Checksum-Fehler.
   - `persistence/node.rs`, `persistence/mmap.rs`: Puffer- und Offset-Fehler.
   - `diskann/persistence.rs`, `diskann/types.rs`: Dateigrößen- und Offset-Gültigkeit.
   - `hnsw/types.rs`: Persistenz, fsync, rename von HNSW-Dateien.
   -> Wird auf `VectorError::Storage` abgebildet.
4. `ContextraError::HnswConnectivityDegraded`:
   - `hnsw/core_search.rs`, `hnsw/vector_index_impl.rs`: Warnschwellenwert für gelöschte Knoten erreicht.
   -> Wird auf `VectorError::ConnectivityDegraded` abgebildet.
5. `ContextraError::GraphRepairFailed`:
   - `hnsw/deletion.rs`: Graph repair DisconnectedComponent / NodeNotFound / VerificationFailed.
   -> Wird auf `VectorError::GraphRepairFailed` abgebildet.
6. `ContextraError::Conflict`:
   - `hnsw/staging.rs`, `hnsw/core_insert.rs`, `hnsw/deletion.rs`: Write-Write-Konflikte oder ID-Kollisionen bei Transaktionen.
   -> Wird auf `VectorError::Conflict` abgebildet.
7. `ContextraError::NotFound`:
   - `diskann/vector_index_impl.rs`: Knoten nicht gefunden.
   -> Wird auf `VectorError::NotFound` abgebildet.
8. `ContextraError::Io`:
   - Datei- und VFS-Operationen.
   -> Wird auf `VectorError::Io` abgebildet.
9. `ContextraError::Internal`:
   - Unzulässiger Zustand im Algorithmus.
   -> Wird auf `VectorError::Internal` abgebildet.
