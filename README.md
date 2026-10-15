# Contextra

Contextra ist eine air-gap-fähige, kryptografisch beweisbare Memory-Engine für KI-Agenten — als Git-Abhängigkeit einbindbar, ohne externen Serverprozess. Sie vereint Vektor-Einbettungen (HNSW / DiskANN), Volltextsuche (BM25), Wissensgraphen (CSR, Forward-Push PPR, PathRAG), deterministische KV-Cache-Verwaltung (KIVI 2-Bit, Irminsul) und hybride Signal-Fusion in einer modularen Pure-Rust-Bibliothek.

> **Dokumentationsstand (14.10.2026):** Normativ abgestimmt mit der [Disjunktiven Endprodukt-Gesamtspezifikation (EP1–EP6)](docs/IMPLEMENTIERUNGSSPEZIFIKATION_ENDPRODUKTE_DISJUNKTIV.md) und den [Mikrofeingranularen Einzel-Spezifikationen](docs/SPEZIFIKATION_EP1_CONTEXTRA_MEMORY.md).

---

## Produktthese & Alleinstellungsmerkmale

1. **Beweisbarkeit statt Zusage (EP2):** Das Löschen von Daten erzeugt im Modus `ErasureMode::Attested` einen signierten `DeletionProofV4` (Ed25519, BSI TR-02102 konform), der das physische Tilgen auf allen 7 Systemspeicherebenen testiert. Externe Verifikation ist über das autonome Crate `contextra-verify` mathematisch unabhängig ohne Datenbankzugriff möglich.
2. **Air-Gap-Fähigkeit & Ein-Prozess-Garantie:** Läuft vollständig ohne Netzwerk, ohne externe API-Keys und ohne Telemetrie im selben Prozess wie die Anwendung.
3. **Hardware-angepasste Inferenz (EP4):**
   - **Ollama / Candle (`park/EP4/v1`):** Inferenz-Treiber für lokale LLMs auf Entwickler-Hardware oder als Opt-in für direkten KV-Tensoren-Zugriff.
4. **Deterministische Performance (P28):** Injektion aller Zeit-, Zufalls- und ID-Quellen über Ports (`contextra-ports`), konsequente `f64::total_cmp`-Gleitkomma-Sortierung und Zero-Panic-Doktrin (P2: 0 `unwrap()` im Produktionspfad).
5. **Zero-Cost-Plugin-Doktrin:** Wer den Löschnachweis nicht benötigt (`ErasureMode::Off`), zahlt 0 Byte Binary-Overhead und 0 ns Lese-Overhead.

---

## Die 6 Disjunktiven Endprodukte (EP1 bis EP6)

Contextra ist modular in sechs klar abgegrenzte Produktkomponenten aufgeteilt:

| Komponente | Bezeichnung | Status | Kernversprechen & Disjunktive Crates |
|---|---|---|---|
| **EP1** | **Contextra Memory** | 🟢 **Release 1.0** | Lokaler Hybrid-Speicher, SSI-Transaktionen, Mandanten-Scoping, Purge v2 (17 Kern-Crates: `contextra`, `contextra-engine`, `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-mvcc`, `contextra-crypto`, `contextra-ports`, `contextra-types`, `contextra-sys`, `contextra-wire`, `contextra-simd`, `contextra-durable-fs`, `contextra-rank`, `contextra-checkpoint`, `contextra-db`, `contextra-mcp`). |
| **EP2** | **Contextra Proof** | 🟡 **Geparkt** (`park/EP2/v1`) | 7-Layer-Löschnachweis (BSI TR-02102, Ed25519), Standalone-Verifier, Art. 30 / AVV-Generator (`contextra-crypto`, Tag `park/EP2/v1`). |
| **EP3** | **Contextra Retrieval+** | 🟡 **Geparkt** (`park/EP3/v1`) | Relationale Wissensgraphen (CSR), OpenIE Extraktion, Budgeted k-Path Diffusion (`PathRAG`), Conformal Fusion Calibration (`contextra-rank`, Tag `park/EP3/v1`). |
| **EP4** | **Contextra Inference** | 🟡 **Geparkt** (`park/EP4/v1`) | Irminsul Radix-Trie Prefix-Reuse, KIVI 2-Bit asymmetrische Quantisierung, SnapKV Attention Eviction (`contextra-checkpoint`, Tag `park/EP4/v1`). |
| **EP5** | **Contextra Cognition** | 🟡 **Geparkt** (`park/EP5/v1`) | Adaptive Kognition, Sketched LinUCB Bandits ($\mathcal{O}(k^2)$ Achlioptas-Projektion), PID Latency Controller, Hyperedge Spectral SVD (Tag `park/EP5/v1`). |
| **EP6** | **Contextra Guard** | 🟡 **Geparkt** (`park/EP6/v1`) | Egress-Firewall (Luhn/IBAN DLP), Fail-Closed MCP Gateway, Hash-Chained Audit Trace, WASM Sandbox Isolation (`contextra-mcp`, Tag `park/EP6/v1`). |

---

## Entwickler- & Agenten-Workflow (`cargo xtask`)

Contextra verwendet `cargo xtask` als **einzige Quelle der Wahrheit** für Analyse, Diagnose, Refactoring und Governance.

### Schnelleinstieg für Entwickler & Agenten:

```bash
# 1. Systemdiagnose & Status-Dashboard
cargo xtask repo-doctor

# 2. Workspace-weites Lücken-Audit (undokumentierte & unverdrahtete Symbole)
cargo xtask find-unwired-gaps

# 3. Kontext eines spezifischen Crates abrufen
cargo xtask crate-context <crate-name>          # z.B. contextra-store, contextra-crypto

# 4. Doktrin- und Zero-Panic-Prüfung für ein Crate
cargo xtask doctrine-scan --crate <crate-name>

# 5. Unverdrahtete Traits & isolierte Ports identifizieren
cargo xtask unwired-ports --only-unwired

# 6. Ring-Architektur, Unsafe-Inseln und DAG-Integrität prüfen
cargo xtask check-ring-capabilities-consistency
cargo xtask check-unsafe-islands
cargo xtask dependency-graph-audit

# 7. Vor Änderungen: Prüfen ob Ziel-Symbol existiert
cargo xtask symbol-exists <modul::Pfad::Symbol>

# 8. Vollständiger Verifikations-Lauf vor Commits
cargo xtask verify --tier kernel
```

Ausführliche Anweisungen und Richtlinien für LLM-Agenten finden sich in [`AGENTS.md`](AGENTS.md).  
Das Entwickler-Handbuch für lokale Hardware-Workflows befindet sich in [`docs/SOLO_DEVELOPER_GUIDE.md`](docs/SOLO_DEVELOPER_GUIDE.md).

---

## Installation & Einbindung

Da Contextra als eingebettete Engine konzipiert ist, binde `contextra` als Git-Abhängigkeit in deine `Cargo.toml` ein:

```toml
[dependencies]
contextra = { git = "https://github.com/tfufuz1/contextra" }
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

---

## Quick Start (Rust)

```rust
use contextra::{Contextra, ContextraConfig};

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
        Some(serde_json::json!({
            "topic": "programming",
            "text": "Rust is a systems programming language with zero-cost abstractions"
        })),
    )
    .await?;

    db.insert(
        "python-lang",
        &[0.9, 0.1, 0.0, 0.0],
        Some(serde_json::json!({
            "topic": "programming",
            "text": "Python is great for AI and rapid prototyping"
        })),
    )
    .await?;

    // 3. Semantische Suche nach den Top 2 treffendsten Dokumenten
    let query = [0.95, 0.05, 0.0, 0.0];
    let results = db.search(&query, 2).await?;

    for result in &results {
        println!("Hit: {} (Score: {:.4})", result.id, result.score);
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

## Verbindliche Dokumentation & Spezifikationen

- **Agenten-Betriebsanleitung & Harness:** [`AGENTS.md`](AGENTS.md)
- **Disjunktive Endprodukt-Gesamtspezifikation:** [`docs/IMPLEMENTIERUNGSSPEZIFIKATION_ENDPRODUKTE_DISJUNKTIV.md`](docs/IMPLEMENTIERUNGSSPEZIFIKATION_ENDPRODUKTE_DISJUNKTIV.md)
- **EP1 Memory Feinspezifikation:** [`docs/SPEZIFIKATION_EP1_CONTEXTRA_MEMORY.md`](docs/SPEZIFIKATION_EP1_CONTEXTRA_MEMORY.md)
- **EP2 Proof Feinspezifikation:** [`docs/SPEZIFIKATION_EP2_CONTEXTRA_PROOF.md`](docs/SPEZIFIKATION_EP2_CONTEXTRA_PROOF.md)
- **EP3 Retrieval+ Feinspezifikation:** [`docs/SPEZIFIKATION_EP3_CONTEXTRA_RETRIEVAL_PLUS.md`](docs/SPEZIFIKATION_EP3_CONTEXTRA_RETRIEVAL_PLUS.md)
- **EP4 Inference Feinspezifikation:** [`docs/SPEZIFIKATION_EP4_CONTEXTRA_INFERENCE.md`](docs/SPEZIFIKATION_EP4_CONTEXTRA_INFERENCE.md)
- **EP5 Cognition Feinspezifikation:** [`docs/SPEZIFIKATION_EP5_CONTEXTRA_COGNITION.md`](docs/SPEZIFIKATION_EP5_CONTEXTRA_COGNITION.md)
- **EP6 Guard Feinspezifikation:** [`docs/SPEZIFIKATION_EP6_CONTEXTRA_GUARD.md`](docs/SPEZIFIKATION_EP6_CONTEXTRA_GUARD.md)
- **Solo-Entwickler Hardware-Handbuch:** [`docs/SOLO_DEVELOPER_GUIDE.md`](docs/SOLO_DEVELOPER_GUIDE.md)
- **Domänentypen-Register:** [`docs/TYPE_REGISTRY.md`](docs/TYPE_REGISTRY.md)
- **Capabilities & Ring-Definition:** [`capabilities.toml`](capabilities.toml)

---

## Lizenzierung

Contextra ist unter folgender Dual-Lizenz freigegeben:
- **Core Engine (Ring `fast`):** MIT oder Apache-2.0
- **Sovereign Extensions (Ring `sovereign`):** BSL 1.1 / Open-Source konforme Krypto-Lizenzen
