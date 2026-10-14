# Contextra

Contextra ist eine air-gap-fähige, kryptografisch beweisbare Memory-Engine für KI-Agenten — als Git-Abhängigkeit einbindbar, ohne externen Serverprozess. Sie vereint Vektor-Einbettungen (HNSW / DiskANN), Volltextsuche (BM25), Wissensgraphen (CSR, Forward-Push PPR, PathRAG), deterministische KV-Cache-Verwaltung (KIVI 2-Bit, Irminsul) und hybride Signal-Fusion in einer modularen Pure-Rust-Bibliothek.

> **Dokumentationsstand (14.10.2026):** Normativ abgestimmt mit der [Disjunktiven Endprodukt-Gesamtspezifikation (SOLL-Zustand v1.0)](docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md).

---

## Produktthese & Alleinstellungsmerkmale

1. **Beweisbarkeit statt Zusage (EP2):** Das Löschen von Daten erzeugt im Modus `ErasureMode::Attested` einen signierten `DeletionProofV4` (Ed25519, BSI TR-02102-orientiert), der das physische Tilgen auf allen 7 Systemspeicherebenen testiert. Externe Verifikation ist über das autonome Crate `contextra-proof-verify` mathematisch unabhängig ohne Datenbankzugriff möglich.
2. **Air-Gap-Fähigkeit & Ein-Prozess-Garantie:** Läuft vollständig ohne Netzwerk, ohne externe API-Keys und ohne Telemetrie im selben Prozess wie die Anwendung.
3. **Hardware-angepasste Inferenz (EP4):**
   - **Ollama (`contextra-infer-ollama`):** Ressourcenschonender Standardtreiber für lokale LLMs.
   - **Candle (`contextra-infer-candle`):** Pure-Rust GGUF-Inferenz als Opt-in für direkten KV-Tensoren-Zugriff.
4. **Deterministische Performance (REG-03 / P28):** Injektion aller Zeit-, Zufalls- und ID-Quellen über Ports (`contextra-ports`), konsequente `f64::total_cmp`-Gleitkomma-Sortierung und Zero-Panic-Doktrin (REG-01).
5. **Zero-Cost-Plugin-Doktrin (REG-07):** Wer den Löschnachweis nicht benötigt (`ErasureMode::Off`), zahlt 0 Byte Binary-Overhead und 0 ns Lese-Overhead.

---

## Die 6 Disjunktiven Endprodukte (EP1 bis EP6)

Contextra ist modular in sechs klar abgegrenzte Produktkomponenten aufgeteilt:

| Komponente | Bezeichnung | Status | Kernversprechen & Disjunktive Crates |
|---|---|---|---|
| **EP1** | **Contextra Memory** | 🟢 **Release 0.1.0** (Target Hülle ≤ 18 Crates) | Lokaler Hybrid-Speicher, SSI-Transaktionen, Mandanten-Scoping, Purge v2 (`contextra-store`, `contextra-vector`, `contextra-text`, `contextra-mvcc`, `contextra-engine`, `contextra`). |
| **EP2** | **Contextra Proof** | 🟢 **Spezifiziert** | Layer-Löschnachweis v4 (Ed25519), Standalone-Verifier, Art. 30 / AVV-Generator (`contextra-crypto-core`, `contextra-crypto-kv`, `contextra-erasure-evidence`, `contextra-proof-verify`, `contextra-audit-export`, `contextra-avv-generator`). |
| **EP3** | **Contextra Retrieval+** | 🟢 **Spezifiziert** | Relationale Wissensgraphen (CSR), OpenIE Extraktion, Budgeted k-Path Diffusion (`PathRAG`), Conformal Fusion Calibration (`contextra-graph`, `contextra-rank`). |
| **EP4** | **Contextra Inference** | 🟢 **Spezifiziert** | Irminsul Radix-Trie Prefix-Reuse, KIVI 2-Bit asymmetrische Quantisierung, SnapKV Attention Eviction (`contextra-kvcache`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-checkpoint`). |
| **EP5** | **Contextra Cognition** | 🟢 **Spezifiziert** | Adaptive Kognition, Sketched LinUCB Bandits ($\mathcal{O}(k^2)$ Achlioptas-Projektion), PID Latency Controller, Hyperedge Spectral SVD (`contextra-cognition`, `contextra-adapt`, `contextra-router`, `contextra-agent`). |
| **EP6** | **Contextra Guard** | 🟢 **Spezifiziert** | Egress-Firewall (Luhn/IBAN DLP), Fail-Closed MCP Gateway, Hash-Chained Audit Trace, WASM Sandbox Isolation (`contextra-privacy`, `contextra-sandbox`, `contextra-mcp`, `contextra-py`, `contextra-license`). |

---

## Entwickler- & Agenten-Workflow (`cargo xtask`)

Contextra verwendet `cargo xtask` als **einzige Quelle der Wahrheit** für Analyse, Diagnose, Refactoring und Governance.

```bash
# 1. Systemdiagnose & Status-Dashboard
cargo xtask repo-doctor

# 2. Workspace-weites Lücken-Audit
cargo xtask find-unwired-gaps

# 3. Kontext eines spezifischen Crates abrufen
cargo xtask crate-context contextra-store

# 4. Doktrin- und Zero-Panic-Prüfung für ein Crate
cargo xtask doctrine-scan --crate contextra-store

# 5. Ring-Architektur & DAG-Integrität prüfen
cargo xtask check-ring-capabilities-consistency
cargo xtask check-unsafe-islands
cargo xtask check-ring-layering

# 6. Vollständige Verifikation
cargo xtask verify
```

Ausführliche Anweisungen finden sich in [`AGENTS.md`](AGENTS.md).

---

## Quick Start (Rust)

```rust
use contextra::{Contextra, ContextraConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config("./example_data", config).await?;

    db.insert(
        "rust-lang",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({
            "topic": "programming",
            "text": "Rust is a systems programming language with zero-cost abstractions"
        })),
    )
    .await?;

    let query = [0.95, 0.05, 0.0, 0.0];
    let results = db.search(&query, 2).await?;

    for result in &results {
        println!("Hit: {} (Score: {:.4})", result.id, result.score);
    }

    db.close().await?;
    let _ = tokio::fs::remove_dir_all("./example_data").await;

    Ok(())
}
```

---

## Verbindliche Dokumentation & Spezifikationen

- **Endprodukt-Spezifikation (SOLL-Zustand v1.0):** [`docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md`](docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md)
- **Agenten-Betriebsanleitung & Harness:** [`AGENTS.md`](AGENTS.md)
- **Source of Truth:** [`docs/SOURCE_OF_TRUTH.md`](docs/SOURCE_OF_TRUTH.md)
- **Domänentypen-Register:** [`docs/TYPE_REGISTRY.md`](docs/TYPE_REGISTRY.md)
- **Capabilities & Ring-Definition:** [`capabilities.toml`](capabilities.toml)
