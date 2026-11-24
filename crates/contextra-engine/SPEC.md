# Spezifikation `contextra-engine` (Klasse C – Orchestrierung und Integration)

## 1. Rolle & Scope
`contextra-engine` orchestriert Speicher- und Index-Bausteine (LSM, Vector, Inverted Text, CSR-Graph) zu einer einheitlichen, ACID-konformen RAG-Engine. Sie enthält keine originäre Datenverwahrung oder spezifische Hardware-SIMD-Kerne, sondern delegiert an Klasse-A- und Klasse-B-Bausteine.

## 2. Invarianten

| Invariante | Beschreibung | Test / Verifikation |
|---|---|---|
| `INV-ENG-TX-1` | 4-Index 2-Phase Commit folgt strikt WAL-first (`Intent::Pending` -> `Storage Commit` -> `Index Staged Ops` -> `CommitIntent::Committed`). | `tests/commit_crash_consistency.rs`, `src/transaction/tests.rs` |
| `INV-ENG-TX-2` | Idempotente Kompensation: Abbruch an jedem Commit-Schritt stößt Rückabwicklung über den `CommitLedger` an. | `tests/tx_fault_injection.rs` |
| `INV-ENG-LOCK-1` | Lock-Hierarchie: Dokument -> Commit-Barriere -> Purge-Barriere -> Journal/WAL -> Speicherebene. Kein Mutex/RwLock-Guard über `.await`. | `tools/concurrency_lock_auditor.py` |
| `INV-ENG-WORKER-1` | Worker arbeiten mit synchronem `step()` (`reap_orphans_step`) und deterministischer Clock-Injektion. | `src/background_workers/orphan_workers.rs` |
| `INV-ENG-PURGE-1` | Physische Bereinigung und Scanning delegieren rein über neutrale Port-Traits (`Purger`, `LayerScanner`). Null DeletionProof/Saga/DSGVO-Kopplung im Crate (M7). | `tools/erasure_leak_prober.py` (0 Verstöße) |
| `INV-ENG-LIMITS-1` | Bounded Resources: Import/Export und Purgers validieren Größenbeschränkungen (z.B. `MAX_IMPORT_COLLECTIONS`, `MAX_ORPHANS_PER_TICK`). | `src/import.rs`, `src/background_workers/orphan_workers.rs` |

## 3. Entkopplungs- und Port-Architektur
Die Engine kommuniziert mit Bausteinen über `contextra-ports`:
- Speicher: `StorageEngine`
- Vektor-Index: `VectorIndex`
- Volltext: `TextIndex`
- Graph-Index: `GraphIndex`
- Umgebungs-Traits: `Clock`, `Rng`, `MetricsSink`

Adapter-Brücken zu externen oder spezifischen Modellen liegen unter `crates/contextra-engine/src/adapters/`.
Löschdomänen-Orchestrierung liegt isoliert in `contextra-erasure`.
