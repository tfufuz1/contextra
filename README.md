# Contextra

Contextra ist eine air-gap-fähige, kryptografisch beweisbare Memory-Engine für KI-Agenten — ein `cargo add`, kein Server. Sie vereint Vektor-Einbettungen (HNSW / DiskANN), Volltextsuche (BM25; BM25F-Spezifikation), Graph-Traversierungen (Forward-Push PPR, Leiden-Community-Detection) und hybride Signal-Fusion in einer eingebetteten Pure Rust Bibliothek.

> **Dokumentationsstand:** Normativ abgestimmt mit der **[Systemspezifikation v15 (30.09.2026)](docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md)**.

---

## Produktthese & Alleinstellungsmerkmale

1. **Beweisbarkeit statt Zusage:** Löschung (`DeletionProof`), Datenzugriff und Agentenhandlungen sind kryptographisch nachprüfbar und ohne Contextra-Zugriff extern verifizierbar.
2. **Air-Gap-Fähigkeit & Ein-Prozess-Garantie:** Läuft vollständig ohne Netzwerk, ohne externe API-Keys, ohne separaten Serverprozess und ohne Telemetrie im selben Prozess wie die Anwendung des Nutzers (`cargo add contextra`).
3. **Pure Rust Inferenz (Candle):** Lokale GGUF-Inferenz ohne C++ / CUDA FFI-Abhängigkeiten oder extern laufende Dämonen.
4. **Deterministische Performance:** Pure Rust, kein GC, In-Memory-Search-Latenz p50 = 2,61 ms bis 5,13 ms (1k–10k Chunks, siehe [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) §1), angestrebtes Zero-Panic-Ziel im Produktionspfad (P7, schrittweise Reduzierung verbleibender Panic-Stellen über CI-Ratchet-Mechanismus) und injizierter Determinismus (P28).
5. **Drei abgestufte Feature-Ringe:**
   - **Ring `fast`:** MIT/Apache-2.0, quelloffen. Vektor+Text+Graph-Retrieval, Candle-Inferenz, Bandit-Routing.
   - **Ring `sovereign`:** Quelloffener Krypto-Code (Löschbeweis, Privacy-Gateway, Zero-Net-Traffic).
   - **Ring `compliance`:** Kommerziell (Lizenzschicht, Mandanten-Scoping, BSI/DSGVO-Reporting).

---

## Capability Overview

| Feature | Ring | Subsystem / Crate | Status | Anmerkung |
|---|---|---|---|---|
| **Vector Search (HNSW / DiskANN)** | `fast` | `contextra-vector` | 🟢 Produktiv | HNSW, DiskANN, SQ8 / RaBitQ Quantisierung. |
| **Full-Text Search (BM25 / BM25F)** | `fast` | `contextra-text` | 🟢 Produktiv / 🟡 Geplant | BM25 produktiv (Block-Max WAND & deutsche Morphologie); BM25F spezifiziert, Implementierung ausstehend. |
| **Knowledge Graph & PPR** | `fast` | `contextra-graph` | 🟢 Produktiv | CSR-Graph, Forward-Push PPR, Leiden-Community-Detection, Hyperkanten. |
| **4-Signal-Fusion & Kalibrierung** | `fast` | `contextra-rank` | 🟢 Produktiv | Multi-Signal-Fusion (RRF), Isotonic- / Platt-Kalibrierung & Drift. |
| **Contextual-Bandit-Routing** | `fast` | `contextra-adapt` | 🟡 In Arbeit | Kernmechanismen (LinUCB, Sherman-Morrison, FC-TS) implementiert, Produktionsverdrahtung in Arbeit. |
| **LSM Storage Engine & WAL** | `fast` | `contextra-store` | 🟢 Produktiv | LSM-Tree, WAL (Group-Commit, HMAC-Kette), MVCC-Pinning. |
| **KV-Cache v2 & Zero-Copy IPC** | `fast` | `contextra-kvcache` | 🟡 In Arbeit | Kernmechanismen (Prefix-Radix-Baum, Tiering, AEAD) implementiert, Produktionsverdrahtung in Arbeit. |
| **Local Inference Backend (Candle)** | `fast` | `contextra-infer-candle` | 🟢 Produktiv | Pure Rust GGUF/Candle als Standard-Inferenzbackend. |
| **Opt-in Inference Backends** | `fast` | `contextra-infer-ollama`, `contextra-infer-onnx` | 🟢 Produktiv | Ollama HTTP & ONNX/ort Reranker (explizites Opt-in). |
| **Model Context Protocol** | `fast` | `contextra-mcp` | 🟢 Produktiv | JSON-RPC 2.0 stdio MCP Server für AI Agenten (`contextra` ohne Ollama). |
| **Python Bindings (`contextra-py`)** | Opt-in | `contextra-py` | 🟢 Produktiv | FFI-Bindings für Python (aus default-members entfernt). |
| **WASM-Sandbox** | Opt-in | `contextra-sandbox` | 🟢 Produktiv | Wasmtime Isolation (aus default-members entfernt). |
| **Kryptographischer Löschbeweis** | `sovereign` | `contextra-crypto` | 🟢 Produktiv | `DeletionProof` (7 Ebenen), AEAD-Schlüsselhierarchie, Anti-Tamper. |
| **Privacy Gateway** | `sovereign` | `contextra-privacy` | 🟢 Produktiv | Egress-Gateway, PII-Vault, DLP, Prompt-Injection-Filter. |
| **AVV Generator (Art. 28 DSGVO)** | `sovereign` | `contextra-avv-generator` | 🟢 Produktiv | AVV-Vertragstemplate-Generator mit technischen Garantien. |
| **Audit- & Art.-30-Export** | `sovereign` | `contextra-audit-export` | 🟢 Produktiv | BSI TR-02102-1 Kryptomapping und DSGVO Art. 30 Export. |
| **Memory Consolidation & Cognition** | `fast` | `contextra-cognition` | 🟢 Produktiv | Konsolidierung, LLM-Kompaktierung, MaintenanceScheduler. |
| **Persistent Checkpoint Engine** | `fast` | `contextra-checkpoint` | 🟢 Produktiv | persistent_checkpoint_store, Hardlink-Cloner, Blake3-Manifest. |
| **Persistent Agent Workflows** | `fast` | `contextra-agent` | 🟢 Produktiv | Agenten-Workflow-Engine, Checkpoint/Execute/Audit-Loop. |
| **Lizenz- & Compliance-Schicht** | `compliance` | `contextra-license` | 🔒 Closed | Lizenzdurchsetzung, Ring-Gating, Mandanten-Scoping. | <!-- crate-ref-ignore -->

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

- **Normative Gesamtspezifikation (Synthese):** [`docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`](docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md)
- **Systemarchitektur & Ring-Modell:** [`ARCHITECTURE.md`](ARCHITECTURE.md)
- **Agenten-Betriebsanleitung:** [`AGENTS.md`](AGENTS.md)
- **Sicherheitsrichtlinie:** [`SECURITY.md`](SECURITY.md)
- **Beitragswesen:** [`CONTRIBUTING.md`](CONTRIBUTING.md)

---

## Benchmark-Kennzahlen

Die folgenden Leistungskennzahlen stammen aus reproduzierbaren Läufen der `criterion`-Benchmark-Suite (`benchmarks/contextra-bench`) sowie systematischen Messberichten (siehe [`docs/reports/performance_baseline_2026-09-25.md`](docs/reports/performance_baseline_2026-09-25.md), [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) und [`docs/BENCHMARKS_DELETION_PROOF.md`](docs/BENCHMARKS_DELETION_PROOF.md)):

> **Wichtiger Hinweis zur Latenz-Abgrenzung:** Misst ausschließlich Speicher- und Indexlatenz. Embedding-Inferenz (Ollama, ONNX, Candle) addiert je nach Modell und Hardware 10-500 ms pro Anfrage.

| Kennzahl | Contextra (Sovereign Local Engine) | Beschreibung & Rahmenbedingungen |
|---|---|---|
| **1. Kaltstart / Recovery** | **Offen / Messung in Arbeit** | Recovery-Zeit für 1 Mio. Einträge in [`docs/reports/performance_baseline_2026-09-25.md`](docs/reports/performance_baseline_2026-09-25.md) (§2.e) als nicht gemessen / fehlerhaft dokumentiert (File-Descriptor-Thematik). |
| **2. Footprint** | **~5,5–12,8 MB Base RSS / 148,85 MB Peak** | RAM Base-Footprint ~5,5–12,8 MB bei kleinen Korpora bis 100 Chunks ([`benches/results/scale_rss.csv`](benches/results/scale_rss.csv)); Peak VmRSS 148,85 MB bei 1.000 Chunks im VM-Messlauf ([`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) §1). |
| **3. Search- & Punktabfrage-Latenz** | **p50 = 2,61 ms (1k) – 5,13 ms (10k) / ~6,9 µs Punktabfrage** | In-Memory-Suchlatenz p50 = 2,61 ms bei 1.000 Chunks, 5,13 ms bei 10.000 Chunks ([`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) §1). Punktabfragen: 690,71 ms für 100.000 Lookups (ca. 6,9 µs je Lookup, [`docs/reports/performance_baseline_2026-09-25.md`](docs/reports/performance_baseline_2026-09-25.md) §2.b). 4-Signal-Hybrid p99: Zielwert (< 15 ms), nicht gemessen ([`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) §0). |
| **4. LoCoMo / LongMemEval-Score** | **78,6 % Accuracy (LongMemEval_s) / 100,0 % Recall@5 (LoCoMo Fixture)** | 78,6 % Accuracy auf LongMemEval-S (500 Testfälle, [`benchmarks/contextra-bench/baseline_metrics.json`](benchmarks/contextra-bench/baseline_metrics.json)); 100,0 % Recall@5 auf synthetischem LoCoMo-Fixture (1 Gespräch, [`benchmarks/contextra-bench/tests/fixtures/locomo_fixture.json`](benchmarks/contextra-bench/tests/fixtures/locomo_fixture.json); im vollen LoCoMo-10 mit 1.540 Fällen beträgt der Recall@5 0,45 %, [`benchmarks/contextra-bench/baseline_metrics.json`](benchmarks/contextra-bench/baseline_metrics.json)). |
| **5. Löschbeweis-Zeit** | **2,28 µs (O(1) Verifikation) / 2,49 µs (1 Key Erzeugung)** | Rechnerische Latenz für kryptographische DSGVO Art. 17 `DeletionProof`-Erzeugung und -Verifikation (Blake3/HMAC-SHA256, ohne Disk-Cleanup; [`docs/reports/performance_baseline_2026-09-25.md`](docs/reports/performance_baseline_2026-09-25.md) §3 und [`docs/BENCHMARKS_DELETION_PROOF.md`](docs/BENCHMARKS_DELETION_PROOF.md) §1). |

### Einordnung im Vergleich zu externen Systemen

Direkte, reproduzierbare Vergleichsmessungen mit standardisierten Datensätzen gegen externe Systeme (z. B. Mem0, Zep, Graphiti, ChromaDB) liegen für das aktuelle Release nicht vor (siehe [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) §4). Contextra ist als eingebettete, rein lokale Engine konzipiert und führt Retrieval, Graph-Traversierungen und kryptographische Löschbeweise im selben Prozess ohne Netzwerk- oder Server-Overhead aus.

---

## Lizenzierung

Contextra ist unter folgender Dual-Lizenz freigegeben:

- **Apache License, Version 2.0** ([`LICENSE-APACHE`](LICENSE-APACHE) oder [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
- **MIT License** ([`LICENSE-MIT`](LICENSE-MIT) oder [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
