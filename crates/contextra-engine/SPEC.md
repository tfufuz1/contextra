# Spezifikation `contextra-engine` (Level 3 – Orchestrierung und Integration, R12 K-58)

> **Revision R12** (Repo-Abgleich, HEAD `60ea034`): Änderungen gemäß `SPEC.md` K-58…K-63 und Anhang B.5. Abschnitte mit Kennzeichnung „R12“ sind normativ.

## 1. Rolle & Scope
`contextra-engine` orchestriert Speicher- und Index-Bausteine (`contextra-storage` über `storage-bridge`, Vector, Inverted Text, CSR-Graph, Rank, Checkpoint, KV-Cache) zu einer einheitlichen, ACID-konformen RAG-Engine. Die alte Abhängigkeit auf `contextra-store` (LSM) entfällt mit G-CUTOVER. Sie enthält keine originäre Datenverwahrung oder spezifische Hardware-SIMD-Kerne, sondern delegiert an Klasse-A- und Klasse-B-Bausteine.

## 2. Invarianten

| Invariante | Beschreibung | Test / Verifikation |
|---|---|---|
| `INV-ENG-TX-1` | 4-Index 2-Phase Commit folgt strikt WAL-first (`Intent::Pending` -> `Storage Commit` -> `Index Staged Ops` -> `CommitIntent::Committed`). **R12:** Übergang: ab G-CUTOVER ist der `Receipt` des Speicherkerns der Commit-Punkt; Index-Seiten folgen über `ConsumerFeed`/Watermark (K-48). | `tests/commit_crash_consistency.rs`, `src/transaction/tests.rs` |
| `INV-ENG-TX-2` | Idempotente Kompensation: Abbruch an jedem Commit-Schritt stößt Rückabwicklung über den `CommitLedger` an. | `tests/tx_fault_injection.rs` |
| `INV-ENG-LOCK-1` | Lock-Hierarchie: Dokument -> Commit-Barriere -> Purge-Barriere -> Journal/WAL -> Speicherebene. Kein Mutex/RwLock-Guard über `.await`. | `tools/concurrency_lock_auditor.py` |
| `INV-ENG-WORKER-1` | Worker arbeiten mit synchronem `step()` (`reap_orphans_step`) und deterministischer Clock-Injektion. | `src/background_workers/orphan_workers.rs` |
| `INV-ENG-PURGE-1` | Physische Bereinigung und Scanning delegieren rein über neutrale Port-Traits (`Purger`, `LayerScanner`). Null DeletionProof/Saga/DSGVO-Kopplung im Crate (M7). | `tools/erasure_leak_prober.py` (0 Verstöße) |
| `INV-ENG-SEP-1` (R12, K-60) | Personenbezogene und Korpus-Chunks werden vor dem Index-Routing getrennt. `Personal`-Chunks gehen ausschließlich an `Personal`-Partitionen. | `tests/personal_routing.rs` (geplant) |
| `INV-ENG-ASYNC-1` (R12, K-62) | `block_on` ist nur in dieser Ebene und im Rand (`crates/*/src/adapters`) erlaubt. Level-2-Crates bleiben synchron. | `tools/concurrency_lock_auditor.py` (Erweiterung R12) |
| `INV-ENG-LIMITS-1` | Bounded Resources: Import/Export und Purgers validieren Größenbeschränkungen (z.B. `MAX_IMPORT_COLLECTIONS`, `MAX_ORPHANS_PER_TICK`). | `src/import.rs`, `src/background_workers/orphan_workers.rs` |

## 3. Entkopplungs- und Port-Architektur
Die Engine kommuniziert mit Bausteinen über `contextra-ports`:
- Speicher: `storage-bridge` (R12). Bis G-CUTOVER übergangsweise `StorageEngine` (K-63).
- Partitionen und Watermarks: `index-bridge` (`SegmentOwner`, `ConsumerFeed`, `Watermark`, R12)
- Vektor-Index: `VectorIndex`
- Volltext: `TextIndex`
- Graph-Index: `GraphIndex`
- Umgebungs-Traits: `Clock`, `Rng`, `MetricsSink`

Adapter-Brücken zu externen oder spezifischen Modellen liegen unter `crates/contextra-engine/src/adapters/`.
Löschdomänen-Orchestrierung liegt isoliert in `contextra-erasure`.
