# Crate-Kopplungsbericht

- Erzeugt: 2026-11-05 22:54
- Workspace-Root: `/home/freddy/Projekte/contextra_31-10-26-Full-Library`
- Analysierte Crates: 36 · Komponenten-Tiefe: 1 · Zeilen (ohne Leerzeilen): 331.865
- Legende: **prod** = src/bin/build ohne `cfg(test)`; **dev** = tests/benches/examples/fuzz und `cfg(test)`-Code.
- Hinweis: Textuelle Analyse ohne Compiler. Glob-Importe (`*`) sind nicht auflösbar. Siehe Kopf des Skripts für Grenzen.

## 1. Überblick

| Crate | Ring | Zeilen | Fan-out prod | Fan-in prod | Cargo-Deps intern (n/d/b) | Pub-Items | API-Lecks | Zyklus |
|---|---|---:|---:|---:|---|---:|---:|---|
| [contextra](#contextra) | 4 | 4156 | 7 | 1 | 11/2/0 | 20 | 12 | – |
| [contextra-adapt](#contextra-adapt) | 0 | 6493 | 2 | 3 | 2/0/0 | 55 | 3 | – |
| [contextra-agent](#contextra-agent) | 3 | 9181 | 5 | 0 | 7/3/0 | 51 | 8 | – |
| [contextra-audit-export](#contextra-audit-export) | 0 | 1229 | 1 | 0 | 2/0/0 | 15 | 1 | – |
| [contextra-avv-generator](#contextra-avv-generator) | 0 | 1424 | 1 | 1 | 1/0/0 | 16 | 1 | – |
| [contextra-bench](#contextra-bench) | tooling | 4142 | 2 | 0 | 4/1/0 | 28 | 2 | – |
| [contextra-checkpoint](#contextra-checkpoint) | 1 | 8374 | 2 | 2 | 3/2/0 | 1 | 0 | – |
| [contextra-cognition](#contextra-cognition) | 3 | 9382 | 5 | 1 | 6/2/0 | 59 | 24 | – |
| [contextra-core](#contextra-core) | 0 | 515 | 4 | 1 | 4/0/0 | 0 | 14 | – |
| [contextra-crypto](#contextra-crypto) | 0 | 15200 | 2 | 6 | 2/2/0 | 84 | 10 | – |
| [contextra-db](#contextra-db) | 3 | 13322 | 8 | 3 | 13/2/0 | 13 | 23 | – |
| [contextra-durable-fs](#contextra-durable-fs) | 0 | 2327 | 1 | 0 | 1/0/0 | 29 | 1 | – |
| [contextra-engine](#contextra-engine) | 3 | 26428 | 12 | 3 | 14/2/0 | 80 | 28 | – |
| [contextra-graph](#contextra-graph) | 0 | 26905 | 2 | 3 | 3/2/0 | 122 | 23 | – |
| [contextra-infer-candle](#contextra-infer-candle) | 2 | 7268 | 4 | 3 | 4/1/0 | 38 | 3 | – |
| [contextra-infer-ollama](#contextra-infer-ollama) | 2 | 5493 | 3 | 1 | 3/0/0 | 36 | 5 | – |
| [contextra-infer-onnx](#contextra-infer-onnx) | 2 | 2738 | 2 | 2 | 3/0/0 | 10 | 5 | – |
| [contextra-ingest](#contextra-ingest) | 4 | 25186 | 2 | 0 | 2/0/0 | 133 | 0 | – |
| [contextra-kvcache](#contextra-kvcache) | 1 | 7306 | 3 | 0 | 3/0/0 | 48 | 4 | – |
| [contextra-license](#contextra-license) | 4 | 2618 | 2 | 1 | 2/0/0 | 15 | 2 | – |
| [contextra-mcp](#contextra-mcp) | 4 | 8706 | 10 | 0 | 12/0/0 | 34 | 12 | – |
| [contextra-mvcc](#contextra-mvcc) | 0 | 5446 | 2 | 5 | 2/1/0 | 21 | 1 | – |
| [contextra-ports](#contextra-ports) | 0 | 4291 | 1 | 22 | 1/0/0 | 75 | 2 | – |
| [contextra-privacy](#contextra-privacy) | 3 | 7209 | 3 | 2 | 3/2/0 | 69 | 6 | – |
| [contextra-py](#contextra-py) | 4 | 3157 | 4 | 0 | 6/0/0 | 36 | 3 | – |
| [contextra-rank](#contextra-rank) | 0 | 6508 | 1 | 7 | 2/1/0 | 20 | 2 | – |
| [contextra-router](#contextra-router) | 3 | 8611 | 4 | 2 | 5/4/0 | 23 | 4 | – |
| [contextra-sandbox](#contextra-sandbox) | 2 | 3781 | 0 | 0 | 0/0/0 | 14 | 0 | – |
| [contextra-simd](#contextra-simd) | 0 | 4269 | 1 | 1 | 1/0/0 | 28 | 13 | – |
| [contextra-store](#contextra-store) | 1 | 45672 | 5 | 3 | 5/5/0 | 113 | 5 | – |
| [contextra-sys](#contextra-sys) | 0 | 1640 | 0 | 4 | 0/0/0 | 23 | 0 | – |
| [contextra-testkit](#contextra-testkit) | tooling | 4320 | 2 | 0 | 2/0/0 | 22 | 8 | – |
| [contextra-text](#contextra-text) | 0 | 13329 | 1 | 1 | 2/0/0 | 50 | 5 | – |
| [contextra-types](#contextra-types) | 0 | 7968 | 0 | 28 | 0/0/0 | 66 | 0 | – |
| [contextra-vector](#contextra-vector) | 0 | 22182 | 6 | 2 | 6/0/0 | 33 | 1 | – |
| [contextra-wire](#contextra-wire) | 0 | 5089 | 0 | 2 | 0/0/0 | 19 | 0 | – |

## 2. Workspace-weite Auffälligkeiten

### 2.1 Zyklen

- Kein Zyklus im prod-Code-Graphen gefunden.
- Kein Zyklus in den normalen Cargo-Abhängigkeiten.

### 2.2 Abweichungen Cargo.toml ↔ Code

| Crate | Ziel | Deklariert als | Befund |
|---|---|---|---|
| `contextra` | `contextra-audit-export` | normal? | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra` | `contextra-avv-generator` | normal? | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra` | `contextra-crypto` | dev/normal? | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra` | `contextra-privacy` | normal? | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-agent` | `contextra-checkpoint` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-agent` | `contextra-graph` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-agent` | `contextra-vector` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-audit-export` | `contextra-ports` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-db` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-store` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-text` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-vector` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-core` | – | genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler) |
| `contextra-bench` | `contextra-infer-candle` | – | genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler) |
| `contextra-checkpoint` | `contextra-testkit` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-checkpoint` | `contextra-types` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-cognition` | `contextra-store` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-crypto` | `contextra-crypto` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-db` | `contextra-crypto` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-db` | `contextra-rank` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-db` | `contextra-store` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-db` | `contextra-text` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-db` | `contextra-vector` | dev/normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-engine` | `contextra-sandbox` | normal? | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-engine` | `contextra-sys` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-graph` | `contextra-adapt` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-graph` | `contextra-testkit` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-infer-onnx` | `contextra-ports` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-mcp` | `contextra-license` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-mcp` | `contextra-wire` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-py` | `contextra-adapt` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-py` | `contextra-store` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-rank` | `contextra-ports` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-router` | `contextra-wire` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-store` | `contextra-graph` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-store` | `contextra-store` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-store` | `contextra-vector` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-text` | `contextra-types` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |

### 2.3 Schwache Kanten (billig zu trennen)

Kanten im prod-Code mit höchstens 3 verschiedenen Symbolen. Reihenfolge: wenigste Symbole, dann wenigste Fundstellen.

| Von | Nach | Symbole | Fundstellen | Dateien | Symbole (prod) |
|---|---|---:|---:|---:|---|
| `contextra-core` | `contextra-wire` | 1 | 1 | 1 | `*` |
| `contextra-db` | `contextra-ports` | 1 | 1 | 1 | `QueryRewriter` |
| `contextra-db` | `contextra-checkpoint` | 1 | 1 | 1 | `(crate)` |
| `contextra-db` | `contextra-graph` | 1 | 1 | 1 | `percolation::PercolationConfig` |
| `contextra-db` | `contextra-sys` | 1 | 1 | 1 | `LockedRegions` |
| `contextra-infer-ollama` | `contextra-rank` | 1 | 1 | 1 | `IsotonicCalibrator` |
| `contextra-router` | `contextra-types` | 1 | 1 | 1 | `RetrievalStrategy` |
| `contextra-router` | `contextra-ports` | 1 | 1 | 1 | `clock` |
| `contextra-infer-candle` | `contextra-rank` | 1 | 1 | 1 | `IsotonicCalibrator` |
| `contextra-infer-candle` | `contextra-crypto` | 1 | 1 | 1 | `EncryptedKvLayer` |
| `contextra-testkit` | `contextra-types` | 1 | 1 | 1 | `TxId` |
| `contextra` | `contextra-rank` | 1 | 1 | 1 | `(crate)` |
| `contextra` | `contextra-infer-candle` | 1 | 1 | 1 | `(crate)` |
| `contextra` | `contextra-db` | 1 | 1 | 1 | `QueryRewriter` |
| `contextra-mvcc` | `contextra-types` | 1 | 1 | 1 | `*` |
| `contextra-engine` | `contextra-mvcc` | 1 | 1 | 1 | `tx_buffer::TxBuffer` |
| `contextra-avv-generator` | `contextra-types` | 1 | 1 | 1 | `TenantId` |
| `contextra-license` | `contextra-types` | 1 | 1 | 1 | `TenantId` |
| `contextra-core` | `contextra-ports` | 1 | 2 | 1 | `*`×2 |
| `contextra-vector` | `contextra-sys` | 1 | 2 | 2 | `mmap_readonly`×2 |
| `contextra-checkpoint` | `contextra-mvcc` | 1 | 2 | 2 | `SnapshotRegistry`×2 |
| `contextra-router` | `contextra-privacy` | 1 | 2 | 1 | `guarded_payload`×2 |
| `contextra-py` | `contextra-rank` | 1 | 2 | 1 | `FusionWeights::new`×2 |
| `contextra-engine` | `contextra-adapt` | 1 | 2 | 1 | `PidController`×2 |
| `contextra-audit-export` | `contextra-types` | 1 | 2 | 2 | `TenantId`×2 |
| `contextra-ingest` | `contextra-engine` | 1 | 2 | 2 | `chunker`×2 |
| `contextra-privacy` | `contextra-ports` | 1 | 3 | 3 | `Clock`×3 |
| `contextra-agent` | `contextra-store` | 1 | 3 | 3 | `LsmStorage`×3 |
| `contextra-mvcc` | `contextra-ports` | 1 | 3 | 1 | `MetricsSink`×3 |
| `contextra-cognition` | `contextra-vector` | 1 | 3 | 3 | `HnswIndex`×3 |
| `contextra-infer-onnx` | `contextra-rank` | 1 | 4 | 3 | `PlattScaler`×4 |
| `contextra-vector` | `contextra-crypto` | 1 | 6 | 2 | `wal_crypto::WalHmac::new`×6 |
| `contextra-durable-fs` | `contextra-sys` | 2 | 2 | 2 | `FlushKind`, `rename_durable` |
| `contextra-db` | `contextra-types` | 2 | 2 | 2 | `DistanceMetric`, `MAX_SEARCH_K` |
| `contextra-bench` | `contextra-infer-candle` | 2 | 2 | 1 | `CandleEmbedClient::from_dir`, `model_registry::CandleQuantization::Q4KM` |
| `contextra` | `contextra-license` | 2 | 2 | 1 | `SignedLicenseGate`, `signed_gate::VerifyingKey::from_bytes` |
| `contextra` | `contextra-store` | 2 | 2 | 2 | `kv::delete_mode::KvDeleteMode`, `lsm::config` |
| `contextra-adapt` | `contextra-ports` | 2 | 2 | 1 | `SeededRng::new`, `Rng::next_u64` |
| `contextra-engine` | `contextra-infer-onnx` | 2 | 2 | 1 | `ensure_onnx_model_download`, `OnnxEmbedder::load` |
| `contextra-store` | `contextra-sys` | 2 | 3 | 2 | `mmap_readonly`×2, `set_restrictive_file_acl` |
| `contextra-vector` | `contextra-simd` | 2 | 3 | 3 | `validate_vector`×2, `kernels::scalar` |
| `contextra-privacy` | `contextra-avv-generator` | 2 | 3 | 1 | `AvvContext`×2, `render_avv_markdown` |
| `contextra-license` | `contextra-ports` | 2 | 3 | 2 | `license`×2, `clock` |
| `contextra-ingest` | `contextra-types` | 2 | 3 | 3 | `DocId`×2, `ContextraError` |
| `contextra-mcp` | `contextra-crypto` | 2 | 4 | 1 | `CryptoKey`×3, `CryptoKey::try_new` |
| `contextra-simd` | `contextra-types` | 2 | 5 | 2 | `Result`×4, `ContextraError` |
| `contextra-text` | `contextra-ports` | 2 | 8 | 5 | `StorageEngine`×6, `scanner`×2 |
| `contextra-mcp` | `contextra-infer-ollama` | 3 | 3 | 1 | `OllamaEmbedder::new`, `OllamaConfig`, `OllamaClient::with_config` |
| `contextra-mcp` | `contextra-infer-onnx` | 3 | 3 | 1 | `OnnxEmbedder::from_path`, `default_model_cache_dir`, `OnnxEmbedder::load` |
| `contextra-mcp` | `contextra-rank` | 3 | 3 | 2 | `fusion::ProvenanceRecord`, `IsotonicCalibrator`, `IsotonicCalibrator::with_defaults` |
| `contextra-py` | `contextra-types` | 3 | 3 | 2 | `ContextraError`, `ContextraErrorDto::from`, `TenantId::SYSTEM` |
| `contextra-mcp` | `contextra-infer-candle` | 3 | 4 | 1 | `model_registry::CandleQuantization::Q4KM`×2, `CandleEmbedClient::from_dir`, `CandleLlmClient::from_dir` |
| `contextra-engine` | `contextra-checkpoint` | 3 | 4 | 3 | `InstanceOrphanRegistry`×2, `InstanceOrphanRegistry::new`, `(crate)` |
| `contextra-engine` | `contextra-crypto` | 3 | 4 | 2 | `deletion_proof::CustodyLevel::Local`×2, `deletion_proof::LayerWitnessExt`, `deletion_proof` |
| `contextra-mcp` | `contextra-types` | 3 | 5 | 2 | `ContextraError`×3, `ContextraError::InvalidInput`, `ContextraError::NotFound` |
| `contextra-agent` | `contextra-db` | 3 | 5 | 4 | `Collection`×2, `Contextra`×2, `volatile_vault` |
| `contextra-kvcache` | `contextra-types` | 3 | 5 | 4 | `TenantId`×3, `error::Result`, `error` |
| `contextra-core` | `contextra-mvcc` | 3 | 6 | 1 | `seq_log`×2, `snapshot`×2, `tx_buffer`×2 |
| `contextra-infer-candle` | `contextra-types` | 3 | 7 | 6 | `ContextraError`×5, `DocId::from`, `ModelFingerprint` |
| `contextra-infer-ollama` | `contextra-types` | 3 | 8 | 4 | `Result`×4, `ContextraError`×3, `ConfigFingerprint` |

**Nur in Tests/Benches/Beispielen** (für das Produkt irrelevant, aber sie koppeln den Test-Build):

| Von | Nach | Fundstellen | Symbole |
|---|---|---:|---|
| `contextra` | `contextra-crypto` | 1 | `deletion_proof` |
| `contextra` | `contextra-testkit` | 1 | `mint_witness_for_tests` |
| `contextra-agent` | `contextra-checkpoint` | 1 | `StateCheckpoint` |
| `contextra-agent` | `contextra-graph` | 2 | `CsrGraph`×2 |
| `contextra-agent` | `contextra-testkit` | 3 | `InMemoryStorageEngine`×2, `TestMetricsSink` |
| `contextra-agent` | `contextra-text` | 5 | `Language::English`×5 |
| `contextra-bench` | `contextra-sys` | 1 | `ensure_fd_limit` |
| `contextra-checkpoint` | `contextra-store` | 1 | `lsm` |
| `contextra-checkpoint` | `contextra-types` | 12 | `TxId`×5, `Result`×3, `ContextraError`×2, `ContextraError::Internal`×2 |
| `contextra-cognition` | `contextra-infer-candle` | 1 | `gasp` |
| `contextra-cognition` | `contextra-store` | 16 | `LsmStorage`×8, `LsmConfig`×8 |
| `contextra-cognition` | `contextra-testkit` | 2 | `ManualClock`, `MockEmbedder` |
| `contextra-crypto` | `contextra-testkit` | 5 | `mint_witness_for_tests`×4, `FaultVfs` |
| `contextra-db` | `contextra-crypto` | 4 | `deletion_proof`×2, `CryptoError`, `deletion_proof::DeletionProof` |
| `contextra-db` | `contextra-rank` | 1 | `IsotonicCalibrator::new` |
| `contextra-db` | `contextra-store` | 12 | `LsmConfig`×5, `LsmStorage::new`×4, `LsmStorage`×2, `lsm` |
| `contextra-db` | `contextra-testkit` | 1 | `mint_witness_for_tests` |
| `contextra-db` | `contextra-text` | 11 | `Language::English`×7, `Language`×2, `bm25::score_term`, `inverted::InvertedIndex` |
| `contextra-db` | `contextra-vector` | 6 | `HnswConfig`×3, `HnswIndex`×2, `HnswIndex::try_new` |
| `contextra-engine` | `contextra-testkit` | 2 | `mint_witness_for_tests`, `InMemoryStorageEngine` |
| `contextra-graph` | `contextra-store` | 2 | `LsmConfig`, `LsmStorage::new` |
| `contextra-infer-onnx` | `contextra-ports` | 1 | `EmbeddingProvider` |
| `contextra-mvcc` | `contextra-testkit` | 6 | `ManualClock`×6 |
| `contextra-privacy` | `contextra-crypto` | 2 | `crypto::KeyManager`, `kv_cipher::ModelFingerprint` |
| `contextra-privacy` | `contextra-testkit` | 2 | `ManualClock`×2 |
| `contextra-rank` | `contextra-adapt` | 2 | `PidController`×2 |
| `contextra-router` | `contextra-db` | 14 | `ContextraConfig`×5, `Contextra::open_with_config`×5, `Collection`, `context::ContextManager::new` … (+2) |
| `contextra-router` | `contextra-graph` | 1 | `CsrGraph::new` |
| `contextra-router` | `contextra-text` | 1 | `Language::English` |
| `contextra-router` | `contextra-vector` | 2 | `HnswConfig::default`, `HnswIndex::try_new` |
| `contextra-store` | `contextra-sandbox` | 1 | `MergeOperatorCapabilities` |
| `contextra-store` | `contextra-testkit` | 8 | `ManualClock`×4, `ReferenceModel`×4 |
| `contextra-text` | `contextra-types` | 136 | `Result`×67, `TOMBSTONE_BIT`×57, `TxId`×7, `TxId::new`×2 … (+2) |

### 2.4 Hub-Symbole (Kandidaten für types/ports)

Symbole, die aus einem Nicht-Basis-Crate von mindestens 3 verschiedenen Crates im prod-Code importiert werden. Wer sie in ein schmales Basis-Crate verschiebt, kann viele Kanten kappen.

| Quelle | Symbol | Nutzer | Crates |
|---|---|---:|---|
| `contextra-rank` | `IsotonicCalibrator` | 4 | `contextra-engine`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-mcp` |
| `contextra-graph` | `percolation::PercolationConfig` | 3 | `contextra-cognition`, `contextra-db`, `contextra-engine` |

### 2.5 API-Lecks

Öffentliche Items, deren Signatur oder öffentliche Felder Typen anderer Workspace-Crates enthalten, sowie `pub use` fremder Crates. Diese Stellen machen eine Entkopplung am schwersten, weil Nutzer des Crates die fremde Abhängigkeit mitsehen.

| Crate | Komponente | Item | Fremde Crates | Fundstelle |
|---|---|---|---|---|
| `contextra` | `collection_profile` | `const DEFAULT_AUTO_EXTRACTION_MODE` | `contextra-types` | `src/collection_profile.rs:39` |
| `contextra` | `collection_profile` | `const ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT` | `contextra-types` | `src/collection_profile.rs:47` |
| `contextra` | `collection_profile` | `struct CollectionProfile` | `contextra-store`, `contextra-types` | `src/collection_profile.rs:66` |
| `contextra` | `collection_profile` | `pub use contextra_types::types::domain::AutoExtractionMode` | `contextra-types` | `src/collection_profile.rs:33` |
| `contextra` | `(crate-root)` | `fn open` | `contextra-types` | `src/lib.rs:131` |
| `contextra` | `(crate-root)` | `fn open_with_config` | `contextra-types` | `src/lib.rs:163` |
| `contextra` | `(crate-root)` | `pub use contextra_types::error::ContextraError` | `contextra-types` | `src/lib.rs:53` |
| `contextra` | `(crate-root)` | `pub use contextra_types::DistanceMetric` | `contextra-types` | `src/lib.rs:55` |
| `contextra` | `(crate-root)` | `pub use contextra_rank` | `contextra-rank` | `src/lib.rs:58` |
| `contextra` | `(crate-root)` | `pub use contextra_infer_candle` | `contextra-infer-candle` | `src/lib.rs:60` |
| `contextra` | `performance_profile` | `struct ResolvedProfileConfig` | `contextra-ports` | `src/performance_profile.rs:52` |
| `contextra` | `performance_profile` | `pub use contextra_ports::license::VectorDeleteMode` | `contextra-ports` | `src/performance_profile.rs:29` |
| `contextra-adapt` | `flow_thompson` | `struct DiagonalApproximationBandit` | `contextra-types` | `src/flow_thompson.rs:656` |
| `contextra-adapt` | `flow_thompson` | `enum OffPolicyError` | `contextra-types` | `src/flow_thompson.rs:711` |
| `contextra-adapt` | `flow_thompson` | `struct FcTsSamplingDistribution` | `contextra-types` | `src/flow_thompson.rs:726` |
| `contextra-agent` | `audit` | `struct AuditEntry` | `contextra-types` | `src/audit.rs:22` |
| `contextra-agent` | `audit` | `struct AuditLog` | `contextra-ports`, `contextra-store` | `src/audit.rs:52` |
| `contextra-agent` | `audit` | `fn validate_audit_payload_and_error` | `contextra-types` | `src/audit.rs:57` |
| `contextra-agent` | `audit` | `fn migrate_legacy_audit_entries` | `contextra-db`, `contextra-ports`, `contextra-types` | `src/audit.rs:203` |
| `contextra-agent` | `budget` | `fn reserve_tokens` | `contextra-types` | `src/budget.rs:17` |
| `contextra-agent` | `context` | `fn validate_task_id` | `contextra-types` | `src/context.rs:27` |
| `contextra-agent` | `context` | `fn validate_node_id` | `contextra-types` | `src/context.rs:49` |
| `contextra-agent` | `context` | `struct AgentContext` | `contextra-db`, `contextra-router`, `contextra-store` | `src/context.rs:84` |
| `contextra-audit-export` | `(crate-root)` | `struct ProcessingRegisterEntry` | `contextra-types` | `src/lib.rs:132` |
| `contextra-avv-generator` | `(crate-root)` | `struct AvvContext` | `contextra-types` | `src/lib.rs:70` |
| `contextra-bench` | `path_rag_sweep` | `fn run_pathrag_sweep_long_mem_eval` | `contextra-core` | `src/path_rag_sweep.rs:39` |
| `contextra-bench` | `path_rag_sweep` | `fn run_pathrag_sweep_locomo` | `contextra-core` | `src/path_rag_sweep.rs:198` |
| `contextra-cognition` | `aggregation_phase` | `struct AggregationEdge` | `contextra-graph` | `src/aggregation_phase.rs:164` |
| `contextra-cognition` | `aggregation_phase` | `struct SuperEdgeDraft` | `contextra-graph` | `src/aggregation_phase.rs:177` |
| `contextra-cognition` | `aggregation_phase` | `fn run_aggregation_pass` | `contextra-ports` | `src/aggregation_phase.rs:562` |
| `contextra-cognition` | `consolidation_executor` | `fn execute_consolidation_pass` | `contextra-engine`, `contextra-types` | `src/consolidation_executor.rs:125` |
| `contextra-cognition` | `consolidation_executor` | `fn execute_background_consolidation` | `contextra-engine`, `contextra-types` | `src/consolidation_executor.rs:245` |
| `contextra-cognition` | `consolidation_executor` | `fn execute_background_consolidation_with_rich_validator` | `contextra-engine`, `contextra-types` | `src/consolidation_executor.rs:369` |
| `contextra-cognition` | `consolidation_executor` | `fn execute_leanrag_aggregation_stage` | `contextra-engine`, `contextra-types` | `src/consolidation_executor.rs:434` |
| `contextra-cognition` | `consolidation_executor` | `fn execute_sleep_cycle` | `contextra-engine`, `contextra-types` | `src/consolidation_executor.rs:449` |
| `contextra-cognition` | `consolidation_executor` | `fn start_consolidation_worker` | `contextra-engine` | `src/consolidation_executor.rs:475` |
| `contextra-cognition` | `consolidation_executor` | `fn start_consolidation_reaper` | `contextra-engine` | `src/consolidation_executor.rs:493` |
| `contextra-cognition` | `consolidation_executor` | `struct ConsolidationEngine` | `contextra-vector` | `src/consolidation_executor.rs:516` |
| `contextra-cognition` | `context_compaction::cleanup` | `fn cleanup_orphaned_consolidation_intents` | `contextra-ports` | `src/context_compaction/cleanup.rs:23` |
| `contextra-cognition` | `context_compaction::session` | `struct ConsolidationSession` | `contextra-vector` | `src/context_compaction/session.rs:32` |
| `contextra-cognition` | `leanrag_input` | `fn build_leanrag_inputs` | `contextra-engine` | `src/leanrag_input.rs:66` |
| `contextra-cognition` | `maintenance_config` | `struct PercolationConfig` | `contextra-graph` | `src/maintenance_config.rs:24` |
| `contextra-cognition` | `maintenance_config` | `struct MaintenanceConfig` | `contextra-engine`, `contextra-graph` | `src/maintenance_config.rs:46` |
| `contextra-cognition` | `maintenance_scheduler` | `struct MaintenanceScheduler` | `contextra-vector` | `src/maintenance_scheduler.rs:45` |
| `contextra-cognition` | `memory_consolidation` | `fn run_structural_synthesis_pass` | `contextra-types` | `src/memory_consolidation.rs:591` |
| `contextra-cognition` | `memory_consolidation` | `struct PreCondensedInput` | `contextra-types` | `src/memory_consolidation.rs:655` |
| `contextra-cognition` | `memory_consolidation` | `fn intake_pre_condensed_inputs` | `contextra-types` | `src/memory_consolidation.rs:790` |
| `contextra-cognition` | `semantic_aggregation_facade` | `fn consolidate_semantic_hyperedges` | `contextra-engine` | `src/semantic_aggregation_facade.rs:58` |
| `contextra-cognition` | `synthesis_phase` | `struct SynthesizedChunk` | `contextra-types` | `src/synthesis_phase.rs:38` |
| `contextra-cognition` | `synthesis_phase` | `fn run_synthesis_pass` | `contextra-ports` | `src/synthesis_phase.rs:62` |
| `contextra-cognition` | `synthesis_phase` | `pub use contextra_ports::SegmentSynthesizer` | `contextra-ports` | `src/synthesis_phase.rs:51` |
| `contextra-core` | `(crate-root)` | `pub use contextra_types::error` | `contextra-types` | `src/lib.rs:48` |
| `contextra-core` | `(crate-root)` | `pub use contextra_types::error_dto` | `contextra-types` | `src/lib.rs:67` |
| `contextra-core` | `(crate-root)` | `pub use contextra_types::model_fingerprint` | `contextra-types` | `src/lib.rs:76` |
| `contextra-core` | `(crate-root)` | `pub use contextra_types::schema` | `contextra-types` | `src/lib.rs:85` |
| `contextra-core` | `(crate-root)` | `pub use contextra_types::tombstone` | `contextra-types` | `src/lib.rs:94` |
| `contextra-core` | `(crate-root)` | `pub use contextra_types::types` | `contextra-types` | `src/lib.rs:101` |
| `contextra-core` | `(crate-root)` | `pub use contextra_ports::*` | `contextra-ports` | `src/lib.rs:122` |
| `contextra-core` | `(crate-root)` | `pub use contextra_mvcc::seq_log` | `contextra-mvcc` | `src/lib.rs:131` |
| `contextra-core` | `(crate-root)` | `pub use contextra_mvcc::snapshot` | `contextra-mvcc` | `src/lib.rs:140` |
| `contextra-core` | `(crate-root)` | `pub use contextra_mvcc::tx_buffer` | `contextra-mvcc` | `src/lib.rs:149` |
| `contextra-core` | `(crate-root)` | `pub use contextra_wire::*` | `contextra-wire` | `src/lib.rs:164` |
| `contextra-core` | `(crate-root)` | `pub use contextra_ports::*` | `contextra-ports` | `src/lib.rs:207` |
| `contextra-core` | `(crate-root)` | `pub use contextra_types::error_dto::ContextraErrorDto` | `contextra-types` | `src/lib.rs:225` |
| `contextra-core` | `(crate-root)` | `pub use contextra_types::model_fingerprint::ModelFingerprint` | `contextra-types` | `src/lib.rs:232` |
| `contextra-crypto` | `deletion_proof::chain` | `fn verify_proof_sequence` | `contextra-types` | `src/deletion_proof/chain.rs:12` |
| `contextra-crypto` | `deletion_proof::types` | `fn serialize` | `contextra-ports` | `src/deletion_proof/types.rs:235` |
| `contextra-crypto` | `deletion_proof::types` | `fn deserialize` | `contextra-ports` | `src/deletion_proof/types.rs:250` |
| `contextra-crypto` | `deletion_proof::types` | `struct LayerWitnessMeasurement` | `contextra-ports` | `src/deletion_proof/types.rs:267` |
| `contextra-crypto` | `deletion_proof::verifier` | `enum VerificationErrorReason` | `contextra-types` | `src/deletion_proof/verifier.rs:13` |
| `contextra-crypto` | `error` | `enum CryptoError` | `contextra-types` | `src/error.rs:19` |
| `contextra-crypto` | `kv_cipher` | `struct EncryptedKvLayer` | `contextra-types` | `src/kv_cipher.rs:51` |
| `contextra-crypto` | `kv_cipher` | `pub use contextra_types::ModelFingerprint` | `contextra-types` | `src/kv_cipher.rs:41` |
| `contextra-crypto` | `kv_segment::segment` | `struct KvSegment` | `contextra-types` | `src/kv_segment/segment.rs:42` |
| `contextra-crypto` | `kv_segment::store` | `type SpillHandler` | `contextra-types` | `src/kv_segment/store.rs:26` |
| `contextra-db` | `homeostat` | `pub use contextra_adapt::homeostat::RerankDeadline` | `contextra-adapt` | `src/homeostat.rs:21` |
| `contextra-db` | `homeostat` | `pub use contextra_adapt::homeostat::RerankPidController` | `contextra-adapt` | `src/homeostat.rs:24` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition` | `contextra-cognition` | `src/lib.rs:79` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine` | `contextra-engine` | `src/lib.rs:90` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine::collection::maintenance::PercolationResult` | `contextra-engine` | `src/lib.rs:97` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine::SandboxBridge` | `contextra-engine` | `src/lib.rs:99` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::context::*` | `contextra-cognition` | `src/lib.rs:110` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::context_compaction::*` | `contextra-cognition` | `src/lib.rs:116` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::memory_consolidation::*` | `contextra-cognition` | `src/lib.rs:122` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::synthesis_phase::*` | `contextra-cognition` | `src/lib.rs:128` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::consolidation_executor::*` | `contextra-cognition` | `src/lib.rs:134` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::consolidation_locks::*` | `contextra-cognition` | `src/lib.rs:140` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::maintenance_scheduler::*` | `contextra-cognition` | `src/lib.rs:146` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::maintenance_config::*` | `contextra-cognition` | `src/lib.rs:152` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine::background_workers::*` | `contextra-engine` | `src/lib.rs:159` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::start_consolidation_reaper` | `contextra-cognition` | `src/lib.rs:161` |
| `contextra-db` | `(crate-root)` | `pub use contextra_adapt::decay_controller::*` | `contextra-adapt` | `src/lib.rs:169` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine::fusion` | `contextra-engine` | `src/lib.rs:171` |
| `contextra-db` | `(crate-root)` | `pub use contextra_adapt::pid_latency_controller::*` | `contextra-adapt` | `src/lib.rs:181` |
| `contextra-db` | `(crate-root)` | `pub use contextra_checkpoint` | `contextra-checkpoint` | `src/lib.rs:203` |
| `contextra-db` | `(crate-root)` | `pub use contextra_graph::percolation::PercolationConfig` | `contextra-graph` | `src/lib.rs:204` |
| `contextra-db` | `(crate-root)` | `pub use contextra_types::DistanceMetric` | `contextra-types` | `src/lib.rs:208` |
| `contextra-db` | `multistep` | `pub use contextra_ports::QueryRewriter` | `contextra-ports` | `src/multistep.rs:30` |
| `contextra-durable-fs` | `(crate-root)` | `pub use contextra_sys::FlushKind` | `contextra-sys` | `src/lib.rs:45` |
| `contextra-engine` | `background_workers::expiry_workers` | `fn start_decay_cleanup_worker` | `contextra-ports` | `src/background_workers/expiry_workers.rs:85` |
| `contextra-engine` | `collection::crud::auto_extraction` | `pub use contextra_types::AutoExtractionMode` | `contextra-types` | `src/collection/crud/auto_extraction.rs:18` |
| `contextra-engine` | `collection::maintenance` | `pub use contextra_graph::percolation::PercolationConfig` | `contextra-graph` | `src/collection/maintenance.rs:870` |
| `contextra-engine` | `collection` | `fn compute_default_importance` | `contextra-types` | `src/collection/mod.rs:124` |
| `contextra-engine` | `collection` | `struct Collection` | `contextra-store`, `contextra-vector` | `src/collection/mod.rs:272` |
| `contextra-engine` | `collection::query_builder::scope` | `struct ScopeConstraint` | `contextra-types` | `src/collection/query_builder/scope.rs:19` |
| `contextra-engine` | `collection::query_builder::strategy` | `enum SearchStrategy` | `contextra-types` | `src/collection/query_builder/strategy.rs:11` |
| `contextra-engine` | `extraction::open_ie` | `fn extract_triples` | `contextra-ports` | `src/extraction/open_ie.rs:80` |
| `contextra-engine` | `fusion` | `fn fuse_search_results_with_strategy` | `contextra-types` | `src/fusion.rs:16` |
| `contextra-engine` | `fusion` | `fn fuse_search_results_with_signal_strategies` | `contextra-types` | `src/fusion.rs:35` |
| `contextra-engine` | `(crate-root)` | `type ConsolidationLauncher` | `contextra-store` | `src/lib.rs:98` |
| `contextra-engine` | `(crate-root)` | `fn register_consolidation_launcher` | `contextra-store` | `src/lib.rs:115` |
| `contextra-engine` | `(crate-root)` | `struct ContextraStats` | `contextra-ports` | `src/lib.rs:148` |
| `contextra-engine` | `(crate-root)` | `struct ContextraConfig` | `contextra-ports`, `contextra-store`, `contextra-types` | `src/lib.rs:214` |
| `contextra-engine` | `(crate-root)` | `type TenantCollectionMap` | `contextra-store` | `src/lib.rs:353` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_ports::TextEmbeddingEngine` | `contextra-ports` | `src/lib.rs:22` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_ports::VectorDeleteMode` | `contextra-ports` | `src/lib.rs:23` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_store::lsm::DurabilityMode` | `contextra-store` | `src/lib.rs:24` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_checkpoint` | `contextra-checkpoint` | `src/lib.rs:92` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_text::Language` | `contextra-text` | `src/lib.rs:93` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_ports::DriftStatusProvider` | `contextra-ports` | `src/lib.rs:164` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_types::DistanceMetric` | `contextra-types` | `src/lib.rs:395` |
| `contextra-engine` | `metrics_names` | `fn record_histogram_safe` | `contextra-ports` | `src/metrics_names.rs:33` |
| `contextra-engine` | `metrics_names` | `fn record_gauge_safe` | `contextra-ports` | `src/metrics_names.rs:45` |
| `contextra-engine` | `metrics_names` | `fn record_counter_safe` | `contextra-ports` | `src/metrics_names.rs:57` |
| `contextra-engine` | `temporal_filter` | `struct ValidityWindow` | `contextra-types` | `src/temporal_filter.rs:25` |
| `contextra-engine` | `temporal_filter` | `fn apply_temporal_validity_filter` | `contextra-types` | `src/temporal_filter.rs:166` |
| `contextra-engine` | `transaction::db_transaction` | `struct DbTransaction` | `contextra-vector` | `src/transaction/db_transaction.rs:18` |
| `contextra-graph` | `apprh::diffusion` | `fn run_apprh_push` | `contextra-types` | `src/apprh/diffusion.rs:23` |
| `contextra-graph` | `apprh` | `fn apprh_local` | `contextra-types` | `src/apprh/mod.rs:35` |
| `contextra-graph` | `apprh` | `fn forward_push_apprh` | `contextra-types` | `src/apprh/mod.rs:51` |
| `contextra-graph` | `apprh::shadow` | `struct ApprhShadowComparison` | `contextra-types` | `src/apprh/shadow.rs:30` |
| `contextra-graph` | `apprh::shadow` | `fn shadow_compare_forward_push_vs_apprh` | `contextra-types` | `src/apprh/shadow.rs:54` |
| `contextra-graph` | `edge_reinforcement_buffer` | `struct CooccurrenceSignal` | `contextra-types` | `src/edge_reinforcement_buffer.rs:26` |
| `contextra-graph` | `edge_reinforcement_buffer` | `struct TraversalSignal` | `contextra-types` | `src/edge_reinforcement_buffer.rs:36` |
| `contextra-graph` | `error` | `pub use contextra_ports::graph::GraphMutationError` | `contextra-ports` | `src/error.rs:4` |
| `contextra-graph` | `hyperedge_suggest` | `fn validate_candidates_with_llm` | `contextra-ports` | `src/hyperedge_suggest.rs:118` |
| `contextra-graph` | `(crate-root)` | `trait GraphIndexExt` | `contextra-ports` | `src/lib.rs:123` |
| `contextra-graph` | `path_rag::k_path` | `struct KPathResult` | `contextra-types` | `src/path_rag/k_path.rs:35` |
| `contextra-graph` | `path_rag` | `struct GraphPath` | `contextra-types` | `src/path_rag/mod.rs:31` |
| `contextra-graph` | `path_rag` | `fn forward_push_ppr` | `contextra-types` | `src/path_rag/mod.rs:96` |
| `contextra-graph` | `path_rag` | `pub use contextra_types::EntityId` | `contextra-types` | `src/path_rag/mod.rs:25` |
| `contextra-graph` | `percolation` | `fn find_rebonding_candidates` | `contextra-ports`, `contextra-types` | `src/percolation.rs:79` |
| `contextra-graph` | `ppr::shadow_hook` | `fn log_tl_hfd_vs_baseline_discrepancy` | `contextra-types` | `src/ppr/shadow_hook.rs:10` |
| `contextra-graph` | `ppr_stream` | `struct PprCandidateStream` | `contextra-ports` | `src/ppr_stream.rs:23` |
| `contextra-graph` | `tl_hfd::diffusion` | `fn run_diffusion` | `contextra-types` | `src/tl_hfd/diffusion.rs:28` |
| `contextra-graph` | `tl_hfd::lovasz` | `fn truncate_participants` | `contextra-types` | `src/tl_hfd/lovasz.rs:14` |
| `contextra-graph` | `tl_hfd::lovasz` | `fn compute_lovasz_extension` | `contextra-types` | `src/tl_hfd/lovasz.rs:56` |
| `contextra-graph` | `tl_hfd` | `fn tl_hfd_local` | `contextra-types` | `src/tl_hfd/mod.rs:31` |
| `contextra-graph` | `tl_hfd::shadow` | `struct ShadowComparison` | `contextra-types` | `src/tl_hfd/shadow.rs:55` |
| `contextra-graph` | `tl_hfd::shadow` | `fn shadow_compare_forward_push_vs_tl_hfd` | `contextra-types` | `src/tl_hfd/shadow.rs:79` |

… 81 weitere (siehe `--json`).

## 3. Kantenliste

| Von | Nach | prod Fundstellen | prod Symbole | dev Fundstellen | Cargo-Art |
|---|---|---:|---:|---:|---|
| `contextra-engine` | `contextra-types` | 234 | 49 | 178 | normal |
| `contextra-vector` | `contextra-types` | 55 | 7 | 42 | normal |
| `contextra-checkpoint` | `contextra-ports` | 47 | 7 | 9 | normal |
| `contextra-graph` | `contextra-types` | 40 | 13 | 57 | normal |
| `contextra-store` | `contextra-ports` | 39 | 11 | 144 | normal |
| `contextra-crypto` | `contextra-types` | 35 | 14 | 18 | normal |
| `contextra-engine` | `contextra-ports` | 33 | 18 | 22 | normal |
| `contextra-adapt` | `contextra-types` | 30 | 9 | 1 | normal |
| `contextra-agent` | `contextra-types` | 26 | 9 | 59 | normal |
| `contextra-store` | `contextra-types` | 26 | 11 | 113 | normal |
| `contextra-engine` | `contextra-store` | 25 | 12 | 104 | normal |
| `contextra-graph` | `contextra-ports` | 25 | 17 | 41 | normal |
| `contextra-vector` | `contextra-ports` | 24 | 9 | 70 | normal |
| `contextra-engine` | `contextra-graph` | 20 | 16 | 75 | normal |
| `contextra-cognition` | `contextra-types` | 18 | 8 | 30 | normal |
| `contextra-infer-candle` | `contextra-ports` | 16 | 6 | 15 | normal |
| `contextra-agent` | `contextra-ports` | 15 | 5 | 18 | normal |
| `contextra-ports` | `contextra-types` | 14 | 9 | 0 | normal |
| `contextra-cognition` | `contextra-engine` | 13 | 6 | 23 | normal |
| `contextra-cognition` | `contextra-graph` | 12 | 11 | 18 | normal |
| `contextra-kvcache` | `contextra-ports` | 12 | 11 | 13 | normal |
| `contextra-rank` | `contextra-types` | 12 | 8 | 7 | normal |
| `contextra-testkit` | `contextra-ports` | 12 | 11 | 1 | normal |
| `contextra-core` | `contextra-types` | 11 | 8 | 0 | normal |
| `contextra-privacy` | `contextra-types` | 11 | 4 | 2 | normal |
| `contextra-db` | `contextra-cognition` | 10 | 10 | 0 | normal |
| `contextra-engine` | `contextra-vector` | 10 | 5 | 117 | dev/normal |
| `contextra-mcp` | `contextra-router` | 10 | 6 | 1 | normal |
| `contextra-store` | `contextra-mvcc` | 10 | 6 | 6 | normal |
| `contextra` | `contextra-types` | 9 | 6 | 25 | normal |
| `contextra-bench` | `contextra-core` | 9 | 6 | 0 | – |
| `contextra-mcp` | `contextra` | 9 | 6 | 10 | normal |
| `contextra-infer-ollama` | `contextra-types` | 8 | 3 | 3 | normal |
| `contextra-kvcache` | `contextra-crypto` | 8 | 5 | 3 | normal |
| `contextra-mcp` | `contextra-ports` | 8 | 4 | 43 | normal |
| `contextra-py` | `contextra-wire` | 8 | 4 | 0 | normal |
| `contextra-text` | `contextra-ports` | 8 | 2 | 87 | normal |
| `contextra-engine` | `contextra-text` | 7 | 4 | 68 | normal |
| `contextra-infer-candle` | `contextra-types` | 7 | 3 | 11 | normal |
| `contextra-infer-ollama` | `contextra-ports` | 7 | 6 | 7 | normal |
| `contextra-mcp` | `contextra-privacy` | 7 | 7 | 1 | normal |
| `contextra-py` | `contextra-db` | 7 | 7 | 3 | normal |
| `contextra-vector` | `contextra-mvcc` | 7 | 6 | 0 | normal |
| `contextra-core` | `contextra-mvcc` | 6 | 3 | 0 | normal |
| `contextra-db` | `contextra-engine` | 6 | 6 | 0 | normal |
| `contextra-engine` | `contextra-rank` | 6 | 5 | 0 | normal |
| `contextra-vector` | `contextra-crypto` | 6 | 1 | 2 | normal |
| `contextra` | `contextra-ports` | 5 | 4 | 12 | normal |
| `contextra-agent` | `contextra-db` | 5 | 3 | 38 | normal |
| `contextra-cognition` | `contextra-ports` | 5 | 4 | 6 | normal |
| `contextra-crypto` | `contextra-ports` | 5 | 5 | 34 | normal |
| `contextra-db` | `contextra-adapt` | 5 | 5 | 1 | normal |
| `contextra-infer-onnx` | `contextra-types` | 5 | 4 | 3 | normal |
| `contextra-kvcache` | `contextra-types` | 5 | 3 | 13 | normal |
| `contextra-mcp` | `contextra-types` | 5 | 3 | 5 | normal |
| `contextra-simd` | `contextra-types` | 5 | 2 | 1 | normal |
| `contextra-store` | `contextra-crypto` | 5 | 4 | 2 | normal |
| `contextra-agent` | `contextra-router` | 4 | 4 | 0 | normal |
| `contextra-engine` | `contextra-checkpoint` | 4 | 3 | 0 | normal |
| `contextra-engine` | `contextra-crypto` | 4 | 3 | 2 | normal |
| `contextra-infer-onnx` | `contextra-rank` | 4 | 1 | 1 | normal |
| `contextra-mcp` | `contextra-crypto` | 4 | 2 | 4 | normal |
| `contextra-mcp` | `contextra-infer-candle` | 4 | 3 | 0 | normal? |
| `contextra-router` | `contextra-adapt` | 4 | 4 | 4 | normal |
| `contextra-agent` | `contextra-store` | 3 | 1 | 0 | normal |
| `contextra-cognition` | `contextra-vector` | 3 | 1 | 16 | normal |
| `contextra-ingest` | `contextra-types` | 3 | 2 | 3 | normal |
| `contextra-license` | `contextra-ports` | 3 | 2 | 9 | normal |
| `contextra-mcp` | `contextra-infer-ollama` | 3 | 3 | 0 | normal? |
| `contextra-mcp` | `contextra-infer-onnx` | 3 | 3 | 0 | normal? |
| `contextra-mcp` | `contextra-rank` | 3 | 3 | 0 | normal |
| `contextra-mvcc` | `contextra-ports` | 3 | 1 | 1 | normal |
| `contextra-privacy` | `contextra-avv-generator` | 3 | 2 | 0 | normal |
| `contextra-privacy` | `contextra-ports` | 3 | 1 | 5 | normal |
| `contextra-py` | `contextra-types` | 3 | 3 | 0 | normal |
| `contextra-store` | `contextra-sys` | 3 | 2 | 4 | normal |
| `contextra-vector` | `contextra-simd` | 3 | 2 | 7 | normal |
| `contextra` | `contextra-license` | 2 | 2 | 2 | normal |
| `contextra` | `contextra-store` | 2 | 2 | 3 | normal |
| `contextra-adapt` | `contextra-ports` | 2 | 2 | 1 | normal |
| `contextra-audit-export` | `contextra-types` | 2 | 1 | 6 | normal |
| `contextra-bench` | `contextra-infer-candle` | 2 | 2 | 0 | – |
| `contextra-checkpoint` | `contextra-mvcc` | 2 | 1 | 2 | normal |
| `contextra-core` | `contextra-ports` | 2 | 1 | 0 | normal |
| `contextra-db` | `contextra-types` | 2 | 2 | 47 | normal |
| `contextra-durable-fs` | `contextra-sys` | 2 | 2 | 2 | normal |
| `contextra-engine` | `contextra-adapt` | 2 | 1 | 1 | normal? |
| `contextra-engine` | `contextra-infer-onnx` | 2 | 2 | 0 | normal? |
| `contextra-ingest` | `contextra-engine` | 2 | 1 | 2 | normal |
| `contextra-py` | `contextra-rank` | 2 | 1 | 0 | normal |
| `contextra-router` | `contextra-privacy` | 2 | 1 | 0 | normal |
| `contextra-vector` | `contextra-sys` | 2 | 1 | 0 | normal |
| `contextra` | `contextra-db` | 1 | 1 | 4 | normal |
| `contextra` | `contextra-infer-candle` | 1 | 1 | 0 | normal? |
| `contextra` | `contextra-rank` | 1 | 1 | 0 | normal |
| `contextra-avv-generator` | `contextra-types` | 1 | 1 | 2 | normal |
| `contextra-core` | `contextra-wire` | 1 | 1 | 0 | normal |
| `contextra-db` | `contextra-checkpoint` | 1 | 1 | 4 | normal |
| `contextra-db` | `contextra-graph` | 1 | 1 | 15 | normal |
| `contextra-db` | `contextra-ports` | 1 | 1 | 15 | normal |
| `contextra-db` | `contextra-sys` | 1 | 1 | 0 | normal |
| `contextra-engine` | `contextra-mvcc` | 1 | 1 | 5 | normal |
| `contextra-infer-candle` | `contextra-crypto` | 1 | 1 | 1 | dev/normal? |
| `contextra-infer-candle` | `contextra-rank` | 1 | 1 | 0 | normal |
| `contextra-infer-ollama` | `contextra-rank` | 1 | 1 | 0 | normal |
| `contextra-license` | `contextra-types` | 1 | 1 | 6 | normal |
| `contextra-mvcc` | `contextra-types` | 1 | 1 | 3 | normal |
| `contextra-router` | `contextra-ports` | 1 | 1 | 18 | normal |
| `contextra-router` | `contextra-types` | 1 | 1 | 16 | normal |
| `contextra-testkit` | `contextra-types` | 1 | 1 | 0 | normal |
| `contextra` | `contextra-crypto` | 0 | 0 | 1 | dev/normal? |
| `contextra` | `contextra-testkit` | 0 | 0 | 1 | dev |
| `contextra-agent` | `contextra-checkpoint` | 0 | 0 | 1 | normal |
| `contextra-agent` | `contextra-graph` | 0 | 0 | 2 | normal |
| `contextra-agent` | `contextra-testkit` | 0 | 0 | 3 | dev |
| `contextra-agent` | `contextra-text` | 0 | 0 | 5 | dev |
| `contextra-bench` | `contextra-sys` | 0 | 0 | 1 | dev |
| `contextra-checkpoint` | `contextra-store` | 0 | 0 | 1 | dev |
| `contextra-checkpoint` | `contextra-types` | 0 | 0 | 12 | normal |
| `contextra-cognition` | `contextra-infer-candle` | 0 | 0 | 1 | dev |
| `contextra-cognition` | `contextra-store` | 0 | 0 | 16 | normal |
| `contextra-cognition` | `contextra-testkit` | 0 | 0 | 2 | dev |
| `contextra-crypto` | `contextra-testkit` | 0 | 0 | 5 | dev |
| `contextra-db` | `contextra-crypto` | 0 | 0 | 4 | normal |
| `contextra-db` | `contextra-rank` | 0 | 0 | 1 | normal |
| `contextra-db` | `contextra-store` | 0 | 0 | 12 | normal |
| `contextra-db` | `contextra-testkit` | 0 | 0 | 1 | dev |
| `contextra-db` | `contextra-text` | 0 | 0 | 11 | normal |
| `contextra-db` | `contextra-vector` | 0 | 0 | 6 | dev/normal |
| `contextra-engine` | `contextra-testkit` | 0 | 0 | 2 | dev |
| `contextra-graph` | `contextra-store` | 0 | 0 | 2 | dev |
| `contextra-infer-onnx` | `contextra-ports` | 0 | 0 | 1 | normal |
| `contextra-mvcc` | `contextra-testkit` | 0 | 0 | 6 | dev |
| `contextra-privacy` | `contextra-crypto` | 0 | 0 | 2 | dev |
| `contextra-privacy` | `contextra-testkit` | 0 | 0 | 2 | dev |
| `contextra-rank` | `contextra-adapt` | 0 | 0 | 2 | dev |
| `contextra-router` | `contextra-db` | 0 | 0 | 14 | dev |
| `contextra-router` | `contextra-graph` | 0 | 0 | 1 | dev |
| `contextra-router` | `contextra-text` | 0 | 0 | 1 | dev |
| `contextra-router` | `contextra-vector` | 0 | 0 | 2 | dev |
| `contextra-store` | `contextra-sandbox` | 0 | 0 | 1 | dev |
| `contextra-store` | `contextra-testkit` | 0 | 0 | 8 | dev |
| `contextra-text` | `contextra-types` | 0 | 0 | 136 | normal |

## 4. Crates im Detail

### contextra

- Pfad: `crates/contextra` · Zeilen: 4156 · Rust-Dateien: 24 · Ring: 4
- Interne Cargo-Abhängigkeiten: `contextra-crypto` (dev), `contextra-testkit` (dev), `contextra-audit-export` (normal/optional via audit-export), `contextra-avv-generator` (normal/optional via avv-generator), `contextra-crypto` (normal/optional), `contextra-db` (normal), `contextra-infer-candle` (normal/optional via candle), `contextra-license` (normal), `contextra-ports` (normal), `contextra-privacy` (normal/optional), `contextra-rank` (normal), `contextra-store` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 9 | 25 | 5 | `error::ContextraError`×3, `DistanceMetric`×2, `types::domain::AutoExtractionMode`, `types::domain`, `error::Result`, `ScoredEntry` | `Result`×2, `ContextraError`×2, `DocId`×2, `DocId::from_u64`×2, `RequestContext` … (+4) |
| `contextra-ports` | 5 | 12 | 4 | `license`×2, `IdGen`, `license::LicenseGate`, `license::VectorDeleteMode` | `BoxFuture`×3, `license::GateToken`, `license::GateToken::for_test`, `SequentialIdGen`, `StorageEngine` … (+4) |
| `contextra-license` | 2 | 2 | 1 | `SignedLicenseGate`, `signed_gate::VerifyingKey::from_bytes` | `OpenFastGate` |
| `contextra-store` | 2 | 3 | 2 | `kv::delete_mode::KvDeleteMode`, `lsm::config` | `lsm::config::DurabilityMode`, `lsm::DurabilityMode` |
| `contextra-db` | 1 | 4 | 1 | `QueryRewriter` | `engine::ProofKeyProvider`, `EmbeddingBackend`, `ContextraConfig` |
| `contextra-infer-candle` | 1 | 0 | 1 | `(crate)` | – |
| `contextra-rank` | 1 | 0 | 1 | `(crate)` | – |
| `contextra-crypto` | 0 | 1 | 0 | – | `deletion_proof` |
| `contextra-testkit` | 0 | 1 | 0 | – | `mint_witness_for_tests` |

**Pro Komponente**

- `(crate-root)` (190 Zeilen)
  - → `contextra-infer-candle`: prod: `(crate)`
  - → `contextra-rank`: prod: `(crate)`
  - → `contextra-types`: prod: `error::ContextraError`, `DistanceMetric`, `types::domain`
- `agent_memory` (374 Zeilen)
  - → `contextra-ports`: prod: `IdGen`
  - → `contextra-types`: prod: `error::ContextraError`
- `builder` (489 Zeilen)
  - → `contextra-license`: prod: `SignedLicenseGate`, `signed_gate::VerifyingKey::from_bytes`
  - → `contextra-ports`: prod: `license`
  - → `contextra-types`: prod: `error::ContextraError`, `DistanceMetric`
- `collection_profile` (172 Zeilen)
  - → `contextra-ports`: prod: `license::LicenseGate`
  - → `contextra-store`: prod: `kv::delete_mode::KvDeleteMode`
  - → `contextra-types`: prod: `types::domain::AutoExtractionMode`
- `performance_profile` (240 Zeilen)
  - → `contextra-ports`: prod: `license::VectorDeleteMode`, `license` · dev: `license::GateToken`, `license::GateToken::for_test`
  - → `contextra-store`: prod: `lsm::config`
- `query_rewriter` (242 Zeilen)
  - → `contextra-db`: prod: `QueryRewriter`
  - → `contextra-types`: prod: `error::Result`, `ScoredEntry`
- `tests/agent_memory_facade.rs` (84 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `error::Result`
- `tests/agent_memory_hyperedges.rs` (149 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `error::Result`
- `tests/agent_memory_idgen_test.rs` (83 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`, `SequentialIdGen`
  - → `contextra-types`: dev: `error::Result`
- `tests/collection_profile_matrix.rs` (61 Zeilen)
  - → `contextra-store`: dev: `kv::delete_mode::KvDeleteMode`
- `tests/deletion_chain_e2e.rs` (519 Zeilen)
  - → `contextra-db`: dev: `engine::ProofKeyProvider`, `EmbeddingBackend`
  - → `contextra-ports`: dev: `StorageEngine`, `scanner`
  - → `contextra-types`: dev: `Result`×2, `RequestContext`, `TxId`
- `tests/license_gate_always_enforced.rs` (200 Zeilen)
  - → `contextra-db`: dev: `ContextraConfig`
  - → `contextra-license`: dev: `SignedLicenseGate`
  - → `contextra-ports`: dev: `license::FeatureRing`
  - → `contextra-store`: dev: `lsm::config::DurabilityMode`
  - → `contextra-types`: dev: `error::ContextraError`
- `tests/license_gate_enforcement.rs` (141 Zeilen)
  - → `contextra-license`: dev: `OpenFastGate`
  - → `contextra-ports`: dev: `license`
  - → `contextra-types`: dev: `error::ContextraError`
- `tests/llm_query_rewriter.rs` (206 Zeilen)
  - → `contextra-db`: dev: `QueryRewriter`
  - → `contextra-types`: dev: `error::ContextraError`, `ScoredEntry`, `error::Result`
- `tests/review_stub_proof.rs` (95 Zeilen)
  - → `contextra-crypto`: dev: `deletion_proof`
  - → `contextra-ports`: dev: `scanner::ScanTechnique`, `SystemClock`
  - → `contextra-testkit`: dev: `mint_witness_for_tests`
- `tests/type_identity_test.rs` (53 Zeilen)
  - → `contextra-types`: dev: `ContextraError`×2, `DocId`×2, `DocId::from_u64`×2, `DistanceMetric`×2 … (+4)
- `tests/wiring_matrix.rs` (127 Zeilen)
  - → `contextra-store`: dev: `lsm::DurabilityMode`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 3 fn
    - `fn builder`
    - `fn open` ⚠ fremd: `contextra-types`
    - `fn open_with_config` ⚠ fremd: `contextra-types`
- `agent_memory`
  - 4 struct
    - `struct MemoryId` · 2 Methoden
    - `struct RelationId`
    - `struct Memory`
    - `struct AgentMemory` · 10 Methoden
- `builder`
  - 1 enum, 1 struct, 1 trait
    - `enum EncryptionMode`
    - `trait ContextraConfigExt` · Methoden: is_encrypted
    - `struct ContextraBuilder` · 15 Methoden
- `collection_profile`
  - 2 const, 2 enum, 2 struct
    - `const DEFAULT_AUTO_EXTRACTION_MODE` ⚠ fremd: `contextra-types`
    - `const ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT` ⚠ fremd: `contextra-types`
    - `struct LsmTuning`
    - `struct CollectionProfile` · 2 Methoden ⚠ fremd: `contextra-store`, `contextra-types`
    - `enum CollectionProfileError`
    - `enum DeploymentTier` · 1 Methoden
- `performance_profile`
  - 2 enum, 1 struct
    - `enum PerformanceProfile` · 1 Methoden
    - `struct ResolvedProfileConfig` · 2 Methoden ⚠ fremd: `contextra-ports`
    - `enum PerformanceProfileError`
- `query_rewriter`
  - 1 struct
    - `struct LlmQueryRewriter` · 4 Methoden
- Zusätzlich 3 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::types::domain::AutoExtractionMode`, `contextra_types::error::ContextraError`, `contextra_types::DistanceMetric`, `contextra_rank`, `contextra_infer_candle`, `contextra_ports::license::VectorDeleteMode`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-mcp` | `Contextra`×3, `Collection`×2, `chunker`, `collection_profile::DeploymentTier`, `collection_profile::AutoExtractionMode::Enabled`, `collection_profile::AutoExtractionMode::Disabled` | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 19 von 20)**: `MemoryId`, `RelationId`, `Memory`, `AgentMemory`, `EncryptionMode`, `ContextraConfigExt`, `ContextraBuilder`, `DEFAULT_AUTO_EXTRACTION_MODE`, `ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT`, `LsmTuning`, `CollectionProfile`, `CollectionProfileError`, `builder`, `open`, `open_with_config`, `PerformanceProfile`, `ResolvedProfileConfig`, `PerformanceProfileError`, `LlmQueryRewriter`

**Auffälligkeiten**

- contextra-audit-export: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-avv-generator: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-crypto: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-privacy: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-adapt

- Pfad: `crates/contextra-adapt` · Zeilen: 6493 · Rust-Dateien: 27 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 30 | 1 | 3 | `ContextraError::InvalidInput`×9, `RetrievalStrategy`×9, `ContextraError`×4, `ContextraError::Internal`×2, `RetrievalStrategy::Vector`×2, `TenantId`, `RetrievalStrategy::Text`, `RetrievalStrategy::Graph`, `RetrievalStrategy::Hybrid` | – |
| `contextra-ports` | 2 | 1 | 1 | `SeededRng::new`, `Rng::next_u64` | `Clock` |

**Pro Komponente**

- `bandit` (1097 Zeilen)
  - → `contextra-ports`: prod: `SeededRng::new`, `Rng::next_u64`
  - → `contextra-types`: prod: `ContextraError::InvalidInput`×2, `ContextraError::Internal`×2, `ContextraError`, `TenantId`
- `flow_thompson` (830 Zeilen)
  - → `contextra-types`: prod: `RetrievalStrategy`×9, `ContextraError::InvalidInput`×4, `ContextraError`×2, `RetrievalStrategy::Vector`×2, `RetrievalStrategy::Text`, `RetrievalStrategy::Graph`, `RetrievalStrategy::Hybrid`
- `rie_greedy` (212 Zeilen)
  - → `contextra-types`: prod: `ContextraError::InvalidInput`×3, `ContextraError`
- `tests/controller_properties_and_convergence.rs` (369 Zeilen)
  - → `contextra-ports`: dev: `Clock`
- `tests/sketched_projection_determinism.rs` (102 Zeilen)
  - → `contextra-types`: dev: `TenantId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `bandit`
  - 1 const, 2 enum, 3 struct, 1 trait
    - `const SHERMAN_MORRISON_REFACTORIZATION_INTERVAL`
    - `enum BanditError`
    - `struct AlignedF32Vec` · 2 Methoden
    - `enum BanditImplementation`
    - `struct SketchMatrix` · 3 Methoden
    - `trait BanditPolicy` · Methoden: apply_drift_penalty
    - `struct BanditProfileState` · 6 Methoden
- `decay_controller`
  - 3 struct
    - `struct DecaySignalInputs`
    - `struct DecayControllerConfig`
    - `struct AdaptiveDecayController` · 7 Methoden
- `drift`
  - 1 enum, 3 struct, 1 trait
    - `enum DriftSignal`
    - `trait DriftDetector` · Methoden: observe, is_drifting
    - `struct CatoniDriftDetector` · 2 Methoden
    - `struct EnsembleDriftWatcher` · 3 Methoden
    - `struct DriftPolicyBridge` · 2 Methoden
- `flow_thompson`
  - 2 const, 2 enum, 1 fn, 9 struct, 3 trait
    - `enum FcTsError`
    - `trait FcTsRng` · Methoden: next_u64, next_f32, next_standard_normal
    - `struct SplitMix64` · 1 Methoden
    - `struct FcTsConfig` · 1 Methoden
    - `struct Ring3Token`
    - `fn ring3_background_task_token`
    - `struct FlowCorrectedThompsonBandit` · 12 Methoden
    - `struct DiagonalApproximationBandit` · 3 Methoden ⚠ fremd: `contextra-types`
    - `trait OffPolicyCompatibility` · Methoden: verify_positivity
    - `enum OffPolicyError` ⚠ fremd: `contextra-types`
    - `struct FcTsSamplingDistribution` · 3 Methoden ⚠ fremd: `contextra-types`
    - `struct FcTsArmSet` · 2 Methoden
    - `const MIN_BANDIT_SHADOW_SAMPLES`
    - `const MAX_CUMULATIVE_REGRET_THRESHOLD`
    - `struct BanditShadowReport`
    - `trait BanditDefaultFlipGate` · Methoden: should_flip
    - `struct DefaultBanditFlipGate`
- `homeostat`
  - 2 struct
    - `struct RerankPidController` · 9 Methoden
    - `struct RerankDeadline` · 5 Methoden
- `lyapunov`
  - 1 enum, 2 struct
    - `struct DriftReason`
    - `enum LyapunovResult` · 2 Methoden
    - `struct LyapunovDriftWatcher` · 7 Methoden
- `offpolicy`
  - 3 struct
    - `struct RandomizedLoggingPolicy` · 3 Methoden
    - `struct OffPolicyStats`
    - `struct OffPolicyEvaluator` · 7 Methoden
- `pid`
  - 2 const, 1 struct, 1 trait
    - `const PID_MIN_POOL_SIZE_DEFAULT`
    - `const PID_MAX_POOL_SIZE_DEFAULT`
    - `trait AntiWindupController` · Methoden: update_with_anti_windup
    - `struct PidController` · 4 Methoden
- `pid_latency_controller`
  - 4 const, 2 struct
    - `const DEFAULT_TARGET_LATENCY_MS`
    - `const MIN_SCALING_FACTOR`
    - `const MAX_SCALING_FACTOR`
    - `const MAX_INTEGRAL`
    - `struct PidLatencyController` · 4 Methoden
    - `struct LatencyBudgetGuard` · 5 Methoden
- `rie_greedy`
  - 1 enum, 1 struct
    - `enum RieGreedyError`
    - `struct RieGreedyProfile` · 5 Methoden
- `shadow_mode`
  - 2 struct, 1 trait
    - `struct ShadowDiscrepancy`
    - `trait ShadowSink` · Methoden: record
    - `struct TracingShadowSink`
- Interne Re-Exporte (`pub use`): `*`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-db` | `homeostat::RerankDeadline`, `homeostat::RerankPidController`, `decay_controller::*`, `pid_latency_controller::*`, `PidController` | – |
| `contextra-router` | `bandit`, `lyapunov`, `offpolicy`, `offpolicy::RandomizedLoggingPolicy` | `bandit::BanditProfileState`×2, `BanditError`, `FcTsError::InvalidConfig` |
| `contextra-engine` | `PidController`×2 | `PidController::default` |
| `contextra-rank` | – | `PidController`×2 |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

### contextra-agent

- Pfad: `crates/contextra-agent` · Zeilen: 9181 · Rust-Dateien: 41 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-testkit` (dev), `contextra-text` (dev), `contextra-vector` (dev), `contextra-checkpoint` (normal), `contextra-db` (normal), `contextra-graph` (normal), `contextra-ports` (normal), `contextra-router` (normal), `contextra-store` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 26 | 59 | 7 | `ContextraError::InvalidInput`×10, `Result`×4, `TxId`×3, `WorkflowState`×3, `TokenBudget::new`×2, `ContextraError::Internal`, `DocId::from_key`, `types::budget`, `error` | `TokenBudget`×13, `TxId::new`×4, `ContextraError::Conflict`, `ContextraError::Serialization`, `ContextraError` |
| `contextra-ports` | 15 | 18 | 5 | `BoxFuture`×7, `MetricsSink`×3, `StorageEngine`×2, `VectorIndex`×2, `Checkpoint` | `SystemClock` |
| `contextra-db` | 5 | 38 | 4 | `Collection`×2, `Contextra`×2, `volatile_vault` | `Contextra::open_with_config`×17, `ContextraConfig::default`×14, `volatile_vault::VaultConfig`×3, `ContextraConfig`×2, `DistanceMetric::Cosine` |
| `contextra-router` | 4 | 0 | 2 | `DefaultRouterEngine`, `DecisionId`, `RoutingOutcome::Success`, `RoutingOutcome::Rejected` | – |
| `contextra-store` | 3 | 0 | 3 | `LsmStorage`×3 | – |
| `contextra-checkpoint` | 0 | 1 | 0 | – | `StateCheckpoint` |
| `contextra-graph` | 0 | 2 | 0 | – | `CsrGraph`×2 |
| `contextra-testkit` | 0 | 3 | 0 | – | `InMemoryStorageEngine`×2, `TestMetricsSink` |
| `contextra-text` | 0 | 5 | 0 | – | `Language::English`×5 |

**Pro Komponente**

- `audit` (449 Zeilen)
  - → `contextra-db`: prod: `Collection`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: prod: `VectorIndex`×2, `StorageEngine`
  - → `contextra-store`: prod: `LsmStorage`
  - → `contextra-testkit`: dev: `InMemoryStorageEngine`
  - → `contextra-text`: dev: `Language::English`×4
  - → `contextra-types`: prod: `ContextraError::InvalidInput`×4, `Result`, `TxId`, `ContextraError::Internal`, `DocId::from_key` · dev: `ContextraError::InvalidInput`×5, `ContextraError::Conflict`, `Result`, `ContextraError::Internal` … (+3)
- `budget` (73 Zeilen)
  - → `contextra-types`: prod: `types::budget`, `Result`
- `clm_scratchpad` (671 Zeilen)
  - → `contextra-db`: prod: `volatile_vault`
- `context` (520 Zeilen)
  - → `contextra-db`: dev: `ContextraConfig::default`×4, `Contextra::open_with_config`×4, `volatile_vault::VaultConfig`
  - → `contextra-router`: prod: `DefaultRouterEngine`, `DecisionId`
  - → `contextra-store`: prod: `LsmStorage`
  - → `contextra-types`: prod: `ContextraError::InvalidInput`×2, `error`, `Result` · dev: `Result`×4
- `dlq` (359 Zeilen)
  - → `contextra-ports`: prod: `MetricsSink`×3, `StorageEngine` · dev: `MetricsSink`
  - → `contextra-testkit`: dev: `InMemoryStorageEngine`, `TestMetricsSink`
- `engine` (1145 Zeilen)
  - → `contextra-db`: prod: `Contextra`×2 · dev: `volatile_vault::VaultConfig`
  - → `contextra-ports`: prod: `BoxFuture`×5, `Checkpoint` · dev: `BoxFuture`×2
  - → `contextra-router`: prod: `RoutingOutcome::Success`, `RoutingOutcome::Rejected`
  - → `contextra-store`: prod: `LsmStorage`
  - → `contextra-types`: prod: `WorkflowState`×3, `TxId`×2, `TokenBudget::new`×2 · dev: `Result`×2, `TokenBudget`, `TxId`
- `event_source` (261 Zeilen)
  - → `contextra-db`: prod: `Collection`
  - → `contextra-types`: prod: `ContextraError::InvalidInput`×3
- `graph` (290 Zeilen)
  - → `contextra-types`: prod: `ContextraError::InvalidInput`
- `step` (104 Zeilen)
  - → `contextra-ports`: prod: `BoxFuture`
- `tools` (444 Zeilen)
  - → `contextra-ports`: prod: `BoxFuture` · dev: `BoxFuture`
  - → `contextra-types`: prod: `Result`
- `tests/agent_recovery.rs` (383 Zeilen)
  - → `contextra-checkpoint`: dev: `StateCheckpoint`
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `TxId::new`×3, `Result`×2, `TokenBudget`, `ContextraError::Internal`
- `tests/audit_isolation_test.rs` (371 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
- `tests/boundary_validation_tests.rs` (133 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `ContextraError`, `Result`
- `tests/budget_race_test.rs` (301 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `ContextraError::Internal`
- `tests/clm_scratchpad_reset_flow.rs` (96 Zeilen)
  - → `contextra-db`: dev: `volatile_vault::VaultConfig`
  - → `contextra-ports`: dev: `SystemClock`
- `tests/clock_tests.rs` (154 Zeilen)
  - → `contextra-db`: dev: `Contextra::open_with_config`×4, `ContextraConfig::default`×2
- `tests/contract_tests.rs` (208 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`, `StorageEngine`
  - → `contextra-types`: dev: `TokenBudget`, `Result`
- `tests/crash_recovery_injection.rs` (181 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `Result`
- `tests/dlq_clock_determinism_test.rs` (104 Zeilen)
  - → `contextra-db`: dev: `ContextraConfig::default`, `Contextra::open_with_config`
- `tests/dlq_tests.rs` (300 Zeilen)
  - → `contextra-db`: dev: `ContextraConfig::default`×5, `Contextra::open_with_config`×5
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `TxId`×2
- `tests/e2e_integration.rs` (184 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `TokenBudget`, `Result`
- `tests/event_loop_integration.rs` (316 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `Result`×3, `TokenBudget`
- `tests/final_state_test.rs` (59 Zeilen)
  - → `contextra-db`: dev: `Contextra`, `ContextraConfig`
  - → `contextra-types`: dev: `TokenBudget`, `Result`
- `tests/graph_integration.rs` (106 Zeilen)
  - → `contextra-db`: dev: `ContextraConfig`, `DistanceMetric::Cosine`, `Contextra::open_with_config`
  - → `contextra-types`: dev: `TokenBudget::new`
- `tests/loop_protection_test.rs` (181 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `Result`
- `tests/persistence_test.rs` (142 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `TokenBudget`, `Result`
- `tests/regression_agt_a.rs` (168 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `Result`
- `tests/system_prompt_pinning_latency.rs` (77 Zeilen)
  - → `contextra-db`: dev: `ContextraConfig::default`×2, `Contextra::open_with_config`×2
  - → `contextra-types`: dev: `Result`×2, `TokenBudget`, `error`
- `tests/workflow_tests.rs` (510 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `TokenBudget`×5, `ContextraError::InvalidInput`×2, `Result`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `audit`
  - 2 fn, 3 struct
    - `struct AuditEntry` ⚠ fremd: `contextra-types`
    - `struct MigrationStats`
    - `struct AuditLog` · 4 Methoden ⚠ fremd: `contextra-ports`, `contextra-store`
    - `fn validate_audit_payload_and_error` ⚠ fremd: `contextra-types`
    - `fn migrate_legacy_audit_entries` ⚠ fremd: `contextra-db`, `contextra-ports`, `contextra-types`
- `budget`
  - 1 fn
    - `fn reserve_tokens` ⚠ fremd: `contextra-types`
- `clm_scratchpad`
  - 1 enum, 8 struct, 2 trait
    - `struct ContextEditAuditRecord`
    - `trait ContextEditAuditSink` · Methoden: record_edit
    - `struct NoopContextEditAuditSink`
    - `struct CountingContextEditAuditSink` · 2 Methoden
    - `trait ScratchpadCacheInvalidator` · Methoden: invalidate_scratchpad_scope
    - `struct NoopScratchpadCacheInvalidator`
    - `struct CountingScratchpadCacheInvalidator` · 2 Methoden
    - `struct PinnedRegionId` · 2 Methoden
    - `enum ScratchpadEditOp`
    - `struct ScratchpadCheckpoint`
    - `struct ClmScratchpad` · 15 Methoden
- `context`
  - 2 const, 1 enum, 2 fn, 2 struct
    - `const MAX_ID_LEN`
    - `const MAX_TELEMETRY_EVENTS`
    - `fn validate_task_id` ⚠ fremd: `contextra-types`
    - `fn validate_node_id` ⚠ fremd: `contextra-types`
    - `enum AgentStatus`
    - `struct AgentContext` · 9 Methoden ⚠ fremd: `contextra-db`, `contextra-router`, `contextra-store`
    - `struct AgentEngine` · 3 Methoden
- `dlq`
  - 1 struct
    - `struct DeadLetterQueue` · 9 Methoden
- `engine`
  - 1 const, 1 enum, 1 fn, 1 struct
    - `fn evaluate_condition_expr` *(re-exportiert)*
    - `enum EventLoopExitReason`
    - `const MAX_WORKFLOW_STEPS`
    - `struct OrchestratorEngine` · 12 Methoden
- `event_source`
  - 2 const, 3 struct, 1 trait
    - `const MAX_EVENT_SOURCE_CAPACITY`
    - `struct BackgroundEvent` · 2 Methoden
    - `trait EventSource` · Methoden: next_event, is_exhausted, wait_for_event
    - `const MAX_PENDING_EVENTS_CAPACITY`
    - `struct PollingDocumentEventSource` · 4 Methoden
    - `struct VecEventSource` · 1 Methoden
- `goal_condition`
  - 1 enum, 1 fn
    - `enum GoalCondition` · 2 Methoden
    - `fn parse_legacy_condition_string`
- `graph`
  - 1 const, 1 enum, 3 struct, 1 type
    - `type NodeId`
    - `const MAX_TEXT_LEN`
    - `enum NodeType`
    - `struct AgentNode`
    - `struct WorkflowEdge` · 2 Methoden
    - `struct StateGraph` · 7 Methoden
- `step`
  - 1 enum, 2 struct, 1 trait
    - `struct StepDeadLetter`
    - `enum DeadLetterReason`
    - `struct StepResult`
    - `trait AgentTool` · Methoden: name, estimated_cost, execute, timeout_ms, is_retriable, max_retries
- `tools`
  - 1 const, 3 struct
    - `const MAX_MANIFEST_DESC_LEN`
    - `struct ToolManifest` · 5 Methoden
    - `struct ToolRegistry` · 9 Methoden
    - `struct RetrieveTool` · 2 Methoden

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 51 von 51)**: `AuditEntry`, `MigrationStats`, `AuditLog`, `validate_audit_payload_and_error`, `migrate_legacy_audit_entries`, `reserve_tokens`, `ContextEditAuditRecord`, `ContextEditAuditSink`, `NoopContextEditAuditSink`, `CountingContextEditAuditSink`, `ScratchpadCacheInvalidator`, `NoopScratchpadCacheInvalidator`, `CountingScratchpadCacheInvalidator`, `PinnedRegionId`, `ScratchpadEditOp`, `ScratchpadCheckpoint`, `ClmScratchpad`, `MAX_ID_LEN`, `MAX_TELEMETRY_EVENTS`, `validate_task_id`, `validate_node_id`, `AgentStatus`, `AgentContext`, `AgentEngine`, `DeadLetterQueue`, `evaluate_condition_expr`, `EventLoopExitReason`, `MAX_WORKFLOW_STEPS`, `OrchestratorEngine`, `MAX_EVENT_SOURCE_CAPACITY` …

**Auffälligkeiten**

- contextra-checkpoint: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-graph: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-vector: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-audit-export

- Pfad: `crates/contextra-audit-export` · Zeilen: 1229 · Rust-Dateien: 11 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 2 | 6 | 2 | `TenantId`×2 | – |

**Pro Komponente**

- `(crate-root)` (258 Zeilen)
  - → `contextra-types`: prod: `TenantId`
- `testkit` (117 Zeilen)
  - → `contextra-types`: prod: `TenantId`
- `tests/campaign_j27_compliance_test.rs` (92 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/error_and_edge_cases.rs` (58 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/golden_file_test.rs` (99 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/register_rendering.rs` (61 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/sanitization_tests.rs` (66 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/validation_tests.rs` (189 Zeilen)
  - → `contextra-types`: dev: `TenantId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 fn, 6 struct, 1 trait
    - `struct ControllerContact` · 1 Methoden
    - `struct ThirdCountryTransfer` · 1 Methoden
    - `struct RetentionPeriod` · 1 Methoden
    - `struct EgressEventSummary`
    - `struct DeletionProofSummary`
    - `struct ProcessingRegisterEntry` · 2 Methoden ⚠ fremd: `contextra-types`
    - `trait ProcessingRegisterSource` · Methoden: collect_entries
    - `fn render_register_json`
- `bsi_mapping`
  - 2 fn, 1 struct
    - `struct BsiMappingEntry`
    - `fn bsi_mapping_table`
    - `fn render_bsi_mapping_markdown`
- `error`
  - 1 enum
    - `enum AuditExportError`
- `markdown_template`
  - 2 fn
    - `fn sanitize_markdown_cell`
    - `fn render_register_markdown`
- `testkit`
  - 1 struct
    - `struct InMemoryProcessingRegisterSource` · 4 Methoden

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 15 von 15)**: `BsiMappingEntry`, `bsi_mapping_table`, `render_bsi_mapping_markdown`, `AuditExportError`, `ControllerContact`, `ThirdCountryTransfer`, `RetentionPeriod`, `EgressEventSummary`, `DeletionProofSummary`, `ProcessingRegisterEntry`, `ProcessingRegisterSource`, `render_register_json`, `sanitize_markdown_cell`, `render_register_markdown`, `InMemoryProcessingRegisterSource`

**Auffälligkeiten**

- contextra-ports: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-avv-generator

- Pfad: `crates/contextra-avv-generator` · Zeilen: 1424 · Rust-Dateien: 11 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 1 | 2 | 1 | `TenantId` | – |

**Pro Komponente**

- `(crate-root)` (289 Zeilen)
  - → `contextra-types`: prod: `TenantId`
- `tests/avv_rendering.rs` (363 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/campaign_j27_avv_test.rs` (93 Zeilen)
  - → `contextra-types`: dev: `TenantId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 3 fn, 2 struct
    - `struct AvvContext` · 4 Methoden ⚠ fremd: `contextra-types`
    - `struct TechnicalMeasure`
    - `fn render_avv_markdown`
    - `fn render_avv_markdown_with_date`
    - `fn default_technical_measures`
- `assistance`
  - 1 fn
    - `fn render_section`
- `audit`
  - 1 fn
    - `fn render_section`
- `confidentiality`
  - 1 fn
    - `fn render_section`
- `data_subject_rights`
  - 1 fn
    - `fn render_section`
- `error`
  - 1 enum, 1 type
    - `enum AvvError`
    - `type AvvGeneratorError`
- `instructions`
  - 1 fn
    - `fn render_section`
- `template`
  - 3 fn
    - `fn escape_markdown`
    - `fn render`
    - `fn render_with_date`
- `termination`
  - 1 fn
    - `fn render_section`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-privacy` | `AvvContext`×2, `render_avv_markdown` | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 14 von 16)**: `render_section`, `render_section`, `render_section`, `render_section`, `AvvError`, `AvvGeneratorError`, `render_section`, `TechnicalMeasure`, `render_avv_markdown_with_date`, `default_technical_measures`, `escape_markdown`, `render`, `render_with_date`, `render_section`

### contextra-bench

- Pfad: `benchmarks/contextra-bench` · Zeilen: 4142 · Rust-Dateien: 12 · Ring: tooling
- Interne Cargo-Abhängigkeiten: `contextra-sys` (dev), `contextra-db` (normal), `contextra-store` (normal), `contextra-text` (normal), `contextra-vector` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-core` | 9 | 0 | 3 | `ContextraError::Internal`×3, `ContextraError::SnapshotUnsupportedForSignal`×2, `traits::VectorIndex`, `types`, `ContextraError::CapabilityUnsupported`, `Result` | – |
| `contextra-infer-candle` | 2 | 0 | 1 | `CandleEmbedClient::from_dir`, `model_registry::CandleQuantization::Q4KM` | – |
| `contextra-sys` | 0 | 1 | 0 | – | `ensure_fd_limit` |

**Pro Komponente**

- `(crate-root)` (1388 Zeilen)
  - → `contextra-core`: prod: `ContextraError::Internal`×3, `ContextraError::CapabilityUnsupported`
  - → `contextra-infer-candle`: prod: `CandleEmbedClient::from_dir`, `model_registry::CandleQuantization::Q4KM`
- `ann_benchmarks` (240 Zeilen)
  - → `contextra-core`: prod: `traits::VectorIndex`, `types`
- `path_rag_sweep` (318 Zeilen)
  - → `contextra-core`: prod: `ContextraError::SnapshotUnsupportedForSignal`×2, `Result`
- `benches/competitive_kv.rs` (553 Zeilen)
  - → `contextra-sys`: dev: `ensure_fd_limit`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `ann_benchmarks`
  - 5 fn, 1 struct
    - `struct AnnBenchResult`
    - `fn generate_synthetic_vectors`
    - `fn compute_ground_truth`
    - `fn recall_at_k`
    - `fn run_ann_benchmark`
    - `fn run_ann_benchmark_synthetic_ci`
- `beir_eval`
  - 6 fn, 3 struct
    - `struct BeirDocument`
    - `struct BeirQuery`
    - `struct BeirMetrics`
    - `fn load_beir_corpus`
    - `fn load_beir_queries`
    - `fn ndcg_at_k`
    - `fn map_at_k`
    - `fn recall_at_k`
    - `fn run_beir_eval`
- `compare`
  - 2 fn, 4 struct
    - `struct LongMemEvalMetricsSummary`
    - `struct LocomoMetricsSummary`
    - `struct CombinedMetrics`
    - `struct MetricComparisonResult`
    - `fn compare_metrics`
    - `fn compare_metrics_files`
- `path_rag_sweep`
  - 2 fn, 2 struct
    - `struct PathRagSweepMetric`
    - `struct PathRagSweepReport`
    - `fn run_pathrag_sweep_long_mem_eval` ⚠ fremd: `contextra-core`
    - `fn run_pathrag_sweep_locomo` ⚠ fremd: `contextra-core`
- `regression_gate`
  - 1 const, 1 fn, 1 struct
    - `const DEFAULT_TOLERANCE_THRESHOLD`
    - `struct RegressionGateResult`
    - `fn run_regression_gate`
- Zusätzlich 7 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 28 von 28)**: `AnnBenchResult`, `generate_synthetic_vectors`, `compute_ground_truth`, `recall_at_k`, `run_ann_benchmark`, `run_ann_benchmark_synthetic_ci`, `BeirDocument`, `BeirQuery`, `BeirMetrics`, `load_beir_corpus`, `load_beir_queries`, `ndcg_at_k`, `map_at_k`, `recall_at_k`, `run_beir_eval`, `LongMemEvalMetricsSummary`, `LocomoMetricsSummary`, `CombinedMetrics`, `MetricComparisonResult`, `compare_metrics`, `compare_metrics_files`, `PathRagSweepMetric`, `PathRagSweepReport`, `run_pathrag_sweep_long_mem_eval`, `run_pathrag_sweep_locomo`, `DEFAULT_TOLERANCE_THRESHOLD`, `RegressionGateResult`, `run_regression_gate`

**Auffälligkeiten**

- contextra-db: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-store: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-text: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-vector: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-core: genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler)
- contextra-infer-candle: genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler)

### contextra-checkpoint

- Pfad: `crates/contextra-checkpoint` · Zeilen: 8374 · Rust-Dateien: 26 · Ring: 1
- Interne Cargo-Abhängigkeiten: `contextra-store` (dev), `contextra-testkit` (dev), `contextra-mvcc` (normal), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 47 | 9 | 4 | `Clock`×19, `StorageEngine`×14, `MetricsSink`×6, `BoxFuture`×3, `SystemClock`×2, `Checkpoint`×2, `CheckpointCoordinator` | – |
| `contextra-mvcc` | 2 | 2 | 2 | `SnapshotRegistry`×2 | – |
| `contextra-store` | 0 | 1 | 0 | – | `lsm` |
| `contextra-types` | 0 | 12 | 0 | – | `TxId`×5, `Result`×3, `ContextraError`×2, `ContextraError::Internal`×2 |

**Pro Komponente**

- `guard` (1100 Zeilen)
  - → `contextra-ports`: prod: `Clock`×9, `StorageEngine`×6, `MetricsSink`×3, `SystemClock` · dev: `MetricsSink`×2, `Clock`
- `hardlink_cloner` (235 Zeilen)
  - → `contextra-mvcc`: prod: `SnapshotRegistry`
  - → `contextra-ports`: prod: `BoxFuture`×2
- `manifest` (210 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `orphan` (766 Zeilen)
  - → `contextra-ports`: prod: `Clock`×7, `SystemClock`, `StorageEngine`
- `store` (1660 Zeilen)
  - → `contextra-mvcc`: prod: `SnapshotRegistry`
  - → `contextra-ports`: prod: `StorageEngine`×7, `Clock`×3, `MetricsSink`×3, `Checkpoint`×2, `BoxFuture`, `CheckpointCoordinator` · dev: `CheckpointCoordinator`×5
- `tests/checkpoint_clock_determinism.rs` (178 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/checkpoint_systematic_crash.rs` (227 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-store`: dev: `lsm`
  - → `contextra-types`: dev: `TxId`
- `tests/clock_determinism.rs` (114 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/crash_recovery_and_tenant_isolation.rs` (562 Zeilen)
  - → `contextra-mvcc`: dev: `SnapshotRegistry`
- `tests/hardlink_cloner_tests.rs` (130 Zeilen)
  - → `contextra-mvcc`: dev: `SnapshotRegistry`
  - → `contextra-types`: dev: `Result`×3, `ContextraError`
- `tests/manifest_fault_injection.rs` (82 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/time_travel_correctness.rs` (737 Zeilen)
  - → `contextra-types`: dev: `ContextraError::Internal`×2, `ContextraError`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `manifest`
  - 1 struct
    - `struct CheckpointManifest` *(re-exportiert)* · 2 Methoden
- Zusätzlich 36 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `InstanceOrphanRegistry`×2, `InstanceOrphanRegistry::new`, `(crate)` | – |
| `contextra-db` | `(crate)` | `PersistentCheckpointStore`×2, `clear_all_orphaned_checkpoints`, `get_orphaned_checkpoints` |
| `contextra-agent` | – | `StateCheckpoint` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 1 von 1)**: `CheckpointManifest`

**Auffälligkeiten**

- contextra-testkit: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-types: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?

### contextra-cognition

- Pfad: `crates/contextra-cognition` · Zeilen: 9382 · Rust-Dateien: 31 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-infer-candle` (dev), `contextra-testkit` (dev), `contextra-engine` (normal), `contextra-graph` (normal), `contextra-ports` (normal), `contextra-store` (normal), `contextra-types` (normal), `contextra-vector` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 18 | 30 | 3 | `Result`×3, `TenantId`×3, `ContextraError::InvalidInput`×3, `ContextraError::Internal`×2, `ContextChunk`×2, `ContextraError::PolicyViolation`×2, `TxId`×2, `DocId` | `MemoryType::Episodic`×6, `DocId::new`×5 |
| `contextra-engine` | 13 | 23 | 9 | `collection`×4, `collection::Collection`×3, `decay_controller::AdaptiveDecayController`×2, `transaction::CommitIntent`×2, `ProvenanceRecord`, `decay_controller::DecayControllerConfig` | `Language::English`×9, `SearchResult`×2, `collection::Collection::new`×2, `Language::German` |
| `contextra-graph` | 12 | 18 | 6 | `EdgeReinforcementBuffer`×2, `HyperEdgeId`, `cascade::CascadeInvalidationReport`, `cascade_invalidate_edges_for_superseded_doc`, `csr::EdgeType`, `CsrGraph`, `hyperedge`, `hyperedge::HyperEdgeId`, `percolation::PercolationConfig`, `EdgeReinforcementConfig` … (+1) | – |
| `contextra-ports` | 5 | 6 | 4 | `SegmentSynthesizer`×2, `LlmTextGenerator`, `StorageEngine`, `SeededRng::from_entropy` | `SeededRng`×2, `BoxFuture`×2, `ResponseGroundingValidator` |
| `contextra-vector` | 3 | 16 | 3 | `HnswIndex`×3 | `HnswConfig`×8 |
| `contextra-infer-candle` | 0 | 1 | 0 | – | `gasp` |
| `contextra-store` | 0 | 16 | 0 | – | `LsmStorage`×8, `LsmConfig`×8 |
| `contextra-testkit` | 0 | 2 | 0 | – | `ManualClock`, `MockEmbedder` |

**Pro Komponente**

- `aggregation_phase` (789 Zeilen)
  - → `contextra-graph`: prod: `HyperEdgeId`
  - → `contextra-ports`: prod: `LlmTextGenerator`
- `consolidation_executor` (1050 Zeilen)
  - → `contextra-engine`: prod: `decay_controller::AdaptiveDecayController`, `collection` · dev: `Language::English`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-types`: prod: `ContextraError::Internal`×2, `ContextChunk`×2, `TxId`×2, `ContextraError::PolicyViolation` · dev: `MemoryType::Episodic`×2, `DocId::new`×2, `Result`, `ContextraError::Internal` … (+1)
  - → `contextra-vector`: prod: `HnswIndex` · dev: `HnswIndex`, `HnswConfig`
- `consolidation_locks` (116 Zeilen)
  - → `contextra-engine`: prod: `collection::Collection`
  - → `contextra-graph`: prod: `cascade::CascadeInvalidationReport`, `cascade_invalidate_edges_for_superseded_doc`
- `context` (583 Zeilen)
  - → `contextra-engine`: dev: `SearchResult`×2, `ProvenanceRecord`×2
  - → `contextra-types`: dev: `DocId`×2, `ContextraError::InvalidInput`
- `context_compaction` (1360 Zeilen)
  - → `contextra-engine`: prod: `transaction::CommitIntent`×2, `collection::Collection`, `ProvenanceRecord`, `collection` · dev: `Language::English`×2, `collection::Collection`, `transaction::CommitIntent`, `ProvenanceRecord`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: prod: `StorageEngine`, `SeededRng::from_entropy` · dev: `SeededRng`×2
  - → `contextra-vector`: prod: `HnswIndex`
- `graph_sink` (94 Zeilen)
  - → `contextra-graph`: prod: `csr::EdgeType`, `CsrGraph`, `hyperedge`
- `leanrag_input` (120 Zeilen)
  - → `contextra-engine`: prod: `collection::Collection`
  - → `contextra-graph`: prod: `hyperedge::HyperEdgeId`
- `maintenance_config` (96 Zeilen)
  - → `contextra-engine`: prod: `decay_controller::DecayControllerConfig`
  - → `contextra-graph`: prod: `percolation::PercolationConfig`, `EdgeReinforcementConfig`, `EdgeReinforcementConfig::default`
- `maintenance_scheduler` (540 Zeilen)
  - → `contextra-engine`: prod: `decay_controller::AdaptiveDecayController`, `collection` · dev: `Language::English`
  - → `contextra-graph`: prod: `EdgeReinforcementBuffer`×2 · dev: `CsrGraph`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-vector`: prod: `HnswIndex` · dev: `HnswIndex`, `HnswConfig`
- `memory_consolidation` (1168 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: prod: `Result`×3, `TenantId`×3, `ContextraError::InvalidInput`×3, `ContextraError::PolicyViolation` · dev: `Result`×2, `ContextraError::Internal`
- `semantic_aggregation_facade` (114 Zeilen)
  - → `contextra-engine`: prod: `collection`
- `synthesis_phase` (208 Zeilen)
  - → `contextra-ports`: prod: `SegmentSynthesizer`×2 · dev: `BoxFuture`
  - → `contextra-types`: prod: `DocId` · dev: `Result`, `ContextraError::Internal`
- `tests/aggregation_phase_tests.rs` (372 Zeilen)
  - → `contextra-graph`: dev: `HyperEdgeId`
- `tests/grounding_in_consolidation_pipeline.rs` (216 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection::new`, `Language::English`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-types`: dev: `MemoryType::Episodic`×2, `ContextraError::PolicyViolation`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/grounding_validator_rich_path_test.rs` (180 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection::new`, `Language::German`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-infer-candle`: dev: `gasp`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-types`: dev: `DocId::new`×3, `MemoryType::Episodic`×2, `Result`, `ContextChunk` … (+1)
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/leanrag_aggregation.rs` (405 Zeilen)
  - → `contextra-graph`: dev: `HyperEdgeId`
- `tests/leanrag_stage3_wiring.rs` (191 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection`, `Language::English`
  - → `contextra-graph`: dev: `csr::EdgeType`, `CsrGraph`, `hyperedge`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-types`: dev: `Result`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/leanrag_worker_opt_in.rs` (387 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection`, `Language::English`
  - → `contextra-graph`: dev: `csr::EdgeType`, `CsrGraph`, `hyperedge`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-types`: dev: `Result`, `ContextraError::Internal`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/maintenance_scheduler_tests.rs` (230 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection`, `Language::English`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-testkit`: dev: `ManualClock`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/pre_condensed_input_intake.rs` (242 Zeilen)
  - → `contextra-ports`: dev: `ResponseGroundingValidator`
- `tests/rng_seed_determinism.rs` (183 Zeilen)
  - → `contextra-graph`: dev: `HyperEdgeId`
- `tests/semantic_aggregation_facade.rs` (275 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection`, `Language::English`
  - → `contextra-graph`: dev: `csr::EdgeType`, `CsrGraph`, `hyperedge`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-testkit`: dev: `MockEmbedder`
  - → `contextra-types`: dev: `Result`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `aggregation_phase`
  - 3 fn, 7 struct, 1 trait
    - `struct AggregationConfig` · 1 Methoden
    - `struct AggregationPhaseResult`
    - `struct ConsolidationPipelineResult`
    - `struct AggregationNode`
    - `struct AggregationEdge` ⚠ fremd: `contextra-graph`
    - `struct SuperEdgeDraft` ⚠ fremd: `contextra-graph`
    - `struct AlphaNode`
    - `trait SuperEdgeSink` · Methoden: tombstone_edge, write_super_edge, commit
    - `fn check_compaction_budget`
    - `fn compute_entity_community_hash`
    - `fn run_aggregation_pass` ⚠ fremd: `contextra-ports`
- `consolidation_executor`
  - 7 fn, 3 struct
    - `struct ConsolidationLockGuard` · 1 Methoden
    - `fn execute_consolidation_pass` ⚠ fremd: `contextra-engine`, `contextra-types`
    - `struct GroundingValidatorAdapter`
    - `fn execute_background_consolidation` ⚠ fremd: `contextra-engine`, `contextra-types`
    - `fn execute_background_consolidation_with_rich_validator` ⚠ fremd: `contextra-engine`, `contextra-types`
    - `fn execute_leanrag_aggregation_stage` ⚠ fremd: `contextra-engine`, `contextra-types`
    - `fn execute_sleep_cycle` ⚠ fremd: `contextra-engine`, `contextra-types`
    - `fn start_consolidation_worker` ⚠ fremd: `contextra-engine`
    - `fn start_consolidation_reaper` ⚠ fremd: `contextra-engine`
    - `struct ConsolidationEngine` · 9 Methoden ⚠ fremd: `contextra-vector`
- `consolidation_locks`
  - 1 struct
    - `struct ConsolidationNodesGuard` · 3 Methoden
- `context`
  - 2 struct
    - `struct ContextManager` · 6 Methoden
    - `struct SpatialFence` · 2 Methoden
- `context_compaction`
  - 1 fn, 2 struct
    - `fn cleanup_orphaned_consolidation_intents` *(re-exportiert)* ⚠ fremd: `contextra-ports`
    - `struct ContextCompactor` *(re-exportiert)* · 7 Methoden
    - `struct ConsolidationSession` *(re-exportiert)* · 7 Methoden ⚠ fremd: `contextra-vector`
- `graph_sink`
  - 1 struct
    - `struct CsrGraphSuperEdgeSink` · 1 Methoden
- `leanrag_input`
  - 1 const, 1 fn, 1 struct
    - `const DEFAULT_MAX_LEANRAG_NODES`
    - `struct LeanRagInputs`
    - `fn build_leanrag_inputs` ⚠ fremd: `contextra-engine`
- `maintenance_config`
  - 2 struct
    - `struct PercolationConfig` ⚠ fremd: `contextra-graph`
    - `struct MaintenanceConfig` ⚠ fremd: `contextra-engine`, `contextra-graph`
- `maintenance_scheduler`
  - 1 struct
    - `struct MaintenanceScheduler` · 8 Methoden ⚠ fremd: `contextra-vector`
- `memory_consolidation`
  - 3 const, 7 fn, 8 struct
    - `struct ConsolidationConfig`
    - `struct TurnSegment`
    - `struct ConsolidationPhaseResult`
    - `fn group_turns_into_segments`
    - `fn detect_near_duplicates`
    - `fn run_consolidation_pass`
    - `struct SynthesisConfig`
    - `struct CommunityStabilityTracker` · 3 Methoden
    - `struct MetaChunk`
    - `struct SynthesisPhaseResult`
    - `fn compute_community_hash`
    - `fn run_structural_synthesis_pass` ⚠ fremd: `contextra-types`
    - `const MAX_PRE_CONDENSED_DIGEST_LEN`
    - `const MAX_PRE_CONDENSED_LABEL_LEN`
    - `const MAX_PRE_CONDENSED_PROVENANCE_COUNT`
    - `struct PreCondensedInput` ⚠ fremd: `contextra-types`
    - `fn intake_pre_condensed_inputs` ⚠ fremd: `contextra-types`
    - `fn compact_segment_via_context_compactor`
- `semantic_aggregation_facade`
  - 1 fn
    - `fn consolidate_semantic_hyperedges` ⚠ fremd: `contextra-engine`
- `synthesis_phase`
  - 1 fn, 2 struct, 1 type
    - `struct SegmentSynthesisResult`
    - `type SynthesisPhaseResult`
    - `struct SynthesizedChunk` ⚠ fremd: `contextra-types`
    - `fn run_synthesis_pass` ⚠ fremd: `contextra-ports`
- `transitivity_veto`
  - 2 fn
    - `fn validate_transitivity_veto`
    - `fn filter_candidates_with_transitivity_veto`
- Zusätzlich 3 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_ports::SegmentSynthesizer`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-db` | `(crate)`, `context::*`, `context_compaction::*`, `memory_consolidation::*`, `synthesis_phase::*`, `consolidation_executor::*`, `consolidation_locks::*`, `maintenance_scheduler::*`, `maintenance_config::*`, `start_consolidation_reaper` | – |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

**Auffälligkeiten**

- contextra-store: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?

### contextra-core

- Pfad: `crates/contextra-core` · Zeilen: 515 · Rust-Dateien: 3 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-mvcc` (normal), `contextra-ports` (normal), `contextra-types` (normal), `contextra-wire` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 11 | 0 | 1 | `error`×2, `schema`×2, `tombstone`×2, `error_dto`, `model_fingerprint`, `types`, `error_dto::ContextraErrorDto`, `model_fingerprint::ModelFingerprint` | – |
| `contextra-mvcc` | 6 | 0 | 1 | `seq_log`×2, `snapshot`×2, `tx_buffer`×2 | – |
| `contextra-ports` | 2 | 0 | 1 | `*`×2 | – |
| `contextra-wire` | 1 | 0 | 1 | `*` | – |

**Pro Komponente**

- `(crate-root)` (232 Zeilen)
  - → `contextra-mvcc`: prod: `seq_log`×2, `snapshot`×2, `tx_buffer`×2
  - → `contextra-ports`: prod: `*`×2
  - → `contextra-types`: prod: `error`×2, `schema`×2, `tombstone`×2, `error_dto`, `model_fingerprint`, `types`, `error_dto::ContextraErrorDto`, `model_fingerprint::ModelFingerprint`
  - → `contextra-wire`: prod: `*`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

Keine pub-Items erkannt (Bibliothek ohne lib.rs, nur Binary, oder alles intern).
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::error`, `contextra_types::error_dto`, `contextra_types::model_fingerprint`, `contextra_types::schema`, `contextra_types::tombstone`, `contextra_types::types`, `contextra_ports::*`, `contextra_mvcc::seq_log`, `contextra_mvcc::snapshot`, `contextra_mvcc::tx_buffer`, `contextra_wire::*`, `contextra_ports::*`, `contextra_types::error_dto::ContextraErrorDto`, `contextra_types::model_fingerprint::ModelFingerprint`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-bench` | `ContextraError::Internal`×3, `ContextraError::SnapshotUnsupportedForSignal`×2, `traits::VectorIndex`, `types`, `ContextraError::CapabilityUnsupported`, `Result` | – |

### contextra-crypto

- Pfad: `crates/contextra-crypto` · Zeilen: 15200 · Rust-Dateien: 64 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-crypto` (dev), `contextra-testkit` (dev), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 35 | 18 | 10 | `TenantId`×8, `Result`×7, `ScanMethod::FeatureDisabled`×3, `ContextraError::Crypto`×3, `TenantScoped`×2, `ScanMethod::Structural`×2, `ScanMethod::NotMeasured`×2, `ContextraError`×2, `RequestContext`, `SagaRecord` … (+4) | `ContextraError::Io`×4, `ContextraError::NotFound` |
| `contextra-ports` | 5 | 34 | 2 | `scanner::LayerWitness`, `Clock`, `scanner::ScanTechnique::NotMeasured`, `ScanTechnique`, `scanner` | `Result`×24, `SystemClock`×6, `scanner::ScanTechnique`×4 |
| `contextra-testkit` | 0 | 5 | 0 | – | `mint_witness_for_tests`×4, `FaultVfs` |

**Pro Komponente**

- `crypto` (821 Zeilen)
  - → `contextra-types`: prod: `TenantId`×4, `TenantScoped`×2 · dev: `TenantId`×2
- `deletion_proof` (1307 Zeilen)
  - → `contextra-ports`: prod: `scanner::LayerWitness`, `Clock`, `scanner::ScanTechnique::NotMeasured`, `ScanTechnique`, `scanner`
  - → `contextra-types`: prod: `ScanMethod::FeatureDisabled`×3, `ScanMethod::Structural`×2, `ScanMethod::NotMeasured`×2, `Result`, `RequestContext`, `SagaRecord`, `error::HnswDeletionError::VerificationFailed`, `ContextraError::Internal`, `TenantId`
- `error` (119 Zeilen)
  - → `contextra-types`: prod: `ContextraError`×2 · dev: `ContextraError`
- `kv_cipher` (344 Zeilen)
  - → `contextra-types`: prod: `Result`×6, `ContextraError::Crypto`×3, `TenantId`, `ModelFingerprint`, `ContextraError::ParseError`
- `kv_segment` (1259 Zeilen)
  - → `contextra-types`: prod: `TenantId`×2 · dev: `TenantId`
- `revocation_log` (441 Zeilen)
  - → `contextra-ports`: dev: `Result`×8, `SystemClock`
  - → `contextra-testkit`: dev: `FaultVfs`
  - → `contextra-types`: dev: `ContextraError::Io`×4, `ContextraError::NotFound`
- `benches/crypto_benchmarks.rs` (127 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `fuzz/fuzz_targets/deletion_proof_tamper.rs` (137 Zeilen)
  - → `contextra-ports`: dev: `scanner::ScanTechnique`, `SystemClock`
  - → `contextra-testkit`: dev: `mint_witness_for_tests`
- `tests/audit_chain_tests.rs` (212 Zeilen)
  - → `contextra-ports`: dev: `scanner::ScanTechnique`, `SystemClock`
  - → `contextra-testkit`: dev: `mint_witness_for_tests`
- `tests/campaign_cov_revocation_log.rs` (37 Zeilen)
  - → `contextra-ports`: dev: `SystemClock`
- `tests/campaign_j25.rs` (426 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/campaign_j26_hypotheses.rs` (783 Zeilen)
  - → `contextra-ports`: dev: `Result`×8, `scanner::ScanTechnique`, `SystemClock`
  - → `contextra-testkit`: dev: `mint_witness_for_tests`
- `tests/deletion_proof_signature_version_serde.rs` (196 Zeilen)
  - → `contextra-ports`: dev: `scanner::ScanTechnique`, `SystemClock`
  - → `contextra-testkit`: dev: `mint_witness_for_tests`
- `tests/eviction_worker_tokio_runtime_nonblocking_regression.rs` (65 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/kv_segment_concurrency.rs` (271 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/kv_segment_integration.rs` (119 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/kv_segment_proptests.rs` (82 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/loom_kv_deferred_zeroize.rs` (161 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/proptests.rs` (129 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/revocation_log_vfs.rs` (200 Zeilen)
  - → `contextra-ports`: dev: `Result`×8
  - → `contextra-types`: dev: `ContextraError`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `anti_tamper`
  - 1 struct
    - `struct VolatileEncryptionKey` · 3 Methoden
- `audit_chain`
  - 1 enum, 1 fn, 5 struct
    - `enum DataClass`
    - `struct AuditChainEntry` · 1 Methoden
    - `struct AuditChainHeadSignature`
    - `struct AuditHeadAnchor` · 2 Methoden
    - `struct EncryptedCommitmentSalt` · 2 Methoden
    - `fn compute_record_commitment`
    - `struct AuditChain` · 10 Methoden
- `crypto`
  - 1 struct
    - `struct KeyManager` · 15 Methoden
- `deletion_proof`
  - 1 const, 7 enum, 8 fn, 7 struct, 1 trait
    - `fn verify_proof_sequence` ⚠ fremd: `contextra-types`
    - `fn compute_wal_delete_receipt`
    - `const DELETION_PROOF_KEY_PURPOSE`
    - `fn derive_deletion_proof_key_for_tenant`
    - `trait LayerWitnessExt` · Methoden: wal_seq_after, control_matches
    - `fn map_witness_layer_to_deletion_layer`
    - `struct GraphRepairAttestation`
    - `fn hash_deleted_keys_length_prefixed`
    - `enum CustodyLevel`
    - `enum SigningKeyHandle`
    - `enum VerificationKey`
    - `struct LayerCleanupProof` · 2 Methoden
    - `enum DeletionLayer`
    - `struct UncoveredLayer`
    - `fn serialize` *(re-exportiert)* ⚠ fremd: `contextra-ports`
    - `fn deserialize` *(re-exportiert)* ⚠ fremd: `contextra-ports`
    - `struct LayerWitnessMeasurement` ⚠ fremd: `contextra-ports`
    - `enum ExcludedScope`
    - `enum DeletionScope`
    - `struct DeletionProof`
    - `fn verify_wal_delete_receipt`
    - `enum VerificationErrorReason` ⚠ fremd: `contextra-types`
    - `struct VerificationResult` · 2 Methoden
    - `struct DeletionProofVerifier` · 3 Methoden
- `ed25519_proof`
  - 2 enum, 2 fn, 1 struct
    - `enum DeletionProofError`
    - `struct DeletionProofKeyPair` · 6 Methoden
    - `fn sign_deletion_proof_v3`
    - `fn verify_deletion_proof_v3`
    - `enum SignatureVersion` · 1 Methoden
- `error`
  - 1 enum, 1 type
    - `type Result`
    - `enum CryptoError` · 1 Methoden ⚠ fremd: `contextra-types`
- `kdf`
  - 10 const, 1 fn, 3 struct
    - `const KDF_HEADER_MAGIC`
    - `const KDF_HEADER_VERSION_1`
    - `const KDF_ID_ARGON2ID`
    - `const MIN_SALT_LEN`
    - `const MIN_M_COST_KIB`
    - `const MIN_T_COST`
    - `const MIN_P_COST`
    - `const DEFAULT_M_COST_KIB`
    - `const DEFAULT_T_COST`
    - `const DEFAULT_P_COST`
    - `struct KdfParams` · 1 Methoden
    - `struct KdfHeader` · 4 Methoden
    - `struct DerivedKey`
    - `fn derive_key_argon2id`
- `kv_cipher`
  - 1 const, 2 struct, 1 trait
    - `const CURRENT_KV_FORMAT_VERSION`
    - `struct EncryptedKvLayer` ⚠ fremd: `contextra-types`
    - `trait KvCipher` · Methoden: seal, open
    - `struct KvSegmentCipher` · 6 Methoden
- `kv_segment`
  - 1 const, 1 fn, 4 struct, 1 type
    - `struct EvictionWorker` · 3 Methoden
    - `fn emergency_wipe`
    - `const CURRENT_KV_KEY_DERIVATION_VERSION`
    - `struct EncryptedSegmentPayload`
    - `struct KvSegment` · 8 Methoden ⚠ fremd: `contextra-types`
    - `type SpillHandler` ⚠ fremd: `contextra-types`
    - `struct TenantIsolatedKvStore` · 15 Methoden
- `kv_shredding`
  - 1 const, 1 fn, 5 struct
    - `const DEFAULT_SHRED_KEY_GROUP_SIZE`
    - `struct SubKey`
    - `struct GroupKek`
    - `struct RecordDek`
    - `struct EncryptedRecordPayload`
    - `fn derive_subkey`
    - `struct KeyRegistry` · 16 Methoden
- `revocation_log`
  - 1 enum, 1 fn, 2 struct
    - `enum RevocationTarget` · 1 Methoden
    - `struct RevocationEntry`
    - `fn compute_entry_hash`
    - `struct RevocationLog` · 8 Methoden
- `wal_completeness`
  - 1 fn
    - `fn verify_wal_chain_completeness`
- `wal_crypto`
  - 6 struct, 1 trait
    - `trait KmsProvider` · Methoden: get_key, active_key_id, derive_data_key, wrap_key, unwrap_key, rotate_key
    - `struct LocalKeyManagerKms` · 2 Methoden
    - `struct CloudEgressGuard` · 3 Methoden
  - … 4 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 9 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `DeletionProofKeyPair`, `hash_deleted_keys_length_prefixed`
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::ModelFingerprint`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-kvcache` | `KvCipher`×2, `KvSegmentCipher`×2, `CryptoError`×2, `CryptoKey`, `ModelFingerprint` | – |
| `contextra-vector` | `wal_crypto::WalHmac::new`×6 | – |
| `contextra-store` | `wal_crypto`×2, `verify_wal_chain_completeness`, `wal_crypto::WalHmac`, `crypto::KeyManager` | – |
| `contextra-engine` | `deletion_proof::CustodyLevel::Local`×2, `deletion_proof::LayerWitnessExt`, `deletion_proof` | – |
| `contextra-mcp` | `CryptoKey`×3, `CryptoKey::try_new` | `deletion_proof::DeletionProof`, `AuditChain` |
| `contextra-infer-candle` | `EncryptedKvLayer` | `KvSegment` |
| `contextra` | – | `deletion_proof` |
| `contextra-db` | – | `deletion_proof`×2, `CryptoError`, `deletion_proof::DeletionProof` |
| `contextra-privacy` | – | `crypto::KeyManager`, `kv_cipher::ModelFingerprint` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 73 von 84)**: `VolatileEncryptionKey`, `DataClass`, `AuditChainEntry`, `AuditChainHeadSignature`, `AuditHeadAnchor`, `EncryptedCommitmentSalt`, `compute_record_commitment`, `verify_proof_sequence`, `compute_wal_delete_receipt`, `DELETION_PROOF_KEY_PURPOSE`, `derive_deletion_proof_key_for_tenant`, `map_witness_layer_to_deletion_layer`, `GraphRepairAttestation`, `hash_deleted_keys_length_prefixed`, `CustodyLevel`, `SigningKeyHandle`, `VerificationKey`, `LayerCleanupProof`, `DeletionLayer`, `UncoveredLayer`, `serialize`, `deserialize`, `LayerWitnessMeasurement`, `ExcludedScope`, `DeletionScope`, `verify_wal_delete_receipt`, `VerificationErrorReason`, `VerificationResult`, `DeletionProofVerifier`, `DeletionProofError` …

**Auffälligkeiten**

- contextra-crypto: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-db

- Pfad: `crates/contextra-db` · Zeilen: 13322 · Rust-Dateien: 61 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-testkit` (dev), `contextra-vector` (dev), `contextra-adapt` (normal), `contextra-checkpoint` (normal), `contextra-cognition` (normal), `contextra-crypto` (normal), `contextra-engine` (normal), `contextra-graph` (normal), `contextra-ports` (normal), `contextra-rank` (normal), `contextra-store` (normal), `contextra-sys` (normal), `contextra-text` (normal), `contextra-types` (normal), `contextra-vector` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-cognition` | 10 | 0 | 1 | `(crate)`, `context::*`, `context_compaction::*`, `memory_consolidation::*`, `synthesis_phase::*`, `consolidation_executor::*`, `consolidation_locks::*`, `maintenance_scheduler::*`, `maintenance_config::*`, `start_consolidation_reaper` | – |
| `contextra-engine` | 6 | 0 | 1 | `(crate)`, `collection::maintenance::PercolationResult`, `SandboxBridge`, `background_workers::*`, `fusion`, `collection` | – |
| `contextra-adapt` | 5 | 1 | 3 | `homeostat::RerankDeadline`, `homeostat::RerankPidController`, `decay_controller::*`, `pid_latency_controller::*`, `PidController` | – |
| `contextra-types` | 2 | 47 | 2 | `DistanceMetric`, `MAX_SEARCH_K` | `Result`×8, `TxId`×6, `DocId`×4, `FusionWeights`×3, `TxId::new`×3 … (+16) |
| `contextra-checkpoint` | 1 | 4 | 1 | `(crate)` | `PersistentCheckpointStore`×2, `clear_all_orphaned_checkpoints`, `get_orphaned_checkpoints` |
| `contextra-graph` | 1 | 15 | 1 | `percolation::PercolationConfig` | `CsrGraph`×10, `path_rag::PathRAGEngine`×2, `csr::CsrGraph`×2, `PathRAGEngine` |
| `contextra-ports` | 1 | 15 | 1 | `QueryRewriter` | `GraphIndex`×4, `StorageEngine`×3, `BoxFuture`×2, `scanner::ScanTechnique`, `SystemClock` … (+4) |
| `contextra-sys` | 1 | 0 | 1 | `LockedRegions` | – |
| `contextra-crypto` | 0 | 4 | 0 | – | `deletion_proof`×2, `CryptoError`, `deletion_proof::DeletionProof` |
| `contextra-rank` | 0 | 1 | 0 | – | `IsotonicCalibrator::new` |
| `contextra-store` | 0 | 12 | 0 | – | `LsmConfig`×5, `LsmStorage::new`×4, `LsmStorage`×2, `lsm` |
| `contextra-testkit` | 0 | 1 | 0 | – | `mint_witness_for_tests` |
| `contextra-text` | 0 | 11 | 0 | – | `Language::English`×7, `Language`×2, `bm25::score_term`, `inverted::InvertedIndex` |
| `contextra-vector` | 0 | 6 | 0 | – | `HnswConfig`×3, `HnswIndex`×2, `HnswIndex::try_new` |

**Pro Komponente**

- `(crate-root)` (885 Zeilen)
  - → `contextra-adapt`: prod: `decay_controller::*`, `pid_latency_controller::*`
  - → `contextra-checkpoint`: prod: `(crate)`
  - → `contextra-cognition`: prod: `(crate)`, `context::*`, `context_compaction::*`, `memory_consolidation::*`, `synthesis_phase::*`, `consolidation_executor::*`, `consolidation_locks::*`, `maintenance_scheduler::*`, `maintenance_config::*`, `start_consolidation_reaper`
  - → `contextra-engine`: prod: `(crate)`, `collection::maintenance::PercolationResult`, `SandboxBridge`, `background_workers::*`, `fusion`, `collection`
  - → `contextra-graph`: prod: `percolation::PercolationConfig`
  - → `contextra-rank`: dev: `IsotonicCalibrator::new`
  - → `contextra-types`: prod: `DistanceMetric` · dev: `TenantId`, `Result`, `ContextraError::InvalidInput`
- `homeostat` (23 Zeilen)
  - → `contextra-adapt`: prod: `homeostat::RerankDeadline`, `homeostat::RerankPidController`
- `multistep` (707 Zeilen)
  - → `contextra-adapt`: prod: `PidController`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: prod: `QueryRewriter` · dev: `BoxFuture`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: prod: `MAX_SEARCH_K` · dev: `ContextraError::Internal`
- `volatile_vault` (532 Zeilen)
  - → `contextra-sys`: prod: `LockedRegions`
- `examples/collections.rs` (114 Zeilen)
  - → `contextra-types`: dev: `Result`, `TenantId::new`
- `examples/hybrid_search.rs` (97 Zeilen)
  - → `contextra-types`: dev: `Result`
- `examples/quickstart.rs` (65 Zeilen)
  - → `contextra-types`: dev: `Result`
- `examples/verify_deletion_proof_external.rs` (227 Zeilen)
  - → `contextra-crypto`: dev: `CryptoError`, `deletion_proof`
  - → `contextra-ports`: dev: `scanner::ScanTechnique`, `SystemClock`
  - → `contextra-testkit`: dev: `mint_witness_for_tests`
  - → `contextra-types`: dev: `Result`×2
- `tests/atomic_commit.rs` (100 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/auto_community_detection_test.rs` (127 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/cascade_invalidation_integration.rs` (139 Zeilen)
  - → `contextra-graph`: dev: `path_rag::PathRAGEngine`
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/cascade_invalidation_test.rs` (116 Zeilen)
  - → `contextra-graph`: dev: `path_rag::PathRAGEngine`
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `LinkRelation`
- `tests/checkpoint_layer_bounds.rs` (137 Zeilen)
  - → `contextra-checkpoint`: dev: `PersistentCheckpointStore`
  - → `contextra-store`: dev: `LsmConfig`×2, `LsmStorage::new`×2
  - → `contextra-types`: dev: `TxId`
- `tests/checkpoint_stress.rs` (78 Zeilen)
  - → `contextra-checkpoint`: dev: `PersistentCheckpointStore`
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/compound_split_recall_impact.rs` (402 Zeilen)
  - → `contextra-types`: dev: `FusionWeights`
- `tests/compound_split_recall_impact_test.rs` (456 Zeilen)
  - → `contextra-types`: dev: `FusionWeights`
- `tests/consolidation_double_trigger_test.rs` (143 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/consolidation_integration_test.rs` (236 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`, `LlmTextGenerator`
  - → `contextra-types`: dev: `DocId`, `Result`
- `tests/context_compaction_retry_test.rs` (207 Zeilen)
  - → `contextra-store`: dev: `LsmConfig`, `LsmStorage::new`
  - → `contextra-types`: dev: `TxId::new`×2
- `tests/cross_domain_chaos_matrix_test.rs` (281 Zeilen)
  - → `contextra-graph`: dev: `csr::CsrGraph`
  - → `contextra-ports`: dev: `EmbeddingError`
  - → `contextra-store`: dev: `lsm`
  - → `contextra-text`: dev: `Language`
  - → `contextra-types`: dev: `DocId`
- `tests/deletion_proof_integration.rs` (258 Zeilen)
  - → `contextra-crypto`: dev: `deletion_proof`
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/diskann_collection_fallback.rs` (97 Zeilen)
  - → `contextra-graph`: dev: `csr::CsrGraph`
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-text`: dev: `Language`
  - → `contextra-types`: dev: `TxId::new`
- `tests/docid_128_consolidation.rs` (42 Zeilen)
  - → `contextra-types`: dev: `DocId`, `TxId`
- `tests/e2e_compliance_scenario.rs` (463 Zeilen)
  - → `contextra-crypto`: dev: `deletion_proof::DeletionProof`
- `tests/e2e_tenant_deletion_proof.rs` (214 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/embedding_dimension_switch.rs` (91 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/fault_injection_2pc.rs` (783 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
- `tests/graph_entity_delete_test.rs` (161 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/graph_hybrid_search.rs` (299 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `ContextraError::SnapshotUnsupportedForSignal`×2, `GraphTraversalStrategy`, `EntityId::from_key`, `PprAlgorithm::Auto` … (+1)
- `tests/hnsw_delete_and_backfill.rs` (223 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
- `tests/instance_orphan_registry_test.rs` (133 Zeilen)
  - → `contextra-checkpoint`: dev: `clear_all_orphaned_checkpoints`, `get_orphaned_checkpoints`
  - → `contextra-types`: dev: `TxId`
- `tests/integration_temporal_filter.rs` (206 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-types`: dev: `TxId`
- `tests/multistep_query_rewriter_modalities.rs` (204 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: dev: `ContextraError::Internal`
- `tests/pid_pool_production_caller.rs` (122 Zeilen)
  - → `contextra-adapt`: dev: `PidController`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: dev: `Clock`
  - → `contextra-text`: dev: `Language::English`
- `tests/proptest_search_invariants.rs` (211 Zeilen)
  - → `contextra-text`: dev: `bm25::score_term`
- `tests/search_result_bound.rs` (221 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/snapshot_api.rs` (165 Zeilen)
  - → `contextra-types`: dev: `ContextraError::CapabilityUnsupported`×2, `EntityId::new`
  - → `contextra-vector`: dev: `HnswIndex::try_new`, `HnswConfig`
- `tests/split_brain_vector_orphan_test.rs` (515 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/sprint2_acid_tests.rs` (349 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-store`: dev: `LsmStorage::new`
  - → `contextra-types`: dev: `TenantId`, `TxId`
- `tests/stabilization_sprint_tests.rs` (145 Zeilen)
  - → `contextra-types`: dev: `FusionWeights`
- `tests/supersedes_cascading_tombstone_test.rs` (171 Zeilen)
  - → `contextra-graph`: dev: `PathRAGEngine`
- `tests/text_integration.rs` (102 Zeilen)
  - → `contextra-text`: dev: `inverted::InvertedIndex`
- `tests/truncation_filter_recall_test.rs` (284 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-types`: dev: `MemoryType`
- `tests/zettelkasten_links_test.rs` (276 Zeilen)
  - → `contextra-types`: dev: `HybridQuery`, `FusionWeights::new`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `multistep`
  - 2 const, 3 struct
    - `const DEFAULT_MAX_SUB_QUERIES`
    - `const DEFAULT_RERANK_POOL_MULTIPLIER`
    - `struct MultiStepConfig`
    - `struct MultiStepResult`
    - `struct MultiStepEngine` · 4 Methoden
- `volatile_vault`
  - 2 enum, 6 struct
    - `enum SignalModality`
    - `struct VaultChunk` · 3 Methoden
    - `struct VaultConfig`
    - `struct PurgeReceipt`
    - `struct CommitReceipt`
    - `enum VaultError`
    - `struct VolatileContextVault` · 9 Methoden
    - `struct VaultChunkMetadata`
- Zusätzlich 8 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `RerankDeadline`, `RerankPidController`
- ⚠ Re-Exporte **fremder Crates**: `contextra_adapt::homeostat::RerankDeadline`, `contextra_adapt::homeostat::RerankPidController`, `contextra_cognition`, `contextra_engine`, `contextra_engine::collection::maintenance::PercolationResult`, `contextra_engine::SandboxBridge`, `contextra_cognition::context::*`, `contextra_cognition::context_compaction::*`, `contextra_cognition::memory_consolidation::*`, `contextra_cognition::synthesis_phase::*`, `contextra_cognition::consolidation_executor::*`, `contextra_cognition::consolidation_locks::*`, `contextra_cognition::maintenance_scheduler::*`, `contextra_cognition::maintenance_config::*`, `contextra_engine::background_workers::*`, `contextra_cognition::start_consolidation_reaper`, `contextra_adapt::decay_controller::*`, `contextra_engine::fusion`, `contextra_adapt::pid_latency_controller::*`, `contextra_checkpoint`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-py` | `Collection`, `Document`, `SearchResult`, `Contextra`, `DistanceMetric::Cosine`, `DistanceMetric::Euclidean`, `DistanceMetric::DotProduct` | – |
| `contextra-agent` | `Collection`×2, `Contextra`×2, `volatile_vault` | `Contextra::open_with_config`×17, `ContextraConfig::default`×14, `volatile_vault::VaultConfig`×3, `ContextraConfig`×2 … (+1) |
| `contextra` | `QueryRewriter` | `engine::ProofKeyProvider`, `EmbeddingBackend`, `ContextraConfig` |
| `contextra-router` | – | `ContextraConfig`×5, `Contextra::open_with_config`×5, `Collection`, `context::ContextManager::new` … (+2) |

**Nach außen ungenutzte öffentliche Items (Heuristik, 12 von 13)**: `DEFAULT_MAX_SUB_QUERIES`, `DEFAULT_RERANK_POOL_MULTIPLIER`, `MultiStepConfig`, `MultiStepResult`, `MultiStepEngine`, `SignalModality`, `VaultChunk`, `PurgeReceipt`, `CommitReceipt`, `VaultError`, `VolatileContextVault`, `VaultChunkMetadata`

**Auffälligkeiten**

- contextra-crypto: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-rank: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-store: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-text: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-vector: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?

### contextra-durable-fs

- Pfad: `crates/contextra-durable-fs` · Zeilen: 2327 · Rust-Dateien: 14 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-sys` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-sys` | 2 | 2 | 2 | `FlushKind`, `rename_durable` | `fs_class` |

**Pro Komponente**

- `(crate-root)` (439 Zeilen)
  - → `contextra-sys`: prod: `FlushKind`
- `vfs` (191 Zeilen)
  - → `contextra-sys`: prod: `rename_durable`
- `tests/durability_tests.rs` (71 Zeilen)
  - → `contextra-sys`: dev: `fs_class`
- `tests/fault_injector_tests.rs` (267 Zeilen)
  - → `contextra-sys`: dev: `FlushKind`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 enum, 8 fn, 4 struct
    - `struct PoisonedHandle`
    - `struct DurableHandle` · 8 Methoden
    - `enum LockMode`
    - `struct DirLock` · 2 Methoden
    - `fn sync_dir`
    - `fn sync_dir_with_injector`
    - `fn atomic_replace`
    - `fn atomic_replace_with_injector`
    - `fn durable_remove`
    - `fn durable_remove_with_injector`
    - `struct ScrubReceipt`
    - `fn scrub_and_remove`
    - `fn scrub_and_remove_with_injector`
- `fault_injector`
  - 1 struct, 1 trait
    - `trait FaultInjector` · Methoden: on_write, on_fsync, on_rename, on_dir_fsync
    - `struct NoFault`
- `health`
  - 2 enum, 1 struct
    - `enum HealthState` · 4 Methoden
    - `enum HealthEvent`
    - `struct AtomicHealthState` · 3 Methoden
- `journal`
  - 1 struct
    - `struct PendingDeleteJournal` · 6 Methoden
- `scrub`
  - 3 enum, 2 fn
    - `enum ScrubPolicy`
    - `enum ScrubRefusal`
    - `enum ScrubOutcome`
    - `fn scrub_if_safe`
    - `fn scrub_if_safe_with_injector`
- `vfs`
  - 3 struct, 2 trait
    - `struct VfsOpenOptions`
    - `trait Vfs` · Methoden: open, create_dir_all, remove_file, rename, read_dir, exists…
    - `trait VfsFile` · Methoden: read_at, write_at, set_len, len, is_empty, sync_data…
    - `struct RealFs`
    - `struct RealFsFile`
- ⚠ Re-Exporte **fremder Crates**: `contextra_sys::FlushKind`

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 29 von 29)**: `FaultInjector`, `NoFault`, `HealthState`, `HealthEvent`, `AtomicHealthState`, `PendingDeleteJournal`, `PoisonedHandle`, `DurableHandle`, `LockMode`, `DirLock`, `sync_dir`, `sync_dir_with_injector`, `atomic_replace`, `atomic_replace_with_injector`, `durable_remove`, `durable_remove_with_injector`, `ScrubReceipt`, `scrub_and_remove`, `scrub_and_remove_with_injector`, `ScrubPolicy`, `ScrubRefusal`, `ScrubOutcome`, `scrub_if_safe`, `scrub_if_safe_with_injector`, `VfsOpenOptions`, `Vfs`, `VfsFile`, `RealFs`, `RealFsFile`

### contextra-engine

- Pfad: `crates/contextra-engine` · Zeilen: 26428 · Rust-Dateien: 123 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-testkit` (dev), `contextra-vector` (dev), `contextra-adapt` (normal/optional via adaptive-candidate-pool-sizing), `contextra-checkpoint` (normal), `contextra-crypto` (normal), `contextra-graph` (normal), `contextra-infer-onnx` (normal/optional via onnx), `contextra-mvcc` (normal), `contextra-ports` (normal), `contextra-rank` (normal), `contextra-sandbox` (normal/optional via sandbox), `contextra-store` (normal), `contextra-sys` (normal), `contextra-text` (normal), `contextra-types` (normal), `contextra-vector` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 234 | 178 | 33 | `ContextraError::invalid_input`×49, `Result`×17, `EntityId`×15, `ContextraError::Internal`×13, `HybridQuery`×13, `MAX_SEARCH_K`×11, `ContextraError::Serialization`×8, `ImportanceScore::new`×8, `domain::MemoryLink`×7, `EntityId::from_key`×7 … (+39) | `ContextraError::from`×10, `domain::LinkRelation::Elaborates`×2, `TenantId`×2, `ContextraError::HnswConnectivityDegraded`, `FusionWeights::new` … (+8) |
| `contextra-ports` | 33 | 22 | 14 | `MetricsSink`×9, `StorageEngine`×5, `VectorIndexStats`×2, `Reranker`×2, `DriftStatusProvider`×2, `VectorIndex`, `Clock`, `SystemClock::new`, `SystemClock`, `scanner` … (+8) | `scanner::ScanTechnique` |
| `contextra-store` | 25 | 104 | 8 | `LsmStorage`×5, `SystemPressure`×3, `tenant_codec::TenantScopedStorage`×3, `tenant_codec::TenantScopedStorage::new`×3, `PressureLevel::Critical`×2, `scanner::lsm_purgers`×2, `scanner::lsm_scanners`×2, `PressureLevel::Normal`, `PressureLevel::Elevated`, `LsmConfig` … (+2) | `lsm`×10, `compaction`×2, `sstable`×2 |
| `contextra-graph` | 20 | 75 | 7 | `hyperedge::HyperEdgeId`×2, `percolation::PercolationConfig`×2, `CsrGraph::load_from_storage`×2, `hyperedge::HyperEdge`×2, `cascade_invalidate_edges_for_superseded_doc`, `percolation::compute_percolation_health`, `percolation::should_trigger_rebonding`, `percolation::find_rebonding_candidates`, `CsrGraph`, `hyperedge::RoleId::new` … (+6) | `csr::CsrGraph`×15, `csr`, `CsrGraph::new` |
| `contextra-vector` | 10 | 117 | 6 | `hnsw::HnswIndex`×3, `HnswIndex`×2, `hnsw_purger`×2, `hnsw_scanner`×2, `acorn::FilteredIndex` | `HnswConfig`×57, `hnsw::HnswConfig::default`, `hnsw::HnswIndex::try_new`, `acorn`, `HnswIndex::try_new` |
| `contextra-text` | 7 | 68 | 3 | `Language`×2, `text_purger`×2, `text_scanner`×2, `inverted::InvertedIndex` | `Language::English`×63, `Language::German`×4 |
| `contextra-rank` | 6 | 0 | 3 | `IsotonicCalibrator`×2, `fusion`, `fusion::fuse_search_results_with_strategy`, `fusion::fuse_search_results_with_signal_strategies`, `fusion::fuse_signals` | – |
| `contextra-checkpoint` | 4 | 0 | 3 | `InstanceOrphanRegistry`×2, `InstanceOrphanRegistry::new`, `(crate)` | – |
| `contextra-crypto` | 4 | 2 | 2 | `deletion_proof::CustodyLevel::Local`×2, `deletion_proof::LayerWitnessExt`, `deletion_proof` | – |
| `contextra-adapt` | 2 | 1 | 1 | `PidController`×2 | `PidController::default` |
| `contextra-infer-onnx` | 2 | 0 | 1 | `ensure_onnx_model_download`, `OnnxEmbedder::load` | – |
| `contextra-mvcc` | 1 | 5 | 1 | `tx_buffer::TxBuffer` | `SnapshotRegistry`×2, `tx_buffer::IndexOp` |
| `contextra-testkit` | 0 | 2 | 0 | – | `mint_witness_for_tests`, `InMemoryStorageEngine` |

**Pro Komponente**

- `(crate-root)` (454 Zeilen)
  - → `contextra-checkpoint`: prod: `(crate)`, `InstanceOrphanRegistry`
  - → `contextra-crypto`: prod: `deletion_proof`
  - → `contextra-ports`: prod: `MetricsSink`×3, `license::LicenseGate`, `BoxFuture`, `StorageEngine`, `TextEmbeddingEngine`, `VectorDeleteMode`, `DriftStatusProvider`, `VectorIndexStats`, `StorageStats`
  - → `contextra-rank`: prod: `IsotonicCalibrator`
  - → `contextra-store`: prod: `lsm::DurabilityMode`, `LsmStorage`, `tenant_codec::TenantScopedStorage`
  - → `contextra-text`: prod: `Language`
  - → `contextra-types`: prod: `Result`×6, `DistanceMetric`×2, `ContextraError::Serialization`×2, `ContextraError::Internal`×2, `DistanceMetric::Cosine`
- `background_workers` (1150 Zeilen)
  - → `contextra-graph`: prod: `hyperedge::HyperEdgeId` · dev: `CsrGraph`×8, `hyperedge::HyperEdgeId`
  - → `contextra-mvcc`: prod: `tx_buffer::TxBuffer` · dev: `tx_buffer::TxBuffer`×2, `tx_buffer::IndexOp`
  - → `contextra-ports`: prod: `StorageEngine`×2, `VectorIndex`, `Clock`, `SystemClock::new`
  - → `contextra-store`: dev: `LsmStorage`×8, `LsmConfig`×8
  - → `contextra-text`: dev: `Language::English`×8
  - → `contextra-types`: prod: `Result`×4 · dev: `Result`×2, `ContextraError::HnswConnectivityDegraded`
  - → `contextra-vector`: prod: `hnsw::HnswIndex`×3 · dev: `HnswIndex`×8, `HnswConfig`×8, `hnsw::HnswConfig::default`, `hnsw::HnswIndex::try_new`
- `collection` (11151 Zeilen)
  - → `contextra-adapt`: prod: `PidController`×2 · dev: `PidController::default`
  - → `contextra-graph`: prod: `percolation::PercolationConfig`×2, `cascade_invalidate_edges_for_superseded_doc`, `percolation::compute_percolation_health`, `percolation::should_trigger_rebonding`, `percolation::find_rebonding_candidates`, `CsrGraph`, `hyperedge::HyperEdgeId`, `hyperedge::RoleId::new`, `hyperedge::HyperEdgeId::new`, `hyperedge::RoleBinding` … (+4) · dev: `CsrGraph`×40, `csr::CsrGraph`×15, `csr`, `percolation::PercolationConfig`
  - → `contextra-ports`: prod: `VectorIndexStats`, `Reranker`, `StorageEngine` · dev: `StorageStats`×5, `StorageEngine`×3, `TextEmbeddingEngine`, `BoxFuture`
  - → `contextra-store`: prod: `SystemPressure`×3, `PressureLevel::Critical`×2, `PressureLevel::Normal`, `PressureLevel::Elevated`, `LsmStorage` · dev: `LsmStorage`×36, `LsmConfig`×35, `lsm`×10
  - → `contextra-text`: prod: `inverted::InvertedIndex`, `Language` · dev: `Language::English`×50, `Language::German`×4, `Language`
  - → `contextra-types`: prod: `ContextraError::invalid_input`×37, `HybridQuery`×12, `MAX_SEARCH_K`×11, `EntityId`×11, `ContextraError::Internal`×9, `ImportanceScore::new`×8, `domain::MemoryLink`×7, `EntityId::from_key`×7, `Entity::new`×6, `DecayFunction::None`×6 … (+34) · dev: `Result`×25, `ContextraError::InvalidInput`×15, `ContextraError::from`×10, `MAX_SEARCH_K`×4 … (+27)
  - → `contextra-vector`: prod: `HnswIndex`, `acorn::FilteredIndex` · dev: `HnswIndex`×44, `HnswConfig`×44
- `contextra_impl` (1842 Zeilen)
  - → `contextra-checkpoint`: prod: `InstanceOrphanRegistry`, `InstanceOrphanRegistry::new`
  - → `contextra-crypto`: prod: `deletion_proof::CustodyLevel::Local`×2, `deletion_proof::LayerWitnessExt`
  - → `contextra-graph`: prod: `CsrGraph::load_from_storage`×2
  - → `contextra-infer-onnx`: prod: `ensure_onnx_model_download`, `OnnxEmbedder::load`
  - → `contextra-ports`: prod: `MetricsSink`×3, `Reranker`, `SystemClock`, `scanner`, `license`, `NoopMetricsSink`, `DriftStatusProvider`
  - → `contextra-rank`: prod: `IsotonicCalibrator`
  - → `contextra-store`: prod: `LsmStorage`×3, `tenant_codec::TenantScopedStorage::new`×3, `tenant_codec::TenantScopedStorage`×2, `scanner::lsm_purgers`×2, `scanner::lsm_scanners`×2, `LsmConfig`
  - → `contextra-text`: prod: `text_purger`×2, `text_scanner`×2
  - → `contextra-types`: prod: `ContextraError::invalid_input`×8, `EntityId`×4, `ContextraError::Internal`×2, `FusionWeights`×2, `ContextraError::PolicyViolation`×2, `ContextraError::Storage`×2, `ContextraError::Transaction`, `MemoryType`, `GraphTraversalStrategy`, `HybridQuery` … (+1) · dev: `Result`×3
  - → `contextra-vector`: prod: `hnsw_purger`×2, `hnsw_scanner`×2
- `export` (343 Zeilen)
  - → `contextra-types`: prod: `TxId::new` · dev: `DocId::from_key`×4
- `extraction` (556 Zeilen)
  - → `contextra-ports`: prod: `LlmTextGenerator`
- `filter` (302 Zeilen)
  - → `contextra-types`: prod: `ContextraError::invalid_input`×2, `ContextraError` · dev: `ContextraError::InvalidInput`×2
- `fusion` (54 Zeilen)
  - → `contextra-rank`: prod: `fusion`, `fusion::fuse_search_results_with_strategy`, `fusion::fuse_search_results_with_signal_strategies`, `fusion::fuse_signals`
  - → `contextra-types`: prod: `FusionStrategy`, `SignalFusionStrategies`
- `import` (120 Zeilen)
  - → `contextra-types`: prod: `ContextraError::Serialization`×2, `ContextraError::invalid_input`×2, `Result`
- `merge_operator_adapter` (181 Zeilen)
  - → `contextra-store`: prod: `MergeOperator`
- `metrics_names` (54 Zeilen)
  - → `contextra-ports`: prod: `MetricsSink`×3
- `temporal_filter` (317 Zeilen)
  - → `contextra-types`: prod: `TxId`
- `transaction` (1502 Zeilen)
  - → `contextra-graph`: prod: `hyperedge::HyperEdge`×2 · dev: `CsrGraph`
  - → `contextra-ports`: prod: `StorageEngine`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: dev: `DocId::new`×4, `DocId`×2, `EntityId`
  - → `contextra-vector`: prod: `HnswIndex` · dev: `HnswIndex`, `HnswConfig`
- `tests/auto_extraction_runtime_mode.rs` (236 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/auto_extraction_wiring.rs` (292 Zeilen)
  - → `contextra-types`: dev: `EntityId::from_key`×5
- `tests/auto_triple_extraction.rs` (118 Zeilen)
  - → `contextra-types`: dev: `EntityId::from_key`×2
- `tests/chunker_golden_tests.rs` (618 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/chunker_hardening_tests.rs` (145 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/chunker_robustness.rs` (65 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/commit_crash_consistency.rs` (296 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/crud_error_robustness.rs` (110 Zeilen)
  - → `contextra-types`: dev: `ContextraError`, `Result`
- `tests/decay_and_expiry.rs` (158 Zeilen)
  - → `contextra-ports`: dev: `Clock`
  - → `contextra-types`: dev: `Result`×2, `MemoryType`
- `tests/engine_commit_crash_points.rs` (225 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/engine_recovery_crash_windows.rs` (375 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/erasure_report_honesty.rs` (68 Zeilen)
  - → `contextra-crypto`: dev: `deletion_proof::LayerWitnessExt`
  - → `contextra-ports`: dev: `scanner`
- `tests/export_import_roundtrip.rs` (92 Zeilen)
  - → `contextra-types`: dev: `ContextraError`, `Result`
- `tests/filter_metadata_eval.rs` (106 Zeilen)
  - → `contextra-types`: dev: `FilterExpr`, `Result`
- `tests/hybrid_adaptive_overfetch.rs` (134 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
- `tests/hybrid_pathrag_snapshot.rs` (112 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
- `tests/hybrid_query_fusion.rs` (95 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/hybrid_signal_failure_semantics.rs` (110 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
- `tests/invariant_determinism.rs` (81 Zeilen)
  - → `contextra-types`: dev: `Result`×2
- `tests/invariant_durability.rs` (107 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/invariant_isolation.rs` (129 Zeilen)
  - → `contextra-types`: dev: `Result`, `ContextraError::Internal`
- `tests/invariant_tenant_isolation.rs` (137 Zeilen)
  - → `contextra-types`: dev: `Result`×2
- `tests/metrics_instrumentation.rs` (197 Zeilen)
  - → `contextra-ports`: dev: `MetricsSink`
  - → `contextra-types`: dev: `Result`×5
- `tests/multi_index_transaction.rs` (106 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/proof_verify_not_constant.rs` (41 Zeilen)
  - → `contextra-crypto`: dev: `deletion_proof`
  - → `contextra-ports`: dev: `scanner::ScanTechnique`, `SystemClock`
  - → `contextra-testkit`: dev: `mint_witness_for_tests`
- `tests/query_builder_scope_acorn.rs` (249 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-vector`: dev: `acorn`
- `tests/query_builder_time_travel.rs` (82 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/rule_based_extraction_determinism.rs` (176 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/ssi_read_path_coverage.rs` (171 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph::new`
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `InMemoryStorageEngine`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: dev: `domain::LinkRelation::Elaborates`, `DistanceMetric::Cosine`
  - → `contextra-vector`: dev: `HnswIndex::try_new`, `HnswConfig`
- `tests/tenant_collection_handle_sharing.rs` (70 Zeilen)
  - → `contextra-types`: dev: `Result`×2, `TenantId`
- `tests/tenant_index_persistence_reload.rs` (95 Zeilen)
  - → `contextra-types`: dev: `Result`×2, `TenantId`
- `tests/tenant_policy_required.rs` (59 Zeilen)
  - → `contextra-types`: dev: `Result`×2
- `tests/tenant_wrapper_coverage.rs` (217 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `Result`
- `tests/tx_fault_injection.rs` (283 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`, `VectorIndex`
  - → `contextra-types`: dev: `Result`×7
- `tests/tx_recovery_and_erasure.rs` (295 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`, `VectorIndex`
  - → `contextra-types`: dev: `Result`×7
- `tests/wasm_merge_operator.rs` (364 Zeilen)
  - → `contextra-mvcc`: dev: `SnapshotRegistry`×2
  - → `contextra-store`: dev: `compaction`×2, `sstable`×2, `MergeOperator`
  - → `contextra-types`: dev: `ContextraError`
- `tests/write_skew_via_engine_api.rs` (207 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `Result`×2, `ContextraError`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 2 enum, 1 fn, 5 struct, 1 trait, 3 type
    - `type ConsolidationLauncher` ⚠ fremd: `contextra-store`
    - `fn register_consolidation_launcher` ⚠ fremd: `contextra-store`
    - `struct Document`
    - `struct ContextraStats` ⚠ fremd: `contextra-ports`
    - `type DbStats`
    - `struct CommunityDetectionConfig`
    - `enum EmbeddingBackend`
    - `enum TenantPolicy`
    - `struct ContextraConfig` · 3 Methoden ⚠ fremd: `contextra-ports`, `contextra-store`, `contextra-types`
    - `type TenantCollectionMap` ⚠ fremd: `contextra-store`
    - `struct Contextra` · 3 Methoden
    - `trait SandboxBridge` · Methoden: db_search, db_insert, db_get
- `background_workers`
  - 1 fn
    - `fn start_decay_cleanup_worker` *(re-exportiert)* ⚠ fremd: `contextra-ports`
- `chunker`
  - 1 fn, 2 struct
    - `struct ChunkerConfig` *(re-exportiert)*
    - `fn estimate_tokens` *(re-exportiert)*
    - `struct MarkdownChunker` · 3 Methoden
- `collection`
  - 4 const, 1 enum, 5 fn, 7 struct
    - `const MAX_SCAN_RESULTS` *(re-exportiert)*
    - `fn sorted_unique_shards`
    - `struct PercolationResult`
    - `struct CollectionConfig`
    - `struct StoredDocument`
    - `struct StoredDocumentMeta`
    - `fn parse_importance_score`
    - `fn compute_default_importance` ⚠ fremd: `contextra-types`
    - `fn ensure_importance_metadata`
    - `fn extract_effective_importance`
    - `struct Collection` · 32 Methoden ⚠ fremd: `contextra-store`, `contextra-vector`
    - `const METRIC_SEARCH_LATENCY_VECTOR`
    - `const METRIC_SEARCH_LATENCY_TEXT`
    - `const METRIC_SEARCH_LATENCY_GRAPH`
    - `struct ScopeConstraint` *(re-exportiert)* · 2 Methoden ⚠ fremd: `contextra-types`
    - `enum SearchStrategy` *(re-exportiert)* · 3 Methoden ⚠ fremd: `contextra-types`
    - `struct SignalWeights` *(re-exportiert)* · 1 Methoden
- `decay_controller`
  - 3 struct
    - `struct DecaySignalInputs`
    - `struct DecayControllerConfig`
    - `struct AdaptiveDecayController` · 6 Methoden
- `export`
  - 1 const, 4 struct
    - `const SCHEMA_VERSION_V1`
    - `struct ExportDocumentV1`
    - `struct ExportCollectionV1`
    - `struct ExportMemoryV1`
    - `struct ExportRelationV1`
- `extraction`
  - 2 const, 1 enum, 4 fn, 4 struct, 1 trait
    - `trait EntityExtractor` · Methoden: extract_triples
    - `struct OpenIeExtractor`
    - `fn create_extractor`
    - `fn extract_triples_for_mode`
    - `fn extract_triples_for_config`
    - `fn extract_triples` ⚠ fremd: `contextra-ports`
    - `const MAX_EXTRACTION_TEXT_LENGTH`
    - `const MAX_EXTRACTED_TRIPLES`
    - `struct RuleBasedExtractor` · 2 Methoden
    - `struct ExtractedTriple`
    - `struct EntityExtractionConfig`
    - `enum ExtractorMode`
- `filter`
  - 2 enum, 1 fn
    - `fn extract_memory_type`
    - `enum FilterOp`
    - `enum MetadataFilter` · 1 Methoden
- `fusion`
  - 3 fn
    - `fn fuse_search_results_with_strategy` ⚠ fremd: `contextra-types`
    - `fn fuse_search_results_with_signal_strategies` ⚠ fremd: `contextra-types`
    - `fn fuse_signals`
- `import`
  - 1 struct
    - `struct ImportSummary`
- `merge_operator_adapter`
  - 1 struct
    - `struct WasmMergeOperator` · 2 Methoden
- `metrics_names`
  - 9 const, 3 fn
    - `const COMMIT_LATENCY`
    - `const REBUILD_DURATION`
    - `const BACKPRESSURE_LEVEL`
    - `const CHECKPOINT_DURATION`
    - `const SEARCH_LATENCY_VECTOR`
    - `const SEARCH_LATENCY_TEXT`
    - `const SEARCH_LATENCY_GRAPH`
    - `const INSERT_LATENCY`
    - `const REAPER_DOCS_REAPED`
    - `fn record_histogram_safe` ⚠ fremd: `contextra-ports`
    - `fn record_gauge_safe` ⚠ fremd: `contextra-ports`
    - `fn record_counter_safe` ⚠ fremd: `contextra-ports`
- `temporal_filter`
  - 3 fn, 1 struct, 1 type
    - `type FusionResult`
    - `struct ValidityWindow` · 2 Methoden ⚠ fremd: `contextra-types`
    - `fn extract_validity_window`
    - `fn apply_temporal_validity_filter` ⚠ fremd: `contextra-types`
    - `fn apply_temporal_validity_filter_at`
- `transaction`
  - 1 enum, 1 struct
    - `struct DbTransaction` *(re-exportiert)* · 10 Methoden ⚠ fremd: `contextra-vector`
    - `enum CommitIntent` *(re-exportiert)*
- Zusätzlich 49 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::AutoExtractionMode`, `contextra_graph::percolation::PercolationConfig`, `contextra_ports::TextEmbeddingEngine`, `contextra_ports::VectorDeleteMode`, `contextra_store::lsm::DurabilityMode`, `contextra_checkpoint`, `contextra_text::Language`, `contextra_ports::DriftStatusProvider`, `contextra_types::DistanceMetric`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-cognition` | `collection`×4, `collection::Collection`×3, `decay_controller::AdaptiveDecayController`×2, `transaction::CommitIntent`×2, `ProvenanceRecord`, `decay_controller::DecayControllerConfig` | `Language::English`×9, `SearchResult`×2, `collection::Collection::new`×2, `Language::German` |
| `contextra-db` | `(crate)`, `collection::maintenance::PercolationResult`, `SandboxBridge`, `background_workers::*`, `fusion`, `collection` | – |
| `contextra-ingest` | `chunker`×2 | – |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

**Auffälligkeiten**

- contextra-sandbox: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-sys: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-graph

- Pfad: `crates/contextra-graph` · Zeilen: 26905 · Rust-Dateien: 121 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-store` (dev), `contextra-testkit` (dev), `contextra-adapt` (normal), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 40 | 57 | 27 | `EntityId`×16, `ContextraError::InvalidInput`×5, `ContextraError`×3, `Result`×3, `Edge`×2, `Edge::new`×2, `TxId`×2, `ContextraError::Internal`×2, `MemoryLink`, `LinkRelation::Supersedes` … (+3) | `Entity::new`×10, `TlHfdParams::default`×4, `PprAlgorithm::Auto`×4, `Entity`×2 |
| `contextra-ports` | 25 | 41 | 12 | `GraphIndex`×4, `BoxFuture`×3, `StorageEngine`×2, `graph::GraphMutationError`×2, `graph::HyperEdgeId::new`×2, `graph::RoleBinding`, `graph::HyperEdgeId`, `graph::GraphMutationError::LockAcquisitionTimeout`, `graph::GraphMutationError::RoleBindingInvalid`, `graph::GraphMutationError::EpochReclamationPending` … (+7) | `Result`×3, `graph_index::GraphIndex` |
| `contextra-store` | 0 | 2 | 0 | – | `LsmConfig`, `LsmStorage::new` |

**Pro Komponente**

- `(crate-root)` (124 Zeilen)
  - → `contextra-ports`: prod: `GraphIndex`, `BoxFuture`
  - → `contextra-types`: prod: `TxId`, `Result`
- `apprh` (982 Zeilen)
  - → `contextra-types`: prod: `EntityId`×4, `ContextraError`×2, `ContextraError::InvalidInput`×2 · dev: `Entity`
- `cascade` (943 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: prod: `ContextraError::InvalidInput`×2 · dev: `Entity::new`×10
- `community` (1292 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `csr` (6043 Zeilen)
  - → `contextra-ports`: prod: `BoxFuture`×2, `graph::HyperEdgeId::new`×2, `StorageEngine`, `graph::RoleBinding`, `graph::HyperEdgeId`, `graph::GraphMutationError`, `graph::GraphMutationError::LockAcquisitionTimeout`, `graph::GraphMutationError::RoleBindingInvalid`, `graph::GraphMutationError::EpochReclamationPending`, `graph::GraphMutationError::InsufficientParticipants` … (+5) · dev: `GraphIndex`×3
  - → `contextra-types`: prod: `EntityId`×3, `Edge`×2, `Edge::new`×2, `MemoryLink`, `LinkRelation::Supersedes`, `PprConfig`, `TxId` · dev: `Result`
- `edge_reinforcement` (148 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `edge_reinforcement_buffer` (191 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: prod: `EntityId` · dev: `EntityId`
- `entity_extraction` (285 Zeilen)
  - → `contextra-types`: prod: `ContextraError::Internal`
- `error` (42 Zeilen)
  - → `contextra-ports`: prod: `graph::GraphMutationError`
  - → `contextra-types`: dev: `ContextraError`
- `hyperedge` (711 Zeilen)
  - → `contextra-ports`: prod: `graph`
- `hyperedge_suggest` (160 Zeilen)
  - → `contextra-ports`: prod: `LlmTextGenerator`
- `path_rag` (1418 Zeilen)
  - → `contextra-types`: prod: `EntityId`×2, `Result`×2, `DocId`
- `percolation` (285 Zeilen)
  - → `contextra-ports`: prod: `GraphIndex` · dev: `GraphIndex`
  - → `contextra-types`: prod: `EntityId`
- `ppr` (3592 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`×3, `GraphIndex`×2
  - → `contextra-store`: dev: `LsmConfig`, `LsmStorage::new`
  - → `contextra-types`: prod: `TlHfdParams`, `EntityId` · dev: `TlHfdParams::default`×4, `TlHfdParams`
- `ppr_stream` (145 Zeilen)
  - → `contextra-ports`: prod: `GraphIndex`
  - → `contextra-types`: prod: `ContextraError::Internal`
- `provenance` (165 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `session_dag` (903 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine`
- `tl_hfd` (748 Zeilen)
  - → `contextra-types`: prod: `EntityId`×4, `ContextraError`, `ContextraError::InvalidInput` · dev: `Entity`
- `benches/csr_traversal_bench.rs` (199 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `Result`×4, `Edge::new`×2
- `tests/apprh_determinism_permutation.rs` (129 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/apprh_gate_monitor.rs` (291 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/apprh_selector_shadow_to_production.rs` (220 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/apprh_tests.rs` (215 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/budgeted_k_path_hub_entity.rs` (151 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/community_hyperedges_included_flag.rs` (127 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/csr_benchmark.rs` (163 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/csr_complexity_bench.rs` (152 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/dangling_nodes_audit_test.rs` (245 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `PprAlgorithm::Auto`×3
- `tests/diffusion_shadow_gate_thresholds.rs` (87 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/entity_extraction_edge_cases.rs` (127 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/entity_extraction_tests.rs` (81 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/golden_ports_type_compatibility.rs` (60 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/graph_collection_mutation_trait_test.rs` (68 Zeilen)
  - → `contextra-ports`: dev: `graph`
- `tests/hub_node_benchmark.rs` (96 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `Edge::new`×2
- `tests/hyperedge_cascade_crash_recovery.rs` (167 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/hyperedge_cascade_queue_persistence.rs` (78 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/hyperedge_memory_budget.rs` (96 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/hyperedge_payload_sharing.rs` (37 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/hyperedge_persistence_survives_restart.rs` (116 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/hyperedge_suggest.rs` (161 Zeilen)
  - → `contextra-ports`: dev: `Result`
- `tests/hyperedge_suggest_llm_test.rs` (117 Zeilen)
  - → `contextra-ports`: dev: `Result`
- `tests/hyperedge_suggest_tests.rs` (329 Zeilen)
  - → `contextra-ports`: dev: `Result`
- `tests/integration_graph.rs` (116 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/leiden_reference_validation.rs` (155 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/path_rag_at_snapshot.rs` (311 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/ppr_alloc_test.rs` (112 Zeilen)
  - → `contextra-types`: dev: `PprAlgorithm::Auto`
- `tests/ppr_candidate_stream.rs` (315 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`×9
  - → `contextra-types`: dev: `Result`×8
- `tests/ppr_snapshot_isolation.rs` (356 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/ppr_strategy_equivalence.rs` (97 Zeilen)
  - → `contextra-ports`: dev: `graph_index::GraphIndex`
- `tests/proptest_csr_invariants.rs` (334 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/proptest_tl_hfd_diffusion.rs` (276 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/proptest_tl_hfd_lovasz.rs` (267 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/source_doc_ids_populated.rs` (154 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/star_expansion_equals_clique.rs` (48 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/tlhfd_shadow_mode_parity.rs` (288 Zeilen)
  - → `contextra-types`: dev: `EntityId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 trait
    - `trait GraphIndexExt` · Methoden: remove_entity ⚠ fremd: `contextra-ports`
- `apprh`
  - 2 enum, 4 fn, 9 struct, 1 trait
    - `fn run_apprh_push` ⚠ fremd: `contextra-types`
    - `enum ApprhError`
    - `struct ApprhGateMonitorConfig` · 1 Methoden
    - `struct ApprhObservationRecord`
    - `struct ApprhGateMonitorSnapshot`
    - `struct ApprhGateMonitor` · 7 Methoden
    - `fn apprh_local` ⚠ fremd: `contextra-types`
    - `fn forward_push_apprh` ⚠ fremd: `contextra-types`
    - `struct ApprhParams` · 1 Methoden
    - `enum ApprhMode`
    - `struct ApprhSelectorConfig`
    - `struct ApprhSelector` · 8 Methoden
    - `trait ApprhFlipGate` · Methoden: should_flip
    - `struct DefaultApprhFlipGate`
    - `struct ApprhShadowComparison` ⚠ fremd: `contextra-types`
    - `fn shadow_compare_forward_push_vs_apprh` ⚠ fremd: `contextra-types`
- `arc_slice`
  - 1 struct
    - `struct ArcSlice` · 9 Methoden
- `cascade`
  - 3 const, 1 enum, 4 fn, 4 struct, 1 trait, 2 type
    - `const MAX_HYPEREDGE_CASCADE_FANOUT`
    - `const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT`
    - `struct CascadeInvalidationReport`
    - `fn cascade_invalidate_edges_for_superseded_doc`
    - `const CASCADE_QUEUE_PREFIX`
    - `enum CascadeStatus`
    - `struct CascadeDeletionRecord`
    - `type DeletionProof`
    - `struct CascadeTicket`
    - `trait GraphGarbageCollection` · Methoden: sweep_orphans
    - `struct HyperedgeCascadeReport`
    - `type CascadeReport`
    - `fn topological_sort_hyperedge_closure`
    - `fn cascade_invalidate_hyperedges_for_superseded_doc`
    - `fn verify_doc_hyperedge_deletion_completeness`
- `community`
  - 1 fn, 4 struct
    - `struct CommunityDetectionConfig`
    - `struct VirtualHyperedgeNode` · 1 Methoden
    - `struct StarExpansionIterator` · 2 Methoden
    - `struct CommunityAssignment`
    - `fn detect_communities`
- `consistency_enforcement`
  - 4 struct, 1 trait, 1 type
    - `type EdgeId`
    - `struct ConflictPattern`
    - `struct EdgeAssertion` · 1 Methoden
    - `trait ContradictionDetector` · Methoden: conflicts
    - `struct ExactPredicateConflictDetector`
    - `struct ConsistencyEnforcer` · 9 Methoden
- `csr`
  - 2 struct
    - `struct CsrGraph` *(re-exportiert)* · 14 Methoden
    - `struct MemoryEstimate` *(re-exportiert)* · 1 Methoden
- `edge_reinforcement`
  - 4 fn, 1 struct
    - `struct EdgeReinforcementConfig`
    - `fn compute_edge_weight`
    - `fn apply_cooccurrence_reinforcement`
    - `fn apply_traversal_reinforcement`
    - `fn apply_weight_normalization`
- `edge_reinforcement_buffer`
  - 3 struct
    - `struct CooccurrenceSignal` ⚠ fremd: `contextra-types`
    - `struct TraversalSignal` ⚠ fremd: `contextra-types`
    - `struct EdgeReinforcementBuffer` · 6 Methoden
- `entity_extraction`
  - 1 fn, 2 struct, 1 trait
    - `struct ExtractedRelation` · 1 Methoden
    - `trait EntityExtractor` · Methoden: extract
    - `struct RuleBasedEntityExtractor` · 1 Methoden
    - `fn ingest_extracted_relations`
- `hyperedge`
  - 2 const, 2 fn, 4 struct
    - `const HYPEREDGE_PREFIX`
    - `const HYPEREDGE_BY_ENTITY_PREFIX`
    - `fn star_weight`
    - `struct HyperEdge` · 9 Methoden
    - `struct HyperEdgeView` · 2 Methoden
    - `struct RoleInterner` · 8 Methoden
    - `fn sort_dedup_entities`
    - `struct ConsolidationNodesGuard` · 4 Methoden
- `hyperedge_suggest`
  - 1 const, 1 enum, 2 fn, 2 struct
    - `const MAX_RELATE_PARTICIPANTS`
    - `struct HyperEdgeCandidate`
    - `struct ValidatedHyperEdgeCandidate`
    - `enum HyperedgeSuggestError`
    - `fn compute_co_occurrence_candidates`
    - `fn validate_candidates_with_llm` ⚠ fremd: `contextra-ports`
- `path_rag`
  - 1 const, 1 fn, 7 struct, 2 trait
    - `struct KPathConfig`
    - `struct KPathResult` ⚠ fremd: `contextra-types`
    - `trait KPathDiffusion` · Methoden: find_k_paths
    - `struct GraphPath` ⚠ fremd: `contextra-types`
    - `trait PathGraph` · Methoden: neighbors_with_weights, predecessors_with_weights, hyperedges_for_entity, get_hyperedge, hyperedge_participants
    - `struct PprParams`
    - `fn forward_push_ppr` ⚠ fremd: `contextra-types`
    - `const DEFAULT_SUFFICIENCY_THRESHOLD`
- `percolation`
  - 3 fn, 1 struct
- `ppr`
  - 3 fn, 6 struct
- `ppr_stream`
  - 1 const, 1 struct
- `provenance`
  - 2 struct
- `session_dag`
  - 2 const, 6 struct, 1 type
- `tl_hfd`
  - 2 const, 1 enum, 5 fn, 4 struct, 1 trait
  - … 42 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 15 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_ports::graph::GraphMutationError`, `contextra_types::EntityId`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `hyperedge::HyperEdgeId`×2, `percolation::PercolationConfig`×2, `CsrGraph::load_from_storage`×2, `hyperedge::HyperEdge`×2, `cascade_invalidate_edges_for_superseded_doc`, `percolation::compute_percolation_health`, `percolation::should_trigger_rebonding`, `percolation::find_rebonding_candidates`, `CsrGraph`, `hyperedge::RoleId::new` … (+6) | `csr::CsrGraph`×15, `csr`, `CsrGraph::new` |
| `contextra-cognition` | `EdgeReinforcementBuffer`×2, `HyperEdgeId`, `cascade::CascadeInvalidationReport`, `cascade_invalidate_edges_for_superseded_doc`, `csr::EdgeType`, `CsrGraph`, `hyperedge`, `hyperedge::HyperEdgeId`, `percolation::PercolationConfig`, `EdgeReinforcementConfig` … (+1) | – |
| `contextra-db` | `percolation::PercolationConfig` | `CsrGraph`×10, `path_rag::PathRAGEngine`×2, `csr::CsrGraph`×2, `PathRAGEngine` |
| `contextra-agent` | – | `CsrGraph`×2 |
| `contextra-router` | – | `CsrGraph::new` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 110 von 122)**: `run_apprh_push`, `ApprhError`, `ApprhGateMonitorConfig`, `ApprhObservationRecord`, `ApprhGateMonitorSnapshot`, `ApprhGateMonitor`, `apprh_local`, `forward_push_apprh`, `ApprhParams`, `ApprhMode`, `ApprhSelectorConfig`, `ApprhSelector`, `ApprhFlipGate`, `DefaultApprhFlipGate`, `ApprhShadowComparison`, `shadow_compare_forward_push_vs_apprh`, `ArcSlice`, `MAX_HYPEREDGE_CASCADE_FANOUT`, `DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT`, `CASCADE_QUEUE_PREFIX`, `CascadeStatus`, `CascadeDeletionRecord`, `DeletionProof`, `CascadeTicket`, `GraphGarbageCollection`, `HyperedgeCascadeReport`, `CascadeReport`, `topological_sort_hyperedge_closure`, `cascade_invalidate_hyperedges_for_superseded_doc`, `verify_doc_hyperedge_deletion_completeness` …

**Auffälligkeiten**

- contextra-adapt: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-testkit: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-infer-candle

- Pfad: `crates/contextra-infer-candle` · Zeilen: 7268 · Rust-Dateien: 33 · Ring: 2
- Interne Cargo-Abhängigkeiten: `contextra-crypto` (dev), `contextra-crypto` (normal/optional via kv-bridge), `contextra-ports` (normal), `contextra-rank` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 16 | 15 | 5 | `kv::KvLayout`×6, `kv::RopeConfig`×6, `embedding::EmbeddingError`, `kv::KvPrefixStore`, `kv`, `kv::KvBlock` | `kv::PrefixKey`×4, `GroundingValidator`×2, `EmbeddingProvider`×2, `BoxFuture`, `ResponseGroundingValidator` … (+2) |
| `contextra-types` | 7 | 11 | 6 | `ContextraError`×5, `DocId::from`, `ModelFingerprint` | `DocId`×3, `Result`×3, `ContextChunk`×2, `TenantId` |
| `contextra-crypto` | 1 | 1 | 1 | `EncryptedKvLayer` | `KvSegment` |
| `contextra-rank` | 1 | 0 | 1 | `IsotonicCalibrator` | – |

**Pro Komponente**

- `embedding_provider` (84 Zeilen)
  - → `contextra-ports`: prod: `embedding::EmbeddingError`
- `gasp` (583 Zeilen)
  - → `contextra-rank`: prod: `IsotonicCalibrator`
  - → `contextra-types`: prod: `DocId::from` · dev: `DocId`
- `gguf_loader` (353 Zeilen)
  - → `contextra-types`: prod: `ContextraError`
- `inference` (2144 Zeilen)
  - → `contextra-ports`: prod: `kv::KvLayout`×6, `kv::RopeConfig`×6, `kv::KvPrefixStore`, `kv` · dev: `GroundingValidator`
  - → `contextra-types`: prod: `ContextraError` · dev: `TenantId`, `ContextChunk`, `DocId`
- `kv_bridge` (601 Zeilen)
  - → `contextra-crypto`: prod: `EncryptedKvLayer` · dev: `KvSegment`
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `Result`
- `kv_state` (282 Zeilen)
  - → `contextra-ports`: prod: `kv::KvBlock`
  - → `contextra-types`: prod: `ContextraError`
- `model` (545 Zeilen)
  - → `contextra-types`: prod: `ContextraError`
- `model_registry` (122 Zeilen)
  - → `contextra-types`: prod: `ContextraError`, `ModelFingerprint`
- `tests/embedding_normalization_proptest.rs` (114 Zeilen)
  - → `contextra-ports`: dev: `EmbeddingProvider`
- `tests/embedding_provider_conformance.rs` (126 Zeilen)
  - → `contextra-ports`: dev: `embedding::EmbeddingError`, `EmbeddingProvider`
  - → `contextra-types`: dev: `Result`
- `tests/gasp_mutant_test.rs` (88 Zeilen)
  - → `contextra-ports`: dev: `GroundingValidator`
- `tests/gguf_corruption_and_batch_boundary_test.rs` (302 Zeilen)
  - → `contextra-ports`: dev: `embedding::EmbeddingError`
  - → `contextra-types`: dev: `Result`
- `tests/gguf_loader_fuzz.rs` (237 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/kv_bridge_and_stress_test.rs` (151 Zeilen)
  - → `contextra-ports`: dev: `ResponseGroundingValidator`
- `tests/kv_prefix_reuse_test.rs` (345 Zeilen)
  - → `contextra-ports`: dev: `kv::PrefixKey`×4, `ContextSegment`, `LlmTextGenerator`, `kv`
- `tests/kv_stage_b_test.rs` (138 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/proptest_candle.rs` (73 Zeilen)
  - → `contextra-types`: dev: `ContextChunk`, `DocId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `attention_exporter`
  - 1 const, 1 struct
    - `const MAX_TRACKED_REQUESTS`
    - `struct CandleAttentionExporter` · 5 Methoden
- `embedding`
  - 1 const, 3 struct, 1 trait
    - `const DEFAULT_MAX_CONCURRENT_EMBEDDINGS`
    - `trait CandleEmbedInner` · Methoden: embed, dim
    - `struct CandleEmbedClient` · 4 Methoden
    - `struct BertEmbedModel` · 1 Methoden
    - `struct DefaultCandleEmbedModel`
- `embedding_provider`
  - 1 const
    - `const MAX_CANDLE_EMBED_BATCH_SIZE`
- `gasp`
  - 1 const, 2 struct
    - `const DEFAULT_GROUNDING_THRESHOLD`
    - `struct GaspConfig`
    - `struct GaspValidator` · 9 Methoden
- `gguf_loader`
  - 4 const, 2 fn, 1 struct
    - `const MAX_GGUF_TENSORS`
    - `const MAX_GGUF_METADATA_KVS`
    - `const MAX_GGUF_STRING_LEN`
    - `const MAX_GGUF_METADATA_DEPTH`
    - `struct GgufMetadata`
    - `fn parse_gguf_metadata` ⚠ fremd: `contextra-types`
    - `fn parse_gguf_metadata_with_checksum`
- `inference`
  - 1 const, 2 fn, 6 struct, 1 trait
    - `trait CandleModelInner` *(re-exportiert)* · Methoden: generate, generate_stream, kv_layout, generate_stream_with_prefix
    - `const DEFAULT_MAX_CONCURRENT_INFERENCES` *(re-exportiert)*
    - `struct CandleLlmClient` *(re-exportiert)* · 11 Methoden
    - `struct QuantizedLlamaModel` *(re-exportiert)* · 1 Methoden
    - `struct PrefixSeed` *(re-exportiert)*
    - `struct PrefixRun` *(re-exportiert)*
    - `struct KvPrefixContext` *(re-exportiert)*
    - `fn build_prefix_key` *(re-exportiert)*
    - `fn seed_from_hit` *(re-exportiert)*
    - `struct QuantizedLlamaModel` *(re-exportiert)* · 5 Methoden
- `kv_bridge`
  - 2 struct
    - `struct KvCacheKey` · 1 Methoden
    - `struct KvBridgeAdapter` · 8 Methoden
- `kv_state`
  - 2 struct
    - `struct LayerKv` · 1 Methoden
    - `struct KvState` · 8 Methoden
- `model`
  - 1 const, 3 struct
    - `const MAX_SEQ_LEN`
    - `struct QuantizedMatMul`
    - `struct LayerWeights`
    - `struct ModelWeights` · 6 Methoden
- `model_registry`
  - 1 enum, 1 fn
    - `enum CandleQuantization`
    - `fn compute_fingerprint` ⚠ fremd: `contextra-types`
- Zusätzlich 1 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::ModelFingerprint`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-mcp` | `model_registry::CandleQuantization::Q4KM`×2, `CandleEmbedClient::from_dir`, `CandleLlmClient::from_dir` | – |
| `contextra-bench` | `CandleEmbedClient::from_dir`, `model_registry::CandleQuantization::Q4KM` | – |
| `contextra` | `(crate)` | – |
| `contextra-cognition` | – | `gasp` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 38 von 38)**: `MAX_TRACKED_REQUESTS`, `CandleAttentionExporter`, `DEFAULT_MAX_CONCURRENT_EMBEDDINGS`, `CandleEmbedInner`, `CandleEmbedClient`, `BertEmbedModel`, `DefaultCandleEmbedModel`, `MAX_CANDLE_EMBED_BATCH_SIZE`, `DEFAULT_GROUNDING_THRESHOLD`, `GaspConfig`, `GaspValidator`, `MAX_GGUF_TENSORS`, `MAX_GGUF_METADATA_KVS`, `MAX_GGUF_STRING_LEN`, `MAX_GGUF_METADATA_DEPTH`, `GgufMetadata`, `parse_gguf_metadata`, `parse_gguf_metadata_with_checksum`, `CandleModelInner`, `DEFAULT_MAX_CONCURRENT_INFERENCES`, `CandleLlmClient`, `QuantizedLlamaModel`, `PrefixSeed`, `PrefixRun`, `KvPrefixContext`, `build_prefix_key`, `seed_from_hit`, `QuantizedLlamaModel`, `KvCacheKey`, `KvBridgeAdapter` …

### contextra-infer-ollama

- Pfad: `crates/contextra-infer-ollama` · Zeilen: 5493 · Rust-Dateien: 18 · Ring: 2
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal), `contextra-rank` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 8 | 3 | 4 | `Result`×4, `ContextraError`×3, `ConfigFingerprint` | – |
| `contextra-ports` | 7 | 7 | 2 | `BoxFuture`×2, `Clock`, `LlmTextGenerator`, `LlmTextGeneratorStreaming`, `BoxStream`, `SegmentSynthesizer` | `SeededRng`×3, `TextEmbeddingEngine`×2, `query_rewriter::QueryRewriter` |
| `contextra-rank` | 1 | 0 | 1 | `IsotonicCalibrator` | – |

**Pro Komponente**

- `client` (3384 Zeilen)
  - → `contextra-ports`: prod: `BoxFuture`×2, `Clock`, `LlmTextGenerator`, `LlmTextGeneratorStreaming`, `BoxStream`, `SegmentSynthesizer` · dev: `SeededRng`×3, `TextEmbeddingEngine`, `SegmentSynthesizer`
  - → `contextra-types`: prod: `Result`×4, `ContextraError`, `ConfigFingerprint` · dev: `ContextraError`, `ConfigFingerprint`
- `context_prefixer` (386 Zeilen)
  - → `contextra-types`: prod: `ContextraError`
- `embedding` (353 Zeilen)
  - → `contextra-ports`: dev: `TextEmbeddingEngine`
  - → `contextra-types`: prod: `ContextraError`
- `importance` (554 Zeilen)
  - → `contextra-rank`: prod: `IsotonicCalibrator`
- `tests/campaign_stub_ollama.rs` (176 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/query_rewriter_tests.rs` (48 Zeilen)
  - → `contextra-ports`: dev: `query_rewriter::QueryRewriter`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `client`
  - 6 const, 8 fn, 4 struct, 1 trait
    - `struct CircuitBreaker` *(re-exportiert)* · 4 Methoden
    - `const DEFAULT_BASE_URL` *(re-exportiert)*
    - `const DEFAULT_EMBED_MODEL` *(re-exportiert)*
    - `const MAX_RETRIES` *(re-exportiert)*
    - `const MAX_BATCH_SIZE` *(re-exportiert)*
    - `const MAX_TEXT_BYTES` *(re-exportiert)*
    - `const DEFAULT_MAX_RESPONSE_BYTES` *(re-exportiert)*
    - `struct OllamaConfig` *(re-exportiert)*
    - `fn is_transient_network_error` *(re-exportiert)*
    - `fn classify_reqwest_error` *(re-exportiert)* ⚠ fremd: `contextra-types`
    - `fn is_transient_error` *(re-exportiert)* ⚠ fremd: `contextra-types`
    - `trait EgressAuditSink` *(re-exportiert)* · Methoden: audit_egress, is_noop
    - `struct NoopEgressAuditSink` *(re-exportiert)*
    - `struct EndpointPolicy` *(re-exportiert)* · 2 Methoden
    - `fn xml_escape` *(re-exportiert)*
    - `fn build_rag_prompt` *(re-exportiert)*
    - `fn validate_text_length` *(re-exportiert)*
    - `fn validate_batch_size` *(re-exportiert)*
    - `fn validate_model_name` *(re-exportiert)*
- `context_prefixer`
  - 2 fn, 2 struct, 1 type
    - `struct ContextPrefixConfig`
    - `struct ContextPrefixEngine` · 3 Methoden
    - `type ContextPrefixer`
    - `fn truncate_chars`
    - `fn truncate_prefix`
- `embedding`
  - 1 struct
    - `struct OllamaEmbedder` *(re-exportiert)* · 7 Methoden
- `importance`
  - 1 enum, 5 fn, 1 struct
    - `fn score_importance_batch` ⚠ fremd: `contextra-rank`
    - `enum Confidence`
    - `struct ImportanceAssessment` · 4 Methoden
    - `fn score_importance`
    - `fn score_importance_with_calibrator` ⚠ fremd: `contextra-rank`
    - `fn record_importance_outcome` ⚠ fremd: `contextra-rank`
    - `fn parse_importance_score_response`
- `model_info`
  - 1 fn, 1 struct
    - `struct ModelInfo`
    - `fn known_dimension`
- `query_rewriter`
  - 2 struct
    - `struct OllamaQueryRewriterConfig`
    - `struct OllamaQueryRewriter` · 2 Methoden
- Zusätzlich 3 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-mcp` | `OllamaEmbedder::new`, `OllamaConfig`, `OllamaClient::with_config` | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 35 von 36)**: `CircuitBreaker`, `DEFAULT_BASE_URL`, `DEFAULT_EMBED_MODEL`, `MAX_RETRIES`, `MAX_BATCH_SIZE`, `MAX_TEXT_BYTES`, `DEFAULT_MAX_RESPONSE_BYTES`, `is_transient_network_error`, `classify_reqwest_error`, `is_transient_error`, `EgressAuditSink`, `NoopEgressAuditSink`, `EndpointPolicy`, `xml_escape`, `build_rag_prompt`, `validate_text_length`, `validate_batch_size`, `validate_model_name`, `ContextPrefixConfig`, `ContextPrefixEngine`, `ContextPrefixer`, `truncate_chars`, `truncate_prefix`, `OllamaEmbedder`, `score_importance_batch`, `Confidence`, `ImportanceAssessment`, `score_importance`, `score_importance_with_calibrator`, `record_importance_outcome` …

### contextra-infer-onnx

- Pfad: `crates/contextra-infer-onnx` · Zeilen: 2738 · Rust-Dateien: 9 · Ring: 2
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal), `contextra-rank` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 5 | 3 | 3 | `ContextraError`×2, `Result`, `ContextraError::CapabilityUnsupported`, `RerankResult` | – |
| `contextra-rank` | 4 | 1 | 3 | `PlattScaler`×4 | – |
| `contextra-ports` | 0 | 1 | 0 | – | `EmbeddingProvider` |

**Pro Komponente**

- `(crate-root)` (1041 Zeilen)
  - → `contextra-ports`: dev: `EmbeddingProvider`
  - → `contextra-types`: prod: `Result`, `ContextraError::CapabilityUnsupported` · dev: `ContextraError`, `Result`, `ContextraError::CapabilityUnsupported`
- `reranker` (1438 Zeilen)
  - → `contextra-rank`: prod: `PlattScaler`×4 · dev: `PlattScaler`
  - → `contextra-types`: prod: `ContextraError`×2, `RerankResult`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 const, 4 fn, 3 struct, 1 type
    - `fn default_model_cache_dir`
    - `struct ModelDownloadOptions`
    - `fn ensure_onnx_model_download` ⚠ fremd: `contextra-types`
    - `fn ensure_onnx_model_download_with_options` ⚠ fremd: `contextra-types`
    - `fn ensure_onnx_model_download` ⚠ fremd: `contextra-types`
    - `const MAX_EMBED_BATCH_SIZE`
    - `struct TextEmbedderConfig` · 1 Methoden
    - `type OnnxEmbedder`
    - `struct TextEmbedder` · 7 Methoden
- `reranker`
  - 1 struct
    - `struct CrossEncoderReranker` *(re-exportiert)* · 15 Methoden
- Zusätzlich 4 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_rank::PlattScaler`, `contextra_types::RerankResult`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-mcp` | `OnnxEmbedder::from_path`, `default_model_cache_dir`, `OnnxEmbedder::load` | – |
| `contextra-engine` | `ensure_onnx_model_download`, `OnnxEmbedder::load` | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 7 von 10)**: `ModelDownloadOptions`, `ensure_onnx_model_download_with_options`, `MAX_EMBED_BATCH_SIZE`, `TextEmbedderConfig`, `OnnxEmbedder`, `TextEmbedder`, `CrossEncoderReranker`

**Auffälligkeiten**

- contextra-ports: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?

### contextra-ingest

- Pfad: `crates/contextra-ingest` · Zeilen: 25186 · Rust-Dateien: 117 · Ring: 4
- Interne Cargo-Abhängigkeiten: `contextra-engine` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 3 | 3 | 3 | `DocId`×2, `ContextraError` | `DocId::new` |
| `contextra-engine` | 2 | 2 | 2 | `chunker`×2 | – |

**Pro Komponente**

- `chunking` (4911 Zeilen)
  - → `contextra-engine`: prod: `chunker`×2
  - → `contextra-types`: prod: `DocId`
- `error` (116 Zeilen)
  - → `contextra-types`: prod: `ContextraError`
- `fuzz_support` (33 Zeilen)
  - → `contextra-types`: prod: `DocId`
- `fuzz/fuzz_targets/fuzz_pipeline.rs` (22 Zeilen)
  - → `contextra-types`: dev: `DocId::new`
- `tests/chunk_prose_parity.rs` (98 Zeilen)
  - → `contextra-engine`: dev: `chunker`
  - → `contextra-types`: dev: `DocId`
- `tests/contract_tests.rs` (330 Zeilen)
  - → `contextra-engine`: dev: `chunker`
- `tests/sheets_test.rs` (378 Zeilen)
  - → `contextra-types`: dev: `DocId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `canonical`
  - 1 fn, 1 struct
    - `struct CanonicalText`
    - `fn sanitize_text`
- `chunking`
  - 1 enum, 6 fn, 15 struct, 1 trait
    - `fn token_count`
    - `fn window_split`
    - `fn finalize`
    - `struct ChunkingConfig` · 1 Methoden
    - `struct ProseOptions`
    - `enum TabularMode`
    - `struct TabularOptions`
    - `struct PagedOptions`
    - `struct StructuredOptions`
    - `struct EmailOptions`
    - `struct EmailChunker` *(re-exportiert)*
    - `struct FallbackChunker`
    - `fn build_chunk_metadata`
    - `struct ChunkDraft`
    - `trait IrChunker` · Methoden: strategy, draft
    - `fn draft_document`
    - `fn draft_block_range`
    - `struct PagedChunker` *(re-exportiert)*
    - `struct ProseChunker` *(re-exportiert)*
    - `struct StructuredChunker` *(re-exportiert)*
    - `struct ProseChunker`
    - `struct TabularChunker`
    - `struct PagedChunker`
- `detect`
  - 1 const, 1 enum, 1 fn, 2 struct
    - `const MAX_DETECT_BYTES`
    - `enum Confidence`
    - `struct DetectHints`
    - `struct Detection`
    - `fn detect`
- `error`
  - 2 enum
    - `enum LimitKind`
    - `enum IngestError`
- `extractor`
  - 3 struct, 1 trait
    - `trait DocumentExtractor` · Methoden: formats, extract
    - `struct ResourceBudget` · 7 Methoden
    - `struct NestGuard`
    - `struct ExtractContext` · 1 Methoden
- `format`
  - 2 enum
    - `enum FormatId` · 5 Methoden
    - `enum ChunkStrategy`
- `formats`
  - 2 enum, 45 fn, 22 struct
    - `struct DocumentParseResult`
    - `fn parse_document_xml`
    - `struct DocxExtractor` · 1 Methoden
    - `struct ExtractedNote`
    - `fn parse_footnotes_or_endnotes`
    - `fn parse_comments`
    - `fn parse_header_footer`
    - `struct NumMap` · 2 Methoden
    - `struct StyleInfo`
    - `struct StyleMap` · 2 Methoden
    - `fn match_heading_string`
    - `fn extract_eml`
    - `fn strip_html`
    - `fn extract_mbox`
    - `struct EmailExtractor` · 1 Methoden
    - `fn parse_content`
    - `fn parse_meta`
    - `struct OdfExtractor` · 1 Methoden
    - `fn parse_presentation`
    - `struct SafeCfbReader` · 7 Methoden
    - `fn extract_doc`
    - `struct OleExtractor` · 1 Methoden
    - `fn extract_msg`
    - `fn extract_ppt`
    - `struct PdfExtractor` · 1 Methoden
    - `fn clean_page_text`
    - `fn join_end_of_line_hyphens`
    - `fn reconstruct_paragraphs`
    - `fn normalize_line_digits`
    - `struct HeaderFooterAnalysis`
    - `fn analyze_headers_footers`
    - `struct PptxExtractor` · 1 Methoden
    - `struct SlideRef`
    - `fn parse_presentation_slides`
    - `struct SlideExtractionResult`
    - `fn parse_slide_content`
    - `fn extract_csv_tsv`
    - `fn build_tables_from_rows`
    - `fn determine_header_rows`
    - `fn extract_excel`
    - `struct SheetsExtractor` · 1 Methoden
    - `enum JsonValue` · 1 Methoden
- `fuzz_support`
  - 1 fn
- `ir`
  - 4 enum, 7 struct
- `limits`
  - 2 struct
- `pipeline`
  - 1 fn, 3 struct
- `registry`
  - 1 struct
- `safe`
  - 1 enum, 2 fn, 4 struct
  - … 53 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 25 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `EmailChunker`, `PagedChunker`, `ProseChunker`, `StructuredChunker`, `TabularChunker`

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 133 von 133)**: `CanonicalText`, `sanitize_text`, `token_count`, `window_split`, `finalize`, `ChunkingConfig`, `ProseOptions`, `TabularMode`, `TabularOptions`, `PagedOptions`, `StructuredOptions`, `EmailOptions`, `EmailChunker`, `FallbackChunker`, `build_chunk_metadata`, `ChunkDraft`, `IrChunker`, `draft_document`, `draft_block_range`, `PagedChunker`, `ProseChunker`, `StructuredChunker`, `ProseChunker`, `TabularChunker`, `PagedChunker`, `MAX_DETECT_BYTES`, `Confidence`, `DetectHints`, `Detection`, `detect` …

### contextra-kvcache

- Pfad: `crates/contextra-kvcache` · Zeilen: 7306 · Rust-Dateien: 26 · Ring: 1
- Interne Cargo-Abhängigkeiten: `contextra-crypto` (normal), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 12 | 13 | 5 | `kv_bridge_port::KvBridgeStorage`×2, `embedding::EmbeddingProvider`, `license::FeatureRing`, `storage::StorageEngine`, `BoxFuture`, `plugin`, `RequestId`, `kv`, `kv::PrefixKey`, `attention::AttentionExporter` … (+1) | `kv::KvLayout`, `kv::RopeConfig`, `plugin::PluginManifest`, `embedding`, `storage` |
| `contextra-crypto` | 8 | 3 | 2 | `KvCipher`×2, `KvSegmentCipher`×2, `CryptoError`×2, `CryptoKey`, `ModelFingerprint` | – |
| `contextra-types` | 5 | 13 | 4 | `TenantId`×3, `error::Result`, `error` | `model_fingerprint::ModelFingerprint`×4, `model_fingerprint::ModelFingerprint::new`, `ContextraError` |

**Pro Komponente**

- `adapters` (204 Zeilen)
  - → `contextra-ports`: prod: `embedding::EmbeddingProvider`, `kv_bridge_port::KvBridgeStorage`, `license::FeatureRing`, `storage::StorageEngine`, `BoxFuture`, `plugin`
- `attention_score` (281 Zeilen)
  - → `contextra-ports`: prod: `RequestId`
  - → `contextra-types`: prod: `TenantId`
- `eviction_worker` (283 Zeilen)
  - → `contextra-types`: prod: `TenantId` · dev: `TenantId`
- `prefix_store` (376 Zeilen)
  - → `contextra-ports`: prod: `kv` · dev: `kv::PrefixKey`, `kv::KvLayout`, `kv::RopeConfig`
  - → `contextra-types`: dev: `model_fingerprint::ModelFingerprint::new`
- `scratchpad_invalidation` (209 Zeilen)
  - → `contextra-ports`: prod: `kv::PrefixKey` · dev: `kv`
  - → `contextra-types`: prod: `error::Result`, `TenantId` · dev: `model_fingerprint::ModelFingerprint`
- `segment` (588 Zeilen)
  - → `contextra-crypto`: prod: `KvCipher`×2, `CryptoKey`
- `service` (524 Zeilen)
  - → `contextra-ports`: prod: `attention::AttentionExporter`, `kv_bridge_port::KvBridgeStorage`, `clock`
- `store` (1026 Zeilen)
  - → `contextra-crypto`: prod: `KvSegmentCipher`×2, `CryptoError`×2, `ModelFingerprint`
  - → `contextra-types`: prod: `error`
- `tests/clm_pinned_base_preserved.rs` (102 Zeilen)
  - → `contextra-ports`: dev: `kv`
  - → `contextra-types`: dev: `model_fingerprint::ModelFingerprint`, `TenantId`
- `tests/content_hash_hit_rate_improvement.rs` (102 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/eviction_segment_mapping.rs` (131 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/eviction_with_attention_exporter.rs` (90 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/kivi_aead_order_enforced.rs` (115 Zeilen)
  - → `contextra-crypto`: dev: `KvCipher`×2, `CryptoKey`
- `tests/kivi_quantize_dequantize_roundtrip.rs` (161 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/prefix_store_port.rs` (206 Zeilen)
  - → `contextra-ports`: dev: `kv`
  - → `contextra-types`: dev: `model_fingerprint::ModelFingerprint`
- `tests/scratchpad_invalidation_isolation.rs` (91 Zeilen)
  - → `contextra-ports`: dev: `kv`
  - → `contextra-types`: dev: `model_fingerprint::ModelFingerprint`, `TenantId`
- `tests/service_tests.rs` (346 Zeilen)
  - → `contextra-ports`: dev: `kv_bridge_port::KvBridgeStorage`, `license::FeatureRing`, `plugin::PluginManifest`, `BoxFuture` … (+2)
- `tests/store_default_attention_source.rs` (111 Zeilen)
  - → `contextra-types`: dev: `TenantId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `adapters`
  - 4 struct
    - `struct StorageEngineKvBridge` · 1 Methoden
    - `struct EmbeddingProviderSemanticEmbedder` · 2 Methoden
    - `struct KvCachePluginConfig` · 1 Methoden ⚠ fremd: `contextra-ports`
    - `struct KvCachePluginManifest` · 1 Methoden
- `attention_score`
  - 3 fn, 1 struct, 1 trait
    - `trait AttentionScoreSource` · Methoden: importance_score, importance_score_for_tenant, register_segment, unregister_segment
    - `struct NullAttentionScoreSource`
    - `fn rank_for_eviction`
    - `fn rank_for_eviction_weighted`
    - `fn rank_for_eviction_weighted_for_tenant` ⚠ fremd: `contextra-types`
- `eviction_worker`
  - 1 fn, 1 struct
    - `struct EvictionWorker` · 8 Methoden
    - `fn emergency_wipe`
- `prefix_store`
  - 1 const, 1 struct
    - `const DEFAULT_BYTE_BUDGET_PER_TENANT`
    - `struct TenantPrefixKvStore` · 7 Methoden
- `quantize_kivi`
  - 6 fn, 4 struct
    - `struct KiviQuantizeConfig`
    - `struct KvTensorView` · 3 Methoden
    - `struct KiviBlockMeta`
    - `struct KiviQuantizedBlock`
    - `fn kivi_quantize`
    - `fn kivi_dequantize`
    - `fn unpack_kivi_block`
    - `fn pack_kivi_block`
    - `fn compress_bytes`
    - `fn decompress_bytes`
- `radix`
  - 2 enum, 6 struct, 1 trait
    - `enum KvReusePolicy` · 1 Methoden
    - `struct KvBlockGuard` · 4 Methoden
    - `struct PrefixMatch`
    - `struct PrefixRadixTree` · 8 Methoden
    - `struct KvSegmentRef` · 2 Methoden
    - `enum KvLookupResult`
    - `trait SemanticEmbedder` · Methoden: embed
    - `struct SemanticCacheConfig` · 1 Methoden
    - `struct ContentAddressedKvStore` · 10 Methoden
- `scratchpad_invalidation`
  - 2 struct, 1 trait
    - `struct ScratchpadCacheScope` · 1 Methoden ⚠ fremd: `contextra-types`
    - `trait ScratchpadInvalidator` · Methoden: invalidate_scope
    - `struct PrefixStoreScratchpadInvalidator` · 2 Methoden ⚠ fremd: `contextra-ports`
- `segment`
  - 1 const, 1 enum, 4 struct
    - `enum KvSegmentContent`
    - `const CURRENT_KV_KEY_DERIVATION_VERSION`
    - `struct ShreddableSegmentKey` · 6 Methoden
    - `struct Tier2EncryptedSegment` · 2 Methoden
    - `struct EncryptedSegmentPayload`
    - `struct KvSegment` · 12 Methoden
- `service`
  - 4 struct
    - `struct KvCacheStats`
    - `struct KvCacheConfig`
    - `struct KvCacheServiceBuilder` · 11 Methoden
    - `struct KvCacheService` · 8 Methoden
- `store`
  - 1 const, 1 struct, 1 type
    - `type SpillHandler`
    - `const MAX_PINNED_BYTES_PER_TENANT`
    - `struct TenantIsolatedKvStore` · 24 Methoden

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 48 von 48)**: `StorageEngineKvBridge`, `EmbeddingProviderSemanticEmbedder`, `KvCachePluginConfig`, `KvCachePluginManifest`, `AttentionScoreSource`, `NullAttentionScoreSource`, `rank_for_eviction`, `rank_for_eviction_weighted`, `rank_for_eviction_weighted_for_tenant`, `EvictionWorker`, `emergency_wipe`, `DEFAULT_BYTE_BUDGET_PER_TENANT`, `TenantPrefixKvStore`, `KiviQuantizeConfig`, `KvTensorView`, `KiviBlockMeta`, `KiviQuantizedBlock`, `kivi_quantize`, `kivi_dequantize`, `unpack_kivi_block`, `pack_kivi_block`, `compress_bytes`, `decompress_bytes`, `KvReusePolicy`, `KvBlockGuard`, `PrefixMatch`, `PrefixRadixTree`, `KvSegmentRef`, `KvLookupResult`, `SemanticEmbedder` …

### contextra-license

- Pfad: `crates/contextra-license` · Zeilen: 2618 · Rust-Dateien: 15 · Ring: 4
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 3 | 9 | 2 | `license`×2, `clock` | `clock::Clock`×9 |
| `contextra-types` | 1 | 6 | 1 | `TenantId` | – |

**Pro Komponente**

- `(crate-root)` (37 Zeilen)
  - → `contextra-ports`: prod: `license`
- `signed_gate` (729 Zeilen)
  - → `contextra-ports`: prod: `clock`, `license`
  - → `contextra-types`: prod: `TenantId`
- `tests/check_ring_order.rs` (222 Zeilen)
  - → `contextra-ports`: dev: `clock::Clock`
- `tests/clock_expiration_boundary.rs` (122 Zeilen)
  - → `contextra-ports`: dev: `clock::Clock`
- `tests/crypto_hygiene_payload.rs` (134 Zeilen)
  - → `contextra-ports`: dev: `clock::Clock`
  - → `contextra-types`: dev: `TenantId`
- `tests/installation_binding.rs` (246 Zeilen)
  - → `contextra-ports`: dev: `clock::Clock`
  - → `contextra-types`: dev: `TenantId`
- `tests/installation_binding_isolation.rs` (87 Zeilen)
  - → `contextra-ports`: dev: `clock::Clock`
- `tests/key_rotation.rs` (181 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/legacy_payload_compat.rs` (169 Zeilen)
  - → `contextra-ports`: dev: `clock::Clock`
  - → `contextra-types`: dev: `TenantId`
- `tests/negative_matrix_and_from_activation.rs` (266 Zeilen)
  - → `contextra-ports`: dev: `clock::Clock`
  - → `contextra-types`: dev: `TenantId`
- `tests/signed_activation_7_steps.rs` (154 Zeilen)
  - → `contextra-ports`: dev: `clock::Clock`
- `tests/signed_gate.rs` (162 Zeilen)
  - → `contextra-ports`: dev: `clock::Clock`
  - → `contextra-types`: dev: `TenantId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `signed_gate`
  - 1 const, 1 enum, 9 fn, 4 struct
    - `const MAX_PAYLOAD_BYTES`
    - `fn constant_time_eq_32`
    - `struct SignedLicenseOptions`
    - `enum StepVerificationError` · 1 Methoden
    - `struct SignedActivation` · 3 Methoden
    - `fn derive_local_installation_id_hash`
    - `struct LicensePayload` ⚠ fremd: `contextra-types`
    - `fn verify_step_1_fast_ring`
    - `fn verify_step_2_presence`
    - `fn verify_step_3_installation`
    - `fn verify_step_4_signature`
    - `fn verify_step_5_expiration` ⚠ fremd: `contextra-ports`
    - `fn verify_step_6_ring_permission`
    - `fn verify_step_7_success`
    - `struct SignedLicenseGate` · 18 Methoden
- Zusätzlich 2 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra` | `SignedLicenseGate`, `signed_gate::VerifyingKey::from_bytes` | `OpenFastGate` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 14 von 15)**: `MAX_PAYLOAD_BYTES`, `constant_time_eq_32`, `SignedLicenseOptions`, `StepVerificationError`, `SignedActivation`, `derive_local_installation_id_hash`, `LicensePayload`, `verify_step_1_fast_ring`, `verify_step_2_presence`, `verify_step_3_installation`, `verify_step_4_signature`, `verify_step_5_expiration`, `verify_step_6_ring_permission`, `verify_step_7_success`

### contextra-mcp

- Pfad: `crates/contextra-mcp` · Zeilen: 8706 · Rust-Dateien: 36 · Ring: 4
- Interne Cargo-Abhängigkeiten: `contextra` (normal), `contextra-crypto` (normal), `contextra-infer-candle` (normal/optional via candle), `contextra-infer-ollama` (normal/optional via ollama), `contextra-infer-onnx` (normal/optional via onnx), `contextra-license` (normal), `contextra-ports` (normal), `contextra-privacy` (normal), `contextra-rank` (normal), `contextra-router` (normal), `contextra-types` (normal), `contextra-wire` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-router` | 10 | 1 | 3 | `SlmProfile`×4, `DefaultRouterEngine`×2, `ports_local::HybridSearchProvider`, `ports_local::CommunityResolver`, `ports_local::ContextPreparer`, `RouterEngine::new` | `SlmProfile::new` |
| `contextra` | 9 | 10 | 6 | `Contextra`×3, `Collection`×2, `chunker`, `collection_profile::DeploymentTier`, `collection_profile::AutoExtractionMode::Enabled`, `collection_profile::AutoExtractionMode::Disabled` | – |
| `contextra-ports` | 8 | 43 | 2 | `DriftStatusProvider`×3, `BoxFuture`×2, `plugin::PluginRegistry`×2, `EmbeddingProvider` | `EmbeddingError`×19, `Clock`, `license::FeatureRing` |
| `contextra-privacy` | 7 | 1 | 3 | `bulk_exfiltration_detector::*`, `egress_gateway::*`, `EgressError`, `egress_vault`, `InjectionDetector`, `egress_guard::*`, `egress_vault::BoxFuture` | `ContextEditAuditRecord` |
| `contextra-types` | 5 | 5 | 2 | `ContextraError`×3, `ContextraError::InvalidInput`, `ContextraError::NotFound` | `TenantId::SYSTEM`, `AutoExtractionMode`, `TokenBudget::new`, `TenantId`, `MAX_SEARCH_K` |
| `contextra-crypto` | 4 | 4 | 1 | `CryptoKey`×3, `CryptoKey::try_new` | `deletion_proof::DeletionProof`, `AuditChain` |
| `contextra-infer-candle` | 4 | 0 | 1 | `model_registry::CandleQuantization::Q4KM`×2, `CandleEmbedClient::from_dir`, `CandleLlmClient::from_dir` | – |
| `contextra-infer-ollama` | 3 | 0 | 1 | `OllamaEmbedder::new`, `OllamaConfig`, `OllamaClient::with_config` | – |
| `contextra-infer-onnx` | 3 | 0 | 1 | `OnnxEmbedder::from_path`, `default_model_cache_dir`, `OnnxEmbedder::load` | – |
| `contextra-rank` | 3 | 0 | 2 | `fusion::ProvenanceRecord`, `IsotonicCalibrator`, `IsotonicCalibrator::with_defaults` | – |

**Pro Komponente**

- `bin` (79 Zeilen)
  - → `contextra`: prod: `Contextra`
  - → `contextra-router`: prod: `SlmProfile`
- `bulk_exfiltration_detector` (3 Zeilen)
  - → `contextra-privacy`: prod: `bulk_exfiltration_detector::*`
- `config` (812 Zeilen)
  - → `contextra-infer-candle`: prod: `model_registry::CandleQuantization::Q4KM`×2, `CandleEmbedClient::from_dir`, `CandleLlmClient::from_dir`
  - → `contextra-infer-ollama`: prod: `OllamaEmbedder::new`, `OllamaConfig`, `OllamaClient::with_config`
  - → `contextra-infer-onnx`: prod: `OnnxEmbedder::from_path`, `default_model_cache_dir`, `OnnxEmbedder::load`
  - → `contextra-ports`: dev: `BoxFuture`×2, `EmbeddingError`
  - → `contextra-router`: prod: `SlmProfile`×3
  - → `contextra-types`: prod: `ContextraError`
- `egress_gateway` (24 Zeilen)
  - → `contextra-privacy`: prod: `egress_gateway::*`, `EgressError`, `egress_vault`, `InjectionDetector`
- `egress_guard` (48 Zeilen)
  - → `contextra`: prod: `Collection`
  - → `contextra-privacy`: prod: `egress_guard::*`, `egress_vault::BoxFuture`
- `explain` (239 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
  - → `contextra-rank`: prod: `fusion::ProvenanceRecord`
- `prompt_injection` (1086 Zeilen)
  - → `contextra-ports`: dev: `Clock`
- `protocol` (73 Zeilen)
  - → `contextra-types`: prod: `ContextraError`×2, `ContextraError::InvalidInput`, `ContextraError::NotFound`
- `routing` (102 Zeilen)
  - → `contextra`: prod: `Contextra`, `Collection`
  - → `contextra-ports`: prod: `DriftStatusProvider`×3, `BoxFuture`×2
  - → `contextra-rank`: prod: `IsotonicCalibrator`, `IsotonicCalibrator::with_defaults`
  - → `contextra-router`: prod: `DefaultRouterEngine`×2, `ports_local::HybridSearchProvider`, `ports_local::CommunityResolver`, `ports_local::ContextPreparer`, `RouterEngine::new`
- `sandbox` (697 Zeilen)
  - → `contextra-crypto`: prod: `CryptoKey`×3, `CryptoKey::try_new` · dev: `CryptoKey::try_new`×2
- `server` (153 Zeilen)
  - → `contextra`: prod: `Contextra`
  - → `contextra-ports`: prod: `plugin::PluginRegistry`×2, `EmbeddingProvider`
- `server_tools` (449 Zeilen)
  - → `contextra`: prod: `chunker`
- `tests` (973 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-crypto`: dev: `deletion_proof::DeletionProof`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
  - → `contextra-types`: dev: `TenantId::SYSTEM`
- `tools_crud` (210 Zeilen)
  - → `contextra`: prod: `collection_profile::DeploymentTier`, `collection_profile::AutoExtractionMode::Enabled`, `collection_profile::AutoExtractionMode::Disabled`
- `tests/campaign_j42_mcp_audit.rs` (71 Zeilen)
  - → `contextra`: dev: `collection_profile::DeploymentTier`
  - → `contextra-crypto`: dev: `AuditChain`
  - → `contextra-privacy`: dev: `ContextEditAuditRecord`
  - → `contextra-types`: dev: `AutoExtractionMode`
- `tests/confirm_required_semantics.rs` (186 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `BoxFuture`×2, `EmbeddingError`×2, `EmbeddingProvider`
- `tests/dispatch_direct_tests.rs` (326 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
- `tests/license_gate_entrypoint.rs` (108 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `license::FeatureRing`, `BoxFuture`, `EmbeddingProvider`
- `tests/mcp_test.rs` (1129 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
  - → `contextra-router`: dev: `SlmProfile::new`
  - → `contextra-types`: dev: `TokenBudget::new`
- `tests/server_lifecycle_tests.rs` (171 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
  - → `contextra-types`: dev: `TenantId`
- `tests/tool_registry_classification.rs` (158 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `BoxFuture`×2, `EmbeddingError`×2, `EmbeddingProvider`
- `tests/tools_crud_direct_tests.rs` (536 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
  - → `contextra-types`: dev: `MAX_SEARCH_K`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `config`
  - 4 fn, 3 struct
    - `fn available_compiled_providers`
    - `fn verify_embedding_provider` ⚠ fremd: `contextra-types`
    - `struct EmbeddingConfig` · 4 Methoden
    - `fn create_embedding_provider` ⚠ fremd: `contextra-types`
    - `struct LlmConfig` · 3 Methoden
    - `fn create_llm_text_generator` ⚠ fremd: `contextra-types`
    - `struct RouterConfig` · 1 Methoden ⚠ fremd: `contextra-router`
- `egress_guard`
  - 1 struct
    - `struct CollectionSearchEngine` ⚠ fremd: `contextra`
- `explain`
  - 2 struct
    - `struct ExplainRequest`
    - `struct ExplainResponse` ⚠ fremd: `contextra-rank`
- `io`
  - 2 const, 1 fn
    - `const MAX_RPC_BYTES`
    - `const MAX_SEARCH_QUERY_BYTES`
    - `fn read_line_bounded`
- `proof_key`
  - 1 fn
    - `fn deletion_proof_key_from_env`
- `proof_key_env`
  - 1 enum, 2 fn, 1 struct
    - `enum ProofKeySource`
    - `struct ProofKeyResolution`
    - `fn resolve_proof_key`
    - `fn resolve_proof_key_from_env`
- `protocol`
  - 1 enum, 1 fn
    - `enum McpError` · 6 Methoden
    - `fn response_from_error`
- `routing`
  - 1 fn, 1 struct
    - `struct RoutingHandle` ⚠ fremd: `contextra-ports`, `contextra-rank`, `contextra-router`
    - `fn setup_routing` ⚠ fremd: `contextra`
- `sandbox`
  - 4 const, 1 enum, 4 struct
    - `const MAX_VOLATILE_RESULTS`
    - `const MAX_VOLATILE_KEY_BYTES`
    - `const MAX_VOLATILE_OUTPUT_BYTES`
    - `struct ToolDefinition`
    - `const TOOL_REGISTRY`
    - `enum ToolCategory`
    - `struct SandboxPolicy`
    - `struct VolatileToolResult` · 2 Methoden
    - `struct McpSandbox` · 8 Methoden
- `server`
  - 1 struct
    - `struct McpServer` · 9 Methoden ⚠ fremd: `contextra`, `contextra-ports`
- `validation`
  - 2 fn
    - `fn is_write_allowed_by_env`
    - `fn validate_collection_name`
- Zusätzlich 9 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_privacy::bulk_exfiltration_detector::*`, `contextra_privacy::egress_gateway::*`, `contextra_privacy::egress_guard::*`

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 34 von 34)**: `available_compiled_providers`, `verify_embedding_provider`, `EmbeddingConfig`, `create_embedding_provider`, `LlmConfig`, `create_llm_text_generator`, `RouterConfig`, `CollectionSearchEngine`, `ExplainRequest`, `ExplainResponse`, `MAX_RPC_BYTES`, `MAX_SEARCH_QUERY_BYTES`, `read_line_bounded`, `deletion_proof_key_from_env`, `ProofKeySource`, `ProofKeyResolution`, `resolve_proof_key`, `resolve_proof_key_from_env`, `McpError`, `response_from_error`, `RoutingHandle`, `setup_routing`, `MAX_VOLATILE_RESULTS`, `MAX_VOLATILE_KEY_BYTES`, `MAX_VOLATILE_OUTPUT_BYTES`, `ToolDefinition`, `TOOL_REGISTRY`, `ToolCategory`, `SandboxPolicy`, `VolatileToolResult` …

**Auffälligkeiten**

- contextra-license: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-wire: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-mvcc

- Pfad: `crates/contextra-mvcc` · Zeilen: 5446 · Rust-Dateien: 21 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-testkit` (dev), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 3 | 1 | 1 | `MetricsSink`×3 | – |
| `contextra-types` | 1 | 3 | 1 | `*` | `TxId`×2, `DocId` |
| `contextra-testkit` | 0 | 6 | 0 | – | `ManualClock`×6 |

**Pro Komponente**

- `(crate-root)` (44 Zeilen)
  - → `contextra-types`: prod: `*`
- `seq_log` (711 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `snapshot` (525 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `ssi` (1018 Zeilen)
  - → `contextra-ports`: prod: `MetricsSink`×3
- `tx_buffer` (1272 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `benches/mvcc_bench.rs` (73 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/b16_reproduction.rs` (113 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `tests/campaign_cov_ssi_coarsening.rs` (51 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/campaign_cov_tx_buffer_reap.rs` (61 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
  - → `contextra-types`: dev: `TxId`
- `tests/isolation_tests.rs` (529 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `tests/ssi_metrics_observability.rs` (130 Zeilen)
  - → `contextra-ports`: dev: `MetricsSink`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `seq_log`
  - 1 const, 1 enum, 2 struct
    - `const DEFAULT_MAX_PIN_DURATION`
    - `struct SeqLogEntry` · 1 Methoden
    - `enum SeqLogChange` · 2 Methoden
    - `struct SequenceLog` · 24 Methoden
- `snapshot`
  - 2 struct
    - `struct SnapshotRegistry` · 13 Methoden
    - `struct SnapshotGuard` · 1 Methoden
- `ssi`
  - 1 const, 3 struct, 1 trait
    - `const DEFAULT_MAX_TRACKED_COMMIT_KEYS`
    - `struct ReadSet` · 13 Methoden
    - `trait SsiValidator` · Methoden: validate
    - `struct PruningBlockerInfo`
    - `struct SequenceLogSsiValidator` · 22 Methoden
- `tx_buffer`
  - 6 const, 1 enum, 2 struct, 1 trait
    - `const STAGING_ENTRY_OVERHEAD_BYTES`
    - `const DEFAULT_SHARD_COUNT`
    - `const DEFAULT_MAX_OPS_PER_TX`
    - `const DEFAULT_MAX_READ_SET_KEYS`
    - `const DEFAULT_MAX_TX_STAGED_BYTES`
    - `const DEFAULT_MAX_TOTAL_STAGED_BYTES`
    - `struct TxBufferConfig`
    - `enum IndexOp` · 2 Methoden
    - `trait StagedOpSize` · Methoden: staged_bytes
    - `struct TxBuffer` · 35 Methoden
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::*`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-store` | `SsiValidator`×4, `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`×2, `SnapshotRegistry`, `SequenceLogSsiValidator`, `IndexOp`, `SequenceLogSsiValidator::new_with_bounds` | `SnapshotRegistry::new` |
| `contextra-vector` | `SequenceLog::new`×2, `SeqLogChange::Insert`, `SeqLogChange::Delete`, `TxBuffer`, `SequenceLog`, `IndexOp` | – |
| `contextra-core` | `seq_log`×2, `snapshot`×2, `tx_buffer`×2 | – |
| `contextra-checkpoint` | `SnapshotRegistry`×2 | – |
| `contextra-engine` | `tx_buffer::TxBuffer` | `SnapshotRegistry`×2, `tx_buffer::IndexOp` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 14 von 21)**: `DEFAULT_MAX_PIN_DURATION`, `SeqLogEntry`, `SeqLogChange`, `SnapshotGuard`, `DEFAULT_MAX_TRACKED_COMMIT_KEYS`, `ReadSet`, `PruningBlockerInfo`, `DEFAULT_SHARD_COUNT`, `DEFAULT_MAX_OPS_PER_TX`, `DEFAULT_MAX_READ_SET_KEYS`, `DEFAULT_MAX_TX_STAGED_BYTES`, `DEFAULT_MAX_TOTAL_STAGED_BYTES`, `TxBufferConfig`, `StagedOpSize`

### contextra-ports

- Pfad: `crates/contextra-ports` · Zeilen: 4291 · Rust-Dateien: 28 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 14 | 0 | 7 | `ContextraError`×3, `RequestContext`×3, `Result`×2, `types::domain`, `model_fingerprint::ModelFingerprint`, `types::domain::TenantId`, `*`, `ContextraError::Plugin`, `ScanMethod` | – |

**Pro Komponente**

- `(crate-root)` (140 Zeilen)
  - → `contextra-types`: prod: `*`
- `graph` (189 Zeilen)
  - → `contextra-types`: prod: `ContextraError`, `types::domain`
- `kv` (220 Zeilen)
  - → `contextra-types`: prod: `model_fingerprint::ModelFingerprint`, `types::domain::TenantId`, `ContextraError`
- `plugin` (383 Zeilen)
  - → `contextra-types`: prod: `ContextraError`, `ContextraError::Plugin`
- `scanner` (476 Zeilen)
  - → `contextra-types`: prod: `RequestContext`×3, `Result`×2, `ScanMethod`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 2 type
    - `type BoxFuture`
    - `type BoxStream`
- `attention`
  - 1 struct, 1 trait
    - `struct RequestId`
    - `trait AttentionExporter` · Methoden: export_attention_weights
- `checkpoint`
  - 2 trait
    - `trait Checkpoint` · Methoden: take_snapshot, restore
    - `trait CheckpointCoordinator` · Methoden: create_named_checkpoint, restore_named_checkpoint, drop_named_checkpoint, list_named_checkpoints
- `clock`
  - 1 struct, 1 trait
    - `trait Clock` · Methoden: now_unix_nanos, monotonic_nanos
    - `struct SystemClock` · 1 Methoden
- `embedding`
  - 1 enum, 1 struct, 5 trait
    - `struct ContextSegment` · 3 Methoden
    - `trait TextEmbeddingEngine` · Methoden: embed, embed_batch
    - `enum EmbeddingError`
    - `trait EmbeddingProvider` · Methoden: provider_name, embed, embedding_dim, embed_batch
    - `trait TextGenerator` · Methoden: generate_text
    - `trait LlmTextGenerator` · Methoden: generate, generate_with_context
    - `trait LlmTextGeneratorStreaming` · Methoden: generate_stream
- `graph`
  - 1 enum, 3 struct, 1 trait
    - `struct RoleId` · 2 Methoden
    - `struct RoleBinding` · 1 Methoden
    - `struct HyperEdgeId` · 2 Methoden
    - `enum GraphMutationError`
    - `trait GraphCollectionMutation` · Methoden: relate_n_ary
- `graph_index`
  - 1 struct, 2 trait
    - `trait GraphIndex` · Methoden: traverse, neighbors, remove_edge, add_bidirectional, multi_traverse, multi_traverse_at…
    - `struct GraphIndexStats`
    - `trait CommunityResolver` · Methoden: get_community
- `id_gen`
  - 1 struct, 1 trait
    - `trait IdGen` · Methoden: next_id
    - `struct SequentialIdGen` · 1 Methoden
- `kv`
  - 5 struct, 1 trait
    - `struct KvLayout`
    - `struct RopeConfig`
    - `struct PrefixKey` ⚠ fremd: `contextra-types`
    - `struct KvBlock`
    - `struct KvPrefixHit`
    - `trait KvPrefixStore` · Methoden: lookup, insert, evict
- `kv_bridge_port`
  - 1 trait
    - `trait KvBridgeStorage` · Methoden: get, put, commit
- `license`
  - 3 enum, 2 struct, 1 trait
    - `enum VectorDeleteMode`
    - `enum FeatureRing`
    - `struct AuthorizedRing` · 1 Methoden
    - `enum LicenseError`
    - `trait LicenseGate` · Methoden: check_ring, authorize
    - `struct OpenFastGate`
- `lifecycle`
  - 1 struct, 3 trait
    - `struct GroundingAssessment`
    - `trait GroundingValidator` · Methoden: validate_grounding
    - `trait ResponseGroundingValidator` · Methoden: score_grounding
    - `trait ContextPreparer` · Methoden: prepare_context
- `metrics`
  - 5 struct, 1 trait
    - `trait MetricsSink` · Methoden: record_counter, record_gauge, record_histogram
    - `struct NoopMetricsSink`
    - `struct MetricKey` · 1 Methoden
    - `struct HistogramSnapshot`
    - `struct MetricSnapshot`
    - `struct InMemoryMetricsSink` · 2 Methoden
- `observability`
  - 1 trait
    - `trait DriftStatusProvider` · Methoden: overall_drift_status
- `plugin`
  - 1 enum, 2 struct, 1 trait
    - `struct PluginCapability` · 1 Methoden
    - `trait PluginManifest` · Methoden: name, requires, conflicts, feature_ring_required, capability, activate
    - `enum PluginError`
    - `struct PluginRegistry` · 5 Methoden
- `query_rewriter`
  - 1 trait
    - `trait QueryRewriter` · Methoden: rewrite, rewrite_structured
- `reranker`
  - 1 trait
    - `trait Reranker` · Methoden: rerank, record_implicit_feedback, rerank_deadline_ms
- `rng`
  - 1 struct, 1 trait
    - `trait Rng` · Methoden: next_u64, fill_bytes, next_unit_f64
    - `struct SeededRng` · 2 Methoden
- `scanner`
  - 2 enum, 4 struct, 2 trait
    - `enum WitnessLayer` · 1 Methoden
    - `enum ScanTechnique`
    - `struct ScanTarget` · 1 Methoden
    - `struct PurgeStats`
    - `trait Purger` · Methoden: purge
    - `trait LayerScanner` · Methoden: layer, scan_units, scan
    - `struct ScanSession` · 2 Methoden
    - `struct LayerWitness` · 8 Methoden
- `storage`
  - 1 const, 1 struct, 1 trait
    - `struct StorageStats`
    - `const MAX_SCAN_MERGE_ACCUMULATOR`
    - `trait StorageEngine` · Methoden: get, get_at_seq, get_tracked, get_at_seq_tracked, scan_prefix_tracked, supports_ssi_tracking…
- `text_index`
  - 1 struct, 2 trait
    - `trait SegmentSynthesizer` · Methoden: synthesize_segment, model_id
    - `struct TextIndexStats`
    - `trait TextIndex` · Methoden: search, search_at, insert, delete, commit, rollback…
- `vector_index`
  - 1 struct, 2 trait
    - `struct VectorIndexStats`
    - `trait VectorIndex` · Methoden: insert, all_doc_ids, insert_batch, search, search_at, search_filtered…
    - `trait HybridSearchProvider` · Methoden: search_hybrid
- `vfs`
  - 1 trait
    - `trait Vfs` · Methoden: read, write_atomic, create_dir_all, remove_file, exists, len…
- Zusätzlich 5 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `TextEmbeddingEngine`
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::*`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-checkpoint` | `Clock`×19, `StorageEngine`×14, `MetricsSink`×6, `BoxFuture`×3, `SystemClock`×2, `Checkpoint`×2, `CheckpointCoordinator` | – |
| `contextra-store` | `Clock`×7, `MetricsSink`×6, `MAX_SCAN_MERGE_ACCUMULATOR`×6, `StorageStats`×5, `scanner`×5, `StorageEngine`×3, `ResourceTracker`×2, `SystemClock::new`×2, `license::FeatureRing`, `license::FeatureRing::Sovereign` … (+1) | `ResourceBudget`×27, `ResourceTracker::new`×26, `clock::SystemClock`, `storage::StorageEngine` |
| `contextra-engine` | `MetricsSink`×9, `StorageEngine`×5, `VectorIndexStats`×2, `Reranker`×2, `DriftStatusProvider`×2, `VectorIndex`, `Clock`, `SystemClock::new`, `SystemClock`, `scanner` … (+8) | `scanner::ScanTechnique` |
| `contextra-graph` | `GraphIndex`×4, `BoxFuture`×3, `StorageEngine`×2, `graph::GraphMutationError`×2, `graph::HyperEdgeId::new`×2, `graph::RoleBinding`, `graph::HyperEdgeId`, `graph::GraphMutationError::LockAcquisitionTimeout`, `graph::GraphMutationError::RoleBindingInvalid`, `graph::GraphMutationError::EpochReclamationPending` … (+7) | `Result`×3, `graph_index::GraphIndex` |
| `contextra-vector` | `MetricsSink`×6, `Clock`×4, `Rng`×3, `VectorIndex`×2, `SystemClock`×2, `scanner`×2, `NoopMetricsSink`×2, `SeededRng::new`×2, `SystemClock::new` | `SeededRng`×5, `vector_index::VectorIndex`, `clock::SystemClock` |
| `contextra-infer-candle` | `kv::KvLayout`×6, `kv::RopeConfig`×6, `embedding::EmbeddingError`, `kv::KvPrefixStore`, `kv`, `kv::KvBlock` | `kv::PrefixKey`×4, `GroundingValidator`×2, `EmbeddingProvider`×2, `BoxFuture` … (+3) |
| `contextra-agent` | `BoxFuture`×7, `MetricsSink`×3, `StorageEngine`×2, `VectorIndex`×2, `Checkpoint` | `SystemClock` |
| `contextra-kvcache` | `kv_bridge_port::KvBridgeStorage`×2, `embedding::EmbeddingProvider`, `license::FeatureRing`, `storage::StorageEngine`, `BoxFuture`, `plugin`, `RequestId`, `kv`, `kv::PrefixKey`, `attention::AttentionExporter` … (+1) | `kv::KvLayout`, `kv::RopeConfig`, `plugin::PluginManifest`, `embedding` … (+1) |
| `contextra-testkit` | `Clock`×2, `GraphIndex`, `MetricsSink`, `Rng`, `StorageEngine`, `TextIndex`, `VectorIndex`, `scanner::WitnessLayer`, `scanner::ScanTechnique`, `scanner::LayerWitness` … (+1) | `TextEmbeddingEngine` |
| `contextra-mcp` | `DriftStatusProvider`×3, `BoxFuture`×2, `plugin::PluginRegistry`×2, `EmbeddingProvider` | `EmbeddingError`×19, `Clock`, `license::FeatureRing` |
| `contextra-text` | `StorageEngine`×6, `scanner`×2 | `BoxFuture`×50, `StorageStats`×35, `clock::SystemClock` |
| `contextra-infer-ollama` | `BoxFuture`×2, `Clock`, `LlmTextGenerator`, `LlmTextGeneratorStreaming`, `BoxStream`, `SegmentSynthesizer` | `SeededRng`×3, `TextEmbeddingEngine`×2, `query_rewriter::QueryRewriter` |
| `contextra` | `license`×2, `IdGen`, `license::LicenseGate`, `license::VectorDeleteMode` | `BoxFuture`×3, `license::GateToken`, `license::GateToken::for_test`, `SequentialIdGen` … (+5) |
| `contextra-cognition` | `SegmentSynthesizer`×2, `LlmTextGenerator`, `StorageEngine`, `SeededRng::from_entropy` | `SeededRng`×2, `BoxFuture`×2, `ResponseGroundingValidator` |
| `contextra-crypto` | `scanner::LayerWitness`, `Clock`, `scanner::ScanTechnique::NotMeasured`, `ScanTechnique`, `scanner` | `Result`×24, `SystemClock`×6, `scanner::ScanTechnique`×4 |
| `contextra-license` | `license`×2, `clock` | `clock::Clock`×9 |
| `contextra-mvcc` | `MetricsSink`×3 | – |
| `contextra-privacy` | `Clock`×3 | – |
| `contextra-adapt` | `SeededRng::new`, `Rng::next_u64` | `Clock` |
| `contextra-core` | `*`×2 | – |
| `contextra-db` | `QueryRewriter` | `GraphIndex`×4, `StorageEngine`×3, `BoxFuture`×2, `scanner::ScanTechnique` … (+5) |
| `contextra-router` | `clock` | `StorageEngine`×8, `HybridSearchProvider`×2, `CommunityResolver`×2, `ContextPreparer`×2 … (+2) |
| `contextra-infer-onnx` | – | `EmbeddingProvider` |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

### contextra-privacy

- Pfad: `crates/contextra-privacy` · Zeilen: 7209 · Rust-Dateien: 22 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-crypto` (dev), `contextra-testkit` (dev), `contextra-avv-generator` (normal), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 11 | 2 | 4 | `TenantId`×5, `TenantScoped`×4, `TenantScopeViolation`, `TenantScoped::new` | – |
| `contextra-avv-generator` | 3 | 0 | 1 | `AvvContext`×2, `render_avv_markdown` | – |
| `contextra-ports` | 3 | 5 | 3 | `Clock`×3 | – |
| `contextra-crypto` | 0 | 2 | 0 | – | `crypto::KeyManager`, `kv_cipher::ModelFingerprint` |
| `contextra-testkit` | 0 | 2 | 0 | – | `ManualClock`×2 |

**Pro Komponente**

- `audit_trace` (114 Zeilen)
  - → `contextra-ports`: prod: `Clock`
- `avv_generator` (49 Zeilen)
  - → `contextra-avv-generator`: prod: `AvvContext`×2, `render_avv_markdown`
- `bulk_exfiltration_detector` (333 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `context_edit_audit` (335 Zeilen)
  - → `contextra-ports`: prod: `Clock`
  - → `contextra-types`: prod: `TenantId`
- `egress_guard` (485 Zeilen)
  - → `contextra-ports`: dev: `Clock`
  - → `contextra-types`: prod: `TenantScoped`, `TenantId`
- `egress_vault` (844 Zeilen)
  - → `contextra-types`: prod: `TenantScoped`×3, `TenantId`×2, `TenantScopeViolation`, `TenantScoped::new`
- `processing_registry` (372 Zeilen)
  - → `contextra-ports`: prod: `Clock`
  - → `contextra-types`: prod: `TenantId`
- `tests/audit_trace_tests.rs` (140 Zeilen)
  - → `contextra-ports`: dev: `Clock`
- `tests/campaign_j27_privacy_test.rs` (454 Zeilen)
  - → `contextra-ports`: dev: `Clock`
- `tests/context_edit_audit_chain.rs` (123 Zeilen)
  - → `contextra-ports`: dev: `Clock`
  - → `contextra-types`: dev: `TenantId`
- `tests/egress_gateway_and_recognizer_tests.rs` (802 Zeilen)
  - → `contextra-ports`: dev: `Clock`
- `tests/privacy_consolidated_tests.rs` (348 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
  - → `contextra-types`: dev: `TenantId`
- `tests/tenant_scoped_egress_isolation.rs` (116 Zeilen)
  - → `contextra-crypto`: dev: `crypto::KeyManager`, `kv_cipher::ModelFingerprint`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `audit_trace`
  - 2 fn, 1 trait
    - `fn extract_rule_id`
    - `fn compute_audit_trace`
    - `trait EgressClassifierTrace` · Methoden: classify_with_trace
- `avv_generator`
  - 1 fn
    - `fn generate_avv_draft` ⚠ fremd: `contextra-avv-generator`
- `bulk_exfiltration_detector`
  - 1 enum, 2 struct
    - `struct SessionId`
    - `enum BulkExfiltrationOutcome`
    - `struct BulkExfiltrationDetector` · 5 Methoden
- `context_edit_audit`
  - 2 enum, 4 fn, 2 struct
    - `enum ContextEditKind`
    - `enum ContextEditAuditError`
    - `struct ContextEditInput` ⚠ fremd: `contextra-types`
    - `struct ContextEditAuditRecord` ⚠ fremd: `contextra-types`
    - `fn compute_record_hash`
    - `fn build_context_edit_audit_record`
    - `fn verify_audit_chain`
    - `fn render_audit_line`
- `egress_gateway`
  - 1 const, 10 fn, 6 struct, 2 trait, 1 type
    - `type DefaultEgressClassifier`
    - `const MAX_SEARCH_QUERY_BYTES`
    - `trait EgressGuardCheck` · Methoden: check
    - `struct DefaultEgressGuard`
    - `trait InjectionDetector` · Methoden: detect
    - `struct EgressGateway` · 3 Methoden
    - `struct EgressGatewayBuilder` · 9 Methoden
    - `struct CloudResponseRehydrator` · 2 Methoden
    - `fn pii_vault_forces_crypto_shred`
    - `fn resolve_effective_kv_delete_mode`
    - `struct CloudQueryRequest`
    - `struct CloudQueryResponse`
    - `fn handle_cloud_query`
    - `fn handle_cloud_query_scoped`
    - `fn handle_cloud_query_scoped_with_guard`
    - `fn handle_cloud_query_scoped_with_bulk_detector`
    - `fn handle_cloud_query_with_guard`
    - `fn check_bulk_exfiltration`
    - `fn handle_cloud_query_with_bulk_detector`
    - `fn process_cloud_response`
- `egress_guard`
  - 3 const, 2 struct, 1 trait
    - `struct TextSearchResult`
    - `trait TextSearchEngine` · Methoden: search_text
    - `const DEFAULT_EGRESS_GUARD_TIMEOUT`
    - `const DEFAULT_EGRESS_GUARD_THRESHOLD`
    - `const DEFAULT_EGRESS_GUARD_MIN_BYTES`
    - `struct EgressGuard` · 7 Methoden
- `egress_vault`
  - 1 const, 4 enum, 2 fn, 4 struct, 2 trait, 1 type
    - `trait EntityRecognizer` · Methoden: recognize
    - `struct NoOpRecognizer`
    - `struct SurrogateVault` · 6 Methoden
    - `type BoxFuture`
    - `enum PolicyCategory`
    - `enum BlockReason` · 1 Methoden
    - `enum EgressClassification`
    - `struct CompiledPattern` · 1 Methoden
    - `enum EgressVaultError` ⚠ fremd: `contextra-types`
    - `const MAX_CLASSIFY_PAYLOAD_BYTES`
    - `fn classify_layer1`
    - `fn classify_layer1_arc`
    - `struct EgressVault` · 12 Methoden
    - `trait EgressClassifier` · Methoden: classify
- `entity_recognizer`
  - 1 const, 2 struct, 1 type
    - `const DEFAULT_MAX_INPUT_LENGTH`
    - `type PatternRecognizer`
    - `struct Gazetteer` · 3 Methoden
    - `struct RuleBasedEntityRecognizer` · 7 Methoden
- `error`
  - 1 enum
    - `enum EgressError` · 3 Methoden
- `guarded_payload`
  - 3 struct
    - `struct Unsanitized`
    - `struct Sanitized`
    - `struct GuardedPayload` · 4 Methoden
- `processing_registry`
  - 1 enum, 2 fn, 3 struct
    - `enum ProcessorRole` · 1 Methoden
    - `struct ProcessingActivityRecord` ⚠ fremd: `contextra-types`
    - `struct ProcessingRegistry` · 4 Methoden
    - `struct ProcessingRegistryExport` · 2 Methoden
    - `fn generate_registry`
    - `fn render_markdown`
- ⚠ Re-Exporte **fremder Crates**: `contextra_avv_generator::AvvContext`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-mcp` | `bulk_exfiltration_detector::*`, `egress_gateway::*`, `EgressError`, `egress_vault`, `InjectionDetector`, `egress_guard::*`, `egress_vault::BoxFuture` | `ContextEditAuditRecord` |
| `contextra-router` | `guarded_payload`×2 | – |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

### contextra-py

- Pfad: `crates/contextra-py` · Zeilen: 3157 · Rust-Dateien: 16 · Ring: 4
- Interne Cargo-Abhängigkeiten: `contextra-adapt` (normal), `contextra-db` (normal), `contextra-rank` (normal), `contextra-store` (normal), `contextra-types` (normal), `contextra-wire` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-wire` | 8 | 0 | 1 | `ScoredDocument::create`×2, `ScoredDocumentArgs`×2, `SearchResponse::create`×2, `SearchResponseArgs`×2 | – |
| `contextra-db` | 7 | 3 | 4 | `Collection`, `Document`, `SearchResult`, `Contextra`, `DistanceMetric::Cosine`, `DistanceMetric::Euclidean`, `DistanceMetric::DotProduct` | – |
| `contextra-types` | 3 | 0 | 2 | `ContextraError`, `ContextraErrorDto::from`, `TenantId::SYSTEM` | – |
| `contextra-rank` | 2 | 0 | 1 | `FusionWeights::new`×2 | – |

**Pro Komponente**

- `bindings` (2736 Zeilen)
  - → `contextra-db`: prod: `Collection`, `Document`, `SearchResult`, `Contextra`, `DistanceMetric::Cosine`, `DistanceMetric::Euclidean`, `DistanceMetric::DotProduct` · dev: `Document`×2, `SearchResult`
  - → `contextra-rank`: prod: `FusionWeights::new`×2
  - → `contextra-types`: prod: `ContextraError`, `ContextraErrorDto::from`, `TenantId::SYSTEM`
  - → `contextra-wire`: prod: `ScoredDocument::create`×2, `ScoredDocumentArgs`×2, `SearchResponse::create`×2, `SearchResponseArgs`×2

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `bindings`
  - 6 const, 21 fn, 8 struct
    - `struct PyCollection` · 6 Methoden
    - `const MIN_WORKER_THREADS`
    - `const MAX_WORKER_THREADS`
    - `const MAX_ID_LENGTH`
    - `const MAX_COLLECTION_NAME_LENGTH`
    - `const MAX_LABEL_LENGTH`
    - `const MAX_BATCH_SIZE`
    - `fn dict_to_json`
    - `fn opt_dict_to_json`
    - `fn validate_label`
    - `fn validate_id`
    - `fn validate_collection_name`
    - `fn validate_db_path`
    - `fn validate_query_text`
    - `fn validate_batch_size`
    - `fn validate_vector`
    - `fn validate_id_obj`
    - `fn check_subinterpreter_guard`
    - `fn run_blocking_ffi`
    - `fn json_to_py`
    - `fn doc_to_py` ⚠ fremd: `contextra-db`
    - `fn results_to_py` ⚠ fremd: `contextra-db`
    - `fn contextra_err` ⚠ fremd: `contextra-types`
    - `struct PyContextra` · 11 Methoden
    - `struct PyDbStats`
    - `struct PyDocument`
    - `fn open`
    - `fn _trigger_panic_for_test`
    - `fn validate_hyperedge_args`
    - `fn parse_worker_threads_env`
    - `struct PyRuntimeState`
    - `fn get_runtime`
    - `struct PySearchResult`
    - `struct PyStorageStats`
    - `struct PyVectorIndexStats`
- `kv_links`
  - 1 fn
    - `fn parse_link_relation`

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 36 von 36)**: `PyCollection`, `MIN_WORKER_THREADS`, `MAX_WORKER_THREADS`, `MAX_ID_LENGTH`, `MAX_COLLECTION_NAME_LENGTH`, `MAX_LABEL_LENGTH`, `MAX_BATCH_SIZE`, `dict_to_json`, `opt_dict_to_json`, `validate_label`, `validate_id`, `validate_collection_name`, `validate_db_path`, `validate_query_text`, `validate_batch_size`, `validate_vector`, `validate_id_obj`, `check_subinterpreter_guard`, `run_blocking_ffi`, `json_to_py`, `doc_to_py`, `results_to_py`, `contextra_err`, `PyContextra`, `PyDbStats`, `PyDocument`, `open`, `_trigger_panic_for_test`, `validate_hyperedge_args`, `parse_worker_threads_env` …

**Auffälligkeiten**

- contextra-adapt: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-store: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-rank

- Pfad: `crates/contextra-rank` · Zeilen: 6508 · Rust-Dateien: 36 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-adapt` (dev), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 12 | 7 | 8 | `ConfigFingerprint`×3, `SignalFusionStrategies`×3, `ContextraError`, `FusionWeights`, `FusionStrategy`, `MAX_SEARCH_K`, `OnSignalFailure`, `DocId` | – |
| `contextra-adapt` | 0 | 2 | 0 | – | `PidController`×2 |

**Pro Komponente**

- `calibration` (678 Zeilen)
  - → `contextra-types`: prod: `ConfigFingerprint`×3
- `fusion` (2031 Zeilen)
  - → `contextra-types`: prod: `SignalFusionStrategies`×3, `ContextraError`, `FusionWeights`, `FusionStrategy`, `MAX_SEARCH_K`, `OnSignalFailure`, `DocId`
- `tests/calibration_deep_tests.rs` (339 Zeilen)
  - → `contextra-adapt`: dev: `PidController`
  - → `contextra-types`: dev: `ConfigFingerprint`
- `tests/calibration_stress_and_fault_tests.rs` (108 Zeilen)
  - → `contextra-adapt`: dev: `PidController`
- `tests/conformal_tests.rs` (185 Zeilen)
  - → `contextra-types`: dev: `ConfigFingerprint`
- `tests/dibud_equivalence.rs` (217 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/global_fusion_community_aware.rs` (119 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/rank_tests.rs` (305 Zeilen)
  - → `contextra-types`: dev: `ConfigFingerprint`
- `tests/rrf_reference.rs` (96 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/score_normalized_fallback.rs` (166 Zeilen)
  - → `contextra-types`: dev: `FusionStrategy`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `calibration`
  - 1 enum, 3 struct, 1 trait
    - `enum ConformalError`
    - `trait ConformalCalibrator` · Methoden: update, threshold
    - `struct AdaptiveConformalCalibrator` · 7 Methoden
    - `struct IsotonicCalibrator` · 12 Methoden
    - `struct PlattScaler` · 10 Methoden
- `dibud`
  - 2 enum, 2 fn, 3 struct
    - `fn fuse_exact_prefix`
    - `fn fuse_exact_prefix_async`
    - `enum DiBudStep`
    - `struct DiBudFusionState` · 7 Methoden
    - `enum BudgetedChannel` · 2 Methoden
    - `struct FusionBudget` · 1 Methoden
    - `struct DiBudOutcome`
- `drift`
  - 1 enum, 1 struct
    - `enum DriftStatus` · 1 Methoden
    - `struct DriftDetector` · 6 Methoden
- `explain`
  - 1 fn, 2 struct
    - `struct ExplanationEntry`
    - `struct RetrievalExplanation` · 1 Methoden
    - `fn explain`
- `fusion`
  - 1 enum, 2 struct
    - `struct ProvenanceBuilder` *(re-exportiert)* · 10 Methoden
    - `enum SignalKind` *(re-exportiert)* · 3 Methoden
    - `struct BoundedTopK` *(re-exportiert)* · 6 Methoden
- Zusätzlich 24 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `SignalKind`
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::FusionStrategy`, `contextra_types::OnSignalFailure`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `IsotonicCalibrator`×2, `fusion`, `fusion::fuse_search_results_with_strategy`, `fusion::fuse_search_results_with_signal_strategies`, `fusion::fuse_signals` | – |
| `contextra-infer-onnx` | `PlattScaler`×4 | – |
| `contextra-mcp` | `fusion::ProvenanceRecord`, `IsotonicCalibrator`, `IsotonicCalibrator::with_defaults` | – |
| `contextra-py` | `FusionWeights::new`×2 | – |
| `contextra` | `(crate)` | – |
| `contextra-infer-candle` | `IsotonicCalibrator` | – |
| `contextra-infer-ollama` | `IsotonicCalibrator` | – |
| `contextra-db` | – | `IsotonicCalibrator::new` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 18 von 20)**: `ConformalError`, `ConformalCalibrator`, `AdaptiveConformalCalibrator`, `fuse_exact_prefix`, `fuse_exact_prefix_async`, `DiBudStep`, `DiBudFusionState`, `BudgetedChannel`, `FusionBudget`, `DiBudOutcome`, `DriftStatus`, `DriftDetector`, `ExplanationEntry`, `RetrievalExplanation`, `explain`, `ProvenanceBuilder`, `SignalKind`, `BoundedTopK`

**Auffälligkeiten**

- contextra-ports: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-router

- Pfad: `crates/contextra-router` · Zeilen: 8611 · Rust-Dateien: 38 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-db` (dev), `contextra-graph` (dev), `contextra-text` (dev), `contextra-vector` (dev), `contextra-adapt` (normal), `contextra-ports` (normal), `contextra-privacy` (normal), `contextra-types` (normal), `contextra-wire` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-adapt` | 4 | 4 | 2 | `bandit`, `lyapunov`, `offpolicy`, `offpolicy::RandomizedLoggingPolicy` | `bandit::BanditProfileState`×2, `BanditError`, `FcTsError::InvalidConfig` |
| `contextra-privacy` | 2 | 0 | 1 | `guarded_payload`×2 | – |
| `contextra-ports` | 1 | 18 | 1 | `clock` | `StorageEngine`×8, `HybridSearchProvider`×2, `CommunityResolver`×2, `ContextPreparer`×2, `clock::Clock`×2 … (+1) |
| `contextra-types` | 1 | 16 | 1 | `RetrievalStrategy` | `ConfigFingerprint`×7, `TokenBudget`×4, `DocId::new`×3, `Result` |
| `contextra-db` | 0 | 14 | 0 | – | `ContextraConfig`×5, `Contextra::open_with_config`×5, `Collection`, `context::ContextManager::new`, `Collection::new` … (+1) |
| `contextra-graph` | 0 | 1 | 0 | – | `CsrGraph::new` |
| `contextra-text` | 0 | 1 | 0 | – | `Language::English` |
| `contextra-vector` | 0 | 2 | 0 | – | `HnswConfig::default`, `HnswIndex::try_new` |

**Pro Komponente**

- `(crate-root)` (80 Zeilen)
  - → `contextra-adapt`: prod: `bandit`, `lyapunov`, `offpolicy`
  - → `contextra-privacy`: prod: `guarded_payload`×2
- `arm_registry` (171 Zeilen)
  - → `contextra-types`: prod: `RetrievalStrategy`
- `bandit_regret_tests` (265 Zeilen)
  - → `contextra-adapt`: dev: `BanditError`
- `router` (1412 Zeilen)
  - → `contextra-adapt`: prod: `offpolicy::RandomizedLoggingPolicy`
  - → `contextra-db`: dev: `ContextraConfig`×4, `Contextra::open_with_config`×4
  - → `contextra-ports`: prod: `clock`
  - → `contextra-types`: dev: `TokenBudget`
- `tests` (3537 Zeilen)
  - → `contextra-db`: dev: `Collection::new`, `ContextraConfig`, `Contextra::open_with_config`
  - → `contextra-graph`: dev: `CsrGraph::new`
  - → `contextra-ports`: dev: `StorageEngine`×3, `clock::Clock`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: dev: `ConfigFingerprint`×7, `DocId::new`×3, `TokenBudget`×2, `Result`
  - → `contextra-vector`: dev: `HnswConfig::default`, `HnswIndex::try_new`
- `benches/cost_routing_savings_bench.rs` (151 Zeilen)
  - → `contextra-types`: dev: `TokenBudget`
- `benches/router_bench.rs` (183 Zeilen)
  - → `contextra-db`: dev: `Collection`, `context::ContextManager::new`
  - → `contextra-ports`: dev: `HybridSearchProvider`, `CommunityResolver`, `ContextPreparer`
- `tests/arm_registry_five_arms.rs` (61 Zeilen)
  - → `contextra-types`: dev: `RetrievalStrategy`
- `tests/fc_ts_dispatch_test.rs` (98 Zeilen)
  - → `contextra-adapt`: dev: `FcTsError::InvalidConfig`
- `tests/regret_monotonicity.rs` (257 Zeilen)
  - → `contextra-adapt`: dev: `bandit::BanditProfileState`
- `tests/router_audit_and_isolation_tests.rs` (699 Zeilen)
  - → `contextra-db`: dev: `ContextManager::new`
  - → `contextra-ports`: dev: `StorageEngine`×5, `BoxFuture`×2, `clock::Clock`, `HybridSearchProvider` … (+2)
- `tests/router_bandit_dispatch.rs` (255 Zeilen)
  - → `contextra-adapt`: dev: `bandit::BanditProfileState`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `arm_registry`
  - 1 enum, 1 struct
    - `struct ArmRegistry` · 2 Methoden
    - `enum ArmRegistryError`
- `dispatch`
  - 1 fn
    - `fn dispatch_to_slm`
- `fc_ts_dispatch`
  - 1 enum, 2 fn
    - `enum FcTsDispatchError`
    - `fn select_profile_fc_ts`
    - `fn deterministic_fc_ts_rng`
- `outcome`
  - 1 enum, 2 struct
    - `struct DecisionId` · 2 Methoden
    - `struct DecisionIdGenerator` · 2 Methoden
    - `enum RoutingOutcome` · 1 Methoden
- `profile`
  - 1 enum, 3 struct, 1 type
    - `enum QuantizationLevel`
    - `struct SlmProfile` · 6 Methoden
    - `struct ProfileCalibrator` · 4 Methoden
    - `type ConformalCalibrator`
    - `struct ProfileCalibrationState` · 6 Methoden
- `router`
  - 4 struct, 1 type
    - `struct ConfidenceMetrics`
    - `struct RoutingDecision`
    - `struct RouterState`
    - `type DefaultRouterEngine`
    - `struct RouterEngine`
- `routing_strategy`
  - 1 enum
    - `enum RoutingStrategy`
- `serde_helpers`
  - 2 fn
    - `fn serialize`
    - `fn deserialize`
- `transport`
  - 1 enum
    - `enum Transport` · 1 Methoden
- Zusätzlich 2 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_adapt::bandit`, `contextra_adapt::lyapunov`, `contextra_adapt::offpolicy`, `contextra_privacy::guarded_payload`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-mcp` | `SlmProfile`×4, `DefaultRouterEngine`×2, `ports_local::HybridSearchProvider`, `ports_local::CommunityResolver`, `ports_local::ContextPreparer`, `RouterEngine::new` | `SlmProfile::new` |
| `contextra-agent` | `DefaultRouterEngine`, `DecisionId`, `RoutingOutcome::Success`, `RoutingOutcome::Rejected` | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 20 von 23)**: `ArmRegistry`, `ArmRegistryError`, `dispatch_to_slm`, `FcTsDispatchError`, `select_profile_fc_ts`, `deterministic_fc_ts_rng`, `DecisionIdGenerator`, `RoutingOutcome`, `QuantizationLevel`, `ProfileCalibrator`, `ConformalCalibrator`, `ProfileCalibrationState`, `ConfidenceMetrics`, `RoutingDecision`, `RouterState`, `RouterEngine`, `RoutingStrategy`, `serialize`, `deserialize`, `Transport`

**Auffälligkeiten**

- contextra-wire: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-sandbox

- Pfad: `crates/contextra-sandbox` · Zeilen: 3781 · Rust-Dateien: 18 · Ring: 2
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `approval`
  - 3 enum, 1 fn, 1 struct
    - `enum ApprovalRisk`
    - `fn classify_risk`
    - `enum ApprovalStatus`
    - `enum ApprovalTransitionError`
    - `struct ApprovalRequest` · 7 Methoden
- `capabilities`
  - 2 struct
    - `struct WasmCapabilities` · 1 Methoden
    - `struct MergeOperatorCapabilities` · 2 Methoden
- `error`
  - 1 enum
    - `enum SandboxError`
- `executor`
  - 2 struct
    - `struct WasmExecutor` · 3 Methoden
    - `struct CapabilityViolationError` *(re-exportiert)*
- `merge`
  - 1 struct
    - `struct WasmMergeFunction` · 4 Methoden
- `output`
  - 1 struct
    - `struct WasmOutput` · 1 Methoden
- `wasi`
  - 2 struct
    - `struct ProcessExitError`
    - `struct OutputLimitExceededError`
- Zusätzlich 1 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-store` | – | `MergeOperatorCapabilities` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 13 von 14)**: `ApprovalRisk`, `classify_risk`, `ApprovalStatus`, `ApprovalTransitionError`, `ApprovalRequest`, `WasmCapabilities`, `SandboxError`, `WasmExecutor`, `CapabilityViolationError`, `WasmMergeFunction`, `WasmOutput`, `ProcessExitError`, `OutputLimitExceededError`

### contextra-simd

- Pfad: `crates/contextra-simd` · Zeilen: 4269 · Rust-Dateien: 12 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 5 | 1 | 2 | `Result`×4, `ContextraError` | `DistanceMetric` |

**Pro Komponente**

- `(crate-root)` (176 Zeilen)
  - → `contextra-types`: prod: `Result`×4
- `dispatch` (919 Zeilen)
  - → `contextra-types`: prod: `ContextraError` · dev: `DistanceMetric`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 4 fn
    - `fn validate_vector` ⚠ fremd: `contextra-types`
    - `fn compute_distance` ⚠ fremd: `contextra-types`
    - `fn compute_distance_trusted` ⚠ fremd: `contextra-types`
    - `fn compute_distance_f32_bytes_trusted` ⚠ fremd: `contextra-types`
- `dispatch`
  - 9 fn
    - `fn cosine_distance` ⚠ fremd: `contextra-types`
    - `fn euclidean_distance` ⚠ fremd: `contextra-types`
    - `fn dot_product_distance` ⚠ fremd: `contextra-types`
    - `fn cosine_distance_f32_bytes` ⚠ fremd: `contextra-types`
    - `fn euclidean_distance_f32_bytes` ⚠ fremd: `contextra-types`
    - `fn dot_product_distance_f32_bytes` ⚠ fremd: `contextra-types`
    - `fn dot_product_u8` ⚠ fremd: `contextra-types`
    - `fn euclidean_distance_sq_u8` ⚠ fremd: `contextra-types`
    - `fn cosine_similarity_parts_u8` ⚠ fremd: `contextra-types`
- `kernels`
  - 13 fn, 2 struct
    - `struct CosineSimilarityPartsU8`
    - `struct CosineSimilarityPartsF32U8`
    - `fn cosine_distance_scalar`
    - `fn euclidean_distance_scalar`
    - `fn dot_product_scalar`
    - `fn cosine_distance_f32_bytes_scalar`
    - `fn euclidean_distance_f32_bytes_scalar`
    - `fn dot_product_f32_bytes_scalar`
    - `fn dot_product_u8_scalar`
    - `fn euclidean_distance_sq_u8_scalar`
    - `fn cosine_similarity_parts_u8_scalar`
    - `fn dot_product_f32_u8`
    - `fn euclidean_distance_sq_f32_u8`
    - `fn cosine_similarity_parts_f32_u8`
    - `fn normalize_inplace`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-vector` | `validate_vector`×2, `kernels::scalar` | `kernels::scalar::dot_product_scalar`, `kernels::scalar::euclidean_distance_scalar`, `kernels::scalar::cosine_distance_scalar`, `kernels::scalar::dot_product_u8_scalar` … (+2) |

**Nach außen ungenutzte öffentliche Items (Heuristik, 21 von 28)**: `cosine_distance`, `euclidean_distance`, `dot_product_distance`, `cosine_distance_f32_bytes`, `euclidean_distance_f32_bytes`, `dot_product_distance_f32_bytes`, `dot_product_u8`, `euclidean_distance_sq_u8`, `cosine_similarity_parts_u8`, `CosineSimilarityPartsU8`, `CosineSimilarityPartsF32U8`, `cosine_distance_f32_bytes_scalar`, `euclidean_distance_f32_bytes_scalar`, `dot_product_f32_bytes_scalar`, `dot_product_f32_u8`, `euclidean_distance_sq_f32_u8`, `cosine_similarity_parts_f32_u8`, `normalize_inplace`, `compute_distance`, `compute_distance_trusted`, `compute_distance_f32_bytes_trusted`

### contextra-store

- Pfad: `crates/contextra-store` · Zeilen: 45672 · Rust-Dateien: 224 · Ring: 1
- Interne Cargo-Abhängigkeiten: `contextra-graph` (dev), `contextra-sandbox` (dev), `contextra-store` (dev), `contextra-testkit` (dev), `contextra-vector` (dev), `contextra-crypto` (normal), `contextra-mvcc` (normal), `contextra-ports` (normal), `contextra-sys` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 39 | 144 | 17 | `Clock`×7, `MetricsSink`×6, `MAX_SCAN_MERGE_ACCUMULATOR`×6, `StorageStats`×5, `scanner`×5, `StorageEngine`×3, `ResourceTracker`×2, `SystemClock::new`×2, `license::FeatureRing`, `license::FeatureRing::Sovereign` … (+1) | `ResourceBudget`×27, `ResourceTracker::new`×26, `clock::SystemClock`, `storage::StorageEngine` |
| `contextra-types` | 26 | 113 | 12 | `ContextraError::Storage`×5, `TOMBSTONE_BIT`×4, `TxId`×4, `ContextraError::MemoryBudgetExceeded`×2, `ContextraError::Internal`×2, `Result`×2, `ContextraError`×2, `TxId::INTERNAL_BASE`×2, `ContextraError::Sandbox`, `ContextraError::DurabilityConfig` … (+1) | `CollectionId`×5, `DocId`×5, `ContextraError::InvalidInput`×2, `TenantId`, `*` |
| `contextra-mvcc` | 10 | 6 | 6 | `SsiValidator`×4, `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`×2, `SnapshotRegistry`, `SequenceLogSsiValidator`, `IndexOp`, `SequenceLogSsiValidator::new_with_bounds` | `SnapshotRegistry::new` |
| `contextra-crypto` | 5 | 2 | 5 | `wal_crypto`×2, `verify_wal_chain_completeness`, `wal_crypto::WalHmac`, `crypto::KeyManager` | – |
| `contextra-sys` | 3 | 4 | 2 | `mmap_readonly`×2, `set_restrictive_file_acl` | `reopen_and_dup2`×2, `verify_file_acl_owner_only`, `set_fd_limit` |
| `contextra-sandbox` | 0 | 1 | 0 | – | `MergeOperatorCapabilities` |
| `contextra-testkit` | 0 | 8 | 0 | – | `ManualClock`×4, `ReferenceModel`×4 |

**Pro Komponente**

- `compaction` (3242 Zeilen)
  - → `contextra-mvcc`: prod: `SnapshotRegistry` · dev: `SnapshotRegistry`
  - → `contextra-ports`: prod: `StorageStats`×2, `ResourceTracker`×2, `Clock`×2, `SystemClock::new` · dev: `ResourceTracker::new`×26, `ResourceBudget`×26, `StorageEngine`×2
  - → `contextra-types`: prod: `ContextraError::MemoryBudgetExceeded`×2, `ContextraError::Internal`×2, `ContextraError::Sandbox`, `Result`, `TOMBSTONE_BIT` · dev: `TxId::new`×3, `TOMBSTONE_BIT`×2, `ContextraError::MemoryBudgetExceeded`×2, `TxId`
- `kv` (216 Zeilen)
  - → `contextra-types`: prod: `ContextraError`
- `lsm` (6282 Zeilen)
  - → `contextra-crypto`: prod: `verify_wal_chain_completeness`
  - → `contextra-mvcc`: prod: `SsiValidator`×4, `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`, `SequenceLogSsiValidator`, `IndexOp`, `SequenceLogSsiValidator::new_with_bounds`
  - → `contextra-ports`: prod: `MAX_SCAN_MERGE_ACCUMULATOR`×6, `Clock`×5, `MetricsSink`×5, `StorageStats`×2, `StorageEngine`×2, `license::FeatureRing`, `license::FeatureRing::Sovereign`, `NoopMetricsSink`, `SystemClock::new` · dev: `Clock`
  - → `contextra-testkit`: dev: `ManualClock`
  - → `contextra-types`: prod: `ContextraError::Storage`×5, `TxId`×4, `ContextraError`, `ContextraError::DurabilityConfig`, `Result`, `TxId::new`, `TxId::INTERNAL_BASE` · dev: `TOMBSTONE_BIT`
- `memtable` (968 Zeilen)
  - → `contextra-mvcc`: prod: `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`
  - → `contextra-types`: dev: `TOMBSTONE_BIT`
- `scanner` (811 Zeilen)
  - → `contextra-ports`: prod: `scanner`×5, `StorageEngine` · dev: `clock::SystemClock`, `StorageEngine`, `scanner`
  - → `contextra-types`: dev: `CollectionId`×5, `DocId`×5
- `sstable` (3296 Zeilen)
  - → `contextra-types`: prod: `TOMBSTONE_BIT`×3, `TxId::INTERNAL_BASE` · dev: `ContextraError`
- `system_pressure` (321 Zeilen)
  - → `contextra-ports`: prod: `MetricsSink`
- `tenant_codec` (821 Zeilen)
  - → `contextra-ports`: prod: `StorageStats`
- `wal` (7066 Zeilen)
  - → `contextra-crypto`: prod: `wal_crypto`×2, `wal_crypto::WalHmac`, `crypto::KeyManager` · dev: `wal_crypto::WalHmac`, `crypto::KeyManager`
  - → `contextra-sys`: prod: `mmap_readonly`×2, `set_restrictive_file_acl` · dev: `verify_file_acl_owner_only`
  - → `contextra-types`: dev: `TxId`×6
- `benches/lsm_concurrency_bench.rs` (129 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `benches/redb_comparative_bench.rs` (508 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `benches/wal_bench.rs` (143 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `examples/chaos_writer.rs` (55 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `fuzz/fuzz_targets/fuzz_compaction_interleave.rs` (110 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `fuzz/fuzz_targets/fuzz_recovery_arbitrary_state.rs` (52 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `fuzz/fuzz_targets/wal_mutation_chaos.rs` (272 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/adaptive_compaction_determinism.rs` (35 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`
- `tests/adaptive_compaction_regression.rs` (35 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/adaptive_compaction_tombstone.rs` (40 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/amplification_benchmark.rs` (343 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/blocking_util_pool_measurement.rs` (90 Zeilen)
  - → `contextra-ports`: dev: `MetricsSink`
- `tests/budget_drift.rs` (54 Zeilen)
  - → `contextra-mvcc`: dev: `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`
- `tests/campaign_fuzzregress_memtable_concurrent.rs` (105 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/campaign_j11_memory_deletion.rs` (48 Zeilen)
  - → `contextra-ports`: dev: `storage::StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/campaign_j21_hypotheses.rs` (488 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/campaign_j41_store.rs` (36 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/campaign_prop_lsm_ops.rs` (397 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/campaign_stress_store.rs` (294 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/chaos_bitflip_sstable.rs` (359 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/chaos_dropped_write.rs` (146 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-sys`: dev: `reopen_and_dup2`×2
- `tests/chaos_matrix.rs` (278 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/chaos_memory_pressure.rs` (217 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/chaos_power_cut.rs` (159 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `ContextraError`
- `tests/chaos_task_massacre.rs` (163 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_cancel_chain_consistency.rs` (49 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_cancellation_atomicity.rs` (147 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_core_batch_rollback.rs` (43 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_core_cancel_safety.rs` (54 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_paths_equivalence.rs` (95 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_read_lock_concurrency.rs` (61 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_rollback_batch_scoped.rs` (97 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/compaction_contiguous_selection.rs` (154 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`, `StorageStats`
  - → `contextra-types`: dev: `TxId`
- `tests/compaction_correctness_and_pinning.rs` (80 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/compaction_key_range_selection.rs` (101 Zeilen)
  - → `contextra-mvcc`: dev: `SnapshotRegistry::new`
  - → `contextra-ports`: dev: `ResourceTracker`, `ResourceBudget`
- `tests/compaction_merge_operator.rs` (508 Zeilen)
  - → `contextra-mvcc`: dev: `SnapshotRegistry`
  - → `contextra-sandbox`: dev: `MergeOperatorCapabilities`
  - → `contextra-testkit`: dev: `ManualClock`
- `tests/compaction_noncontiguous_shadowing.rs` (78 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/compaction_overlap_scan_and_tombstone.rs` (216 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/crash_kill_durability.rs` (441 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/crash_prefix_enumeration.rs` (133 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ReferenceModel`
  - → `contextra-types`: dev: `TxId`
- `tests/crash_recovery.rs` (111 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/differential/redb_operation_sequence.rs` (853 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/disk_full_during_write.rs` (76 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/docid_128_kv_engine.rs` (64 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/durability_mode_feature_gate.rs` (74 Zeilen)
  - → `contextra-ports`: dev: `license::FeatureRing`
- `tests/encryption_test.rs` (106 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/fault_injection_recovery.rs` (120 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/filename_uniqueness_test.rs` (88 Zeilen)
  - → `contextra-mvcc`: dev: `SnapshotRegistry`
  - → `contextra-types`: dev: `TxId`
- `tests/flush_crash_simulation.rs` (179 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/flush_durability.rs` (64 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/flush_mvcc_versions.rs` (184 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/flush_recovery_lifecycle.rs` (186 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/flush_threshold_boundary.rs` (68 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/fsync_syscall_verification.rs` (62 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/fsync_timeout_rollback.rs` (67 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_fault_injection.rs` (96 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/group_commit_flush_race.rs` (171 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_follower_timeout_chain.rs` (69 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_leader_cancel_chain.rs` (65 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_overlapping_batch_rollback.rs` (73 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_stress.rs` (129 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_test.rs` (223 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `ContextraError`
- `tests/group_commit_timeout.rs` (171 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/kv_delete_mode_wiring_test.rs` (107 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/lsm_robustness_fixes.rs` (139 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/lsm_storage_merge_operator.rs` (112 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `ContextraError::InvalidInput`×2
- `tests/manifest_compaction_audit.rs` (117 Zeilen)
  - → `contextra-mvcc`: dev: `SnapshotRegistry`
  - → `contextra-types`: dev: `TOMBSTONE_BIT`
- `tests/manifest_crash_no_resurrection.rs` (206 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/manifest_replace_rank_compat.rs` (68 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/memory_only_crash_semantics.rs` (50 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/metrics_sink_test.rs` (278 Zeilen)
  - → `contextra-ports`: dev: `MetricsSink`, `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/model_based_mvcc_flush.rs` (185 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ReferenceModel`
  - → `contextra-types`: dev: `TxId`
- `tests/observer_blocking_failopen.rs` (169 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/observer_deadline_no_runtime_block.rs` (67 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/proptest_model_based.rs` (237 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`, `Result`
- `tests/read_path_audit.rs` (210 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/recovery_fd_exhaustion_regression.rs` (93 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-sys`: dev: `set_fd_limit`
  - → `contextra-types`: dev: `TxId`
- `tests/resource_leaks.rs` (88 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/rollback_sstables.rs` (62 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/scan_limits_and_ranges.rs` (59 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/ssi_tracked_read_seq_clamp.rs` (122 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/ssi_tracked_reads_via_trait.rs` (120 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/ssi_unbounded_snapshot_seq.rs` (163 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/ssi_write_skew_integration.rs` (274 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/sstable_format_v3_to_v4_compat.rs` (158 Zeilen)
  - → `contextra-types`: dev: `TOMBSTONE_BIT`
- `tests/support/mod.rs` (79 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ReferenceModel`
- `tests/tenant_scoped_storage.rs` (131 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/test_wp02_flush_mvcc_bug.rs` (39 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ReferenceModel`
  - → `contextra-types`: dev: `TxId`, `Result`
- `tests/toctou_put_if_absent.rs` (209 Zeilen)
  - → `contextra-types`: dev: `ContextraError`, `Result`, `*`
- `tests/tombstone_scan.rs` (120 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_backpressure.rs` (294 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_flusher_rollback.rs` (92 Zeilen)
  - → `contextra-types`: dev: `Result`×2, `TxId`
- `tests/wal_hmac_chain_ordering.rs` (178 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_hmac_rollback_race.rs` (122 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_legacy_key_forced_migration.rs` (228 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_legacy_key_migration_store.rs` (93 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_legacy_key_retirement_marker.rs` (109 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_legacy_migration_crash.rs` (44 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_no_hmac_replay.rs` (52 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_observer.rs` (178 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_observer_contract.rs` (150 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ManualClock`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_observer_integration.rs` (269 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ManualClock`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_observer_test.rs` (181 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_recovery_runtime_stall.rs` (116 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_systematic_crash.rs` (199 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `ContextraError::Storage`×2, `TxId`
- `tests/wal_torn_write_fault_injection.rs` (113 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_truncate_ordering.rs` (257 Zeilen)
  - → `contextra-types`: dev: `ContextraError`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `compaction`
  - 1 enum, 1 fn, 7 struct, 2 trait
    - `struct WorkloadMetrics` · 4 Methoden
    - `struct WorkloadMetricsSnapshot`
    - `enum CompactionStrategy`
    - `struct AdaptiveCompactionPlan`
    - `trait AdaptiveCompactionPlanner` · Methoden: plan_compaction
    - `struct CostBasedAdaptivePlanner` · 2 Methoden
    - `struct CompactionConfig` *(re-exportiert)*
    - `struct CompactionEngine` *(re-exportiert)* · 14 Methoden
    - `trait MergeOperator` · Methoden: merge
    - `fn retain_key_versions`
    - `struct TtlMetadata`
- `engine`
  - 1 struct
    - `struct InMemoryStorageEngine` · 1 Methoden
- `fsync`
  - 3 fn
    - `fn sync_file`
    - `fn sync_parent_dir`
    - `fn fsync_parent_dir`
- `kv`
  - 1 enum, 3 struct
    - `enum KvDeleteMode`
    - `struct KvSegmentConfig`
    - `struct KvSegmentPayload`
    - `struct KvSegmentManager` · 6 Methoden
- `lsm`
  - 7 const, 4 enum, 7 struct, 1 trait
    - `enum DurabilityMode` · 1 Methoden
    - `enum DurabilityConfigError`
    - `struct LsmConfig`
    - `enum StorageHealth` *(re-exportiert)*
    - `struct LsmStorage` *(re-exportiert)* · 31 Methoden ⚠ fremd: `contextra-mvcc`
    - `const MAX_KEY_SIZE`
    - `const MAX_VALUE_SIZE`
    - `const MAX_BATCH_SIZE`
    - `const MAX_INTERNAL_MERGE_ENTRIES_FACTOR`
    - `const MAX_GROUP_COMMIT_BATCH_SIZE`
    - `const MIN_ENTRIES_FOR_SSTABLE_REBUILD`
    - `const DEFAULT_MAX_OBSERVER_LATENCY`
    - `enum WriteOrigin`
    - `struct WalEntryRef` · 6 Methoden
    - `struct CommitContext` ⚠ fremd: `contextra-types`
    - `struct CommittedBatch`
    - `trait WalObserver` · Methoden: requires_durability, on_commit
    - `struct ObserverRegistry` · 14 Methoden
    - `struct AsyncObserverAdapter` · 1 Methoden
- `manifest`
  - 2 const, 1 enum, 1 struct
    - `struct Manifest` *(re-exportiert)* · 8 Methoden
    - `enum ManifestEntry` *(re-exportiert)* · 2 Methoden
    - `const MAX_MANIFEST_ENTRY_SIZE`
    - `const DEFAULT_ROLLOVER_THRESHOLD_BYTES`
- `memtable`
  - 1 const, 2 fn, 1 struct
    - `fn is_tombstone`
    - `fn is_valid_range_bounds`
    - `const RESERVED_PREFIXES`
    - `struct MemTable` · 16 Methoden
- `scanner`
  - 2 fn, 4 struct
    - `struct MemtableScanner` · 1 Methoden
    - `fn lsm_scanners`
    - `fn lsm_purgers`
    - `struct LsmPurger` · 1 Methoden
    - `struct SstableScanner` · 1 Methoden
    - `struct WalScanner` · 1 Methoden
- `sstable`
  - 3 const, 8 fn, 10 struct, 1 trait, 2 type
    - `const BLOCK_CACHE_SHARDS` *(re-exportiert)*
    - `struct SieveCacheBackend` *(re-exportiert)*
    - `trait BlockCacheBackend` *(re-exportiert)* · Methoden: new, get, insert, len, is_empty, contains
    - `struct LruBlockCacheBackend` *(re-exportiert)*
    - `struct QuickCacheBlockCacheBackend` *(re-exportiert)*
    - `type BlockCacheShard` *(re-exportiert)*
    - `type BlockCacheShard` *(re-exportiert)*
    - `struct BlockCache` *(re-exportiert)* · 8 Methoden
    - `const SSTABLE_MAGIC_MFSX` *(re-exportiert)*
    - `const SSTABLE_MAGIC_LEGACY` *(re-exportiert)*
    - `fn create_block_cache` *(re-exportiert)*
    - `fn create_block_cache_with_shards` *(re-exportiert)*
    - `fn get_entry_at_index` *(re-exportiert)*
    - `fn get_entry_key_at_index` *(re-exportiert)*
    - `fn binary_search_first_index_in_block` *(re-exportiert)*
    - `fn binary_search_index_in_block` *(re-exportiert)*
    - `fn binary_search_entry_in_block` *(re-exportiert)*
    - `fn block_binary_search` *(re-exportiert)*
    - `struct BloomFilter` *(re-exportiert)* · 6 Methoden
    - `struct BlockBuilder` *(re-exportiert)* · 6 Methoden
    - `struct SstableMetadata` *(re-exportiert)*
    - `struct SstableBuilder` *(re-exportiert)* · 5 Methoden
    - `struct SstableReader` *(re-exportiert)* · 13 Methoden
    - `struct SstableStream` *(re-exportiert)* · 2 Methoden
- `system_pressure`
  - 6 const, 1 enum, 2 struct
    - `const WAL_QUEUE_CRITICAL_THRESHOLD`
    - `const WAL_QUEUE_ELEVATED_THRESHOLD`
    - `const SCHEDULER_QUEUE_CRITICAL`
    - `const SCHEDULER_QUEUE_ELEVATED`
- `tenant_codec`
  - 2 struct, 1 trait
- `wal`
  - 5 const, 4 enum, 2 fn, 6 static, 6 struct, 1 trait, 1 type
  - … 33 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 27 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_crypto::crypto::KeyManager`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `LsmStorage`×5, `SystemPressure`×3, `tenant_codec::TenantScopedStorage`×3, `tenant_codec::TenantScopedStorage::new`×3, `PressureLevel::Critical`×2, `scanner::lsm_purgers`×2, `scanner::lsm_scanners`×2, `PressureLevel::Normal`, `PressureLevel::Elevated`, `LsmConfig` … (+2) | `lsm`×10, `compaction`×2, `sstable`×2 |
| `contextra-agent` | `LsmStorage`×3 | – |
| `contextra` | `kv::delete_mode::KvDeleteMode`, `lsm::config` | `lsm::config::DurabilityMode`, `lsm::DurabilityMode` |
| `contextra-checkpoint` | – | `lsm` |
| `contextra-cognition` | – | `LsmStorage`×8, `LsmConfig`×8 |
| `contextra-db` | – | `LsmConfig`×5, `LsmStorage::new`×4, `LsmStorage`×2, `lsm` |
| `contextra-graph` | – | `LsmConfig`, `LsmStorage::new` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 104 von 113)**: `WorkloadMetrics`, `WorkloadMetricsSnapshot`, `CompactionStrategy`, `AdaptiveCompactionPlan`, `AdaptiveCompactionPlanner`, `CostBasedAdaptivePlanner`, `CompactionConfig`, `CompactionEngine`, `retain_key_versions`, `TtlMetadata`, `InMemoryStorageEngine`, `sync_file`, `sync_parent_dir`, `fsync_parent_dir`, `KvSegmentConfig`, `KvSegmentPayload`, `KvSegmentManager`, `DurabilityConfigError`, `StorageHealth`, `MAX_KEY_SIZE`, `MAX_VALUE_SIZE`, `MAX_BATCH_SIZE`, `MAX_INTERNAL_MERGE_ENTRIES_FACTOR`, `MAX_GROUP_COMMIT_BATCH_SIZE`, `MIN_ENTRIES_FOR_SSTABLE_REBUILD`, `DEFAULT_MAX_OBSERVER_LATENCY`, `WriteOrigin`, `WalEntryRef`, `CommitContext`, `CommittedBatch` …

**Auffälligkeiten**

- contextra-graph: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-store: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-vector: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-sys

- Pfad: `crates/contextra-sys` · Zeilen: 1640 · Rust-Dateien: 7 · Ring: 0
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `acl_win32`
  - 6 fn
    - `fn set_restrictive_file_acl`
    - `fn verify_file_acl_owner_only`
    - `fn set_restrictive_file_acl`
    - `fn verify_file_acl_owner_only`
    - `fn set_restrictive_file_acl`
    - `fn verify_file_acl_owner_only`
- `durability`
  - 3 fn
    - `fn durable_flush_file`
    - `fn durable_flush_dir`
    - `fn write_all_durable`
- `mlock`
  - 1 enum, 7 fn
    - `enum MemLockPolicy`
    - `fn mem_lock_with_policy`
    - `fn mem_unlock`
    - `fn mem_lock_with_policy`
    - `fn mem_unlock`
    - `fn mem_lock_with_policy`
    - `fn mem_unlock`
    - `fn mem_lock`
- `mmap`
  - 1 fn
    - `fn mmap_readonly`
- `posix`
  - 4 fn
    - `fn reopen_and_dup2`
    - `fn reopen_and_dup2`
    - `fn ensure_fd_limit`
    - `fn set_fd_limit`
- `vault`
  - 1 struct
    - `struct LockedRegions` · 7 Methoden

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-store` | `mmap_readonly`×2, `set_restrictive_file_acl` | `reopen_and_dup2`×2, `verify_file_acl_owner_only`, `set_fd_limit` |
| `contextra-durable-fs` | `FlushKind`, `rename_durable` | `fs_class` |
| `contextra-vector` | `mmap_readonly`×2 | – |
| `contextra-db` | `LockedRegions` | – |
| `contextra-bench` | – | `ensure_fd_limit` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 11 von 23)**: `durable_flush_file`, `durable_flush_dir`, `write_all_durable`, `MemLockPolicy`, `mem_lock_with_policy`, `mem_unlock`, `mem_lock_with_policy`, `mem_unlock`, `mem_lock_with_policy`, `mem_unlock`, `mem_lock`

### contextra-testkit

- Pfad: `crates/contextra-testkit` · Zeilen: 4320 · Rust-Dateien: 21 · Ring: tooling
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 12 | 1 | 9 | `Clock`×2, `GraphIndex`, `MetricsSink`, `Rng`, `StorageEngine`, `TextIndex`, `VectorIndex`, `scanner::WitnessLayer`, `scanner::ScanTechnique`, `scanner::LayerWitness` … (+1) | `TextEmbeddingEngine` |
| `contextra-types` | 1 | 0 | 1 | `TxId` | – |

**Pro Komponente**

- `(crate-root)` (108 Zeilen)
  - → `contextra-ports`: prod: `scanner::WitnessLayer`, `scanner::ScanTechnique`, `scanner::LayerWitness`, `scanner::mint_witness_for_testkit`
- `conformance` (1276 Zeilen)
  - → `contextra-ports`: prod: `Clock`, `GraphIndex`, `MetricsSink`, `Rng`, `StorageEngine`, `TextIndex`, `VectorIndex`
  - → `contextra-types`: prod: `TxId`
- `manual_clock` (150 Zeilen)
  - → `contextra-ports`: prod: `Clock`
- `mocks` (276 Zeilen)
  - → `contextra-ports`: dev: `TextEmbeddingEngine`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 fn
    - `fn mint_witness_for_tests` ⚠ fremd: `contextra-ports`
- `conformance`
  - 7 fn, 1 struct, 1 type
    - `fn run_clock_suite` ⚠ fremd: `contextra-ports`
    - `struct ConformanceFailure` · 1 Methoden
    - `type ConformanceResult`
    - `fn run_graph_index_suite` ⚠ fremd: `contextra-ports`
    - `fn run_metrics_sink_suite` ⚠ fremd: `contextra-ports`
    - `fn run_rng_suite` ⚠ fremd: `contextra-ports`
    - `fn run_storage_engine_suite` ⚠ fremd: `contextra-ports`
    - `fn run_text_index_suite` ⚠ fremd: `contextra-ports`
    - `fn run_vector_index_suite` ⚠ fremd: `contextra-ports`
- `fault_vfs`
  - 2 struct
    - `struct FaultConfig`
    - `struct FaultVfs` · 10 Methoden
- `in_memory_graph`
  - 1 struct
    - `struct InMemoryGraphIndex` · 1 Methoden
- `in_memory_store`
  - 1 struct
    - `struct InMemoryStorageEngine` · 1 Methoden
- `in_memory_text`
  - 1 struct
    - `struct InMemoryTextIndex` · 1 Methoden
- `in_memory_vector`
  - 1 struct
    - `struct InMemoryVectorIndex` · 1 Methoden
- `manual_clock`
  - 1 struct
    - `struct ManualClock` · 6 Methoden
- `mocks`
  - 1 enum, 2 struct
    - `struct MockEmbedder` · 2 Methoden
    - `enum MetricEvent`
    - `struct TestMetricsSink` · 3 Methoden
- `reference_model`
  - 1 enum, 1 struct
    - `enum RefOp`
    - `struct ReferenceModel` · 13 Methoden

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra` | – | `mint_witness_for_tests` |
| `contextra-agent` | – | `InMemoryStorageEngine`×2, `TestMetricsSink` |
| `contextra-cognition` | – | `ManualClock`, `MockEmbedder` |
| `contextra-crypto` | – | `mint_witness_for_tests`×4, `FaultVfs` |
| `contextra-db` | – | `mint_witness_for_tests` |
| `contextra-engine` | – | `mint_witness_for_tests`, `InMemoryStorageEngine` |
| `contextra-mvcc` | – | `ManualClock`×6 |
| `contextra-privacy` | – | `ManualClock`×2 |
| `contextra-store` | – | `ManualClock`×4, `ReferenceModel`×4 |

**Nach außen ungenutzte öffentliche Items (Heuristik, 15 von 22)**: `run_clock_suite`, `ConformanceFailure`, `ConformanceResult`, `run_graph_index_suite`, `run_metrics_sink_suite`, `run_rng_suite`, `run_storage_engine_suite`, `run_text_index_suite`, `run_vector_index_suite`, `FaultConfig`, `InMemoryGraphIndex`, `InMemoryTextIndex`, `InMemoryVectorIndex`, `MetricEvent`, `RefOp`

### contextra-text

- Pfad: `crates/contextra-text` · Zeilen: 13329 · Rust-Dateien: 50 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 8 | 87 | 5 | `StorageEngine`×6, `scanner`×2 | `BoxFuture`×50, `StorageStats`×35, `clock::SystemClock` |
| `contextra-types` | 0 | 136 | 0 | – | `Result`×67, `TOMBSTONE_BIT`×57, `TxId`×7, `TxId::new`×2, `ContextraError::CapabilityUnsupported`×2 … (+1) |

**Pro Komponente**

- `(crate-root)` (296 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine`×3 · dev: `BoxFuture`
- `inverted` (2798 Zeilen)
  - → `contextra-ports`: prod: `scanner`×2, `StorageEngine` · dev: `StorageStats`×3
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6, `ContextChunk`
- `stream` (162 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine`
- `wand` (809 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine` · dev: `StorageStats`×2, `BoxFuture`
  - → `contextra-types`: dev: `TxId`×7, `TOMBSTONE_BIT`×6, `TxId::new`×2
- `tests/alloc_profiler.rs` (567 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `Result`×16
- `tests/audit_inspection.rs` (577 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`×32, `StorageStats`×4
  - → `contextra-types`: dev: `Result`×34, `TOMBSTONE_BIT`×10
- `tests/bm25_candidate_stream.rs` (320 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
- `tests/concurrent_metadata.rs` (195 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
- `tests/docid_128_tombstone_resolution.rs` (248 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`×16, `StorageStats`×2
  - → `contextra-types`: dev: `Result`×17, `TOMBSTONE_BIT`×6
- `tests/pipeline_identity_regression.rs` (270 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6
- `tests/proptest_bm25_invariants.rs` (334 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6
- `tests/text_scanner_purger_tests.rs` (302 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2, `clock::SystemClock`, `scanner`
- `tests/text_search_at_wrappers.rs` (330 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6, `ContextraError::CapabilityUnsupported`×2
- `tests/tombstone_read_your_writes.rs` (266 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6
- `tests/tombstone_update.rs` (220 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
- `tests/wand_differential.rs` (427 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×5
- `tests/wand_tie_break_threshold.rs` (229 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
- `tests/write_amplification.rs` (218 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 struct
    - `struct Bm25Scorer` · 1 Methoden ⚠ fremd: `contextra-ports`
- `bm25`
  - 2 const, 3 fn, 3 struct, 1 type
    - `const BM25_K1`
    - `const BM25_B`
    - `struct BM25` · 2 Methoden
    - `fn score_term`
    - `fn score_term_with_params`
    - `type FieldId`
    - `struct FieldWeight` · 1 Methoden
    - `struct BM25F` · 2 Methoden
    - `fn score_term_bm25f`
- `domain`
  - 4 fn, 4 static, 2 struct, 1 trait
    - `struct LegalDomainVocabulary`
    - `fn get_legal_compound_stems`
    - `fn get_legal_protected_terms`
    - `static LEGAL_COMPOUND_STEMS`
    - `static LEGAL_PROTECTED_TERMS`
    - `struct MedicalDomainVocabulary`
    - `fn get_medical_compound_stems`
    - `fn get_medical_protected_terms`
    - `static MEDICAL_COMPOUND_STEMS`
    - `static MEDICAL_PROTECTED_TERMS`
    - `trait DomainVocabulary` · Methoden: compound_stems, protected_terms, domain_id
- `inverted`
  - 1 enum, 2 fn, 5 struct
    - `struct InvertedIndex` *(re-exportiert)* · 12 Methoden
    - `struct BM25MorphIndex` *(re-exportiert)* · 2 Methoden
    - `struct PurgeReport` · 3 Methoden
    - `struct TextPurger` · 5 Methoden ⚠ fremd: `contextra-ports`
    - `fn text_purger` ⚠ fremd: `contextra-ports`
    - `enum ParsedTextTarget` · 1 Methoden
    - `struct TextScanner` · 4 Methoden
    - `fn text_scanner`
- `morphology`
  - 2 struct
    - `struct TokenReductionMetrics` *(re-exportiert)* · 1 Methoden
    - `struct PassthroughTokenizer` *(re-exportiert)* · 1 Methoden
- `posting_list`
  - 2 const, 4 struct, 2 type
    - `const POSTING_LIST_V2_MAGIC`
    - `const BLOCK_SIZE`
    - `type DocIdRaw`
    - `type DocIdRaw`
    - `struct PostingBlockInfo`
    - `struct Posting` · 2 Methoden
    - `struct PostingList` · 10 Methoden
    - `struct ResidentPostingIndex` · 7 Methoden
- `stream`
  - 1 const, 1 struct
    - `const DEFAULT_STREAM_BATCH_SIZE`
    - `struct Bm25CandidateStream` · 7 Methoden ⚠ fremd: `contextra-ports`
- `tokenizer`
  - 3 fn, 2 struct, 1 trait
    - `fn compile_protected_regex`
    - `fn get_protected_regex`
    - `trait Tokenizer` · Methoden: tokenize
    - `struct DefaultTokenizer` · 2 Methoden
    - `struct GermanMorphTokenizer` · 2 Methoden
    - `fn tokenize`
- `wand`
  - 1 fn, 2 struct
    - `struct BoundedTopK` · 4 Methoden
    - `struct WandSearchResult`
    - `fn block_max_wand_search` ⚠ fremd: `contextra-ports`
- Zusätzlich 9 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `Language`×2, `text_purger`×2, `text_scanner`×2, `inverted::InvertedIndex` | `Language::English`×63, `Language::German`×4 |
| `contextra-agent` | – | `Language::English`×5 |
| `contextra-db` | – | `Language::English`×7, `Language`×2, `bm25::score_term`, `inverted::InvertedIndex` |
| `contextra-router` | – | `Language::English` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 46 von 50)**: `BM25_K1`, `BM25_B`, `BM25`, `score_term_with_params`, `FieldId`, `FieldWeight`, `BM25F`, `score_term_bm25f`, `LegalDomainVocabulary`, `get_legal_compound_stems`, `get_legal_protected_terms`, `LEGAL_COMPOUND_STEMS`, `LEGAL_PROTECTED_TERMS`, `MedicalDomainVocabulary`, `get_medical_compound_stems`, `get_medical_protected_terms`, `MEDICAL_COMPOUND_STEMS`, `MEDICAL_PROTECTED_TERMS`, `DomainVocabulary`, `BM25MorphIndex`, `PurgeReport`, `TextPurger`, `ParsedTextTarget`, `TextScanner`, `Bm25Scorer`, `TokenReductionMetrics`, `PassthroughTokenizer`, `POSTING_LIST_V2_MAGIC`, `BLOCK_SIZE`, `DocIdRaw` …

**Auffälligkeiten**

- contextra-types: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?

### contextra-types

- Pfad: `crates/contextra-types` · Zeilen: 7968 · Rust-Dateien: 32 · Ring: 0
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `error`
  - 3 enum, 1 struct, 1 type
    - `enum HnswDeletionError`
    - `struct StepId` · 2 Methoden
    - `enum CacheDirective`
    - `type Result`
    - `enum ContextraError` · 14 Methoden
- `error_dto`
  - 1 struct
    - `struct ContextraErrorDto` · 2 Methoden
- `model_fingerprint`
  - 1 struct
    - `struct ModelFingerprint` · 1 Methoden
- `retrieval_strategy`
  - 1 enum
    - `enum RetrievalStrategy`
- `schema`
  - 2 enum
    - `enum DocIdWidth` · 2 Methoden
    - `enum ManifestSchemaVersion` · 4 Methoden
- `tenant_scope`
  - 1 enum, 1 struct
    - `struct TenantScoped` · 4 Methoden
    - `enum TenantScopeViolation`
- `tombstone`
  - 1 struct, 1 trait
    - `trait TombstoneSemanticsCheck` · Methoden: is_tombstone, make_tombstone
    - `struct SeqBitTombstone`
- `types`
  - 3 const, 12 enum, 2 fn, 30 struct, 2 type
    - `enum BudgetStrategy`
    - `struct TokenBudget` · 11 Methoden
    - `struct Reservation` · 1 Methoden
    - `struct ResourceBudget`
    - `struct ResourceTracker` · 6 Methoden
    - `enum DistanceMetric` *(re-exportiert)* · 2 Methoden
    - `struct Embedding` *(re-exportiert)* · 5 Methoden
    - `struct ScoredDocument` *(re-exportiert)* · 1 Methoden
    - `struct RerankResult` *(re-exportiert)*
    - `struct TenantId` *(re-exportiert)* · 5 Methoden
    - `struct CollectionId` *(re-exportiert)* · 4 Methoden
    - `struct DocId` *(re-exportiert)*
    - `struct DocId` *(re-exportiert)* · 8 Methoden
    - `struct EntityId` *(re-exportiert)* · 8 Methoden
    - `struct TxId` *(re-exportiert)* · 6 Methoden
    - `fn serialize` *(re-exportiert)*
    - `fn deserialize` *(re-exportiert)*
    - `struct WorkflowState` *(re-exportiert)*
    - `const TOMBSTONE_BIT` *(re-exportiert)*
    - `const EXPIRY_METADATA_KEY` *(re-exportiert)*
    - `const MAX_SEARCH_K` *(re-exportiert)*
    - `enum LinkRelation` *(re-exportiert)*
    - `struct MemoryLink` *(re-exportiert)*
    - `struct Entity` *(re-exportiert)* · 2 Methoden
    - `struct Edge` *(re-exportiert)* · 7 Methoden
    - `enum MemoryType` *(re-exportiert)* · 3 Methoden
    - `struct TlHfdParams` *(re-exportiert)*
    - `enum PprAlgorithm` *(re-exportiert)*
    - `struct PprConfig` *(re-exportiert)*
    - `struct ConfigFingerprint` *(re-exportiert)* · 4 Methoden
    - `enum AutoExtractionMode` *(re-exportiert)* · 1 Methoden
    - `enum FilterExpr` · 1 Methoden
    - `struct ImportanceScore` · 2 Methoden
    - `enum DecayFunction` · 2 Methoden
    - `struct MemoryImportance` · 2 Methoden
    - `enum FusionStrategy`
    - `struct SignalFusionStrategies` · 2 Methoden
    - `enum GraphTraversalStrategy`
    - `struct FusionWeights` · 5 Methoden
    - `struct ContextChunk` · 3 Methoden
    - `struct ContextWindow`
    - `enum Signal`
    - `enum OnSignalFailure`
    - `struct QueryRewriteOutput` · 3 Methoden
    - `struct ScoredEntry`
    - `struct HybridQuery` · 1 Methoden
    - `type QueryOptions`
    - `struct HybridQueryBuilder` · 17 Methoden
    - `type QueryOptionsBuilder`
- `witness`
  - 1 enum, 2 struct
    - `enum ScanMethod`
    - `struct RequestContext` · 1 Methoden
    - `struct SagaRecord` · 1 Methoden

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `ContextraError::invalid_input`×49, `Result`×17, `EntityId`×15, `ContextraError::Internal`×13, `HybridQuery`×13, `MAX_SEARCH_K`×11, `ContextraError::Serialization`×8, `ImportanceScore::new`×8, `domain::MemoryLink`×7, `EntityId::from_key`×7 … (+39) | `ContextraError::from`×10, `domain::LinkRelation::Elaborates`×2, `TenantId`×2, `ContextraError::HnswConnectivityDegraded` … (+9) |
| `contextra-vector` | `ContextraError::invalid_input`×19, `MAX_SEARCH_K`×15, `Result`×8, `ContextraError::HnswConnectivityDegraded`×4, `ContextraError::Index`×4, `DocId`×3, `DistanceMetric`×2 | `ContextraError::InvalidInput`×6, `ContextraError::EmbeddingDimensionMismatch`×4, `DistanceMetric::Euclidean`×3, `ContextraError`×3 … (+3) |
| `contextra-graph` | `EntityId`×16, `ContextraError::InvalidInput`×5, `ContextraError`×3, `Result`×3, `Edge`×2, `Edge::new`×2, `TxId`×2, `ContextraError::Internal`×2, `MemoryLink`, `LinkRelation::Supersedes` … (+3) | `Entity::new`×10, `TlHfdParams::default`×4, `PprAlgorithm::Auto`×4, `Entity`×2 |
| `contextra-crypto` | `TenantId`×8, `Result`×7, `ScanMethod::FeatureDisabled`×3, `ContextraError::Crypto`×3, `TenantScoped`×2, `ScanMethod::Structural`×2, `ScanMethod::NotMeasured`×2, `ContextraError`×2, `RequestContext`, `SagaRecord` … (+4) | `ContextraError::Io`×4, `ContextraError::NotFound` |
| `contextra-adapt` | `ContextraError::InvalidInput`×9, `RetrievalStrategy`×9, `ContextraError`×4, `ContextraError::Internal`×2, `RetrievalStrategy::Vector`×2, `TenantId`, `RetrievalStrategy::Text`, `RetrievalStrategy::Graph`, `RetrievalStrategy::Hybrid` | – |
| `contextra-agent` | `ContextraError::InvalidInput`×10, `Result`×4, `TxId`×3, `WorkflowState`×3, `TokenBudget::new`×2, `ContextraError::Internal`, `DocId::from_key`, `types::budget`, `error` | `TokenBudget`×13, `TxId::new`×4, `ContextraError::Conflict`, `ContextraError::Serialization` … (+1) |
| `contextra-store` | `ContextraError::Storage`×5, `TOMBSTONE_BIT`×4, `TxId`×4, `ContextraError::MemoryBudgetExceeded`×2, `ContextraError::Internal`×2, `Result`×2, `ContextraError`×2, `TxId::INTERNAL_BASE`×2, `ContextraError::Sandbox`, `ContextraError::DurabilityConfig` … (+1) | `CollectionId`×5, `DocId`×5, `ContextraError::InvalidInput`×2, `TenantId` … (+1) |
| `contextra-cognition` | `Result`×3, `TenantId`×3, `ContextraError::InvalidInput`×3, `ContextraError::Internal`×2, `ContextChunk`×2, `ContextraError::PolicyViolation`×2, `TxId`×2, `DocId` | `MemoryType::Episodic`×6, `DocId::new`×5 |
| `contextra-ports` | `ContextraError`×3, `RequestContext`×3, `Result`×2, `types::domain`, `model_fingerprint::ModelFingerprint`, `types::domain::TenantId`, `*`, `ContextraError::Plugin`, `ScanMethod` | – |
| `contextra-rank` | `ConfigFingerprint`×3, `SignalFusionStrategies`×3, `ContextraError`, `FusionWeights`, `FusionStrategy`, `MAX_SEARCH_K`, `OnSignalFailure`, `DocId` | – |
| `contextra-core` | `error`×2, `schema`×2, `tombstone`×2, `error_dto`, `model_fingerprint`, `types`, `error_dto::ContextraErrorDto`, `model_fingerprint::ModelFingerprint` | – |
| `contextra-privacy` | `TenantId`×5, `TenantScoped`×4, `TenantScopeViolation`, `TenantScoped::new` | – |
| `contextra` | `error::ContextraError`×3, `DistanceMetric`×2, `types::domain::AutoExtractionMode`, `types::domain`, `error::Result`, `ScoredEntry` | `Result`×2, `ContextraError`×2, `DocId`×2, `DocId::from_u64`×2 … (+5) |
| `contextra-infer-ollama` | `Result`×4, `ContextraError`×3, `ConfigFingerprint` | – |
| `contextra-infer-candle` | `ContextraError`×5, `DocId::from`, `ModelFingerprint` | `DocId`×3, `Result`×3, `ContextChunk`×2, `TenantId` |
| `contextra-infer-onnx` | `ContextraError`×2, `Result`, `ContextraError::CapabilityUnsupported`, `RerankResult` | – |
| `contextra-kvcache` | `TenantId`×3, `error::Result`, `error` | `model_fingerprint::ModelFingerprint`×4, `model_fingerprint::ModelFingerprint::new`, `ContextraError` |
| `contextra-mcp` | `ContextraError`×3, `ContextraError::InvalidInput`, `ContextraError::NotFound` | `TenantId::SYSTEM`, `AutoExtractionMode`, `TokenBudget::new`, `TenantId` … (+1) |
| `contextra-simd` | `Result`×4, `ContextraError` | `DistanceMetric` |
| `contextra-ingest` | `DocId`×2, `ContextraError` | `DocId::new` |
| `contextra-py` | `ContextraError`, `ContextraErrorDto::from`, `TenantId::SYSTEM` | – |
| `contextra-audit-export` | `TenantId`×2 | – |
| `contextra-db` | `DistanceMetric`, `MAX_SEARCH_K` | `Result`×8, `TxId`×6, `DocId`×4, `FusionWeights`×3 … (+17) |
| `contextra-avv-generator` | `TenantId` | – |
| `contextra-license` | `TenantId` | – |
| `contextra-mvcc` | `*` | `TxId`×2, `DocId` |
| `contextra-router` | `RetrievalStrategy` | `ConfigFingerprint`×7, `TokenBudget`×4, `DocId::new`×3, `Result` |
| `contextra-testkit` | `TxId` | – |
| `contextra-checkpoint` | – | `TxId`×5, `Result`×3, `ContextraError`×2, `ContextraError::Internal`×2 |
| `contextra-text` | – | `Result`×67, `TOMBSTONE_BIT`×57, `TxId`×7, `TxId::new`×2 … (+2) |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

### contextra-vector

- Pfad: `crates/contextra-vector` · Zeilen: 22182 · Rust-Dateien: 106 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-crypto` (normal), `contextra-mvcc` (normal), `contextra-ports` (normal), `contextra-simd` (normal), `contextra-sys` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 55 | 42 | 13 | `ContextraError::invalid_input`×19, `MAX_SEARCH_K`×15, `Result`×8, `ContextraError::HnswConnectivityDegraded`×4, `ContextraError::Index`×4, `DocId`×3, `DistanceMetric`×2 | `ContextraError::InvalidInput`×6, `ContextraError::EmbeddingDimensionMismatch`×4, `DistanceMetric::Euclidean`×3, `ContextraError`×3, `TxId::new` … (+2) |
| `contextra-ports` | 24 | 70 | 6 | `MetricsSink`×6, `Clock`×4, `Rng`×3, `VectorIndex`×2, `SystemClock`×2, `scanner`×2, `NoopMetricsSink`×2, `SeededRng::new`×2, `SystemClock::new` | `SeededRng`×5, `vector_index::VectorIndex`, `clock::SystemClock` |
| `contextra-mvcc` | 7 | 0 | 3 | `SequenceLog::new`×2, `SeqLogChange::Insert`, `SeqLogChange::Delete`, `TxBuffer`, `SequenceLog`, `IndexOp` | – |
| `contextra-crypto` | 6 | 2 | 2 | `wal_crypto::WalHmac::new`×6 | – |
| `contextra-simd` | 3 | 7 | 3 | `validate_vector`×2, `kernels::scalar` | `kernels::scalar::dot_product_scalar`, `kernels::scalar::euclidean_distance_scalar`, `kernels::scalar::cosine_distance_scalar`, `kernels::scalar::dot_product_u8_scalar`, `kernels::scalar::euclidean_distance_sq_u8_scalar` … (+1) |
| `contextra-sys` | 2 | 0 | 2 | `mmap_readonly`×2 | – |

**Pro Komponente**

- `acorn` (180 Zeilen)
  - → `contextra-types`: prod: `DocId`
- `candidate_stream` (147 Zeilen)
  - → `contextra-ports`: prod: `VectorIndex`
- `diskann` (3620 Zeilen)
  - → `contextra-crypto`: prod: `wal_crypto::WalHmac::new`×6 · dev: `wal_crypto::WalHmac::new`×2
  - → `contextra-ports`: prod: `VectorIndex` · dev: `VectorIndex`×2
  - → `contextra-simd`: prod: `validate_vector`
  - → `contextra-sys`: prod: `mmap_readonly`
  - → `contextra-types`: prod: `MAX_SEARCH_K`×9
- `distance` (216 Zeilen)
  - → `contextra-simd`: prod: `kernels::scalar` · dev: `kernels::scalar::dot_product_scalar`, `kernels::scalar::euclidean_distance_scalar`, `kernels::scalar::cosine_distance_scalar`, `kernels::scalar::dot_product_u8_scalar` … (+2)
  - → `contextra-types`: dev: `ContextraError::EmbeddingDimensionMismatch`×4, `DistanceMetric`
- `hnsw` (5460 Zeilen)
  - → `contextra-mvcc`: prod: `SequenceLog::new`×2, `SeqLogChange::Insert`, `SeqLogChange::Delete`, `TxBuffer`, `SequenceLog`, `IndexOp`
  - → `contextra-ports`: prod: `MetricsSink`×6, `Clock`×4, `Rng`×3, `SystemClock`×2, `scanner`×2, `NoopMetricsSink`×2, `SeededRng::new`×2, `SystemClock::new` · dev: `SeededRng::new`×3, `Rng`×2, `VectorIndex`, `MetricsSink`
  - → `contextra-simd`: prod: `validate_vector`
  - → `contextra-types`: prod: `MAX_SEARCH_K`×6, `ContextraError::HnswConnectivityDegraded`×4, `ContextraError::Index`×4, `DocId`×2, `Result`×2, `DistanceMetric` · dev: `DistanceMetric`
- `persistence` (1101 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`×3
  - → `contextra-sys`: prod: `mmap_readonly`
  - → `contextra-types`: dev: `Result`×2
- `quantize` (976 Zeilen)
  - → `contextra-types`: prod: `ContextraError::invalid_input`×19, `Result`×6, `DistanceMetric` · dev: `ContextraError::InvalidInput`×6, `Result`×2
- `benches/acorn_vs_post_filter.rs` (53 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `benches/audit_benchmarks.rs` (430 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/distance_bench.rs` (34 Zeilen)
  - → `contextra-simd`: dev: `kernels::scalar`
- `benches/distance_nan_impact_bench.rs` (24 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `benches/filtered_search_selectivity_bench.rs` (166 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/flush_threshold_amplification.rs` (177 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/hnsw_bench.rs` (90 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/hnsw_delete_search_bench.rs` (86 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/overfetch_selectivity.rs` (79 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`, `VectorIndex`
- `benches/partial_rebuild_recall_bench.rs` (136 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/sq8_bench.rs` (40 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `fuzz/fuzz_targets/fuzz_hnsw_persistence.rs` (119 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `fuzz/fuzz_targets/hnsw_insert_search.rs` (79 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/acorn_hnsw_differential.rs` (253 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/acorn_naive_recall.rs` (70 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/adaptive_ef_recall.rs` (279 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/campaign_hnsw_backoff.rs` (49 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/campaign_j23_vector_index.rs` (478 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `ContextraError::Index`×6
- `tests/campaign_j24_test.rs` (609 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/deleted_nodes_lock_contention.rs` (176 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/deletion_ghost_pointer_proptest.rs` (47 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/determinism_test.rs` (111 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/differential_testing.rs` (78 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/diskann_corruption_fallback_test.rs` (103 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `TxId::new`
- `tests/diskann_docid128_tombstones.rs` (67 Zeilen)
  - → `contextra-ports`: dev: `vector_index::VectorIndex`
  - → `contextra-types`: dev: `ContextraError::Io`
- `tests/diskann_fault_injection_test.rs` (73 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/diskann_recall_test.rs` (219 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `TxId`
- `tests/diskann_search_filtered.rs` (171 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `MAX_SEARCH_K`
- `tests/distance_determinism.rs` (64 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/docid_128_hnsw.rs` (70 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `Result`×2
- `tests/filtered_selectivity_recall.rs` (154 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`, `VectorIndex`
- `tests/hnsw_edge_cases.rs` (321 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/hnsw_layer_determinism.rs` (110 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/hnsw_rebuild_search_consistency.rs` (134 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `DistanceMetric::Euclidean`
- `tests/hnsw_scanner_purge_test.rs` (227 Zeilen)
  - → `contextra-ports`: dev: `clock::SystemClock`, `VectorIndex`, `scanner`
- `tests/hnsw_seed_determinism.rs` (295 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`, `VectorIndex`
- `tests/hnsw_snapshot_delete_test.rs` (129 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/k02_snapshot_rebuild_determinism.rs` (153 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `DistanceMetric::Euclidean`
- `tests/loom_quantizer_race_test.rs` (116 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`×2
  - → `contextra-types`: dev: `ContextraError`
- `tests/mmap_toctou_test.rs` (228 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`×2
- `tests/nan_validation_policy.rs` (171 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/partial_rebuild_recall.rs` (146 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/partial_rebuild_recall_regression.rs` (328 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/partial_rebuild_veto_f02_guard.rs` (98 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`, `VectorIndex`
- `tests/poisoning.rs` (109 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/proptest_distance_quantize.rs` (89 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/quantize_persistence_audit.rs` (302 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `ContextraError`
- `tests/quantizer_encapsulation_regression.rs` (35 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/rabitq_recall_calibration.rs` (117 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/ram_reduction.rs` (59 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/recalibration.rs` (56 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `DistanceMetric::Euclidean`
- `tests/recall.rs` (67 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/recall_audit.rs` (405 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/recall_vs_brute_force.rs` (155 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/reconstruction_attack_test.rs` (71 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/rollback.rs` (55 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/simd_numerical_audit.rs` (171 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/sq8_bias_calibration.rs` (20 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/sq8_rerank_recall.rs` (163 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`, `VectorIndex`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `acorn`
  - 1 enum, 1 fn, 1 struct, 1 trait
    - `fn compute_gamma_edge_budget`
    - `trait FilteredIndex` · Methoden: search_knn_acorn
    - `enum AcornError`
    - `struct NaiveReferenceIndex` · 3 Methoden
- `candidate_stream`
  - 1 const, 1 struct
    - `const DEFAULT_VECTOR_STREAM_BATCH_SIZE`
    - `struct VectorCandidateStream` · 8 Methoden ⚠ fremd: `contextra-ports`
- `compute_pool`
  - 2 struct
    - `struct TaskHandle` · 1 Methoden
    - `struct ComputePool` · 5 Methoden
- `diskann`
  - 1 struct
    - `struct DiskAnnIndex` *(re-exportiert)* · 2 Methoden
- `hnsw`
  - 2 const, 2 fn, 9 struct, 1 trait, 1 type
    - `struct AdaptiveEfPolicy` · 2 Methoden
    - `struct AdaptiveEfStats`
    - `struct AdaptiveEfStateMachine` · 6 Methoden
    - `const ARENA_ALIGNMENT_BYTES`
    - `const ARENA_ALIGNMENT_U32`
    - `type BacklinkKey`
    - `struct BacklinkTable` · 4 Methoden
    - `struct HnswArena` · 9 Methoden
    - `struct DeletionStats`
    - `trait GhostFreeVectorIndex` · Methoden: remove_with_graph_repair
    - `struct HnswPurger` · 1 Methoden
    - `fn hnsw_purger`
    - `struct HnswScanner` · 1 Methoden
    - `fn hnsw_scanner`
    - `struct Sq8Bias` · 2 Methoden
- `partial_rebuild`
  - 1 fn, 2 struct
    - `struct PartialRebuildConfig`
    - `struct TraversalTracker` · 4 Methoden
    - `fn should_trigger_partial_rebuild`
- `persistence`
  - 2 struct
    - `struct MmapIndex` *(re-exportiert)* · 5 Methoden
    - `struct NodeRecord` *(re-exportiert)* · 2 Methoden
- `quantize`
  - 2 const, 1 struct
    - `const DEFAULT_P_LOW`
    - `const DEFAULT_P_HIGH`
    - `struct ScalarQuantizer` · 17 Methoden
- `quantize_rabitq`
  - 1 struct
    - `struct RaBitQQuantizer` · 5 Methoden
- Zusätzlich 26 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `hnsw::HnswIndex`×3, `HnswIndex`×2, `hnsw_purger`×2, `hnsw_scanner`×2, `acorn::FilteredIndex` | `HnswConfig`×57, `hnsw::HnswConfig::default`, `hnsw::HnswIndex::try_new`, `acorn` … (+1) |
| `contextra-cognition` | `HnswIndex`×3 | `HnswConfig`×8 |
| `contextra-db` | – | `HnswConfig`×3, `HnswIndex`×2, `HnswIndex::try_new` |
| `contextra-router` | – | `HnswConfig::default`, `HnswIndex::try_new` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 30 von 33)**: `compute_gamma_edge_budget`, `AcornError`, `NaiveReferenceIndex`, `DEFAULT_VECTOR_STREAM_BATCH_SIZE`, `VectorCandidateStream`, `TaskHandle`, `ComputePool`, `DiskAnnIndex`, `AdaptiveEfPolicy`, `AdaptiveEfStats`, `AdaptiveEfStateMachine`, `ARENA_ALIGNMENT_BYTES`, `ARENA_ALIGNMENT_U32`, `BacklinkKey`, `BacklinkTable`, `HnswArena`, `DeletionStats`, `GhostFreeVectorIndex`, `HnswPurger`, `HnswScanner`, `Sq8Bias`, `PartialRebuildConfig`, `TraversalTracker`, `should_trigger_partial_rebuild`, `MmapIndex`, `NodeRecord`, `DEFAULT_P_LOW`, `DEFAULT_P_HIGH`, `ScalarQuantizer`, `RaBitQQuantizer`

### contextra-wire

- Pfad: `crates/contextra-wire` · Zeilen: 5089 · Rust-Dateien: 14 · Ring: 0
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `adapter`
  - 1 struct
    - `struct WireBuffer` · 3 Methoden
- `jsonrpc`
  - 1 enum, 2 fn, 4 struct
    - `struct JsonRpcConfig`
    - `enum JsonRpcPayload`
    - `struct JsonRpcRequest` · 1 Methoden
    - `struct JsonRpcResponse` · 2 Methoden
    - `struct JsonRpcError` · 5 Methoden
    - `fn parse_jsonrpc_payload`
    - `fn parse_jsonrpc_payload_with_config`
- `verification`
  - 1 enum, 9 fn, 1 struct
    - `enum FlatBufferError`
    - `struct FlatBufferConfig`
    - `fn verify_flatbuffer`
    - `fn verify_and_parse_search_response`
    - `fn verify_and_parse_embedding`
    - `fn verify_and_parse_scored_document`
    - `fn verify_and_parse_vector_index_update`
    - `fn verify_and_parse_role_id`
    - `fn verify_and_parse_hyper_edge_id`
    - `fn verify_and_parse_role_binding`
    - `fn verify_and_parse_hyper_edge`
- Zusätzlich 40 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-py` | `ScoredDocument::create`×2, `ScoredDocumentArgs`×2, `SearchResponse::create`×2, `SearchResponseArgs`×2 | – |
| `contextra-core` | `*` | – |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

