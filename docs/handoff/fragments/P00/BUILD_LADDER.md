# BUILD_LADDER (Welle 0 - P00)

Status aller 36 Workspace-Mitglieder nach Ausführung von WP-01 bis WP-03.

| Crate | Ergebnis (grün/rot) | erster Fehler (1 Zeile) | Ursache in Crate | zuständiger Auftrag |
|---|---|---|---|---|
| `contextra-types` | grün | - | `-` | - |
| `contextra-ports` | grün | - | `-` | - |
| `contextra-sys` | grün | - | `-` | - |
| `contextra-wire` | grün | - | `-` | - |
| `contextra-mvcc` | grün | - | `-` | - |
| `contextra-simd` | grün | - | `-` | - |
| `contextra-crypto` | rot | error: unexpected `cfg` condition value: `serde` | `contextra-crypto` | unzugeordnet |
| `contextra-store` | rot | error: unexpected `cfg` condition value: `serde` | `contextra-crypto` | unzugeordnet |
| `contextra-rank` | grün | - | `-` | - |
| `contextra-text` | grün | - | `-` | - |
| `contextra-graph` | rot | error[E0428]: the name `contract` is defined multiple times | `contextra-graph` | P06 |
| `contextra-vector` | rot | error[E0592]: duplicate definitions with name `remove` | `contextra-vector` | P03 |
| `contextra-checkpoint` | grün | - | `-` | - |
| `contextra-engine` | rot | error[E0428]: the name `contract` is defined multiple times | `contextra-graph` | P06 |
| `contextra-db` | rot | error[E0428]: the name `contract` is defined multiple times | `contextra-graph` | P06 |
| `contextra` | rot | error[E0428]: the name `contract` is defined multiple times | `contextra-graph` | P06 |
| `contextra-mcp` | rot | error[E0428]: the name `contract` is defined multiple times | `contextra-graph` | P06 |
| `contextra-testkit` | grün | - | `-` | - |
| `contextra-kvcache` | rot | error: unexpected `cfg` condition value: `serde` | `contextra-crypto` | unzugeordnet |
| `contextra-durable-fs` | grün | - | `-` | - |
| `contextra-ingest` | grün | - | `-` | - |
| `contextra-adapt` | grün | - | `-` | - |
| `contextra-router` | grün | - | `-` | - |
| `contextra-privacy` | grün | - | `-` | - |
| `contextra-infer-ollama` | grün | - | `-` | - |
| `contextra-py` | rot | error[E0428]: the name `contract` is defined multiple times | `contextra-graph` | P06 |
| `contextra-bench` | rot | error: unexpected `cfg` condition value: `serde` | `contextra-crypto` | unzugeordnet |
| `contextra-infer-candle` | rot | error[E0433]: failed to resolve: use of unresolved module or unlinked crate `thiserror` | `contextra-infer-candle` | unzugeordnet |
| `contextra-infer-onnx` | grün | - | `-` | - |
| `contextra-sandbox` | grün | - | `-` | - |
| `contextra-agent` | rot | error[E0428]: the name `contract` is defined multiple times | `contextra-graph` | P06 |
| `contextra-cognition` | rot | error[E0428]: the name `contract` is defined multiple times | `contextra-graph` | P06 |
| `contextra-license` | grün | - | `-` | - |
| `contextra-erasure` | grün | - | `-` | - |
| `contextra-audit-export` | grün | - | `-` | - |
| `contextra-avv-generator` | grün | - | `-` | - |
