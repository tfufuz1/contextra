# Contextra

Contextra ist eine air-gap-fähige, kryptografisch beweisbare Memory-Engine für KI-Agenten — als Git-Abhängigkeit einbindbar, ohne externen Serverprozess. Sie vereint Vektor-Einbettungen (HNSW / DiskANN), Volltextsuche (BM25), Wissensgraphen (CSR, Forward-Push PPR, PathRAG), deterministische KV-Cache-Verwaltung (KIVI 2-Bit, Irminsul) und hybride Signal-Fusion in einer modularen Pure-Rust-Bibliothek.

> **Dokumentationsstand (15.10.2026):** Normativ abgestimmt mit der [Contextra Endprodukt-Spezifikation v1.4](docs/spec/CONTEXTRA_ENDPRODUKT_SPEZIFIKATION_v1_4.md).

---

## Produktthese & Alleinstellungsmerkmale

1. **Memory ohne Serverinfrastruktur:** Reine Rust-Bibliothek (`#![forbid(unsafe_code)]` im Kern). Keine HTTP-Overheads, keine Docker-Dienste, kein verdecktes Cloud-Logging.
2. **Kryptografische Löschgarantie (Crypto-Erase / Deletion Proofs):** Revisionssichere, Ed25519-signierte Löschnachweise (v4 TLV) mit hierarchischen Evidence-Layern für DSGVO / AI Act Auditierbarkeit.
3. **Multi-Modal Signal Retrieval:** Hybrides Ranking (RRF / DiBud) über unstrukturierten Text, strukturierte Wissensgraphen und hochdimensionale Vektoren.
4. **Deterministic KV-Cache & Latency Guard:** Integrierte KIVI 2-Bit Quantisierung und bandit-gesteuertes Latency Routing für LLM-Inferenz-Pipelines.

---

## Schnelleinstieg (Quickstart)

Füge Contextra zu deiner `Cargo.toml` hinzu:

```toml
[dependencies]
contextra-engine = { git = "https://github.com/tfufuz1/contextra.git" }
```

Beispielhafte Initialisierung über den Composition Root:

```rust
use contextra_engine::collection::CollectionConfig;
use contextra_engine::ContextraEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Basic memory engine setup
    let engine = ContextraEngine::builder()
        .with_memory_only(true)
        .build()?;

    let collection = engine.create_collection("agent_memory", CollectionConfig::default()).await?;
    println!("Collection initialized: {:?}", collection.name());

    Ok(())
}
```

---

## Workspace Architecture & Crate Overview

Das Repository ist als strikter Ring-DAG (Ringe 0 bis 4) organisiert:

| Ring | Crate | Zweck | Status |
| :--- | :--- | :--- | :--- |
| **Ring 0** | `contextra-types` | Invariantenfreie Grundtypen, IDs (`TenantId`, `DocId`), Fehler-Taxonomie | 🟢 Stable |
| **Ring 0** | `contextra-ports` | Abstrakte Traits (`StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`, `Clock`) | 🟢 Stable |
| **Ring 0** | `contextra-sys` | OS-Insel für `mlock`, Secure Allocator & Process Isolation | 🟢 Stable |
| **Ring 0** | `contextra-simd` | Hand-optimierte AVX2 / NEON / WASM SIMD Kernel | 🟢 Stable |
| **Ring 0** | `contextra-wire` | Low-Level IPC, Zero-Copy Shared Memory & Serialization | 🟢 Stable |
| **Ring 1** | `contextra-store` | LSM-Tree Storage Engine, WAL V3, SSTable, BlockCache, Erasure Ledger | 🟢 Stable |
| **Ring 1** | `contextra-crypto` | AES-256-GCM-SIV, Argon2id, Deletion Proof V4 (Ed25519) | 🟢 Stable |
| **Ring 1** | `contextra-durable-fs` | Crash-sichere FS-Operationen, Durable Removal & Scrubbing | 🟢 Stable |
| **Ring 1** | `contextra-mvcc` | Snapshot Isolation, SSI Validator & Floor Tracker | 🟢 Stable |
| **Ring 2** | `contextra-vector` | HNSW & DiskANN Vektor-Indizes | 🟢 Stable |
| **Ring 2** | `contextra-text` | BM25 & Inverted Index | 🟢 Stable |
| **Ring 2** | `contextra-graph` | CSR Graph, Forward-Push PPR, Hyperedge & PathRAG | 🟢 Stable |
| **Ring 2** | `contextra-kvcache` | KIVI 2-Bit Quantisierter KV-Cache & Prefix Trie | 🟢 Stable |
| **Ring 2** | `contextra-checkpoint` | Persistent Checkpoints & Restore Barrier | 🟢 Stable |
| **Ring 2** | `contextra-mcp` | Model Context Protocol Stdio Server (JSON-RPC 2.0) | 🟢 Stable |
| **Ring 3** | `contextra-engine` | Storage & Index Coordinator, Transaction & Composition Root | 🟢 Stable |
| **Ring 3** | `contextra-db` | High-Level Multi-Tenant Database Facade | 🟢 Stable |
| **Ring 4** | `contextra-py` | PyO3 Python FFI Bindings (`contextra` auf PyPI) | 🟢 Stable |

---

## Qualitätssicherung & Governance (xtask Tooling)

Das Repository erzwingt harten Qualitäts-Doktrinen über `cargo xtask`:

```bash
# Gesamten Workspace verifizieren (Gates, Linters, Type Registry)
cargo xtask verify

# Dokumentations-Link-Gate prüfen (G-33)
cargo xtask check-doc-links

# Ring-DAG und Unsafe-Inseln auditiere
cargo xtask check-dag
cargo xtask check-unsafe-islands
```

---

## Verbindliche Dokumentation & Spezifikationen

- **Agenten-Betriebsanleitung & Harness:** [`AGENTS.md`](AGENTS.md)
- **Contextra Endprodukt-Spezifikation v1.4 (Lebende Quelle):** [`docs/spec/CONTEXTRA_ENDPRODUKT_SPEZIFIKATION_v1_4.md`](docs/spec/CONTEXTRA_ENDPRODUKT_SPEZIFIKATION_v1_4.md)
- **EP1 Memory Feinspezifikation (Archiv):** [`docs/archive/spec/EP1_CONTEXTRA_MEMORY_SPEC.md`](docs/archive/spec/EP1_CONTEXTRA_MEMORY_SPEC.md)
- **EP2 Proof Feinspezifikation (Archiv):** [`docs/archive/spec/EP2_CONTEXTRA_PROOF_SPEC.md`](docs/archive/spec/EP2_CONTEXTRA_PROOF_SPEC.md)
- **EP3 Retrieval+ Feinspezifikation (Archiv):** [`docs/archive/spec/EP3_CONTEXTRA_RETRIEVAL_PLUS_SPEC.md`](docs/archive/spec/EP3_CONTEXTRA_RETRIEVAL_PLUS_SPEC.md)
- **EP4 Inference Feinspezifikation (Archiv):** [`docs/archive/spec/EP4_CONTEXTRA_INFERENCE_SPEC.md`](docs/archive/spec/EP4_CONTEXTRA_INFERENCE_SPEC.md)
- **EP5 Cognition Feinspezifikation (Archiv):** [`docs/archive/spec/EP5_CONTEXTRA_COGNITION_ROUTER_SPEC.md`](docs/archive/spec/EP5_CONTEXTRA_COGNITION_ROUTER_SPEC.md)
- **EP6 Guard Feinspezifikation (Archiv):** [`docs/archive/spec/EP6_CONTEXTRA_GUARD_SANDBOX_SPEC.md`](docs/archive/spec/EP6_CONTEXTRA_GUARD_SANDBOX_SPEC.md)
- **Solo-Entwickler Hardware-Handbuch (Archiv):** [`docs/archive/spec/SOLO_DEVELOPER_GUIDE.md`](docs/archive/spec/SOLO_DEVELOPER_GUIDE.md)
- **Domänentypen-Register:** [`docs/TYPE_REGISTRY.md`](docs/TYPE_REGISTRY.md)
- **Capabilities & Ring-Definition:** [`capabilities.toml`](capabilities.toml)

---

## Lizenz

Copyright © 2026 Contextra Contributors. Dual-licensed under MIT or Apache 2.0.
