# Contextra — Feature Catalog

> **Hinweis**: Diese Datei ist autogeneriert durch `cargo xtask gen-feature-catalog`.
> Sie listet alle verfuegbaren Cargo Feature Flags aller Workspace-Crates auf.

## Crate `contextra`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `candle` | `contextra-infer-candle` |
| `default` | `default` / keine weiteren Flags |
| `ollama` | `contextra-infer-ollama` |
| `onnx` | `contextra-infer-onnx/onnx`, `contextra-db/onnx` |
| `router` | `contextra-router`, `contextra-calibration` |

## Crate `contextra-adapt`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `bandit-routing` | `default` / keine weiteren Flags |
| `default` | `bandit-routing` |
| `egress-sherman-morrison` | `bandit-routing` |

## Crate `contextra-agent`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `test-utils` | `default` / keine weiteren Flags |

## Crate `contextra-bench`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `external-benchmarks` | `default` / keine weiteren Flags |
| `onnx-bench` | `contextra-infer-onnx/onnx`, `contextra-db/reranking` |

## Crate `contextra-calibration`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `replicator-dynamics-weights` | `default` / keine weiteren Flags |

## Crate `contextra-checkpoint`

*Keine expliziten Feature Flags deklariert.*

## Crate `contextra-cognition`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `contextra-core/docid-128`, `contextra-engine/docid-128` |
| `edge-reinforcement-learning` | `contextra-graph/edge-reinforcement-learning`, `contextra-engine/edge-reinforcement-learning` |
| `graph-connectivity-health` | `contextra-graph/physio-percolation`, `contextra-engine/graph-connectivity-health` |

## Crate `contextra-core`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `docid-128` | `contextra-types/docid-128` |
| `test-utils` | `default` / keine weiteren Flags |

## Crate `contextra-crypto`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `cloud-egress-guard` | `default` / keine weiteren Flags |
| `kv-encryption` | `default` / keine weiteren Flags |
| `test-utils` | `default` / keine weiteren Flags |

## Crate `contextra-db`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `adaptive-candidate-pool-sizing` | `default` / keine weiteren Flags |
| `background-maintenance` | `default` / keine weiteren Flags |
| `bench` | `default` / keine weiteren Flags |
| `coherence-bonus-fusion` | `default` / keine weiteren Flags |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `contextra-core/docid-128` |
| `edge-reinforcement-learning` | `contextra-graph/edge-reinforcement-learning` |
| `experimental-diskann` | `contextra-index/experimental-diskann` |
| `graph-connectivity-health` | `contextra-graph/physio-percolation` |
| `onnx` | `default` / keine weiteren Flags |
| `reranking` | `default` / keine weiteren Flags |
| `sandbox` | `default` / keine weiteren Flags |
| `volatile-vault` | `default` / keine weiteren Flags |

## Crate `contextra-engine`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `adaptive-candidate-pool-sizing` | `default` / keine weiteren Flags |
| `background-maintenance` | `default` / keine weiteren Flags |
| `bench` | `default` / keine weiteren Flags |
| `coherence-bonus-fusion` | `default` / keine weiteren Flags |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `contextra-core/docid-128` |
| `edge-reinforcement-learning` | `contextra-graph/edge-reinforcement-learning` |
| `experimental-diskann` | `contextra-vector/experimental-diskann` |
| `graph-connectivity-health` | `contextra-graph/physio-percolation` |
| `onnx` | `default` / keine weiteren Flags |
| `reranking` | `default` / keine weiteren Flags |
| `sandbox` | `default` / keine weiteren Flags |

## Crate `contextra-graph`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `contextra-core/docid-128` |
| `edge-reinforcement-learning` | `default` / keine weiteren Flags |
| `graph-connectivity-health` | `default` / keine weiteren Flags |
| `physio-percolation` | `graph-connectivity-health` |
| `physio-synaptic-edges` | `edge-reinforcement-learning` |
| `ppr-forward-push` | `default` / keine weiteren Flags |

## Crate `contextra-infer-candle`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `candle` | `default` / keine weiteren Flags |
| `cuda` | `candle-core/cuda`, `candle-nn/cuda`, `candle-transformers/cuda` |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `contextra-core/docid-128` |
| `kv-bridge` | `dep:contextra-crypto`, `contextra-crypto/kv-encryption`, `dep:bincode`, `dep:contextra-store`, `contextra-store` |
| `contextra-store` | `dep:contextra-store` |

## Crate `contextra-infer-ollama`

*Keine expliziten Feature Flags deklariert.*

## Crate `contextra-infer-onnx`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `candle-backend` | `dep:contextra-candle` |
| `default` | `default` / keine weiteren Flags |
| `onnx` | `ort`, `tokenizers`, `ndarray`, `dep:reqwest` |

## Crate `contextra-kvcache`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `kv-encryption` | `contextra-crypto/kv-encryption` |

## Crate `contextra-mcp`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `agent-workflows` | `contextra-agent` |
| `candle` | `dep:contextra-infer-candle` |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `contextra-core/docid-128` |
| `kv-bridge` | `contextra-infer-candle/kv-bridge`, `dep:contextra-infer-candle` |
| `onnx` | `contextra-infer-onnx`, `contextra-infer-onnx/onnx` |
| `test-utils` | `contextra-core/test-utils` |

## Crate `contextra-mvcc`

*Keine expliziten Feature Flags deklariert.*

## Crate `contextra-ports`

*Keine expliziten Feature Flags deklariert.*

## Crate `contextra-privacy`

*Keine expliziten Feature Flags deklariert.*

## Crate `contextra-py`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `contextra-core/docid-128` |

## Crate `contextra-rank`

*Keine expliziten Feature Flags deklariert.*

## Crate `contextra-router`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `bandit-routing` | `egress-sherman-morrison` |
| `cloud-egress-guard` | `default` / keine weiteren Flags |
| `default` | `default` / keine weiteren Flags |
| `egress-sherman-morrison` | `default` / keine weiteren Flags |

## Crate `contextra-sandbox`

*Keine expliziten Feature Flags deklariert.*

## Crate `contextra-simd`

*Keine expliziten Feature Flags deklariert.*

## Crate `contextra-store`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `block-cache-v2` | `dep:quick_cache` |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `contextra-core/docid-128` |
| `fault-injection` | `default` / keine weiteren Flags |

## Crate `contextra-sys`

*Keine expliziten Feature Flags deklariert.*

## Crate `contextra-testkit`

*Keine expliziten Feature Flags deklariert.*

## Crate `contextra-text`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `bm25f` | `default` / keine weiteren Flags |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `contextra-core/docid-128` |

## Crate `contextra-types`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `default` / keine weiteren Flags |

## Crate `contextra-vector`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `contextra-core/docid-128` |
| `experimental-diskann` | `default` / keine weiteren Flags |
| `experimental-rabitq` | `default` / keine weiteren Flags |
| `graph` | `default` / keine weiteren Flags |
| `partial-index-rebuild` | `default` / keine weiteren Flags |

## Crate `contextra-wire`

*Keine expliziten Feature Flags deklariert.*
