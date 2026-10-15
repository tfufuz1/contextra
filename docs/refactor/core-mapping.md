# Contextra Core Re-Export Mapping Report

**Commit SHA:** `4ecbb9f55fdf41d7892da9ab22fcc5909b629df5`
**Gesamtanzahl erfasster Symbole:** 57
**Anzahl offener Symbole (`OFFEN`):** 0

## Vollständige Symbol-Zuordnung

| Symbol | Ursprungscrate | Ursprungsmodul | Core Re-Export Pfad | Empfohlenes Zielcrate | Status |
|---|---|---|---|---|---|
| `AttentionExporter` | `contextra_ports` | `contextra_ports::attention` | `contextra_core::traits::AttentionExporter` | `contextra_ports` | KLAR |
| `BoxFuture` | `contextra_ports` | `contextra_ports` | `contextra_core::traits::BoxFuture` | `contextra_ports` | KLAR |
| `BoxStream` | `contextra_ports` | `contextra_ports` | `contextra_core::traits::BoxStream` | `contextra_ports` | KLAR |
| `Clock` | `contextra_ports` | `contextra_ports::clock` | `contextra_core::traits::Clock` | `contextra_ports` | KLAR |
| `ContextraError` | `contextra_types` | `contextra_types::error` | `contextra_core::contextra_types::error::ContextraError` | `contextra_types` | KLAR |
| `ContextraErrorDto` | `contextra_types` | `contextra_types::error_dto` | `contextra_core::contextra_types::error_dto::ContextraErrorDto` | `contextra_types` | KLAR |
| `DocIdWidth` | `contextra_types` | `contextra_types::schema` | `contextra_core::contextra_types::schema::DocIdWidth` | `contextra_types` | KLAR |
| `EntropyRng` | `contextra_ports` | `contextra_ports::rng` | `contextra_core::traits::EntropyRng` | `contextra_ports` | KLAR |
| `ErasureBackend` | `contextra_ports` | `contextra_ports::erasure` | `contextra_core::traits::ErasureBackend` | `contextra_ports` | KLAR |
| `ErasureHooks` | `contextra_ports` | `contextra_ports::erasure` | `contextra_core::traits::ErasureHooks` | `contextra_ports` | KLAR |
| `FreeSpaceProbe` | `contextra_ports` | `contextra_ports::erasure` | `contextra_core::traits::FreeSpaceProbe` | `contextra_ports` | KLAR |
| `GraphCollectionMutation` | `contextra_ports` | `contextra_ports::graph` | `contextra_core::traits::GraphCollectionMutation` | `contextra_ports` | KLAR |
| `GraphIndex` | `contextra_ports` | `contextra_ports::graph_index` | `contextra_core::traits::GraphIndex` | `contextra_ports` | KLAR |
| `GraphPersistence` | `contextra_ports` | `contextra_ports::graph_persistence` | `contextra_core::traits::GraphPersistence` | `contextra_ports` | KLAR |
| `IdGen` | `contextra_ports` | `contextra_ports::id_gen` | `contextra_core::traits::IdGen` | `contextra_ports` | KLAR |
| `IdGenerator` | `contextra_ports` | `contextra_ports::id_gen` | `contextra_core::traits::IdGenerator` | `contextra_ports` | KLAR |
| `IndexOp` | `contextra_mvcc` | `contextra_mvcc::tx_buffer` | `contextra_core::contextra_mvcc::tx_buffer::IndexOp` | `contextra_mvcc` | KLAR |
| `JsonRpcError` | `contextra_wire` | `contextra_wire::jsonrpc` | `contextra_core::ipc::JsonRpcError` | `contextra_wire` | KLAR |
| `JsonRpcRequest` | `contextra_wire` | `contextra_wire::jsonrpc` | `contextra_core::ipc::JsonRpcRequest` | `contextra_wire` | KLAR |
| `JsonRpcResponse` | `contextra_wire` | `contextra_wire::jsonrpc` | `contextra_core::ipc::JsonRpcResponse` | `contextra_wire` | KLAR |
| `KvBridgeStorage` | `contextra_ports` | `contextra_ports::kv_bridge_port` | `contextra_core::traits::KvBridgeStorage` | `contextra_ports` | KLAR |
| `KvPrefixStore` | `contextra_ports` | `contextra_ports::kv` | `contextra_core::traits::KvPrefixStore` | `contextra_ports` | KLAR |
| `LayerScan` | `contextra_ports` | `contextra_ports::erasure` | `contextra_core::traits::LayerScan` | `contextra_ports` | KLAR |
| `LicenseGate` | `contextra_ports` | `contextra_ports::license` | `contextra_core::traits::LicenseGate` | `contextra_ports` | KLAR |
| `ManifestSchemaVersion` | `contextra_types` | `contextra_types::schema` | `contextra_core::contextra_types::schema::ManifestSchemaVersion` | `contextra_types` | KLAR |
| `MergeOperator` | `contextra_ports` | `contextra_ports::merge_operator` | `contextra_core::traits::MergeOperator` | `contextra_ports` | KLAR |
| `MergeOperatorFactory` | `contextra_ports` | `contextra_ports::merge_operator_factory` | `contextra_core::traits::MergeOperatorFactory` | `contextra_ports` | KLAR |
| `MetricsSink` | `contextra_ports` | `contextra_ports::metrics` | `contextra_core::traits::MetricsSink` | `contextra_ports` | KLAR |
| `ModelFingerprint` | `contextra_types` | `contextra_types::model_fingerprint` | `contextra_core::contextra_types::model_fingerprint::ModelFingerprint` | `contextra_types` | KLAR |
| `QueryRewriter` | `contextra_ports` | `contextra_ports::query_rewriter` | `contextra_core::traits::QueryRewriter` | `contextra_ports` | KLAR |
| `Reranker` | `contextra_ports` | `contextra_ports::reranker` | `contextra_core::traits::Reranker` | `contextra_ports` | KLAR |
| `RestoreGuard` | `contextra_ports` | `contextra_ports::erasure` | `contextra_core::traits::RestoreGuard` | `contextra_ports` | KLAR |
| `Result` | `contextra_types` | `contextra_types::error` | `contextra_core::contextra_types::error::Result` | `contextra_types` | KLAR |
| `Rng` | `contextra_ports` | `contextra_ports::rng` | `contextra_core::traits::Rng` | `contextra_ports` | KLAR |
| `SeededRng` | `contextra_ports` | `contextra_ports::rng` | `contextra_core::traits::SeededRng` | `contextra_ports` | KLAR |
| `SeqBitTombstone` | `contextra_types` | `contextra_types::tombstone` | `contextra_core::contextra_types::tombstone::SeqBitTombstone` | `contextra_types` | KLAR |
| `SeqLogChange` | `contextra_mvcc` | `contextra_mvcc::seq_log` | `contextra_core::contextra_mvcc::seq_log::SeqLogChange` | `contextra_mvcc` | KLAR |
| `SeqLogEntry` | `contextra_mvcc` | `contextra_mvcc::seq_log` | `contextra_core::contextra_mvcc::seq_log::SeqLogEntry` | `contextra_mvcc` | KLAR |
| `SequenceLog` | `contextra_mvcc` | `contextra_mvcc::seq_log` | `contextra_core::contextra_mvcc::seq_log::SequenceLog` | `contextra_mvcc` | KLAR |
| `SnapshotGuard` | `contextra_mvcc` | `contextra_mvcc::snapshot` | `contextra_core::contextra_mvcc::snapshot::SnapshotGuard` | `contextra_mvcc` | KLAR |
| `SnapshotReader` | `contextra_ports` | `contextra_ports::snapshot` | `contextra_core::traits::SnapshotReader` | `contextra_ports` | KLAR |
| `SnapshotRegistry` | `contextra_mvcc` | `contextra_mvcc::snapshot` | `contextra_core::contextra_mvcc::snapshot::SnapshotRegistry` | `contextra_mvcc` | KLAR |
| `StorageEngine` | `contextra_ports` | `contextra_ports::storage` | `contextra_core::traits::StorageEngine` | `contextra_ports` | KLAR |
| `TextEmbeddingEngine` | `contextra_ports` | `contextra_ports::embedding` | `contextra_core::traits::TextEmbeddingEngine` | `contextra_ports` | KLAR |
| `TombstoneSemanticsCheck` | `contextra_types` | `contextra_types::tombstone` | `contextra_core::contextra_types::tombstone::TombstoneSemanticsCheck` | `contextra_types` | KLAR |
| `TxBuffer` | `contextra_mvcc` | `contextra_mvcc::tx_buffer` | `contextra_core::contextra_mvcc::tx_buffer::TxBuffer` | `contextra_mvcc` | KLAR |
| `WireWiring` | `contextra_wire` | `contextra_wire::wiring` | `contextra_core::ipc::WireWiring` | `contextra_wire` | KLAR |
| `WriteGate` | `contextra_ports` | `contextra_ports::erasure` | `contextra_core::traits::WriteGate` | `contextra_ports` | KLAR |
| `error` | `contextra_types` | `contextra_types` | `contextra_core::contextra_types::error` | `contextra_types` | KLAR |
| `error_dto` | `contextra_types` | `contextra_types` | `contextra_core::contextra_types::error_dto` | `contextra_types` | KLAR |
| `model_fingerprint` | `contextra_types` | `contextra_types` | `contextra_core::contextra_types::model_fingerprint` | `contextra_types` | KLAR |
| `schema` | `contextra_types` | `contextra_types` | `contextra_core::contextra_types::schema` | `contextra_types` | KLAR |
| `seq_log` | `contextra_mvcc` | `contextra_mvcc` | `contextra_core::contextra_mvcc::seq_log` | `contextra_mvcc` | KLAR |
| `snapshot` | `contextra_mvcc` | `contextra_mvcc` | `contextra_core::contextra_mvcc::snapshot` | `contextra_mvcc` | KLAR |
| `tombstone` | `contextra_types` | `contextra_types` | `contextra_core::contextra_types::tombstone` | `contextra_types` | KLAR |
| `tx_buffer` | `contextra_mvcc` | `contextra_mvcc` | `contextra_core::contextra_mvcc::tx_buffer` | `contextra_mvcc` | KLAR |
| `types` | `contextra_types` | `contextra_types` | `contextra_core::contextra_types::types` | `contextra_types` | KLAR |
