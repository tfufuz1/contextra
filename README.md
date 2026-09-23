# Contextra

Contextra is an embedded, multi-signal hybrid search vector database written in Rust. It combines dense vector embeddings, full-text BM25 search, graph traversals, and score calibration within a single low-latency local storage engine.

> **Development Status: Alpha**
> Contextra is currently under active development. Core Rust storage, vector indexing, full-text search, and multi-signal fusion engines are fully implemented and tested, but APIs may evolve prior to a 1.0 release.

---

## Capability Overview

| Feature | Current Status | Notes |
|---|---|---|
| **Vector Search (HNSW / DiskANN)** | 🟢 Supported | Fast approximate nearest neighbor search over dense embeddings. |
| **Full-Text BM25 Search** | 🟢 Supported | Tokenization, term frequency indexing, and BM25 scoring. |
| **Multi-Signal Fusion (RRF)** | 🟢 Supported | Reciprocal Rank Fusion combining vector, text, and graph scores. |
| **Graph Signal Traversals** | 🟡 Partial | Graph storage and path traversals require manual `relate()` calls. |
| **Automatic Entity Extraction** | 🔴 Not Implemented | Text chunking does not automatically extract entities/relations into the graph. |
| **Python Bindings (`contextra-py`)** | 🟡 Alpha | Basic PyO3 FFI bindings (see status in crate documentation). |
| **Model Context Protocol (`contextra-mcp`)** | 🟡 Alpha | Local MCP server interface for AI agents. |

---

## Installation

Add `contextra-db` to your `Cargo.toml`:

```toml
[dependencies]
contextra-db = "0.1"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

---

## Quick Start (Rust)

Below is a minimal workflow demonstrating database initialization, document insertion with vector embeddings, semantic search, and document lookup:

```rust
use contextra_db::{Contextra, ContextraConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Open (or create) a database with 4-dimensional vectors
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config("./example_data", config).await?;

    // 2. Insert documents with embeddings and metadata
    db.insert(
        "rust-lang",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"topic": "programming", "text": "Rust is a systems programming language"})),
    )
    .await?;

    db.insert(
        "python-lang",
        &[0.9, 0.1, 0.0, 0.0],
        Some(serde_json::json!({"topic": "programming", "text": "Python is great for AI and data science"})),
    )
    .await?;

    // 3. Semantic search — find the 2 most similar documents
    let query = [0.95, 0.05, 0.0, 0.0];
    let results = db.search(&query, 2).await?;

    for result in &results {
        println!("{} (score: {:.4})", result.id, result.score);
    }

    // 4. Retrieve a specific document by key
    if let Some(doc) = db.get("rust-lang").await? {
        println!("Found: {} {:?}", doc.id, doc.metadata);
    }

    // 5. Clean up
    db.close().await?;
    let _ = tokio::fs::remove_dir_all("./example_data").await;

    Ok(())
}
```

*For the complete executable example, see [`examples/quickstart.rs`](examples/quickstart.rs).*

---

## Language Bindings & Protocol Servers

- **Python Bindings:** See [`crates/contextra-py/README.md`](crates/contextra-py/README.md) for installation and status.
- **Model Context Protocol Server:** See [`crates/contextra-mcp/README.md`](crates/contextra-mcp/README.md) for setup with Claude Desktop / Cursor.

---

## Documentation & Architecture

- **Architecture Overview:** [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- **Contributing Guide:** [`CONTRIBUTING.md`](CONTRIBUTING.md)
- **Developer Guide:** [`DEVELOPERS.md`](DEVELOPERS.md)
- **Normative Architecture Specification:** [`docs/specs/CONTEXTRA_SPEC_v2.md`](docs/specs/CONTEXTRA_SPEC_v2.md)

---

## License

Contextra is dual-licensed under either of the following licenses at your option:

- **Apache License, Version 2.0** ([`LICENSE-APACHE`](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
- **MIT License** ([`LICENSE-MIT`](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
