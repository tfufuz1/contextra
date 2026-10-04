# AGENTS.md — contextra-infer-ollama
> Ring 2 · stable · Quelle: capabilities.toml · Spec: K.13, L.10

## 1. Zweck
Schnittstelle zu lokalen oder externen Ollama-Instanzen. Stellt `OllamaClient` für HTTP-Textgenerierung und RAG-Streaming, `OllamaEmbedder` für Vektor-Embeddings, `ContextPrefixEngine` für kontextuelles Chunk-Präfixen und `OllamaQueryRewriter` für iterative Anfragenanpassung bereit.

## 2. Modul-Karte

| Datei / Verzeichnis | Verantwortung |
|---|---|
| `src/lib.rs` | Public API Re-Exports und Crate-Dokumentation |
| `src/client/` | `OllamaClient`, `OllamaConfig`, Error-Klassifizierung (`is_transient_network_error`) und XML-Escaping (`build_rag_prompt`) |
| `src/context_prefixer.rs` | `ContextPrefixEngine` (Alias `ContextPrefixer`) und Wortgrenzen-Truncation (`truncate_prefix`) |
| `src/embedding.rs` | `OllamaEmbedder` mit `TextEmbeddingEngine`-Implementierung |
| `src/importance.rs` | LLM-gestützte Chunk-Wichtigkeitsbewertung (`score_importance`, `score_importance_batch`) mit `IsotonicCalibrator` |
| `src/model_info.rs` | `ModelInfo` und statische Dimensionsermittlung (`known_dimension`) |
| `src/query_rewriter.rs` | `OllamaQueryRewriter` und `OllamaQueryRewriterConfig` zur Abfrage-Umformulierung |

## 3. Invarianten

- **Keine Root-Abhängigkeit:** Externe HTTP-Inferenz ist opt-in in Ring 2 und darf nie als Root-Standardabhängigkeit verwendet werden (`cargo test -p contextra-infer-ollama`).
- **Input- & Batch-Limits:** Validierung erzwingt `MAX_TEXT_BYTES = 10_000_000` (10 MB) und `MAX_BATCH_SIZE = 512` zur Vermeidung von OOM/DoS (`client/validation.rs`).
- **Prompt-Injection-Schutz:** RAG-Prompts nutzen XML-Escaping (`xml_escape`) und Tag-Isolation (`<rag_context>`, `<user_query>`).
- **Präfix-Grenzen:** `truncate_prefix` wahrt Wort- und Codepoint-Grenzen bis `max_tokens` / `max_chars` (`context_prefixer.rs`).

## 4. Verboten / Anti-Patterns

- **Verboten:** Direkte HTTP-Aufrufe an Ollama ohne `OllamaClient` vorbeizuführen (umgeht Retry-Logik und Timeout-Handling).
- **Verboten:** Modelling- oder Batch-Aufrufe ohne Längen- und Modellsicherheitsprüfungen auszuführen.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- `OllamaClient` verarbeitet Requests thread-safe über einen gekapselten `reqwest::Client` mit Connection-Pool.
- Batch-Processing in `score_importance_batch` nutzt `tokio::spawn` mit expliziter Concurrency-Begrenzung (`max_concurrent`).
- `generate_prefix_batch` in `ContextPrefixEngine` arbeitet sequenziell, um GPU-Exhaustion bei lokalen Modellen zu verhindern.

## 6. Verifikation

- `cargo test -p contextra-infer-ollama --locked`
- `cargo test -p contextra-infer-ollama --test campaign_stub_ollama`

## 7. Bekannte Lücken / SOLL

- `ContextPrefixEngine` setzt eine erreichbare Ollama-Instanz voraus; bei Nichterreichbarkeit schlägt der Aufruf mit `ContextraError::Io` / `Storage` fehl.
- `known_dimension` liefert für unbesetzte Modelle `None`; in diesem Fall muss die Dimension dynamisch über ein Test-Embedding ermittelt werden.
