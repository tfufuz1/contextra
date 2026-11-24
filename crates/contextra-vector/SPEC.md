# SPEC: `contextra-vector` (HNSW-Vektorindex mit Quantisierung & Filterung)

## 1. Zweck und Architektur-Rolle
`contextra-vector` ist ein autarker Baustein der **Klasse A** (SPEC v4.0 §4.3, §5.1, §11.4) zur graphbasierten approximativen Nächste-Nachbarn-Suche (Hierarchical Navigable Small World, HNSW) und Quantisierung (SQ8, RaBitQ).
Der Baustein ist vollständig unabhängig von Transaktions- und Storage-Kernen (`contextra-mvcc`, `contextra-types`, `contextra-ports`).

Der Kern-Vertrag ist in `src/contract.rs` eingefroren und definiert `Index`, `Maintain`, `Error`, `Result`, `ExternalId`, `Metric`, `ScoredPoint`, `DistanceFn`, `Clock` und `Rng`.

## 2. Invarianten
- **INV-VEC-CONNECTIVITY-1:** Nach dem Entfernen eines Knotens bleibt die Graph-Konnektivität durch gezielte Nachbarschaftsreparatur oder vollständigen Rebuild erhalten.
  - *Verifiziert durch:* `tests/connectivity_regression.rs`, `tests/partial_rebuild_recall_regression.rs`.
- **INV-VEC-ZEROIZE-1:** Physisch freigegebene Vektorslots werden im Speicher mit Nullen überschrieben (`zeroize`), bevor Slots wiederverwendet werden.
  - *Verifiziert durch:* `tests/hnsw_snapshot_delete_test.rs`, `tests/reconstruction_attack_test.rs`.
- **INV-HNSW-SLOT-1 / VETO-F02:** Löschen entspricht Tombstone-Markierung + Slot-Nullung gefolgt von einem vollständigen Rebuild. Ein direktes Re-Wiring ohne Rebuild ist als Löschstrategie untersagt.
  - *Verifiziert durch:* `tests/partial_rebuild_veto_f02_guard.rs`.
- **INV-INDEX-APPLY-1:** HNSW-Ebenen folgen deterministisch aus `hash(doc_id, seed)` bzw. einem injizierten `Rng`-Generator (SplitMix64 / `SeededRng`).
  - *Verifiziert durch:* `tests/hnsw_layer_determinism.rs`, `tests/hnsw_seed_determinism.rs`.
- **INV-TENANT-IDX-1:** Indizes sind physisch streng isoliert; es gibt keine mandantenübergreifenden Kanten oder Caches.
  - *Verifiziert durch:* Isolationsprüfungen und Namespace-Partitionierung.
- **INV-SIMD-FINITE-1 / INV-SIMD-ALIGN-1:** Vektoren mit `NaN` oder `±Inf` werden strikt zurückgewiesen.
  - *Verifiziert durch:* `tests/nan_validation_policy.rs`, `tests/simd_numerical_audit.rs`.

## 3. Konstanten & Schranken
- `MAX_SEARCH_K = 10_000`: Höchstgrenze für `k` in Ähnlichkeitssuchen.
- `DEFAULT_VECTOR_STREAM_BATCH_SIZE = 16`: Standard-Batchgröße für das geometrische Nachladen im Kandidatenstrom.
- `TOMBSTONE_BIT = 1 << 63`: Höchstwertiges Bit zur Markierung von Tombstones in Sequenznummern.

## 4. Concurrency & Lock-Modell
Das Crate folgt einer strikten Lock-Hierarchie und Concurrency-Disziplin:
- **Hierarchie:** `write_mutex` (exklusive Mutation / Rebuild-Koordination) $\to$ `entry_point` $\to$ `arena / nodes / doc_to_node / deleted_nodes`.
- **Lock-Free / Lock-Arme Reads:** Leseoperationen (`search`, `search_at`) halten zu keinem Zeitpunkt den `write_mutex`. Parallele Suchen operieren nebenläufig auf Snapshot-Sichten.
- **Kein Lock über `.await`:** Gemäß `INV-LOCK-ORDER-1` werden synchrone Guards (`parking_lot::MutexGuard`, `RwLockReadGuard`, etc.) niemals über `.await`-Punkte gehalten. Alle asynchronen Übergänge erfolgen nach dem Freigeben der synchronen Sperren.
- **Atomic Swap für Rebuilds:** Hintergrund-Rebuilds erzeugen einen neuen Indexkern nebenläufig und tauschen den Core atomar aus (`core.swap(...)`).

## 5. Fehlermodell (B4)
Ein primärer Fehlertyp `Error` (`#[derive(thiserror::Error)]`, `#[non_exhaustive]`):
- `InvalidInput(&'static str)`
- `CapacityExceeded { requested: usize, maximum: usize }`
- `Io(std::io::Error)`
- `StateConflict(&'static str)`
- `Corrupt(&'static str)`
Sowie `pub type Result<T> = core::result::Result<T, Error>;`.
