# Befunde (`FINDINGS.md`) — `contextra-vector` (S-36)

## 1. Nicht-blockierende Befunde
- **Warnungen bezüglich veralteter Typen:**
  `contextra_types::ContextraError::HnswConnectivityDegraded` und `GraphRepairFailed` sind als `#[deprecated]` markiert. Diese Typen gehören in das Vektor-Crate und werden im Rahmen von S-36 in `VectorError` konsolidiert.
- **Determinismus-Heuristiken:**
  In Testdateien sowie im experimentellen RaBitQ/DiskANN-Code werden teilweise Standard-`HashSet`s verwendet. Diese berühren nicht den Kern-HNSW-Indexierungspfad, wurden aber inventarisiert.

## 2. Blockierende Befunde (minimal behoben)
- **`tests/partial_rebuild_recall_regression.rs` (Kompilierfehler E0277):**
  - **Symptom:** Beim Kompilieren unter `--features partial-index-rebuild` schlug der Test fehl mit:
    `error[E0277]: 'Result<(), ContextraError>' is not a future ... remove the .await`.
  - **Ursache:** `HnswIndex::rebuild_region` ist eine synchrone Hilfsfunktion (`pub fn rebuild_region(&self, ...) -> Result<()>`). Im Test wurde fälschlicherweise dreimal `.await` angehängt.
  - **Behebung:** Entfernen von `.await` an den drei Aufrufstellen in `tests/partial_rebuild_recall_regression.rs`.
