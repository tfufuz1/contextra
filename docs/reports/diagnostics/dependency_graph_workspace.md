# Ring Boundary & Dependency Graph Architecture Audit

> **Werkzeug:** `cargo_dependency_graph.py`  
> **Erstellungszeit:** `2026-11-07T23:03:20.449378+00:00`  
> **Status/Befunde:** `0 Befunde erfasst`  

## Beschreibung
Verifies the Ring model (Ring 4 -> 0), validates capabilities.toml may_depend_on compliance (M1), and prevents dev-dependency leaks (M6).

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `total_crates` | `37` |
| `m1_ring_violations` | `0` |
| `m6_dev_dep_violations` | `0` |
| `circular_dependency_cycles` | `0` |

## Crate-Ring-Matrix
| Crate | Ring | Interne Abhängigkeiten | Dev-Abhängigkeiten |
|---|---|---|---|
| `contextra` | ? | 6 | 2 |
| `contextra-adapt` | ? | 2 | 0 |
| `contextra-agent` | ? | 6 | 4 |
| `contextra-audit-export` | ? | 1 | 0 |
| `contextra-avv-generator` | ? | 1 | 0 |
| `contextra-bench` | ? | 4 | 1 |
| `contextra-checkpoint` | ? | 2 | 1 |
| `contextra-cognition` | ? | 4 | 1 |
| `contextra-core` | ? | 4 | 0 |
| `contextra-crypto` | ? | 2 | 2 |
| `contextra-db` | ? | 3 | 7 |
| `contextra-durable-fs` | ? | 3 | 1 |
| `contextra-engine` | ? | 13 | 2 |
| `contextra-erasure` | ? | 3 | 0 |
| `contextra-graph` | ? | 2 | 1 |
| `contextra-infer-candle` | ? | 4 | 1 |
| `contextra-infer-ollama` | ? | 2 | 0 |
| `contextra-infer-onnx` | ? | 3 | 0 |
| `contextra-ingest` | ? | 1 | 0 |
| `contextra-kvcache` | ? | 3 | 0 |
| `contextra-license` | ? | 2 | 0 |
| `contextra-mcp` | ? | 11 | 0 |
| `contextra-mvcc` | ? | 2 | 1 |
| `contextra-ports` | ? | 1 | 0 |
| `contextra-privacy` | ? | 2 | 2 |
| `contextra-py` | ? | 3 | 0 |
| `contextra-rank` | ? | 0 | 0 |
| `contextra-router` | ? | 4 | 0 |
| `contextra-sandbox` | ? | 0 | 0 |
| `contextra-simd` | ? | 1 | 0 |
| `contextra-store` | ? | 5 | 2 |
| `contextra-sys` | ? | 0 | 0 |
| `contextra-testkit` | ? | 2 | 0 |
| `contextra-text` | ? | 2 | 0 |
| `contextra-types` | ? | 0 | 0 |
| `contextra-vector` | ? | 5 | 0 |
| `contextra-wire` | ? | 0 | 0 |

---
*Automatisch generiert durch `cargo_dependency_graph.py` für Agentic Engineering & Context Auditing.*