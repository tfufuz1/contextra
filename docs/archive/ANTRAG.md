# ANTRAG: Dokumenteninventar und Archivierungsvorschlag (`docs/archive/ANTRAG.md`)

**Karte:** `W4-04` | **Produkt:** `EP1` | **Meilenstein:** `K2` | **Label:** `scope:decouple` | **ADR-Bezug:** `ADR-PARK-001`
**Datum:** 15. Oktober 2026
**Gesamtzahl erfasster Dokumente:** 355

## 1. Übersicht

Dieses Dokument enthält die geforderte Aufstellung aller Dokumente unter `docs/` (ausgenommen `docs/decisions/`, `docs/verification/`, `docs/archive/` und `docs/unwrap_baseline_history.jsonl`).
Für jedes Dokument wird der relative Pfad, alle Verweise aus den Crates (`crates/`) ermittelt via `rg -l`, ein Vorschlag (`archivieren`, `behalten` oder `unklar`) sowie eine kurze Begründung aufgeführt.

## 2. Dokumentenliste

| Pfad | Verweise aus Crates (`rg -l`) | Vorschlag | Begründung |
| --- | --- | --- | --- |
| `docs/ARCHITECTURE.md` | `crates/contextra-adapt/README.md`, `<br>`crates/contextra-agent/README.md`, `<br>`crates/contextra-checkpoint/README.md`, `<br>`crates/contextra-core/README.md`, `<br>`crates/contextra-crypto/README.md`, `<br>`crates/contextra-db/README.md`, `<br>`crates/contextra-graph/README.md`, `<br>`crates/contextra-mcp/README.md`, `<br>`crates/contextra-mvcc/README.md`, `<br>`crates/contextra-ports/README.md`, `<br>`crates/contextra-router/README.md`, `<br>`crates/contextra-simd/README.md`, `<br>`crates/contextra-store/README.md`, `<br>`crates/contextra-sys/README.md`, `<br>`crates/contextra-testkit/README.md`, `<br>`crates/contextra-text/README.md`, `<br>`crates/contextra-types/README.md`, `<br>`crates/contextra-vector/README.md`, `<br>`crates/contextra-wire/README.md`, `<br>`crates/contextra/README.md` | **behalten** | Aktive System- / Architektur-Spezifikation |
| `docs/BENCH-LOG.md` | Keine | **archivieren** | Historisches Log / Forschungsbericht / Alt-Spezifikation |
| `docs/BENCHMARKS.md` | Keine | **archivieren** | Historisches Log / Forschungsbericht / Alt-Spezifikation |
| `docs/BENCHMARKS_DELETION_PROOF.md` | Keine | **archivieren** | Historisches Log / Forschungsbericht / Alt-Spezifikation |
| `docs/BENCHMARKS_ROUTING_COST.md` | Keine | **archivieren** | Historisches Log / Forschungsbericht / Alt-Spezifikation |
| `docs/BENCHMARK_PROTOCOL.md` | Keine | **behalten** | Aktive System- / Architektur-Spezifikation |
| `docs/CHANGELOG.md` | Keine | **archivieren** | Historisches Log / Forschungsbericht / Alt-Spezifikation |
| `docs/CONTEXTRA_SOTA_FORSCHUNGSBERICHT_2025_2026.md` | Keine | **archivieren** | Historisches Log / Forschungsbericht / Alt-Spezifikation |
| `docs/EP1_CONTEXTRA_MEMORY_SPEC.md` | Keine | **archivieren** | Historische Einzel-Spezifikation (in v4 konsolidiert) |
| `docs/EP2_CONTEXTRA_PROOF_SPEC.md` | Keine | **archivieren** | Historische Einzel-Spezifikation (in v4 konsolidiert) |
| `docs/EP3_CONTEXTRA_RETRIEVAL_PLUS_SPEC.md` | Keine | **archivieren** | Historische Einzel-Spezifikation (in v4 konsolidiert) |
| `docs/EP4_CONTEXTRA_INFERENCE_SPEC.md` | Keine | **archivieren** | Historische Einzel-Spezifikation (in v4 konsolidiert) |
| `docs/EP5_CONTEXTRA_COGNITION_ROUTER_SPEC.md` | Keine | **archivieren** | Historische Einzel-Spezifikation (in v4 konsolidiert) |
| `docs/EP6_CONTEXTRA_GUARD_SANDBOX_SPEC.md` | Keine | **archivieren** | Historische Einzel-Spezifikation (in v4 konsolidiert) |
| `docs/EXPORT_FORMAT.md` | Keine | **behalten** | Aktive System- / Architektur-Spezifikation |
| `docs/FEATURES_SPECIFICATION.md` | Keine | **archivieren** | Historisches Log / Forschungsbericht / Alt-Spezifikation |
| `docs/GESAMTSPEZIFIKATION.md` | Keine | **behalten** | Aktive System- / Architektur-Spezifikation |
| `docs/QUALITAETSPRUEFUNG_COMMIT_FUNKTIONEN_SPEZIFIKATION.md` | Keine | **archivieren** | Historisches Log / Forschungsbericht / Alt-Spezifikation |
| `docs/SOLO_DEVELOPER_GUIDE.md` | Keine | **behalten** | Aktive System- / Architektur-Spezifikation |
| `docs/SOURCE_OF_TRUTH.md` | Keine | **behalten** | Aktive System- / Architektur-Spezifikation |
| `docs/TYPE_REGISTRY.md` | `crates/contextra-types/AGENTS.md` | **behalten** | Aktive System- / Architektur-Spezifikation |
| `docs/analysis/AUDIT_GESAMTSYNTHESE_2026-10-14.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_benchmarks_DEEP_2026-10-13.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_collection-cognition-router-agent-db_DEEP_2026-10-13.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_composition-root_DEEP_2026-10-13.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_contextra-crypto_DEEP_2026-10-13.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_contextra-deep_DEEP_2026-10-13.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_contextra-mvcc_DEEP_2026-10-13.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_contextra-mvcc_DEEP_2026-10-14.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_contextra-sandbox_DEEP_2026-10-13.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_contextra-store_DEEP_2026-10-13.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_contextra-store_DEEP_2026-10-14.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_contextra-vector_DEEP_2026-10-13.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_contextra-vector_DEEP_2026-10-14.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_governance_harness_DEEP_2026-10-13.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_mcp_py_facade_DEEP_2026-10-13.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_privacy-license-compliance_DEEP_2026-10-13.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_testkit-fuzz-loom-governance_DEEP_2026-10-13.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_text-graph-rank_DEEP_2026-10-13.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_unsafe-islands_DEEP_2026-10-13.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_unsafe-islands_DEEP_2026-10-14.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/AUDIT_wal_durable_fs_DEEP_2026-10-13.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/analysis/MCP_PY_FACADE_AUDIT_REPORT.md` | Keine | **archivieren** | Historischer Analyse- / Audit-Synthesebericht |
| `docs/architecture/dependency-graph.mmd` | Keine | **behalten** | Aktive Architektur-Diagramme und Schemata |
| `docs/architecture/kv-bridge.md` | Keine | **behalten** | Aktive Architektur-Diagramme und Schemata |
| `docs/audit/GOOGLE_JULES_DIAGNOSTICS_AND_OPTIMIZATIONS.md` | Keine | **archivieren** | Historischer Audit-Leitfaden |
| `docs/audits/AUDIT_adaptive-controller_DEEP_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_bm25_DEEP_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_compaction_DEEP_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-adapt_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-adapt_2026-10-04.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-adapt_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-agent_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-audit-export_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-avv-generator_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-checkpoint_2026-10-04.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-cognition_2026-03-06.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-cognition_2026-10-11.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-core_2026-10-04.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-core_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-crypto_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-crypto_2026-10-04.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-crypto_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-db_2026-10-04.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-db_2026-10-11.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-db_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-engine_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-facade_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-graph_2026-10-04.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-graph_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-infer-candle_2026-10-04.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-infer-candle_2026-10-11.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-infer-candle_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-infer-ollama_2026-10-04.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-infer-ollama_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-infer-onnx_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-kvcache_2026-10-04.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-kvcache_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-license_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-mcp_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-mvcc_2026-10-04.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-mvcc_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-ports_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-ports_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-privacy_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-py_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-rank_2026-10-04.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-rank_2026-10-11.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-rank_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-router_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-sandbox_2026-10-04.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-sandbox_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-simd_2026-10-04.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-simd_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-store_2026-10-04.md` | `crates/contextra-store/src/lib.rs` | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-store_2026-10-11.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-store_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-sys_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-testkit_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-text_2026-10-04.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-text_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-types_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-types_2026-10-04.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-types_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-vector_2026-10-04.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-vector_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-vector_P03_2026-10-06.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-wire_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-wire_2026-10-04.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_contextra-wire_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_crypto-kernel_DEEP_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_csr-ppr_DEEP_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_deletion_proof_verification_2026-10-07.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_fuzz-campaign_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_hnsw_DEEP_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_kdf-migration-caller-inventory_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_kvcache_DEEP_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_lsm-core_DEEP_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_mutation-testing_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_release-gate_v0.1.0_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_ring-dag-integrity_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_simd-kernels_DEEP_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_soll-ist-gap_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_sstable_DEEP_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_wal-subsystem_DEEP_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/AUDIT_zero-panic-unsafe_WORKSPACE_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/MANIFEST_RANK_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/MUTANTS_crypto_2026-10-12.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/P02_LSM_Tree_Korrektheitspruefung.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/P05_unsafe_mmap_wasm_isolation_review.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/RA1-kv-cache-kern.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/adaptive-controller_DEEP_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/advisory-reachability-2026-10.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/bm25_DEEP_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/chaos-matrix_AUDIT_2026-09-28.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/compaction_DEEP_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-adapt_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-agent_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-audit-export_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-avv-generator_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-checkpoint_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-cognition_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-core_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-crypto_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-db_AUDIT_2026-03-30.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-engine_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-facade_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-graph_AUDIT_2026-03-30.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-infer-candle_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-infer-ollama_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-infer-onnx_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-kvcache_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-mcp_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-mcp_AUDIT_2026-09-28.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-mvcc_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-ports_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-privacy_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-py_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-rank_AUDIT_2026-03-31.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-router_AUDIT_2026-09-17.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-sandbox_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-simd_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-store_AUDIT_2026-08-30.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-sys_AUDIT_2026-09-27.md` | `crates/contextra-sys/src/acl_win32.rs`, `<br>`crates/contextra-sys/src/vault.rs` | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-testkit_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-text_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-types_AUDIT_2026-03-30.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-vector_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/contextra-wire_AUDIT_2026-09-27.md` | `crates/contextra-wire/src/adapter.rs`, `<br>`crates/contextra-wire/src/jsonrpc.rs`, `<br>`crates/contextra-wire/src/lib.rs` | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/crypto-kernel_DEEP_AUDIT_2026-09-28.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/crypto-wal-audit.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/csr-ppr_DEEP_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/deletion-claims-inventory-2026-10.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/deletion-proof-coverage-semantics-2026-10.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/drop-collection-race-2026-10.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/fuzz-campaign_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/gate-gap-4494-2026-10.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/hnsw-slots-entrypoint-2026-10.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/kvcache_DEEP_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/loom-coverage-2026-10.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/lsm-core_DEEP_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/mvcc-audit.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/physical-scan-feasibility-2026-10.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/qa-soundness-loom-miri_AUDIT_2026-09-28.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/release-gate_v0.1.0_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/ring-dag-integrity_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/simd-kernels_DEEP_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/soll-ist-gap_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/store-manifest-compaction-audit.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/store-readpath-audit.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/store-sstable-audit.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/wal-subsystem_DEEP_AUDIT_2026-09-27.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/audits/wal-v1-tail-2026-10.md` | Keine | **archivieren** | Historischer Audit-Bericht |
| `docs/baseline/2026-10.md` | Keine | **archivieren** | Historische Baseline-Messung |
| `docs/bsi-grundschutz-mapping.md` | `crates/contextra-audit-export/src/bsi_mapping.rs` | **behalten** | Aktive System- / Architektur-Spezifikation |
| `docs/ci/BRANCH_PROTECTION_CHECKLIST.md` | Keine | **behalten** | Aktive CI- / Branch-Protection Richtlinien |
| `docs/ci/VETO_DEADLINE_GATE.md` | Keine | **behalten** | Aktive CI- / Branch-Protection Richtlinien |
| `docs/ci/cargo-audit.md` | Keine | **behalten** | Aktive CI- / Branch-Protection Richtlinien |
| `docs/ci/fuzzing.md` | Keine | **behalten** | Aktive CI- / Branch-Protection Richtlinien |
| `docs/compliance/BSI_TR02102_MAPPING.md` | Keine | **behalten** | Aktive Compliance- / BSI TR-02102 Mapping Dokumentation |
| `docs/compliance/README.md` | `crates/contextra-adapt/README.md`, `<br>`crates/contextra-agent/README.md`, `<br>`crates/contextra-checkpoint/README.md`, `<br>`crates/contextra-core/README.md`, `<br>`crates/contextra-crypto/README.md`, `<br>`crates/contextra-db/README.md`, `<br>`crates/contextra-graph/README.md`, `<br>`crates/contextra-mcp/README.md`, `<br>`crates/contextra-mcp/pyproject.toml`, `<br>`crates/contextra-mvcc/README.md`, `<br>`crates/contextra-ports/README.md`, `<br>`crates/contextra-router/README.md`, `<br>`crates/contextra-simd/README.md`, `<br>`crates/contextra-store/README.md`, `<br>`crates/contextra-testkit/README.md`, `<br>`crates/contextra-text/README.md`, `<br>`crates/contextra-types/README.md`, `<br>`crates/contextra-vector/README.md`, `<br>`crates/contextra-wire/README.md`, `<br>`crates/contextra/README.md` | **behalten** | Aktive Compliance- / BSI TR-02102 Mapping Dokumentation |
| `docs/deletion/CANARY_ORACLE.md` | Keine | **behalten** | Aktive Erasure- / Deletion-Spezifikation und Testvektoren |
| `docs/deletion/KEYFAMILIES.toml` | `crates/contextra-db/tests/p08_canary_oracle.rs`, `<br>`crates/contextra-engine/src/deletion/registry.rs`, `<br>`crates/contextra-engine/tests/key_family_registry.rs` | **behalten** | Aktiv vom Engine-Integrationstest via include_str! verwendet |
| `docs/deletion/LEGAL_MATRIX_ERASURE.md` | Keine | **behalten** | Aktive Erasure- / Deletion-Spezifikation und Testvektoren |
| `docs/deletion/OPERATIONS_BASELINE_ERASURE.md` | Keine | **behalten** | Aktive Erasure- / Deletion-Spezifikation und Testvektoren |
| `docs/deletion/POLICY_RULES.md` | Keine | **behalten** | Aktive Erasure- / Deletion-Spezifikation und Testvektoren |
| `docs/deletion/vectors/v4_proof_sample.json` | Keine | **behalten** | Aktive Erasure- / Deletion-Spezifikation und Testvektoren |
| `docs/explain/check-commit-diff-integrity.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/check-dag.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/check-duplicate-symbols.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/check-mutation-score-gate.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/check-phantom-files.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/check-ring-layering.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/check-unsafe-islands.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/check-vetoes.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/claims-in-pr.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/debt-audit.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/determinism-check.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/diff-budget.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/doc-truth.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/env-attest.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/fuzz-smoke.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/gate-weakening.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/ledger.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/lock-audit.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/loop-guard.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/mutants-diff.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/plan-lint.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/protected-paths.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/ratchet.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/registry-check.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/scope-guard.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/symbol-exists.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/sync-docs.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/task-card.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/test-integrity.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/unsafe-audit.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/verdict.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/explain/wal-replay-verify.toml` | Keine | **behalten** | Aktiv von Governance-Engine / Gate-Guard verwendet |
| `docs/gaps/pub_fn_ohne_aufrufer.csv` | Keine | **archivieren** | Historischer Lücken-Analyse-Export |
| `docs/gaps/undokumentiert_streng.csv` | Keine | **archivieren** | Historischer Lücken-Analyse-Export |
| `docs/gates/blast-radius.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/can-merge.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/claims-in-pr.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/codeowners-coverage.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/dependency-gate.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/determinism-check.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/diagnostic-artifact-generator.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/diff-budget.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/env-attest.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/explain.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/flake-report.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/fuzz-smoke.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/gate-arch.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/gate-canary.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/gate-guard.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/gate-integrity.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/gate-orchestrator.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/gate-weakening.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/gates-catalog-sync.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/gov-d-cross-file-duplicates.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/harness-foundation.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/harness-state.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/jules-dispatch.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/jules-facade.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/ledger.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/lock-audit.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/loop-guard.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/marker-generator.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/mutants-diff.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/mutation-score-enforcement.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/phantom-commit-gate.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/plan-lint.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/postmortem.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/protected-paths.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/ratchet.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/registry-check.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/risk.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/scope-guard.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/session-report.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/symbol-exists.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/task-card.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/test-integrity.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/unsafe-audit.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/unwrap-ratchet-removal-status.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/verdict.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/verify.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/wal-replay-verify.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/gates/xtask-loc-reduction-audit.md` | Keine | **behalten** | Aktiv von xtask Gate-Guard / Governance-System verwendet |
| `docs/generated/capability-markers.md` | Keine | **behalten** | Aktiver generierter Merkmalskatalog / Marker |
| `docs/generated/feature-catalog.md` | Keine | **behalten** | Aktiver generierter Merkmalskatalog / Marker |
| `docs/mcp/TOOL_SEMANTICS.md` | Keine | **behalten** | Aktive MCP Tool-Semantik Spezifikation |
| `docs/mutation_score_history.jsonl` | Keine | **archivieren** | Historischer Mutation-Score Verlauf |
| `docs/parking/ENTFERNUNG.md` | Keine | **behalten** | Aktives Entkopplungs- / Parkplatz-Inventar und Regelwerk |
| `docs/parking/INVENTORY.md` | Keine | **behalten** | Aktives Entkopplungs- / Parkplatz-Inventar und Regelwerk |
| `docs/postmortems/PM-0001-commit-integrity-guard.md` | Keine | **archivieren** | Historischer Incident- / Postmortem-Bericht |
| `docs/postmortems/_TEMPLATE.md` | Keine | **behalten** | Postmortem-Vorlage |
| `docs/refactor/core-mapping.json` | Keine | **archivieren** | Historisches Refactoring-Protokoll / Entscheidung |
| `docs/refactor/core-mapping.md` | Keine | **archivieren** | Historisches Refactoring-Protokoll / Entscheidung |
| `docs/refactor/license-binding-decision.md` | Keine | **archivieren** | Historisches Refactoring-Protokoll / Entscheidung |
| `docs/refactor/wasm-merge-operator-plan.md` | Keine | **archivieren** | Historisches Refactoring-Protokoll / Entscheidung |
| `docs/scope/CORE_SCOPE.md` | Keine | **behalten** | Aktive Scope- & Ring-Debt Verträge |
| `docs/scope/PLATFORM_SCOPE.md` | Keine | **behalten** | Aktive Scope- & Ring-Debt Verträge |
| `docs/scope/RING_DEBT.md` | Keine | **behalten** | Aktive Scope- & Ring-Debt Verträge |
| `docs/scope/ROBUSTHEITSVERTRAG.md` | Keine | **behalten** | Aktive Scope- & Ring-Debt Verträge |
| `docs/spec-sync/C-01.md` | Keine | **archivieren** | Historisches Spec-Sync Stabilisierungsprotokoll |
| `docs/spec-sync/E-01.md` | Keine | **archivieren** | Historisches Spec-Sync Stabilisierungsprotokoll |
| `docs/spec-sync/G-01.md` | Keine | **archivieren** | Historisches Spec-Sync Stabilisierungsprotokoll |
| `docs/spec-sync/K-01.md` | Keine | **archivieren** | Historisches Spec-Sync Stabilisierungsprotokoll |
| `docs/spec-sync/L02.md` | Keine | **archivieren** | Historisches Spec-Sync Stabilisierungsprotokoll |
| `docs/spec-sync/S-01.md` | Keine | **archivieren** | Historisches Spec-Sync Stabilisierungsprotokoll |
| `docs/spec-sync/S-02.md` | Keine | **archivieren** | Historisches Spec-Sync Stabilisierungsprotokoll |
| `docs/spec-sync/S-03.md` | Keine | **archivieren** | Historisches Spec-Sync Stabilisierungsprotokoll |
| `docs/spec-sync/S-04.md` | Keine | **archivieren** | Historisches Spec-Sync Stabilisierungsprotokoll |
| `docs/spec/00-meta-workspace-layout.md` | Keine | **behalten** | Aktive v4 Produktspezifikation |
| `docs/spec/00a-titel-und-einleitung.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/00b-teil-a-stabilisierungsauftrag.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/00c-teil-a2-zielarchitektur-v2.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/00d-teil-a3-aktueller-umsetzungsstand.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/00e-teil-a4-architekten-review-sota.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/01-kernthese-und-leitprinzip.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/02-produktvision-und-nicht-ziele.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/03-architekturprinzipien.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/04-crate-graph.md` | Keine | **behalten** | Aktive v4 Produktspezifikation |
| `docs/spec/05-speicherschicht.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/06-wissensgraph-datenmodell-a.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/06-wissensgraph-datenmodell-b.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/07-retrieval-pipeline.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/08-contextual-bandit-routing.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/09-inferenz-kv-cache-ipc.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/10-sicherheits-und-datenschutzmodell.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/11-betriebsmodi.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/12-flatbuffers-schema.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/13-fehlertaxonomie.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/14-feature-flag-politik.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/14-jules-environment-spec.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/15-test-und-ci-spezifikation.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/16-abnahmekriterien.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/17-optimierungs-roadmap.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/18-gesamtroadmap.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/19-rueckverfolgbarkeitsmatrix.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/20-migrationsplan-und-adr.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/21-sota-algorithmen-erweiterung-a.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/21-sota-algorithmen-erweiterung-b.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/22-jules-autonomous-agent-specification.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/90-anhang-b-begruendungen-a.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/90-anhang-b-begruendungen-b.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/91-anhang-c-aenderungsprotokoll-v2-1.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/92-anhang-d-aenderungsprotokoll-v4.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` | `crates/contextra-adapt/src/homeostat.rs` | **behalten** | Aktive v4 Produktspezifikation |
| `docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/spec/GESAMTSPEZIFIKATION-v4.md` | Keine | **behalten** | Aktive v4 Produktspezifikation |
| `docs/spec/INDEX.md` | Keine | **behalten** | Aktive v4 Produktspezifikation |
| `docs/spec/SOURCE_OF_TRUTH_DIFF.md` | Keine | **archivieren** | Historische Spezifikations-Teildokumente v2/v3/Anhang |
| `docs/supply-chain-trust.md` | Keine | **behalten** | Aktive System- / Architektur-Spezifikation |
