# AGENTS.md — contextra-infer-onnx
> Ring 2 · experimental · Quelle: capabilities.toml · Spec: K.14

## 1. Zweck
Stellt lokale, offline-fähige Vektor-Einbettungen und Cross-Encoder-Reranking bereit. Bietet `TextEmbedder` (`OnnxEmbedder`) für ONNX-Runtime-basierte Einbettungen (Candle-basierte Embedder liegen in `contextra-infer-candle`) sowie `CrossEncoderReranker` zur Präzisionsverbesserung von Retrieval-Ergebnissen.

## 2. Modul-Karte

| Datei / Verzeichnis | Verantwortung |
|---|---|
| `src/lib.rs` | `TextEmbedder`, `TextEmbedderConfig` und Modell-Download (`ensure_onnx_model_download`) |
| `src/reranker/config.rs` | `RerankConfig` mit `MAX_CANDIDATES = 10_000` Allokationsbegrenzung |
| `src/reranker/cross_encoder.rs` | `CrossEncoderReranker` und Kalibrierungs-/Feedback-Methoden (`record_implicit_feedback`) |

## 3. Invarianten

- **Feature-Gating (`onnx` / `candle-backend`):** Die ONNX-Runtime-Abhängigkeit ist hinter `cfg(feature = "onnx")` gekapselt, um Pure-Rust-Builds standardmäßig nicht zu brechen.
- **Kein C++-FFI Cross-Encoder als Standard:** `CrossEncoderReranker` ist backend-abstrahiert; eine C++-FFI-Lösung darf niemals als unüberspringbare Standardabhängigkeit durchgreifen.
- **Kandidatenbegrenzung:** `RerankConfig` deckelt Reranking auf maximal `MAX_CANDIDATES = 10_000` Dokumente pro Aufruf zur Vermeidung von Speichererschöpfung.

## 4. Verboten / Anti-Patterns

- **Verboten:** ONNX- oder Candle-Forward-Ausführungen direkt im async-Executor auszuführen; CPU-intensives Tokenisieren/Inferieren gehört in `tokio::task::spawn_blocking`.
- **Verboten:** Modelle pro Suchanfrage neu zu laden (`TextEmbedder::load`); Instanzen müssen thread-safe wiederverwendet werden.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- `TextEmbedder` und `CrossEncoderReranker` verwalten Modell- und Session-Zustände thread-safe und sind ohne explizite Mutexes parallel nutzbar.
- Schwerlast-Inferenz wird über `embed_async` bzw. `spawn_blocking` an dedizierte Worker-Threads delegiert.

## 6. Verifikation

```bash
cargo test -p contextra-infer-onnx --locked
cargo test -p contextra-infer-onnx --test onnx_embedder_test
cargo test -p contextra-infer-onnx --test reranker_adversarial_test
cargo xtask check-agents-integrity
cargo xtask doctrine-scan --crate contextra-infer-onnx
```

## 7. Bekannte Lücken / SOLL

- Automatische Modelldownloads (`ensure_onnx_model_download`) setzen Netzwerzzugriff oder ein vorausgefülltes Cache-Verzeichnis (`~/.contextra/models`) voraus.
- Das Crate besitzt laut `capabilities.toml` den Reifestatus `experimental`.
