# Zeilenrechnung für das behaltene Kernprodukt Contextra

**Repository:** https://github.com/tfufuz1/contextra
**Analysierter Commit:** `c4f7299b644913c49a8bd184b0db2c2499bbc0cc` (origin/main, Stand 25.10.2026)
**Zielgröße:** **74.467 Zeilen Produktionscode** (hergeleitet aus der Liste der behaltenen Kern-Crates)

---

## 1. Übersicht nach Crates

| Crate | Zweck im Kernprodukt | Zeilen Produktionscode |
|---|---|---|
| `contextra` | Facade Crate & Builder | 1.101 |
| `contextra-types` | Kanonische Domänenmodelle, Fehlertypen & Konfiguration | 5.774 |
| `contextra-ports` | Abstraktionsschnittstellen & Trait-Ports | 3.627 |
| `contextra-sys` | Systemprimitiven, Entropie & OS-Abstraktion | 709 |
| `contextra-crypto` | Keyrings, AEAD-Verschlüsselung & DeletionProof v4 | 8.145 |
| `contextra-store` | LSM-Engine, WAL, v5 SSTable & Tombstone-Purge | 17.430 |
| `contextra-vector` | HNSW-Vektorindex, Distanzmetriken & Zeroization | 11.433 |
| `contextra-text` | Invertierter BM25-Index & Tokenizer | 4.383 |
| `contextra-engine` | Engine Orchestrator, Collection & Deletion Driver | 13.477 |
| `contextra-db` | High-Level Datenbank-API Handle | 2.106 |
| `contextra-mcp` | Model Context Protocol Server & Tool-Interface | 5.499 |
| `contextra-testkit` | Test-Harness, Fault Injection & Mocks | 783 |
| **Gesamtsumme Kernprodukt** | **12 Kern-Crates** | **74.467** |

---

## 2. Herleitung der Reduktion

- **Ausgangsbestand:** 146.335 Zeilen Produktionscode über 35 Workspace-Mitglieder.
- **Archivierte / Entfernte Komponenten (71.868 Zeilen Prod-Code):**
  - Periphere / Optionale Module: `contextra-graph` (14.374), `contextra-cognition` (5.610), `contextra-adapt` (4.789), `contextra-agent` (4.719), `contextra-kvcache` (4.546), `contextra-rank` (3.921), `contextra-infer-candle` (4.387), `contextra-infer-ollama` (3.383), `contextra-infer-onnx` (1.543), `contextra-privacy` (3.213), `contextra-checkpoint` (3.081), `contextra-router` (2.923), `contextra-sandbox` (2.125), `contextra-py` (1.893), `contextra-simd` (1.496).
  - Obsolete Stubs / Redundanzen: `contextra-mvcc` (3.508), `contextra-wire` (1.601), `contextra-license` (410), `contextra-audit-export` (377), `contextra-avv-generator` (253), `contextra-core` (136).
- **Zielbestand Kernprodukt:** **74.467 Zeilen Produktionscode** (-49,1 % Reduktion).

---

## 3. Detaillierte Aufschlüsselung Datei für Datei

### Crate: `contextra`

| Datei / Modul | Zeilen Produktionscode |
|---|---|
| `src/agent_memory.rs` | 326 |
| `src/builder.rs` | 279 |
| `src/collection_profile.rs` | 122 |
| `src/lib.rs` | 91 |
| `src/performance_profile.rs` | 88 |
| `src/query_rewriter.rs` | 195 |
| **Summe `contextra`** | **1101** |

### Crate: `contextra-types`

| Datei / Modul | Zeilen Produktionscode |
|---|---|
| `src/error.rs` | 1115 |
| `src/error_dto.rs` | 579 |
| `src/lib.rs` | 30 |
| `src/model_fingerprint.rs` | 53 |
| `src/retrieval_strategy.rs` | 86 |
| `src/schema.rs` | 102 |
| `src/tenant_scope.rs` | 63 |
| `src/tombstone.rs` | 131 |
| `src/types.rs` | 20 |
| `src/types/budget.rs` | 691 |
| `src/types/domain/document.rs` | 223 |
| `src/types/domain/ids.rs` | 464 |
| `src/types/domain/misc.rs` | 559 |
| `src/types/domain/mod.rs` | 30 |
| `src/types/filter.rs` | 417 |
| `src/types/importance.rs` | 310 |
| `src/types/saos.rs` | 901 |
| **Summe `contextra-types`** | **5774** |

### Crate: `contextra-ports`

| Datei / Modul | Zeilen Produktionscode |
|---|---|
| `src/attention.rs` | 16 |
| `src/checkpoint.rs` | 57 |
| `src/clock.rs` | 92 |
| `src/embedding.rs` | 324 |
| `src/graph.rs` | 214 |
| `src/graph_index.rs` | 520 |
| `src/id_gen.rs` | 96 |
| `src/kv.rs` | 250 |
| `src/kv_bridge_port.rs` | 24 |
| `src/lib.rs` | 139 |
| `src/license.rs` | 111 |
| `src/lifecycle.rs` | 55 |
| `src/metrics.rs` | 223 |
| `src/observability.rs` | 14 |
| `src/plugin.rs` | 434 |
| `src/query_rewriter.rs` | 46 |
| `src/reranker.rs` | 22 |
| `src/rng.rs` | 186 |
| `src/storage.rs` | 277 |
| `src/text_index.rs` | 214 |
| `src/vector_index.rs` | 313 |
| **Summe `contextra-ports`** | **3627** |

### Crate: `contextra-sys`

| Datei / Modul | Zeilen Produktionscode |
|---|---|
| `src/acl_win32.rs` | 324 |
| `src/lib.rs` | 15 |
| `src/mlock.rs` | 128 |
| `src/mmap.rs` | 39 |
| `src/posix.rs` | 92 |
| `src/vault.rs` | 111 |
| **Summe `contextra-sys`** | **709** |

### Crate: `contextra-crypto`

| Datei / Modul | Zeilen Produktionscode |
|---|---|
| `fuzz/fuzz_targets/deletion_proof_tamper.rs` | 117 |
| `fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs` | 127 |
| `fuzz/fuzz_targets/wal_hmac_chain_verify_fuzz.rs` | 155 |
| `src/anti_tamper.rs` | 134 |
| `src/audit_chain.rs` | 329 |
| `src/crypto.rs` | 925 |
| `src/deletion_proof.rs` | 1916 |
| `src/ed25519_proof.rs` | 210 |
| `src/error.rs` | 130 |
| `src/kdf.rs` | 278 |
| `src/kv_cipher.rs` | 383 |
| `src/kv_segment/eviction_worker.rs` | 210 |
| `src/kv_segment/mod.rs` | 11 |
| `src/kv_segment/segment.rs` | 352 |
| `src/kv_segment/store.rs` | 845 |
| `src/kv_shredding.rs` | 567 |
| `src/lib.rs` | 57 |
| `src/revocation_log.rs` | 450 |
| `src/wal_completeness.rs` | 66 |
| `src/wal_crypto.rs` | 883 |
| **Summe `contextra-crypto`** | **8145** |

### Crate: `contextra-store`

| Datei / Modul | Zeilen Produktionscode |
|---|---|
| `fuzz/fuzz_targets/fuzz_compaction_interleave.rs` | 122 |
| `fuzz/fuzz_targets/fuzz_manifest_load.rs` | 29 |
| `fuzz/fuzz_targets/fuzz_memtable_concurrent.rs` | 118 |
| `fuzz/fuzz_targets/fuzz_recovery_arbitrary_state.rs` | 62 |
| `fuzz/fuzz_targets/fuzz_sstable_binary_search.rs` | 33 |
| `fuzz/fuzz_targets/fuzz_wal_replay.rs` | 42 |
| `fuzz/fuzz_targets/wal_mutation_chaos.rs` | 299 |
| `fuzz/fuzz_targets/wal_roundtrip.rs` | 40 |
| `src/compaction.rs` | 20 |
| `src/compaction/adaptive.rs` | 356 |
| `src/compaction/config.rs` | 91 |
| `src/compaction/engine.rs` | 837 |
| `src/compaction/merge_operator.rs` | 16 |
| `src/compaction/retention.rs` | 80 |
| `src/engine/memory_only.rs` | 360 |
| `src/engine/mod.rs` | 4 |
| `src/kv/delete_mode.rs` | 46 |
| `src/kv/mod.rs` | 10 |
| `src/kv/segment.rs` | 169 |
| `src/lib.rs` | 59 |
| `src/lsm/commit.rs` | 146 |
| `src/lsm/config.rs` | 114 |
| `src/lsm/engine.rs` | 420 |
| `src/lsm/flush.rs` | 18 |
| `src/lsm/group_commit.rs` | 76 |
| `src/lsm/guard.rs` | 13 |
| `src/lsm/mod.rs` | 120 |
| `src/lsm/observer.rs` | 486 |
| `src/lsm/ops.rs` | 153 |
| `src/lsm/ops/compaction.rs` | 249 |
| `src/lsm/ops/maintenance.rs` | 52 |
| `src/lsm/ops/read.rs` | 327 |
| `src/lsm/ops/write.rs` | 746 |
| `src/lsm/recovery.rs` | 878 |
| `src/lsm/scan.rs` | 202 |
| `src/lsm/validate.rs` | 43 |
| `src/manifest/core.rs` | 382 |
| `src/manifest/entry.rs` | 370 |
| `src/manifest/mod.rs` | 22 |
| `src/manifest/recovery.rs` | 126 |
| `src/manifest/rollover.rs` | 193 |
| `src/memtable.rs` | 1023 |
| `src/sstable.rs` | 20 |
| `src/sstable/block_cache.rs` | 361 |
| `src/sstable/block_search.rs` | 199 |
| `src/sstable/bloom.rs` | 162 |
| `src/sstable/builder.rs` | 498 |
| `src/sstable/io.rs` | 29 |
| `src/sstable/reader.rs` | 1075 |
| `src/sstable/reader_ext.rs` | 299 |
| `src/sstable/stream.rs` | 133 |
| `src/system_pressure.rs` | 367 |
| `src/tenant_codec.rs` | 877 |
| `src/util.rs` | 83 |
| `src/wal/encode.rs` | 452 |
| `src/wal/flusher.rs` | 726 |
| `src/wal/hmac.rs` | 510 |
| `src/wal/io.rs` | 1032 |
| `src/wal/mod.rs` | 906 |
| `src/wal/replay.rs` | 749 |
| **Summe `contextra-store`** | **17430** |

### Crate: `contextra-vector`

| Datei / Modul | Zeilen Produktionscode |
|---|---|
| `fuzz/fuzz_targets/fuzz_hnsw_persistence.rs` | 136 |
| `fuzz/fuzz_targets/hnsw_insert_search.rs` | 85 |
| `src/acorn/gamma_augmentation.rs` | 53 |
| `src/acorn/mod.rs` | 71 |
| `src/acorn/naive_reference.rs` | 89 |
| `src/candidate_stream.rs` | 169 |
| `src/compute_pool.rs` | 207 |
| `src/diskann/build.rs` | 795 |
| `src/diskann/config.rs` | 78 |
| `src/diskann/filtered.rs` | 73 |
| `src/diskann/format.rs` | 194 |
| `src/diskann/mod.rs` | 22 |
| `src/diskann/persistence.rs` | 689 |
| `src/diskann/predicate_augmented.rs` | 273 |
| `src/diskann/search.rs` | 397 |
| `src/diskann/types.rs` | 280 |
| `src/diskann/vector_index_impl.rs` | 191 |
| `src/distance.rs` | 241 |
| `src/hnsw/acorn_filtered.rs` | 297 |
| `src/hnsw/adaptive_ef.rs` | 229 |
| `src/hnsw/arena.rs` | 369 |
| `src/hnsw/batch.rs` | 89 |
| `src/hnsw/config.rs` | 210 |
| `src/hnsw/core_insert.rs` | 444 |
| `src/hnsw/core_rebuild.rs` | 913 |
| `src/hnsw/core_search.rs` | 738 |
| `src/hnsw/deletion.rs` | 353 |
| `src/hnsw/mod.rs` | 36 |
| `src/hnsw/sq8_bias.rs` | 108 |
| `src/hnsw/types.rs` | 812 |
| `src/hnsw/vector_index_impl.rs` | 569 |
| `src/lib.rs` | 44 |
| `src/partial_rebuild.rs` | 363 |
| `src/persistence/header.rs` | 437 |
| `src/persistence/mmap.rs` | 218 |
| `src/persistence/mod.rs` | 19 |
| `src/persistence/node.rs` | 82 |
| `src/quantize/mod.rs` | 556 |
| `src/quantize_rabitq.rs` | 504 |
| **Summe `contextra-vector`** | **11433** |

### Crate: `contextra-text`

| Datei / Modul | Zeilen Produktionscode |
|---|---|
| `fuzz/fuzz_targets/fuzz_bm25_tokenize.rs` | 11 |
| `src/bm25.rs` | 662 |
| `src/domain/legal_de.rs` | 48 |
| `src/domain/medical_de.rs` | 48 |
| `src/domain/mod.rs` | 131 |
| `src/inverted.rs` | 25 |
| `src/inverted/index_struct.rs` | 665 |
| `src/inverted/morph_index.rs` | 81 |
| `src/inverted/types.rs` | 48 |
| `src/lib.rs` | 271 |
| `src/morphology.rs` | 23 |
| `src/morphology/compound_splitter.rs` | 303 |
| `src/morphology/metrics.rs` | 22 |
| `src/morphology/passthrough.rs` | 23 |
| `src/morphology/stopwords.rs` | 51 |
| `src/morphology/trie.rs` | 56 |
| `src/posting_list.rs` | 505 |
| `src/stream.rs` | 146 |
| `src/tokenizer.rs` | 436 |
| `src/wand.rs` | 828 |
| **Summe `contextra-text`** | **4383** |

### Crate: `contextra-engine`

| Datei / Modul | Zeilen Produktionscode |
|---|---|
| `src/background_workers/config.rs` | 63 |
| `src/background_workers/expiry_workers.rs` | 113 |
| `src/background_workers/hyperedge_worker.rs` | 102 |
| `src/background_workers/mod.rs` | 29 |
| `src/background_workers/orphan_workers.rs` | 176 |
| `src/chunker.rs` | 1092 |
| `src/collection/crud/auto_extraction.rs` | 299 |
| `src/collection/crud/delete.rs` | 64 |
| `src/collection/crud/insert.rs` | 548 |
| `src/collection/crud/internal.rs` | 72 |
| `src/collection/crud/kv.rs` | 62 |
| `src/collection/crud/links.rs` | 222 |
| `src/collection/crud/mod.rs` | 21 |
| `src/collection/crud/read.rs` | 296 |
| `src/collection/crud/update.rs` | 120 |
| `src/collection/kv_lock.rs` | 223 |
| `src/collection/maintenance.rs` | 945 |
| `src/collection/mod.rs` | 740 |
| `src/collection/query_builder/builder.rs` | 305 |
| `src/collection/query_builder/builder_exec.rs` | 463 |
| `src/collection/query_builder/mod.rs` | 30 |
| `src/collection/query_builder/scope.rs` | 58 |
| `src/collection/query_builder/strategy.rs` | 80 |
| `src/collection/query_builder/weights.rs` | 40 |
| `src/collection/relate.rs` | 282 |
| `src/collection/search/basic.rs` | 119 |
| `src/collection/search/checkpoint.rs` | 99 |
| `src/collection/search/filtered.rs` | 251 |
| `src/collection/search/hybrid/mod.rs` | 377 |
| `src/collection/search/hybrid/query.rs` | 492 |
| `src/collection/search/hydrate.rs` | 105 |
| `src/collection/search/mod.rs` | 25 |
| `src/collection/tx.rs` | 35 |
| `src/contextra_impl/collections.rs` | 391 |
| `src/contextra_impl/crud.rs` | 334 |
| `src/contextra_impl/diagnostics.rs` | 60 |
| `src/contextra_impl/lifecycle.rs` | 577 |
| `src/contextra_impl/mod.rs` | 5 |
| `src/contextra_impl/wiring.rs` | 32 |
| `src/decay_controller.rs` | 5 |
| `src/export.rs` | 388 |
| `src/extraction/mod.rs` | 93 |
| `src/extraction/open_ie.rs` | 122 |
| `src/extraction/rule_based.rs` | 233 |
| `src/extraction/types.rs` | 44 |
| `src/filter.rs` | 322 |
| `src/fusion.rs` | 59 |
| `src/import.rs` | 126 |
| `src/kv_cache_integration/mod.rs` | 78 |
| `src/lib.rs` | 663 |
| `src/merge_operator_adapter.rs` | 194 |
| `src/metrics_names.rs` | 60 |
| `src/temporal_filter.rs` | 352 |
| `src/transaction/cleanup.rs` | 92 |
| `src/transaction/compensating_actions.rs` | 373 |
| `src/transaction/db_transaction.rs` | 172 |
| `src/transaction/db_transaction_commit.rs` | 384 |
| `src/transaction/db_transaction_rollback.rs` | 291 |
| `src/transaction/intent.rs` | 38 |
| `src/transaction/mod.rs` | 41 |
| **Summe `contextra-engine`** | **13477** |

### Crate: `contextra-db`

| Datei / Modul | Zeilen Produktionscode |
|---|---|
| `fuzz/fuzz_targets/rrf_fusion.rs` | 58 |
| `src/homeostat.rs` | 7 |
| `src/lib.rs` | 886 |
| `src/multistep.rs` | 704 |
| `src/volatile_vault.rs` | 451 |
| **Summe `contextra-db`** | **2106** |

### Crate: `contextra-mcp`

| Datei / Modul | Zeilen Produktionscode |
|---|---|
| `fuzz/fuzz_targets/fuzz_jsonrpc_parsing.rs` | 37 |
| `fuzz/fuzz_targets/fuzz_prompt_injection_guard.rs` | 67 |
| `src/bin/contextra-mcp-server.rs` | 88 |
| `src/bulk_exfiltration_detector.rs` | 4 |
| `src/config.rs` | 598 |
| `src/egress_gateway.rs` | 28 |
| `src/egress_guard.rs` | 135 |
| `src/explain.rs` | 268 |
| `src/io.rs` | 82 |
| `src/lib.rs` | 80 |
| `src/plugin_status.rs` | 43 |
| `src/prompt_injection.rs` | 57 |
| `src/prompt_injection/audit.rs` | 67 |
| `src/prompt_injection/guard.rs` | 635 |
| `src/prompt_injection/policy.rs` | 61 |
| `src/proof_key.rs` | 151 |
| `src/proof_key_env.rs` | 187 |
| `src/protocol.rs` | 87 |
| `src/routing.rs` | 137 |
| `src/sandbox.rs` | 886 |
| `src/server.rs` | 178 |
| `src/server_dispatch.rs` | 414 |
| `src/server_tools.rs` | 943 |
| `src/tools_crud.rs` | 237 |
| `src/validation.rs` | 29 |
| **Summe `contextra-mcp`** | **5499** |

### Crate: `contextra-testkit`

| Datei / Modul | Zeilen Produktionscode |
|---|---|
| `src/fault_vfs.rs` | 189 |
| `src/in_memory_store.rs` | 250 |
| `src/lib.rs` | 16 |
| `src/manual_clock.rs` | 109 |
| `src/reference_model.rs` | 219 |
| **Summe `contextra-testkit`** | **783** |

## Gesamtsumme behaltener Produktionscode: **74467 Zeilen**
