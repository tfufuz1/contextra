# Architecture Decision Records (ADR)

> **Hinweis:** Diese Datei ist ein generierter Index. Inhaltliche ADRs liegen ausschließlich unter `docs/decisions/`.

## Dokumentierte Lücken & Umnummerierungen

* ADR-057: Lücken-Dokumentation (Umnummerierung / Ausgelassen im Zuge paralleler Audit-Sessions)
* ADR-067: Umnummeriert zu ADR-074 (Normative Kalibrierung des PathRAG Sufficiency-Gate Thresholds)
* ADR-068: Umnummeriert zu ADR-076 (Studie zur DiskANN PENDING_FLUSH_THRESHOLD Write-Amplification)

---

## ADR Index

| Nummer | Titel | Datum | Link |
|---|---|---|---|
| **ADR-001** | LSM-Tree für Persistenz | 2026-05-10 | [ADR-001-lsm-tree-fuer-persistenz.md](docs/decisions/ADR-001-lsm-tree-fuer-persistenz.md) |
| **ADR-002** | HNSW für Vektor-Indexierung | 2026-05-15 | [ADR-002-hnsw-fuer-vektor-indexierung.md](docs/decisions/ADR-002-hnsw-fuer-vektor-indexierung.md) |
| **ADR-003** | RRF (Reciprocal Rank Fusion) für Hybridisierung | 2026-05-20 | [ADR-003-rrf-reciprocal-rank-fusion-fuer.md](docs/decisions/ADR-003-rrf-reciprocal-rank-fusion-fuer.md) |
| **ADR-004** | Sovereign Core (Pure Rust Policy) | 2026-06-01 | [ADR-004-sovereign-core-pure-rust-policy.md](docs/decisions/ADR-004-sovereign-core-pure-rust-policy.md) |
| **ADR-005** | Feature-Based Scaling | 2026-06-15 | [ADR-005-feature-based-scaling.md](docs/decisions/ADR-005-feature-based-scaling.md) |
| **ADR-006** | Eigenständige DECISIONS.md statt inline in SOURCE_OF_TRUTH.md | 2026-07-17 | [ADR-006-eigenstaendige-decisions-md-statt-inline.md](docs/decisions/ADR-006-eigenstaendige-decisions-md-statt-inline.md) |
| **ADR-007** | Produktstrategie — Lokale Agent-Memory-Library (Richtung C) [TEILWEISE ERSETZT durch ADR-018 bzgl. Vertriebskanal-Priorisierung, 2026-08-24] | 2026-07-19 | [ADR-007-produktstrategie-lokale-agent-memory.md](docs/decisions/ADR-007-produktstrategie-lokale-agent-memory.md) |
| **ADR-008** | Embedding-Backend — ONNX (memfuse-embed) → Ollama HTTP (memfuse-ollama) | 2026-08-22 | [ADR-008-embedding-backend-onnx-memfuse-embed.md](docs/decisions/ADR-008-embedding-backend-onnx-memfuse-embed.md) |
| **ADR-009** | Crate `memfuse-tauri` als Grundgerüst für Desktop-App ("MemFuse Brain") | 2026-07-20 | [ADR-009-crate-memfuse-tauri-als-grundgeruest.md](docs/decisions/ADR-009-crate-memfuse-tauri-als-grundgeruest.md) |
| **ADR-010** | MCP-Transport — HTTP-REST-Stub → stdio JSON-RPC 2.0 | 2026-08-23 | [ADR-010-mcp-transport-http-rest-stub-stdio-json.md](docs/decisions/ADR-010-mcp-transport-http-rest-stub-stdio-json.md) |
| **ADR-011** | Consolidate Checkpoint Subsystems (CheckpointCoordinator Trait) | 2026-08-23 | [ADR-011-consolidate-checkpoint-subsystems.md](docs/decisions/ADR-011-consolidate-checkpoint-subsystems.md) |
| **ADR-012** | Invarianten-Spannungsfeld — std::fs innerhalb spawn_blocking vs. Pure Async-I/O | 2026-08-23 | [ADR-012-invarianten-spannungsfeld-std-fs.md](docs/decisions/ADR-012-invarianten-spannungsfeld-std-fs.md) |
| **ADR-013** | Gestuftes Vektorindex-Modell — HNSW Default + DiskANN Tier (memfuse-index) | 2026-08-23 (Revidiert 2026-09-16 via ADR-083 / ADR §16.2) | [ADR-013-gestuftes-vektorindex-modell-hnsw.md](docs/decisions/ADR-013-gestuftes-vektorindex-modell-hnsw.md) |
| **ADR-014** | Regex-Engine-Wahl & ReDoS-Härtung für `run_regex_transformation` | 2026-08-24 | [ADR-014-regex-engine-wahl-redos-haertung-fuer.md](docs/decisions/ADR-014-regex-engine-wahl-redos-haertung-fuer.md) |
| **ADR-015** | RAII CheckpointGuard Integration & Konsolidierung in `memfuse-checkpoint` (AGT-CKPT-001 / AGT-STORE-002) | 2026-08-24 | [ADR-015-raii-checkpointguard-integration.md](docs/decisions/ADR-015-raii-checkpointguard-integration.md) |
| **ADR-016** | DocId 64-Bit BLAKE3-Trunkierung und Kollisionsschutz (BEFUND AGT-CORE-002) | 2026-08-25 | [ADR-016-docid-64-bit-blake3-trunkierung-und.md](docs/decisions/ADR-016-docid-64-bit-blake3-trunkierung-und.md) |
| **ADR-017** | Explicit Authorization of `unsafe` Mmap in DiskANN (BEFUND AGT-AUDIT-002) | 2026-08-24 | [ADR-017-explicit-authorization-of-unsafe-mmap-in.md](docs/decisions/ADR-017-explicit-authorization-of-unsafe-mmap-in.md) |
| **ADR-018** | Doppelstrategie — PyPI-Library UND Desktop-App (Auflösung ADR-007/ADR-009-Konflikt) | 2026-08-24 | [ADR-018-doppelstrategie-pypi-library-und-desktop.md](docs/decisions/ADR-018-doppelstrategie-pypi-library-und-desktop.md) |
| **ADR-019** | Contextual Retrieval via `combined_text_owned()` | 2026-08-25 | [ADR-019-contextual-retrieval-via-combined-text.md](docs/decisions/ADR-019-contextual-retrieval-via-combined-text.md) |
| **ADR-020** | Cognitive Operating System als Produktvision | 2026-08-27 | [ADR-020-cognitive-operating-system-als.md](docs/decisions/ADR-020-cognitive-operating-system-als.md) |
| **ADR-021** | Multi-Signal RAG-Pipeline (Contextual → RRF → Reranking) | 2026-08-27 | [ADR-021-multi-signal-rag-pipeline-contextual-rrf.md](docs/decisions/ADR-021-multi-signal-rag-pipeline-contextual-rrf.md) |
| **ADR-022** | Dokumenten-Entduplizierung & Single Responsibility Protocol | 2026-08-27 | [ADR-022-dokumenten-entduplizierung-single.md](docs/decisions/ADR-022-dokumenten-entduplizierung-single.md) |
| **ADR-023** | Kompensierende Transaktion für Multi-Store relate() Operations (F-01 / AGT-DB-005) | 2026-08-28 | [ADR-023-kompensierende-transaktion-fuer-multi.md](docs/decisions/ADR-023-kompensierende-transaktion-fuer-multi.md) |
| **ADR-024** | Snapshot-Isolation auf Storage- und Text-Signale beschränkt (Vektor/Graph nicht snapshot-isoliert) | 2026-08-28 | [ADR-024-snapshot-isolation-auf-storage-und-text.md](docs/decisions/ADR-024-snapshot-isolation-auf-storage-und-text.md) |
| **ADR-025** | Memory Importance Score & Recency-Decay als Post-Processing-Filter (Erweiterung ADR-021 & ADR-024) | 2026-08-28 | [ADR-025-memory-importance-score-recency-decay.md](docs/decisions/ADR-025-memory-importance-score-recency-decay.md) |
| **ADR-026** | Personalized PageRank (PPR) Graph Retrieval | 2026-08-28 | [ADR-026-personalized-pagerank-ppr-graph.md](docs/decisions/ADR-026-personalized-pagerank-ppr-graph.md) |
| **ADR-027** | Leiden-Algorithmus für Community Detection & GraphRAG (Revidiert) | 2026-08-27 (Revidiert 2026-09-16) | [ADR-027-community-detection.md](docs/decisions/ADR-027-community-detection.md) |
| **ADR-028** | Dezentrales Inline-Kontextsystem, Sekundengenaue Zeitstempel & Verpflichtendes Mehrfach-Session-Review | 2026-08-29 | [ADR-028-dezentrales-inline-kontextsystem.md](docs/decisions/ADR-028-dezentrales-inline-kontextsystem.md) |
| **ADR-029** | WAL-V3 Format & tx_id HMAC-Integritätskette | 2026-08-29 | [ADR-029-wal-v3-format-tx-id-hmac.md](docs/decisions/ADR-029-wal-v3-format-tx-id-hmac.md) |
| **ADR-030** | Pre-Commit-Hook für rustfmt & Workflow-Automatisierung | 2026-08-29 | [ADR-030-pre-commit-hook-fuer-rustfmt-workflow.md](docs/decisions/ADR-030-pre-commit-hook-fuer-rustfmt-workflow.md) |
| **ADR-031** | Realistic-Scale Benchmark Suite & Semantische Retrieval-Evaluierung | 2026-08-29 | [ADR-031-realistic-scale-benchmark-suite.md](docs/decisions/ADR-031-realistic-scale-benchmark-suite.md) |
| **ADR-032** | Async LLM-Summarization & Provenance Tracking in ContextCompactor (ID: AGT-DB-004) | 2026-08-28 | [ADR-032-async-llm-summarization-provenance.md](docs/decisions/ADR-032-async-llm-summarization-provenance.md) |
| **ADR-033** | Bi-temporale Zeitachsen (Validitätszeit + Transaktionszeit) im Wissensgraphen (Phase 2 Roadmap) | 2026-08-28 | [ADR-033-bi-temporale-zeitachsen-validitaetszeit.md](docs/decisions/ADR-033-bi-temporale-zeitachsen-validitaetszeit.md) |
| **ADR-034** | Runtime-Precondition Assertions in öffentlichen Low-Level-Distanzfunktionen (`memfuse-index`) | 2026-08-28 | [ADR-034-runtime-precondition-assertions-in.md](docs/decisions/ADR-034-runtime-precondition-assertions-in.md) |
| **ADR-035** | Governance-System-Härtung — Prozessregeln gegen wiederkehrende Trait-Default-, Typ-Dopplungs- und Stale-Finding-Fehler | 2026-08-28 | [ADR-035-governance-system-haertung-prozessregeln.md](docs/decisions/ADR-035-governance-system-haertung-prozessregeln.md) |
| **ADR-036** | unsafe-Scope-Erweiterung für test-only crypto anti_tamper | 2026-08-29 | [ADR-036-unsafe-scope-erweiterung-fuer-test-only.md](docs/decisions/ADR-036-unsafe-scope-erweiterung-fuer-test-only.md) |
| **ADR-037** | VectorIndex-Generalisierung in Collection<S, V> | 2026-08-29 | [ADR-037-vectorindex-generalisierung-in.md](docs/decisions/ADR-037-vectorindex-generalisierung-in.md) |
| **ADR-038** | Zettelkasten Memory Links (A-MEM) & Supersedes Displacement Logic | 2026-08-29 | [ADR-038-zettelkasten-memory-links-a-mem.md](docs/decisions/ADR-038-zettelkasten-memory-links-a-mem.md) |
| **ADR-039** | reqwest als Workspace-Dependency für memfuse-router | 2026-08-29 | [ADR-039-reqwest-als-workspace-dependency-fuer.md](docs/decisions/ADR-039-reqwest-als-workspace-dependency-fuer.md) |
| **ADR-040** | collection.rs Modularisierung (God Object Auflösung) <!-- doc-ref-ignore --> | 2026-08-29 | [ADR-040-collection-rs-modularisierung-god-object.md](docs/decisions/ADR-040-collection-rs-modularisierung-god-object.md) |
| **ADR-041** | TOMBSTONE_BIT-Disziplin in Sequenznummer-Berechnungen und rollback_to_tx | 2026-08-29 | [ADR-041-tombstone-bit-disziplin-in-sequenznummer.md](docs/decisions/ADR-041-tombstone-bit-disziplin-in-sequenznummer.md) |
| **ADR-042** | Re-Integration von `memfuse-saos-agent` | 2026-08-29 | [ADR-042-re-integration-von-memfuse-saos-agent.md](docs/decisions/ADR-042-re-integration-von-memfuse-saos-agent.md) |
| **ADR-043** | Aktualisierung von `last_committed_tx` vor der Sichtbarmachung von SSTables in `LsmStorage::flush` | 2026-08-29 | [ADR-043-aktualisierung-von-last-committed-tx-vor.md](docs/decisions/ADR-043-aktualisierung-von-last-committed-tx-vor.md) |
| **ADR-044** | MCP Write-Authorization & Sandbox Policy (Default Read-Only) | 2026-08-30 | [ADR-044-mcp-write-authorization-sandbox-policy.md](docs/decisions/ADR-044-mcp-write-authorization-sandbox-policy.md) |
| **ADR-045** | Entkopplung von `memfuse-router` und `memfuse-mcp` durch IPC JSON-RPC Typverschiebung | 2026-08-31 | [ADR-045-entkopplung-von-memfuse-router-und.md](docs/decisions/ADR-045-entkopplung-von-memfuse-router-und.md) |
| **ADR-046** | Wiederherstellung von `memfuse-agent` aus dem Archiv | - | [ADR-046-wiederherstellung-von-memfuse-agent-aus.md](docs/decisions/ADR-046-wiederherstellung-von-memfuse-agent-aus.md) |
| **ADR-047** | SIMD-Implementierungsstrategie — std::arch vs portable_simd (AGT-INDEX-002) | 2026-09-03 | [ADR-047-simd-implementierungsstrategie-std-arch.md](docs/decisions/ADR-047-simd-implementierungsstrategie-std-arch.md) |
| **ADR-048** | WAL Legacy-Key Feature-Gating & Downgrade Protection | 2026-09-03 | [ADR-048-wal-legacy-key-feature-gating-downgrade.md](docs/decisions/ADR-048-wal-legacy-key-feature-gating-downgrade.md) |
| **ADR-049** | Audit-Log Append-Only Enforcement via `put_kv_if_absent` | 2026-09-03 | [ADR-049-audit-log-append-only-enforcement-via.md](docs/decisions/ADR-049-audit-log-append-only-enforcement-via.md) |
| **ADR-050** | Router Single-Conformal Calibration & Lock Scope Consolidation | 2026-09-03 | [ADR-050-router-single-conformal-calibration-lock.md](docs/decisions/ADR-050-router-single-conformal-calibration-lock.md) |
| **ADR-051** | Context Compaction Delete Error Propagation | 2026-09-03 | [ADR-051-context-compaction-delete-error.md](docs/decisions/ADR-051-context-compaction-delete-error.md) |
| **ADR-052** | Synchronous PinGuard Drop Orphan Registration | 2026-09-03 | [ADR-052-synchronous-pinguard-drop-orphan.md](docs/decisions/ADR-052-synchronous-pinguard-drop-orphan.md) |
| **ADR-053** | Instance-Scoped Orphan State in PersistentCheckpointStore | 2026-09-03 | [ADR-053-instance-scoped-orphan-state-in.md](docs/decisions/ADR-053-instance-scoped-orphan-state-in.md) |
| **ADR-054** | Unified Router Scoring & TOCTOU-Safe Calibration Scope | 2026-09-03 | [ADR-054-unified-router-scoring-toctou-safe.md](docs/decisions/ADR-054-unified-router-scoring-toctou-safe.md) |
| **ADR-055** | WAL Legacy Key Fallback Protection | 2026-09-03 | [ADR-055-wal-legacy-key-fallback-protection.md](docs/decisions/ADR-055-wal-legacy-key-fallback-protection.md) |
| **ADR-056** | Python FFI Panic Isolation via PyErr Exception Mapping | 2026-09-03 | [ADR-056-python-ffi-panic-isolation-via-pyerr.md](docs/decisions/ADR-056-python-ffi-panic-isolation-via-pyerr.md) |
| **ADR-057** | Lücken-Dokumentation (Umnummerierung / Ausgelassen) | 2026-09-04 | [ADR-057-luecken-dokumentation-umnummerierung.md](docs/decisions/ADR-057-luecken-dokumentation-umnummerierung.md) |
| **ADR-058** | Error-Logging-Pattern für synchrones Orphan-State Persistieren in Checkpoint | 2026-09-04 | [ADR-058-error-logging-pattern-fuer-synchrones.md](docs/decisions/ADR-058-error-logging-pattern-fuer-synchrones.md) |
| **ADR-059** | Python FFI Panic Isolation (ehemals docs/decisions/ADR-048) | 2026-09-03 | [ADR-059-python-ffi-panic-isolation-ehemals-docs.md](docs/decisions/ADR-059-python-ffi-panic-isolation-ehemals-docs.md) |
| **ADR-060** | ADR-Governance — Konsolidierung auf DECISIONS.md als Einzel-Quelle | 2026-09-04 | [ADR-060-adr-governance-konsolidierung-auf.md](docs/decisions/ADR-060-adr-governance-konsolidierung-auf.md) |
| **ADR-061** | 2-Phasen-Lock für HNSW Rebuild | 2026-09-04 | [ADR-061-2-phasen-lock-fuer-hnsw-rebuild.md](docs/decisions/ADR-061-2-phasen-lock-fuer-hnsw-rebuild.md) |
| **ADR-062** | Fault-Injection-Testsuite für WAL V3/MVCC (adaptiert aus chimeraDB SPEC-035) | 2026-09-05 | [ADR-062-fault-injection-testsuite-fuer-wal-v3.md](docs/decisions/ADR-062-fault-injection-testsuite-fuer-wal-v3.md) |
| **ADR-063** | F-02 Nucleation — Tombstone-Pruning-Variante vs. ursprüngliches Rebuild-Veto | 2026-09-07 | [ADR-063-f-02-nucleation-tombstone-pruning.md](docs/decisions/ADR-063-f-02-nucleation-tombstone-pruning.md) |
| **ADR-064** | memfuse-py als separater Cargo-Workspace (Panic-Strategie-Isolation) | 2026-09-07 | [ADR-064-memfuse-py-als-separater-cargo-workspace.md](docs/decisions/ADR-064-memfuse-py-als-separater-cargo-workspace.md) |
| **ADR-065** | Duplicate Symbol CI-Gate zur Prävention von Merge-Kollisionen | - | [ADR-065-duplicate-symbol-ci-gate-zur-praevention.md](docs/decisions/ADR-065-duplicate-symbol-ci-gate-zur-praevention.md) |
| **ADR-066** | Activation of Feature Flag physio-resonance-fusion for Feature F-09 | - | [ADR-066-activation-of-feature-flag-physio.md](docs/decisions/ADR-066-activation-of-feature-flag-physio.md) |
| **ADR-069** | Standard-Terminologie statt biologischer Metaphern und Anbieter-Branding | - | [ADR-069-standard-terminologie-statt-biologischer.md](docs/decisions/ADR-069-standard-terminologie-statt-biologischer.md) |
| **ADR-070** | F-02 Scope-Abgrenzung — Reines Tombstone-Pruning vs. Ursprüngliches Veto | - | [ADR-070-f-02-scope-abgrenzung-reines-tombstone.md](docs/decisions/ADR-070-f-02-scope-abgrenzung-reines-tombstone.md) |
| **ADR-071** | Additive Härtung der TenantId-Konstruktoren zur Erzwingung von INV-TENANT-1 | - | [ADR-071-additive-haertung-der-tenantid.md](docs/decisions/ADR-071-additive-haertung-der-tenantid.md) |
| **ADR-072** | KV-Bridge Increment 2 — KvSegment-Verschlüsselung, ModelFingerprint und RoPE-Offset (K14) | - | [ADR-072-kv-bridge-increment-2-kvsegment.md](docs/decisions/ADR-072-kv-bridge-increment-2-kvsegment.md) |
| **ADR-073** | GASP Post-Hoc Halluzinations-Validator (Initiale Implementierung K19) | - | [ADR-073-gasp-post-hoc-halluzinations-validator.md](docs/decisions/ADR-073-gasp-post-hoc-halluzinations-validator.md) |
| **ADR-074** | Normative Kalibrierung des PathRAG Sufficiency-Gate Thresholds | - | [ADR-074-normative-kalibrierung-des-pathrag.md](docs/decisions/ADR-074-normative-kalibrierung-des-pathrag.md) |
| **ADR-075** | PID-Regler min_pool_size Kalibrierung und Default-Konsolidierung | - | [ADR-075-pid-regler-min-pool-size-kalibrierung.md](docs/decisions/ADR-075-pid-regler-min-pool-size-kalibrierung.md) |
| **ADR-076** | Studie zur DiskANN PENDING_FLUSH_THRESHOLD Write-Amplification und Empfehlung für adaptiven Schwellenwert | - | [ADR-076-studie-zur-diskann-pending-flush.md](docs/decisions/ADR-076-studie-zur-diskann-pending-flush.md) |
| **ADR-077** | Produktvision PyPI-Library Fokus und Tauri Deprecation | - | [ADR-077-produktvision-pypi-library-fokus-und.md](docs/decisions/ADR-077-produktvision-pypi-library-fokus-und.md) |
| **ADR-078** | Konsolidierung aller NEW_STRATEGY-Dokumente in GESAMTSPEZIFIKATION_v10.0 | - | [ADR-078-konsolidierung-aller-new-strategy.md](docs/decisions/ADR-078-konsolidierung-aller-new-strategy.md) |
| **ADR-079** | Event-getriebene Drift-Erkennung (F-11) im Router statt im periodischen Maintenance-Tick | - | [ADR-079-event-getriebene-drift-erkennung-f-11-im.md](docs/decisions/ADR-079-event-getriebene-drift-erkennung-f-11-im.md) |
| **ADR-080** | Weak<RouterEngine>-Injection in MemFuse für H-17 Stats Live-Daten | - | [ADR-080-weak-routerengine-injection-in-memfuse.md](docs/decisions/ADR-080-weak-routerengine-injection-in-memfuse.md) |
| **ADR-081** | gegenseitiger Ausschluss von MaintenanceScheduler und ConsolidationEngine | - | [ADR-081-gegenseitiger-ausschluss-von.md](docs/decisions/ADR-081-gegenseitiger-ausschluss-von.md) |
| **ADR-082** | Gestufte Architekturentscheidungen Endprodukt v9 | - | [ADR-082-docid-width.md](docs/decisions/ADR-082-docid-width.md) |
| **ADR-084** | FlatBuffers Schema Drift Gate (IP-16) & Dead Types Recommendation | - | [ADR-084-flatbuffers-drift-gate.md](docs/decisions/ADR-084-flatbuffers-drift-gate.md) |
| **ADR-N01** | Maschinelle Durchsetzung der Layer-Regeln (DAG & Ring-Schichtenmodell) | - | [ADR-N01-layer-regeln-maschinell.md](docs/decisions/ADR-N01-layer-regeln-maschinell.md) |
| **ADR-N02** | Sync-Kern — StorageRead synchron, StorageWrite asynchron (begrenzter ComputePool) | - | [ADR-N02-sync-kern-storageread-storagewrite.md](docs/decisions/ADR-N02-sync-kern-storageread-storagewrite.md) |
| **ADR-N03** | Drei erlaubte Unsafe-Inseln, Deny/Forbid-Mechanik und Unsafe-Transition-Tracking | - | [ADR-N03-drei-unsafe-inseln.md](docs/decisions/ADR-N03-drei-unsafe-inseln.md) |
| **ADR-N04** | Panic-Profilierung — Workspace unwind vs. Release-Abort & FFI Panic-Isolation | - | [ADR-N04-panic-profile.md](docs/decisions/ADR-N04-panic-profile.md) |
| **ADR-N05** | Dokumenten-Identifikatoren — Externes DocId(128-Bit/Key) vs. Internes DocIdx(u32) | - | [ADR-N05-docid-identifikatoren.md](docs/decisions/ADR-N05-docid-identifikatoren.md) |
| **ADR-N06** | Konsistenzmodell — WAL als einzige Wahrheit vs. 2PC-Härtung | - | [ADR-N06-konsistenzmodell-wal-als-wahrheit.md](docs/decisions/ADR-N06-konsistenzmodell-wal-als-wahrheit.md) |
| **ADR-N07** | KV-Cache-Stufen A, B und C (Prefix-Reuse, KvState-Modellierung, Verschlüsselte Segmentdateien) | - | [ADR-N07-kv-stufen-a-b-c.md](docs/decisions/ADR-N07-kv-stufen-a-b-c.md) |
| **ADR-N08** | Generierte Spezifikation und Capability-Manifest (`capabilities.toml`) | - | [ADR-N08-generierte-spec-capability-manifest.md](docs/decisions/ADR-N08-generierte-spec-capability-manifest.md) |
| **ADR-N09** | Crate-Schnittkriterien I/U/C/S/D | - | [ADR-N09-crate-schnittkriterien-iucs.md](docs/decisions/ADR-N09-crate-schnittkriterien-iucs.md) |
| **ADR-N10** | Kein globaler veränderlicher Zustand | - | [ADR-N10-kein-globaler-veraenderlicher-zustand.md](docs/decisions/ADR-N10-kein-globaler-veraenderlicher-zustand.md) |
| **ADR-0XX** | memfuse-sandbox — WASM Execution Boundary | - | [ADR-0XX-memfuse-sandbox.md](docs/decisions/ADR-0XX-memfuse-sandbox.md) |
