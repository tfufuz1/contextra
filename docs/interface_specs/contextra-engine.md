# Schnittstellenspezifikation: `contextra-engine`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-engine` |
| Version | workspace |
| Pfad | `crates/contextra-engine` |
| Rolle | Engine (Re-Export db) |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # contextra-engine (Deprecated Façade)
>
> Dieses Crate wurde gemäß SPEC(11).md S-63/S-64 und ADR-0122 aufgelöst.
> Alle Funktionalitäten sind nach `contextra-db` migriert.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `bytes` | dev | workspace | nein |  |
| `contextra-checkpoint` | dev | workspace | nein | contextra-agent, contextra-db, contextra-engine |
| `contextra-db` | normal | workspace | nein | contextra, contextra-agent, contextra-bench, contextra-cognition, contextra-db-fuzz, contextra-engine |
| `contextra-graph` | dev | workspace | nein | contextra-agent, contextra-cognition, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz |
| `contextra-ingest` | dev | workspace | nein | contextra-db, contextra-engine, contextra-ingest-fuzz |
| `contextra-ports` | dev | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-rank` | dev | workspace | nein | contextra-bench, contextra-cognition, contextra-db, contextra-engine, contextra-infer-candle, contextra-mcp |
| `contextra-store` | dev | workspace | nein | contextra-agent, contextra-bench, contextra-db, contextra-engine, contextra-store-fuzz |
| `contextra-testkit` | dev | workspace | nein | contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-store |
| `contextra-text` | dev | workspace | nein | contextra-agent, contextra-bench, contextra-cognition, contextra-db, contextra-engine, contextra-text-fuzz |
| `contextra-types` | dev | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `contextra-vector` | dev | workspace | nein | contextra-agent, contextra-bench, contextra-db, contextra-engine, contextra-vector-fuzz |
| `proptest` | dev | registry | nein |  |
| `serde` | dev | workspace | nein |  |
| `serde_json` | dev | workspace | nein |  |
| `sha2` | dev | registry | nein |  |
| `tempfile` | dev | workspace | nein |  |
| `tokio` | dev | workspace | nein |  |
| `tracing` | dev | workspace | nein |  |

**Verwendet von (Crate-Ebene):** –

## 3. Features

| Feature | Aktiviert |
|---|---|
| `adaptive-candidate-pool-sizing` | `contextra-db/adaptive-candidate-pool-sizing` |
| `auto-extraction-opt-out` | `contextra-db/auto-extraction-opt-out` |
| `background-maintenance` | `contextra-db/background-maintenance` |
| `default` | – |
| `edge-reinforcement-learning` | `contextra-db/edge-reinforcement-learning` |
| `entity-extraction` | `contextra-db/entity-extraction` |
| `exp-extraction` | `contextra-db/exp-extraction` |
| `experimental-diskann` | `contextra-db/experimental-diskann` |
| `graph-connectivity-health` | `contextra-db/graph-connectivity-health` |
| `onnx` | `contextra-db/onnx` |
| `reranking` | `contextra-db/reranking` |
| `sandbox` | `contextra-db/sandbox` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 1 |
| Quellzeilen | 15 |
| Öffentliche Items (Modul-Ebene) | 0 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 1 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 0 |
| `#[deprecated]` | 1 |
| `#[cfg(feature…)]`-Gates | 0 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 0 / 0 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `crate` | `crates/contextra-engine/src/lib.rs` | 15 |

## 6. Re-Exports (`pub use`)

- `pub use contextra_db::*` — `crates/contextra-engine/src/lib.rs:15` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item
