# SPEC: `contextra-vector` (HNSW-Vektorindex mit Quantisierung & Filterung)

## 1. Zweck und Architektur-Rolle
`contextra-vector` ist ein autarker Baustein der **Klasse A** (SPEC v4.0 §5.1, §11.4) zur graphbasierten approximativen Nächste-Nachbarn-Suche (Hierarchical Navigable Small World, HNSW) und Quantisierung (SQ8, RaBitQ).
Der Baustein ist vollständig unabhängig von Transaktions- und Storage-Kernen (`contextra-mvcc`, `contextra-types`, `contextra-ports`).

## 2. Invarianten
- **INV-VEC-CONNECTIVITY-1:** Nach dem Entfernen eines Knotens bleibt die Graph-Konnektivität durch gezielte Nachbarschaftsreparatur oder vollständigen Rebuild erhalten.
- **INV-VEC-ZEROIZE-1:** Physisch freigegebene Vektorslots werden im Speicher mit Nullen überschrieben (`zeroize`), bevor Slots wiederverwendet werden.
- **INV-HNSW-SLOT-1 / VETO-F02:** Löschen entspricht Tombstone-Markierung + Slot-Nullung gefolgt von einem vollständigen Rebuild. Ein direktes Re-Wiring ohne Rebuild ist als Löschstrategie untersagt.
- **INV-INDEX-APPLY-1:** HNSW-Ebenen folgen deterministisch aus `hash(doc_id, seed)` bzw. einem injizierten `Rng`-Generator.

## 3. Konstanten & Schranken
- `MAX_SEARCH_K = 10_000`: Höchstgrenze für `k` in Ähnlichkeitssuchen.
- `TOMBSTONE_BIT = 1 << 63`: Höchstwertiges Bit zur Markierung von Tombstones in Sequenznummern.

## 4. Concurrency & Nebenläufigkeit
- Suchen (`search`, `search_at`) laufen nebenläufig unter Snapshot-Isolation ohne Schreibblockaden.
- Schreiboperationen (`insert`, `delete`, Rebuild) werden über einen atomaren Applier serialisiert.
- Keine Locks werden über `.await`-Punkte gehalten.

## 5. Fehlermodell (B4)
Ein einheitlicher Fehlertyp `VectorError` (`#[derive(thiserror::Error)]`, `#[non_exhaustive]`):
- `InvalidInput(String)`
- `Index(String)`
- `DimensionMismatch { expected: usize, actual: usize }`
- `ConnectivityDegraded { deleted_ratio: f64 }`
- `GraphRepairFailed(String)`
- `Concurrency(String)`
- `Io(String)`
- `Unsupported(String)`
