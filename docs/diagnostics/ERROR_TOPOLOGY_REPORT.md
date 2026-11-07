# Error Topology & Data Leak Audit (INV-LOG-1 / HR-4)

> Generiert durch `tools/error_topology_mapper.py`.
> Untersucht typisierte Fehlerstrukturen und verhindert Nutzdaten-/Geheimnis-Leaks in Fehlermeldungen.

## Crate Übersicht

| Crate | Definierte Fehler-Typen | Potenzielle Nutzdaten-Leaks |
|---|---|---|
| `contextra` | 3 | 0 |
| `contextra-adapt` | 4 | 0 |
| `contextra-agent` | 0 | 0 |
| `contextra-audit-export` | 1 | 0 |
| `contextra-avv-generator` | 1 | 0 |
| `contextra-checkpoint` | 0 | 0 |
| `contextra-cognition` | 0 | 0 |
| `contextra-core` | 0 | 0 |
| `contextra-crypto` | 3 | 0 |
| `contextra-db` | 1 | 0 |
| `contextra-durable-fs` | 0 | 0 |
| `contextra-engine` | 0 | 0 |
| `contextra-erasure` | 3 | 0 |
| `contextra-graph` | 3 | 0 |
| `contextra-infer-candle` | 0 | 0 |
| `contextra-infer-ollama` | 0 | 0 |
| `contextra-infer-onnx` | 0 | 0 |
| `contextra-ingest` | 1 | 0 |
| `contextra-kvcache` | 0 | 0 |
| `contextra-license` | 1 | 0 |
| `contextra-mcp` | 1 | 0 |
| `contextra-mvcc` | 0 | 0 |
| `contextra-ports` | 6 | 0 |
| `contextra-privacy` | 3 | **2** |
| `contextra-py` | 0 | 0 |
| `contextra-rank` | 1 | 0 |
| `contextra-router` | 2 | 0 |
| `contextra-sandbox` | 2 | 0 |
| `contextra-simd` | 0 | 0 |
| `contextra-store` | 2 | 0 |
| `contextra-sys` | 0 | 0 |
| `contextra-testkit` | 0 | 0 |
| `contextra-text` | 0 | 0 |
| `contextra-types` | 2 | 0 |
| `contextra-vector` | 1 | 0 |
| `contextra-wire` | 1 | 0 |

## Verdächtige Fehlermeldungs-Formate (Möglicher Leak von {key}, {secret}...)

### `contextra-privacy`

- `/app/crates/contextra-privacy/src/egress_vault.rs:244` [enthält `plaintext`]: `Surrogate token collision detected for token '{surrogate}': existing plaintext '{existing_plaintext}' != new plaintext '{new_plaintext}'`
- `/app/crates/contextra-privacy/src/error.rs:28` [enthält `plaintext`]: `Surrogate token collision detected for token '{surrogate}': existing plaintext '{existing_plaintext}' != new plaintext '{new_plaintext}'`
