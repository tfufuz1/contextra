# Spezifikation: `contextra-engine`

**Ring:** Ring 3 (Orchestration & Integration)
**Normativität:** Normativ gemäß `docs/SPEC.md` §1.1, §6.3
**Klasse:** C (Integrations- und Orchestrierungs-Triebwerk)
**MSRV:** 1.89.0
**Code-Disziplin:** `#![forbid(unsafe_code)]`, Zero-Panic

---

## 1. Zweck & Systemgrenzen
`contextra-engine` orchestriert Speicher- und Index-Bausteine (LSM, Vector, Inverted Text, CSR-Graph) zu einer einheitlichen, ACID-konformen RAG-Engine:
- **Scope:** 4-Index 2-Phase-Commit Transaktionen, Savepoints, Hintergrund-Worker (Flush, Compaction, Orphan-Reaping) und Inferenz-Adapter.
- **Systemgrenzen:** Enthält keine originäre persistente Datenverwahrung oder spezifische Hardware-SIMD-Kerne, sondern delegiert an Klasse-A- und Klasse-B-Bausteine.
- **Entkopplung der Löschdomäne:** Physische Bereinigung und Scanning delegieren rein über neutrale Port-Traits (`Purger`, `LayerScanner`). Null DeletionProof/Saga-Kopplung im Crate (Regel M7, 0 Verstöße im Leak-Prober).

---

## 2. Metriken & Werkzeug-Audit-Befunde
*Erfasst via `tools/decouple_inspector.py -c contextra-engine`:*
- **LOC:** 17.641 LOC über 87 Rust-Dateien.
- **Typen-Inventar:** 25 Traits, 107 Structs, 31 Enums, 357 öffentliche Funktionen (`pub fn`).
- **Abhängigkeiten:** 12 interne Crates (`contextra-types`, `contextra-ports`, `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-crypto`, `contextra-checkpoint`, `contextra-mvcc`, `contextra-privacy`, `contextra-rank`, `contextra-adapt`).
- **Audit-Befund:** 9 Legacy-Verweise auf Purge-Begriffe; 0 Unsafe.

---

## 3. Architektur & Modulkarte

| Modul | Verantwortung | Primäre Typen / Traits |
|---|---|---|
| `transaction/` | 4-Index Transaktionskoordination, 2PC, Intent-Logs | `Transaction`, `CommitIntent`, `TxContext` |
| `background_workers/` | Flush-, Compaction- und Orphan-Reaper-Worker | `OrphanWorker`, `WorkerHandle` |
| `adapters/` | Entkoppelte Adapter zu Inferenz-, Text- und Privacy-Bausteinen | `TextAdapter`, `PrivacyAdapter`, `OllamaAdapter` |
| `query/` | Hybridsuch-Ausführung und Candidate-Fusion | `HybridSearchEngine`, `QueryPlan` |
| `import.rs` / `export.rs` | Bounded Collection Import/Export mit Validierung | `CollectionExporter`, `ImportLimits` |

---

## 4. Invarianten (INV-ENG-*)

| Invariante | Beschreibung | Verifikation |
|---|---|---|
| `INV-ENG-TX-1` | 4-Index 2-Phase Commit folgt strikt WAL-first (`Pending` -> `Storage Commit` -> `Index Staged Ops` -> `Committed`). | `tests/commit_crash_consistency.rs` |
| `INV-ENG-TX-2` | Idempotente Kompensation: Abbruch an jedem Commit-Schritt stößt LIFO-Rückabwicklung an. | `tests/tx_fault_injection.rs` |
| `INV-ENG-LOCK-1` | Lock-Hierarchie: Dokument -> Commit -> Purge -> WAL -> Speicherebene. Kein Mutex/RwLock-Guard über `.await`. | `tools/concurrency_lock_auditor.py` |
| `INV-ENG-WORKER-1` | Worker arbeiten mit synchronem `step()` (`reap_orphans_step`) und deterministischer Clock-Injektion. | `src/background_workers/orphan_workers.rs` |
| `INV-ENG-PURGE-1` | Physische Bereinigung delegiert ausschließlich über neutrale Port-Traits (`Purger`, `LayerScanner`). | `tools/erasure_leak_prober.py` |
| `INV-ENG-LIMITS-1` | Bounded Resources: Import und Purgers validieren Größenbeschränkungen (`MAX_IMPORT_COLLECTIONS = 10_000`). | `src/import.rs` |

---

## 5. Zielzustand & Migrationspfad
Gemäß `docs/SPEC.md` §6.3:
1. `chunker/` als dünnen Adapter über `contextra-ingest::chunking` abbilden.
2. `temporal_filter.rs` nach `contextra-rank` überführen.
3. `decay_controller.rs` nach `contextra-adapt` verlagern.
4. Lizenzprüfungen (`contextra_impl/lifecycle.rs`) vollständig in `contextra-license` kapseln.
