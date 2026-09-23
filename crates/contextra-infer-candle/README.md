# contextra-candle

`contextra-candle` stellt die native GGUF-Inferenz und In-Memory-Inferenzunterstützung via Candle bereit (Ring 2).

## Zweck

Führt lokale LLM-/SLM-Modelle im GGUF-Format ohne externe Serverabhängigkeit aus. Dient als Grundlage für eingebettetes Local Retrieval und Prompt-Verarbeitung.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 2 (Blatt-Crate / Flüchtige Abhängigkeit)
- **Status:** 🟡 In Migration (Zielzustand: `contextra-infer-candle`)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

- **Client & Embedding:** `CandleEmbedClient`, `CandleEmbedConfig`
- **Model Registry & Fingerprint:** `ModelFingerprint`
- **Grounding & Validation:** `GaspConfig`, `GaspValidator`

## Architektur & Verweise

Details zum schrittweisen Umbau in `contextra-infer-candle` gemäß §20 finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) und `README.md` §9.
