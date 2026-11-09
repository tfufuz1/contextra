# contextra-infer-candle

Native Candle GGUF ML Inferenz-Baustein für Contextra (Klasse A).

Normativitätsklasse: **Klasse A – Autarker Baustein** (Ring 2, Baukasten B16).

## Zweck

Führt lokale LLM-/SLM-Modelle und Embedding-Modelle (GGUF, Quantized Llama, BERT) aus.
Funktioniert als autarker Baustein mit zero internal workspace dependencies und bietet bei inaktivem Feature `candle` ein leichtgewichtiges Fake-/Mock-Backend.

## Schnittstellen & Traits

- `Embedder`: Text-Embedding-Generierung (`embed`, `embed_batch`, `dimension`).
- `TextGenerator`: Deterministische Textgenerierung mit `GenerationConfig`.
- `parse_gguf_metadata`: Defensiver GGUF-Header- und Metadaten-Parser mit SHA-256 Validierung.

## Minimalbeispiel (Pure Rust ohne Candle)

```rust
use contextra_infer_candle::{Embedder, FakeEmbedder, TextGenerator, FakeTextGenerator, GenerationConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let embedder = FakeEmbedder::new(384);
    let vector = embedder.embed("Hallo Welt")?;
    assert_eq!(vector.len(), 384);

    let generator = FakeTextGenerator::new("Antwort");
    let config = GenerationConfig::default();
    let text = generator.generate("Was ist Contextra?", &config)?;
    println!("{}", text);
    Ok(())
}
```

## Features

- `default = []` (Pure-Rust Fallbacks und Traits ohne externe Frameworks).
- `candle`: Aktiviert die vollständige Candle-Inferenz und Tokenizer-Unterstützung.
- `cuda`: Aktiviert CUDA-Beschleunigung für Candle.
- `metal`: Aktiviert Metal-Beschleunigung für Candle.
