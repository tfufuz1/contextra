# AGENTS.md — contextra-infer-candle
> Ring 2 · stable · Quelle: capabilities.toml · Spec: K.12

## 1. Zweck
Native GGUF-Modellausführung und Vektor-Einbettungen auf Basis von Candle (`candle-core`, `candle-transformers`) für Datenhoheit ohne externe Dienstanbieter. Bietet `CandleLlmClient` für Textgenerierung, `CandleEmbedClient` für Vektor-Embeddings und `KvBridgeAdapter` zur Anbindung an den verschlüsselten KV-Cache. Integriert `GaspValidator` zur Post-Hoc-Halluzinationsprüfung.

## 2. Modul-Karte

| Datei / Verzeichnis | Verantwortung |
|---|---|
| `src/lib.rs` | Public API Re-Exports und Feature-Wiring |
| `src/attention_exporter.rs` | `CandleAttentionExporter` zur Erfassung und Aggregation von Attention-Gewichten |
| `src/embedding.rs` | `CandleEmbedClient`, `BertEmbedModel` und `DefaultCandleEmbedModel` für Vektor-Einbettungen |
| `src/embedding_provider.rs` | `MAX_CANDLE_EMBED_BATCH_SIZE` Konstante für Batching |
| `src/gasp.rs` | `GaspValidator` und `GaspConfig` für Post-Hoc Grounding-Validierung (GASP) |
| `src/gguf_loader.rs` | `parse_gguf_metadata` und `GgufMetadata` zum Auslesen von GGUF-Headern |
| `src/inference/` | `CandleLlmClient`, `CandleModelInner`, `QuantizedLlamaModel` und Prefix-Reuse (`PrefixSeed`, `KvPrefixContext`) |
| `src/kv_bridge.rs` | `KvBridgeAdapter` und `KvCacheKey` für verschlüsselten KV-Cache-Zugriff |
| `src/kv_state.rs` | `KvState` und `LayerKv` zur expliziten Verwaltung von KV-Cache-Tensoren |
| `src/model/` | `ModelWeights`, `LayerWeights` und `QuantizedMatMul` für Llama-GGUF-Forward-Passes |
| `src/model_registry.rs` | `CandleQuantization` Enum und `compute_fingerprint` für Modell-Fingerprints |

## 3. Invarianten

- **Zero-Panic (P7):** Kein `unwrap()`, `expect()` oder `panic!()` in Produktionspfaden; Fehler per `Result` propagieren (`cargo test -p contextra-infer-candle`).
- **Async Backpressure:** Semaphore-Begrenzung für Inferenz (`DEFAULT_MAX_CONCURRENT_INFERENCES = 4`) und Embeddings (`DEFAULT_MAX_CONCURRENT_EMBEDDINGS = 8`) (`cargo test -p contextra-infer-candle --test real_inference_test`).
- **Fallback bei Fehlen von Gewicht-Dateien:** Client-Initialisierung nutzt `DefaultCandleLlmModel` bzw. `DefaultCandleEmbedModel` als Safe Mock (`cargo test -p contextra-infer-candle --test gguf_loader_test`).

## 4. Verboten / Anti-Patterns

- **Verboten:** Blockierende Candle-Tensor-Aktionen direkt im tokio-Executor ausführen; CPU/GPU-Pfade müssen in `tokio::task::spawn_blocking` gekapselt werden.
- **Verboten:** Aufwärts-Imports auf Ring-3-Crates (z.B. `contextra-engine` oder `contextra-mcp`).

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- Semaphore-Steuerung (`tokio::sync::Semaphore`) für gleichzeitige Modellauswertungen verhindert Memory-Exhaustion / OOM.
- Inferenz- und Tensor-Operationen laufen in `spawn_blocking`-Threads ohne Sperren über `.await`-Punkte zu halten.

## 6. Verifikation

```bash
cargo test -p contextra-infer-candle --locked
cargo test -p contextra-infer-candle --test real_inference_test
cargo test -p contextra-infer-candle --test kv_bridge_and_stress_test
cargo xtask check-agents-integrity
cargo xtask doctrine-scan --crate contextra-infer-candle
cargo xtask check-unsafe-islands
```

## 7. Bekannte Lücken / SOLL

- `GaspValidator` ist vollständig implementiert (`gasp.rs`), verlässt sich jedoch auf externe Grounding-Feedback-Quellen.
- `KvBridgeAdapter` erfordert ein aktives Encrypted KV Storage Gateway für verschlüsselte Segment-Persistenz.
