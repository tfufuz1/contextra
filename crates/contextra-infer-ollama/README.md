# contextra-infer-ollama

Autarker REST/HTTP-Adapter für Ollama-Inferenzserver.

Normativitätsklasse: **Klasse A** (Ring 2; SPEC v4.0 §5.1, `capabilities.toml`).

## Zweck

Kommuniziert mit Ollama-Instanzen für Embedding- und LLM-Inferenz, führt Prompt-Formatierung, Contextual-Chunk-Prefixing und Wichtigkeits-Scoring durch.

## Merkmale & Eigenschaften (Klasse A)

- **Unabhängig & Autark:** 0 interne Abhängigkeiten im Core-Build (`default = []`).
- **Feature-Gated I/O:** HTTP-Stack (`reqwest`, `tokio`, `futures`) hinter dem Feature `http`.
- **Typisierte Fehler:** Einheitliches Fehler-Enum `OllamaError` (`#[non_exhaustive]`).
- **Zero-Panic & Determinismus:** Keine `unwrap()`/`expect()`/`panic!` in Produktionspfaden.
- **Transitional Compat:** Feature `ports-compat` für temporäre Rückwärtskompatibilität zu `contextra-ports` und `contextra-types`.

## Features

| Feature | Zweck |
|---|---|
| `http` | Aktiviert asynchronen HTTP-Client, Embedding-Engine und Netzwerktreiber (`reqwest`, `tokio`). |
| `ports-compat` | Implementiert `contextra-ports`-Traits (`EmbeddingProvider`, `LlmTextGenerator`, etc.) für bestehende Workspace-Konsumenten. |

## Verwendung

### Standalone Export
```bash
python3 tools/export_crate.py contextra-infer-ollama --out /tmp/export-contextra-infer-ollama
```

### Minimalbeispiel (mit `http`)
```rust,no_run
#[cfg(feature = "http")]
use contextra_infer_ollama::{OllamaClient, OllamaConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "http")]
    {
        let client = OllamaClient::new("http://localhost:11434");
        let vector = client.embed("nomic-embed-text", "Hello, world!").await?;
        println!("Embedding dimension: {}", vector.len());
    }
    Ok(())
}
```

### Synchrones Prompt-Building (ohne `http`)
```rust
use contextra_infer_ollama::{build_rag_prompt, xml_escape};

let prompt = build_rag_prompt("System context", "Reference data", "User question");
assert!(prompt.contains("<system>"));
```

## Lizenzen

Lizenziert unter `MIT OR Apache-2.0`.
EOF
