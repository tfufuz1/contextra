# Baseline Report: contextra-infer-ollama & contextra-infer-candle (INFER-3 / IN3.0)

## 1. Übersicht & Status
- **Datum**: 2026-11-09
- **Toolchain**: rustc 1.89.0 (29483883e 2025-08-04)

## 2. Kompilierbarkeits-Status
| Crate | Default-Build | No-Default-Features | Status |
|---|---|---|---|
| `contextra-infer-ollama` | Grün | Grün | Keine Baseline-Reparatur nötig |
| `contextra-infer-candle` | Grün (nach Baseline-Reparatur `thiserror`) | Grün | `thiserror = { workspace = true }` in `Cargo.toml` ergänzt |

## 3. Nullprüfzahlen (Importe contextra_types / contextra_ports / contextra_testkit)
| Crate | Prod-Code | #[cfg(test)] / tests.rs | tests/ Verzeichnis |
|---|---|---|---|
| `contextra-infer-ollama` | 51 | 4 | 0 |
| `contextra-infer-candle` | 39 | 7 | 25 |

## 4. Feature-Befund
### `contextra-infer-ollama`
- Deklarierte Features: `client`, `http`, `types-compat`, `ports-compat`
- Weiterleitungen in Workspace: `contextra-mcp` leitet `ollama` -> `dep:contextra-infer-ollama` weiter.

### `contextra-infer-candle`
- Deklarierte Features: `candle`, `kv-bridge`, `kv-stage-b`, `cuda`, `test-utils`
- Weiterleitungen in Workspace:
  - `contextra`: Feature `candle` -> `contextra-infer-candle`
  - `contextra-mcp`: Feature `candle` -> `dep:contextra-infer-candle`

## 5. Externe Verwender von `contextra_infer_candle::*`
| Datei | Verwendetes Symbol |
|---|---|
| `benchmarks/contextra-bench/src/main.rs` | `CandleEmbedClient::from_dir`, `model_registry::CandleQuantization::Q4KM` |
| `crates/contextra-mcp/src/config.rs` | `CandleEmbedClient::from_dir`, `CandleLlmClient::from_dir`, `model_registry::CandleQuantization::Q4KM` |
| `crates/contextra/src/lib.rs` | `pub use contextra_infer_candle as candle;` |

## 6. Externe Verwender von `contextra_infer_ollama::*`
| Datei | Verwendetes Symbol |
|---|---|
| `crates/contextra-mcp/src/config.rs` | `OllamaEmbedder::new`, `OllamaConfig`, `OllamaClient::with_config` |
