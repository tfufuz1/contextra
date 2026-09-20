# memfuse-embed

`memfuse-embed` stellt die In-Process ONNX Embedding Engine und Cross-Encoder-Reranker bereit (Ring 2).

## Zweck

Erzeugt Vektor-Embeddings und führt Cross-Encoder-Reranking über ONNX Runtime aus. Dient als optionaler Embedding-Provider.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 2 (Blatt-Crate / Flüchtige Abhängigkeit)
- **Status:** 🟡 In Migration (Zielzustand: `memfuse-infer-onnx`)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

- **Embedder:** `TextEmbedder`, `TextEmbedderConfig`, `OnnxEmbedder`
- **Reranker:** `CrossEncoderReranker`, `RerankConfig`, `RerankResult`

## Architektur & Verweise

Details zum Umbau nach `memfuse-infer-onnx` finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) und `README.md` §9.4.
