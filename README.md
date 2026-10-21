# Contextra

Contextra ist ein lokaler MCP-Coding-Kontext-Server und eine eingebettete Memory-Engine in Pure Rust (`cargo add`). In Version v1.0 liegt der Fokus auf lokaler Repository-Indizierung, strukturierter Kontextbereitstellung mit Herkunftsnachweis, lokaler GGUF-Inferenz (Qwen-Coder via Candle) und exaktem KV-Präfix-Caching.

> **Dokumentationsstand:** V1.0-Ausrichtung gemäß **[ADR-V1-0001](docs/decisions/ADR-V1-0001-local-coding-context.md)**.

---

## Status v1.0-Ausrichtung

In der Version v1.0 ist Contextra als schlanker, lokaler MCP-Coding-Kontext-Server (stdio) definiert:
- **Produktdefinition:** Lokaler MCP-Server für KI-gestützte Entwicklungsumgebungen.
- **Funktionsumfang:** Repository-Indexierung, strukturierte Kontextblöcke mit Herkunftsnachweis, lokale GGUF-Inferenz (Qwen-Coder via Candle) und exakter KV-Präfix-Cache (`contextra-kvcache`).
- **Löschverhalten:** `contextra_forget` markiert Dokumente als gelöscht (Tombstone); ein Löschnachweis ist für v1.5 geplant.
- **Architekturentscheidung:** Details zu geparkten Modulen (Wissensgraph, Routing, Löschnachweis, Python-Bindings, Agenten-Workflows, Compliance) und der Wiederherstellungs-Roadmap finden sich in [ADR-V1-0001](docs/decisions/ADR-V1-0001-local-coding-context.md).

---

## Produktthese & Alleinstellungsmerkmale

1. **Löschen via Tombstone:** `contextra_forget` markiert Dokumente als gelöscht (Tombstone); ein Löschnachweis ist für v1.5 geplant.
2. **Air-Gap-Fähigkeit & Ein-Prozess-Garantie:** Läuft vollständig ohne Netzwerk, ohne externe API-Keys, ohne separaten Serverprozess und ohne Telemetrie im selben Prozess wie die Anwendung des Nutzers (`cargo add contextra`).
3. **Pure Rust Inferenz (Candle & Qwen-Coder):** Lokale GGUF-Inferenz ohne C++ / CUDA FFI-Abhängigkeiten oder extern laufende Dämonen.
4. **Deterministische Performance:** Pure Rust, kein GC, Zero-Panic-Garantie im Produktionspfad und injizierter Determinismus.
5. **v1.0-Fokus:** Vektor- und Volltextsuche, exakter KV-Präfix-Cache und Candle-Inferenz. Erweiterte Features (Wissensgraph, Routing, Löschnachweis, Python-Bindings, Agenten-Workflows) sind geparkt (siehe [ADR-V1-0001](docs/decisions/ADR-V1-0001-local-coding-context.md)).

---

## Capability Overview

| Feature | Ring | Subsystem / Crate | Status | Anmerkung |
|---|---|---|---|---|
| **Vector Search (HNSW / DiskANN)** | `fast` | `contextra-vector` | 🟢 v1.0 Aktiv | HNSW, DiskANN, SQ8 / RaBitQ Quantisierung. |
| **Full-Text Search (BM25 / BM25F)** | `fast` | `contextra-text` | 🟢 v1.0 Aktiv | BM25/BM25F mit Block-Max WAND & deutscher Morphologie. |
| **LSM Storage Engine & WAL** | `fast` | `contextra-store` | 🟢 v1.0 Aktiv | LSM-Tree, WAL (Group-Commit, HMAC-Kette), MVCC-Pinning. `contextra_forget` markiert Dokumente als gelöscht (Tombstone); ein Löschnachweis ist für v1.5 geplant. |
| **KV-Cache v2 & Zero-Copy IPC** | `fast` | `contextra-kvcache` | 🟢 v1.0 Aktiv | Exakter Präfix-Radix-Baum, Tiering, Segmentdateien. |
| **Local Inference Backend (Candle)** | `fast` | `contextra-infer-candle` | 🟢 v1.0 Aktiv | Pure Rust GGUF/Candle mit Qwen-Coder als Standard-Inferenzbackend. |
| **Model Context Protocol** | `fast` | `contextra-mcp` | 🟢 v1.0 Aktiv | JSON-RPC 2.0 stdio MCP Server für Coding-Kontext. |
| **Knowledge Graph & PPR** | `fast` | `contextra-graph` | 📦 Geparkt | Geparkt für v1.2 (siehe [ADR-V1-0001](docs/decisions/ADR-V1-0001-local-coding-context.md)). |
| **Contextual-Bandit-Routing** | `fast` | `contextra-adapt` | 📦 Geparkt | Geparkt für v1.1 (siehe [ADR-V1-0001](docs/decisions/ADR-V1-0001-local-coding-context.md)). |
| **Memory Consolidation & Cognition** | `fast` | `contextra-cognition` | 📦 Geparkt | Geparkt für v1.2 (siehe [ADR-V1-0001](docs/decisions/ADR-V1-0001-local-coding-context.md)). |
| **Persistent Checkpoint Engine** | `fast` | `contextra-checkpoint` | 📦 Geparkt | Geparkt für v1.4 (siehe [ADR-V1-0001](docs/decisions/ADR-V1-0001-local-coding-context.md)). |
| **Persistent Agent Workflows** | `fast` | `contextra-agent` | 📦 Geparkt | Geparkt für v1.6 (siehe [ADR-V1-0001](docs/decisions/ADR-V1-0001-local-coding-context.md)). |
| **Python Bindings (`contextra-py`)** | Opt-in | `contextra-py` | 📦 Geparkt | Geparkt für v1.6 (siehe [ADR-V1-0001](docs/decisions/ADR-V1-0001-local-coding-context.md)). |
| **WASM-Sandbox** | Opt-in | `contextra-sandbox` | 📦 Geparkt | Geparkt für v1.8 (siehe [ADR-V1-0001](docs/decisions/ADR-V1-0001-local-coding-context.md)). |
| **Kryptographischer Löschnachweis** | `sovereign` | `contextra-crypto` | 📦 Geparkt | `contextra_forget` markiert Dokumente als gelöscht (Tombstone); ein Löschnachweis ist für v1.5 geplant (siehe [ADR-V1-0001](docs/decisions/ADR-V1-0001-local-coding-context.md)). |
| **Privacy Gateway** | `sovereign` | `contextra-privacy` | 📦 Geparkt | Geparkt für v1.5 (siehe [ADR-V1-0001](docs/decisions/ADR-V1-0001-local-coding-context.md)). |
| **AVV Generator (Art. 28 DSGVO)** | `sovereign` | `contextra-avv-generator` | 📦 Geparkt | Geparkt für v1.7 (siehe [ADR-V1-0001](docs/decisions/ADR-V1-0001-local-coding-context.md)). |
| **Audit- & Art.-30-Export** | `sovereign` | `contextra-audit-export` | 📦 Geparkt | Geparkt für v1.7 (siehe [ADR-V1-0001](docs/decisions/ADR-V1-0001-local-coding-context.md)). |
| **Lizenz- & Compliance-Schicht** | `compliance` | `contextra-license` | 📦 Geparkt | Geparkt für v1.7 (siehe [ADR-V1-0001](docs/decisions/ADR-V1-0001-local-coding-context.md)). |

---

## Installation & Einbindung

Füge `contextra` zu deiner `Cargo.toml` hinzu:

```toml
[dependencies]
contextra = "0.1"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

---

## Quick Start (Rust)

```rust
use contextra_db::{Contextra, ContextraConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Datenbankinstanz mit Vektordimension 4 öffnen
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config("./example_data", config).await?;

    // 2. Dokumente mit Vektoreinbettungen und Metadaten einfügen
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

    // 3. Semantische Suche nach den Top 2 treffendsten Dokumenten
    let query = [0.95, 0.05, 0.0, 0.0];
    let results = db.search(&query, 2).await?;

    for result in &results {
        println!("{} (score: {:.4})", result.id, result.score);
    }

    // 4. Dokument per ID abrufen
    if let Some(doc) = db.get("rust-lang").await? {
        println!("Gefunden: {} {:?}", doc.id, doc.metadata);
    }

    // 5. Ordnungsgemäß schließen
    db.close().await?;
    let _ = tokio::fs::remove_dir_all("./example_data").await;

    Ok(())
}
```

---

## Dokumentation & Spezifikationen

- **Lokaler Coding-Kontext & Parken (v1.0):** [`docs/decisions/ADR-V1-0001-local-coding-context.md`](docs/decisions/ADR-V1-0001-local-coding-context.md)
- **Normative Gesamtspezifikation (Synthese):** [`docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`](docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md)
- **Systemarchitektur & Ring-Modell:** [`ARCHITECTURE.md`](ARCHITECTURE.md)
- **Agenten-Betriebsanleitung:** [`AGENTS.md`](AGENTS.md)
- **Sicherheitsrichtlinie:** [`SECURITY.md`](SECURITY.md)
- **Beitragswesen:** [`CONTRIBUTING.md`](CONTRIBUTING.md)

---

## Benchmark-Kennzahlen

Die folgenden Leistungskennzahlen stammen aus reproduzierbaren Läufen der `criterion`-Benchmark-Suite (`benchmarks/contextra-bench`) sowie systematischen Messberichten (siehe [`docs/reports/performance_baseline_2026-09-25.md`](docs/reports/performance_baseline_2026-09-25.md) und [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md)):

> **Wichtiger Hinweis zur Latenz-Abgrenzung:** Misst ausschließlich Speicher- und Indexlatenz. Embedding-Inferenz (Candle) addiert je nach Modell und Hardware Latenz pro Anfrage.

| Kennzahl | Contextra (Local Engine) | Beschreibung & Rahmenbedingungen |
|---|---|---|
| **1. Kaltstart / Recovery** | **Offen / Messung in Arbeit** | Recovery-Zeit für 1 Mio. Einträge in [`docs/reports/performance_baseline_2026-09-25.md`](docs/reports/performance_baseline_2026-09-25.md) (§2.e) als nicht gemessen / fehlerhaft dokumentiert (File-Descriptor-Thematik). |
| **2. Footprint** | **~5,5–12,8 MB Base RSS / 148,85 MB Peak** | RAM Base-Footprint ~5,5–12,8 MB bei kleinen Korpora bis 100 Chunks ([`benches/results/scale_rss.csv`](benches/results/scale_rss.csv)); Peak VmRSS 148,85 MB bei 1.000 Chunks im VM-Messlauf ([`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) §1). |
| **3. Search- & Punktabfrage-Latenz** | **p50 = 2,61 ms (1k) – 5,13 ms (10k) / ~6,9 µs Punktabfrage** | In-Memory-Suchlatenz p50 = 2,61 ms bei 1.000 Chunks, 5,13 ms bei 10.000 Chunks ([`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) §1). Punktabfragen: 690,71 ms für 100.000 Lookups (ca. 6,9 µs je Lookup, [`docs/reports/performance_baseline_2026-09-25.md`](docs/reports/performance_baseline_2026-09-25.md) §2.b). |
| **4. LoCoMo / LongMemEval-Score** | **78,6 % Accuracy (LongMemEval_s) / 100,0 % Recall@5 (LoCoMo Fixture)** | 78,6 % Accuracy auf LongMemEval-S (500 Testfälle, [`benchmarks/contextra-bench/baseline_metrics.json`](benchmarks/contextra-bench/baseline_metrics.json)); 100,0 % Recall@5 auf synthetischem LoCoMo-Fixture (1 Gespräch). |
| **5. Löschverhalten in v1.0** | **Tombstone-Löschen** | `contextra_forget` markiert Dokumente als gelöscht (Tombstone); ein Löschnachweis ist für v1.5 geplant. |

### Einordnung im Vergleich zu externen Systemen

Direkte, reproduzierbare Vergleichsmessungen mit standardisierten Datensätzen gegen externe Systeme (z. B. Mem0, Zep, Graphiti, ChromaDB) liegen für das aktuelle Release nicht vor (siehe [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) §4). Contextra ist als eingebetteter, rein lokaler MCP-Coding-Kontext-Server konzipiert und führt Repository-Indexierung, Retrieval und lokale Inferenz im selben Prozess ohne Netzwerk- oder Server-Overhead aus.

---

## Lizenzierung

Contextra ist unter folgender Dual-Lizenz freigegeben:

- **Apache License, Version 2.0** ([`LICENSE-APACHE`](LICENSE-APACHE) oder [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
- **MIT License** ([`LICENSE-MIT`](LICENSE-MIT) oder [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
