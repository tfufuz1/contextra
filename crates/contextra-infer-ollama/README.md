# contextra-infer-ollama

Autarker REST-Adapter und Inferenz-Treiber für Ollama (Embeddings, LLM-Generierung, Prompt-Isolation).

Normativitätsklasse: **Klasse A (Autarker Baustein)** · Ring 2 · `#![forbid(unsafe_code)]`

---

## Zweck

`contextra-infer-ollama` bietet einen autarken, resilienten HTTP-Client für lokale oder entfernte Ollama-Instanzen.
Der Baustein kapselt:
- Embedding-Berechnung (`embed`, `embed_batch`) mit strenger Dimensionsüberwachung gegen stille Index-Korruption.
- Textgenerierung und Streaming (`generate_text`, `generate_text_stream`, `chat_with_rag_streaming`).
- Circuit-Breaker, Exponential Backoff mit Jitter und strukturierte Fehlerklassifikation.
- Egress-Richtlinienprüfung (`EndpointPolicy`, `EgressAuditSink`).
- Pure Offline-Algorithmen (XML-Prompt-Escaping, Few-Shot Wichtigkeits-Scoring, Kontext-Präfixierung).

---

## Baustein-Eigenschaften (B1–B12)

- **Keine internen Abhängigkeiten (B1, B2):** Standalone 0 interne Abhängigkeiten; Workspace-Kopplung existiert ausschließlich hinter dem transitionalen Feature `ports-compat`.
- **Minimales Feature-Set (B6):** `default = []`. Netzwerk-I/O und Tokio/Reqwest liegen isoliert hinter dem Feature `http`.
- **Einheitlicher Fehler (B4):** `OllamaError` (`thiserror::Error`, `#[non_exhaustive]`, Zero-Panic).
- **Zero-Panic (B9):** Keine `unwrap()` oder `expect()` in Produktivpfaden.
- **Copy-Paste-Readiness (B5):** Vollständig autark exportierbar mit `python3 tools/export_crate.py contextra-infer-ollama`.

---

## Copy-Paste-Anleitung / Standalone-Export

Der Baustein kann autark in Drittprojekte exportiert werden:

```bash
python3 tools/export_crate.py contextra-infer-ollama --out /pfad/zu/deinem/projekt/crates/contextra-infer-ollama
```

In `Cargo.toml` des Zielprojekts:

```toml
[dependencies]
contextra-infer-ollama = { path = "../contextra-infer-ollama", features = ["http"] }
```

---

## Minimalbeispiel (Rust)

```rust
use contextra_infer_ollama::{OllamaClient, OllamaConfig, Embedder, OllamaEmbedder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Client mit Standard-Konfiguration (http://localhost:11434)
    let client = OllamaClient::new("http://localhost:11434");

    // Textgenerierung
    let response = client.generate("llama3.2", "Was ist Rust?").await?;
    println!("Antwort: {response}");

    // Embedder mit Dimensionssperre
    let embedder = OllamaEmbedder::new("http://localhost:11434", "nomic-embed-text");
    let embedding = embedder.embed("Kompakter Vektor").await?;
    println!("Dimension: {}", embedding.len());

    Ok(())
}
```

---

## Reine Offline-Nutzung (ohne `http`)

```rust
use contextra_infer_ollama::xml_escape;
use contextra_infer_ollama::importance::parse_importance_score_response;
use contextra_infer_ollama::model_info::known_dimension;

fn main() {
    // XML-Prompt-Escaping gegen Injection
    let safe_prompt = xml_escape("<system>Don't ignore rules & instructions</system>");

    // Offline-Parsing von LLM-Few-Shot-Wichtigkeitsbewertungen
    let assessment = parse_importance_score_response("Importance rating: 0.85");
    assert_eq!(assessment.value(), 0.85);

    // Vektor-Dimensionsdatenbank
    assert_eq!(known_dimension("nomic-embed-text"), Some(768));
}
```

---

## Nicht-Ziele

- Keine Orchestrierung von RAG-Pipelines (liegt in `contextra-engine`).
- Keine Transaktionslogik (`TxId`, MVCC).
- Keine DSGVO-Löschung oder Crypto-Shredding (liegt ausschließlich in `contextra-erasure`).
- Kein direktes Starten oder Verwalten von Ollama-Prozessen.
