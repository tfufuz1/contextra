# Crate-Kopplungsbericht

- Erzeugt: 2026-11-05 23:09
- Workspace-Root: `/home/freddy/Projekte/contextra_06-10-26`
- Analysierte Crates: 36 · Komponenten-Tiefe: 1 · Zeilen (ohne Leerzeilen): 325.555
- Legende: **prod** = src/bin/build ohne `cfg(test)`; **dev** = tests/benches/examples/fuzz und `cfg(test)`-Code.
- Hinweis: Textuelle Analyse ohne Compiler. Glob-Importe (`*`) sind nicht auflösbar. Siehe Kopf des Skripts für Grenzen.

## 1. Überblick

| Crate | Ring | Zeilen | Fan-out prod | Fan-in prod | Cargo-Deps intern (n/d/b) | Pub-Items | API-Lecks | Zyklus |
|---|---|---:|---:|---:|---|---:|---:|---|
| [contextra](#contextra) | 4 | 2884 | 11 | 1 | 15/0/0 | 18 | 15 | – |
| [contextra-adapt](#contextra-adapt) | 0 | 6285 | 2 | 5 | 2/0/0 | 56 | 3 | – |
| [contextra-agent](#contextra-agent) | 3 | 8564 | 5 | 0 | 7/2/0 | 47 | 8 | – |
| [contextra-audit-export](#contextra-audit-export) | 0 | 856 | 1 | 0 | 2/0/0 | 14 | 2 | – |
| [contextra-avv-generator](#contextra-avv-generator) | 0 | 591 | 1 | 0 | 1/0/0 | 7 | 1 | – |
| [contextra-bench](#contextra-bench) | tooling | 7192 | 3 | 1 | 12/0/0 | 49 | 4 | – |
| [contextra-checkpoint](#contextra-checkpoint) | 1 | 7429 | 2 | 2 | 3/2/0 | 1 | 0 | – |
| [contextra-cognition](#contextra-cognition) | 3 | 8032 | 5 | 1 | 6/1/0 | 61 | 25 | – |
| [contextra-core](#contextra-core) | 0 | 322 | 4 | 7 | 4/0/0 | 0 | 14 | – |
| [contextra-crypto](#contextra-crypto) | 0 | 15950 | 2 | 6 | 2/1/0 | 73 | 4 | – |
| [contextra-db](#contextra-db) | 3 | 12443 | 8 | 4 | 13/1/0 | 13 | 25 | – |
| [contextra-engine](#contextra-engine) | 3 | 24027 | 14 | 2 | 15/4/0 | 81 | 29 | – |
| [contextra-graph](#contextra-graph) | 0 | 25811 | 3 | 3 | 3/2/0 | 125 | 23 | – |
| [contextra-infer-candle](#contextra-infer-candle) | 2 | 6657 | 4 | 4 | 4/1/0 | 33 | 6 | – |
| [contextra-infer-ollama](#contextra-infer-ollama) | 2 | 4906 | 3 | 2 | 3/1/0 | 30 | 5 | – |
| [contextra-infer-onnx](#contextra-infer-onnx) | 2 | 2004 | 2 | 3 | 4/0/0 | 8 | 4 | – |
| [contextra-kvcache](#contextra-kvcache) | 1 | 6435 | 3 | 1 | 3/1/0 | 40 | 3 | – |
| [contextra-license](#contextra-license) | 4 | 1415 | 2 | 2 | 2/0/0 | 4 | 1 | – |
| [contextra-mcp](#contextra-mcp) | 4 | 9821 | 11 | 0 | 13/0/0 | 36 | 14 | – |
| [contextra-mvcc](#contextra-mvcc) | 0 | 4572 | 2 | 3 | 2/0/0 | 22 | 1 | – |
| [contextra-ports](#contextra-ports) | 0 | 3602 | 1 | 21 | 1/0/0 | 65 | 2 | – |
| [contextra-privacy](#contextra-privacy) | 3 | 4634 | 2 | 2 | 2/1/0 | 74 | 7 | – |
| [contextra-py](#contextra-py) | 4 | 2196 | 3 | 0 | 7/0/0 | 36 | 3 | – |
| [contextra-rank](#contextra-rank) | 0 | 6069 | 1 | 6 | 2/1/0 | 21 | 2 | – |
| [contextra-router](#contextra-router) | 3 | 8045 | 3 | 3 | 5/4/0 | 23 | 4 | – |
| [contextra-sandbox](#contextra-sandbox) | 2 | 2956 | 0 | 0 | 0/0/0 | 14 | 0 | – |
| [contextra-simd](#contextra-simd) | 0 | 1472 | 1 | 1 | 1/0/0 | 53 | 13 | – |
| [contextra-store](#contextra-store) | 1 | 46092 | 5 | 3 | 6/4/0 | 105 | 6 | – |
| [contextra-sys](#contextra-sys) | 0 | 632 | 0 | 3 | 0/0/0 | 14 | 0 | – |
| [contextra-testkit](#contextra-testkit) | tooling | 916 | 1 | 0 | 2/0/0 | 6 | 0 | – |
| [contextra-text](#contextra-text) | 0 | 10725 | 1 | 1 | 2/0/0 | 42 | 3 | – |
| [contextra-types](#contextra-types) | 0 | 6761 | 0 | 25 | 0/0/0 | 65 | 0 | – |
| [contextra-vector](#contextra-vector) | 0 | 21641 | 6 | 2 | 6/0/0 | 29 | 0 | – |
| [contextra-wire](#contextra-wire) | 0 | 1892 | 0 | 1 | 0/0/0 | 44 | 0 | – |
| [xtask](#xtask) | tooling | 51482 | 1 | 0 | 1/0/0 | 549 | 2 | – |
| [xtask-heavy](#xtask-heavy) | tooling | 244 | 2 | 0 | 2/0/0 | 0 | 0 | – |

## 2. Workspace-weite Auffälligkeiten

### 2.1 Zyklen

- Kein Zyklus im prod-Code-Graphen gefunden.
- Kein Zyklus in den normalen Cargo-Abhängigkeiten.

### 2.2 Abweichungen Cargo.toml ↔ Code

| Crate | Ziel | Deklariert als | Befund |
|---|---|---|---|
| `contextra` | `contextra-audit-export` | normal? | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra` | `contextra-avv-generator` | normal? | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra` | `contextra-crypto` | normal? | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra` | `contextra-privacy` | normal? | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-agent` | `contextra-checkpoint` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-agent` | `contextra-graph` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-agent` | `contextra-vector` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-audit-export` | `contextra-ports` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-crypto` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-bench` | `contextra-graph` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-infer-onnx` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-kvcache` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-ports` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-store` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-bench` | `contextra-text` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-types` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-vector` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-checkpoint` | `contextra-testkit` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-checkpoint` | `contextra-types` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-cognition` | `contextra-store` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-crypto` | `contextra-crypto` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-db` | `contextra-crypto` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-db` | `contextra-rank` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-db` | `contextra-store` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-db` | `contextra-text` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-db` | `contextra-vector` | dev/normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-engine` | `contextra-core` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-engine` | `contextra-sandbox` | normal? | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-engine` | `contextra-sys` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-engine` | `contextra-infer-onnx` | – | genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler) |
| `contextra-graph` | `contextra-testkit` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-infer-ollama` | `contextra-infer-onnx` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-infer-onnx` | `contextra-infer-candle` | normal? | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-infer-onnx` | `contextra-ports` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-kvcache` | `contextra-kvcache` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-mcp` | `contextra-agent` | normal? | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-mcp` | `contextra-wire` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-py` | `contextra-adapt` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-py` | `contextra-rank` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-py` | `contextra-router` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-py` | `contextra-store` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-rank` | `contextra-ports` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-router` | `contextra-ports` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-router` | `contextra-wire` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-store` | `contextra-graph` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-store` | `contextra-types` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-store` | `contextra-vector` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-testkit` | `contextra-types` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-text` | `contextra-types` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |

### 2.3 Schwache Kanten (billig zu trennen)

Kanten im prod-Code mit höchstens 3 verschiedenen Symbolen. Reihenfolge: wenigste Symbole, dann wenigste Fundstellen.

| Von | Nach | Symbole | Fundstellen | Dateien | Symbole (prod) |
|---|---|---:|---:|---:|---|
| `contextra-core` | `contextra-wire` | 1 | 1 | 1 | `*` |
| `contextra-vector` | `contextra-types` | 1 | 1 | 1 | `SaturatingU16::from` |
| `contextra-db` | `contextra-checkpoint` | 1 | 1 | 1 | `(crate)` |
| `contextra-db` | `contextra-graph` | 1 | 1 | 1 | `percolation::PercolationConfig` |
| `contextra-db` | `contextra-ports` | 1 | 1 | 1 | `QueryRewriter` |
| `contextra-db` | `contextra-sys` | 1 | 1 | 1 | `LockedRegions` |
| `contextra-graph` | `contextra-adapt` | 1 | 1 | 1 | `TracingShadowSink` |
| `contextra-mcp` | `contextra-infer-onnx` | 1 | 1 | 1 | `OnnxEmbedder::from_path` |
| `contextra-mcp` | `contextra-license` | 1 | 1 | 1 | `OpenFastGate` |
| `contextra-infer-ollama` | `contextra-rank` | 1 | 1 | 1 | `IsotonicCalibrator` |
| `contextra-router` | `contextra-types` | 1 | 1 | 1 | `RetrievalStrategy` |
| `contextra-bench` | `contextra-db` | 1 | 1 | 1 | `Collection` |
| `contextra-infer-candle` | `contextra-rank` | 1 | 1 | 1 | `IsotonicCalibrator` |
| `contextra-infer-candle` | `contextra-crypto` | 1 | 1 | 1 | `EncryptedKvLayer` |
| `contextra-py` | `contextra-types` | 1 | 1 | 1 | `TenantId::SYSTEM` |
| `contextra-testkit` | `contextra-ports` | 1 | 1 | 1 | `Clock` |
| `contextra` | `contextra-router` | 1 | 1 | 1 | `(crate)` |
| `contextra` | `contextra-rank` | 1 | 1 | 1 | `(crate)` |
| `contextra` | `contextra-infer-ollama` | 1 | 1 | 1 | `(crate)` |
| `contextra` | `contextra-infer-candle` | 1 | 1 | 1 | `(crate)` |
| `contextra` | `contextra-infer-onnx` | 1 | 1 | 1 | `(crate)` |
| `contextra` | `contextra-db` | 1 | 1 | 1 | `QueryRewriter` |
| `contextra` | `contextra-types` | 1 | 1 | 1 | `ScoredEntry` |
| `contextra-mvcc` | `contextra-types` | 1 | 1 | 1 | `*` |
| `contextra-engine` | `contextra-mvcc` | 1 | 1 | 1 | `tx_buffer::TxBuffer` |
| `contextra-engine` | `contextra-infer-candle` | 1 | 1 | 1 | `CandleAttentionExporter` |
| `contextra-avv-generator` | `contextra-types` | 1 | 1 | 1 | `TenantId` |
| `contextra-license` | `contextra-types` | 1 | 1 | 1 | `TenantId` |
| `xtask` | `contextra-types` | 1 | 1 | 1 | `ContextraError` |
| `xtask-heavy` | `contextra-bench` | 1 | 1 | 1 | `regression_gate` |
| `contextra-core` | `contextra-ports` | 1 | 2 | 1 | `*`×2 |
| `contextra-vector` | `contextra-sys` | 1 | 2 | 2 | `mmap_readonly`×2 |
| `contextra-checkpoint` | `contextra-core` | 1 | 2 | 2 | `SnapshotRegistry`×2 |
| `contextra-crypto` | `contextra-ports` | 1 | 2 | 2 | `Clock`×2 |
| `contextra-privacy` | `contextra-ports` | 1 | 2 | 2 | `Clock`×2 |
| `contextra-router` | `contextra-privacy` | 1 | 2 | 1 | `guarded_payload`×2 |
| `contextra-audit-export` | `contextra-types` | 1 | 2 | 2 | `TenantId`×2 |
| `contextra-agent` | `contextra-store` | 1 | 3 | 3 | `LsmStorage`×3 |
| `contextra-cognition` | `contextra-vector` | 1 | 3 | 3 | `HnswIndex`×3 |
| `contextra-infer-onnx` | `contextra-rank` | 1 | 4 | 3 | `PlattScaler`×4 |
| `contextra-text` | `contextra-ports` | 1 | 5 | 3 | `StorageEngine`×5 |
| `contextra-mvcc` | `contextra-ports` | 1 | 5 | 1 | `MetricsSink`×5 |
| `contextra-vector` | `contextra-crypto` | 1 | 6 | 2 | `wal_crypto::WalHmac::new`×6 |
| `contextra-db` | `contextra-types` | 2 | 2 | 2 | `DistanceMetric`, `MAX_SEARCH_K` |
| `contextra-mcp` | `contextra-adapt` | 2 | 2 | 1 | `PidController`, `PidController::default` |
| `contextra-bench` | `contextra-infer-candle` | 2 | 2 | 1 | `CandleEmbedClient::from_dir`, `model_registry::CandleQuantization::Q4KM` |
| `contextra` | `contextra-license` | 2 | 2 | 1 | `SignedLicenseGate`, `signed_gate::VerifyingKey::from_bytes` |
| `contextra` | `contextra-store` | 2 | 2 | 2 | `kv::delete_mode::KvDeleteMode`, `lsm::config` |
| `contextra-adapt` | `contextra-ports` | 2 | 2 | 1 | `SeededRng::new`, `Rng::next_u64` |
| `contextra-engine` | `contextra-infer-onnx` | 2 | 2 | 1 | `ensure_onnx_model_download`, `OnnxEmbedder::load` |
| `xtask-heavy` | `contextra-router` | 2 | 2 | 1 | `BanditProfileState`, `BanditImplementation::DiagonalApproximation` |
| `contextra-store` | `contextra-sys` | 2 | 3 | 2 | `mmap_readonly`×2, `set_restrictive_file_acl` |
| `contextra-vector` | `contextra-simd` | 2 | 3 | 3 | `validate_vector`×2, `kernels::scalar` |
| `contextra-engine` | `contextra-text` | 2 | 3 | 2 | `Language`×2, `inverted::InvertedIndex` |
| `contextra-license` | `contextra-ports` | 2 | 3 | 2 | `license`×2, `clock` |
| `contextra-simd` | `contextra-core` | 2 | 5 | 2 | `Result`×4, `ContextraError` |
| `contextra-engine` | `contextra-adapt` | 2 | 5 | 4 | `PidController`×4, `decay_controller` |
| `contextra-engine` | `contextra-crypto` | 2 | 6 | 2 | `TenantIsolatedKvStore`×5, `deletion_proof` |
| `contextra-mcp` | `contextra-infer-ollama` | 3 | 3 | 1 | `OllamaEmbedder::new`, `OllamaConfig`, `OllamaClient::with_config` |
| `contextra-mcp` | `contextra-rank` | 3 | 3 | 2 | `fusion::ProvenanceRecord`, `IsotonicCalibrator`, `IsotonicCalibrator::with_defaults` |

… 8 weitere Kanten (siehe `--json`).

**Nur in Tests/Benches/Beispielen** (für das Produkt irrelevant, aber sie koppeln den Test-Build):

| Von | Nach | Fundstellen | Symbole |
|---|---|---:|---|
| `contextra-agent` | `contextra-checkpoint` | 1 | `StateCheckpoint` |
| `contextra-agent` | `contextra-graph` | 2 | `CsrGraph`×2 |
| `contextra-agent` | `contextra-text` | 5 | `Language::English`×5 |
| `contextra-bench` | `contextra-crypto` | 2 | `kv_cipher::KvSegmentCipher`, `KeyManager` |
| `contextra-bench` | `contextra-store` | 1 | `lsm` |
| `contextra-checkpoint` | `contextra-store` | 1 | `lsm` |
| `contextra-checkpoint` | `contextra-types` | 12 | `TxId`×5, `Result`×3, `ContextraError`×2, `ContextraError::Internal`×2 |
| `contextra-cognition` | `contextra-infer-candle` | 1 | `gasp` |
| `contextra-cognition` | `contextra-store` | 16 | `LsmStorage`×8, `LsmConfig`×8 |
| `contextra-db` | `contextra-crypto` | 4 | `CryptoError`, `deletion_proof`, `deletion_proof::GraphRepairAttestation`, `deletion_proof::DeletionProof` |
| `contextra-db` | `contextra-rank` | 1 | `IsotonicCalibrator::new` |
| `contextra-db` | `contextra-store` | 12 | `LsmConfig`×5, `LsmStorage::new`×4, `LsmStorage`×2, `lsm` |
| `contextra-db` | `contextra-text` | 11 | `Language::English`×7, `Language`×2, `bm25::score_term`, `inverted::InvertedIndex` |
| `contextra-db` | `contextra-vector` | 6 | `HnswConfig`×3, `HnswIndex`×2, `HnswIndex::try_new` |
| `contextra-engine` | `contextra-testkit` | 1 | `InMemoryStorageEngine` |
| `contextra-graph` | `contextra-store` | 2 | `LsmConfig`, `LsmStorage::new` |
| `contextra-infer-onnx` | `contextra-ports` | 1 | `EmbeddingProvider` |
| `contextra-privacy` | `contextra-crypto` | 2 | `crypto::KeyManager`, `kv_cipher::ModelFingerprint` |
| `contextra-rank` | `contextra-adapt` | 2 | `PidController`×2 |
| `contextra-router` | `contextra-db` | 13 | `ContextraConfig`×5, `Contextra::open_with_config`×5, `Collection`, `context::ContextManager::new` … (+1) |
| `contextra-router` | `contextra-graph` | 1 | `CsrGraph::new` |
| `contextra-router` | `contextra-ports` | 6 | `StorageEngine`×3, `HybridSearchProvider`, `CommunityResolver`, `ContextPreparer` |
| `contextra-router` | `contextra-text` | 1 | `Language::English` |
| `contextra-router` | `contextra-vector` | 2 | `HnswConfig::default`, `HnswIndex::try_new` |
| `contextra-store` | `contextra-sandbox` | 1 | `MergeOperatorCapabilities` |
| `contextra-store` | `contextra-testkit` | 7 | `ReferenceModel`×4, `ManualClock`×3 |
| `contextra-store` | `contextra-types` | 2 | `TxId`, `TenantId` |
| `contextra-text` | `contextra-types` | 132 | `Result`×67, `TOMBSTONE_BIT`×52, `TxId`×7, `TxId::new`×2 … (+3) |

### 2.4 Hub-Symbole (Kandidaten für types/ports)

Symbole, die aus einem Nicht-Basis-Crate von mindestens 3 verschiedenen Crates im prod-Code importiert werden. Wer sie in ein schmales Basis-Crate verschiebt, kann viele Kanten kappen.

| Quelle | Symbol | Nutzer | Crates |
|---|---|---:|---|
| `contextra-core` | `Result` | 4 | `contextra-bench`, `contextra-simd`, `contextra-store`, `contextra-vector` |
| `contextra-rank` | `IsotonicCalibrator` | 4 | `contextra-engine`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-mcp` |
| `contextra-adapt` | `PidController` | 3 | `contextra-db`, `contextra-engine`, `contextra-mcp` |
| `contextra-core` | `ContextraError` | 3 | `contextra-py`, `contextra-simd`, `contextra-store` |
| `contextra-db` | `Collection` | 3 | `contextra-agent`, `contextra-bench`, `contextra-py` |
| `contextra-graph` | `percolation::PercolationConfig` | 3 | `contextra-cognition`, `contextra-db`, `contextra-engine` |

### 2.5 API-Lecks

Öffentliche Items, deren Signatur oder öffentliche Felder Typen anderer Workspace-Crates enthalten, sowie `pub use` fremder Crates. Diese Stellen machen eine Entkopplung am schwersten, weil Nutzer des Crates die fremde Abhängigkeit mitsehen.

| Crate | Komponente | Item | Fremde Crates | Fundstelle |
|---|---|---|---|---|
| `contextra` | `collection_profile` | `const DEFAULT_AUTO_EXTRACTION_MODE` | `contextra-core` | `src/collection_profile.rs:11` |
| `contextra` | `collection_profile` | `const ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT` | `contextra-core` | `src/collection_profile.rs:16` |
| `contextra` | `collection_profile` | `struct CollectionProfile` | `contextra-core`, `contextra-store` | `src/collection_profile.rs:27` |
| `contextra` | `collection_profile` | `pub use contextra_core::types::domain::AutoExtractionMode` | `contextra-core` | `src/collection_profile.rs:6` |
| `contextra` | `(crate-root)` | `fn open` | `contextra-core` | `src/lib.rs:46` |
| `contextra` | `(crate-root)` | `fn open_with_config` | `contextra-core` | `src/lib.rs:51` |
| `contextra` | `(crate-root)` | `pub use contextra_core::error::ContextraError` | `contextra-core` | `src/lib.rs:15` |
| `contextra` | `(crate-root)` | `pub use contextra_core::DistanceMetric` | `contextra-core` | `src/lib.rs:17` |
| `contextra` | `(crate-root)` | `pub use contextra_router` | `contextra-router` | `src/lib.rs:25` |
| `contextra` | `(crate-root)` | `pub use contextra_rank` | `contextra-rank` | `src/lib.rs:28` |
| `contextra` | `(crate-root)` | `pub use contextra_infer_ollama` | `contextra-infer-ollama` | `src/lib.rs:31` |
| `contextra` | `(crate-root)` | `pub use contextra_infer_candle` | `contextra-infer-candle` | `src/lib.rs:34` |
| `contextra` | `(crate-root)` | `pub use contextra_infer_onnx` | `contextra-infer-onnx` | `src/lib.rs:37` |
| `contextra` | `performance_profile` | `struct ResolvedProfileConfig` | `contextra-ports` | `src/performance_profile.rs:23` |
| `contextra` | `performance_profile` | `pub use contextra_ports::license::VectorDeleteMode` | `contextra-ports` | `src/performance_profile.rs:5` |
| `contextra-adapt` | `flow_thompson` | `struct DiagonalApproximationBandit` | `contextra-types` | `src/flow_thompson.rs:633` |
| `contextra-adapt` | `flow_thompson` | `enum OffPolicyError` | `contextra-types` | `src/flow_thompson.rs:675` |
| `contextra-adapt` | `flow_thompson` | `struct FcTsSamplingDistribution` | `contextra-types` | `src/flow_thompson.rs:689` |
| `contextra-agent` | `audit` | `struct AuditEntry` | `contextra-types` | `src/audit.rs:24` |
| `contextra-agent` | `audit` | `struct AuditLog` | `contextra-ports`, `contextra-store` | `src/audit.rs:45` |
| `contextra-agent` | `audit` | `fn validate_audit_payload_and_error` | `contextra-types` | `src/audit.rs:50` |
| `contextra-agent` | `audit` | `fn migrate_legacy_audit_entries` | `contextra-db`, `contextra-ports`, `contextra-types` | `src/audit.rs:145` |
| `contextra-agent` | `budget` | `fn reserve_tokens` | `contextra-types` | `src/budget.rs:17` |
| `contextra-agent` | `context` | `fn validate_task_id` | `contextra-types` | `src/context.rs:27` |
| `contextra-agent` | `context` | `fn validate_node_id` | `contextra-types` | `src/context.rs:49` |
| `contextra-agent` | `context` | `struct AgentContext` | `contextra-db`, `contextra-router`, `contextra-store` | `src/context.rs:80` |
| `contextra-audit-export` | `(crate-root)` | `struct ProcessingRegisterEntry` | `contextra-types` | `src/lib.rs:50` |
| `contextra-audit-export` | `(crate-root)` | `struct Art30ProcessingRegisterEntry` | `contextra-types` | `src/lib.rs:69` |
| `contextra-avv-generator` | `(crate-root)` | `struct AvvContext` | `contextra-types` | `src/lib.rs:17` |
| `contextra-bench` | `locomo` | `fn load_locomo_dataset` | `contextra-core` | `src/locomo.rs:144` |
| `contextra-bench` | `locomo` | `fn run_locomo_eval` | `contextra-core` | `src/locomo.rs:240` |
| `contextra-bench` | `path_rag_sweep` | `fn run_pathrag_sweep_long_mem_eval` | `contextra-core` | `src/path_rag_sweep.rs:39` |
| `contextra-bench` | `path_rag_sweep` | `fn run_pathrag_sweep_locomo` | `contextra-core` | `src/path_rag_sweep.rs:198` |
| `contextra-cognition` | `aggregation_phase` | `struct AggregationEdge` | `contextra-graph` | `src/aggregation_phase.rs:141` |
| `contextra-cognition` | `aggregation_phase` | `struct SuperEdgeDraft` | `contextra-graph` | `src/aggregation_phase.rs:152` |
| `contextra-cognition` | `aggregation_phase` | `fn run_aggregation_pass` | `contextra-ports` | `src/aggregation_phase.rs:238` |
| `contextra-cognition` | `consolidation_executor` | `fn execute_consolidation_pass` | `contextra-engine`, `contextra-types` | `src/consolidation_executor.rs:63` |
| `contextra-cognition` | `consolidation_executor` | `fn execute_background_consolidation` | `contextra-engine`, `contextra-types` | `src/consolidation_executor.rs:219` |
| `contextra-cognition` | `consolidation_executor` | `fn execute_background_consolidation_with_rich_validator` | `contextra-engine`, `contextra-types` | `src/consolidation_executor.rs:247` |
| `contextra-cognition` | `consolidation_executor` | `fn execute_leanrag_aggregation_stage` | `contextra-engine`, `contextra-types` | `src/consolidation_executor.rs:387` |
| `contextra-cognition` | `consolidation_executor` | `fn execute_sleep_cycle` | `contextra-engine`, `contextra-types` | `src/consolidation_executor.rs:402` |
| `contextra-cognition` | `consolidation_executor` | `fn start_consolidation_worker` | `contextra-engine` | `src/consolidation_executor.rs:424` |
| `contextra-cognition` | `consolidation_executor` | `fn start_consolidation_worker_full` | `contextra-engine` | `src/consolidation_executor.rs:441` |
| `contextra-cognition` | `consolidation_executor` | `fn start_consolidation_reaper` | `contextra-engine` | `src/consolidation_executor.rs:464` |
| `contextra-cognition` | `consolidation_executor` | `struct ConsolidationEngine` | `contextra-vector` | `src/consolidation_executor.rs:483` |
| `contextra-cognition` | `context_compaction::cleanup` | `fn cleanup_orphaned_consolidation_intents` | `contextra-ports` | `src/context_compaction/cleanup.rs:10` |
| `contextra-cognition` | `context_compaction::session` | `struct ConsolidationSession` | `contextra-vector` | `src/context_compaction/session.rs:14` |
| `contextra-cognition` | `leanrag_input` | `fn build_leanrag_inputs` | `contextra-engine` | `src/leanrag_input.rs:42` |
| `contextra-cognition` | `maintenance_config` | `struct PercolationConfig` | `contextra-graph` | `src/maintenance_config.rs:11` |
| `contextra-cognition` | `maintenance_config` | `struct MaintenanceConfig` | `contextra-engine`, `contextra-graph` | `src/maintenance_config.rs:31` |
| `contextra-cognition` | `maintenance_scheduler` | `struct MaintenanceScheduler` | `contextra-vector` | `src/maintenance_scheduler.rs:22` |
| `contextra-cognition` | `memory_consolidation` | `fn run_structural_synthesis_pass` | `contextra-types` | `src/memory_consolidation.rs:409` |
| `contextra-cognition` | `memory_consolidation` | `struct PreCondensedInput` | `contextra-types` | `src/memory_consolidation.rs:548` |
| `contextra-cognition` | `memory_consolidation` | `fn intake_pre_condensed_inputs` | `contextra-types` | `src/memory_consolidation.rs:565` |
| `contextra-cognition` | `semantic_aggregation_facade` | `fn consolidate_semantic_hyperedges` | `contextra-engine` | `src/semantic_aggregation_facade.rs:40` |
| `contextra-cognition` | `synthesis_phase` | `struct SynthesizedChunk` | `contextra-types` | `src/synthesis_phase.rs:24` |
| `contextra-cognition` | `synthesis_phase` | `fn run_synthesis_pass` | `contextra-ports` | `src/synthesis_phase.rs:44` |
| `contextra-cognition` | `synthesis_phase` | `pub use contextra_ports::SegmentSynthesizer` | `contextra-ports` | `src/synthesis_phase.rs:35` |
| `contextra-core` | `(crate-root)` | `pub use contextra_types::error` | `contextra-types` | `src/lib.rs:15` |
| `contextra-core` | `(crate-root)` | `pub use contextra_types::error_dto` | `contextra-types` | `src/lib.rs:24` |
| `contextra-core` | `(crate-root)` | `pub use contextra_types::model_fingerprint` | `contextra-types` | `src/lib.rs:29` |
| `contextra-core` | `(crate-root)` | `pub use contextra_types::schema` | `contextra-types` | `src/lib.rs:34` |
| `contextra-core` | `(crate-root)` | `pub use contextra_types::tombstone` | `contextra-types` | `src/lib.rs:39` |
| `contextra-core` | `(crate-root)` | `pub use contextra_types::types` | `contextra-types` | `src/lib.rs:44` |
| `contextra-core` | `(crate-root)` | `pub use contextra_ports::*` | `contextra-ports` | `src/lib.rs:56` |
| `contextra-core` | `(crate-root)` | `pub use contextra_mvcc::seq_log` | `contextra-mvcc` | `src/lib.rs:59` |
| `contextra-core` | `(crate-root)` | `pub use contextra_mvcc::snapshot` | `contextra-mvcc` | `src/lib.rs:64` |
| `contextra-core` | `(crate-root)` | `pub use contextra_mvcc::tx_buffer` | `contextra-mvcc` | `src/lib.rs:69` |
| `contextra-core` | `(crate-root)` | `pub use contextra_wire::*` | `contextra-wire` | `src/lib.rs:81` |
| `contextra-core` | `(crate-root)` | `pub use contextra_ports::*` | `contextra-ports` | `src/lib.rs:102` |
| `contextra-core` | `(crate-root)` | `pub use contextra_types::error_dto::ContextraErrorDto` | `contextra-types` | `src/lib.rs:114` |
| `contextra-core` | `(crate-root)` | `pub use contextra_types::model_fingerprint::ModelFingerprint` | `contextra-types` | `src/lib.rs:120` |
| `contextra-crypto` | `kv_cipher` | `struct EncryptedKvLayer` | `contextra-types` | `src/kv_cipher.rs:40` |
| `contextra-crypto` | `kv_cipher` | `pub use contextra_types::ModelFingerprint` | `contextra-types` | `src/kv_cipher.rs:30` |
| `contextra-crypto` | `kv_segment::segment` | `struct KvSegment` | `contextra-types` | `src/kv_segment/segment.rs:34` |
| `contextra-crypto` | `kv_segment::store` | `type SpillHandler` | `contextra-types` | `src/kv_segment/store.rs:17` |
| `contextra-db` | `homeostat` | `pub use contextra_adapt::homeostat::RerankDeadline` | `contextra-adapt` | `src/homeostat.rs:5` |
| `contextra-db` | `homeostat` | `pub use contextra_adapt::homeostat::RerankPidController` | `contextra-adapt` | `src/homeostat.rs:6` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition` | `contextra-cognition` | `src/lib.rs:5` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine` | `contextra-engine` | `src/lib.rs:8` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine::MetadataFilter` | `contextra-engine` | `src/lib.rs:11` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine::collection::maintenance::PercolationResult` | `contextra-engine` | `src/lib.rs:23` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine::SandboxBridge` | `contextra-engine` | `src/lib.rs:25` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::execute_sleep_cycle` | `contextra-cognition` | `src/lib.rs:29` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::context::*` | `contextra-cognition` | `src/lib.rs:43` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::context_compaction::*` | `contextra-cognition` | `src/lib.rs:46` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::memory_consolidation::*` | `contextra-cognition` | `src/lib.rs:49` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::synthesis_phase::*` | `contextra-cognition` | `src/lib.rs:52` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::consolidation_executor::*` | `contextra-cognition` | `src/lib.rs:55` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::consolidation_locks::*` | `contextra-cognition` | `src/lib.rs:58` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::maintenance_scheduler::*` | `contextra-cognition` | `src/lib.rs:61` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::maintenance_config::*` | `contextra-cognition` | `src/lib.rs:64` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine::background_workers::*` | `contextra-engine` | `src/lib.rs:69` |
| `contextra-db` | `(crate-root)` | `pub use contextra_cognition::start_consolidation_reaper` | `contextra-cognition` | `src/lib.rs:71` |
| `contextra-db` | `(crate-root)` | `pub use contextra_adapt::decay_controller::*` | `contextra-adapt` | `src/lib.rs:77` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine::fusion` | `contextra-engine` | `src/lib.rs:79` |
| `contextra-db` | `(crate-root)` | `pub use contextra_adapt::pid_latency_controller::*` | `contextra-adapt` | `src/lib.rs:83` |
| `contextra-db` | `(crate-root)` | `pub use contextra_checkpoint` | `contextra-checkpoint` | `src/lib.rs:105` |
| `contextra-db` | `(crate-root)` | `pub use contextra_graph::percolation::PercolationConfig` | `contextra-graph` | `src/lib.rs:106` |
| `contextra-db` | `(crate-root)` | `pub use contextra_types::DistanceMetric` | `contextra-types` | `src/lib.rs:110` |
| `contextra-db` | `multistep` | `pub use contextra_ports::QueryRewriter` | `contextra-ports` | `src/multistep.rs:9` |
| `contextra-engine` | `background_workers::expiry_workers` | `fn start_decay_cleanup_worker` | `contextra-ports` | `src/background_workers/expiry_workers.rs:67` |
| `contextra-engine` | `collection::crud::auto_extraction` | `pub use contextra_types::AutoExtractionMode` | `contextra-types` | `src/collection/crud/auto_extraction.rs:11` |
| `contextra-engine` | `collection` | `fn compute_default_importance` | `contextra-types` | `src/collection/mod.rs:115` |
| `contextra-engine` | `collection` | `struct Collection` | `contextra-store`, `contextra-vector` | `src/collection/mod.rs:288` |
| `contextra-engine` | `collection::query_builder::scope` | `struct ScopeConstraint` | `contextra-types` | `src/collection/query_builder/scope.rs:12` |
| `contextra-engine` | `collection::query_builder::strategy` | `enum SearchStrategy` | `contextra-types` | `src/collection/query_builder/strategy.rs:4` |
| `contextra-engine` | `extraction::open_ie` | `fn extract_triples` | `contextra-ports` | `src/extraction/open_ie.rs:73` |
| `contextra-engine` | `fusion` | `fn fuse_search_results_with_strategy` | `contextra-types` | `src/fusion.rs:16` |
| `contextra-engine` | `fusion` | `fn fuse_search_results_with_signal_strategies` | `contextra-types` | `src/fusion.rs:35` |
| `contextra-engine` | `(crate-root)` | `type ConsolidationLauncher` | `contextra-store` | `src/lib.rs:408` |
| `contextra-engine` | `(crate-root)` | `fn register_consolidation_launcher` | `contextra-store` | `src/lib.rs:422` |
| `contextra-engine` | `(crate-root)` | `struct ContextraStats` | `contextra-ports` | `src/lib.rs:453` |
| `contextra-engine` | `(crate-root)` | `struct ContextraConfig` | `contextra-ports`, `contextra-store`, `contextra-types` | `src/lib.rs:511` |
| `contextra-engine` | `(crate-root)` | `type TenantCollectionMap` | `contextra-store` | `src/lib.rs:642` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_ports::TextEmbeddingEngine` | `contextra-ports` | `src/lib.rs:328` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_ports::VectorDeleteMode` | `contextra-ports` | `src/lib.rs:329` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_store::lsm::DurabilityMode` | `contextra-store` | `src/lib.rs:330` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_checkpoint` | `contextra-checkpoint` | `src/lib.rs:400` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_graph::percolation::PercolationConfig` | `contextra-graph` | `src/lib.rs:401` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_text::Language` | `contextra-text` | `src/lib.rs:403` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_ports::DriftStatusProvider` | `contextra-ports` | `src/lib.rs:467` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_types::DistanceMetric` | `contextra-types` | `src/lib.rs:678` |
| `contextra-engine` | `metrics_names` | `fn record_histogram_safe` | `contextra-ports` | `src/metrics_names.rs:27` |
| `contextra-engine` | `metrics_names` | `fn record_gauge_safe` | `contextra-ports` | `src/metrics_names.rs:39` |
| `contextra-engine` | `metrics_names` | `fn record_counter_safe` | `contextra-ports` | `src/metrics_names.rs:51` |
| `contextra-engine` | `temporal_filter` | `struct ValidityWindow` | `contextra-types` | `src/temporal_filter.rs:18` |
| `contextra-engine` | `temporal_filter` | `fn apply_temporal_validity_filter` | `contextra-types` | `src/temporal_filter.rs:159` |
| `contextra-engine` | `transaction::cleanup` | `fn cleanup_orphaned_consolidation_intents` | `contextra-ports` | `src/transaction/cleanup.rs:7` |
| `contextra-engine` | `transaction::db_transaction` | `struct DbTransaction` | `contextra-vector` | `src/transaction/db_transaction.rs:11` |
| `contextra-graph` | `apprh::diffusion` | `fn run_apprh_push` | `contextra-types` | `src/apprh/diffusion.rs:22` |
| `contextra-graph` | `apprh` | `fn apprh_local` | `contextra-types` | `src/apprh/mod.rs:34` |
| `contextra-graph` | `apprh` | `fn forward_push_apprh` | `contextra-types` | `src/apprh/mod.rs:49` |
| `contextra-graph` | `apprh::shadow` | `struct ApprhShadowComparison` | `contextra-types` | `src/apprh/shadow.rs:28` |
| `contextra-graph` | `apprh::shadow` | `fn shadow_compare_forward_push_vs_apprh` | `contextra-types` | `src/apprh/shadow.rs:50` |
| `contextra-graph` | `edge_reinforcement_buffer` | `struct CooccurrenceSignal` | `contextra-types` | `src/edge_reinforcement_buffer.rs:26` |
| `contextra-graph` | `edge_reinforcement_buffer` | `struct TraversalSignal` | `contextra-types` | `src/edge_reinforcement_buffer.rs:35` |
| `contextra-graph` | `hyperedge_suggest` | `fn validate_candidates_with_llm` | `contextra-ports` | `src/hyperedge_suggest.rs:113` |
| `contextra-graph` | `(crate-root)` | `trait GraphIndexExt` | `contextra-ports` | `src/lib.rs:121` |
| `contextra-graph` | `path_rag::k_path` | `struct KPathResult` | `contextra-types` | `src/path_rag/k_path.rs:34` |
| `contextra-graph` | `path_rag` | `struct GraphPath` | `contextra-types` | `src/path_rag/mod.rs:31` |
| `contextra-graph` | `path_rag` | `fn forward_push_ppr` | `contextra-types` | `src/path_rag/mod.rs:92` |
| `contextra-graph` | `path_rag` | `pub use contextra_types::EntityId` | `contextra-types` | `src/path_rag/mod.rs:25` |
| `contextra-graph` | `percolation` | `fn find_rebonding_candidates` | `contextra-ports`, `contextra-types` | `src/percolation.rs:71` |
| `contextra-graph` | `ppr::shadow_hook` | `fn log_tl_hfd_vs_baseline_discrepancy` | `contextra-types` | `src/ppr/shadow_hook.rs:9` |
| `contextra-graph` | `ppr::shadow_hook` | `fn log_tl_hfd_discrepancy_tracing` | `contextra-types` | `src/ppr/shadow_hook.rs:49` |
| `contextra-graph` | `ppr_stream` | `struct PprCandidateStream` | `contextra-ports` | `src/ppr_stream.rs:21` |
| `contextra-graph` | `tl_hfd::diffusion` | `fn run_diffusion` | `contextra-types` | `src/tl_hfd/diffusion.rs:27` |
| `contextra-graph` | `tl_hfd::lovasz` | `fn truncate_participants` | `contextra-types` | `src/tl_hfd/lovasz.rs:13` |
| `contextra-graph` | `tl_hfd::lovasz` | `fn compute_lovasz_extension` | `contextra-types` | `src/tl_hfd/lovasz.rs:54` |

… 79 weitere (siehe `--json`).

## 3. Kantenliste

| Von | Nach | prod Fundstellen | prod Symbole | dev Fundstellen | Cargo-Art |
|---|---|---:|---:|---:|---|
| `contextra-engine` | `contextra-types` | 252 | 52 | 172 | normal |
| `contextra-vector` | `contextra-core` | 60 | 11 | 65 | normal |
| `contextra-checkpoint` | `contextra-ports` | 47 | 7 | 9 | normal |
| `contextra-graph` | `contextra-types` | 39 | 12 | 53 | normal |
| `contextra-store` | `contextra-core` | 35 | 14 | 115 | normal |
| `contextra-engine` | `contextra-ports` | 33 | 16 | 21 | normal |
| `contextra-adapt` | `contextra-types` | 32 | 9 | 2 | normal |
| `contextra-privacy` | `contextra-types` | 31 | 4 | 1 | normal |
| `contextra-engine` | `contextra-store` | 30 | 11 | 106 | normal |
| `contextra-crypto` | `contextra-types` | 25 | 9 | 14 | normal |
| `contextra-agent` | `contextra-types` | 24 | 9 | 75 | normal |
| `contextra-infer-candle` | `contextra-ports` | 22 | 11 | 29 | normal |
| `contextra-engine` | `contextra-graph` | 20 | 16 | 76 | normal |
| `contextra-graph` | `contextra-ports` | 20 | 15 | 40 | normal |
| `contextra-mcp` | `contextra` | 20 | 14 | 15 | normal |
| `contextra-mcp` | `contextra-ports` | 19 | 10 | 51 | normal |
| `contextra-cognition` | `contextra-types` | 15 | 8 | 30 | normal |
| `contextra-agent` | `contextra-ports` | 14 | 5 | 18 | normal |
| `contextra-store` | `contextra-ports` | 14 | 6 | 10 | normal |
| `contextra-cognition` | `contextra-engine` | 13 | 6 | 25 | normal |
| `contextra-cognition` | `contextra-graph` | 13 | 11 | 20 | normal |
| `contextra-py` | `contextra-core` | 13 | 8 | 1 | normal |
| `contextra-vector` | `contextra-ports` | 13 | 4 | 10 | normal |
| `contextra-core` | `contextra-types` | 11 | 8 | 0 | normal |
| `contextra-db` | `contextra-cognition` | 11 | 11 | 0 | normal |
| `contextra-kvcache` | `contextra-crypto` | 11 | 5 | 5 | normal |
| `contextra-rank` | `contextra-types` | 10 | 8 | 9 | normal |
| `contextra-bench` | `contextra-core` | 9 | 6 | 10 | normal |
| `contextra-mcp` | `contextra-types` | 9 | 6 | 4 | normal |
| `contextra-ports` | `contextra-types` | 9 | 7 | 0 | normal |
| `contextra` | `contextra-core` | 8 | 5 | 12 | normal |
| `contextra-infer-ollama` | `contextra-types` | 8 | 3 | 5 | normal |
| `contextra-mcp` | `contextra-crypto` | 8 | 4 | 7 | normal |
| `contextra-mcp` | `contextra-infer-candle` | 8 | 5 | 1 | normal? |
| `contextra-kvcache` | `contextra-types` | 7 | 4 | 16 | normal |
| `contextra-py` | `contextra-db` | 7 | 7 | 0 | normal |
| `contextra-core` | `contextra-mvcc` | 6 | 3 | 0 | normal |
| `contextra-db` | `contextra-engine` | 6 | 6 | 1 | normal |
| `contextra-engine` | `contextra-crypto` | 6 | 2 | 2 | dev/normal? |
| `contextra-engine` | `contextra-rank` | 6 | 5 | 0 | normal |
| `contextra-infer-candle` | `contextra-types` | 6 | 3 | 20 | normal |
| `contextra-infer-ollama` | `contextra-ports` | 6 | 5 | 6 | normal |
| `contextra-kvcache` | `contextra-ports` | 6 | 4 | 13 | normal |
| `contextra-mcp` | `contextra-privacy` | 6 | 6 | 3 | normal |
| `contextra-vector` | `contextra-crypto` | 6 | 1 | 2 | normal |
| `contextra` | `contextra-ports` | 5 | 4 | 8 | normal |
| `contextra-agent` | `contextra-db` | 5 | 3 | 42 | normal |
| `contextra-db` | `contextra-adapt` | 5 | 5 | 2 | normal |
| `contextra-engine` | `contextra-adapt` | 5 | 2 | 2 | normal |
| `contextra-engine` | `contextra-vector` | 5 | 3 | 119 | dev/normal |
| `contextra-mvcc` | `contextra-ports` | 5 | 1 | 2 | normal |
| `contextra-simd` | `contextra-core` | 5 | 2 | 0 | normal |
| `contextra-store` | `contextra-crypto` | 5 | 4 | 2 | normal |
| `contextra-store` | `contextra-mvcc` | 5 | 4 | 1 | normal |
| `contextra-text` | `contextra-ports` | 5 | 1 | 81 | normal |
| `contextra-agent` | `contextra-router` | 4 | 4 | 0 | normal |
| `contextra-cognition` | `contextra-ports` | 4 | 3 | 4 | normal |
| `contextra-engine` | `contextra-checkpoint` | 4 | 3 | 0 | normal |
| `contextra-infer-onnx` | `contextra-rank` | 4 | 1 | 1 | normal |
| `contextra-infer-onnx` | `contextra-types` | 4 | 4 | 3 | normal |
| `contextra-router` | `contextra-adapt` | 4 | 4 | 6 | normal |
| `contextra-agent` | `contextra-store` | 3 | 1 | 0 | normal |
| `contextra-cognition` | `contextra-vector` | 3 | 1 | 16 | normal |
| `contextra-engine` | `contextra-kvcache` | 3 | 3 | 2 | normal |
| `contextra-engine` | `contextra-text` | 3 | 2 | 69 | normal |
| `contextra-license` | `contextra-ports` | 3 | 2 | 5 | normal |
| `contextra-mcp` | `contextra-infer-ollama` | 3 | 3 | 0 | normal? |
| `contextra-mcp` | `contextra-rank` | 3 | 3 | 0 | normal |
| `contextra-store` | `contextra-sys` | 3 | 2 | 3 | normal |
| `contextra-vector` | `contextra-simd` | 3 | 2 | 7 | normal |
| `contextra` | `contextra-license` | 2 | 2 | 2 | normal |
| `contextra` | `contextra-store` | 2 | 2 | 3 | normal |
| `contextra-adapt` | `contextra-ports` | 2 | 2 | 0 | normal |
| `contextra-audit-export` | `contextra-types` | 2 | 1 | 5 | normal |
| `contextra-bench` | `contextra-infer-candle` | 2 | 2 | 0 | normal |
| `contextra-checkpoint` | `contextra-core` | 2 | 1 | 3 | normal |
| `contextra-core` | `contextra-ports` | 2 | 1 | 0 | normal |
| `contextra-crypto` | `contextra-ports` | 2 | 1 | 9 | normal |
| `contextra-db` | `contextra-types` | 2 | 2 | 47 | normal |
| `contextra-engine` | `contextra-infer-onnx` | 2 | 2 | 0 | – |
| `contextra-mcp` | `contextra-adapt` | 2 | 2 | 0 | normal |
| `contextra-privacy` | `contextra-ports` | 2 | 1 | 4 | normal |
| `contextra-router` | `contextra-privacy` | 2 | 1 | 0 | normal |
| `contextra-vector` | `contextra-sys` | 2 | 1 | 0 | normal |
| `xtask-heavy` | `contextra-router` | 2 | 2 | 0 | normal |
| `contextra` | `contextra-db` | 1 | 1 | 4 | normal |
| `contextra` | `contextra-infer-candle` | 1 | 1 | 0 | normal? |
| `contextra` | `contextra-infer-ollama` | 1 | 1 | 0 | normal? |
| `contextra` | `contextra-infer-onnx` | 1 | 1 | 0 | normal? |
| `contextra` | `contextra-rank` | 1 | 1 | 0 | normal? |
| `contextra` | `contextra-router` | 1 | 1 | 0 | normal? |
| `contextra` | `contextra-types` | 1 | 1 | 2 | normal |
| `contextra-avv-generator` | `contextra-types` | 1 | 1 | 4 | normal |
| `contextra-bench` | `contextra-db` | 1 | 1 | 0 | normal |
| `contextra-core` | `contextra-wire` | 1 | 1 | 0 | normal |
| `contextra-db` | `contextra-checkpoint` | 1 | 1 | 4 | normal |
| `contextra-db` | `contextra-graph` | 1 | 1 | 15 | normal |
| `contextra-db` | `contextra-ports` | 1 | 1 | 13 | normal |
| `contextra-db` | `contextra-sys` | 1 | 1 | 0 | normal |
| `contextra-engine` | `contextra-infer-candle` | 1 | 1 | 0 | normal |
| `contextra-engine` | `contextra-mvcc` | 1 | 1 | 3 | normal |
| `contextra-graph` | `contextra-adapt` | 1 | 1 | 0 | normal |
| `contextra-infer-candle` | `contextra-crypto` | 1 | 1 | 1 | dev/normal? |
| `contextra-infer-candle` | `contextra-rank` | 1 | 1 | 0 | normal |
| `contextra-infer-ollama` | `contextra-rank` | 1 | 1 | 0 | normal |
| `contextra-license` | `contextra-types` | 1 | 1 | 3 | normal |
| `contextra-mcp` | `contextra-infer-onnx` | 1 | 1 | 0 | normal? |
| `contextra-mcp` | `contextra-license` | 1 | 1 | 0 | normal |
| `contextra-mvcc` | `contextra-types` | 1 | 1 | 3 | normal |
| `contextra-py` | `contextra-types` | 1 | 1 | 0 | normal |
| `contextra-router` | `contextra-types` | 1 | 1 | 16 | normal |
| `contextra-testkit` | `contextra-ports` | 1 | 1 | 0 | normal |
| `contextra-vector` | `contextra-types` | 1 | 1 | 0 | normal |
| `xtask` | `contextra-types` | 1 | 1 | 0 | normal |
| `xtask-heavy` | `contextra-bench` | 1 | 1 | 0 | normal |
| `contextra-agent` | `contextra-checkpoint` | 0 | 0 | 1 | normal |
| `contextra-agent` | `contextra-graph` | 0 | 0 | 2 | normal |
| `contextra-agent` | `contextra-text` | 0 | 0 | 5 | dev |
| `contextra-bench` | `contextra-crypto` | 0 | 0 | 2 | normal |
| `contextra-bench` | `contextra-store` | 0 | 0 | 1 | normal |
| `contextra-checkpoint` | `contextra-store` | 0 | 0 | 1 | dev |
| `contextra-checkpoint` | `contextra-types` | 0 | 0 | 12 | normal |
| `contextra-cognition` | `contextra-infer-candle` | 0 | 0 | 1 | dev |
| `contextra-cognition` | `contextra-store` | 0 | 0 | 16 | normal |
| `contextra-db` | `contextra-crypto` | 0 | 0 | 4 | normal |
| `contextra-db` | `contextra-rank` | 0 | 0 | 1 | normal |
| `contextra-db` | `contextra-store` | 0 | 0 | 12 | normal |
| `contextra-db` | `contextra-text` | 0 | 0 | 11 | normal |
| `contextra-db` | `contextra-vector` | 0 | 0 | 6 | dev/normal |
| `contextra-engine` | `contextra-testkit` | 0 | 0 | 1 | dev |
| `contextra-graph` | `contextra-store` | 0 | 0 | 2 | dev |
| `contextra-infer-onnx` | `contextra-ports` | 0 | 0 | 1 | normal |
| `contextra-privacy` | `contextra-crypto` | 0 | 0 | 2 | dev |
| `contextra-rank` | `contextra-adapt` | 0 | 0 | 2 | dev |
| `contextra-router` | `contextra-db` | 0 | 0 | 13 | dev |
| `contextra-router` | `contextra-graph` | 0 | 0 | 1 | dev |
| `contextra-router` | `contextra-ports` | 0 | 0 | 6 | normal |
| `contextra-router` | `contextra-text` | 0 | 0 | 1 | dev |
| `contextra-router` | `contextra-vector` | 0 | 0 | 2 | dev |
| `contextra-store` | `contextra-sandbox` | 0 | 0 | 1 | dev |
| `contextra-store` | `contextra-testkit` | 0 | 0 | 7 | dev |
| `contextra-store` | `contextra-types` | 0 | 0 | 2 | normal |
| `contextra-text` | `contextra-types` | 0 | 0 | 132 | normal |

## 4. Crates im Detail

### contextra

- Pfad: `crates/contextra` · Zeilen: 2884 · Rust-Dateien: 22 · Ring: 4
- Interne Cargo-Abhängigkeiten: `contextra-audit-export` (normal/optional via audit-export), `contextra-avv-generator` (normal/optional via avv-generator), `contextra-core` (normal), `contextra-crypto` (normal/optional via sovereign), `contextra-db` (normal), `contextra-infer-candle` (normal/optional via candle), `contextra-infer-ollama` (normal/optional via ollama), `contextra-infer-onnx` (normal/optional via onnx), `contextra-license` (normal), `contextra-ports` (normal), `contextra-privacy` (normal/optional via sovereign), `contextra-rank` (normal/optional via router), `contextra-router` (normal/optional via router), `contextra-store` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-core` | 8 | 12 | 5 | `error::ContextraError`×3, `DistanceMetric`×2, `types::domain::AutoExtractionMode`, `types::domain`, `error::Result` | – |
| `contextra-ports` | 5 | 8 | 4 | `license`×2, `IdGen`, `license::LicenseGate`, `license::VectorDeleteMode` | `license::OpenFastGate`×2, `SequentialIdGen`, `license::LicenseError`, `license::FeatureRing` |
| `contextra-license` | 2 | 2 | 1 | `SignedLicenseGate`, `signed_gate::VerifyingKey::from_bytes` | `OpenFastGate` |
| `contextra-store` | 2 | 3 | 2 | `kv::delete_mode::KvDeleteMode`, `lsm::config` | `lsm::config::DurabilityMode`, `lsm::DurabilityMode` |
| `contextra-db` | 1 | 4 | 1 | `QueryRewriter` | `EmbeddingBackend`, `ContextraConfig` |
| `contextra-infer-candle` | 1 | 0 | 1 | `(crate)` | – |
| `contextra-infer-ollama` | 1 | 0 | 1 | `(crate)` | – |
| `contextra-infer-onnx` | 1 | 0 | 1 | `(crate)` | – |
| `contextra-rank` | 1 | 0 | 1 | `(crate)` | – |
| `contextra-router` | 1 | 0 | 1 | `(crate)` | – |
| `contextra-types` | 1 | 2 | 1 | `ScoredEntry` | – |

**Pro Komponente**

- `(crate-root)` (74 Zeilen)
  - → `contextra-core`: prod: `error::ContextraError`, `DistanceMetric`, `types::domain`
  - → `contextra-infer-candle`: prod: `(crate)`
  - → `contextra-infer-ollama`: prod: `(crate)`
  - → `contextra-infer-onnx`: prod: `(crate)`
  - → `contextra-rank`: prod: `(crate)`
  - → `contextra-router`: prod: `(crate)`
- `agent_memory` (294 Zeilen)
  - → `contextra-core`: prod: `error::ContextraError`
  - → `contextra-ports`: prod: `IdGen`
- `builder` (322 Zeilen)
  - → `contextra-core`: prod: `error::ContextraError`, `DistanceMetric`
  - → `contextra-license`: prod: `SignedLicenseGate`, `signed_gate::VerifyingKey::from_bytes`
  - → `contextra-ports`: prod: `license`
- `collection_profile` (112 Zeilen)
  - → `contextra-core`: prod: `types::domain::AutoExtractionMode`
  - → `contextra-ports`: prod: `license::LicenseGate`
  - → `contextra-store`: prod: `kv::delete_mode::KvDeleteMode`
- `performance_profile` (78 Zeilen)
  - → `contextra-ports`: prod: `license::VectorDeleteMode`, `license`
  - → `contextra-store`: prod: `lsm::config`
- `query_rewriter` (199 Zeilen)
  - → `contextra-core`: prod: `error::Result`
  - → `contextra-db`: prod: `QueryRewriter`
  - → `contextra-types`: prod: `ScoredEntry`
- `tests/agent_memory_idgen_test.rs` (72 Zeilen)
  - → `contextra-ports`: dev: `SequentialIdGen`
- `tests/builder_levers_contract.rs` (202 Zeilen)
  - → `contextra-core`: dev: `error::ContextraError`, `DistanceMetric`, `error::Result`
  - → `contextra-ports`: dev: `license`, `license::LicenseError`
- `tests/collection_profile_matrix.rs` (51 Zeilen)
  - → `contextra-store`: dev: `kv::delete_mode::KvDeleteMode`
- `tests/j36-contextra_symbols_wiring_test.rs` (137 Zeilen)
  - → `contextra-core`: dev: `error::Result`×3
  - → `contextra-db`: dev: `QueryRewriter`
  - → `contextra-ports`: dev: `license::OpenFastGate`
  - → `contextra-types`: dev: `ScoredEntry`
- `tests/j36closure-contextra_symbols_wiring_test.rs` (93 Zeilen)
  - → `contextra-core`: dev: `DistanceMetric`, `error::ContextraError`
  - → `contextra-db`: dev: `EmbeddingBackend`
  - → `contextra-ports`: dev: `license::OpenFastGate`, `license::LicenseGate`
- `tests/license_gate_always_enforced.rs` (184 Zeilen)
  - → `contextra-core`: dev: `error::ContextraError`
  - → `contextra-db`: dev: `ContextraConfig`
  - → `contextra-license`: dev: `SignedLicenseGate`
  - → `contextra-ports`: dev: `license::FeatureRing`
  - → `contextra-store`: dev: `lsm::config::DurabilityMode`
- `tests/license_gate_enforcement.rs` (128 Zeilen)
  - → `contextra-core`: dev: `error::ContextraError`
  - → `contextra-license`: dev: `OpenFastGate`
  - → `contextra-ports`: dev: `license`
- `tests/llm_query_rewriter.rs` (180 Zeilen)
  - → `contextra-core`: dev: `error::ContextraError`, `error::Result`
  - → `contextra-db`: dev: `QueryRewriter`
  - → `contextra-types`: dev: `ScoredEntry`
- `tests/wiring_matrix.rs` (109 Zeilen)
  - → `contextra-store`: dev: `lsm::DurabilityMode`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 3 fn
    - `fn builder`
    - `fn open` ⚠ fremd: `contextra-core`
    - `fn open_with_config` ⚠ fremd: `contextra-core`
- `agent_memory`
  - 4 struct
    - `struct MemoryId` · 2 Methoden
    - `struct RelationId`
    - `struct Memory`
    - `struct AgentMemory` · 10 Methoden
- `builder`
  - 1 struct
    - `struct ContextraBuilder` · 22 Methoden
- `collection_profile`
  - 2 const, 2 enum, 2 struct
    - `const DEFAULT_AUTO_EXTRACTION_MODE` ⚠ fremd: `contextra-core`
    - `const ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT` ⚠ fremd: `contextra-core`
    - `struct LsmTuning`
    - `struct CollectionProfile` · 2 Methoden ⚠ fremd: `contextra-core`, `contextra-store`
    - `enum CollectionProfileError`
    - `enum DeploymentTier` · 1 Methoden
- `performance_profile`
  - 2 enum, 1 struct
    - `enum PerformanceProfile` · 1 Methoden
    - `struct ResolvedProfileConfig` · 2 Methoden ⚠ fremd: `contextra-ports`
    - `enum PerformanceProfileError`
- `query_rewriter`
  - 1 struct
    - `struct LlmQueryRewriter` · 8 Methoden
- Zusätzlich 3 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_core::types::domain::AutoExtractionMode`, `contextra_core::error::ContextraError`, `contextra_core::DistanceMetric`, `contextra_router`, `contextra_rank`, `contextra_infer_ollama`, `contextra_infer_candle`, `contextra_infer_onnx`, `contextra_ports::license::VectorDeleteMode`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-mcp` | `router::SlmProfile`×4, `Contextra`×3, `router::DefaultRouterEngine`×2, `Collection`, `router::ports_local::HybridSearchProvider`, `router::ports_local::CommunityResolver`, `router::ports_local::ContextPreparer`, `router::RouterEngine::new`, `chunker`, `execute_background_consolidation` … (+4) | `router::SlmProfile::new` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 17 von 18)**: `MemoryId`, `RelationId`, `Memory`, `AgentMemory`, `ContextraBuilder`, `DEFAULT_AUTO_EXTRACTION_MODE`, `ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT`, `LsmTuning`, `CollectionProfile`, `CollectionProfileError`, `builder`, `open`, `open_with_config`, `PerformanceProfile`, `ResolvedProfileConfig`, `PerformanceProfileError`, `LlmQueryRewriter`

**Auffälligkeiten**

- contextra-audit-export: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-avv-generator: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-crypto: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-privacy: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-adapt

- Pfad: `crates/contextra-adapt` · Zeilen: 6285 · Rust-Dateien: 33 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 32 | 2 | 3 | `ContextraError::InvalidInput`×9, `RetrievalStrategy`×9, `ContextraError`×4, `TenantId`×3, `ContextraError::Internal`×2, `RetrievalStrategy::Vector`×2, `RetrievalStrategy::Text`, `RetrievalStrategy::Graph`, `RetrievalStrategy::Hybrid` | – |
| `contextra-ports` | 2 | 0 | 1 | `SeededRng::new`, `Rng::next_u64` | – |

**Pro Komponente**

- `bandit` (1068 Zeilen)
  - → `contextra-ports`: prod: `SeededRng::new`, `Rng::next_u64`
  - → `contextra-types`: prod: `TenantId`×3, `ContextraError::InvalidInput`×2, `ContextraError::Internal`×2, `ContextraError`
- `flow_thompson` (808 Zeilen)
  - → `contextra-types`: prod: `RetrievalStrategy`×9, `ContextraError::InvalidInput`×4, `ContextraError`×2, `RetrievalStrategy::Vector`×2, `RetrievalStrategy::Text`, `RetrievalStrategy::Graph`, `RetrievalStrategy::Hybrid`
- `rie_greedy` (229 Zeilen)
  - → `contextra-types`: prod: `ContextraError::InvalidInput`×3, `ContextraError`
- `tests/j11-contextra-adapt_bandit_tenant.rs` (35 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/sketched_projection_determinism.rs` (99 Zeilen)
  - → `contextra-types`: dev: `TenantId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `bandit`
  - 1 const, 2 enum, 3 struct, 1 trait
    - `const SHERMAN_MORRISON_REFACTORIZATION_INTERVAL`
    - `enum BanditError`
    - `struct AlignedF32Vec` · 2 Methoden
    - `enum BanditImplementation`
    - `struct SketchMatrix` · 4 Methoden
    - `trait BanditPolicy` · Methoden: apply_drift_penalty
    - `struct BanditProfileState` · 8 Methoden
- `decay_controller`
  - 3 struct
    - `struct DecaySignalInputs`
    - `struct DecayControllerConfig`
    - `struct AdaptiveDecayController` · 6 Methoden
- `drift`
  - 1 enum, 3 struct, 1 trait
    - `enum DriftSignal`
    - `trait DriftDetector` · Methoden: observe, is_drifting
    - `struct CatoniDriftDetector` · 2 Methoden
    - `struct EnsembleDriftWatcher` · 4 Methoden
    - `struct DriftPolicyBridge` · 2 Methoden
- `flow_thompson`
  - 2 const, 2 enum, 1 fn, 9 struct, 3 trait
    - `enum FcTsError`
    - `trait FcTsRng` · Methoden: next_u64, next_f32, next_standard_normal
    - `struct SplitMix64` · 1 Methoden
    - `struct FcTsConfig` · 1 Methoden
    - `struct Ring3Token`
    - `fn ring3_background_task_token`
    - `struct FlowCorrectedThompsonBandit` · 13 Methoden
    - `struct DiagonalApproximationBandit` · 3 Methoden ⚠ fremd: `contextra-types`
    - `trait OffPolicyCompatibility` · Methoden: verify_positivity
    - `enum OffPolicyError` ⚠ fremd: `contextra-types`
    - `struct FcTsSamplingDistribution` · 3 Methoden ⚠ fremd: `contextra-types`
    - `struct FcTsArmSet` · 4 Methoden
    - `const MIN_BANDIT_SHADOW_SAMPLES`
    - `const MAX_CUMULATIVE_REGRET_THRESHOLD`
    - `struct BanditShadowReport`
    - `trait BanditDefaultFlipGate` · Methoden: should_flip
    - `struct DefaultBanditFlipGate`
- `homeostat`
  - 1 fn, 2 struct
    - `struct RerankPidController` · 9 Methoden
    - `fn pid_regulated_candidate_pool`
    - `struct RerankDeadline` · 2 Methoden
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
    - `struct PidLatencyController` · 6 Methoden
    - `struct LatencyBudgetGuard` · 4 Methoden
- `rie_greedy`
  - 1 enum, 1 struct
    - `enum RieGreedyError`
    - `struct RieGreedyProfile` · 7 Methoden
- `shadow_mode`
  - 2 struct, 1 trait
    - `struct ShadowDiscrepancy`
    - `trait ShadowSink` · Methoden: record
    - `struct TracingShadowSink`
- Interne Re-Exporte (`pub use`): `*`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-db` | `homeostat::RerankDeadline`, `homeostat::RerankPidController`, `decay_controller::*`, `pid_latency_controller::*`, `PidController` | `PidController::new` |
| `contextra-engine` | `PidController`×4, `decay_controller` | `PidController::default`×2 |
| `contextra-router` | `bandit`, `lyapunov`, `offpolicy`, `offpolicy::RandomizedLoggingPolicy` | `bandit::BanditProfileState`×4, `BanditError`, `FcTsError::InvalidConfig` |
| `contextra-mcp` | `PidController`, `PidController::default` | – |
| `contextra-graph` | `TracingShadowSink` | – |
| `contextra-rank` | – | `PidController`×2 |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

### contextra-agent

- Pfad: `crates/contextra-agent` · Zeilen: 8564 · Rust-Dateien: 37 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-text` (dev), `contextra-vector` (dev), `contextra-checkpoint` (normal), `contextra-db` (normal), `contextra-graph` (normal), `contextra-ports` (normal), `contextra-router` (normal), `contextra-store` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 24 | 75 | 6 | `ContextraError::InvalidInput`×10, `Result`×3, `WorkflowState`×3, `TxId`×2, `TokenBudget::new`×2, `ContextraError::Internal`, `DocId::from_key`, `types::budget`, `error` | `TokenBudget`×13, `TxId::new`×5, `ContextraError::Conflict`, `ContextraError::Serialization`, `ContextraError` |
| `contextra-ports` | 14 | 18 | 4 | `BoxFuture`×6, `MetricsSink`×4, `StorageEngine`×2, `VectorIndex`, `Checkpoint` | `StorageStats`×2, `SystemClock` |
| `contextra-db` | 5 | 42 | 4 | `Collection`×2, `Contextra`×2, `volatile_vault` | `Contextra::open_with_config`×18, `ContextraConfig::default`×15, `volatile_vault::VaultConfig`×5, `ContextraConfig`×2, `DistanceMetric::Cosine` |
| `contextra-router` | 4 | 0 | 2 | `DefaultRouterEngine`, `DecisionId`, `RoutingOutcome::Success`, `RoutingOutcome::Rejected` | – |
| `contextra-store` | 3 | 0 | 3 | `LsmStorage`×3 | – |
| `contextra-checkpoint` | 0 | 1 | 0 | – | `StateCheckpoint` |
| `contextra-graph` | 0 | 2 | 0 | – | `CsrGraph`×2 |
| `contextra-text` | 0 | 5 | 0 | – | `Language::English`×5 |

**Pro Komponente**

- `audit` (652 Zeilen)
  - → `contextra-db`: prod: `Collection`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: prod: `StorageEngine`, `VectorIndex` · dev: `StorageStats`×2, `BoxFuture`
  - → `contextra-store`: prod: `LsmStorage`
  - → `contextra-text`: dev: `Language::English`×4
  - → `contextra-types`: prod: `ContextraError::InvalidInput`×4, `Result`, `TxId`, `ContextraError::Internal`, `DocId::from_key` · dev: `ContextraError::Internal`×7, `TxId`×7, `ContextraError::InvalidInput`×5, `TxId::new`×2 … (+4)
- `budget` (72 Zeilen)
  - → `contextra-types`: prod: `types::budget`, `Result`
- `clm_scratchpad` (631 Zeilen)
  - → `contextra-db`: prod: `volatile_vault`
- `context` (482 Zeilen)
  - → `contextra-db`: dev: `ContextraConfig::default`×4, `Contextra::open_with_config`×4, `volatile_vault::VaultConfig`
  - → `contextra-router`: prod: `DefaultRouterEngine`, `DecisionId`
  - → `contextra-store`: prod: `LsmStorage`
  - → `contextra-types`: prod: `ContextraError::InvalidInput`×2, `error`, `Result` · dev: `Result`×4
- `dlq` (350 Zeilen)
  - → `contextra-ports`: prod: `MetricsSink`×3, `StorageEngine` · dev: `MetricsSink`
- `engine` (1165 Zeilen)
  - → `contextra-db`: prod: `Contextra`×2 · dev: `volatile_vault::VaultConfig`
  - → `contextra-ports`: prod: `BoxFuture`×5, `MetricsSink`, `Checkpoint` · dev: `BoxFuture`×2
  - → `contextra-router`: prod: `RoutingOutcome::Success`, `RoutingOutcome::Rejected`
  - → `contextra-store`: prod: `LsmStorage`
  - → `contextra-types`: prod: `WorkflowState`×3, `TokenBudget::new`×2, `TxId` · dev: `Result`×2, `TokenBudget`, `TxId`
- `event_source` (262 Zeilen)
  - → `contextra-db`: prod: `Collection`
  - → `contextra-types`: prod: `ContextraError::InvalidInput`×3
- `graph` (275 Zeilen)
  - → `contextra-types`: prod: `ContextraError::InvalidInput`
- `step` (83 Zeilen)
  - → `contextra-ports`: prod: `BoxFuture`
- `tests/agent_recovery.rs` (372 Zeilen)
  - → `contextra-checkpoint`: dev: `StateCheckpoint`
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `TxId::new`×3, `Result`×2, `TokenBudget`, `ContextraError::Internal`
- `tests/audit_isolation_test.rs` (360 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
- `tests/boundary_validation_tests.rs` (122 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `ContextraError`, `Result`
- `tests/budget_race_test.rs` (290 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `ContextraError::Internal`
- `tests/clm_scratchpad_reset_flow.rs` (85 Zeilen)
  - → `contextra-db`: dev: `volatile_vault::VaultConfig`
  - → `contextra-ports`: dev: `SystemClock`
- `tests/clock_tests.rs` (143 Zeilen)
  - → `contextra-db`: dev: `Contextra::open_with_config`×4, `ContextraConfig::default`×2
- `tests/contract_tests.rs` (197 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`, `StorageEngine`
  - → `contextra-types`: dev: `TokenBudget`, `Result`
- `tests/dlq_clock_determinism_test.rs` (93 Zeilen)
  - → `contextra-db`: dev: `ContextraConfig::default`, `Contextra::open_with_config`
- `tests/dlq_tests.rs` (289 Zeilen)
  - → `contextra-db`: dev: `ContextraConfig::default`×5, `Contextra::open_with_config`×5
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `TxId`×2
- `tests/e2e_integration.rs` (173 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `TokenBudget`, `Result`
- `tests/event_loop_integration.rs` (305 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `Result`×3, `TokenBudget`
- `tests/final_state_test.rs` (48 Zeilen)
  - → `contextra-db`: dev: `Contextra`, `ContextraConfig`
  - → `contextra-types`: dev: `TokenBudget`, `Result`
- `tests/graph_integration.rs` (95 Zeilen)
  - → `contextra-db`: dev: `ContextraConfig`, `DistanceMetric::Cosine`, `Contextra::open_with_config`
  - → `contextra-types`: dev: `TokenBudget::new`
- `tests/j01-contextra-agent-scratchpad-kontext_test.rs` (96 Zeilen)
  - → `contextra-db`: dev: `volatile_vault::VaultConfig`
  - → `contextra-types`: dev: `error`, `Result`
- `tests/j01closure-contextra-agent_wiring_test.rs` (91 Zeilen)
  - → `contextra-db`: dev: `volatile_vault::VaultConfig`, `ContextraConfig::default`, `Contextra::open_with_config`
  - → `contextra-types`: dev: `error`, `Result`
- `tests/persistence_test.rs` (131 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `TokenBudget`, `Result`
- `tests/regression_agt_a.rs` (157 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `Result`
- `tests/system_prompt_pinning_latency.rs` (66 Zeilen)
  - → `contextra-db`: dev: `ContextraConfig::default`×2, `Contextra::open_with_config`×2
  - → `contextra-types`: dev: `Result`×2, `TokenBudget`, `error`
- `tests/workflow_tests.rs` (499 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `TokenBudget`×5, `ContextraError::InvalidInput`×2, `Result`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `audit`
  - 2 fn, 3 struct
    - `struct AuditEntry` ⚠ fremd: `contextra-types`
    - `struct MigrationStats`
    - `struct AuditLog` · 5 Methoden ⚠ fremd: `contextra-ports`, `contextra-store`
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
    - `struct ClmScratchpad` · 16 Methoden
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
    - `struct DeadLetterQueue` · 10 Methoden
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
    - `struct PollingDocumentEventSource` · 5 Methoden
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

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 47 von 47)**: `AuditEntry`, `MigrationStats`, `AuditLog`, `validate_audit_payload_and_error`, `migrate_legacy_audit_entries`, `reserve_tokens`, `ContextEditAuditRecord`, `ContextEditAuditSink`, `NoopContextEditAuditSink`, `CountingContextEditAuditSink`, `ScratchpadCacheInvalidator`, `NoopScratchpadCacheInvalidator`, `CountingScratchpadCacheInvalidator`, `PinnedRegionId`, `ScratchpadEditOp`, `ScratchpadCheckpoint`, `ClmScratchpad`, `MAX_ID_LEN`, `MAX_TELEMETRY_EVENTS`, `validate_task_id`, `validate_node_id`, `AgentStatus`, `AgentContext`, `AgentEngine`, `DeadLetterQueue`, `evaluate_condition_expr`, `EventLoopExitReason`, `MAX_WORKFLOW_STEPS`, `OrchestratorEngine`, `MAX_EVENT_SOURCE_CAPACITY` …

**Auffälligkeiten**

- contextra-checkpoint: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-graph: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-vector: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-audit-export

- Pfad: `crates/contextra-audit-export` · Zeilen: 856 · Rust-Dateien: 10 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 2 | 5 | 2 | `TenantId`×2 | – |

**Pro Komponente**

- `(crate-root)` (144 Zeilen)
  - → `contextra-types`: prod: `TenantId`
- `testkit` (74 Zeilen)
  - → `contextra-types`: prod: `TenantId`
- `tests/art30_register_complete.rs` (111 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/campaign_j27_compliance_test.rs` (88 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/j16-contextra-audit-export_wiring_test.rs` (43 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/j16closure-contextra-audit-export_test.rs` (80 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/register_rendering.rs` (61 Zeilen)
  - → `contextra-types`: dev: `TenantId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 fn, 5 struct, 1 trait
    - `struct EgressEventSummary`
    - `struct DeletionProofSummary`
    - `struct ProcessingRegisterEntry` ⚠ fremd: `contextra-types`
    - `struct Art30ProcessingRegisterEntry` ⚠ fremd: `contextra-types`
    - `trait ProcessingRegisterSource` · Methoden: collect_entries
    - `fn render_register_json`
    - `struct AuditRegisterExporter` · 7 Methoden
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
    - `fn render_register_markdown`
    - `fn render_art30_register_markdown`
- `testkit`
  - 1 struct
    - `struct InMemoryProcessingRegisterSource` · 3 Methoden

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 14 von 14)**: `BsiMappingEntry`, `bsi_mapping_table`, `render_bsi_mapping_markdown`, `AuditExportError`, `EgressEventSummary`, `DeletionProofSummary`, `ProcessingRegisterEntry`, `Art30ProcessingRegisterEntry`, `ProcessingRegisterSource`, `render_register_json`, `AuditRegisterExporter`, `render_register_markdown`, `render_art30_register_markdown`, `InMemoryProcessingRegisterSource`

**Auffälligkeiten**

- contextra-ports: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-avv-generator

- Pfad: `crates/contextra-avv-generator` · Zeilen: 591 · Rust-Dateien: 7 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 1 | 4 | 1 | `TenantId` | – |

**Pro Komponente**

- `(crate-root)` (154 Zeilen)
  - → `contextra-types`: prod: `TenantId`
- `tests/avv_rendering.rs` (91 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/campaign_j27_avv_test.rs` (75 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/j17-contextra-avv-generator_wiring_test.rs` (49 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/j17closure-contextra-avv-generator_symbols_wiring_test.rs` (61 Zeilen)
  - → `contextra-types`: dev: `TenantId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 2 fn, 3 struct
    - `struct AvvContext` · 4 Methoden ⚠ fremd: `contextra-types`
    - `struct AvvContextBuilder` · 9 Methoden
    - `struct TechnicalMeasure`
    - `fn render_avv_markdown`
    - `fn default_technical_measures`
- `error`
  - 1 enum
    - `enum AvvGeneratorError`
- `template`
  - 1 fn
    - `fn render`

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 7 von 7)**: `AvvGeneratorError`, `AvvContext`, `AvvContextBuilder`, `TechnicalMeasure`, `render_avv_markdown`, `default_technical_measures`, `render`

### contextra-bench

- Pfad: `benchmarks/contextra-bench` · Zeilen: 7192 · Rust-Dateien: 18 · Ring: tooling
- Interne Cargo-Abhängigkeiten: `contextra-core` (normal), `contextra-crypto` (normal), `contextra-db` (normal), `contextra-graph` (normal), `contextra-infer-candle` (normal), `contextra-infer-onnx` (normal), `contextra-kvcache` (normal), `contextra-ports` (normal), `contextra-store` (normal), `contextra-text` (normal), `contextra-types` (normal), `contextra-vector` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-core` | 9 | 10 | 3 | `ContextraError::Internal`×3, `ContextraError::SnapshotUnsupportedForSignal`×2, `traits::VectorIndex`, `types`, `ContextraError::CapabilityUnsupported`, `Result` | `model_fingerprint::ModelFingerprint`, `TenantId`, `kv`, `ContextraError::InvalidInput` |
| `contextra-infer-candle` | 2 | 0 | 1 | `CandleEmbedClient::from_dir`, `model_registry::CandleQuantization::Q4KM` | – |
| `contextra-db` | 1 | 0 | 1 | `Collection` | – |
| `contextra-crypto` | 0 | 2 | 0 | – | `kv_cipher::KvSegmentCipher`, `KeyManager` |
| `contextra-store` | 0 | 1 | 0 | – | `lsm` |

**Pro Komponente**

- `(crate-root)` (1488 Zeilen)
  - → `contextra-core`: prod: `ContextraError::Internal`×3, `ContextraError::CapabilityUnsupported`
  - → `contextra-infer-candle`: prod: `CandleEmbedClient::from_dir`, `model_registry::CandleQuantization::Q4KM`
- `ann_benchmarks` (240 Zeilen)
  - → `contextra-core`: prod: `traits::VectorIndex`, `types`
- `locomo` (475 Zeilen)
  - → `contextra-core`: dev: `Result`
- `long_mem_eval` (1349 Zeilen)
  - → `contextra-db`: prod: `Collection`
- `path_rag_sweep` (318 Zeilen)
  - → `contextra-core`: prod: `ContextraError::SnapshotUnsupportedForSignal`×2, `Result`
- `benches/competitive_kv.rs` (769 Zeilen)
  - → `contextra-store`: dev: `lsm`
- `benches/kv_cache_effect.rs` (285 Zeilen)
  - → `contextra-core`: dev: `model_fingerprint::ModelFingerprint`, `TenantId`, `kv`
  - → `contextra-crypto`: dev: `kv_cipher::KvSegmentCipher`, `KeyManager`
- `tests/external_benchmarks_test.rs` (376 Zeilen)
  - → `contextra-core`: dev: `Result`×5, `ContextraError::InvalidInput`

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
- `locomo`
  - 1 enum, 2 fn, 4 struct, 1 type
    - `enum LocomoQuestionCategory` · 1 Methoden
    - `struct LocomoCase`
    - `struct LocomoReport`
    - `type LocomoConversationIndex`
    - `struct LocomoDataset`
    - `struct RawLocomoTurn`
    - `fn load_locomo_dataset` ⚠ fremd: `contextra-core`
    - `fn run_locomo_eval` ⚠ fremd: `contextra-core`
- `long_mem_eval`
  - 1 enum, 3 fn, 8 struct, 1 type
    - `type BoxFuture`
    - `struct ScoredChunk`
    - `enum LongMemEvalQuestionType`
    - `struct LongMemEvalCase` · 2 Methoden
    - `struct LongMemEvalReport`
    - `struct SessionTurn`
    - `struct Session`
    - `struct MultiSessionScenario`
    - `struct RegressionSuite` · 2 Methoden
    - `struct RegressionReport`
    - `fn check_regression`
    - `fn load_from_jsonl`
    - `fn run_long_mem_eval`
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

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `xtask-heavy` | `regression_gate` | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 49 von 49)**: `AnnBenchResult`, `generate_synthetic_vectors`, `compute_ground_truth`, `recall_at_k`, `run_ann_benchmark`, `run_ann_benchmark_synthetic_ci`, `BeirDocument`, `BeirQuery`, `BeirMetrics`, `load_beir_corpus`, `load_beir_queries`, `ndcg_at_k`, `map_at_k`, `recall_at_k`, `run_beir_eval`, `LongMemEvalMetricsSummary`, `LocomoMetricsSummary`, `CombinedMetrics`, `MetricComparisonResult`, `compare_metrics`, `compare_metrics_files`, `LocomoQuestionCategory`, `LocomoCase`, `LocomoReport`, `LocomoConversationIndex`, `LocomoDataset`, `RawLocomoTurn`, `load_locomo_dataset`, `run_locomo_eval`, `BoxFuture` …

**Auffälligkeiten**

- contextra-crypto: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-graph: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-infer-onnx: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-kvcache: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-ports: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-store: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-text: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-types: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-vector: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-checkpoint

- Pfad: `crates/contextra-checkpoint` · Zeilen: 7429 · Rust-Dateien: 27 · Ring: 1
- Interne Cargo-Abhängigkeiten: `contextra-store` (dev), `contextra-testkit` (dev), `contextra-core` (normal), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 47 | 9 | 4 | `Clock`×19, `StorageEngine`×14, `MetricsSink`×6, `BoxFuture`×3, `SystemClock`×2, `Checkpoint`×2, `CheckpointCoordinator` | – |
| `contextra-core` | 2 | 3 | 2 | `SnapshotRegistry`×2 | – |
| `contextra-store` | 0 | 1 | 0 | – | `lsm` |
| `contextra-types` | 0 | 12 | 0 | – | `TxId`×5, `Result`×3, `ContextraError`×2, `ContextraError::Internal`×2 |

**Pro Komponente**

- `guard` (955 Zeilen)
  - → `contextra-ports`: prod: `Clock`×9, `StorageEngine`×6, `MetricsSink`×3, `SystemClock` · dev: `MetricsSink`×2, `Clock`
- `hardlink_cloner` (177 Zeilen)
  - → `contextra-core`: prod: `SnapshotRegistry`
  - → `contextra-ports`: prod: `BoxFuture`×2
- `manifest` (154 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `orphan` (667 Zeilen)
  - → `contextra-ports`: prod: `Clock`×7, `StorageEngine`×2, `SystemClock`
- `store` (1540 Zeilen)
  - → `contextra-core`: prod: `SnapshotRegistry`
  - → `contextra-ports`: prod: `StorageEngine`×6, `Clock`×3, `MetricsSink`×3, `Checkpoint`×2, `BoxFuture`, `CheckpointCoordinator` · dev: `CheckpointCoordinator`×5
- `tests/checkpoint_clock_determinism.rs` (160 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/checkpoint_systematic_crash.rs` (211 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-store`: dev: `lsm`
  - → `contextra-types`: dev: `TxId`
- `tests/clock_determinism.rs` (98 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/hardlink_cloner_tests.rs` (113 Zeilen)
  - → `contextra-core`: dev: `SnapshotRegistry`
  - → `contextra-types`: dev: `Result`×3, `ContextraError`
- `tests/j21-contextra-checkpoint_wire_tests.rs` (195 Zeilen)
  - → `contextra-core`: dev: `SnapshotRegistry`
- `tests/j21closure-contextra-checkpoint_closure_tests.rs` (215 Zeilen)
  - → `contextra-core`: dev: `SnapshotRegistry`
- `tests/manifest_fault_injection.rs` (68 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/time_travel_correctness.rs` (723 Zeilen)
  - → `contextra-types`: dev: `ContextraError::Internal`×2, `ContextraError`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `manifest`
  - 1 struct
    - `struct CheckpointManifest` *(re-exportiert)* · 2 Methoden
- Zusätzlich 34 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

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

- Pfad: `crates/contextra-cognition` · Zeilen: 8032 · Rust-Dateien: 31 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-infer-candle` (dev), `contextra-engine` (normal), `contextra-graph` (normal), `contextra-ports` (normal), `contextra-store` (normal), `contextra-types` (normal), `contextra-vector` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 15 | 30 | 3 | `ContextraError::InvalidInput`×3, `ContextraError::Internal`×2, `ContextChunk`×2, `ContextraError::PolicyViolation`×2, `Result`×2, `TenantId`×2, `TxId`, `DocId` | `MemoryType::Episodic`×6, `DocId::new`×5 |
| `contextra-engine` | 13 | 25 | 9 | `collection`×4, `collection::Collection`×3, `decay_controller::AdaptiveDecayController`×2, `transaction::CommitIntent`×2, `ProvenanceRecord`, `decay_controller::DecayControllerConfig` | `Language::English`×10, `SearchResult`×2, `collection::Collection::new`×2, `Language::German` |
| `contextra-graph` | 13 | 20 | 6 | `EdgeReinforcementBuffer`×3, `HyperEdgeId`, `cascade::CascadeInvalidationReport`, `cascade_invalidate_edges_for_superseded_doc`, `csr::EdgeType`, `CsrGraph`, `hyperedge`, `hyperedge::HyperEdgeId`, `percolation::PercolationConfig`, `EdgeReinforcementConfig` … (+1) | `EdgeReinforcementBuffer::new`×2 |
| `contextra-ports` | 4 | 4 | 3 | `SegmentSynthesizer`×2, `LlmTextGenerator`, `StorageEngine` | `BoxFuture`×2, `SeededRng`, `ResponseGroundingValidator` |
| `contextra-vector` | 3 | 16 | 3 | `HnswIndex`×3 | `HnswConfig`×8 |
| `contextra-infer-candle` | 0 | 1 | 0 | – | `gasp` |
| `contextra-store` | 0 | 16 | 0 | – | `LsmStorage`×8, `LsmConfig`×8 |

**Pro Komponente**

- `aggregation_phase` (612 Zeilen)
  - → `contextra-graph`: prod: `HyperEdgeId`
  - → `contextra-ports`: prod: `LlmTextGenerator`
- `consolidation_executor` (1046 Zeilen)
  - → `contextra-engine`: prod: `decay_controller::AdaptiveDecayController`, `collection` · dev: `Language::English`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-types`: prod: `ContextraError::Internal`×2, `ContextChunk`×2, `ContextraError::PolicyViolation`, `TxId` · dev: `MemoryType::Episodic`×2, `DocId::new`×2, `Result`, `ContextraError::Internal` … (+1)
  - → `contextra-vector`: prod: `HnswIndex` · dev: `HnswIndex`, `HnswConfig`
- `consolidation_locks` (94 Zeilen)
  - → `contextra-engine`: prod: `collection::Collection`
  - → `contextra-graph`: prod: `cascade::CascadeInvalidationReport`, `cascade_invalidate_edges_for_superseded_doc`
- `context` (508 Zeilen)
  - → `contextra-engine`: dev: `SearchResult`×2, `ProvenanceRecord`×2
  - → `contextra-types`: dev: `DocId`×2, `ContextraError::InvalidInput`
- `context_compaction` (1154 Zeilen)
  - → `contextra-engine`: prod: `transaction::CommitIntent`×2, `collection::Collection`, `ProvenanceRecord`, `collection` · dev: `Language::English`×2, `collection::Collection`, `transaction::CommitIntent`, `ProvenanceRecord`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: prod: `StorageEngine` · dev: `SeededRng`
  - → `contextra-vector`: prod: `HnswIndex`
- `graph_sink` (79 Zeilen)
  - → `contextra-graph`: prod: `csr::EdgeType`, `CsrGraph`, `hyperedge`
- `leanrag_input` (97 Zeilen)
  - → `contextra-engine`: prod: `collection::Collection`
  - → `contextra-graph`: prod: `hyperedge::HyperEdgeId`
- `maintenance_config` (80 Zeilen)
  - → `contextra-engine`: prod: `decay_controller::DecayControllerConfig`
  - → `contextra-graph`: prod: `percolation::PercolationConfig`, `EdgeReinforcementConfig`, `EdgeReinforcementConfig::default`
- `maintenance_scheduler` (505 Zeilen)
  - → `contextra-engine`: prod: `decay_controller::AdaptiveDecayController`, `collection` · dev: `Language::English`
  - → `contextra-graph`: prod: `EdgeReinforcementBuffer`×3 · dev: `CsrGraph`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-vector`: prod: `HnswIndex` · dev: `HnswIndex`, `HnswConfig`
- `memory_consolidation` (1001 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: prod: `ContextraError::InvalidInput`×3, `Result`×2, `TenantId`×2, `ContextraError::PolicyViolation` · dev: `Result`×2, `ContextraError::Internal`
- `semantic_aggregation_facade` (97 Zeilen)
  - → `contextra-engine`: prod: `collection`
- `synthesis_phase` (177 Zeilen)
  - → `contextra-ports`: prod: `SegmentSynthesizer`×2 · dev: `BoxFuture`
  - → `contextra-types`: prod: `DocId` · dev: `Result`, `ContextraError::Internal`
- `tests/grounding_in_consolidation_pipeline.rs` (197 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection::new`, `Language::English`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-types`: dev: `MemoryType::Episodic`×2, `ContextraError::PolicyViolation`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/grounding_validator_rich_path_test.rs` (163 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection::new`, `Language::German`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-infer-candle`: dev: `gasp`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-types`: dev: `DocId::new`×3, `MemoryType::Episodic`×2, `Result`, `ContextChunk` … (+1)
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/j13-contextra-cognition_wiring_test.rs` (196 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection`, `Language::English`
  - → `contextra-graph`: dev: `CsrGraph`, `EdgeReinforcementBuffer::new`
- `tests/j13closure-contextra-cognition_wiring_test.rs` (137 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection`, `Language::English`
  - → `contextra-graph`: dev: `CsrGraph`, `EdgeReinforcementBuffer::new`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/leanrag_aggregation.rs` (375 Zeilen)
  - → `contextra-graph`: dev: `HyperEdgeId`
- `tests/leanrag_stage3_wiring.rs` (174 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection`, `Language::English`
  - → `contextra-graph`: dev: `csr::EdgeType`, `CsrGraph`, `hyperedge`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-types`: dev: `Result`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/leanrag_worker_opt_in.rs` (355 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection`, `Language::English`
  - → `contextra-graph`: dev: `csr::EdgeType`, `CsrGraph`, `hyperedge`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-types`: dev: `Result`, `ContextraError::Internal`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/pre_condensed_input_intake.rs` (215 Zeilen)
  - → `contextra-ports`: dev: `ResponseGroundingValidator`
- `tests/rng_seed_determinism.rs` (162 Zeilen)
  - → `contextra-graph`: dev: `HyperEdgeId`
- `tests/semantic_aggregation_facade.rs` (253 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection`, `Language::English`
  - → `contextra-graph`: dev: `csr::EdgeType`, `CsrGraph`, `hyperedge`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
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
  - 8 fn, 3 struct
    - `struct ConsolidationLockGuard` · 1 Methoden
    - `fn execute_consolidation_pass` ⚠ fremd: `contextra-engine`, `contextra-types`
    - `struct GroundingValidatorAdapter`
    - `fn execute_background_consolidation` ⚠ fremd: `contextra-engine`, `contextra-types`
    - `fn execute_background_consolidation_with_rich_validator` ⚠ fremd: `contextra-engine`, `contextra-types`
    - `fn execute_leanrag_aggregation_stage` ⚠ fremd: `contextra-engine`, `contextra-types`
    - `fn execute_sleep_cycle` ⚠ fremd: `contextra-engine`, `contextra-types`
    - `fn start_consolidation_worker` ⚠ fremd: `contextra-engine`
    - `fn start_consolidation_worker_full` ⚠ fremd: `contextra-engine`
    - `fn start_consolidation_reaper` ⚠ fremd: `contextra-engine`
    - `struct ConsolidationEngine` · 10 Methoden ⚠ fremd: `contextra-vector`
- `consolidation_locks`
  - 1 struct
    - `struct ConsolidationNodesGuard` · 3 Methoden
- `context`
  - 2 struct
    - `struct ContextManager` · 8 Methoden
    - `struct SpatialFence` · 2 Methoden
- `context_compaction`
  - 1 fn, 2 struct
    - `fn cleanup_orphaned_consolidation_intents` *(re-exportiert)* ⚠ fremd: `contextra-ports`
    - `struct ContextCompactor` *(re-exportiert)* · 6 Methoden
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
  - 2 struct
    - `struct MaintenanceScheduler` · 10 Methoden ⚠ fremd: `contextra-vector`
    - `struct ActiveSessionGuard`
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
| `contextra-db` | `(crate)`, `execute_sleep_cycle`, `context::*`, `context_compaction::*`, `memory_consolidation::*`, `synthesis_phase::*`, `consolidation_executor::*`, `consolidation_locks::*`, `maintenance_scheduler::*`, `maintenance_config::*` … (+1) | – |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

**Auffälligkeiten**

- contextra-store: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?

### contextra-core

- Pfad: `crates/contextra-core` · Zeilen: 322 · Rust-Dateien: 3 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-mvcc` (normal), `contextra-ports` (normal), `contextra-types` (normal), `contextra-wire` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 11 | 0 | 1 | `error`×2, `schema`×2, `tombstone`×2, `error_dto`, `model_fingerprint`, `types`, `error_dto::ContextraErrorDto`, `model_fingerprint::ModelFingerprint` | – |
| `contextra-mvcc` | 6 | 0 | 1 | `seq_log`×2, `snapshot`×2, `tx_buffer`×2 | – |
| `contextra-ports` | 2 | 0 | 1 | `*`×2 | – |
| `contextra-wire` | 1 | 0 | 1 | `*` | – |

**Pro Komponente**

- `(crate-root)` (122 Zeilen)
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
| `contextra-vector` | `ContextraError::invalid_input`×19, `MAX_SEARCH_K`×15, `Result`×8, `ContextraError::HnswConnectivityDegraded`×4, `ContextraError::Index`×4, `DocId`×3, `DistanceMetric`×2, `SequenceLog::new`×2, `SeqLogChange::Insert`, `SeqLogChange::Delete` … (+1) | `types`×9, `traits::VectorIndex`×8, `ContextraError::InvalidInput`×6, `ContextraError::EmbeddingDimensionMismatch`×4 … (+6) |
| `contextra-store` | `MAX_SCAN_MERGE_ACCUMULATOR`×6, `ContextraError::Storage`×5, `TOMBSTONE_BIT`×4, `TxId`×4, `ResourceTracker`×2, `ContextraError::MemoryBudgetExceeded`×2, `ContextraError::Internal`×2, `Result`×2, `ContextraError`×2, `TxId::INTERNAL_BASE`×2 … (+4) | `ResourceBudget`×28, `ResourceTracker::new`×27, `StorageEngine`×2, `SnapshotRegistry::new`×2 … (+5) |
| `contextra-py` | `ipc::ScoredDocument::create`×2, `ipc::ScoredDocumentArgs`×2, `ipc::SearchResponse::create`×2, `ipc::SearchResponseArgs`×2, `FusionWeights::new`×2, `ContextraError`, `ContextraErrorDto::from`, `types::domain` | – |
| `contextra-bench` | `ContextraError::Internal`×3, `ContextraError::SnapshotUnsupportedForSignal`×2, `traits::VectorIndex`, `types`, `ContextraError::CapabilityUnsupported`, `Result` | `model_fingerprint::ModelFingerprint`, `TenantId`, `kv`, `ContextraError::InvalidInput` |
| `contextra` | `error::ContextraError`×3, `DistanceMetric`×2, `types::domain::AutoExtractionMode`, `types::domain`, `error::Result` | – |
| `contextra-simd` | `Result`×4, `ContextraError` | – |
| `contextra-checkpoint` | `SnapshotRegistry`×2 | – |

### contextra-crypto

- Pfad: `crates/contextra-crypto` · Zeilen: 15950 · Rust-Dateien: 67 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-crypto` (dev), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 25 | 14 | 5 | `TenantId`×8, `Result`×6, `ContextraError::Crypto`×3, `TenantScoped::new`×2, `TenantScoped`×2, `TxId`, `ContextraError`, `ModelFingerprint`, `ContextraError::ParseError` | – |
| `contextra-ports` | 2 | 9 | 2 | `Clock`×2 | `SystemClock`×9 |

**Pro Komponente**

- `crypto` (835 Zeilen)
  - → `contextra-types`: prod: `TenantId`×5, `TenantScoped::new`×2, `TenantScoped`×2, `TxId` · dev: `TenantId`×2
- `error` (108 Zeilen)
  - → `contextra-types`: prod: `ContextraError` · dev: `ContextraError`
- `kv_cipher` (386 Zeilen)
  - → `contextra-ports`: dev: `SystemClock`
  - → `contextra-types`: prod: `Result`×6, `ContextraError::Crypto`×3, `TenantId`, `ModelFingerprint`, `ContextraError::ParseError`
- `kv_segment` (1224 Zeilen)
  - → `contextra-types`: prod: `TenantId`×2 · dev: `TenantId`
- `kv_shredding` (629 Zeilen)
  - → `contextra-ports`: prod: `Clock`
- `revocation_log` (418 Zeilen)
  - → `contextra-ports`: prod: `Clock` · dev: `SystemClock`
- `benches/crypto_benchmarks.rs` (124 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/campaign_cov_revocation_log.rs` (33 Zeilen)
  - → `contextra-ports`: dev: `SystemClock`
- `tests/campaign_j25.rs` (422 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/campaign_j26_hypotheses.rs` (604 Zeilen)
  - → `contextra-ports`: dev: `SystemClock`
- `tests/eviction_worker_tokio_runtime_nonblocking_regression.rs` (61 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/j24closure-contextra-crypto_closure_tests.rs` (116 Zeilen)
  - → `contextra-types`: dev: `TxId`×2
- `tests/j25-contextra-crypto-crypto-shredding-audit-_wiring_test.rs` (160 Zeilen)
  - → `contextra-ports`: dev: `SystemClock`
- `tests/j25closure-contextra-crypto_symbols_test.rs` (142 Zeilen)
  - → `contextra-ports`: dev: `SystemClock`
- `tests/kv_revocation_wiring.rs` (67 Zeilen)
  - → `contextra-ports`: dev: `SystemClock`
- `tests/kv_segment_concurrency.rs` (267 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/kv_segment_integration.rs` (115 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/kv_segment_proptests.rs` (78 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/loom_kv_deferred_zeroize.rs` (157 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/proptests.rs` (125 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/revocation_and_wal_error_paths.rs` (128 Zeilen)
  - → `contextra-ports`: dev: `SystemClock`
- `tests/revocation_log_atomic_persist.rs` (125 Zeilen)
  - → `contextra-ports`: dev: `SystemClock`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `anti_tamper`
  - 1 struct
    - `struct VolatileEncryptionKey` · 3 Methoden
- `audit_chain`
  - 1 enum, 1 fn, 5 struct
    - `enum DataClass`
    - `struct AuditChainEntry` · 1 Methoden
    - `struct AuditChainAnchor`
    - `struct AuditChainHeadSignature`
    - `struct EncryptedCommitmentSalt` · 2 Methoden
    - `fn compute_record_commitment`
    - `struct AuditChain` · 13 Methoden
- `crypto`
  - 1 struct
    - `struct KeyManager` · 18 Methoden
- `deletion_proof`
  - 4 enum, 3 fn, 4 struct
    - `struct GraphRepairAttestation`
    - `fn hash_deleted_keys_length_prefixed`
    - `struct DeletionProofKeyPair` · 3 Methoden
    - `enum VerificationKey`
    - `struct LayerCleanupProof` · 3 Methoden
    - `enum DeletionLayer`
    - `enum ExcludedScope`
    - `enum DeletionScope`
    - `struct DeletionProof` · 12 Methoden
    - `fn compute_wal_delete_receipt`
    - `fn verify_wal_delete_receipt`
- `deletion_proof_typestate`
  - 3 struct
    - `struct Missing`
    - `struct Cleaned`
    - `struct DeletionProofBuilder` · 10 Methoden
- `ed25519_proof`
  - 2 enum, 2 fn, 1 struct
    - `enum DeletionProofError`
    - `struct DeletionProofKeyPair` · 4 Methoden
    - `fn sign_deletion_proof_v3`
    - `fn verify_deletion_proof_v3`
    - `enum SignatureVersion` · 1 Methoden
- `error`
  - 1 enum, 1 type
    - `type Result`
    - `enum CryptoError` · 1 Methoden
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
    - `struct KvSegmentCipher` · 7 Methoden
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
  - 1 const, 1 fn, 5 struct, 1 trait
    - `const DEFAULT_SHRED_KEY_GROUP_SIZE`
    - `struct SubKey`
    - `struct GroupKek`
    - `struct RecordDek`
    - `struct EncryptedRecordPayload`
    - `fn derive_subkey`
    - `trait GroupRevocationListener` · Methoden: on_group_revoked
    - `struct KeyRegistry` · 19 Methoden
- `revocation_log`
  - 1 enum, 1 fn, 2 struct
    - `enum RevocationTarget` · 1 Methoden
    - `struct RevocationEntry`
    - `fn compute_entry_hash`
    - `struct RevocationLog` · 7 Methoden
- `wal_completeness`
  - 1 fn
    - `fn verify_wal_chain_completeness`
- `wal_crypto`
  - 4 struct, 1 trait
    - `trait KmsProvider` · Methoden: get_key
    - `struct EncryptedWal` · 3 Methoden
    - `struct WalHmac` · 3 Methoden
    - `struct WalEntrySnapshot`
    - `struct IntegrityVerifier` · 10 Methoden
- Zusätzlich 9 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `hash_deleted_keys_length_prefixed`
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::ModelFingerprint`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-kvcache` | `KvCipher`×5, `KvSegmentCipher`×2, `CryptoError`×2, `CryptoKey`, `ModelFingerprint` | `CryptoKey::try_new` |
| `contextra-mcp` | `CryptoKey`×3, `CryptoKey::try_new`×2, `TenantIsolatedKvStore::new`×2, `KvSegmentCipher::new` | `deletion_proof::DeletionProof`, `AuditChain` |
| `contextra-engine` | `TenantIsolatedKvStore`×5, `deletion_proof` | `kv_segment::KvSegment` |
| `contextra-vector` | `wal_crypto::WalHmac::new`×6 | – |
| `contextra-store` | `wal_crypto`×2, `verify_wal_chain_completeness`, `wal_crypto::WalHmac`, `crypto::KeyManager` | – |
| `contextra-infer-candle` | `EncryptedKvLayer` | `KvSegment` |
| `contextra-bench` | – | `kv_cipher::KvSegmentCipher`, `KeyManager` |
| `contextra-db` | – | `CryptoError`, `deletion_proof`, `deletion_proof::GraphRepairAttestation`, `deletion_proof::DeletionProof` |
| `contextra-privacy` | – | `crypto::KeyManager`, `kv_cipher::ModelFingerprint` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 61 von 73)**: `VolatileEncryptionKey`, `DataClass`, `AuditChainEntry`, `AuditChainAnchor`, `AuditChainHeadSignature`, `EncryptedCommitmentSalt`, `compute_record_commitment`, `hash_deleted_keys_length_prefixed`, `DeletionProofKeyPair`, `VerificationKey`, `LayerCleanupProof`, `DeletionLayer`, `ExcludedScope`, `DeletionScope`, `compute_wal_delete_receipt`, `verify_wal_delete_receipt`, `Missing`, `Cleaned`, `DeletionProofBuilder`, `DeletionProofError`, `DeletionProofKeyPair`, `sign_deletion_proof_v3`, `verify_deletion_proof_v3`, `SignatureVersion`, `Result`, `KDF_HEADER_MAGIC`, `KDF_HEADER_VERSION_1`, `KDF_ID_ARGON2ID`, `MIN_SALT_LEN`, `MIN_M_COST_KIB` …

**Auffälligkeiten**

- contextra-crypto: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-db

- Pfad: `crates/contextra-db` · Zeilen: 12443 · Rust-Dateien: 63 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-vector` (dev), `contextra-adapt` (normal), `contextra-checkpoint` (normal), `contextra-cognition` (normal), `contextra-crypto` (normal), `contextra-engine` (normal), `contextra-graph` (normal), `contextra-ports` (normal), `contextra-rank` (normal), `contextra-store` (normal), `contextra-sys` (normal), `contextra-text` (normal), `contextra-types` (normal), `contextra-vector` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-cognition` | 11 | 0 | 1 | `(crate)`, `execute_sleep_cycle`, `context::*`, `context_compaction::*`, `memory_consolidation::*`, `synthesis_phase::*`, `consolidation_executor::*`, `consolidation_locks::*`, `maintenance_scheduler::*`, `maintenance_config::*` … (+1) | – |
| `contextra-engine` | 6 | 1 | 1 | `(crate)`, `MetadataFilter`, `collection::maintenance::PercolationResult`, `SandboxBridge`, `background_workers::*`, `fusion` | `DeletionLayer` |
| `contextra-adapt` | 5 | 2 | 3 | `homeostat::RerankDeadline`, `homeostat::RerankPidController`, `decay_controller::*`, `pid_latency_controller::*`, `PidController` | `PidController::new` |
| `contextra-types` | 2 | 47 | 2 | `DistanceMetric`, `MAX_SEARCH_K` | `Result`×7, `TxId`×6, `DocId`×4, `FusionWeights`×3, `TxId::new`×3 … (+17) |
| `contextra-checkpoint` | 1 | 4 | 1 | `(crate)` | `PersistentCheckpointStore`×2, `clear_all_orphaned_checkpoints`, `get_orphaned_checkpoints` |
| `contextra-graph` | 1 | 15 | 1 | `percolation::PercolationConfig` | `CsrGraph`×10, `path_rag::PathRAGEngine`×2, `csr::CsrGraph`×2, `PathRAGEngine` |
| `contextra-ports` | 1 | 13 | 1 | `QueryRewriter` | `GraphIndex`×4, `StorageEngine`×3, `BoxFuture`×2, `LlmTextGenerator`, `EmbeddingError` … (+2) |
| `contextra-sys` | 1 | 0 | 1 | `LockedRegions` | – |
| `contextra-crypto` | 0 | 4 | 0 | – | `CryptoError`, `deletion_proof`, `deletion_proof::GraphRepairAttestation`, `deletion_proof::DeletionProof` |
| `contextra-rank` | 0 | 1 | 0 | – | `IsotonicCalibrator::new` |
| `contextra-store` | 0 | 12 | 0 | – | `LsmConfig`×5, `LsmStorage::new`×4, `LsmStorage`×2, `lsm` |
| `contextra-text` | 0 | 11 | 0 | – | `Language::English`×7, `Language`×2, `bm25::score_term`, `inverted::InvertedIndex` |
| `contextra-vector` | 0 | 6 | 0 | – | `HnswConfig`×3, `HnswIndex`×2, `HnswIndex::try_new` |

**Pro Komponente**

- `(crate-root)` (775 Zeilen)
  - → `contextra-adapt`: prod: `decay_controller::*`, `pid_latency_controller::*` · dev: `PidController::new`
  - → `contextra-checkpoint`: prod: `(crate)`
  - → `contextra-cognition`: prod: `(crate)`, `execute_sleep_cycle`, `context::*`, `context_compaction::*`, `memory_consolidation::*`, `synthesis_phase::*`, `consolidation_executor::*`, `consolidation_locks::*`, `maintenance_scheduler::*`, `maintenance_config::*` … (+1)
  - → `contextra-engine`: prod: `(crate)`, `MetadataFilter`, `collection::maintenance::PercolationResult`, `SandboxBridge`, `background_workers::*`, `fusion`
  - → `contextra-graph`: prod: `percolation::PercolationConfig`
  - → `contextra-rank`: dev: `IsotonicCalibrator::new`
  - → `contextra-types`: prod: `DistanceMetric` · dev: `TenantId`, `Result`, `ContextraError::InvalidInput`
- `homeostat` (6 Zeilen)
  - → `contextra-adapt`: prod: `homeostat::RerankDeadline`, `homeostat::RerankPidController`
- `multistep` (620 Zeilen)
  - → `contextra-adapt`: prod: `PidController`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: prod: `QueryRewriter` · dev: `BoxFuture`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: prod: `MAX_SEARCH_K` · dev: `ContextraError::Internal`
- `volatile_vault` (450 Zeilen)
  - → `contextra-sys`: prod: `LockedRegions`
- `examples/collections.rs` (107 Zeilen)
  - → `contextra-types`: dev: `Result`, `TenantId::new`
- `examples/hybrid_search.rs` (85 Zeilen)
  - → `contextra-types`: dev: `Result`
- `examples/quickstart.rs` (56 Zeilen)
  - → `contextra-types`: dev: `Result`
- `examples/verify_deletion_proof_external.rs` (228 Zeilen)
  - → `contextra-crypto`: dev: `CryptoError`, `deletion_proof`, `deletion_proof::GraphRepairAttestation`
  - → `contextra-types`: dev: `Result`×2, `DocId::new`
- `tests/atomic_commit.rs` (92 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/auto_community_detection_test.rs` (117 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/cascade_invalidation_integration.rs` (133 Zeilen)
  - → `contextra-graph`: dev: `path_rag::PathRAGEngine`
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/cascade_invalidation_test.rs` (110 Zeilen)
  - → `contextra-graph`: dev: `path_rag::PathRAGEngine`
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `LinkRelation`
- `tests/checkpoint_layer_bounds.rs` (133 Zeilen)
  - → `contextra-checkpoint`: dev: `PersistentCheckpointStore`
  - → `contextra-store`: dev: `LsmConfig`×2, `LsmStorage::new`×2
  - → `contextra-types`: dev: `TxId`
- `tests/checkpoint_stress.rs` (74 Zeilen)
  - → `contextra-checkpoint`: dev: `PersistentCheckpointStore`
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/compound_split_recall_impact.rs` (400 Zeilen)
  - → `contextra-types`: dev: `FusionWeights`
- `tests/compound_split_recall_impact_test.rs` (444 Zeilen)
  - → `contextra-types`: dev: `FusionWeights`
- `tests/consolidation_double_trigger_test.rs` (133 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/consolidation_integration_test.rs` (226 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`, `LlmTextGenerator`
  - → `contextra-types`: dev: `DocId`, `Result`
- `tests/context_compaction_retry_test.rs` (187 Zeilen)
  - → `contextra-store`: dev: `LsmConfig`, `LsmStorage::new`
  - → `contextra-types`: dev: `TxId::new`×2
- `tests/cross_domain_chaos_matrix_test.rs` (259 Zeilen)
  - → `contextra-graph`: dev: `csr::CsrGraph`
  - → `contextra-ports`: dev: `EmbeddingError`
  - → `contextra-store`: dev: `lsm`
  - → `contextra-text`: dev: `Language`
  - → `contextra-types`: dev: `DocId`
- `tests/deletion_proof_integration.rs` (270 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/diskann_collection_fallback.rs` (91 Zeilen)
  - → `contextra-graph`: dev: `csr::CsrGraph`
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-text`: dev: `Language`
  - → `contextra-types`: dev: `TxId::new`
- `tests/docid_128_consolidation.rs` (34 Zeilen)
  - → `contextra-types`: dev: `DocId`, `TxId`
- `tests/drop_collection_proof_vs_files.rs` (81 Zeilen)
  - → `contextra-engine`: dev: `DeletionLayer`
- `tests/e2e_compliance_scenario.rs` (459 Zeilen)
  - → `contextra-crypto`: dev: `deletion_proof::DeletionProof`
- `tests/embedding_dimension_switch.rs` (85 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/fault_injection_2pc.rs` (717 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
- `tests/graph_entity_delete_test.rs` (149 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/graph_hybrid_search.rs` (287 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `ContextraError::SnapshotUnsupportedForSignal`×2, `GraphTraversalStrategy`, `EntityId::from_key`, `PprAlgorithm::Auto` … (+1)
- `tests/hnsw_delete_and_backfill.rs` (195 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
- `tests/instance_orphan_registry_test.rs` (123 Zeilen)
  - → `contextra-checkpoint`: dev: `clear_all_orphaned_checkpoints`, `get_orphaned_checkpoints`
  - → `contextra-types`: dev: `TxId`
- `tests/integration_temporal_filter.rs` (192 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-types`: dev: `TxId`
- `tests/multistep_query_rewriter_modalities.rs` (186 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: dev: `ContextraError::Internal`
- `tests/pid_pool_production_caller.rs` (106 Zeilen)
  - → `contextra-adapt`: dev: `PidController`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: dev: `Clock`
  - → `contextra-text`: dev: `Language::English`
- `tests/proptest_search_invariants.rs` (207 Zeilen)
  - → `contextra-text`: dev: `bm25::score_term`
- `tests/search_result_bound.rs` (205 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/snapshot_api.rs` (163 Zeilen)
  - → `contextra-types`: dev: `ContextraError::CapabilityUnsupported`×2, `EntityId::new`
  - → `contextra-vector`: dev: `HnswIndex::try_new`, `HnswConfig`
- `tests/split_brain_vector_orphan_test.rs` (501 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/sprint2_acid_tests.rs` (345 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-store`: dev: `LsmStorage::new`
  - → `contextra-types`: dev: `TenantId`, `TxId`
- `tests/stabilization_sprint_tests.rs` (135 Zeilen)
  - → `contextra-types`: dev: `FusionWeights`
- `tests/supersedes_cascading_tombstone_test.rs` (161 Zeilen)
  - → `contextra-graph`: dev: `PathRAGEngine`
- `tests/text_integration.rs` (96 Zeilen)
  - → `contextra-text`: dev: `inverted::InvertedIndex`
- `tests/truncation_filter_recall_test.rs` (270 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-types`: dev: `MemoryType`
- `tests/zettelkasten_links_test.rs` (262 Zeilen)
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
- ⚠ Re-Exporte **fremder Crates**: `contextra_adapt::homeostat::RerankDeadline`, `contextra_adapt::homeostat::RerankPidController`, `contextra_cognition`, `contextra_engine`, `contextra_engine::MetadataFilter`, `contextra_engine::collection::maintenance::PercolationResult`, `contextra_engine::SandboxBridge`, `contextra_cognition::execute_sleep_cycle`, `contextra_cognition::context::*`, `contextra_cognition::context_compaction::*`, `contextra_cognition::memory_consolidation::*`, `contextra_cognition::synthesis_phase::*`, `contextra_cognition::consolidation_executor::*`, `contextra_cognition::consolidation_locks::*`, `contextra_cognition::maintenance_scheduler::*`, `contextra_cognition::maintenance_config::*`, `contextra_engine::background_workers::*`, `contextra_cognition::start_consolidation_reaper`, `contextra_adapt::decay_controller::*`, `contextra_engine::fusion`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-py` | `Collection`, `Document`, `SearchResult`, `Contextra`, `DistanceMetric::Cosine`, `DistanceMetric::Euclidean`, `DistanceMetric::DotProduct` | – |
| `contextra-agent` | `Collection`×2, `Contextra`×2, `volatile_vault` | `Contextra::open_with_config`×18, `ContextraConfig::default`×15, `volatile_vault::VaultConfig`×5, `ContextraConfig`×2 … (+1) |
| `contextra` | `QueryRewriter` | `EmbeddingBackend`, `ContextraConfig` |
| `contextra-bench` | `Collection` | – |
| `contextra-router` | – | `ContextraConfig`×5, `Contextra::open_with_config`×5, `Collection`, `context::ContextManager::new` … (+1) |

**Nach außen ungenutzte öffentliche Items (Heuristik, 12 von 13)**: `DEFAULT_MAX_SUB_QUERIES`, `DEFAULT_RERANK_POOL_MULTIPLIER`, `MultiStepConfig`, `MultiStepResult`, `MultiStepEngine`, `SignalModality`, `VaultChunk`, `PurgeReceipt`, `CommitReceipt`, `VaultError`, `VolatileContextVault`, `VaultChunkMetadata`

**Auffälligkeiten**

- contextra-crypto: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-rank: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-store: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-text: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-vector: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?

### contextra-engine

- Pfad: `crates/contextra-engine` · Zeilen: 24027 · Rust-Dateien: 117 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-core` (dev), `contextra-crypto` (dev), `contextra-testkit` (dev), `contextra-vector` (dev), `contextra-adapt` (normal), `contextra-checkpoint` (normal), `contextra-crypto` (normal/optional via encryption-at-rest), `contextra-graph` (normal), `contextra-infer-candle` (normal), `contextra-kvcache` (normal), `contextra-mvcc` (normal), `contextra-ports` (normal), `contextra-rank` (normal), `contextra-sandbox` (normal/optional via sandbox), `contextra-store` (normal), `contextra-sys` (normal), `contextra-text` (normal), `contextra-types` (normal), `contextra-vector` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 252 | 172 | 33 | `ContextraError::invalid_input`×50, `Result`×23, `ContextraError::Internal`×19, `EntityId`×14, `MAX_SEARCH_K`×12, `ContextraError::Serialization`×8, `ImportanceScore::new`×8, `domain::MemoryLink`×7, `HybridQuery`×7, `EntityId::from_key`×7 … (+42) | `ContextraError::from`×10, `domain::LinkRelation::Elaborates`×2, `ContextraError::HnswConnectivityDegraded`, `FusionWeights::new`, `PprConfig::default` … (+7) |
| `contextra-ports` | 33 | 21 | 11 | `MetricsSink`×9, `StorageEngine`×5, `KvLifecycleHooks`×4, `VectorIndexStats`×2, `DriftStatusProvider`×2, `VectorIndex`, `Reranker`, `license`, `SystemClock::new`, `NoopMetricsSink` … (+6) | `Clock` |
| `contextra-store` | 30 | 106 | 7 | `LsmStorage`×6, `tenant_codec::TenantScopedStorage`×5, `kv::KvSegmentManager`×4, `tenant_codec::TenantScopedStorage::new`×4, `SystemPressure`×3, `kv::KvSegmentPayload`×2, `tenant_codec::TenantKeyCodec`×2, `PressureLevel::Critical`, `LsmConfig`, `lsm::DurabilityMode` … (+1) | `lsm`×10, `compaction`×2, `sstable`×2 |
| `contextra-graph` | 20 | 76 | 8 | `hyperedge::HyperEdgeId`×2, `percolation::PercolationConfig`×2, `CsrGraph::load_from_storage`×2, `hyperedge::HyperEdge`×2, `cascade_invalidate_edges_for_superseded_doc`, `percolation::compute_percolation_health`, `percolation::should_trigger_rebonding`, `percolation::find_rebonding_candidates`, `CsrGraph`, `hyperedge::RoleId::new` … (+6) | `csr::CsrGraph`×15, `csr`, `CsrGraph::new` |
| `contextra-crypto` | 6 | 2 | 2 | `TenantIsolatedKvStore`×5, `deletion_proof` | `kv_segment::KvSegment` |
| `contextra-rank` | 6 | 0 | 3 | `IsotonicCalibrator`×2, `fusion`, `fusion::fuse_search_results_with_strategy`, `fusion::fuse_search_results_with_signal_strategies`, `fusion::fuse_signals` | – |
| `contextra-adapt` | 5 | 2 | 4 | `PidController`×4, `decay_controller` | `PidController::default`×2 |
| `contextra-vector` | 5 | 119 | 5 | `hnsw::HnswIndex`×2, `HnswIndex`×2, `acorn::FilteredIndex` | `HnswConfig`×58, `hnsw::HnswConfig::default`, `hnsw::HnswIndex::try_new`, `acorn`, `HnswIndex::try_new` |
| `contextra-checkpoint` | 4 | 0 | 3 | `InstanceOrphanRegistry`×2, `InstanceOrphanRegistry::new`, `(crate)` | – |
| `contextra-kvcache` | 3 | 2 | 2 | `TenantIsolatedKvStore::new`, `EvictionWorker`, `AttentionScoreSource` | `TenantIsolatedKvStore`×2 |
| `contextra-text` | 3 | 69 | 2 | `Language`×2, `inverted::InvertedIndex` | `Language::English`×64, `Language::German`×4 |
| `contextra-infer-onnx` | 2 | 0 | 1 | `ensure_onnx_model_download`, `OnnxEmbedder::load` | – |
| `contextra-infer-candle` | 1 | 0 | 1 | `CandleAttentionExporter` | – |
| `contextra-mvcc` | 1 | 3 | 1 | `tx_buffer::TxBuffer` | `tx_buffer::IndexOp` |
| `contextra-testkit` | 0 | 1 | 0 | – | `InMemoryStorageEngine` |

**Pro Komponente**

- `(crate-root)` (718 Zeilen)
  - → `contextra-adapt`: prod: `PidController`
  - → `contextra-checkpoint`: prod: `(crate)`, `InstanceOrphanRegistry`
  - → `contextra-crypto`: prod: `deletion_proof`
  - → `contextra-graph`: prod: `percolation::PercolationConfig`
  - → `contextra-kvcache`: prod: `EvictionWorker`, `AttentionScoreSource`
  - → `contextra-ports`: prod: `MetricsSink`×3, `KvLifecycleHooks`×3, `license::LicenseGate`, `BoxFuture`, `StorageEngine`, `TextEmbeddingEngine`, `VectorDeleteMode`, `DriftStatusProvider`, `VectorIndexStats`, `StorageStats`
  - → `contextra-rank`: prod: `IsotonicCalibrator`
  - → `contextra-store`: prod: `lsm::DurabilityMode`, `LsmStorage`, `tenant_codec::TenantScopedStorage`
  - → `contextra-text`: prod: `Language`
  - → `contextra-types`: prod: `Result`×13, `ContextraError::Internal`×10, `DistanceMetric`×2, `DocId`×2, `ContextraError::InvalidInput`×2, `ContextraError::Serialization`×2, `DistanceMetric::Cosine` · dev: `DocId::new`
- `background_workers` (1075 Zeilen)
  - → `contextra-graph`: prod: `hyperedge::HyperEdgeId` · dev: `CsrGraph`×8, `hyperedge::HyperEdgeId`
  - → `contextra-mvcc`: prod: `tx_buffer::TxBuffer` · dev: `tx_buffer::TxBuffer`×2, `tx_buffer::IndexOp`
  - → `contextra-ports`: prod: `StorageEngine`×2, `VectorIndex`
  - → `contextra-store`: dev: `LsmStorage`×8, `LsmConfig`×8
  - → `contextra-text`: dev: `Language::English`×8
  - → `contextra-types`: prod: `Result`×4 · dev: `Result`×2, `ContextraError::HnswConnectivityDegraded`
  - → `contextra-vector`: prod: `hnsw::HnswIndex`×2 · dev: `HnswIndex`×8, `HnswConfig`×8, `hnsw::HnswConfig::default`, `hnsw::HnswIndex::try_new`
- `collection` (10912 Zeilen)
  - → `contextra-adapt`: prod: `PidController`×2 · dev: `PidController::default`×2
  - → `contextra-crypto`: prod: `TenantIsolatedKvStore`×5
  - → `contextra-graph`: prod: `cascade_invalidate_edges_for_superseded_doc`, `percolation::PercolationConfig`, `percolation::compute_percolation_health`, `percolation::should_trigger_rebonding`, `percolation::find_rebonding_candidates`, `CsrGraph`, `hyperedge::HyperEdgeId`, `hyperedge::RoleId::new`, `hyperedge::HyperEdgeId::new`, `hyperedge::RoleBinding` … (+4) · dev: `CsrGraph`×40, `csr::CsrGraph`×15, `csr`, `percolation::PercolationConfig`
  - → `contextra-ports`: prod: `VectorIndexStats`, `StorageEngine` · dev: `StorageStats`×5, `StorageEngine`×3, `TextEmbeddingEngine`, `BoxFuture`
  - → `contextra-store`: prod: `kv::KvSegmentManager`×4, `SystemPressure`×3, `LsmStorage`×2, `tenant_codec::TenantScopedStorage`×2, `kv::KvSegmentPayload`×2, `tenant_codec::TenantKeyCodec`×2, `PressureLevel::Critical` · dev: `LsmStorage`×36, `LsmConfig`×35, `lsm`×10
  - → `contextra-text`: prod: `inverted::InvertedIndex`, `Language` · dev: `Language::English`×50, `Language::German`×4, `Language`
  - → `contextra-types`: prod: `ContextraError::invalid_input`×37, `MAX_SEARCH_K`×12, `EntityId`×10, `ImportanceScore::new`×8, `ContextraError::Internal`×7, `domain::MemoryLink`×7, `EntityId::from_key`×7, `Entity::new`×6, `DecayFunction::None`×6, `HybridQuery`×6 … (+36) · dev: `Result`×25, `ContextraError::InvalidInput`×15, `ContextraError::from`×10, `MAX_SEARCH_K`×4 … (+27)
  - → `contextra-vector`: prod: `HnswIndex`, `acorn::FilteredIndex` · dev: `HnswIndex`×44, `HnswConfig`×44
- `contextra_impl` (1330 Zeilen)
  - → `contextra-adapt`: prod: `PidController`
  - → `contextra-checkpoint`: prod: `InstanceOrphanRegistry`, `InstanceOrphanRegistry::new`
  - → `contextra-graph`: prod: `CsrGraph::load_from_storage`×2
  - → `contextra-infer-onnx`: prod: `ensure_onnx_model_download`, `OnnxEmbedder::load`
  - → `contextra-kvcache`: prod: `TenantIsolatedKvStore::new`
  - → `contextra-ports`: prod: `MetricsSink`×3, `Reranker`, `license`, `SystemClock::new`, `NoopMetricsSink`, `KvLifecycleHooks`, `DriftStatusProvider`
  - → `contextra-rank`: prod: `IsotonicCalibrator`
  - → `contextra-store`: prod: `tenant_codec::TenantScopedStorage::new`×4, `LsmStorage`×3, `tenant_codec::TenantScopedStorage`×2, `LsmConfig`
  - → `contextra-types`: prod: `ContextraError::invalid_input`×9, `EntityId`×4, `ContextraError::Internal`×2, `FusionWeights`×2, `ContextraError::PolicyViolation`×2, `ContextraError::Storage`×2, `ContextraError::Transaction`, `MemoryType`, `GraphTraversalStrategy`, `HybridQuery` … (+1) · dev: `Result`×3
- `decay_controller` (7 Zeilen)
  - → `contextra-adapt`: prod: `decay_controller`
- `export` (337 Zeilen)
  - → `contextra-types`: prod: `TxId::new` · dev: `DocId::from_key`×4
- `extraction` (517 Zeilen)
  - → `contextra-ports`: prod: `LlmTextGenerator`
- `filter` (290 Zeilen)
  - → `contextra-types`: prod: `ContextraError::invalid_input`×2, `ContextraError` · dev: `ContextraError::InvalidInput`×2
- `fusion` (54 Zeilen)
  - → `contextra-rank`: prod: `fusion`, `fusion::fuse_search_results_with_strategy`, `fusion::fuse_search_results_with_signal_strategies`, `fusion::fuse_signals`
  - → `contextra-types`: prod: `FusionStrategy`, `SignalFusionStrategies`
- `import` (114 Zeilen)
  - → `contextra-types`: prod: `ContextraError::Serialization`×2, `ContextraError::invalid_input`×2, `Result`
- `kv_cache_integration` (72 Zeilen)
  - → `contextra-infer-candle`: prod: `CandleAttentionExporter`
- `merge_operator_adapter` (175 Zeilen)
  - → `contextra-store`: prod: `MergeOperator`
- `metrics_names` (50 Zeilen)
  - → `contextra-ports`: prod: `MetricsSink`×3
- `temporal_filter` (311 Zeilen)
  - → `contextra-types`: prod: `TxId`
- `transaction` (1350 Zeilen)
  - → `contextra-crypto`: dev: `kv_segment::KvSegment`, `TenantIsolatedKvStore`
  - → `contextra-graph`: prod: `hyperedge::HyperEdge`×2 · dev: `CsrGraph`
  - → `contextra-ports`: prod: `StorageEngine`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: prod: `TenantId::try_new` · dev: `DocId::new`×4, `DocId`×2, `EntityId`, `TenantId::try_new`
  - → `contextra-vector`: prod: `HnswIndex` · dev: `HnswIndex`, `HnswConfig`
- `tests/auto_extraction_runtime_mode.rs` (225 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/auto_extraction_wiring.rs` (277 Zeilen)
  - → `contextra-types`: dev: `EntityId::from_key`×5
- `tests/auto_triple_extraction.rs` (111 Zeilen)
  - → `contextra-types`: dev: `EntityId::from_key`×2
- `tests/chunker_robustness.rs` (58 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/commit_crash_consistency.rs` (287 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/crud_error_robustness.rs` (103 Zeilen)
  - → `contextra-types`: dev: `ContextraError`, `Result`
- `tests/decay_and_expiry.rs` (147 Zeilen)
  - → `contextra-ports`: dev: `Clock`
  - → `contextra-types`: dev: `Result`×2, `MemoryType`
- `tests/engine_commit_crash_points.rs` (214 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/engine_recovery_crash_windows.rs` (362 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/export_import_roundtrip.rs` (85 Zeilen)
  - → `contextra-types`: dev: `ContextraError`, `Result`
- `tests/filter_metadata_eval.rs` (99 Zeilen)
  - → `contextra-types`: dev: `FilterExpr`, `Result`
- `tests/hybrid_adaptive_overfetch.rs` (123 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
- `tests/hybrid_pathrag_snapshot.rs` (103 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
- `tests/hybrid_query_fusion.rs` (88 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/hybrid_signal_failure_semantics.rs` (99 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
- `tests/invariant_determinism.rs` (74 Zeilen)
  - → `contextra-types`: dev: `Result`×2
- `tests/invariant_durability.rs` (100 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/invariant_isolation.rs` (122 Zeilen)
  - → `contextra-types`: dev: `Result`, `ContextraError::Internal`
- `tests/invariant_tenant_isolation.rs` (128 Zeilen)
  - → `contextra-types`: dev: `Result`×2
- `tests/j03_contextra_engine_collection_crud_mainten_wiring_tests.rs` (116 Zeilen)
  - → `contextra-kvcache`: dev: `TenantIsolatedKvStore`
  - → `contextra-ports`: dev: `LlmTextGenerator`, `BoxFuture`
  - → `contextra-types`: dev: `DocId`, `Result`
- `tests/j03closure-contextra-engine_wiring_test.rs` (119 Zeilen)
  - → `contextra-kvcache`: dev: `TenantIsolatedKvStore`
  - → `contextra-ports`: dev: `LlmTextGenerator`, `BoxFuture`
  - → `contextra-types`: dev: `DocId`, `Result`
- `tests/kv_cache_integration_test.rs` (110 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/kv_hooks_production_wiring.rs` (88 Zeilen)
  - → `contextra-ports`: dev: `KvLifecycleHooks`
  - → `contextra-types`: dev: `Result`×3
- `tests/kv_lifecycle_hooks_rollback.rs` (77 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: dev: `KvLifecycleHooks`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/metrics_instrumentation.rs` (122 Zeilen)
  - → `contextra-types`: dev: `Result`×4
- `tests/multi_index_transaction.rs` (99 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/no_crypto_proof_stub.rs` (136 Zeilen)
  - → `contextra-types`: dev: `ContextraError::Internal`
- `tests/orphan_worker_started_on_open.rs` (82 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/query_builder_scope_acorn.rs` (238 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-vector`: dev: `acorn`
- `tests/query_builder_time_travel.rs` (75 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/rule_based_extraction_determinism.rs` (165 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/ssi_read_path_coverage.rs` (158 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph::new`
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `InMemoryStorageEngine`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: dev: `domain::LinkRelation::Elaborates`, `DistanceMetric::Cosine`
  - → `contextra-vector`: dev: `HnswIndex::try_new`, `HnswConfig`
- `tests/tenant_collection_handle_sharing.rs` (61 Zeilen)
  - → `contextra-types`: dev: `Result`×2, `TenantId`
- `tests/tenant_index_persistence_reload.rs` (85 Zeilen)
  - → `contextra-types`: dev: `Result`×2, `TenantId`
- `tests/tenant_policy_required.rs` (50 Zeilen)
  - → `contextra-types`: dev: `Result`×2
- `tests/tenant_wrapper_coverage.rs` (210 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `Result`
- `tests/wasm_merge_operator.rs` (339 Zeilen)
  - → `contextra-store`: dev: `compaction`×2, `sstable`×2, `MergeOperator`
  - → `contextra-types`: dev: `ContextraError`
- `tests/write_skew_via_engine_api.rs` (198 Zeilen)
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
    - `struct Contextra` · 6 Methoden
    - `trait SandboxBridge` · Methoden: db_search, db_insert, db_get
- `background_workers`
  - 1 fn
    - `fn start_decay_cleanup_worker` *(re-exportiert)* ⚠ fremd: `contextra-ports`
- `chunker`
  - 3 fn, 2 struct
    - `fn estimate_tokens`
    - `struct ChunkerConfig`
    - `struct MarkdownChunker` · 3 Methoden
    - `fn chunk_text_with_overlap`
    - `fn chunk_text`
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
    - `struct Collection` · 48 Methoden ⚠ fremd: `contextra-store`, `contextra-vector`
    - `const METRIC_SEARCH_LATENCY_VECTOR`
    - `const METRIC_SEARCH_LATENCY_TEXT`
    - `const METRIC_SEARCH_LATENCY_GRAPH`
    - `struct ScopeConstraint` *(re-exportiert)* · 2 Methoden ⚠ fremd: `contextra-types`
    - `enum SearchStrategy` *(re-exportiert)* · 3 Methoden ⚠ fremd: `contextra-types`
    - `struct SignalWeights` *(re-exportiert)* · 1 Methoden
- `export`
  - 1 const, 4 struct
    - `const SCHEMA_VERSION_V1`
    - `struct ExportDocumentV1`
    - `struct ExportCollectionV1`
    - `struct ExportMemoryV1`
    - `struct ExportRelationV1`
- `extraction`
  - 2 const, 4 fn, 4 struct, 1 trait
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
- `kv_cache_integration`
  - 3 fn, 1 struct
    - `struct AccessCounterAttentionExporter` · 3 Methoden
    - `fn build_eviction_worker`
    - `fn build_eviction_worker_with_exporter`
    - `fn start_kv_cache_eviction`
- `merge_operator_adapter`
  - 1 struct
    - `struct WasmMergeOperator` · 2 Methoden
- `metrics_names`
  - 7 const, 3 fn
    - `const COMMIT_LATENCY`
    - `const REBUILD_DURATION`
    - `const BACKPRESSURE_LEVEL`
    - `const CHECKPOINT_DURATION`
    - `const SEARCH_LATENCY_VECTOR`
    - `const SEARCH_LATENCY_TEXT`
    - `const SEARCH_LATENCY_GRAPH`
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
  - 1 enum, 1 fn, 1 struct
    - `fn cleanup_orphaned_consolidation_intents` *(re-exportiert)* ⚠ fremd: `contextra-ports`
    - `struct DbTransaction` *(re-exportiert)* · 10 Methoden ⚠ fremd: `contextra-vector`
  - … 1 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 40 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::AutoExtractionMode`, `contextra_ports::TextEmbeddingEngine`, `contextra_ports::VectorDeleteMode`, `contextra_store::lsm::DurabilityMode`, `contextra_checkpoint`, `contextra_graph::percolation::PercolationConfig`, `contextra_text::Language`, `contextra_ports::DriftStatusProvider`, `contextra_types::DistanceMetric`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-cognition` | `collection`×4, `collection::Collection`×3, `decay_controller::AdaptiveDecayController`×2, `transaction::CommitIntent`×2, `ProvenanceRecord`, `decay_controller::DecayControllerConfig` | `Language::English`×10, `SearchResult`×2, `collection::Collection::new`×2, `Language::German` |
| `contextra-db` | `(crate)`, `MetadataFilter`, `collection::maintenance::PercolationResult`, `SandboxBridge`, `background_workers::*`, `fusion` | `DeletionLayer` |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

**Auffälligkeiten**

- contextra-core: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-sandbox: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-sys: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-infer-onnx: genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler)

### contextra-graph

- Pfad: `crates/contextra-graph` · Zeilen: 25811 · Rust-Dateien: 109 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-store` (dev), `contextra-testkit` (dev), `contextra-adapt` (normal), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 39 | 53 | 25 | `EntityId`×14, `ContextraError::InvalidInput`×5, `ContextraError`×4, `PprConfig`×3, `Result`×3, `Edge::new`×2, `TxId`×2, `ContextraError::Internal`×2, `Edge`, `MemoryLink` … (+2) | `Entity::new`×10, `TlHfdParams::default`×4, `PprAlgorithm::Auto`×4, `TlHfdParams` |
| `contextra-ports` | 20 | 40 | 8 | `GraphIndex`×3, `StorageEngine`×2, `BoxFuture`×2, `graph::GraphMutationError::Internal`×2, `graph::RoleBinding`, `graph::HyperEdgeId`, `graph::GraphMutationError`, `graph::GraphMutationError::LockAcquisitionTimeout`, `graph::GraphMutationError::RoleBindingInvalid`, `graph::GraphMutationError::EpochReclamationPending` … (+5) | `Result`×2, `graph`, `graph_index::GraphIndex` |
| `contextra-adapt` | 1 | 0 | 1 | `TracingShadowSink` | – |
| `contextra-store` | 0 | 2 | 0 | – | `LsmConfig`, `LsmStorage::new` |

**Pro Komponente**

- `(crate-root)` (122 Zeilen)
  - → `contextra-ports`: prod: `GraphIndex`, `BoxFuture`
  - → `contextra-types`: prod: `TxId`, `Result`
- `apprh` (911 Zeilen)
  - → `contextra-types`: prod: `EntityId`×4, `ContextraError`×2, `ContextraError::InvalidInput`×2
- `cascade` (907 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: prod: `ContextraError::InvalidInput`×2 · dev: `Entity::new`×10
- `community` (1269 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `csr` (5984 Zeilen)
  - → `contextra-ports`: prod: `graph::GraphMutationError::Internal`×2, `StorageEngine`, `BoxFuture`, `graph::RoleBinding`, `graph::HyperEdgeId`, `graph::GraphMutationError`, `graph::GraphMutationError::LockAcquisitionTimeout`, `graph::GraphMutationError::RoleBindingInvalid`, `graph::GraphMutationError::EpochReclamationPending`, `graph::GraphMutationError::InsufficientParticipants` … (+3) · dev: `GraphIndex`×2
  - → `contextra-types`: prod: `PprConfig`×3, `Edge::new`×2, `Edge`, `MemoryLink`, `LinkRelation::Supersedes`, `EntityId`, `TxId` · dev: `Result`
- `edge_reinforcement` (159 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `edge_reinforcement_buffer` (183 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: prod: `EntityId` · dev: `EntityId`
- `entity_extraction` (271 Zeilen)
  - → `contextra-types`: prod: `ContextraError::Internal`
- `error` (90 Zeilen)
  - → `contextra-types`: prod: `ContextraError`
- `hyperedge_suggest` (155 Zeilen)
  - → `contextra-ports`: prod: `LlmTextGenerator`
- `path_rag` (1371 Zeilen)
  - → `contextra-types`: prod: `EntityId`×2, `Result`×2, `DocId`
- `percolation` (267 Zeilen)
  - → `contextra-ports`: prod: `GraphIndex` · dev: `GraphIndex`
  - → `contextra-types`: prod: `EntityId`
- `ppr` (3173 Zeilen)
  - → `contextra-adapt`: prod: `TracingShadowSink`
  - → `contextra-ports`: dev: `StorageEngine`×3, `GraphIndex`×2
  - → `contextra-store`: dev: `LsmConfig`, `LsmStorage::new`
  - → `contextra-types`: prod: `EntityId` · dev: `TlHfdParams::default`×4, `TlHfdParams`
- `ppr_stream` (137 Zeilen)
  - → `contextra-ports`: prod: `GraphIndex`
  - → `contextra-types`: prod: `ContextraError::Internal`
- `provenance` (229 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `session_dag` (854 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine`
- `tl_hfd` (698 Zeilen)
  - → `contextra-types`: prod: `EntityId`×4, `ContextraError`, `ContextraError::InvalidInput`
- `benches/csr_traversal_bench.rs` (193 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `Result`×4, `Edge::new`×2
- `tests/apprh_determinism_permutation.rs` (128 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/apprh_gate_monitor.rs` (284 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/apprh_selector_shadow_to_production.rs` (215 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/apprh_tests.rs` (210 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/budgeted_k_path_hub_entity.rs` (144 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/community_hyperedges_included_flag.rs` (122 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/csr_benchmark.rs` (157 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/csr_complexity_bench.rs` (146 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/dangling_nodes_audit_test.rs` (238 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `PprAlgorithm::Auto`×3
- `tests/diffusion_shadow_gate_thresholds.rs` (85 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/entity_extraction_tests.rs` (74 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/graph_collection_mutation_trait_test.rs` (63 Zeilen)
  - → `contextra-ports`: dev: `graph`
- `tests/hub_node_benchmark.rs` (95 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `Edge::new`×2
- `tests/hyperedge_cascade_crash_recovery.rs` (162 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/hyperedge_cascade_queue_persistence.rs` (73 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/hyperedge_memory_budget.rs` (90 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/hyperedge_payload_sharing.rs` (32 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/hyperedge_persistence_survives_restart.rs` (98 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/hyperedge_suggest.rs` (152 Zeilen)
  - → `contextra-ports`: dev: `Result`
- `tests/hyperedge_suggest_llm_test.rs` (110 Zeilen)
  - → `contextra-ports`: dev: `Result`
- `tests/integration_graph.rs` (109 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/j15-contextra-graph-graph-nebenmodule-appr-p_arc_slice_tests.rs` (26 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/leiden_reference_validation.rs` (149 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/path_rag_at_snapshot.rs` (299 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/ppr_alloc_test.rs` (111 Zeilen)
  - → `contextra-types`: dev: `PprAlgorithm::Auto`
- `tests/ppr_candidate_stream.rs` (308 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`×9
  - → `contextra-types`: dev: `Result`×8
- `tests/ppr_snapshot_isolation.rs` (346 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/ppr_strategy_equivalence.rs` (96 Zeilen)
  - → `contextra-ports`: dev: `graph_index::GraphIndex`
- `tests/proptest_csr_invariants.rs` (325 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/proptest_tl_hfd_diffusion.rs` (273 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/proptest_tl_hfd_lovasz.rs` (261 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/provenance_delete_cascade.rs` (128 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/source_doc_ids_populated.rs` (147 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/star_expansion_equals_clique.rs` (42 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/tlhfd_shadow_mode_parity.rs` (281 Zeilen)
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
  - 3 const, 1 enum, 4 fn, 4 struct, 1 trait, 1 type
    - `const MAX_HYPEREDGE_CASCADE_FANOUT`
    - `const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT`
    - `struct CascadeInvalidationReport`
    - `fn cascade_invalidate_edges_for_superseded_doc`
    - `const CASCADE_QUEUE_PREFIX`
    - `enum CascadeStatus`
    - `struct DeletionProof`
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
  - 1 struct
    - `struct CsrGraph` *(re-exportiert)* · 53 Methoden
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
- `error`
  - 1 enum
    - `enum GraphMutationError`
- `hyperedge`
  - 2 const, 2 fn, 7 struct
    - `const HYPEREDGE_PREFIX`
    - `const HYPEREDGE_BY_ENTITY_PREFIX`
    - `fn star_weight`
    - `struct HyperEdgeId` · 2 Methoden
    - `struct RoleId` · 2 Methoden
    - `struct RoleBinding` · 1 Methoden
    - `struct HyperEdge` · 9 Methoden
    - `struct HyperEdgeView` · 3 Methoden
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
- `percolation`
  - 3 fn, 1 struct
- `ppr`
  - 4 fn, 6 struct
- `ppr_stream`
  - 1 const, 1 struct
- `provenance`
  - 2 struct
- `session_dag`
  - 2 const, 6 struct, 1 type
- `tl_hfd`
  - 2 const, 1 enum, 5 fn, 4 struct, 1 trait
  - … 45 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 17 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::EntityId`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `hyperedge::HyperEdgeId`×2, `percolation::PercolationConfig`×2, `CsrGraph::load_from_storage`×2, `hyperedge::HyperEdge`×2, `cascade_invalidate_edges_for_superseded_doc`, `percolation::compute_percolation_health`, `percolation::should_trigger_rebonding`, `percolation::find_rebonding_candidates`, `CsrGraph`, `hyperedge::RoleId::new` … (+6) | `csr::CsrGraph`×15, `csr`, `CsrGraph::new` |
| `contextra-cognition` | `EdgeReinforcementBuffer`×3, `HyperEdgeId`, `cascade::CascadeInvalidationReport`, `cascade_invalidate_edges_for_superseded_doc`, `csr::EdgeType`, `CsrGraph`, `hyperedge`, `hyperedge::HyperEdgeId`, `percolation::PercolationConfig`, `EdgeReinforcementConfig` … (+1) | `EdgeReinforcementBuffer::new`×2 |
| `contextra-db` | `percolation::PercolationConfig` | `CsrGraph`×10, `path_rag::PathRAGEngine`×2, `csr::CsrGraph`×2, `PathRAGEngine` |
| `contextra-agent` | – | `CsrGraph`×2 |
| `contextra-router` | – | `CsrGraph::new` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 111 von 125)**: `run_apprh_push`, `ApprhError`, `ApprhGateMonitorConfig`, `ApprhObservationRecord`, `ApprhGateMonitorSnapshot`, `ApprhGateMonitor`, `apprh_local`, `forward_push_apprh`, `ApprhParams`, `ApprhMode`, `ApprhSelectorConfig`, `ApprhSelector`, `ApprhFlipGate`, `DefaultApprhFlipGate`, `ApprhShadowComparison`, `shadow_compare_forward_push_vs_apprh`, `ArcSlice`, `MAX_HYPEREDGE_CASCADE_FANOUT`, `DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT`, `CASCADE_QUEUE_PREFIX`, `CascadeStatus`, `DeletionProof`, `CascadeTicket`, `GraphGarbageCollection`, `HyperedgeCascadeReport`, `CascadeReport`, `topological_sort_hyperedge_closure`, `cascade_invalidate_hyperedges_for_superseded_doc`, `verify_doc_hyperedge_deletion_completeness`, `CommunityDetectionConfig` …

**Auffälligkeiten**

- contextra-testkit: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-infer-candle

- Pfad: `crates/contextra-infer-candle` · Zeilen: 6657 · Rust-Dateien: 33 · Ring: 2
- Interne Cargo-Abhängigkeiten: `contextra-crypto` (dev), `contextra-crypto` (normal/optional via kv-bridge), `contextra-ports` (normal), `contextra-rank` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 22 | 29 | 5 | `kv::KvLayout`×4, `kv::RopeConfig`×4, `RequestId`×3, `ContextSegment`×2, `IdGen`×2, `kv::KvPrefixStore`×2, `embedding::EmbeddingError`, `SequentialIdGen::new`, `kv`, `KvBridgeStorage` … (+1) | `kv::PrefixKey`×10, `GroundingValidator`×2, `EmbeddingProvider`×2, `kv::KvPrefixHit`×2, `BoxFuture` … (+2) |
| `contextra-types` | 6 | 20 | 5 | `ContextraError`×4, `DocId::from`, `ModelFingerprint` | `TenantId`×7, `DocId`×3, `Result`×3, `ContextChunk`×2 |
| `contextra-crypto` | 1 | 1 | 1 | `EncryptedKvLayer` | `KvSegment` |
| `contextra-rank` | 1 | 0 | 1 | `IsotonicCalibrator` | – |

**Pro Komponente**

- `embedding_provider` (84 Zeilen)
  - → `contextra-ports`: prod: `embedding::EmbeddingError`
- `gasp` (586 Zeilen)
  - → `contextra-rank`: prod: `IsotonicCalibrator`
  - → `contextra-types`: prod: `DocId::from` · dev: `DocId`
- `gguf_loader` (83 Zeilen)
  - → `contextra-types`: prod: `ContextraError`
- `inference` (1702 Zeilen)
  - → `contextra-ports`: prod: `kv::KvLayout`×4, `kv::RopeConfig`×4, `RequestId`×3, `IdGen`×2, `kv::KvPrefixStore`×2, `ContextSegment`, `SequentialIdGen::new`, `kv` · dev: `GroundingValidator`
  - → `contextra-types`: dev: `TenantId`, `ContextChunk`, `DocId`
- `kv_bridge` (602 Zeilen)
  - → `contextra-crypto`: prod: `EncryptedKvLayer` · dev: `KvSegment`
  - → `contextra-ports`: prod: `ContextSegment`, `KvBridgeStorage` · dev: `BoxFuture`
  - → `contextra-types`: dev: `Result`
- `kv_state` (285 Zeilen)
  - → `contextra-ports`: prod: `kv::KvBlock`
  - → `contextra-types`: prod: `ContextraError`
- `model` (515 Zeilen)
  - → `contextra-types`: prod: `ContextraError`
- `model_registry` (122 Zeilen)
  - → `contextra-types`: prod: `ContextraError`, `ModelFingerprint`
- `tests/embedding_normalization_proptest.rs` (108 Zeilen)
  - → `contextra-ports`: dev: `EmbeddingProvider`
- `tests/embedding_provider_conformance.rs` (120 Zeilen)
  - → `contextra-ports`: dev: `embedding::EmbeddingError`, `EmbeddingProvider`
  - → `contextra-types`: dev: `Result`
- `tests/gasp_mutant_test.rs` (83 Zeilen)
  - → `contextra-ports`: dev: `GroundingValidator`
- `tests/gguf_corruption_and_batch_boundary_test.rs` (296 Zeilen)
  - → `contextra-ports`: dev: `embedding::EmbeddingError`
  - → `contextra-types`: dev: `Result`
- `tests/j05-contextra-infer-candle_wiring_test.rs` (125 Zeilen)
  - → `contextra-ports`: dev: `kv::PrefixKey`×3, `RequestId`, `kv::KvPrefixStore`, `kv::KvPrefixHit` … (+1)
  - → `contextra-types`: dev: `TenantId`×3, `ContextraError`×2
- `tests/j05closure-contextra-infer-candle_closure_test.rs` (144 Zeilen)
  - → `contextra-ports`: dev: `kv::PrefixKey`×3, `RequestId`, `kv::KvPrefixStore`, `kv::KvPrefixHit` … (+1)
  - → `contextra-types`: dev: `TenantId`×3, `ContextraError`×2
- `tests/kv_bridge_and_stress_test.rs` (146 Zeilen)
  - → `contextra-ports`: dev: `ResponseGroundingValidator`
- `tests/kv_prefix_reuse_test.rs` (340 Zeilen)
  - → `contextra-ports`: dev: `kv::PrefixKey`×4, `ContextSegment`, `LlmTextGenerator`, `kv`
- `tests/kv_stage_b_test.rs` (133 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/proptest_candle.rs` (68 Zeilen)
  - → `contextra-types`: dev: `ContextChunk`, `DocId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `attention_exporter`
  - 1 const, 1 struct
    - `const MAX_TRACKED_REQUESTS`
    - `struct CandleAttentionExporter` · 6 Methoden
- `embedding`
  - 1 const, 3 struct, 1 trait
    - `const DEFAULT_MAX_CONCURRENT_EMBEDDINGS`
    - `trait CandleEmbedInner` · Methoden: embed, dim
    - `struct CandleEmbedClient` · 5 Methoden
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
  - 1 fn, 1 struct
    - `struct GgufMetadata`
    - `fn parse_gguf_metadata` ⚠ fremd: `contextra-types`
- `inference`
  - 1 const, 2 fn, 6 struct, 1 trait
    - `trait CandleModelInner` *(re-exportiert)* · Methoden: generate, generate_stream, last_attention_scores, kv_layout, generate_stream_with_prefix
    - `const DEFAULT_MAX_CONCURRENT_INFERENCES` *(re-exportiert)*
    - `struct CandleLlmClient` *(re-exportiert)* · 15 Methoden ⚠ fremd: `contextra-ports`
    - `struct QuantizedLlamaModel` *(re-exportiert)* · 1 Methoden
    - `struct DefaultCandleLlmModel` *(re-exportiert)*
    - `struct PrefixSeed` *(re-exportiert)*
    - `struct PrefixRun` *(re-exportiert)*
    - `struct KvPrefixContext` *(re-exportiert)*
    - `fn build_prefix_key` *(re-exportiert)*
    - `fn seed_from_hit` *(re-exportiert)*
- `kv_bridge`
  - 2 struct
    - `struct KvCacheKey` · 1 Methoden
    - `struct KvBridgeAdapter` · 8 Methoden ⚠ fremd: `contextra-ports`
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
- ⚠ Re-Exporte **fremder Crates**: `contextra_ports::ContextSegment`, `contextra_types::ModelFingerprint`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-mcp` | `KvBridgeAdapter`×3, `model_registry::CandleQuantization::Q4KM`×2, `CandleEmbedClient::from_dir`, `CandleLlmClient::from_dir`, `KvBridgeAdapter::new` | – |
| `contextra-bench` | `CandleEmbedClient::from_dir`, `model_registry::CandleQuantization::Q4KM` | – |
| `contextra` | `(crate)` | – |
| `contextra-engine` | `CandleAttentionExporter` | – |
| `contextra-cognition` | – | `gasp` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 31 von 33)**: `MAX_TRACKED_REQUESTS`, `DEFAULT_MAX_CONCURRENT_EMBEDDINGS`, `CandleEmbedInner`, `CandleEmbedClient`, `BertEmbedModel`, `DefaultCandleEmbedModel`, `MAX_CANDLE_EMBED_BATCH_SIZE`, `DEFAULT_GROUNDING_THRESHOLD`, `GaspConfig`, `GaspValidator`, `GgufMetadata`, `parse_gguf_metadata`, `CandleModelInner`, `DEFAULT_MAX_CONCURRENT_INFERENCES`, `CandleLlmClient`, `QuantizedLlamaModel`, `DefaultCandleLlmModel`, `PrefixSeed`, `PrefixRun`, `KvPrefixContext`, `build_prefix_key`, `seed_from_hit`, `KvCacheKey`, `LayerKv`, `KvState`, `MAX_SEQ_LEN`, `QuantizedMatMul`, `LayerWeights`, `ModelWeights`, `CandleQuantization` …

### contextra-infer-ollama

- Pfad: `crates/contextra-infer-ollama` · Zeilen: 4906 · Rust-Dateien: 17 · Ring: 2
- Interne Cargo-Abhängigkeiten: `contextra-infer-onnx` (dev), `contextra-ports` (normal), `contextra-rank` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 8 | 5 | 4 | `Result`×4, `ContextraError`×3, `ConfigFingerprint` | `ImportanceScore`×2 |
| `contextra-ports` | 6 | 6 | 1 | `BoxFuture`×2, `LlmTextGenerator`, `LlmTextGeneratorStreaming`, `BoxStream`, `SegmentSynthesizer` | `SeededRng`×3, `TextEmbeddingEngine`×2 |
| `contextra-rank` | 1 | 0 | 1 | `IsotonicCalibrator` | – |

**Pro Komponente**

- `client` (2655 Zeilen)
  - → `contextra-ports`: prod: `BoxFuture`×2, `LlmTextGenerator`, `LlmTextGeneratorStreaming`, `BoxStream`, `SegmentSynthesizer` · dev: `SeededRng`×3, `TextEmbeddingEngine`, `SegmentSynthesizer`
  - → `contextra-types`: prod: `Result`×4, `ContextraError`, `ConfigFingerprint` · dev: `ContextraError`, `ConfigFingerprint`
- `context_prefixer` (386 Zeilen)
  - → `contextra-types`: prod: `ContextraError`
- `embedding` (280 Zeilen)
  - → `contextra-ports`: dev: `TextEmbeddingEngine`
  - → `contextra-types`: prod: `ContextraError`
- `importance` (556 Zeilen)
  - → `contextra-rank`: prod: `IsotonicCalibrator`
- `tests/campaign_stub_ollama.rs` (176 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/j28-contextra-infer-ollama_wire_tests.rs` (149 Zeilen)
  - → `contextra-types`: dev: `ImportanceScore`
- `tests/j28closure-contextra-infer-ollama_symbols_test.rs` (102 Zeilen)
  - → `contextra-types`: dev: `ImportanceScore`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `client`
  - 5 const, 8 fn, 2 struct
    - `const DEFAULT_BASE_URL` *(re-exportiert)*
    - `const DEFAULT_EMBED_MODEL` *(re-exportiert)*
    - `const MAX_RETRIES` *(re-exportiert)*
    - `const MAX_BATCH_SIZE` *(re-exportiert)*
    - `const MAX_TEXT_BYTES` *(re-exportiert)*
    - `struct OllamaConfig` *(re-exportiert)*
    - `struct OllamaClient` *(re-exportiert)* · 21 Methoden
    - `fn is_transient_network_error` *(re-exportiert)*
    - `fn classify_reqwest_error` *(re-exportiert)* ⚠ fremd: `contextra-types`
    - `fn is_transient_error` *(re-exportiert)* ⚠ fremd: `contextra-types`
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
- Zusätzlich 2 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-mcp` | `OllamaEmbedder::new`, `OllamaConfig`, `OllamaClient::with_config` | – |
| `contextra` | `(crate)` | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 29 von 30)**: `DEFAULT_BASE_URL`, `DEFAULT_EMBED_MODEL`, `MAX_RETRIES`, `MAX_BATCH_SIZE`, `MAX_TEXT_BYTES`, `OllamaClient`, `is_transient_network_error`, `classify_reqwest_error`, `is_transient_error`, `xml_escape`, `build_rag_prompt`, `validate_text_length`, `validate_batch_size`, `validate_model_name`, `ContextPrefixConfig`, `ContextPrefixEngine`, `ContextPrefixer`, `truncate_chars`, `truncate_prefix`, `OllamaEmbedder`, `score_importance_batch`, `Confidence`, `ImportanceAssessment`, `score_importance`, `score_importance_with_calibrator`, `record_importance_outcome`, `parse_importance_score_response`, `ModelInfo`, `known_dimension`

**Auffälligkeiten**

- contextra-infer-onnx: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-infer-onnx

- Pfad: `crates/contextra-infer-onnx` · Zeilen: 2004 · Rust-Dateien: 10 · Ring: 2
- Interne Cargo-Abhängigkeiten: `contextra-infer-candle` (normal/optional via candle-backend), `contextra-ports` (normal), `contextra-rank` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-rank` | 4 | 1 | 3 | `PlattScaler`×4 | – |
| `contextra-types` | 4 | 3 | 3 | `Result`, `ContextraError::CapabilityUnsupported`, `RerankResult`, `ContextraError` | – |
| `contextra-ports` | 0 | 1 | 0 | – | `EmbeddingProvider` |

**Pro Komponente**

- `(crate-root)` (742 Zeilen)
  - → `contextra-ports`: dev: `EmbeddingProvider`
  - → `contextra-types`: prod: `Result`, `ContextraError::CapabilityUnsupported` · dev: `ContextraError`, `Result`, `ContextraError::CapabilityUnsupported`
- `reranker` (1028 Zeilen)
  - → `contextra-rank`: prod: `PlattScaler`×4 · dev: `PlattScaler`
  - → `contextra-types`: prod: `RerankResult`, `ContextraError`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 const, 3 fn, 2 struct, 1 type
    - `fn default_model_cache_dir`
    - `fn ensure_onnx_model_download` ⚠ fremd: `contextra-types`
    - `fn ensure_onnx_model_download` ⚠ fremd: `contextra-types`
    - `const MAX_EMBED_BATCH_SIZE`
    - `struct TextEmbedderConfig` · 1 Methoden
    - `type OnnxEmbedder`
    - `struct TextEmbedder` · 7 Methoden
- `reranker`
  - 1 struct
    - `struct CrossEncoderReranker` *(re-exportiert)* · 13 Methoden
- Zusätzlich 2 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_rank::PlattScaler`, `contextra_types::RerankResult`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `ensure_onnx_model_download`, `OnnxEmbedder::load` | – |
| `contextra` | `(crate)` | – |
| `contextra-mcp` | `OnnxEmbedder::from_path` | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 6 von 8)**: `default_model_cache_dir`, `MAX_EMBED_BATCH_SIZE`, `TextEmbedderConfig`, `OnnxEmbedder`, `TextEmbedder`, `CrossEncoderReranker`

**Auffälligkeiten**

- contextra-infer-candle: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-ports: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?

### contextra-kvcache

- Pfad: `crates/contextra-kvcache` · Zeilen: 6435 · Rust-Dateien: 27 · Ring: 1
- Interne Cargo-Abhängigkeiten: `contextra-kvcache` (dev), `contextra-crypto` (normal), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-crypto` | 11 | 5 | 2 | `KvCipher`×5, `KvSegmentCipher`×2, `CryptoError`×2, `CryptoKey`, `ModelFingerprint` | `CryptoKey::try_new` |
| `contextra-types` | 7 | 16 | 4 | `TenantId`×3, `DocId`×2, `error::Result`, `error` | `model_fingerprint::ModelFingerprint`×6, `model_fingerprint::ModelFingerprint::new`, `ContextraError` |
| `contextra-ports` | 6 | 13 | 4 | `RequestId`×3, `kv`, `kv::PrefixKey`, `KvLifecycleHooks` | `kv::KvLayout`, `kv::RopeConfig` |

**Pro Komponente**

- `attention_score` (233 Zeilen)
  - → `contextra-ports`: prod: `RequestId`
  - → `contextra-types`: prod: `TenantId`
- `eviction_worker` (262 Zeilen)
  - → `contextra-types`: prod: `TenantId` · dev: `TenantId`
- `prefix_store` (353 Zeilen)
  - → `contextra-ports`: prod: `kv` · dev: `kv::PrefixKey`, `kv::KvLayout`, `kv::RopeConfig`
  - → `contextra-types`: dev: `model_fingerprint::ModelFingerprint::new`
- `scratchpad_invalidation` (187 Zeilen)
  - → `contextra-ports`: prod: `kv::PrefixKey` · dev: `kv`
  - → `contextra-types`: prod: `error::Result`, `TenantId` · dev: `model_fingerprint::ModelFingerprint`
- `segment` (517 Zeilen)
  - → `contextra-crypto`: prod: `KvCipher`×3, `CryptoKey`
- `store` (1062 Zeilen)
  - → `contextra-crypto`: prod: `KvSegmentCipher`×2, `CryptoError`×2, `KvCipher`×2, `ModelFingerprint`
  - → `contextra-ports`: prod: `RequestId`×2, `KvLifecycleHooks`
  - → `contextra-types`: prod: `DocId`×2, `error`
- `tests/clm_pinned_base_preserved.rs` (99 Zeilen)
  - → `contextra-ports`: dev: `kv`
  - → `contextra-types`: dev: `model_fingerprint::ModelFingerprint`, `TenantId`
- `tests/content_hash_hit_rate_improvement.rs` (95 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/e2e_lifecycle_with_engine.rs` (238 Zeilen)
  - → `contextra-ports`: dev: `kv`
  - → `contextra-types`: dev: `model_fingerprint::ModelFingerprint`
- `tests/eviction_segment_mapping.rs` (122 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/eviction_with_attention_exporter.rs` (82 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/j06_contextra_kvcache_wiring.rs` (167 Zeilen)
  - → `contextra-crypto`: dev: `CryptoKey::try_new`
  - → `contextra-ports`: dev: `RequestId`, `kv`
  - → `contextra-types`: dev: `model_fingerprint::ModelFingerprint`, `TenantId`
- `tests/j06closure-contextra-kvcache_wiring.rs` (163 Zeilen)
  - → `contextra-crypto`: dev: `KvCipher`
  - → `contextra-ports`: dev: `RequestId`, `kv`
- `tests/kivi_aead_order_enforced.rs` (106 Zeilen)
  - → `contextra-crypto`: dev: `KvCipher`×2, `CryptoKey`
- `tests/kivi_quantize_dequantize_roundtrip.rs` (159 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/kv_lifecycle_hooks_tenant_isolation.rs` (66 Zeilen)
  - → `contextra-ports`: dev: `KvLifecycleHooks`
- `tests/prefix_store_port.rs` (194 Zeilen)
  - → `contextra-ports`: dev: `kv`
  - → `contextra-types`: dev: `model_fingerprint::ModelFingerprint`
- `tests/scratchpad_invalidation_isolation.rs` (87 Zeilen)
  - → `contextra-ports`: dev: `kv`
  - → `contextra-types`: dev: `model_fingerprint::ModelFingerprint`, `TenantId`
- `tests/store_default_attention_source.rs` (101 Zeilen)
  - → `contextra-types`: dev: `TenantId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

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
    - `struct TenantPrefixKvStore` · 8 Methoden
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
    - `struct Tier2EncryptedSegment` · 4 Methoden
    - `struct EncryptedSegmentPayload`
    - `struct KvSegment` · 13 Methoden
- `store`
  - 1 const, 1 struct, 1 type
    - `type SpillHandler`
    - `const MAX_PINNED_BYTES_PER_TENANT`
    - `struct TenantIsolatedKvStore` · 32 Methoden

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `TenantIsolatedKvStore::new`, `EvictionWorker`, `AttentionScoreSource` | `TenantIsolatedKvStore`×2 |

**Nach außen ungenutzte öffentliche Items (Heuristik, 37 von 40)**: `NullAttentionScoreSource`, `rank_for_eviction`, `rank_for_eviction_weighted`, `rank_for_eviction_weighted_for_tenant`, `emergency_wipe`, `DEFAULT_BYTE_BUDGET_PER_TENANT`, `TenantPrefixKvStore`, `KiviQuantizeConfig`, `KvTensorView`, `KiviBlockMeta`, `KiviQuantizedBlock`, `kivi_quantize`, `kivi_dequantize`, `unpack_kivi_block`, `pack_kivi_block`, `compress_bytes`, `decompress_bytes`, `KvReusePolicy`, `KvBlockGuard`, `PrefixMatch`, `PrefixRadixTree`, `KvSegmentRef`, `KvLookupResult`, `SemanticEmbedder`, `SemanticCacheConfig`, `ContentAddressedKvStore`, `ScratchpadCacheScope`, `ScratchpadInvalidator`, `PrefixStoreScratchpadInvalidator`, `KvSegmentContent` …

**Auffälligkeiten**

- contextra-kvcache: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-license

- Pfad: `crates/contextra-license` · Zeilen: 1415 · Rust-Dateien: 12 · Ring: 4
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 3 | 5 | 2 | `license`×2, `clock` | `clock::Clock`×5 |
| `contextra-types` | 1 | 3 | 1 | `TenantId` | – |

**Pro Komponente**

- `(crate-root)` (9 Zeilen)
  - → `contextra-ports`: prod: `license`
- `signed_gate` (397 Zeilen)
  - → `contextra-ports`: prod: `clock`, `license`
  - → `contextra-types`: prod: `TenantId`
- `tests/check_ring_order.rs` (198 Zeilen)
  - → `contextra-ports`: dev: `clock::Clock`
- `tests/installation_binding.rs` (227 Zeilen)
  - → `contextra-ports`: dev: `clock::Clock`
  - → `contextra-types`: dev: `TenantId`
- `tests/legacy_payload_compat.rs` (124 Zeilen)
  - → `contextra-ports`: dev: `clock::Clock`
  - → `contextra-types`: dev: `TenantId`
- `tests/signed_activation_7_steps.rs` (138 Zeilen)
  - → `contextra-ports`: dev: `clock::Clock`
- `tests/signed_gate.rs` (150 Zeilen)
  - → `contextra-ports`: dev: `clock::Clock`
  - → `contextra-types`: dev: `TenantId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `signed_gate`
  - 1 fn, 3 struct
    - `struct SignedActivation` · 3 Methoden
    - `fn derive_local_installation_id_hash`
    - `struct LicensePayload` ⚠ fremd: `contextra-types`
    - `struct SignedLicenseGate` · 14 Methoden
- Zusätzlich 2 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra` | `SignedLicenseGate`, `signed_gate::VerifyingKey::from_bytes` | `OpenFastGate` |
| `contextra-mcp` | `OpenFastGate` | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 3 von 4)**: `SignedActivation`, `derive_local_installation_id_hash`, `LicensePayload`

### contextra-mcp

- Pfad: `crates/contextra-mcp` · Zeilen: 9821 · Rust-Dateien: 41 · Ring: 4
- Interne Cargo-Abhängigkeiten: `contextra` (normal), `contextra-adapt` (normal), `contextra-agent` (normal/optional via agent-workflows), `contextra-crypto` (normal), `contextra-infer-candle` (normal/optional via candle,kv-bridge), `contextra-infer-ollama` (normal/optional via ollama), `contextra-infer-onnx` (normal/optional via onnx), `contextra-license` (normal), `contextra-ports` (normal), `contextra-privacy` (normal), `contextra-rank` (normal), `contextra-types` (normal), `contextra-wire` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra` | 20 | 15 | 6 | `router::SlmProfile`×4, `Contextra`×3, `router::DefaultRouterEngine`×2, `Collection`, `router::ports_local::HybridSearchProvider`, `router::ports_local::CommunityResolver`, `router::ports_local::ContextPreparer`, `router::RouterEngine::new`, `chunker`, `execute_background_consolidation` … (+4) | `router::SlmProfile::new` |
| `contextra-ports` | 19 | 51 | 5 | `plugin::PluginRegistry`×4, `BoxFuture`×3, `DriftStatusProvider`×3, `license::FeatureRing`×2, `plugin::PluginCapability`×2, `MockEmbedder::new`, `EmbeddingProvider`, `StorageEngine`, `ContextSegment::new`, `plugin::PluginRegistry::new` | `EmbeddingError`×20, `MockEmbedder`×2, `Clock` |
| `contextra-types` | 9 | 4 | 5 | `ContextraError`×4, `ContextraError::InvalidInput`, `ContextraError::NotFound`, `TenantId::try_new`, `TenantId::SYSTEM`, `TenantId` | `DocId`, `AutoExtractionMode`, `EntityId::from_key`, `TokenBudget::new` |
| `contextra-crypto` | 8 | 7 | 2 | `CryptoKey`×3, `CryptoKey::try_new`×2, `TenantIsolatedKvStore::new`×2, `KvSegmentCipher::new` | `deletion_proof::DeletionProof`, `AuditChain` |
| `contextra-infer-candle` | 8 | 1 | 3 | `KvBridgeAdapter`×3, `model_registry::CandleQuantization::Q4KM`×2, `CandleEmbedClient::from_dir`, `CandleLlmClient::from_dir`, `KvBridgeAdapter::new` | – |
| `contextra-privacy` | 6 | 3 | 3 | `bulk_exfiltration_detector::*`, `egress_gateway::*`, `EgressError`, `egress_vault`, `InjectionDetector`, `EgressGuard` | `ContextEditAuditRecord`, `egress_gateway::EgressGuardCheck` |
| `contextra-infer-ollama` | 3 | 0 | 1 | `OllamaEmbedder::new`, `OllamaConfig`, `OllamaClient::with_config` | – |
| `contextra-rank` | 3 | 0 | 2 | `fusion::ProvenanceRecord`, `IsotonicCalibrator`, `IsotonicCalibrator::with_defaults` | – |
| `contextra-adapt` | 2 | 0 | 1 | `PidController`, `PidController::default` | – |
| `contextra-infer-onnx` | 1 | 0 | 1 | `OnnxEmbedder::from_path` | – |
| `contextra-license` | 1 | 0 | 1 | `OpenFastGate` | – |

**Pro Komponente**

- `bin` (79 Zeilen)
  - → `contextra`: prod: `Contextra`, `router::SlmProfile`
- `bulk_exfiltration_detector` (3 Zeilen)
  - → `contextra-privacy`: prod: `bulk_exfiltration_detector::*`
- `config` (554 Zeilen)
  - → `contextra`: prod: `router::SlmProfile`×3
  - → `contextra-infer-candle`: prod: `model_registry::CandleQuantization::Q4KM`×2, `CandleEmbedClient::from_dir`, `CandleLlmClient::from_dir`
  - → `contextra-infer-ollama`: prod: `OllamaEmbedder::new`, `OllamaConfig`, `OllamaClient::with_config`
  - → `contextra-infer-onnx`: prod: `OnnxEmbedder::from_path`
  - → `contextra-ports`: prod: `MockEmbedder::new`, `BoxFuture`
  - → `contextra-types`: prod: `ContextraError`
- `egress_gateway` (24 Zeilen)
  - → `contextra-privacy`: prod: `egress_gateway::*`, `EgressError`, `egress_vault`, `InjectionDetector`
- `egress_guard` (3 Zeilen)
  - → `contextra-privacy`: prod: `EgressGuard`
- `explain` (239 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
  - → `contextra-rank`: prod: `fusion::ProvenanceRecord`
- `plugin_status` (39 Zeilen)
  - → `contextra-ports`: prod: `license::FeatureRing`×2, `plugin::PluginCapability`×2, `plugin::PluginRegistry`
- `prompt_injection` (1204 Zeilen)
  - → `contextra-ports`: dev: `Clock`
- `protocol` (73 Zeilen)
  - → `contextra-types`: prod: `ContextraError`×2, `ContextraError::InvalidInput`, `ContextraError::NotFound`
- `routing` (285 Zeilen)
  - → `contextra`: prod: `router::DefaultRouterEngine`×2, `Contextra`, `Collection`, `router::ports_local::HybridSearchProvider`, `router::ports_local::CommunityResolver`, `router::ports_local::ContextPreparer`, `router::RouterEngine::new` · dev: `router::ports_local::ContextPreparer`
  - → `contextra-adapt`: prod: `PidController`, `PidController::default`
  - → `contextra-crypto`: prod: `TenantIsolatedKvStore::new`×2, `CryptoKey::try_new`, `KvSegmentCipher::new`
  - → `contextra-infer-candle`: prod: `KvBridgeAdapter`, `KvBridgeAdapter::new`
  - → `contextra-ports`: prod: `DriftStatusProvider`×3, `BoxFuture`×2
  - → `contextra-rank`: prod: `IsotonicCalibrator`, `IsotonicCalibrator::with_defaults`
  - → `contextra-types`: dev: `DocId`
- `sandbox` (823 Zeilen)
  - → `contextra-crypto`: prod: `CryptoKey`×3, `CryptoKey::try_new` · dev: `CryptoKey::try_new`×2
- `server` (197 Zeilen)
  - → `contextra`: prod: `Contextra`
  - → `contextra-infer-candle`: prod: `KvBridgeAdapter`×2
  - → `contextra-ports`: prod: `plugin::PluginRegistry`×3, `EmbeddingProvider`
  - → `contextra-types`: prod: `ContextraError`
- `server_tools` (870 Zeilen)
  - → `contextra`: prod: `chunker`, `execute_background_consolidation`, `memory_consolidation::ConsolidationConfig::default`
  - → `contextra-license`: prod: `OpenFastGate`
  - → `contextra-ports`: prod: `StorageEngine`, `ContextSegment::new`, `plugin::PluginRegistry::new`
  - → `contextra-types`: prod: `TenantId::try_new`, `TenantId::SYSTEM`
- `tests` (1207 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-crypto`: dev: `deletion_proof::DeletionProof`, `TenantIsolatedKvStore::new`, `CryptoKey::try_new`, `KvSegmentCipher::new`
  - → `contextra-infer-candle`: dev: `KvBridgeAdapter::new`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
- `tools_crud` (175 Zeilen)
  - → `contextra`: prod: `collection_profile::DeploymentTier`, `collection_profile::AutoExtractionMode::Enabled`, `collection_profile::AutoExtractionMode::Disabled`
  - → `contextra-types`: prod: `TenantId`
- `tests/campaign_j42_mcp_audit.rs` (76 Zeilen)
  - → `contextra`: dev: `collection_profile::DeploymentTier`
  - → `contextra-crypto`: dev: `AuditChain`
  - → `contextra-privacy`: dev: `ContextEditAuditRecord`
  - → `contextra-types`: dev: `AutoExtractionMode`
- `tests/cloud_query_scoped_guard.rs` (160 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
  - → `contextra-privacy`: dev: `egress_gateway::EgressGuardCheck`, `egress_vault`
- `tests/confirm_required_semantics.rs` (186 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `BoxFuture`×2, `EmbeddingError`×2, `EmbeddingProvider`
- `tests/contextra_delete_no_unverified_proof.rs` (175 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `BoxFuture`×2, `EmbeddingError`×2, `EmbeddingProvider`
- `tests/j18-contextra-mcp_wiring_test.rs` (129 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `MockEmbedder`, `BoxFuture`, `EmbeddingProvider`, `plugin::PluginRegistry`
- `tests/j18closure-contextra-mcp_wiring_test.rs` (66 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `MockEmbedder`, `EmbeddingProvider`
- `tests/license_gate_entrypoint.rs` (108 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `license::FeatureRing`, `BoxFuture`, `EmbeddingProvider`
- `tests/mcp_relate_tools.rs` (353 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `EmbeddingProvider`
  - → `contextra-types`: dev: `EntityId::from_key`
- `tests/mcp_test.rs` (1129 Zeilen)
  - → `contextra`: dev: `Contextra`, `router::SlmProfile::new`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
  - → `contextra-types`: dev: `TokenBudget::new`
- `tests/plugin_status_tool.rs` (144 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `license::FeatureRing`, `EmbeddingProvider`
- `tests/tool_registry_classification.rs` (167 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `BoxFuture`×2, `EmbeddingError`×2, `EmbeddingProvider`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `config`
  - 2 fn, 4 struct
    - `struct EmbeddingConfig` · 2 Methoden
    - `fn create_embedding_provider` ⚠ fremd: `contextra-types`
    - `struct LlmConfig` · 2 Methoden
    - `fn create_llm_text_generator` ⚠ fremd: `contextra-types`
    - `struct RouterConfig` · 1 Methoden ⚠ fremd: `contextra`
    - `struct MockLlmGenerator`
- `explain`
  - 2 struct
    - `struct ExplainRequest`
    - `struct ExplainResponse` ⚠ fremd: `contextra-rank`
- `io`
  - 2 const, 1 fn
    - `const MAX_RPC_BYTES`
    - `const MAX_SEARCH_QUERY_BYTES`
    - `fn read_line_bounded`
- `plugin_status`
  - 1 fn, 2 struct
    - `struct PluginStatusResponse` ⚠ fremd: `contextra-ports`
    - `struct PluginStatusEntry` ⚠ fremd: `contextra-ports`
    - `fn handle_plugin_status` ⚠ fremd: `contextra-ports`
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
  - 2 fn, 1 struct
    - `struct RoutingHandle` ⚠ fremd: `contextra`, `contextra-adapt`, `contextra-ports`, `contextra-rank`
    - `fn setup_routing` ⚠ fremd: `contextra`
    - `fn setup_kv_bridge` ⚠ fremd: `contextra`, `contextra-infer-candle`
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
    - `struct McpServer` · 12 Methoden ⚠ fremd: `contextra`, `contextra-infer-candle`, `contextra-ports`
- `validation`
  - 2 fn
    - `fn is_write_allowed_by_env`
    - `fn validate_collection_name`
- Zusätzlich 9 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_privacy::bulk_exfiltration_detector::*`, `contextra_privacy::egress_gateway::*`, `contextra_privacy::EgressGuard`

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 36 von 36)**: `EmbeddingConfig`, `create_embedding_provider`, `LlmConfig`, `create_llm_text_generator`, `RouterConfig`, `MockLlmGenerator`, `ExplainRequest`, `ExplainResponse`, `MAX_RPC_BYTES`, `MAX_SEARCH_QUERY_BYTES`, `read_line_bounded`, `PluginStatusResponse`, `PluginStatusEntry`, `handle_plugin_status`, `deletion_proof_key_from_env`, `ProofKeySource`, `ProofKeyResolution`, `resolve_proof_key`, `resolve_proof_key_from_env`, `McpError`, `response_from_error`, `RoutingHandle`, `setup_routing`, `setup_kv_bridge`, `MAX_VOLATILE_RESULTS`, `MAX_VOLATILE_KEY_BYTES`, `MAX_VOLATILE_OUTPUT_BYTES`, `ToolDefinition`, `TOOL_REGISTRY`, `ToolCategory` …

**Auffälligkeiten**

- contextra-agent: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-wire: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-mvcc

- Pfad: `crates/contextra-mvcc` · Zeilen: 4572 · Rust-Dateien: 22 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 5 | 2 | 1 | `MetricsSink`×5 | `metrics` |
| `contextra-types` | 1 | 3 | 1 | `*` | `TxId`×2, `DocId` |

**Pro Komponente**

- `(crate-root)` (21 Zeilen)
  - → `contextra-types`: prod: `*`
- `ssi` (1008 Zeilen)
  - → `contextra-ports`: prod: `MetricsSink`×5
- `benches/mvcc_bench.rs` (63 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/campaign_cov_ssi_coarsening.rs` (46 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/campaign_cov_tx_buffer_reap.rs` (48 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/ssi_metrics_observability.rs` (119 Zeilen)
  - → `contextra-ports`: dev: `MetricsSink`, `metrics`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `seq_log`
  - 1 const, 1 enum, 2 struct
    - `const DEFAULT_MAX_PIN_DURATION`
    - `struct SeqLogEntry` · 1 Methoden
    - `enum SeqLogChange` · 2 Methoden
    - `struct SequenceLog` · 19 Methoden
- `snapshot`
  - 2 struct
    - `struct SnapshotRegistry` · 9 Methoden
    - `struct SnapshotGuard` · 1 Methoden
- `ssi`
  - 1 const, 4 struct, 1 trait
    - `const DEFAULT_MAX_TRACKED_COMMIT_KEYS`
    - `struct ReadSet` · 13 Methoden
    - `trait SsiValidator` · Methoden: validate
    - `struct PruningBlockerInfo`
    - `struct SequenceLogSsiValidator` · 19 Methoden
    - `struct SequenceLogSsiValidatorBuilder` · 7 Methoden
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
    - `struct TxBuffer` · 36 Methoden
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::*`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-core` | `seq_log`×2, `snapshot`×2, `tx_buffer`×2 | – |
| `contextra-store` | `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`×2, `SequenceLogSsiValidator`, `SsiValidator`, `SequenceLogSsiValidator::new_with_bounds` | – |
| `contextra-engine` | `tx_buffer::TxBuffer` | `tx_buffer::IndexOp` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 17 von 22)**: `DEFAULT_MAX_PIN_DURATION`, `SeqLogEntry`, `SeqLogChange`, `SequenceLog`, `SnapshotRegistry`, `SnapshotGuard`, `DEFAULT_MAX_TRACKED_COMMIT_KEYS`, `ReadSet`, `PruningBlockerInfo`, `SequenceLogSsiValidatorBuilder`, `DEFAULT_SHARD_COUNT`, `DEFAULT_MAX_OPS_PER_TX`, `DEFAULT_MAX_READ_SET_KEYS`, `DEFAULT_MAX_TX_STAGED_BYTES`, `DEFAULT_MAX_TOTAL_STAGED_BYTES`, `TxBufferConfig`, `StagedOpSize`

### contextra-ports

- Pfad: `crates/contextra-ports` · Zeilen: 3602 · Rust-Dateien: 25 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 9 | 0 | 4 | `ContextraError`×3, `types::domain`, `model_fingerprint::ModelFingerprint`, `types::domain::TenantId`, `DocId`, `*`, `ContextraError::Plugin` | – |

**Pro Komponente**

- `(crate-root)` (130 Zeilen)
  - → `contextra-types`: prod: `*`
- `graph` (185 Zeilen)
  - → `contextra-types`: prod: `ContextraError`, `types::domain`
- `kv` (231 Zeilen)
  - → `contextra-types`: prod: `model_fingerprint::ModelFingerprint`, `types::domain::TenantId`, `ContextraError`, `DocId`
- `plugin` (387 Zeilen)
  - → `contextra-types`: prod: `ContextraError`, `ContextraError::Plugin`

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
  - 1 enum, 2 struct, 4 trait
    - `struct ContextSegment` · 4 Methoden
    - `trait TextEmbeddingEngine` · Methoden: embed, embed_batch
    - `enum EmbeddingError`
    - `trait EmbeddingProvider` · Methoden: provider_name, embed, embedding_dim, embed_batch
    - `trait LlmTextGenerator` · Methoden: generate, generate_with_context
    - `trait LlmTextGeneratorStreaming` · Methoden: generate_stream
    - `struct MockEmbedder` · 3 Methoden
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
  - 5 struct, 2 trait
    - `struct KvLayout`
    - `struct RopeConfig`
    - `struct PrefixKey` ⚠ fremd: `contextra-types`
    - `struct KvBlock`
    - `struct KvPrefixHit`
    - `trait KvPrefixStore` · Methoden: lookup, insert, evict
    - `trait KvLifecycleHooks` · Methoden: on_rollback, remove_doc_segments, purge_tenant
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
  - 1 enum, 2 struct, 1 trait
    - `trait MetricsSink` · Methoden: record_counter, record_gauge, record_histogram
    - `struct NoopMetricsSink`
    - `enum MetricEvent`
    - `struct TestMetricsSink` · 3 Methoden
- `observability`
  - 1 trait
    - `trait DriftStatusProvider` · Methoden: overall_drift_status
- `plugin`
  - 1 enum, 2 struct, 1 trait
    - `struct PluginCapability` · 1 Methoden
    - `trait PluginManifest` · Methoden: name, requires, conflicts, feature_ring_required, capability, activate
    - `enum PluginError`
    - `struct PluginRegistry` · 6 Methoden
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
- Interne Re-Exporte (`pub use`): `TextEmbeddingEngine`
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::*`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-checkpoint` | `Clock`×19, `StorageEngine`×14, `MetricsSink`×6, `BoxFuture`×3, `SystemClock`×2, `Checkpoint`×2, `CheckpointCoordinator` | – |
| `contextra-engine` | `MetricsSink`×9, `StorageEngine`×5, `KvLifecycleHooks`×4, `VectorIndexStats`×2, `DriftStatusProvider`×2, `VectorIndex`, `Reranker`, `license`, `SystemClock::new`, `NoopMetricsSink` … (+6) | `Clock` |
| `contextra-infer-candle` | `kv::KvLayout`×4, `kv::RopeConfig`×4, `RequestId`×3, `ContextSegment`×2, `IdGen`×2, `kv::KvPrefixStore`×2, `embedding::EmbeddingError`, `SequentialIdGen::new`, `kv`, `KvBridgeStorage` … (+1) | `kv::PrefixKey`×10, `GroundingValidator`×2, `EmbeddingProvider`×2, `kv::KvPrefixHit`×2 … (+3) |
| `contextra-graph` | `GraphIndex`×3, `StorageEngine`×2, `BoxFuture`×2, `graph::GraphMutationError::Internal`×2, `graph::RoleBinding`, `graph::HyperEdgeId`, `graph::GraphMutationError`, `graph::GraphMutationError::LockAcquisitionTimeout`, `graph::GraphMutationError::RoleBindingInvalid`, `graph::GraphMutationError::EpochReclamationPending` … (+5) | `Result`×2, `graph`, `graph_index::GraphIndex` |
| `contextra-mcp` | `plugin::PluginRegistry`×4, `BoxFuture`×3, `DriftStatusProvider`×3, `license::FeatureRing`×2, `plugin::PluginCapability`×2, `MockEmbedder::new`, `EmbeddingProvider`, `StorageEngine`, `ContextSegment::new`, `plugin::PluginRegistry::new` | `EmbeddingError`×20, `MockEmbedder`×2, `Clock` |
| `contextra-agent` | `BoxFuture`×6, `MetricsSink`×4, `StorageEngine`×2, `VectorIndex`, `Checkpoint` | `StorageStats`×2, `SystemClock` |
| `contextra-store` | `MetricsSink`×6, `Clock`×4, `SystemClock::new`, `license::FeatureRing`, `license::FeatureRing::Sovereign`, `NoopMetricsSink` | `StorageEngine`×4, `StorageStats`, `storage::StorageEngine`, `SystemClock` |
| `contextra-vector` | `MetricsSink`×6, `Rng`×3, `NoopMetricsSink`×2, `SeededRng::new`×2 | `SeededRng`×5 |
| `contextra-infer-ollama` | `BoxFuture`×2, `LlmTextGenerator`, `LlmTextGeneratorStreaming`, `BoxStream`, `SegmentSynthesizer` | `SeededRng`×3, `TextEmbeddingEngine`×2 |
| `contextra-kvcache` | `RequestId`×3, `kv`, `kv::PrefixKey`, `KvLifecycleHooks` | `kv::KvLayout`, `kv::RopeConfig` |
| `contextra` | `license`×2, `IdGen`, `license::LicenseGate`, `license::VectorDeleteMode` | `license::OpenFastGate`×2, `SequentialIdGen`, `license::LicenseError`, `license::FeatureRing` |
| `contextra-mvcc` | `MetricsSink`×5 | `metrics` |
| `contextra-text` | `StorageEngine`×5 | `BoxFuture`×50, `StorageStats`×31 |
| `contextra-cognition` | `SegmentSynthesizer`×2, `LlmTextGenerator`, `StorageEngine` | `BoxFuture`×2, `SeededRng`, `ResponseGroundingValidator` |
| `contextra-license` | `license`×2, `clock` | `clock::Clock`×5 |
| `contextra-adapt` | `SeededRng::new`, `Rng::next_u64` | – |
| `contextra-core` | `*`×2 | – |
| `contextra-crypto` | `Clock`×2 | `SystemClock`×9 |
| `contextra-privacy` | `Clock`×2 | – |
| `contextra-db` | `QueryRewriter` | `GraphIndex`×4, `StorageEngine`×3, `BoxFuture`×2, `LlmTextGenerator` … (+3) |
| `contextra-testkit` | `Clock` | – |
| `contextra-infer-onnx` | – | `EmbeddingProvider` |
| `contextra-router` | – | `StorageEngine`×3, `HybridSearchProvider`, `CommunityResolver`, `ContextPreparer` |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

### contextra-privacy

- Pfad: `crates/contextra-privacy` · Zeilen: 4634 · Rust-Dateien: 22 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-crypto` (dev), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 31 | 1 | 4 | `TenantScoped`×13, `TenantId`×12, `TenantScoped::new`×5, `TenantScopeViolation` | – |
| `contextra-ports` | 2 | 4 | 2 | `Clock`×2 | – |
| `contextra-crypto` | 0 | 2 | 0 | – | `crypto::KeyManager`, `kv_cipher::ModelFingerprint` |

**Pro Komponente**

- `audit_trace` (66 Zeilen)
  - → `contextra-ports`: prod: `Clock`
- `context_edit_audit` (271 Zeilen)
  - → `contextra-ports`: prod: `Clock`
  - → `contextra-types`: prod: `TenantId`
- `egress_gateway` (597 Zeilen)
  - → `contextra-types`: prod: `TenantScoped`×6, `TenantId`×5, `TenantScoped::new`×4
- `egress_guard` (471 Zeilen)
  - → `contextra-ports`: dev: `Clock`
  - → `contextra-types`: prod: `TenantScoped`×2, `TenantId`×2
- `egress_vault` (1127 Zeilen)
  - → `contextra-types`: prod: `TenantScoped`×5, `TenantId`×4, `TenantScopeViolation`, `TenantScoped::new`
- `tests/audit_trace_tests.rs` (111 Zeilen)
  - → `contextra-ports`: dev: `Clock`
- `tests/campaign_j27_privacy_test.rs` (414 Zeilen)
  - → `contextra-ports`: dev: `Clock`
- `tests/context_edit_audit_chain.rs` (118 Zeilen)
  - → `contextra-ports`: dev: `Clock`
  - → `contextra-types`: dev: `TenantId`
- `tests/tenant_scoped_egress_isolation.rs` (107 Zeilen)
  - → `contextra-crypto`: dev: `crypto::KeyManager`, `kv_cipher::ModelFingerprint`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `audit_trace`
  - 2 fn, 1 trait
    - `fn extract_rule_id`
    - `fn compute_audit_trace`
    - `trait EgressClassifierTrace` · Methoden: classify_with_trace
- `avv_generator`
  - 1 fn, 1 struct
    - `struct AvvContext`
    - `fn generate_avv_draft`
- `bulk_exfiltration_detector`
  - 1 enum, 2 struct
    - `struct SessionId`
    - `enum BulkExfiltrationOutcome`
    - `struct BulkExfiltrationDetector` · 2 Methoden
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
  - 1 const, 13 fn, 3 struct, 2 trait, 1 type
    - `type DefaultEgressClassifier`
    - `const MAX_SEARCH_QUERY_BYTES`
    - `trait EgressGuardCheck` · Methoden: check, check_scoped
    - `trait InjectionDetector` · Methoden: detect
    - `fn extract_surrogate_tokens`
    - `struct CloudResponseRehydrator` · 4 Methoden
    - `fn pii_vault_forces_crypto_shred`
    - `fn resolve_effective_kv_delete_mode`
    - `struct CloudQueryRequest`
    - `struct CloudQueryResponse`
    - `fn handle_cloud_query`
    - `fn handle_cloud_query_scoped` ⚠ fremd: `contextra-types`
    - `fn handle_cloud_query_scoped_with_guard` ⚠ fremd: `contextra-types`
    - `fn handle_cloud_query_scoped_with_bulk_detector` ⚠ fremd: `contextra-types`
    - `fn handle_cloud_query_with_guard`
    - `fn check_bulk_exfiltration`
    - `fn handle_cloud_query_with_bulk_detector`
    - `fn process_cloud_response`
    - `fn guard_and_sanitize_payload`
    - `fn guard_and_sanitize_payload_scoped` ⚠ fremd: `contextra-types`
- `egress_guard`
  - 3 const, 2 struct, 1 trait
    - `struct TextSearchResult`
    - `trait TextSearchEngine` · Methoden: search_text
    - `const DEFAULT_EGRESS_GUARD_TIMEOUT`
    - `const DEFAULT_EGRESS_GUARD_THRESHOLD`
    - `const DEFAULT_EGRESS_GUARD_MIN_BYTES`
    - `struct EgressGuard` · 7 Methoden
- `egress_vault`
  - 2 const, 5 enum, 8 fn, 4 struct, 2 trait, 1 type
    - `const SURROGATE_PREFIX`
    - `fn is_luhn_valid`
    - `fn is_iban_valid`
    - `fn normalize_payload`
    - `trait EntityRecognizer` · Methoden: recognize
    - `struct NoOpRecognizer`
    - `struct SurrogateVault` · 6 Methoden
    - `type BoxFuture`
    - `enum PolicyCategory`
    - `enum BlockReason` · 1 Methoden
    - `enum EgressClassification`
    - `fn validate_luhn`
    - `fn validate_iban_mod97`
    - `fn normalize_payload_for_dlp`
    - `enum PatternValidator`
    - `struct CompiledPattern` · 3 Methoden
    - `enum EgressVaultError` ⚠ fremd: `contextra-types`
    - `const MAX_CLASSIFY_PAYLOAD_BYTES`
    - `fn classify_layer1`
    - `fn classify_layer1_arc`
    - `struct EgressVault` · 13 Methoden
    - `trait EgressClassifier` · Methoden: classify, classify_scoped
- `error`
  - 1 enum
    - `enum EgressError` · 3 Methoden
- `guarded_payload`
  - 3 struct
    - `struct Unsanitized`
    - `struct Sanitized`
    - `struct GuardedPayload` · 4 Methoden
- `processing_registry`
  - 1 enum, 2 fn, 2 struct, 1 type
    - `enum ProcessorRole` · 1 Methoden
    - `type TenantId`
    - `struct ProcessingActivityRecord`
    - `struct ProcessingRegistryExport`
    - `fn generate_registry`
    - `fn render_markdown`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-mcp` | `bulk_exfiltration_detector::*`, `egress_gateway::*`, `EgressError`, `egress_vault`, `InjectionDetector`, `EgressGuard` | `ContextEditAuditRecord`, `egress_gateway::EgressGuardCheck` |
| `contextra-router` | `guarded_payload`×2 | – |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

### contextra-py

- Pfad: `crates/contextra-py` · Zeilen: 2196 · Rust-Dateien: 16 · Ring: 4
- Interne Cargo-Abhängigkeiten: `contextra-adapt` (normal), `contextra-core` (normal), `contextra-db` (normal), `contextra-rank` (normal), `contextra-router` (normal), `contextra-store` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-core` | 13 | 1 | 3 | `ipc::ScoredDocument::create`×2, `ipc::ScoredDocumentArgs`×2, `ipc::SearchResponse::create`×2, `ipc::SearchResponseArgs`×2, `FusionWeights::new`×2, `ContextraError`, `ContextraErrorDto::from`, `types::domain` | – |
| `contextra-db` | 7 | 0 | 4 | `Collection`, `Document`, `SearchResult`, `Contextra`, `DistanceMetric::Cosine`, `DistanceMetric::Euclidean`, `DistanceMetric::DotProduct` | – |
| `contextra-types` | 1 | 0 | 1 | `TenantId::SYSTEM` | – |

**Pro Komponente**

- `bindings` (1931 Zeilen)
  - → `contextra-core`: prod: `ipc::ScoredDocument::create`×2, `ipc::ScoredDocumentArgs`×2, `ipc::SearchResponse::create`×2, `ipc::SearchResponseArgs`×2, `FusionWeights::new`×2, `ContextraError`, `ContextraErrorDto::from` · dev: `ContextraError`
  - → `contextra-db`: prod: `Collection`, `Document`, `SearchResult`, `Contextra`, `DistanceMetric::Cosine`, `DistanceMetric::Euclidean`, `DistanceMetric::DotProduct`
  - → `contextra-types`: prod: `TenantId::SYSTEM`
- `kv_links` (193 Zeilen)
  - → `contextra-core`: prod: `types::domain`

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
    - `fn contextra_err` ⚠ fremd: `contextra-core`
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
- contextra-rank: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-router: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-store: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-rank

- Pfad: `crates/contextra-rank` · Zeilen: 6069 · Rust-Dateien: 42 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-adapt` (dev), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 10 | 9 | 8 | `ConfigFingerprint`×3, `ContextraError`, `FusionWeights`, `FusionStrategy`, `SignalFusionStrategies`, `MAX_SEARCH_K`, `OnSignalFailure`, `DocId` | – |
| `contextra-adapt` | 0 | 2 | 0 | – | `PidController`×2 |

**Pro Komponente**

- `calibration` (635 Zeilen)
  - → `contextra-types`: prod: `ConfigFingerprint`×3
- `fusion` (2051 Zeilen)
  - → `contextra-types`: prod: `ContextraError`, `FusionWeights`, `FusionStrategy`, `SignalFusionStrategies`, `MAX_SEARCH_K`, `OnSignalFailure`, `DocId`
- `tests/calibration_deep_tests.rs` (315 Zeilen)
  - → `contextra-adapt`: dev: `PidController`
  - → `contextra-types`: dev: `ConfigFingerprint`
- `tests/calibration_stress_and_fault_tests.rs` (97 Zeilen)
  - → `contextra-adapt`: dev: `PidController`
- `tests/conformal_tests.rs` (176 Zeilen)
  - → `contextra-types`: dev: `ConfigFingerprint`
- `tests/dibud_equivalence.rs` (210 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/global_fusion_community_aware.rs` (111 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/j12closure-contextra-rank_dibud_provenance.rs` (24 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/rank_tests.rs` (290 Zeilen)
  - → `contextra-types`: dev: `ConfigFingerprint`
- `tests/rrf_reference.rs` (99 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/rrf_two_phase_scaling.rs` (173 Zeilen)
  - → `contextra-types`: dev: `FusionStrategy`
- `tests/score_normalized_fallback.rs` (158 Zeilen)
  - → `contextra-types`: dev: `FusionStrategy`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `calibration`
  - 1 const, 1 enum, 3 struct, 1 trait
    - `enum ConformalError`
    - `trait ConformalCalibrator` · Methoden: update, threshold
    - `struct AdaptiveConformalCalibrator` · 8 Methoden
    - `const DEFAULT_ECE_REBUILD_THRESHOLD`
    - `struct IsotonicCalibrator` · 13 Methoden
    - `struct PlattScaler` · 11 Methoden
- `dibud`
  - 2 enum, 2 fn, 3 struct
    - `fn fuse_exact_prefix`
    - `fn fuse_exact_prefix_async`
    - `enum DiBudStep`
    - `struct DiBudFusionState` · 8 Methoden
    - `enum BudgetedChannel` · 3 Methoden
    - `struct FusionBudget` · 1 Methoden
    - `struct DiBudOutcome`
- `drift`
  - 1 enum, 1 struct
    - `enum DriftStatus` · 1 Methoden
    - `struct DriftDetector` · 6 Methoden
- `explain`
  - 1 enum, 1 fn, 2 struct
    - `enum SignalKind` · 1 Methoden
    - `struct ExplanationEntry`
    - `struct RetrievalExplanation` · 1 Methoden
    - `fn explain`
- `fusion`
  - 2 struct
    - `struct ProvenanceBuilder` *(re-exportiert)* · 10 Methoden
    - `struct BoundedTopK` *(re-exportiert)* · 6 Methoden
- Zusätzlich 25 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::FusionStrategy`, `contextra_types::OnSignalFailure`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `IsotonicCalibrator`×2, `fusion`, `fusion::fuse_search_results_with_strategy`, `fusion::fuse_search_results_with_signal_strategies`, `fusion::fuse_signals` | – |
| `contextra-infer-onnx` | `PlattScaler`×4 | – |
| `contextra-mcp` | `fusion::ProvenanceRecord`, `IsotonicCalibrator`, `IsotonicCalibrator::with_defaults` | – |
| `contextra` | `(crate)` | – |
| `contextra-infer-candle` | `IsotonicCalibrator` | – |
| `contextra-infer-ollama` | `IsotonicCalibrator` | – |
| `contextra-db` | – | `IsotonicCalibrator::new` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 19 von 21)**: `ConformalError`, `ConformalCalibrator`, `AdaptiveConformalCalibrator`, `DEFAULT_ECE_REBUILD_THRESHOLD`, `fuse_exact_prefix`, `fuse_exact_prefix_async`, `DiBudStep`, `DiBudFusionState`, `BudgetedChannel`, `FusionBudget`, `DiBudOutcome`, `DriftStatus`, `DriftDetector`, `SignalKind`, `ExplanationEntry`, `RetrievalExplanation`, `explain`, `ProvenanceBuilder`, `BoundedTopK`

**Auffälligkeiten**

- contextra-ports: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-router

- Pfad: `crates/contextra-router` · Zeilen: 8045 · Rust-Dateien: 43 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-db` (dev), `contextra-graph` (dev), `contextra-text` (dev), `contextra-vector` (dev), `contextra-adapt` (normal), `contextra-ports` (normal), `contextra-privacy` (normal), `contextra-types` (normal), `contextra-wire` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-adapt` | 4 | 6 | 2 | `bandit`, `lyapunov`, `offpolicy`, `offpolicy::RandomizedLoggingPolicy` | `bandit::BanditProfileState`×4, `BanditError`, `FcTsError::InvalidConfig` |
| `contextra-privacy` | 2 | 0 | 1 | `guarded_payload`×2 | – |
| `contextra-types` | 1 | 16 | 1 | `RetrievalStrategy` | `ConfigFingerprint`×7, `TokenBudget`×4, `DocId::new`×3, `Result` |
| `contextra-db` | 0 | 13 | 0 | – | `ContextraConfig`×5, `Contextra::open_with_config`×5, `Collection`, `context::ContextManager::new`, `Collection::new` |
| `contextra-graph` | 0 | 1 | 0 | – | `CsrGraph::new` |
| `contextra-ports` | 0 | 6 | 0 | – | `StorageEngine`×3, `HybridSearchProvider`, `CommunityResolver`, `ContextPreparer` |
| `contextra-text` | 0 | 1 | 0 | – | `Language::English` |
| `contextra-vector` | 0 | 2 | 0 | – | `HnswConfig::default`, `HnswIndex::try_new` |

**Pro Komponente**

- `(crate-root)` (62 Zeilen)
  - → `contextra-adapt`: prod: `bandit`, `lyapunov`, `offpolicy`
  - → `contextra-privacy`: prod: `guarded_payload`×2
- `arm_registry` (158 Zeilen)
  - → `contextra-types`: prod: `RetrievalStrategy`
- `bandit_regret_tests` (257 Zeilen)
  - → `contextra-adapt`: dev: `BanditError`
- `router` (1367 Zeilen)
  - → `contextra-adapt`: prod: `offpolicy::RandomizedLoggingPolicy`
  - → `contextra-db`: dev: `ContextraConfig`×4, `Contextra::open_with_config`×4
  - → `contextra-types`: dev: `TokenBudget`
- `tests` (3220 Zeilen)
  - → `contextra-db`: dev: `Collection::new`, `ContextraConfig`, `Contextra::open_with_config`
  - → `contextra-graph`: dev: `CsrGraph::new`
  - → `contextra-ports`: dev: `StorageEngine`×3
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: dev: `ConfigFingerprint`×7, `DocId::new`×3, `TokenBudget`×2, `Result`
  - → `contextra-vector`: dev: `HnswConfig::default`, `HnswIndex::try_new`
- `benches/cost_routing_savings_bench.rs` (144 Zeilen)
  - → `contextra-types`: dev: `TokenBudget`
- `benches/router_bench.rs` (176 Zeilen)
  - → `contextra-db`: dev: `Collection`, `context::ContextManager::new`
  - → `contextra-ports`: dev: `HybridSearchProvider`, `CommunityResolver`, `ContextPreparer`
- `tests/arm_registry_five_arms.rs` (50 Zeilen)
  - → `contextra-types`: dev: `RetrievalStrategy`
- `tests/fc_ts_dispatch_test.rs` (85 Zeilen)
  - → `contextra-adapt`: dev: `FcTsError::InvalidConfig`
- `tests/feature_cleanup_validation.rs` (106 Zeilen)
  - → `contextra-adapt`: dev: `bandit::BanditProfileState`
- `tests/j10-contextra-router-router-lifecycle-outcom_test.rs` (230 Zeilen)
  - → `contextra-adapt`: dev: `bandit::BanditProfileState`
- `tests/regret_monotonicity.rs` (248 Zeilen)
  - → `contextra-adapt`: dev: `bandit::BanditProfileState`
- `tests/router_bandit_dispatch.rs` (238 Zeilen)
  - → `contextra-adapt`: dev: `bandit::BanditProfileState`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `arm_registry`
  - 1 enum, 1 struct
    - `struct ArmRegistry` · 6 Methoden
    - `enum ArmRegistryError`
- `dispatch`
  - 2 fn
    - `fn dispatch_to_slm`
    - `fn route_and_dispatch_to_slm`
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
  - 1 enum, 3 struct
    - `enum QuantizationLevel`
    - `struct SlmProfile` · 7 Methoden
    - `struct ConformalCalibrator` · 4 Methoden
    - `struct ProfileCalibrationState` · 9 Methoden
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
| `contextra-agent` | `DefaultRouterEngine`, `DecisionId`, `RoutingOutcome::Success`, `RoutingOutcome::Rejected` | – |
| `xtask-heavy` | `BanditProfileState`, `BanditImplementation::DiagonalApproximation` | – |
| `contextra` | `(crate)` | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 21 von 23)**: `ArmRegistry`, `ArmRegistryError`, `dispatch_to_slm`, `route_and_dispatch_to_slm`, `FcTsDispatchError`, `select_profile_fc_ts`, `deterministic_fc_ts_rng`, `DecisionIdGenerator`, `RoutingOutcome`, `QuantizationLevel`, `SlmProfile`, `ConformalCalibrator`, `ProfileCalibrationState`, `ConfidenceMetrics`, `RoutingDecision`, `RouterState`, `RouterEngine`, `RoutingStrategy`, `serialize`, `deserialize`, `Transport`

**Auffälligkeiten**

- contextra-ports: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-wire: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-sandbox

- Pfad: `crates/contextra-sandbox` · Zeilen: 2956 · Rust-Dateien: 15 · Ring: 2
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
    - `struct CapabilityViolationError`
    - `struct WasmExecutor` · 3 Methoden
- `merge`
  - 1 struct
    - `struct WasmMergeFunction` · 3 Methoden
- `output`
  - 1 struct
    - `struct WasmOutput` · 1 Methoden
- `wasi`
  - 2 struct
    - `struct ProcessExitError`
    - `struct OutputLimitExceededError`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-store` | – | `MergeOperatorCapabilities` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 13 von 14)**: `ApprovalRisk`, `classify_risk`, `ApprovalStatus`, `ApprovalTransitionError`, `ApprovalRequest`, `WasmCapabilities`, `SandboxError`, `CapabilityViolationError`, `WasmExecutor`, `WasmMergeFunction`, `WasmOutput`, `ProcessExitError`, `OutputLimitExceededError`

### contextra-simd

- Pfad: `crates/contextra-simd` · Zeilen: 1472 · Rust-Dateien: 9 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-core` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-core` | 5 | 0 | 2 | `Result`×4, `ContextraError` | – |

**Pro Komponente**

- `(crate-root)` (70 Zeilen)
  - → `contextra-core`: prod: `Result`×4
- `dispatch` (203 Zeilen)
  - → `contextra-core`: prod: `ContextraError`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 4 fn
    - `fn validate_vector` ⚠ fremd: `contextra-core`
    - `fn compute_distance` ⚠ fremd: `contextra-core`
    - `fn compute_distance_trusted` ⚠ fremd: `contextra-core`
    - `fn compute_distance_f32_bytes_trusted` ⚠ fremd: `contextra-core`
- `dispatch`
  - 9 fn
    - `fn cosine_distance` ⚠ fremd: `contextra-core`
    - `fn euclidean_distance` ⚠ fremd: `contextra-core`
    - `fn dot_product_distance` ⚠ fremd: `contextra-core`
    - `fn cosine_distance_f32_bytes` ⚠ fremd: `contextra-core`
    - `fn euclidean_distance_f32_bytes` ⚠ fremd: `contextra-core`
    - `fn dot_product_distance_f32_bytes` ⚠ fremd: `contextra-core`
    - `fn dot_product_u8` ⚠ fremd: `contextra-core`
    - `fn euclidean_distance_sq_u8` ⚠ fremd: `contextra-core`
    - `fn cosine_similarity_parts_u8` ⚠ fremd: `contextra-core`
- `kernels`
  - 38 fn, 2 struct
    - `fn cosine_distance_avx2`
    - `fn euclidean_distance_avx2`
    - `fn dot_product_avx2`
    - `fn cosine_distance_f32_bytes_avx2`
    - `fn euclidean_distance_f32_bytes_avx2`
    - `fn dot_product_f32_bytes_avx2`
    - `fn hsum256_ps_avx`
    - `fn dot_product_u8_avx2`
    - `fn euclidean_distance_sq_u8_avx2`
    - `fn cosine_similarity_parts_u8_avx2`
    - `fn hsum256_epi32_avx2`
    - `fn cosine_distance_avx512`
    - `fn euclidean_distance_avx512`
    - `fn dot_product_avx512`
    - `fn cosine_distance_f32_bytes_avx512`
    - `fn euclidean_distance_f32_bytes_avx512`
    - `fn dot_product_f32_bytes_avx512`
    - `fn hsum512_ps_avx`
    - `fn dot_product_u8_avx512vnni`
    - `fn euclidean_distance_sq_u8_avx512`
    - `fn cosine_similarity_parts_u8_avx512`
    - `fn hsum512_epi32_avx512`
    - `fn cosine_distance_neon`
    - `fn euclidean_distance_neon`
    - `fn dot_product_neon`
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

**Nach außen ungenutzte öffentliche Items (Heuristik, 46 von 53)**: `cosine_distance`, `euclidean_distance`, `dot_product_distance`, `cosine_distance_f32_bytes`, `euclidean_distance_f32_bytes`, `dot_product_distance_f32_bytes`, `dot_product_u8`, `euclidean_distance_sq_u8`, `cosine_similarity_parts_u8`, `cosine_distance_avx2`, `euclidean_distance_avx2`, `dot_product_avx2`, `cosine_distance_f32_bytes_avx2`, `euclidean_distance_f32_bytes_avx2`, `dot_product_f32_bytes_avx2`, `hsum256_ps_avx`, `dot_product_u8_avx2`, `euclidean_distance_sq_u8_avx2`, `cosine_similarity_parts_u8_avx2`, `hsum256_epi32_avx2`, `cosine_distance_avx512`, `euclidean_distance_avx512`, `dot_product_avx512`, `cosine_distance_f32_bytes_avx512`, `euclidean_distance_f32_bytes_avx512`, `dot_product_f32_bytes_avx512`, `hsum512_ps_avx`, `dot_product_u8_avx512vnni`, `euclidean_distance_sq_u8_avx512`, `cosine_similarity_parts_u8_avx512` …

### contextra-store

- Pfad: `crates/contextra-store` · Zeilen: 46092 · Rust-Dateien: 231 · Ring: 1
- Interne Cargo-Abhängigkeiten: `contextra-graph` (dev), `contextra-sandbox` (dev), `contextra-testkit` (dev), `contextra-vector` (dev), `contextra-core` (normal), `contextra-crypto` (normal), `contextra-mvcc` (normal), `contextra-ports` (normal), `contextra-sys` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-core` | 35 | 115 | 14 | `MAX_SCAN_MERGE_ACCUMULATOR`×6, `ContextraError::Storage`×5, `TOMBSTONE_BIT`×4, `TxId`×4, `ResourceTracker`×2, `ContextraError::MemoryBudgetExceeded`×2, `ContextraError::Internal`×2, `Result`×2, `ContextraError`×2, `TxId::INTERNAL_BASE`×2 … (+4) | `ResourceBudget`×28, `ResourceTracker::new`×27, `StorageEngine`×2, `SnapshotRegistry::new`×2, `ContextraError::InvalidInput`×2 … (+4) |
| `contextra-ports` | 14 | 10 | 5 | `MetricsSink`×6, `Clock`×4, `SystemClock::new`, `license::FeatureRing`, `license::FeatureRing::Sovereign`, `NoopMetricsSink` | `StorageEngine`×4, `StorageStats`, `storage::StorageEngine`, `SystemClock` |
| `contextra-crypto` | 5 | 2 | 5 | `wal_crypto`×2, `verify_wal_chain_completeness`, `wal_crypto::WalHmac`, `crypto::KeyManager` | – |
| `contextra-mvcc` | 5 | 1 | 5 | `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`×2, `SequenceLogSsiValidator`, `SsiValidator`, `SequenceLogSsiValidator::new_with_bounds` | – |
| `contextra-sys` | 3 | 3 | 2 | `mmap_readonly`×2, `set_restrictive_file_acl` | `reopen_and_dup2`×2, `verify_file_acl_owner_only` |
| `contextra-sandbox` | 0 | 1 | 0 | – | `MergeOperatorCapabilities` |
| `contextra-testkit` | 0 | 7 | 0 | – | `ReferenceModel`×4, `ManualClock`×3 |
| `contextra-types` | 0 | 2 | 0 | – | `TxId`, `TenantId` |

**Pro Komponente**

- `compaction` (3215 Zeilen)
  - → `contextra-core`: prod: `ResourceTracker`×2, `ContextraError::MemoryBudgetExceeded`×2, `ContextraError::Internal`×2, `ContextraError::Sandbox`, `Result`, `TOMBSTONE_BIT` · dev: `ResourceTracker::new`×26, `ResourceBudget`×26, `TxId::new`×3, `ContextraError::MemoryBudgetExceeded`×2 … (+3)
  - → `contextra-ports`: prod: `Clock`×2, `SystemClock::new`
- `kv` (211 Zeilen)
  - → `contextra-core`: prod: `ContextraError`
- `lsm` (6144 Zeilen)
  - → `contextra-core`: prod: `MAX_SCAN_MERGE_ACCUMULATOR`×6, `ContextraError::Storage`×5, `TxId`×4, `ContextraError`, `ContextraError::DurabilityConfig`, `Result`, `TxId::new`, `TxId::INTERNAL_BASE`, `StorageStats` · dev: `TOMBSTONE_BIT`
  - → `contextra-crypto`: prod: `verify_wal_chain_completeness`
  - → `contextra-mvcc`: prod: `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`, `SequenceLogSsiValidator`, `SsiValidator`, `SequenceLogSsiValidator::new_with_bounds`
  - → `contextra-ports`: prod: `MetricsSink`×5, `Clock`×2, `license::FeatureRing`, `license::FeatureRing::Sovereign`, `NoopMetricsSink`
- `memtable` (918 Zeilen)
  - → `contextra-core`: dev: `TOMBSTONE_BIT`
  - → `contextra-mvcc`: prod: `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`
- `sstable` (3316 Zeilen)
  - → `contextra-core`: prod: `TOMBSTONE_BIT`×3, `TxId::INTERNAL_BASE` · dev: `ContextraError`
- `system_pressure` (321 Zeilen)
  - → `contextra-ports`: prod: `MetricsSink`
- `wal` (7189 Zeilen)
  - → `contextra-core`: dev: `TxId`×6
  - → `contextra-crypto`: prod: `wal_crypto`×2, `wal_crypto::WalHmac`, `crypto::KeyManager` · dev: `wal_crypto::WalHmac`, `crypto::KeyManager`
  - → `contextra-sys`: prod: `mmap_readonly`×2, `set_restrictive_file_acl` · dev: `verify_file_acl_owner_only`
- `benches/wal_bench.rs` (140 Zeilen)
  - → `contextra-core`: dev: `TxId`
- `fuzz/fuzz_targets/fuzz_recovery_arbitrary_state.rs` (52 Zeilen)
  - → `contextra-core`: dev: `StorageEngine`
- `fuzz/fuzz_targets/wal_mutation_chaos.rs` (272 Zeilen)
  - → `contextra-core`: dev: `TxId`
- `tests/adaptive_compaction_determinism.rs` (38 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`
- `tests/blocking_util_pool_measurement.rs` (90 Zeilen)
  - → `contextra-ports`: dev: `MetricsSink`
- `tests/budget_drift.rs` (54 Zeilen)
  - → `contextra-mvcc`: dev: `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`
- `tests/campaign_j11_memory_deletion.rs` (48 Zeilen)
  - → `contextra-ports`: dev: `storage::StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/campaign_j41_store.rs` (36 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/chaos_dropped_write.rs` (145 Zeilen)
  - → `contextra-sys`: dev: `reopen_and_dup2`×2
- `tests/compaction_contiguous_selection.rs` (153 Zeilen)
  - → `contextra-core`: dev: `StorageStats`
- `tests/compaction_key_range_selection.rs` (101 Zeilen)
  - → `contextra-core`: dev: `ResourceTracker`, `SnapshotRegistry::new`, `ResourceBudget`
- `tests/compaction_merge_operator.rs` (508 Zeilen)
  - → `contextra-sandbox`: dev: `MergeOperatorCapabilities`
  - → `contextra-testkit`: dev: `ManualClock`
- `tests/crash_prefix_enumeration.rs` (132 Zeilen)
  - → `contextra-testkit`: dev: `ReferenceModel`
- `tests/deletion_proof_unregistered.rs` (51 Zeilen)
  - → `contextra-core`: dev: `ContextraError`
- `tests/dirlock_fail_closed.rs` (167 Zeilen)
  - → `contextra-core`: dev: `ContextraError`
- `tests/durability_mode_feature_gate.rs` (74 Zeilen)
  - → `contextra-ports`: dev: `license::FeatureRing`
- `tests/group_commit_test.rs` (222 Zeilen)
  - → `contextra-core`: dev: `ContextraError`
- `tests/j07-contextra-store-lsm-engine-kv-segment-te_integration_test.rs` (241 Zeilen)
  - → `contextra-core`: dev: `StorageStats`
  - → `contextra-ports`: dev: `SystemClock`, `Clock`
- `tests/j07closure-contextra-store_wiring_test.rs` (115 Zeilen)
  - → `contextra-core`: dev: `Result`×2, `TxId`, `SnapshotRegistry::new`, `ResourceTracker::new` … (+1)
- `tests/kv_delete_mode_wiring_test.rs` (107 Zeilen)
  - → `contextra-core`: dev: `ContextraError`
- `tests/kv_deletion_proof_registry.rs` (68 Zeilen)
  - → `contextra-core`: dev: `ContextraError`
- `tests/lsm_storage_merge_operator.rs` (111 Zeilen)
  - → `contextra-core`: dev: `ContextraError::InvalidInput`×2
- `tests/manifest_replace_rank_compat.rs` (68 Zeilen)
  - → `contextra-core`: dev: `Result`
- `tests/model_based_mvcc_flush.rs` (184 Zeilen)
  - → `contextra-testkit`: dev: `ReferenceModel`
- `tests/observer_deadline_no_runtime_block.rs` (67 Zeilen)
  - → `contextra-core`: dev: `TxId`
- `tests/proptest_model_based.rs` (236 Zeilen)
  - → `contextra-core`: dev: `Result`
- `tests/scan_limits_and_ranges.rs` (59 Zeilen)
  - → `contextra-core`: dev: `TxId`
- `tests/ssi_tracked_read_seq_clamp.rs` (122 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/ssi_tracked_reads_via_trait.rs` (89 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/sstable_format_v3_to_v4_compat.rs` (158 Zeilen)
  - → `contextra-core`: dev: `TOMBSTONE_BIT`
- `tests/support/mod.rs` (79 Zeilen)
  - → `contextra-core`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ReferenceModel`
- `tests/tenant_codec_isolation_props.rs` (226 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/tenant_scoped_storage.rs` (131 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/test_wp02_flush_mvcc_bug.rs` (38 Zeilen)
  - → `contextra-core`: dev: `Result`
  - → `contextra-testkit`: dev: `ReferenceModel`
- `tests/toctou_put_if_absent.rs` (209 Zeilen)
  - → `contextra-core`: dev: `error::ContextraError`, `types::*`, `Result`, `traits`
- `tests/wal_backpressure.rs` (294 Zeilen)
  - → `contextra-core`: dev: `TxId`
- `tests/wal_flusher_rollback.rs` (92 Zeilen)
  - → `contextra-core`: dev: `Result`×2, `TxId`
- `tests/wal_legacy_key_forced_migration.rs` (228 Zeilen)
  - → `contextra-core`: dev: `TxId`
- `tests/wal_legacy_key_migration_store.rs` (93 Zeilen)
  - → `contextra-core`: dev: `TxId`
- `tests/wal_legacy_key_retirement_marker.rs` (109 Zeilen)
  - → `contextra-core`: dev: `TxId`
- `tests/wal_legacy_migration_crash.rs` (44 Zeilen)
  - → `contextra-core`: dev: `TxId`
- `tests/wal_no_hmac_replay.rs` (52 Zeilen)
  - → `contextra-core`: dev: `TxId`
- `tests/wal_observer_contract.rs` (149 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `tests/wal_observer_integration.rs` (268 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `tests/wal_recovery_runtime_stall.rs` (116 Zeilen)
  - → `contextra-core`: dev: `TxId`
- `tests/wal_systematic_crash.rs` (198 Zeilen)
  - → `contextra-core`: dev: `ContextraError::Storage`×2
- `tests/wal_torn_write_fault_injection.rs` (113 Zeilen)
  - → `contextra-core`: dev: `TxId`
- `tests/wal_truncate_ordering.rs` (257 Zeilen)
  - → `contextra-core`: dev: `ContextraError`

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
    - `struct LsmStorage` *(re-exportiert)* · 37 Methoden ⚠ fremd: `contextra-mvcc`
    - `const MAX_KEY_SIZE`
    - `const MAX_VALUE_SIZE`
    - `const MAX_BATCH_SIZE`
    - `const MAX_INTERNAL_MERGE_ENTRIES_FACTOR`
    - `const MAX_GROUP_COMMIT_BATCH_SIZE`
    - `const MIN_ENTRIES_FOR_SSTABLE_REBUILD`
    - `const DEFAULT_MAX_OBSERVER_LATENCY`
    - `enum WriteOrigin`
    - `struct WalEntryRef` · 6 Methoden
    - `struct CommitContext` ⚠ fremd: `contextra-core`
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
    - `struct SstableBuilder` *(re-exportiert)* · 7 Methoden
    - `struct SstableReader` *(re-exportiert)* · 13 Methoden
    - `struct SstableStream` *(re-exportiert)* · 2 Methoden
- `system_pressure`
  - 6 const, 1 enum, 2 struct
    - `const WAL_QUEUE_CRITICAL_THRESHOLD`
    - `const WAL_QUEUE_ELEVATED_THRESHOLD`
    - `const SCHEDULER_QUEUE_CRITICAL`
    - `const SCHEDULER_QUEUE_ELEVATED`
    - `const BLOCKING_UTIL_CRITICAL`
    - `const BLOCKING_UTIL_ELEVATED`
    - `enum PressureLevel`
    - `struct SystemPressure`
    - `struct SystemPressureMonitor` · 4 Methoden
- `tenant_codec`
  - 2 struct, 1 trait
    - `struct TenantKeyCodec` · 6 Methoden
    - `trait StorageEngineHandle` · Methoden: get_handle, get_at_seq_handle, get_tracked_handle, get_at_seq_tracked_handle, scan_prefix_tracked_handle, supports_ssi_tracking_handle…
    - `struct TenantScopedStorage` · 2 Methoden
- `wal`
  - 5 const, 4 enum, 2 fn, 6 static, 7 struct, 1 trait, 1 type
    - `enum WalParseError`
  - … 25 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 27 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_crypto::crypto::KeyManager`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `LsmStorage`×6, `tenant_codec::TenantScopedStorage`×5, `kv::KvSegmentManager`×4, `tenant_codec::TenantScopedStorage::new`×4, `SystemPressure`×3, `kv::KvSegmentPayload`×2, `tenant_codec::TenantKeyCodec`×2, `PressureLevel::Critical`, `LsmConfig`, `lsm::DurabilityMode` … (+1) | `lsm`×10, `compaction`×2, `sstable`×2 |
| `contextra-agent` | `LsmStorage`×3 | – |
| `contextra` | `kv::delete_mode::KvDeleteMode`, `lsm::config` | `lsm::config::DurabilityMode`, `lsm::DurabilityMode` |
| `contextra-bench` | – | `lsm` |
| `contextra-checkpoint` | – | `lsm` |
| `contextra-cognition` | – | `LsmStorage`×8, `LsmConfig`×8 |
| `contextra-db` | – | `LsmConfig`×5, `LsmStorage::new`×4, `LsmStorage`×2, `lsm` |
| `contextra-graph` | – | `LsmConfig`, `LsmStorage::new` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 95 von 105)**: `WorkloadMetrics`, `WorkloadMetricsSnapshot`, `CompactionStrategy`, `AdaptiveCompactionPlan`, `AdaptiveCompactionPlanner`, `CostBasedAdaptivePlanner`, `CompactionConfig`, `CompactionEngine`, `retain_key_versions`, `TtlMetadata`, `InMemoryStorageEngine`, `KvSegmentConfig`, `DurabilityConfigError`, `StorageHealth`, `MAX_KEY_SIZE`, `MAX_VALUE_SIZE`, `MAX_BATCH_SIZE`, `MAX_INTERNAL_MERGE_ENTRIES_FACTOR`, `MAX_GROUP_COMMIT_BATCH_SIZE`, `MIN_ENTRIES_FOR_SSTABLE_REBUILD`, `DEFAULT_MAX_OBSERVER_LATENCY`, `WriteOrigin`, `WalEntryRef`, `CommitContext`, `CommittedBatch`, `WalObserver`, `ObserverRegistry`, `AsyncObserverAdapter`, `Manifest`, `ManifestEntry` …

**Auffälligkeiten**

- contextra-graph: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-types: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-vector: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-sys

- Pfad: `crates/contextra-sys` · Zeilen: 632 · Rust-Dateien: 6 · Ring: 0
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `acl_win32`
  - 4 fn
    - `fn set_restrictive_file_acl`
    - `fn verify_file_acl_owner_only`
    - `fn verify_file_acl_owner_only`
    - `fn set_restrictive_file_acl`
- `mlock`
  - 6 fn
    - `fn mem_lock`
    - `fn mem_unlock`
    - `fn mem_lock`
    - `fn mem_unlock`
    - `fn mem_lock`
    - `fn mem_unlock`
- `mmap`
  - 1 fn
    - `fn mmap_readonly`
- `posix`
  - 2 fn
    - `fn reopen_and_dup2`
    - `fn reopen_and_dup2`
- `vault`
  - 1 struct
    - `struct LockedRegions` · 5 Methoden

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-store` | `mmap_readonly`×2, `set_restrictive_file_acl` | `reopen_and_dup2`×2, `verify_file_acl_owner_only` |
| `contextra-vector` | `mmap_readonly`×2 | – |
| `contextra-db` | `LockedRegions` | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 6 von 14)**: `mem_lock`, `mem_unlock`, `mem_lock`, `mem_unlock`, `mem_lock`, `mem_unlock`

### contextra-testkit

- Pfad: `crates/contextra-testkit` · Zeilen: 916 · Rust-Dateien: 9 · Ring: tooling
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 1 | 0 | 1 | `Clock` | – |

**Pro Komponente**

- `manual_clock` (92 Zeilen)
  - → `contextra-ports`: prod: `Clock`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `fault_vfs`
  - 2 struct
    - `struct FaultConfig`
    - `struct FaultVfs` · 10 Methoden
- `in_memory_store`
  - 1 struct
    - `struct InMemoryStorageEngine` · 1 Methoden
- `manual_clock`
  - 1 struct
    - `struct ManualClock` · 6 Methoden
- `reference_model`
  - 1 enum, 1 struct
    - `enum RefOp`
    - `struct ReferenceModel` · 13 Methoden

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | – | `InMemoryStorageEngine` |
| `contextra-store` | – | `ReferenceModel`×4, `ManualClock`×3 |

**Nach außen ungenutzte öffentliche Items (Heuristik, 3 von 6)**: `FaultConfig`, `FaultVfs`, `RefOp`

**Auffälligkeiten**

- contextra-types: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-text

- Pfad: `crates/contextra-text` · Zeilen: 10725 · Rust-Dateien: 45 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 5 | 81 | 3 | `StorageEngine`×5 | `BoxFuture`×50, `StorageStats`×31 |
| `contextra-types` | 0 | 132 | 0 | – | `Result`×67, `TOMBSTONE_BIT`×52, `TxId`×7, `TxId::new`×2, `ContextraError::CapabilityUnsupported`×2 … (+2) |

**Pro Komponente**

- `(crate-root)` (225 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine`×3 · dev: `BoxFuture`
- `inverted` (1963 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×3
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6, `ContextChunk`
- `stream` (125 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine`
- `wand` (753 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine` · dev: `StorageStats`×2, `BoxFuture`
  - → `contextra-types`: dev: `TxId`×7, `TOMBSTONE_BIT`×6, `TxId::new`×2
- `tests/alloc_profiler.rs` (448 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `Result`×16
- `tests/audit_inspection.rs` (563 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`×32, `StorageStats`×4
  - → `contextra-types`: dev: `Result`×34, `TOMBSTONE_BIT`×10
- `tests/bm25_candidate_stream.rs` (306 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
- `tests/concurrent_metadata.rs` (195 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
- `tests/docid_128_tombstone_resolution.rs` (242 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`×16, `StorageStats`×2
  - → `contextra-types`: dev: `Result`×17, `TOMBSTONE_BIT`×6
- `tests/j31closure-contextra-text_closure_verification_test.rs` (64 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/pipeline_identity_regression.rs` (270 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6
- `tests/proptest_bm25_invariants.rs` (334 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6
- `tests/text_search_at_wrappers.rs` (322 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6, `ContextraError::CapabilityUnsupported`×2
- `tests/tombstone_read_your_writes.rs` (266 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6
- `tests/tombstone_update.rs` (220 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
- `tests/wand_tie_break_threshold.rs` (221 Zeilen)
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
  - 2 struct
    - `struct InvertedIndex` *(re-exportiert)* · 9 Methoden
    - `struct BM25MorphIndex` *(re-exportiert)* · 2 Methoden
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
  - 1 fn, 2 struct, 1 trait
    - `trait Tokenizer` · Methoden: tokenize
    - `struct DefaultTokenizer`
    - `struct GermanMorphTokenizer` · 1 Methoden
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
| `contextra-engine` | `Language`×2, `inverted::InvertedIndex` | `Language::English`×64, `Language::German`×4 |
| `contextra-agent` | – | `Language::English`×5 |
| `contextra-db` | – | `Language::English`×7, `Language`×2, `bm25::score_term`, `inverted::InvertedIndex` |
| `contextra-router` | – | `Language::English` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 40 von 42)**: `BM25_K1`, `BM25_B`, `BM25`, `score_term_with_params`, `FieldId`, `FieldWeight`, `BM25F`, `score_term_bm25f`, `LegalDomainVocabulary`, `get_legal_compound_stems`, `get_legal_protected_terms`, `LEGAL_COMPOUND_STEMS`, `LEGAL_PROTECTED_TERMS`, `MedicalDomainVocabulary`, `get_medical_compound_stems`, `get_medical_protected_terms`, `MEDICAL_COMPOUND_STEMS`, `MEDICAL_PROTECTED_TERMS`, `DomainVocabulary`, `BM25MorphIndex`, `Bm25Scorer`, `TokenReductionMetrics`, `PassthroughTokenizer`, `POSTING_LIST_V2_MAGIC`, `BLOCK_SIZE`, `DocIdRaw`, `DocIdRaw`, `PostingBlockInfo`, `Posting`, `PostingList` …

**Auffälligkeiten**

- contextra-types: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?

### contextra-types

- Pfad: `crates/contextra-types` · Zeilen: 6761 · Rust-Dateien: 29 · Ring: 0
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
    - `enum ContextraError` · 17 Methoden
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
    - `enum DocIdWidth` · 3 Methoden
    - `enum ManifestSchemaVersion` · 7 Methoden
- `tenant_scope`
  - 1 enum, 1 struct
    - `struct TenantScoped` · 4 Methoden
    - `enum TenantScopeViolation`
- `tombstone`
  - 1 struct, 1 trait
    - `trait TombstoneSemanticsCheck` · Methoden: is_tombstone, make_tombstone
    - `struct SeqBitTombstone`
- `types`
  - 3 const, 12 enum, 2 fn, 32 struct, 2 type
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
    - `struct EntityId` *(re-exportiert)* · 6 Methoden
    - `struct TxId` *(re-exportiert)* · 7 Methoden
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
    - `enum MemoryType` *(re-exportiert)* · 4 Methoden
    - `struct TlHfdParams` *(re-exportiert)*
    - `enum PprAlgorithm` *(re-exportiert)*
    - `struct PprConfig` *(re-exportiert)*
    - `struct ConfigFingerprint` *(re-exportiert)* · 4 Methoden
    - `enum AutoExtractionMode` *(re-exportiert)* · 1 Methoden
    - `enum FilterExpr` · 1 Methoden
    - `struct ImportanceScore` · 2 Methoden
    - `enum DecayFunction` · 2 Methoden
    - `struct MemoryImportance` · 2 Methoden
    - `struct SaturatingU8` · 2 Methoden
    - `enum FusionStrategy`
    - `struct SignalFusionStrategies` · 2 Methoden
    - `enum GraphTraversalStrategy`
    - `struct FusionWeights` · 5 Methoden
    - `struct ContextChunk` · 3 Methoden
    - `struct ContextWindow`
    - `enum Signal`
    - `enum OnSignalFailure`
    - `struct QueryRewriteOutput` · 4 Methoden
    - `struct ScoredEntry`
    - `struct HybridQuery` · 11 Methoden
    - `type QueryOptions`
    - `struct HybridQueryBuilder` · 17 Methoden
    - `type QueryOptionsBuilder`
    - `struct SaturatingU16` · 2 Methoden
- Interne Re-Exporte (`pub use`): `RerankResult`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `ContextraError::invalid_input`×50, `Result`×23, `ContextraError::Internal`×19, `EntityId`×14, `MAX_SEARCH_K`×12, `ContextraError::Serialization`×8, `ImportanceScore::new`×8, `domain::MemoryLink`×7, `HybridQuery`×7, `EntityId::from_key`×7 … (+42) | `ContextraError::from`×10, `domain::LinkRelation::Elaborates`×2, `ContextraError::HnswConnectivityDegraded`, `FusionWeights::new` … (+8) |
| `contextra-graph` | `EntityId`×14, `ContextraError::InvalidInput`×5, `ContextraError`×4, `PprConfig`×3, `Result`×3, `Edge::new`×2, `TxId`×2, `ContextraError::Internal`×2, `Edge`, `MemoryLink` … (+2) | `Entity::new`×10, `TlHfdParams::default`×4, `PprAlgorithm::Auto`×4, `TlHfdParams` |
| `contextra-adapt` | `ContextraError::InvalidInput`×9, `RetrievalStrategy`×9, `ContextraError`×4, `TenantId`×3, `ContextraError::Internal`×2, `RetrievalStrategy::Vector`×2, `RetrievalStrategy::Text`, `RetrievalStrategy::Graph`, `RetrievalStrategy::Hybrid` | – |
| `contextra-privacy` | `TenantScoped`×13, `TenantId`×12, `TenantScoped::new`×5, `TenantScopeViolation` | – |
| `contextra-crypto` | `TenantId`×8, `Result`×6, `ContextraError::Crypto`×3, `TenantScoped::new`×2, `TenantScoped`×2, `TxId`, `ContextraError`, `ModelFingerprint`, `ContextraError::ParseError` | – |
| `contextra-agent` | `ContextraError::InvalidInput`×10, `Result`×3, `WorkflowState`×3, `TxId`×2, `TokenBudget::new`×2, `ContextraError::Internal`, `DocId::from_key`, `types::budget`, `error` | `TokenBudget`×13, `TxId::new`×5, `ContextraError::Conflict`, `ContextraError::Serialization` … (+1) |
| `contextra-cognition` | `ContextraError::InvalidInput`×3, `ContextraError::Internal`×2, `ContextChunk`×2, `ContextraError::PolicyViolation`×2, `Result`×2, `TenantId`×2, `TxId`, `DocId` | `MemoryType::Episodic`×6, `DocId::new`×5 |
| `contextra-core` | `error`×2, `schema`×2, `tombstone`×2, `error_dto`, `model_fingerprint`, `types`, `error_dto::ContextraErrorDto`, `model_fingerprint::ModelFingerprint` | – |
| `contextra-rank` | `ConfigFingerprint`×3, `ContextraError`, `FusionWeights`, `FusionStrategy`, `SignalFusionStrategies`, `MAX_SEARCH_K`, `OnSignalFailure`, `DocId` | – |
| `contextra-mcp` | `ContextraError`×4, `ContextraError::InvalidInput`, `ContextraError::NotFound`, `TenantId::try_new`, `TenantId::SYSTEM`, `TenantId` | `DocId`, `AutoExtractionMode`, `EntityId::from_key`, `TokenBudget::new` |
| `contextra-ports` | `ContextraError`×3, `types::domain`, `model_fingerprint::ModelFingerprint`, `types::domain::TenantId`, `DocId`, `*`, `ContextraError::Plugin` | – |
| `contextra-infer-ollama` | `Result`×4, `ContextraError`×3, `ConfigFingerprint` | `ImportanceScore`×2 |
| `contextra-kvcache` | `TenantId`×3, `DocId`×2, `error::Result`, `error` | `model_fingerprint::ModelFingerprint`×6, `model_fingerprint::ModelFingerprint::new`, `ContextraError` |
| `contextra-infer-candle` | `ContextraError`×4, `DocId::from`, `ModelFingerprint` | `TenantId`×7, `DocId`×3, `Result`×3, `ContextChunk`×2 |
| `contextra-infer-onnx` | `Result`, `ContextraError::CapabilityUnsupported`, `RerankResult`, `ContextraError` | – |
| `contextra-audit-export` | `TenantId`×2 | – |
| `contextra-db` | `DistanceMetric`, `MAX_SEARCH_K` | `Result`×7, `TxId`×6, `DocId`×4, `FusionWeights`×3 … (+18) |
| `contextra` | `ScoredEntry` | – |
| `contextra-avv-generator` | `TenantId` | – |
| `contextra-license` | `TenantId` | – |
| `contextra-mvcc` | `*` | `TxId`×2, `DocId` |
| `contextra-py` | `TenantId::SYSTEM` | – |
| `contextra-router` | `RetrievalStrategy` | `ConfigFingerprint`×7, `TokenBudget`×4, `DocId::new`×3, `Result` |
| `contextra-vector` | `SaturatingU16::from` | – |
| `xtask` | `ContextraError` | – |
| `contextra-checkpoint` | – | `TxId`×5, `Result`×3, `ContextraError`×2, `ContextraError::Internal`×2 |
| `contextra-store` | – | `TxId`, `TenantId` |
| `contextra-text` | – | `Result`×67, `TOMBSTONE_BIT`×52, `TxId`×7, `TxId::new`×2 … (+3) |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

### contextra-vector

- Pfad: `crates/contextra-vector` · Zeilen: 21641 · Rust-Dateien: 109 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-core` (normal), `contextra-crypto` (normal), `contextra-ports` (normal), `contextra-simd` (normal), `contextra-sys` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-core` | 60 | 65 | 13 | `ContextraError::invalid_input`×19, `MAX_SEARCH_K`×15, `Result`×8, `ContextraError::HnswConnectivityDegraded`×4, `ContextraError::Index`×4, `DocId`×3, `DistanceMetric`×2, `SequenceLog::new`×2, `SeqLogChange::Insert`, `SeqLogChange::Delete` … (+1) | `types`×9, `traits::VectorIndex`×8, `ContextraError::InvalidInput`×6, `ContextraError::EmbeddingDimensionMismatch`×4, `DistanceMetric::Euclidean`×3 … (+5) |
| `contextra-ports` | 13 | 10 | 1 | `MetricsSink`×6, `Rng`×3, `NoopMetricsSink`×2, `SeededRng::new`×2 | `SeededRng`×5 |
| `contextra-crypto` | 6 | 2 | 2 | `wal_crypto::WalHmac::new`×6 | – |
| `contextra-simd` | 3 | 7 | 3 | `validate_vector`×2, `kernels::scalar` | `kernels::scalar::dot_product_scalar`, `kernels::scalar::euclidean_distance_scalar`, `kernels::scalar::cosine_distance_scalar`, `kernels::scalar::dot_product_u8_scalar`, `kernels::scalar::euclidean_distance_sq_u8_scalar` … (+1) |
| `contextra-sys` | 2 | 0 | 2 | `mmap_readonly`×2 | – |
| `contextra-types` | 1 | 0 | 1 | `SaturatingU16::from` | – |

**Pro Komponente**

- `acorn` (183 Zeilen)
  - → `contextra-core`: prod: `DocId`
- `diskann` (3590 Zeilen)
  - → `contextra-core`: prod: `MAX_SEARCH_K`×9
  - → `contextra-crypto`: prod: `wal_crypto::WalHmac::new`×6 · dev: `wal_crypto::WalHmac::new`×2
  - → `contextra-simd`: prod: `validate_vector`
  - → `contextra-sys`: prod: `mmap_readonly`
- `distance` (208 Zeilen)
  - → `contextra-core`: dev: `ContextraError::EmbeddingDimensionMismatch`×4, `DistanceMetric`
  - → `contextra-simd`: prod: `kernels::scalar` · dev: `kernels::scalar::dot_product_scalar`, `kernels::scalar::euclidean_distance_scalar`, `kernels::scalar::cosine_distance_scalar`, `kernels::scalar::dot_product_u8_scalar` … (+2)
- `hnsw` (4932 Zeilen)
  - → `contextra-core`: prod: `MAX_SEARCH_K`×6, `ContextraError::HnswConnectivityDegraded`×4, `ContextraError::Index`×4, `DocId`×2, `Result`×2, `SequenceLog::new`×2, `SeqLogChange::Insert`, `SeqLogChange::Delete`, `DistanceMetric`, `SequenceLog` · dev: `DistanceMetric`
  - → `contextra-ports`: prod: `MetricsSink`×6, `Rng`×3, `NoopMetricsSink`×2, `SeededRng::new`×2 · dev: `SeededRng::new`×3, `Rng`, `MetricsSink`
  - → `contextra-simd`: prod: `validate_vector`
  - → `contextra-types`: prod: `SaturatingU16::from`
- `persistence` (1095 Zeilen)
  - → `contextra-core`: dev: `Result`×2
  - → `contextra-sys`: prod: `mmap_readonly`
- `quantize` (972 Zeilen)
  - → `contextra-core`: prod: `ContextraError::invalid_input`×19, `Result`×6, `DistanceMetric` · dev: `ContextraError::InvalidInput`×6, `Result`×2
- `benches/acorn_vs_post_filter.rs` (53 Zeilen)
  - → `contextra-core`: dev: `DocId`
- `benches/audit_benchmarks.rs` (430 Zeilen)
  - → `contextra-core`: dev: `traits::VectorIndex`, `types`
- `benches/distance_bench.rs` (34 Zeilen)
  - → `contextra-simd`: dev: `kernels::scalar`
- `benches/distance_nan_impact_bench.rs` (24 Zeilen)
  - → `contextra-core`: dev: `DistanceMetric`
- `benches/flush_threshold_amplification.rs` (177 Zeilen)
  - → `contextra-core`: dev: `traits::VectorIndex`, `types`
- `benches/hnsw_bench.rs` (90 Zeilen)
  - → `contextra-core`: dev: `traits::VectorIndex`, `types`
- `benches/hnsw_delete_search_bench.rs` (86 Zeilen)
  - → `contextra-core`: dev: `traits::VectorIndex`, `types`
- `benches/overfetch_selectivity.rs` (78 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`
- `benches/sq8_bench.rs` (40 Zeilen)
  - → `contextra-core`: dev: `DistanceMetric`
- `tests/acorn_naive_recall.rs` (70 Zeilen)
  - → `contextra-core`: dev: `DocId`
- `tests/campaign_j23_vector_index.rs` (477 Zeilen)
  - → `contextra-core`: dev: `ContextraError::Index`×6
- `tests/campaign_j24_test.rs` (609 Zeilen)
  - → `contextra-core`: dev: `VectorIndex`, `types`
- `tests/diskann_corruption_fallback_test.rs` (102 Zeilen)
  - → `contextra-core`: dev: `TxId::new`
- `tests/diskann_docid128_tombstones.rs` (44 Zeilen)
  - → `contextra-core`: dev: `ContextraError::Io`, `TxId`
- `tests/diskann_recall_test.rs` (218 Zeilen)
  - → `contextra-core`: dev: `TxId`
- `tests/diskann_search_filtered.rs` (170 Zeilen)
  - → `contextra-core`: dev: `MAX_SEARCH_K`
- `tests/distance_determinism.rs` (64 Zeilen)
  - → `contextra-core`: dev: `DistanceMetric`
- `tests/docid_128_hnsw.rs` (69 Zeilen)
  - → `contextra-core`: dev: `Result`×2
- `tests/docid_high_bits_no_collision.rs` (197 Zeilen)
  - → `contextra-core`: dev: `Result`×2, `ContextraError::Io`
- `tests/filtered_selectivity_recall.rs` (154 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`
- `tests/hnsw_rebuild_search_consistency.rs` (133 Zeilen)
  - → `contextra-core`: dev: `DistanceMetric::Euclidean`
- `tests/hnsw_seed_determinism.rs` (294 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`
- `tests/j32closure-contextra-vector_acorn_test.rs` (12 Zeilen)
  - → `contextra-core`: dev: `DocId`
- `tests/k02_snapshot_rebuild_determinism.rs` (152 Zeilen)
  - → `contextra-core`: dev: `DistanceMetric::Euclidean`
- `tests/loom_quantizer_race_test.rs` (114 Zeilen)
  - → `contextra-core`: dev: `ContextraError`
- `tests/partial_rebuild_veto_f02_guard.rs` (97 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`
- `tests/poisoning.rs` (109 Zeilen)
  - → `contextra-core`: dev: `traits::VectorIndex`, `types`
- `tests/proptest_distance_quantize.rs` (89 Zeilen)
  - → `contextra-core`: dev: `DistanceMetric`
- `tests/quantize_persistence_audit.rs` (302 Zeilen)
  - → `contextra-core`: dev: `traits::VectorIndex`, `ContextraError`, `types`
- `tests/rabitq_recall_calibration.rs` (117 Zeilen)
  - → `contextra-core`: dev: `ContextraError`
- `tests/recalibration.rs` (55 Zeilen)
  - → `contextra-core`: dev: `DistanceMetric::Euclidean`
- `tests/recall_audit.rs` (405 Zeilen)
  - → `contextra-core`: dev: `traits::VectorIndex`, `types`
- `tests/rollback.rs` (55 Zeilen)
  - → `contextra-core`: dev: `traits::VectorIndex`, `types`
- `tests/simd_numerical_audit.rs` (171 Zeilen)
  - → `contextra-core`: dev: `DistanceMetric`
- `tests/sq8_bias_calibration.rs` (20 Zeilen)
  - → `contextra-core`: dev: `DistanceMetric`
- `tests/sq8_rerank_recall.rs` (163 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`

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
    - `struct VectorCandidateStream` · 8 Methoden
- `compute_pool`
  - 2 struct
    - `struct TaskHandle` · 1 Methoden
    - `struct ComputePool` · 5 Methoden
- `diskann`
  - 1 struct
    - `struct DiskAnnIndex` *(re-exportiert)* · 2 Methoden
- `hnsw`
  - 2 const, 7 struct, 1 trait, 1 type
    - `struct AdaptiveEfPolicy` · 2 Methoden
    - `struct AdaptiveEfStats`
    - `struct AdaptiveEfStateMachine` · 6 Methoden
    - `const ARENA_ALIGNMENT_BYTES`
    - `const ARENA_ALIGNMENT_U32`
    - `type BacklinkKey`
    - `struct BacklinkTable` · 4 Methoden
    - `struct HnswArena` · 8 Methoden
    - `struct DeletionStats`
    - `trait GhostFreeVectorIndex` · Methoden: remove_with_graph_repair
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
| `contextra-engine` | `hnsw::HnswIndex`×2, `HnswIndex`×2, `acorn::FilteredIndex` | `HnswConfig`×58, `hnsw::HnswConfig::default`, `hnsw::HnswIndex::try_new`, `acorn` … (+1) |
| `contextra-cognition` | `HnswIndex`×3 | `HnswConfig`×8 |
| `contextra-db` | – | `HnswConfig`×3, `HnswIndex`×2, `HnswIndex::try_new` |
| `contextra-router` | – | `HnswConfig::default`, `HnswIndex::try_new` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 28 von 29)**: `compute_gamma_edge_budget`, `AcornError`, `NaiveReferenceIndex`, `DEFAULT_VECTOR_STREAM_BATCH_SIZE`, `VectorCandidateStream`, `TaskHandle`, `ComputePool`, `DiskAnnIndex`, `AdaptiveEfPolicy`, `AdaptiveEfStats`, `AdaptiveEfStateMachine`, `ARENA_ALIGNMENT_BYTES`, `ARENA_ALIGNMENT_U32`, `BacklinkKey`, `BacklinkTable`, `HnswArena`, `DeletionStats`, `GhostFreeVectorIndex`, `Sq8Bias`, `PartialRebuildConfig`, `TraversalTracker`, `should_trigger_partial_rebuild`, `MmapIndex`, `NodeRecord`, `DEFAULT_P_LOW`, `DEFAULT_P_HIGH`, `ScalarQuantizer`, `RaBitQQuantizer`

### contextra-wire

- Pfad: `crates/contextra-wire` · Zeilen: 1892 · Rust-Dateien: 8 · Ring: 0
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `adapter`
  - 1 struct
    - `struct WireBuffer` · 3 Methoden
- `contextra_generated`
  - 8 enum, 8 fn, 24 struct
    - `enum EmbeddingOffset`
    - `struct Embedding` · 5 Methoden
    - `struct EmbeddingArgs`
    - `struct EmbeddingBuilder` · 5 Methoden
    - `enum ScoredDocumentOffset`
    - `struct ScoredDocument` · 6 Methoden
    - `struct ScoredDocumentArgs`
    - `struct ScoredDocumentBuilder` · 6 Methoden
    - `enum SearchResponseOffset`
    - `struct SearchResponse` · 5 Methoden
    - `struct SearchResponseArgs`
    - `struct SearchResponseBuilder` · 5 Methoden
    - `enum VectorIndexUpdateOffset`
    - `struct VectorIndexUpdate` · 5 Methoden
    - `struct VectorIndexUpdateArgs`
    - `struct VectorIndexUpdateBuilder` · 5 Methoden
    - `enum RoleIdOffset`
    - `struct RoleId` · 3 Methoden
    - `struct RoleIdArgs`
    - `struct RoleIdBuilder` · 3 Methoden
    - `enum HyperEdgeIdOffset`
    - `struct HyperEdgeId` · 3 Methoden
    - `struct HyperEdgeIdArgs`
    - `struct HyperEdgeIdBuilder` · 3 Methoden
    - `enum RoleBindingOffset`
    - `struct RoleBinding` · 4 Methoden
    - `struct RoleBindingArgs`
    - `struct RoleBindingBuilder` · 4 Methoden
    - `enum HyperEdgeOffset`
    - `struct HyperEdge` · 12 Methoden
    - `struct HyperEdgeArgs`
    - `struct HyperEdgeBuilder` · 12 Methoden
    - `fn root_as_search_response`
    - `fn size_prefixed_root_as_search_response`
    - `fn root_as_search_response_with_opts`
    - `fn size_prefixed_root_as_search_response_with_opts`
    - `fn root_as_search_response_unchecked`
    - `fn size_prefixed_root_as_search_response_unchecked`
    - `fn finish_search_response_buffer`
    - `fn finish_size_prefixed_search_response_buffer`
- `jsonrpc`
  - 3 struct
    - `struct JsonRpcRequest`
    - `struct JsonRpcResponse` · 2 Methoden
    - `struct JsonRpcError`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-core` | `*` | – |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

### xtask

- Pfad: `xtask` · Zeilen: 51482 · Rust-Dateien: 236 · Ring: tooling
- Interne Cargo-Abhängigkeiten: `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 1 | 0 | 1 | `ContextraError` | – |

**Pro Komponente**

- `session_history` (224 Zeilen)
  - → `contextra-types`: prod: `ContextraError`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 enum, 48 fn, 8 struct
    - `fn find_root_dir`
    - `fn chrono_or_today_with_tags`
    - `fn chrono_or_today`
    - `struct TagItem`
    - `struct CapabilitiesManifest`
    - `struct CapabilityCrateEntry`
    - `fn load_capabilities_manifest`
    - `struct CrateInfo`
    - `fn scan_tags`
    - `fn calculate_crate_loc`
    - `fn compute_crate_layers`
    - `fn get_workspace_crates`
    - `fn get_workspace_crates_from_root`
    - `fn render_updated_markdown`
    - `fn update_markdown_section`
    - `fn generate_session_continuity_section`
    - `fn generate_session_continuity_section_from_root`
    - `fn tokenize_title`
    - `fn jaccard_similarity`
    - `fn extract_scope`
    - `fn generate_full_working_state`
    - `fn generate_full_working_state_from_root`
    - `fn generate_invariants_table_section`
    - `fn generate_changelog`
    - `fn generate_ai_tags_section`
    - `fn generate_dag_topology_section`
    - `fn format_loc`
    - `fn generate_crate_inventory_section`
    - `fn severity_weight`
    - `fn run_sync_docs`
    - `fn check_readme_crate_count`
    - `fn check_crate_agents`
    - `fn check_no_orphan_adr_files`
    - `fn check_adr_consistency_dir`
    - `fn check_adr_consistency`
    - `fn get_changed_rs_files_from_git_diff`
    - `fn get_git_file_last_modified`
    - `fn levenshtein_distance`
    - `struct SentenceItem`
    - `fn split_into_sentences`
    - `fn sentences_are_similar`
    - `fn check_sentence_overlap`
    - `fn run_check_consistency`
    - `enum TagDateStatus`
    - `fn validate_working_state_tag_date`
    - `fn validate_tags_items`
    - `fn run_validate_tags`
    - `struct DagViolation`
    - `fn check_dag_layer_violations`
    - `fn run_check_dag`
    - `fn run_check_review_coverage`
    - `struct ContextTagNdjson`
    - `struct ContextTagFilter`
    - `fn extract_crate_name`
    - `fn parse_context_tag_args`
    - `fn filter_tags`
    - `fn run_context_tags`
- `agent_lifecycle`
  - 3 fn
    - `fn run_check_branch_overlap`
    - `fn run_check_branch_overlap_with_args`
    - `fn run_prune_branches`
- `artifact_header`
  - 1 enum, 1 struct
    - `enum ArtifactHeaderError`
    - `struct ArtifactHeader` · 3 Methoden
- `audit_integrity_check`
  - 1 enum, 2 fn, 1 struct
    - `enum GapSeverity`
    - `struct AuditFixGap`
    - `fn run_audit_integrity_check_impl`
    - `fn run_audit_integrity_check`
- `bench_compile`
  - 3 fn, 1 struct
    - `struct BenchCompileResult`
    - `fn discover_benches`
    - `fn build_bench_command_args`
    - `fn run_bench_compile`
- `bench_download`
  - 4 const, 1 enum, 3 fn, 1 struct
    - `enum Dataset`
    - `const SIFT1M_URL`
    - `const BEIR_NFCORPUS_URL`
    - `const ONNX_MODEL_URL`
    - `const ONNX_TOKENIZER_URL`
    - `struct DownloadTarget`
    - `fn get_dataset_targets`
    - `fn build_curl_args`
    - `fn run_bench_download`
- `check_action_pinning`
  - 1 fn
    - `fn run_check_action_pinning`
- `check_adr_deadlines`
  - 1 const, 5 fn, 2 struct
- `check_agents_freshness`
  - 3 fn, 1 struct
- `check_agents_integrity`
  - 1 fn
- `check_audit_duplication`
  - 7 fn, 2 struct
- `check_audit_tool_evidence`
  - 1 const, 3 fn, 2 struct
- `check_audit_verdict_independence`
  - 1 const, 6 fn, 2 struct
- `check_commit_diff_integrity`
  - 7 fn, 1 struct
- `check_commit_messages`
  - 2 fn
- `check_compile`
  - 1 fn
- `check_coverage_gate`
  - 3 fn, 1 struct
- `check_doc_references`
  - 1 const, 5 fn, 1 struct
- `check_duplicate_core_primitives`
  - 2 fn, 2 struct
- `check_duplicate_intent`
  - 18 fn, 3 struct
- `check_duplicate_symbols`
  - 5 fn, 1 struct
- `check_duplicate_symbols_cross_file`
  - 3 fn, 1 struct
- `check_ffi_panic_boundary`
  - 3 fn
- `check_flatbuffers_drift`
  - 2 fn
- `check_jules_context_freshness`
  - 1 enum, 3 fn
- `check_manifest_completeness`
  - 1 const, 4 fn
- `check_max_results_unbound`
  - 2 fn, 1 struct
- `check_module_reachability`
  - 1 const, 2 fn, 1 struct
- `check_mutation_score_gate`
  - 1 enum, 3 fn, 2 struct
- `check_nan_validation_in_hot_loop`
  - 2 fn, 1 struct
- `check_orphan_modules`
  - 1 fn
- `check_phantom_files`
  - 1 fn
- `check_placeholder_refs`
  - 1 enum, 4 fn, 1 struct
- `check_recall_stability`
  - 1 const, 4 fn, 1 struct
- `check_result_dropped_on_io`
  - 2 fn, 1 struct
- `check_ring0_async_purity`
  - 1 const, 3 fn, 2 struct
- `check_ring_capabilities_consistency`
  - 2 fn, 1 struct
- `check_ring_layering`
  - 1 const, 1 enum, 10 fn, 2 struct
- `check_stale_tags`
  - 1 const, 1 enum, 6 fn, 2 struct
- `check_toc_integrity`
  - 1 const, 3 fn, 1 struct
- `check_toctou_trait_defaults`
  - 2 fn, 1 struct
- `check_type_registry`
  - 2 fn, 2 struct
- `check_unsafe_islands`
  - 2 const, 1 enum, 4 fn, 2 struct
- `check_vetoes`
  - 1 const, 6 fn, 2 struct
- `check_workflow_commands`
  - 4 fn, 1 struct
- `claim`
  - 1 const, 6 fn, 2 struct
- `cli`
  - 94 fn, 1 static, 1 type
- `commit_health`
  - 2 fn
- `context_pack`
  - 2 fn, 2 struct
- `crate_context`
  - 1 enum, 1 fn, 1 struct
- `env_validate`
  - 1 enum, 1 fn, 1 struct
- `feature_matrix`
  - 1 enum, 3 fn, 2 struct
- `gate_check`
  - 1 const, 2 enum, 1 fn, 2 struct
- `gates`
  - 16 fn, 2 struct
- `gen_arch_docs`
  - 2 fn
- `gen_feature_catalog`
  - 1 fn
- `gen_prompter_data`
  - 1 fn
- `generate_adr`
  - 2 fn, 1 struct
- `generate_diagnostics`
  - 1 fn, 1 struct
- `generate_markers`
  - 5 fn, 3 struct
- `generators`
  - 1 fn
- `harness`
  - 1 fn
- `hotspot_report`
  - 1 enum, 2 fn, 1 struct
- `init_audit_fix`
  - 1 fn
- `jules_preflight`
  - 1 enum, 2 fn
- `jules_submit_gate`
  - 10 fn, 2 struct
- `lint_unsafe_slice_bounds`
  - 2 fn, 1 struct
- `loom_run`
  - 2 fn, 2 struct
- `migrate_docid_128`
  - 3 fn, 3 struct
- `panic_inventory`
  - 1 enum, 2 fn, 1 struct
- `post_merge_report`
  - 1 fn
- `proof`
  - 1 fn
- `record_mutation_score`
  - 1 fn, 1 struct
- `reproducible_build`
  - 5 fn
- `security_scan`
  - 1 enum, 2 fn, 1 struct
- `session_history`
  - 1 const, 4 fn, 2 struct
- `session_init`
  - 2 fn, 1 struct
- `shell_commit_audit`
  - 2 fn, 2 struct
- `tag_health`
  - 1 enum, 2 fn, 1 struct
- `validate_pr_checklist`
  - 1 fn
- `veto_deadline_gate`
  - 1 const, 4 fn, 1 struct
- `workspace_index`
  - 6 struct
- `workspace_verify`
  - 3 enum, 4 fn, 3 struct
- `ws_cache`
  - 3 fn
  - … 469 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 265 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 549 von 549)**: `run_check_branch_overlap`, `run_check_branch_overlap_with_args`, `run_prune_branches`, `ArtifactHeaderError`, `ArtifactHeader`, `GapSeverity`, `AuditFixGap`, `run_audit_integrity_check_impl`, `run_audit_integrity_check`, `BenchCompileResult`, `discover_benches`, `build_bench_command_args`, `run_bench_compile`, `Dataset`, `SIFT1M_URL`, `BEIR_NFCORPUS_URL`, `ONNX_MODEL_URL`, `ONNX_TOKENIZER_URL`, `DownloadTarget`, `get_dataset_targets`, `build_curl_args`, `run_bench_download`, `run_check_action_pinning`, `ADR_DEADLINE_WARNING_THRESHOLD_DAYS`, `AdrDeadlineEntry`, `parse_adr_deadlines`, `AdrDeadlineCheckResult`, `check_adr_deadlines_at`, `check_adr_deadlines_with_root_and_date`, `check_adr_deadlines_with_root` …

### xtask-heavy

- Pfad: `xtask-heavy` · Zeilen: 244 · Rust-Dateien: 3 · Ring: tooling
- Interne Cargo-Abhängigkeiten: `contextra-bench` (normal), `contextra-router` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-router` | 2 | 0 | 1 | `BanditProfileState`, `BanditImplementation::DiagonalApproximation` | – |
| `contextra-bench` | 1 | 0 | 1 | `regression_gate` | – |

**Pro Komponente**

- `bench_gate` (54 Zeilen)
  - → `contextra-bench`: prod: `regression_gate`
- `check_bandit_latency_budget` (157 Zeilen)
  - → `contextra-router`: prod: `BanditProfileState`, `BanditImplementation::DiagonalApproximation`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

Keine pub-Items erkannt (Bibliothek ohne lib.rs, nur Binary, oder alles intern).
- Zusätzlich 6 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

