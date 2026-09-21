# memfuse-ollama

`memfuse-ollama` stellt den Inferenz-Client für Ollama und Contextual Prefixing bereit (Ring 2).

## Zweck

Kommuniziert mit Ollama-Instanzen für Embedding- und LLM-Inferenz, führt Prompt-Formattierung, Contextual-Chunk-Prefixing und Wichtigkeits-Scoring durch.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 2 (Blatt-Crate / HTTP-Inferenz)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

- **Client & Embedder:** `OllamaClient`, `OllamaConfig`, `OllamaEmbedder`
- **Context Prefixer:** `ContextPrefixer`, `ContextPrefixEngine`, `ContextPrefixConfig`
- **Importance Scoring:** `score_importance`, `ImportanceAssessment`

## Architektur & Verweise

Details zur Ollama-Inferenz finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) und `README.md` §9.4.
