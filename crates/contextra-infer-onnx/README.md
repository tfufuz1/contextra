# contextra-infer-onnx

ONNX Embeddings und Cross-Encoder Reranking Execution Provider.

Normativitätsklasse: **Rand `[R]`** (Ring 2, `experimental`; §6.0, `capabilities.toml`).

`contextra-infer-onnx` stellt die In-Process ONNX Embedding Engine und Cross-Encoder-Reranker bereit (Ring 2).

## Zweck

Erzeugt Vektor-Embeddings und führt Cross-Encoder-Reranking über ONNX Runtime aus. Dient als optionaler Embedding- und Reranker-Provider.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 2 (Blatt-Crate / Flüchtige Abhängigkeit)
- **Status:** 🟡 Experimentell / Optional
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Baukasten-Eigenschaften (M1–M11)

- **Abhängigkeiten (nur abwärts):** `contextra-types`, `contextra-ports`, `contextra-rank`.
- **Features:**
  - `default = []` (minimaler Build ohne Schwergewichte, kein automatischer Download von ONNX-Modellen oder Binaries).
  - `reranking = ["dep:ort"]`: Aktiviert ONNX Runtime für Cross-Encoder.
  - `onnx = ["dep:ort", "dep:tokenizers", "dep:ndarray", "dep:reqwest", "dep:sha2"]`: Aktiviert vollständige lokale In-Process-Embeddings und Reranking.
- **Offline-First:** Downloads sind standardmäßig verboten (`allow_download = false`). Bei aktiven Downloads sind Prüfsummen und HTTPS-Host-Allowlists verpflichtend.
- **Vfs-Integration:** Dateiprüfungen und Modellvalidierungen unterstützen das abstrakte VFS-Port-Interface (`ensure_onnx_model_exists_vfs`).

## Öffentliche API-Übersicht

- **Embedder:** `TextEmbedder`, `TextEmbedderConfig`, `OnnxEmbedder`, `default_model_cache_dir`, `ensure_onnx_model_exists_vfs`.
- **Reranker:** `CrossEncoderReranker`, `RerankConfig`, `OnnxConfig`, `OnnxRerankerBuilder`, `RerankResult`, `PlattScaledSigmoid`.

## Einbettungsbeispiel

```rust
use contextra_infer_onnx::reranker::CrossEncoderReranker;
use contextra_ports::Reranker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Reranker im isolierten Passthrough-Modus ohne ONNX Runtime
    let reranker = CrossEncoderReranker::passthrough();
    let query = "Contextra Baukasten";
    let candidates = vec![
        "Sichere lokale Speicherkerne".to_string(),
        "Generische Volltextsuche".to_string(),
    ];

    let results = reranker.rerank(query, &candidates).await?;
    assert_eq!(results.len(), 2);
    Ok(())
}
```

Ein vollständiges eigenständiges Beispiel befindet sich unter `examples/fake_reranker.rs`.
