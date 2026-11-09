# contextra-infer-onnx

Autarke In-Process ONNX Inferenz Engine für Embeddings und Cross-Encoder Reranking (Klasse A, Ring 2).

## Baukasten-Profil (SPEC v4.0, B1–B12)
- **Ring:** Ring 2
- **Klasse:** A (Vollkommen autarker Baustein)
- **0 Interne Abhängigkeiten:** Baut und testet vollständig isoliert ohne Abhängigkeiten zu `contextra-*`.
- **Zero-Panic Invariante:** `#![forbid(unsafe_code)]`, keine Panics/Unwraps in Produktivpfaden, bounds-checked Tensoren.
- **Determinismus:** Deterministische Fake-Implementierungen (`FakeEmbedder`, `FakeReranker`) für Standalone-Betrieb.

## Features
- `default = []` (reiner Pure-Rust Minimalbau ohne externe Shared-Libraries oder C-FFI).
- `onnx = ["dep:ort", "dep:tokenizers", "dep:ndarray", "dep:sha2"]` (aktiviert In-Process Inferenz via ONNX Runtime).
- `download = ["dep:reqwest"]` (optionaler Download von Modellen über gesicherte HTTPS-Kanäle).
- `std`, `serde`, `tracing`, `inspect`, `experimental` (Standard-Baukasten Features).

## Schnelleinstieg

```rust
use contextra_infer_onnx::traits::{Embedder, Reranker};
use contextra_infer_onnx::{FakeEmbedder, FakeReranker};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Deterministischer Fake-Embedder für Tests und Offline-Betrieb
    let embedder = FakeEmbedder::new(128)?;
    let vec = embedder.embed("Contextra Autarker Baustein")?;
    assert_eq!(vec.len(), 128);

    // Deterministischer Cross-Encoder Reranker
    let reranker = FakeReranker::new();
    let scores = reranker.score("query", &["relevanter text", "irrelevanter text"])?;
    assert_eq!(scores.len(), 2);
    Ok(())
}
```
