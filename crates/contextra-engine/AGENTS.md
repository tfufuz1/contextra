# AGENTS.md — contextra-engine
> Ring 3 · stable · Quelle: capabilities.toml · Spec: K.10, III.14, L.8, L.11, L.15

## 1. Zweck

Orchestriert die Multi-Index-Speicherung, Transaktionsabwicklung (`DbTransaction`) und hybride Retrieval-Suche über LSM-Store, HNSW-Vector, BM25-Text und Wissensgraph. Bietet die zentrale `Collection`- und `Contextra`-Laufzeitumgebung sowie Hintergrundarbeiter für Expiry-, Orphan- und Hyperkanten-Wartung. Garantiert atomare 2-Phasen-Commits und bi-temporale Gültigkeitsfilterung.

## 2. Modul-Karte

| Datei / Verzeichnis | Verantwortung |
|---|---|
| `src/lib.rs` | `#![forbid(unsafe_code)]`, Re-Exports, `ContextraConfig`, `Contextra` Shell |
| `src/background_workers/` | `start_expiry_cleanup_worker()`, Orphan- und Hyperkanten-Hintergrundarbeiter |
| `src/collection/` | `Collection`, Key-Locking, CRUD-Operationen (`crud/`), Query-Builder (`query_builder/`), Suche (`search/`) |
| `src/contextra_impl/` | Multitenant Collection-Registry, Lifecycle, Wiring |
| `src/extraction/` | Rule-based & LLM OpenIE Entitätsextraktion (`OpenIE`, `ExtractedTriple`) |
| `src/transaction/` | `DbTransaction`, 2-Phasen-Commit, `CommitLedger` & Compensating Actions |
| `src/export.rs` / `import.rs` | JSON Export/Import Format v1 (`SCHEMA_VERSION_V1 = "1.0"`, `ExportDocumentV1`) |
| `src/temporal_filter.rs` | Bi-temporale Filterung (`ValidityWindow`, `apply_temporal_validity_filter`) |
| `src/filter.rs` | Metadaten-Prädikatfilterung (`MetadataFilter`, `FilterOp`) |
| `src/fusion.rs` | RRF & Score-normalisierte Signal-Fusion (`fuse_search_results_with_strategy`) |
| `src/chunker.rs` | `MarkdownChunker` für strukturierte Dokumentaufteilung |
| `src/decay_controller.rs` | `AdaptiveDecayController` für Time-Weighted Eviction |

## 3. Invarianten

- **ENG-026 Sperrenhierarchie:** `collections (RwLock)` -> `kv_locks (KvKeyLocks)` -> `embedder (RwLock)`.
- **INV-TTL-1 / L.15 Expiry:** Engine reapet Dokumente batchweise über `reap_expired_documents()`; Store filtert abgelaufene Einträge beim Lesen/Compaction.
- **L.8 Bi-Temporaler Filter:** Fail-Open für Altdaten ohne Metadaten (`ValidityWindow::is_empty()`); Schnittmenge aus System- und Geschäftszeit.
- **L.8 Hard-Scope:** Harte Dokumentgrenzen erzwingen ACORN-Traversierung über `ScopeConstraint`.
- **INV-FAIL-CLOSED:** Tenant-Isolierung (`TenantPolicy::Required`) und Multi-Index-Rollbacks schlagen bei Fehlern fail-closed fehl.

## 4. Verboten / Anti-Patterns

- **Keine Locks über `.await`:** MutexGuard/RwLockGuard niemals über Asynchronitätsgrenzen halten.
- **Keine direkten System-Zeitstempel als TxId:** Transaktions-IDs zwingend über `collection.allocate_tx()` oder `Contextra::allocate_tx()` beziehen.
- **Kein Nicht-deterministisches Scope-Set:** Scope-IDs müssen `BTreeSet<DocId>` nutzen.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

Reine Async-Laufzeitumgebung auf Layer 3 (Tokio). Zugriffe auf `Collection`-Ressourcen folgen strikt der Sperrenhierarchie (`collections` -> `kv_locks` -> `embedder`). `KvKeyLocks` sperrt Key-Granular gehasht; Multi-Key-Locks nutzen `lock_many_sorted` zur Deadlock-Vermeidung.

## 6. Verifikation

```bash
cargo test -p contextra-engine
cargo test -p contextra-engine --test multi_index_transaction
cargo test -p contextra-engine --test invariant_isolation
cargo test -p contextra-engine --test engine_commit_crash_points
cargo xtask check-agents-integrity
cargo xtask doctrine-scan --crate contextra-engine
cargo xtask check-unsafe-islands
cargo xtask wal-replay-verify
```

## 7. Bekannte Lücken / SOLL

- Testabdeckung für hypothetische 3-Node/Cluster-Crashes entfällt, da Single-Node-Ein-Prozess-Architektur gilt (v15 Teil F).
- Integrations- und Crash-Tests befinden sich in `crates/contextra-engine/tests/` (z. B. `write_skew_via_engine_api.rs`, `commit_crash_consistency.rs`).

