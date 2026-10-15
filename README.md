# Contextra

Contextra ist eine air-gap-fähige, kryptografisch beweisbare Memory-Engine für KI-Agenten — als Git-Abhängigkeit einbindbar, ohne externen Serverprozess. Sie vereint Vektor-Einbettungen (HNSW), Volltextsuche (BM25), deterministische Speichermodelle und hybride Signal-Fusion in einer modularen Pure-Rust-Bibliothek.

> **Dokumentationsstand (15.10.2026):** Normativ abgestimmt mit der [Disjunktiven Endprodukt-Gesamtspezifikation (EP1–EP6)](docs/IMPLEMENTIERUNGSSPEZIFIKATION_ENDPRODUKTE_DISJUNKTIV.md) und der [Mikrofeingranularen EP1 Spezifikation](docs/SPEZIFIKATION_EP1_CONTEXTRA_MEMORY.md).

---

## Produktthese & Alleinstellungsmerkmale

1. **Beweisbarkeit statt Zusage (EP2):** Das Löschen von Daten erzeugt im Modus `ErasureMode::Attested` einen signierten `DeletionProofV4` (Ed25519, BSI TR-02102 konform), der das physische Tilgen auf allen Systemspeicherebenen testiert.
2. **Air-Gap-Fähigkeit & Ein-Prozess-Garantie:** Läuft vollständig ohne Netzwerk, ohne externe API-Keys und ohne Telemetrie im selben Prozess wie die Anwendung.
3. **Flexible Inferenz-Portschnittstellen:** Anbindung von Inferenz- und Embedding-Anbietern über entkoppelte Trait-Schnittstellen (`EmbeddingProvider`). Externe Inferenz-Module sind im Tag `park/EP4/v1` gesichert.
4. **Deterministische Performance (P28):** Injektion aller Zeit-, Zufalls- und ID-Quellen über Ports (`contextra-ports`), konsequente `f64::total_cmp`-Gleitkomma-Sortierung und Zero-Panic-Doktrin (P2: 0 `unwrap()` im Produktionspfad).
5. **Zero-Cost-Plugin-Doktrin:** Wer den Löschnachweis nicht benötigt (`ErasureMode::Off`), zahlt 0 Byte Binary-Overhead und 0 ns Lese-Overhead.

---

## Die 6 Disjunktiven Endprodukte (EP1 bis EP6)

Contextra ist modular in sechs klar abgegrenzte Produktkomponenten aufgeteilt:

| Komponente | Bezeichnung | Status | Kernversprechen & Disjunktive Crates / Tags |
|---|---|---|---|
| **EP1** | **Contextra Memory** | 🟢 **Release 1.0** | Lokaler Hybrid-Speicher, SSI-Transaktionen, Mandanten-Scoping, Purge v2 (17 Kern-Crates in Ring 0–4: `contextra`, `contextra-engine`, `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-mvcc`, `contextra-db`, `contextra-mcp`, etc.). |
| **EP2** | **Contextra Proof** | 📦 **Geparkt** | 7-Layer-Löschnachweis (BSI TR-02102, Ed25519), Standalone-Verifier, Art. 30 / AVV-Template (Code gesichert in Tag `park/EP2/v1`). |
| **EP3** | **Contextra Retrieval+** | 📦 **Geparkt** | Relationale Wissensgraphen (CSR), OpenIE Extraktion, Budgeted k-Path Diffusion (`PathRAG`), Conformal Fusion Calibration (Code gesichert in Tag `park/EP3/v1`). |
| **EP4** | **Contextra Inference** | 📦 **Geparkt** | Irminsul Radix-Trie Prefix-Reuse, KIVI 2-Bit Quantisierung, SnapKV Attention Eviction (Code gesichert in Tag `park/EP4/v1`). |
| **EP5** | **Contextra Cognition** | 📦 **Geparkt** | Adaptive Kognition, Sketched LinUCB Bandits, PID Latency Controller, Hyperedge Spectral SVD (Code gesichert in Tag `park/EP5/v1`). |
| **EP6** | **Contextra Guard** | 📦 **Geparkt** | Egress-Firewall (Luhn/IBAN DLP), Fail-Closed Gateway, Hash-Chained Audit Trace, WASM Sandbox Isolation (Code gesichert in Tag `park/EP6/v1`). |

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
