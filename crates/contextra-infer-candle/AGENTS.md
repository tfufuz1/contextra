# AGENTS.md — contextra-infer-candle
> Layer 3 | Native Candle GGUF ML Inferenz & Embedding Provider | ~800 LOC

## 1. Zweck & Architekturrolle

Inferenz-Backend auf Basis von Candle (`candle-core`, `candle-transformers`) für native GGUF-Modellausführung (Datenhoheit ohne externe Services).
In `contextra-db` als `EmbeddingBackend::Candle` in die Haupt-Serving-Pipeline eingebunden.

## 2. Modul-Karte

| Datei | Verantwortung |
|---|---|
| `lib.rs` | Crate-Exports und Initialisierung |
| `inference.rs` | Candle GGUF LlmTextGenerator Implementierung (Inkrementelle Inferenz-Semaphore & Backpressure-Härtung H-5 geschlossen) |
| `embedding.rs` | Candle GGUF EmbeddingProvider Implementierung |

## 3. Kritische Invarianten

### Zero-Panic-Doctrine
Keinesfalls `.unwrap()` oder `.expect()` im Produktionscode verwenden.

### Async Thread Safety
Candle-Tensor-Operationen sind CPU-blockierend. Alle Inferenz- und Embed-Aufrufe MÜSSEN via `tokio::task::spawn_blocking` ausgeführt werden.

### Error Handling
Alle Fehler sind als `ContextraError` zu strukturieren.

## 4. Public API Quick-Reference

```rust
pub struct CandleLlmGenerator { ... }
pub struct CandleEmbedClient { ... }
```

## 5. Anti-Patterns & LLM-Fallstricke

```rust
// ❌ FALSCH — Direct blocking Candle tensor ops inside async:
let output = model.forward(&input)?;

// ✅ KORREKT — Wrapped in spawn_blocking:
tokio::task::spawn_blocking(move || model.forward(&input)).await??;
```

## 6. Concurrency & Lock-Hierarchie

Inferenz-Sessions verwalten Thread-sichere Gewichte und Caches. Blocking Thread Pools trennen Heavy ML-Tensors vom Tokio Async Reactor.

## 7. Cross-Crate-Schnittstellen & DAG-Grenzen

- **Erlaubte Imports**: `contextra-core` (L0)
- **Verbotene Imports**: `contextra-mcp` (L4 Upper)

## 8. Relevante ADRs & Rules

| ADR/Rule | Relevanz |
|---|---|
| Strategy B | Native Candle GGUF Inferenz |

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: kein direkter Bezug zum Storage-Umbau. Es nutzt `contextra-store` nicht im Code (Stand 22.11.2026) und ändert sich bis G-6 nicht. Bei Fragen zum Kern gilt `docs/SPEC.md` (R8).
