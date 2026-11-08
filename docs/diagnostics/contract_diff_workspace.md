# Port & Contract Interface Drift Diagnostic Report

> **Werkzeug:** `contract_diff_detector.py`  
> **Erstellungszeit:** `2026-11-07T23:03:01.009917+00:00`  
> **Status/Befunde:** `225 Befunde erfasst`  

## Beschreibung
Analyzes canonical traits in contextra-ports, verifies '# Fehler' and '# Crash' contract documentation, and maps implementations across the workspace.

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `total_port_traits` | `45` |
| `total_port_methods` | `104` |
| `methods_missing_fehler_doc` | `60` |
| `methods_missing_crash_doc` | `63` |
| `workspace_port_implementations` | `121` |

## Port-Implementierungs-Matrix
| Crate | Anzahl Port-Impls | Implementierte Ports |
|---|---|---|
| `contextra` | 2 | `LicenseGate, QueryRewriter` |
| `contextra-agent` | 1 | `Checkpoint` |
| `contextra-checkpoint` | 4 | `StorageEngine` |
| `contextra-cognition` | 9 | `Clock, LlmTextGenerator, ResponseGroundingValidator, SegmentSynthesizer, StorageEngine` |
| `contextra-crypto` | 1 | `Vfs` |
| `contextra-db` | 1 | `DriftStatusProvider` |
| `contextra-durable-fs` | 2 | `Vfs, VfsFile` |
| `contextra-engine` | 10 | `LlmTextGenerator, Reranker, StorageEngine, TextEmbeddingEngine` |
| `contextra-erasure` | 11 | `CommitBarrier, ErasureObserver, FileKeyring, LedgerSink, PurgeBarrier, ReadGate, RecordSealer, ReportSigner, StorageBackend, VfsFile` |
| `contextra-graph` | 4 | `GraphCollectionMutation, GraphIndex, Purger, StorageEngine` |
| `contextra-infer-candle` | 7 | `AttentionExporter, EmbeddingProvider, GroundingValidator, KvBridgeStorage, LlmTextGenerator, LlmTextGeneratorStreaming, ResponseGroundingValidator` |
| `contextra-infer-ollama` | 5 | `EmbeddingProvider, LlmTextGenerator, LlmTextGeneratorStreaming, QueryRewriter, SegmentSynthesizer` |
| `contextra-infer-onnx` | 5 | `EmbeddingProvider, Reranker, TextEmbeddingEngine, Vfs` |
| `contextra-kvcache` | 4 | `KvBridgeStorage, KvPrefixStore, LayerScanner, PluginManifest` |
| `contextra-license` | 1 | `LicenseGate` |
| `contextra-mcp` | 9 | `Clock, CommunityResolver, ContextPreparer, DriftStatusProvider, EmbeddingProvider, HybridSearchProvider, LlmTextGenerator` |
| `contextra-privacy` | 2 | `Clock` |
| `contextra-rank` | 1 | `Clock` |
| `contextra-router` | 12 | `Clock, CommunityResolver, ContextPreparer, HybridSearchProvider` |
| `contextra-store` | 7 | `LayerScanner, Purger, StorageEngine` |
| `contextra-testkit` | 8 | `Clock, EmbeddingProvider, GraphIndex, MetricsSink, StorageEngine, TextIndex, VectorIndex, Vfs` |
| `contextra-text` | 9 | `LayerScanner, Purger, StorageEngine, TextIndex` |
| `contextra-vector` | 6 | `Clock, LayerScanner, MetricsSink, Purger, VectorIndex` |

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra-ports` | `crates/contextra-ports/src/attention.rs` | `11` | Port method `AttentionExporter::export_attention_weights` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/checkpoint.rs` | `26` | Port method `CheckpointCoordinator::drop_named_checkpoint` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/checkpoint.rs` | `26` | Port method `CheckpointCoordinator::list_named_checkpoints` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/clock.rs` | `8` | Port method `Clock::now_unix_nanos` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/clock.rs` | `8` | Port method `Clock::monotonic_nanos` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/barrier.rs` | `91` | Port method `CommitBarrier::shared` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/barrier.rs` | `91` | Port method `CommitBarrier::exclusive` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/observability.rs` | `10` | Port method `DriftStatusProvider::overall_drift_status` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/policy.rs` | `19` | Port method `EgressPolicy::evaluate` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/embedding.rs` | `83` | Port method `EmbeddingProvider::provider_name` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/embedding.rs` | `83` | Port method `EmbeddingProvider::embedding_dim` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/erasure.rs` | `236` | Port method `ErasureLayer::id` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/erasure.rs` | `236` | Port method `ErasureLayer::resolve` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/erasure.rs` | `236` | Port method `ErasureLayer::on_gate` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/erasure.rs` | `236` | Port method `ErasureLayer::purge` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/erasure.rs` | `236` | Port method `ErasureLayer::key_coverage` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/keys.rs` | `177` | Port method `FileKeyring::create` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/keys.rs` | `177` | Port method `FileKeyring::open` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/keys.rs` | `177` | Port method `FileKeyring::destroy` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/id_gen.rs` | `7` | Port method `IdGen::next_id` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/kv.rs` | `139` | Port method `KvPrefixStore::lookup` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/kv.rs` | `139` | Port method `KvPrefixStore::evict` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/scanner/mod.rs` | `52` | Port method `LayerScanner::layer` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/scanner/mod.rs` | `52` | Port method `LayerScanner::scan_units` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/ledger.rs` | `152` | Port method `LedgerSink::append` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/ledger.rs` | `152` | Port method `LedgerSink::read_all` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/ledger.rs` | `152` | Port method `LedgerSink::prune` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/license.rs` | `70` | Port method `LicenseGate::check_ring` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/license.rs` | `70` | Port method `LicenseGate::authorize` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/metrics.rs` | `19` | Port method `MetricsSink::record_counter` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/metrics.rs` | `19` | Port method `MetricsSink::record_gauge` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/metrics.rs` | `19` | Port method `MetricsSink::record_histogram` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/plugin.rs` | `40` | Port method `PluginManifest::name` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/plugin.rs` | `40` | Port method `PluginManifest::requires` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/plugin.rs` | `40` | Port method `PluginManifest::conflicts` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/plugin.rs` | `40` | Port method `PluginManifest::feature_ring_required` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/plugin.rs` | `40` | Port method `PluginManifest::capability` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/plugin.rs` | `40` | Port method `PluginManifest::activate` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/barrier.rs` | `112` | Port method `PurgeBarrier::compaction` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/barrier.rs` | `112` | Port method `PurgeBarrier::purge` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/barrier.rs` | `112` | Port method `PurgeBarrier::purge_waiting` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/scanner/layer.rs` | `115` | Port method `Purger::purge` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/gate.rs` | `128` | Port method `ReadGate::check` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/gate.rs` | `128` | Port method `ReadGate::check_bound` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/gate.rs` | `128` | Port method `ReadGate::variant_for_range` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/gate.rs` | `128` | Port method `ReadGate::gated_seq` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/gate.rs` | `128` | Port method `ReadGate::gate_epoch` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/gate.rs` | `128` | Port method `ReadGate::covers` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/gate.rs` | `128` | Port method `ReadGate::check_raw` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/gate.rs` | `128` | Port method `ReadGate::covers_raw` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/gate.rs` | `128` | Port method `ReadGate::gated_seq_raw` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/keys.rs` | `128` | Port method `RecordSealer::evict` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/keys.rs` | `128` | Port method `RecordSealer::make_slots_durable` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/report_signer.rs` | `45` | Port method `ReportSigner::signer_id` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/reranker.rs` | `7` | Port method `Reranker::record_implicit_feedback` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/reranker.rs` | `7` | Port method `Reranker::rerank_deadline_ms` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/lifecycle.rs` | `38` | Port method `ResponseGroundingValidator::score_grounding` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/rng.rs` | `12` | Port method `Rng::next_u64` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/rng.rs` | `12` | Port method `Rng::fill_bytes` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/rng.rs` | `12` | Port method `Rng::next_unit_f64` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/text_index.rs` | `16` | Port method `SegmentSynthesizer::model_id` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/storage.rs` | `35` | Port method `StorageEngine::supports_ssi_tracking` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/text_index.rs` | `41` | Port method `TextIndex::insert` (Fehler-Doc: True, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/text_index.rs` | `41` | Port method `TextIndex::delete` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/text_index.rs` | `41` | Port method `TextIndex::commit` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/text_index.rs` | `41` | Port method `TextIndex::rollback` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/text_index.rs` | `41` | Port method `TextIndex::rollback_to_tx` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/text_index.rs` | `41` | Port method `TextIndex::last_tx_id` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/text_index.rs` | `41` | Port method `TextIndex::len` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/text_index.rs` | `41` | Port method `TextIndex::is_empty` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/text_index.rs` | `41` | Port method `TextIndex::stats` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/vector_index.rs` | `37` | Port method `VectorIndex::all_doc_ids` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/vector_index.rs` | `37` | Port method `VectorIndex::delete` (Fehler-Doc: True, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/vector_index.rs` | `37` | Port method `VectorIndex::commit` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/vector_index.rs` | `37` | Port method `VectorIndex::rollback` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/vector_index.rs` | `37` | Port method `VectorIndex::rollback_to_tx` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/vector_index.rs` | `37` | Port method `VectorIndex::last_tx_id` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/vector_index.rs` | `37` | Port method `VectorIndex::len` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/vector_index.rs` | `37` | Port method `VectorIndex::is_empty` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/vector_index.rs` | `37` | Port method `VectorIndex::stats` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/vector_index.rs` | `37` | Port method `VectorIndex::is_rebuild_required` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/vector_index.rs` | `37` | Port method `VectorIndex::trigger_rebuild_async` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `165` | Port method `Vfs::open` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `165` | Port method `Vfs::read_at` (Fehler-Doc: True, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `165` | Port method `Vfs::read` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `165` | Port method `Vfs::write_atomic` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `165` | Port method `Vfs::create_dir_all` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `165` | Port method `Vfs::remove_file` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `165` | Port method `Vfs::unlink` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `165` | Port method `Vfs::rename` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `165` | Port method `Vfs::read_dir` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `165` | Port method `Vfs::exists` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `165` | Port method `Vfs::len` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `165` | Port method `Vfs::list_dir` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `165` | Port method `Vfs::list` (Fehler-Doc: False, Crash-Doc: False) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `165` | Port method `Vfs::sync_dir` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `76` | Port method `VfsFile::read_at` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `76` | Port method `VfsFile::read_verified` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `76` | Port method `VfsFile::write_at` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `76` | Port method `VfsFile::set_len` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `76` | Port method `VfsFile::len` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `76` | Port method `VfsFile::is_empty` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `76` | Port method `VfsFile::sync_data` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra-ports` | `crates/contextra-ports/src/vfs.rs` | `76` | Port method `VfsFile::sync_all` (Fehler-Doc: True, Crash-Doc: True) |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `209` | Implementation: `MockLicenseGate` implements Port `LicenseGate` |
| `contextra` | `crates/contextra/src/query_rewriter.rs` | `93` | Implementation: `LlmQueryRewriter` implements Port `QueryRewriter` |
| `contextra-agent` | `crates/contextra-agent/src/engine.rs` | `820` | Implementation: `FallbackRegistry` implements Port `Checkpoint` |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard.rs` | `534` | Implementation: `MockStorage` implements Port `StorageEngine` |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard/tests.rs` | `26` | Implementation: `MockStorage` implements Port `StorageEngine` |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `38` | Implementation: `MockStorage` implements Port `StorageEngine` |
| `contextra-checkpoint` | `crates/contextra-checkpoint/benches/checkpoint_bench.rs` | `37` | Implementation: `BenchStorage` implements Port `StorageEngine` |
| `contextra-cognition` | `crates/contextra-cognition/src/consolidation_executor.rs` | `193` | Implementation: `GroundingValidatorAdapter` implements Port `ResponseGroundingValidator` |
| `contextra-cognition` | `crates/contextra-cognition/src/consolidation_executor.rs` | `853` | Implementation: `TestMockLlm` implements Port `LlmTextGenerator` |
| `contextra-cognition` | `crates/contextra-cognition/src/maintenance_scheduler.rs` | `515` | Implementation: `TestClock` implements Port `Clock` |
| `contextra-cognition` | `crates/contextra-cognition/src/memory_consolidation.rs` | `1057` | Implementation: `MockLlmGenerator` implements Port `LlmTextGenerator` |
| `contextra-cognition` | `crates/contextra-cognition/src/memory_consolidation.rs` | `1249` | Implementation: `MockLowScoreGroundingValidator` implements Port `ResponseGroundingValidator` |
| `contextra-cognition` | `crates/contextra-cognition/src/synthesis_phase.rs` | `143` | Implementation: `MockSynthesizer` implements Port `SegmentSynthesizer` |
| `contextra-cognition` | `crates/contextra-cognition/src/context_compaction/tests.rs` | `173` | Implementation: `UnreachableLlmGenerator` implements Port `LlmTextGenerator` |
| `contextra-cognition` | `crates/contextra-cognition/src/context_compaction/tests.rs` | `236` | Implementation: `MockLlmGenerator` implements Port `LlmTextGenerator` |
| `contextra-cognition` | `crates/contextra-cognition/src/context_compaction/tests.rs` | `313` | Implementation: `FaultyDeleteStorage` implements Port `StorageEngine` |
| `contextra-crypto` | `crates/contextra-crypto/src/revocation_log.rs` | `346` | Implementation: `TestVfs` implements Port `Vfs` |
| `contextra-db` | `crates/contextra-db/src/lib.rs` | `372` | Implementation: `DummyRouter` implements Port `DriftStatusProvider` |
| `contextra-durable-fs` | `crates/contextra-durable-fs/src/vfs.rs` | `39` | Implementation: `RealFs` implements Port `Vfs` |
| `contextra-durable-fs` | `crates/contextra-durable-fs/src/vfs.rs` | `160` | Implementation: `RealFsFile` implements Port `VfsFile` |
| `contextra-engine` | `crates/contextra-engine/src/collection/query_builder/tests.rs` | `46` | Implementation: `SlowPartialMockReranker` implements Port `Reranker` |
| `contextra-engine` | `crates/contextra-engine/src/collection/query_builder/tests.rs` | `80` | Implementation: `MockReranker` implements Port `Reranker` |
| `contextra-engine` | `crates/contextra-engine/src/collection/crud/auto_extraction.rs` | `69` | Implementation: `NoopLlmGenerator` implements Port `LlmTextGenerator` |
| `contextra-engine` | `crates/contextra-engine/src/collection/tests/insert_tests.rs` | `244` | Implementation: `FailingRollbackMockStorage` implements Port `StorageEngine` |
| `contextra-engine` | `crates/contextra-engine/src/collection/tests/link_tests.rs` | `73` | Implementation: `FailOnStorageCommit` implements Port `StorageEngine` |
| `contextra-engine` | `crates/contextra-engine/src/collection/tests/link_tests.rs` | `188` | Implementation: `FailOnPutStorage` implements Port `StorageEngine` |
| `contextra-engine` | `crates/contextra-engine/src/collection/tests/misc_tests.rs` | `19` | Implementation: `FakeEmbedder` implements Port `TextEmbeddingEngine` |
| `contextra-engine` | `crates/contextra-engine/src/collection/tests/read_tests.rs` | `24` | Implementation: `BoundedScanMockStorage` implements Port `StorageEngine` |
| `contextra-engine` | `crates/contextra-engine/src/collection/tests/search_tests.rs` | `69` | Implementation: `PinTrackingFailingStorage` implements Port `StorageEngine` |
| `contextra-engine` | `crates/contextra-engine/src/extraction/tests.rs` | `34` | Implementation: `MockLlmGenerator` implements Port `LlmTextGenerator` |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger.rs` | `170` | Implementation: `InMemoryLedger` implements Port `LedgerSink` |
| `contextra-erasure` | `crates/contextra-erasure/src/test_mocks.rs` | `32` | Implementation: `MockStorageBackend` implements Port `StorageBackend` |
| `contextra-erasure` | `crates/contextra-erasure/src/types_local.rs` | `357` | Implementation: `Arc` implements Port `CommitBarrier` |
| `contextra-erasure` | `crates/contextra-erasure/src/types_local.rs` | `366` | Implementation: `Arc` implements Port `PurgeBarrier` |
| `contextra-erasure` | `crates/contextra-erasure/src/types_local.rs` | `1096` | Implementation: `Arc` implements Port `ReportSigner` |
| `contextra-erasure` | `crates/contextra-erasure/src/types_local.rs` | `1171` | Implementation: `Arc` implements Port `ErasureObserver` |
| `contextra-erasure` | `crates/contextra-erasure/src/types_local.rs` | `1428` | Implementation: `MemoryVfsFile` implements Port `VfsFile` |
| `contextra-erasure` | `crates/contextra-erasure/src/vault/keyring.rs` | `84` | Implementation: `VaultKeyring` implements Port `FileKeyring` |
| `contextra-erasure` | `crates/contextra-erasure/src/vault/sealer.rs` | `159` | Implementation: `VaultSealer` implements Port `RecordSealer` |
| `contextra-erasure` | `crates/contextra-erasure/src/gate/erased_set.rs` | `305` | Implementation: `ErasedSet` implements Port `ReadGate` |
| `contextra-erasure` | `crates/contextra-erasure/src/gate/read_gate.rs` | `84` | Implementation: `GateGuard` implements Port `ReadGate` |
| `contextra-graph` | `crates/contextra-graph/src/purger.rs` | `43` | Implementation: `GraphPurger` implements Port `Purger` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write/mod.rs` | `213` | Implementation: `CsrGraph` implements Port `GraphCollectionMutation` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index/mod.rs` | `95` | Implementation: `CsrGraph` implements Port `GraphIndex` |
| `contextra-graph` | `crates/contextra-graph/fuzz/fuzz_targets/fuzz_graph_persist.rs` | `45` | Implementation: `FuzzStorage` implements Port `StorageEngine` |
| `contextra-infer-candle` | `crates/contextra-infer-candle/src/attention_exporter.rs` | `100` | Implementation: `CandleAttentionExporter` implements Port `AttentionExporter` |
| `contextra-infer-candle` | `crates/contextra-infer-candle/src/embedding_provider.rs` | `19` | Implementation: `CandleEmbedClient` implements Port `EmbeddingProvider` |
| `contextra-infer-candle` | `crates/contextra-infer-candle/src/gasp.rs` | `265` | Implementation: `GaspValidator` implements Port `ResponseGroundingValidator` |
| `contextra-infer-candle` | `crates/contextra-infer-candle/src/gasp.rs` | `284` | Implementation: `GaspValidator` implements Port `GroundingValidator` |
| `contextra-infer-candle` | `crates/contextra-infer-candle/src/kv_bridge.rs` | `316` | Implementation: `MockKvStorage` implements Port `KvBridgeStorage` |
| `contextra-infer-candle` | `crates/contextra-infer-candle/src/inference/core.rs` | `833` | Implementation: `CandleLlmClient` implements Port `LlmTextGenerator` |
| `contextra-infer-candle` | `crates/contextra-infer-candle/src/inference/core.rs` | `936` | Implementation: `CandleLlmClient` implements Port `LlmTextGeneratorStreaming` |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/embedding.rs` | `110` | Implementation: `OllamaEmbedder` implements Port `EmbeddingProvider` |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/query_rewriter.rs` | `71` | Implementation: `OllamaQueryRewriter` implements Port `QueryRewriter` |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/trait_impls.rs` | `3` | Implementation: `OllamaClient` implements Port `LlmTextGenerator` |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/trait_impls.rs` | `12` | Implementation: `OllamaClient` implements Port `LlmTextGeneratorStreaming` |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/trait_impls.rs` | `41` | Implementation: `OllamaClient` implements Port `SegmentSynthesizer` |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/lib.rs` | `530` | Implementation: `TextEmbedder` implements Port `EmbeddingProvider` |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/lib.rs` | `1052` | Implementation: `MockEngine` implements Port `TextEmbeddingEngine` |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/lib.rs` | `1075` | Implementation: `MockOrderedEngine` implements Port `TextEmbeddingEngine` |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/lib.rs` | `1233` | Implementation: `TestMockVfs` implements Port `Vfs` |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/cross_encoder.rs` | `268` | Implementation: `CrossEncoderReranker` implements Port `Reranker` |
| `contextra-kvcache` | `crates/contextra-kvcache/src/adapters.rs` | `52` | Implementation: `StorageEngineKvBridge` implements Port `KvBridgeStorage` |
| `contextra-kvcache` | `crates/contextra-kvcache/src/adapters.rs` | `175` | Implementation: `KvCachePluginManifest` implements Port `PluginManifest` |
| `contextra-kvcache` | `crates/contextra-kvcache/src/prefix_store.rs` | `348` | Implementation: `TenantPrefixKvStore` implements Port `KvPrefixStore` |
| `contextra-kvcache` | `crates/contextra-kvcache/src/store.rs` | `853` | Implementation: `KvCacheScanner` implements Port `LayerScanner` |
| `contextra-license` | `crates/contextra-license/src/signed_gate.rs` | `737` | Implementation: `SignedLicenseGate` implements Port `LicenseGate` |
| `contextra-mcp` | `crates/contextra-mcp/src/config.rs` | `562` | Implementation: `MockEmbedder` implements Port `EmbeddingProvider` |
| `contextra-mcp` | `crates/contextra-mcp/src/config.rs` | `602` | Implementation: `MockLlmGenerator` implements Port `LlmTextGenerator` |
| `contextra-mcp` | `crates/contextra-mcp/src/explain.rs` | `131` | Implementation: `MockEmbedder` implements Port `EmbeddingProvider` |
| `contextra-mcp` | `crates/contextra-mcp/src/routing.rs` | `8` | Implementation: `CollectionAdapter` implements Port `HybridSearchProvider` |
| `contextra-mcp` | `crates/contextra-mcp/src/routing.rs` | `38` | Implementation: `CommunityAdapter` implements Port `CommunityResolver` |
| `contextra-mcp` | `crates/contextra-mcp/src/routing.rs` | `49` | Implementation: `ContextPreparerAdapter` implements Port `ContextPreparer` |
| `contextra-mcp` | `crates/contextra-mcp/src/routing.rs` | `69` | Implementation: `RouterDriftAdapter` implements Port `DriftStatusProvider` |
| `contextra-mcp` | `crates/contextra-mcp/src/tests.rs` | `17` | Implementation: `MockEmbedder` implements Port `EmbeddingProvider` |
| `contextra-mcp` | `crates/contextra-mcp/src/prompt_injection/tests.rs` | `357` | Implementation: `FixedTestClock` implements Port `Clock` |
| `contextra-privacy` | `crates/contextra-privacy/src/context_edit_audit.rs` | `285` | Implementation: `FixedTestClock` implements Port `Clock` |
| `contextra-privacy` | `crates/contextra-privacy/src/egress_guard.rs` | `351` | Implementation: `MockClock` implements Port `Clock` |
| `contextra-rank` | `crates/contextra-rank/src/calibration/isotonic.rs` | `334` | Implementation: `FixedTestClock` implements Port `Clock` |
| `contextra-router` | `crates/contextra-router/examples/simple_router.rs` | `13` | Implementation: `ExampleSearchProvider` implements Port `HybridSearchProvider` |
| `contextra-router` | `crates/contextra-router/examples/simple_router.rs` | `36` | Implementation: `ExampleCommunityResolver` implements Port `CommunityResolver` |
| `contextra-router` | `crates/contextra-router/examples/simple_router.rs` | `44` | Implementation: `ExampleContextPreparer` implements Port `ContextPreparer` |
| `contextra-router` | `crates/contextra-router/src/tests/fixtures.rs` | `35` | Implementation: `TestClock` implements Port `Clock` |
| `contextra-router` | `crates/contextra-router/src/tests/fixtures.rs` | `103` | Implementation: `TestCollection` implements Port `HybridSearchProvider` |
| `contextra-router` | `crates/contextra-router/src/tests/fixtures.rs` | `161` | Implementation: `TestCollection` implements Port `CommunityResolver` |
| `contextra-router` | `crates/contextra-router/src/tests/fixtures.rs` | `180` | Implementation: `CollectionAdapter` implements Port `HybridSearchProvider` |
| `contextra-router` | `crates/contextra-router/src/tests/fixtures.rs` | `191` | Implementation: `CollectionAdapter` implements Port `CommunityResolver` |
| `contextra-router` | `crates/contextra-router/src/tests/fixtures.rs` | `199` | Implementation: `TestContextPreparer` implements Port `ContextPreparer` |
| `contextra-router` | `crates/contextra-router/benches/router_bench.rs` | `42` | Implementation: `BenchAdapter` implements Port `HybridSearchProvider` |
| `contextra-router` | `crates/contextra-router/benches/router_bench.rs` | `85` | Implementation: `BenchAdapter` implements Port `CommunityResolver` |
| `contextra-router` | `crates/contextra-router/benches/router_bench.rs` | `97` | Implementation: `BenchContextPreparer` implements Port `ContextPreparer` |
| `contextra-store` | `crates/contextra-store/src/tenant_codec.rs` | `554` | Implementation: `TenantScopedStorage` implements Port `StorageEngine` |
| `contextra-store` | `crates/contextra-store/src/scanner/memtable.rs` | `53` | Implementation: `MemtableScanner` implements Port `LayerScanner` |
| `contextra-store` | `crates/contextra-store/src/scanner/purge.rs` | `35` | Implementation: `LsmPurger` implements Port `Purger` |
| `contextra-store` | `crates/contextra-store/src/scanner/sstable.rs` | `103` | Implementation: `SstableScanner` implements Port `LayerScanner` |
| `contextra-store` | `crates/contextra-store/src/scanner/wal.rs` | `73` | Implementation: `WalScanner` implements Port `LayerScanner` |
| `contextra-store` | `crates/contextra-store/src/lsm/ops.rs` | `12` | Implementation: `LsmStorage` implements Port `StorageEngine` |
| `contextra-store` | `crates/contextra-store/src/engine/memory_only.rs` | `57` | Implementation: `InMemoryStorageEngine` implements Port `StorageEngine` |
| `contextra-testkit` | `crates/contextra-testkit/src/fault_vfs.rs` | `239` | Implementation: `FaultVfs` implements Port `Vfs` |
| `contextra-testkit` | `crates/contextra-testkit/src/in_memory_graph.rs` | `75` | Implementation: `InMemoryGraphIndex` implements Port `GraphIndex` |
| `contextra-testkit` | `crates/contextra-testkit/src/in_memory_store.rs` | `95` | Implementation: `InMemoryStorageEngine` implements Port `StorageEngine` |
| `contextra-testkit` | `crates/contextra-testkit/src/in_memory_text.rs` | `79` | Implementation: `InMemoryTextIndex` implements Port `TextIndex` |
| `contextra-testkit` | `crates/contextra-testkit/src/in_memory_vector.rs` | `76` | Implementation: `InMemoryVectorIndex` implements Port `VectorIndex` |
| `contextra-testkit` | `crates/contextra-testkit/src/manual_clock.rs` | `113` | Implementation: `ManualClock` implements Port `Clock` |
| `contextra-testkit` | `crates/contextra-testkit/src/mocks.rs` | `68` | Implementation: `MockEmbedder` implements Port `EmbeddingProvider` |
| `contextra-testkit` | `crates/contextra-testkit/src/mocks.rs` | `170` | Implementation: `TestMetricsSink` implements Port `MetricsSink` |
| `contextra-text` | `crates/contextra-text/src/lib.rs` | `95` | Implementation: `Bm25Scorer` implements Port `TextIndex` |
| `contextra-text` | `crates/contextra-text/src/lib.rs` | `189` | Implementation: `MockStorage` implements Port `StorageEngine` |
| `contextra-text` | `crates/contextra-text/src/wand.rs` | `647` | Implementation: `MockStorage` implements Port `StorageEngine` |
| `contextra-text` | `crates/contextra-text/src/inverted/index_struct.rs` | `669` | Implementation: `InvertedIndex` implements Port `TextIndex` |
| `contextra-text` | `crates/contextra-text/src/inverted/morph_index.rs` | `41` | Implementation: `BM25MorphIndex` implements Port `TextIndex` |
| `contextra-text` | `crates/contextra-text/src/inverted/purge.rs` | `485` | Implementation: `TextPurger` implements Port `Purger` |
| `contextra-text` | `crates/contextra-text/src/inverted/scanner.rs` | `193` | Implementation: `TextScanner` implements Port `LayerScanner` |
| `contextra-text` | `crates/contextra-text/src/inverted/tests/mock.rs` | `31` | Implementation: `MockStorage` implements Port `StorageEngine` |
| `contextra-text` | `crates/contextra-text/src/inverted/tests/search_tests.rs` | `615` | Implementation: `CountingStorage` implements Port `StorageEngine` |
| `contextra-vector` | `crates/contextra-vector/src/diskann/vector_index_impl.rs` | `14` | Implementation: `DiskAnnIndex` implements Port `VectorIndex` |
| `contextra-vector` | `crates/contextra-vector/src/hnsw/core_rebuild.rs` | `997` | Implementation: `TestClock` implements Port `Clock` |
| `contextra-vector` | `crates/contextra-vector/src/hnsw/purge.rs` | `43` | Implementation: `HnswPurger` implements Port `Purger` |
| `contextra-vector` | `crates/contextra-vector/src/hnsw/scanner.rs` | `224` | Implementation: `HnswScanner` implements Port `LayerScanner` |
| `contextra-vector` | `crates/contextra-vector/src/hnsw/tests.rs` | `235` | Implementation: `TestMetricsSink` implements Port `MetricsSink` |
| `contextra-vector` | `crates/contextra-vector/src/hnsw/vector_index_impl.rs` | `12` | Implementation: `HnswIndex` implements Port `VectorIndex` |

---
*Automatisch generiert durch `contract_diff_detector.py` für Agentic Engineering & Context Auditing.*