# MemFuse — Feature Catalog

> **Hinweis**: Diese Datei ist autogeneriert durch `cargo xtask gen-feature-catalog`.
> Sie listet alle verfuegbaren Cargo Feature Flags aller Workspace-Crates auf.

## Crate `memfuse`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `candle` | `memfuse-infer-candle` |
| `default` | `default` / keine weiteren Flags |
| `ollama` | `memfuse-infer-ollama` |
| `onnx` | `memfuse-infer-onnx/onnx`, `memfuse-db/onnx` |
| `router` | `memfuse-router`, `memfuse-calibration` |

## Crate `memfuse-adapt`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `bandit-routing` | `default` / keine weiteren Flags |
| `default` | `bandit-routing` |
| `egress-sherman-morrison` | `bandit-routing` |

## Crate `memfuse-agent`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `test-utils` | `default` / keine weiteren Flags |

## Crate `memfuse-bench`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `external-benchmarks` | `default` / keine weiteren Flags |
| `onnx-bench` | `memfuse-infer-onnx/onnx`, `memfuse-db/reranking` |

## Crate `memfuse-calibration`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `replicator-dynamics-weights` | `default` / keine weiteren Flags |

## Crate `memfuse-checkpoint`

*Keine expliziten Feature Flags deklariert.*

## Crate `memfuse-cognition`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `memfuse-core/docid-128`, `memfuse-engine/docid-128` |
| `edge-reinforcement-learning` | `memfuse-graph/edge-reinforcement-learning`, `memfuse-engine/edge-reinforcement-learning` |
| `graph-connectivity-health` | `memfuse-graph/physio-percolation`, `memfuse-engine/graph-connectivity-health` |

## Crate `memfuse-core`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `docid-128` | `memfuse-types/docid-128` |
| `test-utils` | `default` / keine weiteren Flags |

## Crate `memfuse-crypto`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `cloud-egress-guard` | `default` / keine weiteren Flags |
| `kv-encryption` | `default` / keine weiteren Flags |
| `test-utils` | `default` / keine weiteren Flags |

## Crate `memfuse-db`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `adaptive-candidate-pool-sizing` | `default` / keine weiteren Flags |
| `background-maintenance` | `default` / keine weiteren Flags |
| `bench` | `default` / keine weiteren Flags |
| `coherence-bonus-fusion` | `default` / keine weiteren Flags |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `memfuse-core/docid-128` |
| `edge-reinforcement-learning` | `memfuse-graph/edge-reinforcement-learning` |
| `experimental-diskann` | `memfuse-index/experimental-diskann` |
| `graph-connectivity-health` | `memfuse-graph/physio-percolation` |
| `onnx` | `default` / keine weiteren Flags |
| `reranking` | `default` / keine weiteren Flags |
| `sandbox` | `default` / keine weiteren Flags |
| `volatile-vault` | `default` / keine weiteren Flags |

## Crate `memfuse-engine`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `adaptive-candidate-pool-sizing` | `default` / keine weiteren Flags |
| `background-maintenance` | `default` / keine weiteren Flags |
| `bench` | `default` / keine weiteren Flags |
| `coherence-bonus-fusion` | `default` / keine weiteren Flags |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `memfuse-core/docid-128` |
| `edge-reinforcement-learning` | `memfuse-graph/edge-reinforcement-learning` |
| `experimental-diskann` | `memfuse-vector/experimental-diskann` |
| `graph-connectivity-health` | `memfuse-graph/physio-percolation` |
| `onnx` | `default` / keine weiteren Flags |
| `reranking` | `default` / keine weiteren Flags |
| `sandbox` | `default` / keine weiteren Flags |

## Crate `memfuse-graph`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `memfuse-core/docid-128` |
| `edge-reinforcement-learning` | `default` / keine weiteren Flags |
| `graph-connectivity-health` | `default` / keine weiteren Flags |
| `physio-percolation` | `graph-connectivity-health` |
| `physio-synaptic-edges` | `edge-reinforcement-learning` |
| `ppr-forward-push` | `default` / keine weiteren Flags |

## Crate `memfuse-infer-candle`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `candle` | `default` / keine weiteren Flags |
| `cuda` | `candle-core/cuda`, `candle-nn/cuda`, `candle-transformers/cuda` |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `memfuse-core/docid-128` |
| `kv-bridge` | `dep:memfuse-crypto`, `memfuse-crypto/kv-encryption`, `dep:bincode`, `dep:memfuse-store`, `memfuse-store` |
| `memfuse-store` | `dep:memfuse-store` |

## Crate `memfuse-infer-ollama`

*Keine expliziten Feature Flags deklariert.*

## Crate `memfuse-infer-onnx`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `candle-backend` | `dep:memfuse-candle` |
| `default` | `default` / keine weiteren Flags |
| `onnx` | `ort`, `tokenizers`, `ndarray`, `dep:reqwest` |

## Crate `memfuse-kvcache`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `kv-encryption` | `memfuse-crypto/kv-encryption` |

## Crate `memfuse-mcp`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `agent-workflows` | `memfuse-agent` |
| `candle` | `dep:memfuse-infer-candle` |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `memfuse-core/docid-128` |
| `kv-bridge` | `memfuse-infer-candle/kv-bridge`, `dep:memfuse-infer-candle` |
| `onnx` | `memfuse-infer-onnx`, `memfuse-infer-onnx/onnx` |
| `test-utils` | `memfuse-core/test-utils` |

## Crate `memfuse-mvcc`

*Keine expliziten Feature Flags deklariert.*

## Crate `memfuse-ports`

*Keine expliziten Feature Flags deklariert.*

## Crate `memfuse-privacy`

*Keine expliziten Feature Flags deklariert.*

## Crate `memfuse-py`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `memfuse-core/docid-128` |

## Crate `memfuse-rank`

*Keine expliziten Feature Flags deklariert.*

## Crate `memfuse-router`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `bandit-routing` | `egress-sherman-morrison` |
| `cloud-egress-guard` | `default` / keine weiteren Flags |
| `default` | `default` / keine weiteren Flags |
| `egress-sherman-morrison` | `default` / keine weiteren Flags |

## Crate `memfuse-sandbox`

*Keine expliziten Feature Flags deklariert.*

## Crate `memfuse-simd`

*Keine expliziten Feature Flags deklariert.*

## Crate `memfuse-store`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `block-cache-v2` | `dep:quick_cache` |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `memfuse-core/docid-128` |
| `fault-injection` | `default` / keine weiteren Flags |

## Crate `memfuse-sys`

*Keine expliziten Feature Flags deklariert.*

## Crate `memfuse-testkit`

*Keine expliziten Feature Flags deklariert.*

## Crate `memfuse-text`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `bm25f` | `default` / keine weiteren Flags |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `memfuse-core/docid-128` |

## Crate `memfuse-types`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `default` / keine weiteren Flags |

## Crate `memfuse-vector`

| Feature Flag | Aktivierte Abhaengigkeiten / Flags |
| :--- | :--- |
| `default` | `default` / keine weiteren Flags |
| `docid-128` | `memfuse-core/docid-128` |
| `experimental-diskann` | `default` / keine weiteren Flags |
| `experimental-rabitq` | `default` / keine weiteren Flags |
| `graph` | `default` / keine weiteren Flags |
| `partial-index-rebuild` | `default` / keine weiteren Flags |

## Crate `memfuse-wire`

*Keine expliziten Feature Flags deklariert.*
