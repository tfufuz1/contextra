# Contextra

Contextra ist eine souveräne, vollständig lokal betriebene Gedächtnis- und Ausführungsschicht für KI-Agenten, geschrieben in Pure Rust. Sie vereint Vektor-Einbettungen (HNSW / DiskANN), Volltextsuche (BM25 / BM25F), Graph-Traversierungen (Forward-Push PPR, Leiden-Community-Detection) und Contextual-Bandit-Routing in einem einzigen, latenzarmen lokalen Speicher- und Inferenzkern.

> **Dokumentationsstand:** Normativ abgestimmt mit der **[Finalen Produktspezifikation (Synthese)](docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md)**.

---

## Produktthese & Alleinstellungsmerkmale

1. **Beweisbarkeit statt Zusage:** Löschung (`DeletionProof`), Datenzugriff und Agentenhandlungen sind kryptographisch nachprüfbar und ohne Contextra-Zugriff extern verifizierbar.
2. **Air-Gap-Fähigkeit:** Läuft vollständig ohne Netzwerk, ohne externe API-Keys und ohne Telemetrie.
3. **Deterministische Performance:** Pure Rust, kein GC, Kaltstart < 50 ms, Zero-Panic-Garantie im Produktionspfad.
4. **Ein Kern, zwei Märkte:** Ein einziger verifizierter Kern bedient sowohl den performance-getriebenen Personal-/Agent-AI-Nutzer als auch den nachweispflichtigen regulierten Betrieb (über Feature-Kompilation entkoppelt):
   - **Ring `fast`:** MIT/Apache-2.0, quelloffen. Vektor+Text+Graph-Retrieval, Bandit-Routing, lokale Inferenz.
   - **Ring `sovereign`:** Quelloffener Krypto-Code (Löschbeweis, Privacy-Gateway, Zero-Net-Traffic).
   - **Ring `compliance`:** Closed-Source, kommerziell (Lizenzschicht, Mandanten-Scoping, BSI/GDPR-Reporting).

---

## Capability Overview

| Feature | Ring | Subsystem / Crate | Status | Anmerkung |
|---|---|---|---|---|
| **Vector Search (HNSW / DiskANN)** | `fast` | `contextra-vector` | 🟢 Produktiv | HNSW, DiskANN, SQ8 / RaBitQ Quantisierung. |
| **Full-Text Search (BM25 / BM25F)** | `fast` | `contextra-text` | 🟢 Produktiv | BM25/BM25F mit Block-Max WAND & deutscher Morphologie. |
| **Knowledge Graph & PPR** | `fast` | `contextra-graph` | 🟢 Produktiv | CSR-Graph, Forward-Push PPR, Leiden-Community-Detection, Hyperkanten. |
| **4-Signal-Fusion & Kalibrierung** | `fast` | `contextra-rank` | 🟢 Produktiv | Multi-Signal-Fusion (RRF), Isotonic- / Platt-Kalibrierung & Drift. |
| **Contextual-Bandit-Routing** | `fast` | `contextra-adapt` | 🟢 Produktiv | LinUCB, Sherman-Morrison, FC-TS, Lyapunov-Drift-Regler & PID. |
| **LSM Storage Engine & WAL** | `fast` | `contextra-store` | 🟢 Produktiv | LSM-Tree, WAL (Group-Commit, HMAC-Kette), MVCC-Pinning. |
| **KV-Cache v2 & Zero-Copy IPC** | `fast` | `contextra-kvcache` | 🟢 Produktiv | Prefix-Radix-Baum, Tiering, AEAD, Segmentdateien. |
| **Local Inference Backends** | `fast` | `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx` | 🟢 Produktiv | GGUF/Candle, Ollama HTTP, ONNX/ort Reranker. |
| **Model Context Protocol** | `fast` | `contextra-mcp` | 🟢 Produktiv | JSON-RPC 2.0 stdio MCP Server für AI Agenten. |
| **Python Bindings (`contextra-py`)** | `fast` | `contextra-py` | 🟢 Produktiv | PyO3 FFI Bindings für Python, GIL-safe, Zero-Copy NumPy. |
| **WASM-Sandbox** | `fast` | `contextra-sandbox` | 🟢 Produktiv | Wasmtime Isolation, Fuel / Wall-Clock Budgets. |
| **Kryptographischer Löschbeweis** | `sovereign` | `contextra-crypto` | 🟢 Produktiv | `DeletionProof`, AEAD-Schlüsselhierarchie, Anti-Tamper. |
| **Privacy Gateway** | `sovereign` | `contextra-privacy` | 🟢 Produktiv | Egress-Gateway, PII-Vault, DLP, Prompt-Injection-Filter. |
| **Lizenz- & Compliance-Schicht** | `compliance` | `contextra-license` | 🔒 Closed | Lizenzdurchsetzung, BSI TR-02102-1 Audit, Mandanten-Scoping. | <!-- crate-ref-ignore -->

---

## Installation & Einbindung

Füge `contextra-db` oder `contextra` zu deiner `Cargo.toml` hinzu:

```toml
[dependencies]
contextra-db = "0.1"
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

Die folgenden Leistungskennzahlen stammen aus reproduzierbaren Läufen der `criterion`-Benchmark-Suite (`benchmarks/contextra-bench`) sowie systematischen Messberichten (siehe [`docs/reports/performance_baseline_2026-09-25.md`](docs/reports/performance_baseline_2026-09-25.md) und [`docs/BENCHMARKS_DELETION_PROOF.md`](docs/BENCHMARKS_DELETION_PROOF.md)):

| Kennzahl | Contextra (Sovereign Local Engine) | Beschreibung & Rahmenbedingungen |
|---|---|---|
| **1. Kaltstart** | **< 50 ms** | Vollständige Datenbank- & Index-Initialisierung (LSM-Storage Recovery & Re-Open in 1,12 ms bis 14,85 ms). |
| **2. Footprint** | **~12–25 MB** | Arbeitsspeicherbedarf (RAM Base Footprint) ohne externe Modellgewichtungen im Air-Gap Betrieb. |
| **3. p99-Latenz** | **< 15 ms** | End-to-End p99-Latenz für 4-Signal-Hybrid-Retrieval (RRF Vektor + BM25 + Graph PPR + Temporal); Punktabfragen p50 ~0,69 ms. |
| **4. LoCoMo/LongMemEval-Score** | **78,6 % Accuracy** (LongMemEval_s) / **100,0 % Recall@5** (LoCoMo Fixture) | Evaluierung im `RetrievalStrategy::Hybrid`-Modus auf LongMemEval und LoCoMo Multi-Session Gesprächshistorien. |
| **5. Löschbeweis-Zeit** | **2,28 µs** (O(1) Verifikation) / **2,49 µs** (1 Key Erzeugung) | Rechnerische Latenz für kryptographische DSGVO Art. 17 `DeletionProof`-Erzeugung und -Verifikation (Blake3/HMAC-SHA256). |

### Vergleich mit öffentlich publizierten Referenzwerten

Im Vergleich zu cloud-basierten oder netzwerkgebundenen Open-Source-Memory-Systemen zeichnet sich Contextra durch rein lokale Ausführung ohne Netzwerk-Overhead aus:

- **[Mem0 Benchmark](https://github.com/mem0ai/mem0)**: Mem0 verzeichnet Genauigkeitswerte von ca. 66,9 % bis 83,0 % auf LoCoMo/Conversational-Retrieval mit durchschnittlichen Abfragelatenzen von ca. 150–300 ms (bedingt durch externe Vektordatenbanken und Cloud-LLM-Netzwerkaufrufe). Contextra erreicht lokal vergleichbare bis höhere Retrieval-Genauigkeiten bei einer um eine Größenordnung geringeren p99-Abfragelatenz (< 15 ms).
- **[Zep Evaluation](https://github.com/getzep/zep)**: Zep erzielt auf LongMemEval und Conversational-Benchmarks Genauigkeiten von ca. 82,0 % bis 84,5 % unter Nutzung graphbasierter Hybrid-Suchen mit p95-Latenzen von ca. 100–250 ms. Contextra erreicht mit `RetrievalStrategy::Hybrid` 78,6 % Genauigkeit auf LongMemEval_s bei deterministischer, lokaler p99-Latenz von < 15 ms.
- **[Graphiti Temporal Knowledge Graph](https://github.com/getzep/graphiti)**: Graphiti bietet zeitliche Graph-Traversierungen mit Kanten-Recall-Werten von ca. 80–85 %, erfordert jedoch durch asynchrone LLM-Graphkonstruktion Extraktionszeiten im dreistelligen Millisekundenbereich. Contextra führt zeitliche Graph-Traversierungen (Forward-Push PPR) in Sub-Millisekunden rein lokal aus.
- **Kryptographischer Löschbeweis (`DeletionProof`)**: Weder Mem0, Zep noch Graphiti bieten mathematisch verifizierbare kryptographische Löschbeweise. Contextra stellt mit einer O(1)-Verifikationszeit von 2,28 µs eine echte Nachweisbarkeit für regulierte Umgebungen bereit.

---

## Lizenzierung

Contextra ist unter folgender Dual-Lizenz freigegeben:

- **Apache License, Version 2.0** ([`LICENSE-APACHE`](LICENSE-APACHE) oder [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
- **MIT License** ([`LICENSE-MIT`](LICENSE-MIT) oder [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
