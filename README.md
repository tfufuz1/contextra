# Contextra

Contextra ist eine souveräne, vollständig lokal betriebene Gedächtnisschicht für KI-Agenten (§1, REQ-ARCH-001): Schlüssel-Wert-Speicher, Vektorindex (HNSW/SQ8; §9.0, REQ-IDX-001), Volltextsuche (BM25/BM25F; §10.1, REQ-IDX-005) und Wissensgraph (§11.1, REQ-IDX-008) bilden mit Metadaten, Audit und Kryptografie eine einzige, atomar konsistente Wahrheit (ein WAL) je Mandant getrennt (§1, §9.2, REQ-DUR-011, REQ-TEN-001). Zugänglich als reine Rust-Bibliothek und als MCP-Server über stdio (§1, §21.1, REQ-MCP-001).

> **Dokumentationsstand:** Normativ abgestimmt mit der **Spezifikation 1.2** (`docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`, §0–§34).

---

## Produktvision & Missionen M1–M5 (§2.0)

Contextra richtet sich an den Fünf-Missions-Zielrahmen (§2.0):

- **M1 — Persistenter Prefix-Cache (Zielzustand / Opt-in 🟡, §13.1):** Ein persistenter, mandantenisolierter Prefix-Radix-Tree für Inferenz-KV-Blöcke. *Hinweis zum Ist-Zustand:* Der In-Memory-Prefix-Store ist vorhanden; die persistenten Segmentdateien und das B1–B3 Tiering (§13.1) befinden sich im Zielzustand und werden erst nach Beleg behauptet.
- **M2 — Heterogene Hybridsuche (Ist-Zustand 🟢, §7.1, REQ-RET-001):** 4-Signal-Hybridsuche über Vektoren (HNSW), Volltext (BM25/BM25F), Wissensgraph (CSR + PPR) und Metadaten-Filter, fusioniert per Reciprocal Rank Fusion (RRF; §12.1).
- **M3 — Transaktionale Speicher-Engine (Ist-Zustand 🟢, §7.1, REQ-DUR-001):** LSM-Tree Storage Engine mit MPSC Group-Commit WAL, HMAC-Integritätskette und MVCC Snapshot Isolation (§8.1, REQ-MVCC-001).
- **M4 — Verifizierbarer Löschnachweis (Ist-Zustand 🟢, §20.1, REQ-DEL-006):** Kryptographisch belegbares Crypto-Shredding und `DeletionProof` (Formatversion 4; §20.4) ohne unhaltbare Pauschalversprechen (INV-CLAIMS-1, §4.3).
- **M5 — Sandboxed MCP Integration (Ist-Zustand 🟢, §15.1, §21.1, REQ-MCP-001):** Agent-Tool-Ausführung in Wasmtime-Isolation und stdio-basierter MCP-Server.

---

## Alleinstellungsmerkmale & Löschaussagen (INV-CLAIMS-1, §4.3, §20.12)

1. **Ehrlicher Löschnachweis statt Zusage (INV-CLAIMS-1, §4.3, REQ-DEL-006):**
   Aussagen zur Löschung folgen zwingend der Außenformulierung gemäß §4.3 und §20.12 der Spezifikation:
   > „Nach `forget` bzw. `drop_collection` ist der Scope logisch gelöscht (Tombstone, fsync; §20.12). Ist sein Schlüssel vernichtet, ist er zusätzlich kryptografisch unlesbar; das weist der Beweis aus. Die physische Bereinigung folgt nach Flush, Purge-Kompaktion und WAL-Bereinigung; erst dann wird ein vollständiger Nachweis für die im Beweis aufgeführten Schichten ausgestellt. Nicht aufgeführte Schichten sind nicht abgedeckt. Für den physischen Datenträger gelten die dokumentierten Rahmenbedingungen (verschlüsseltes Volume, Swap deaktiviert oder verschlüsselt, Backups ohne Keystore).“

   *Unzulässige Formulierungen:* Pauschale Behauptungen wie „100 % gelöscht“, „DSGVO-konform“ oder „garantiert“ sind ausdrücklich unzulässig (INV-CLAIMS-1, §4.3, §20.12).

2. **Embedded & Single-Node-Garantie (REQ-ARCH-001):**
   Läuft vollständig ohne Netzwerk, ohne externe API-Keys, ohne separaten Serverprozess und ohne Telemetrie im selben Prozess wie die Anwendung des Nutzers (`cargo add contextra`). Single-Node, kein Sharding, keine Replikation (§1, §4.4).

3. **Pure Rust Inferenz (`contextra-infer-candle`, Rand `[R]`, stable; §14.1):**
   Lokale Inferenz ohne C++ / CUDA FFI-Abhängigkeiten oder extern laufende Dämonen (§6.0).

4. **Deterministischer Kern (INV-RING0-DETERMINISM-1, P28, REQ-ARCH-015):**
   Gleicher Anfangszustand und gleiche WAL-Folge liefern bit-identischen Zustand. Nichtdeterministische Quellen (LLM, Embeddings, Uhr) werden über injizierte Ports aufgezeichnet (P28). Zero-Panic-Garantie im Produktionspfad (P7; §27.6).

---

## Capability Overview & Reifegrad-Stand (§6.0, `capabilities.toml`)

Capabilities, Normativitätsklassen (`Kern [K]`, `Rand [R]`, `Experimentell [E]`) und Reifegrade (`stable`, `experimental`) richten sich streng nach `capabilities.toml` und §6.0 der Spezifikation 1.2:

| Feature / Crate | Klasse | Ring | Status / Maturity | Spezifikationsbezug |
|---|---|---|---|---|
| **`contextra`** | Kern `[K]` | Ring 4 | `stable` | Haupt-Library Facade (§22.1, REQ-FAC-001) |
| **`contextra-store`** | Kern `[K]` | Ring 1 | `stable` | LSM-Tree Storage, WAL, Compaction (§7.1, REQ-DUR-001) |
| **`contextra-vector`** | Kern `[K]` | Ring 0 | `stable` | Vektorsuche (HNSW, DiskANN, SQ8) (§9.0, REQ-IDX-001) |
| **`contextra-text`** | Kern `[K]` | Ring 0 | `stable` | Volltextsuche (BM25/BM25F, WAND) (§10.1, REQ-IDX-005) |
| **`contextra-graph`** | Kern `[K]` | Ring 0 | `stable` | CSR Wissensgraph & PPR (§11.1, REQ-IDX-008) |
| **`contextra-rank`** | Kern `[K]` | Ring 0 | `stable` | Signal-Fusion (RRF) & Kalibrierung (§12.1, REQ-RET-001) |
| **`contextra-crypto`** | Kern `[K]` | Ring 0 | `stable` | AES-256-GCM-SIV, HMAC, DeletionProof v4 (§20.1, REQ-KEY-001) |
| **`contextra-kvcache`** | Kern `[K]` | Ring 1 | `stable` | Tiered KV-Cache & Segment Storage (§13.1, REQ-MEM-001) |
| **`contextra-mcp`** | Kern `[K]` | Ring 4 | `stable` | MCP Server über stdio (§21.1, REQ-MCP-001) |
| **`contextra-cognition`** | Kern `[K]` | Ring 3 | `stable` | Gedächtniskonsolidierung & Maintenance (§17.1, REQ-COG-001) |
| **`contextra-checkpoint`** | Kern `[K]` | Ring 1 | `stable` | Persistent Checkpoint Engine (§16.1, REQ-DUR-010) |
| **`contextra-agent`** | Kern `[K]` | Ring 3 | `stable` | Agent Workflow & Audit Loop (§16.2, REQ-AUD-001) |
| **`contextra-infer-candle`** | Rand `[R]` | Ring 2 | `stable` | Pure Rust GGUF/Candle Inferenz (§14.1, REQ-INF-001) |
| **`contextra-infer-ollama`** | Rand `[R]` | Ring 2 | `stable` | Remote Ollama Provider Client (§14.2, REQ-INF-002) |
| **`contextra-infer-onnx`** | Rand `[R]` | Ring 2 | `experimental` | ONNX Embedding & Reranker (§14.3, REQ-INF-003) |
| **`contextra-sandbox`** | Rand `[R]` | Ring 2 | `stable` | Wasmtime Isolation Sandbox (§15.1, REQ-SBX-001) |
| **`contextra-privacy`** | Rand `[R]` | Ring 3 | `stable` | Egress-Vault & PII-Filter (§19.1, REQ-PRV-001) |
| **`contextra-adapt`** | Experimentell `[E]` | Ring 0 | `experimental` | Adaptive Regler (PID / LinUCB) (§18.1, REQ-EXP-001) |
| **`contextra-router`** | Experimentell `[E]` | Ring 3 | `stable` | Cascade & Drift Router Engine (§18.2, REQ-EXP-002) |
| **`contextra-audit-export`** | Experimentell `[E]` | Ring 0 | `experimental` | DSGVO Art. 30 Audit-Export (§20.10, REQ-EXP-004) |
| **`contextra-avv-generator`** | Experimentell `[E]` | Ring 0 | `experimental` | Art. 28 AVV-Template-Generator (§20.11, REQ-EXP-005) |
| **`contextra-license`** | Experimentell `[E]` | Ring 4 | `experimental` | Lizenzschicht & Ring-Gating (§23.5, REQ-EXP-003) |

---

## Installation & Einbindung (§22.1)

Füge `contextra` zu deiner `Cargo.toml` hinzu (§22.1):

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
    // 1. Datenbankinstanz konfigurieren und öffnen (§22.1)
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config("./example_data", config).await?;

    // 2. Dokumente mit Vektoreinbettungen und Metadaten einfügen
    db.insert(
        "rust-lang",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"topic": "programming", "text": "Rust systems programming"})),
    )
    .await?;

    // 3. Semantische Suche nach den Top 2 Treffern
    let query = [0.95, 0.05, 0.0, 0.0];
    let results = db.search(&query, 2).await?;

    for result in &results {
        println!("{} (score: {:.4})", result.id, result.score);
    }

    // 4. Ordnungsgemäß schließen
    db.close().await?;
    let _ = tokio::fs::remove_dir_all("./example_data").await;

    Ok(())
}
```

---

## Dokumentation & Spezifikationen

- **Normative Spezifikation 1.2:** [`docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`](docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md)
- **Systemarchitektur & Ring-Modell:** [`ARCHITECTURE.md`](ARCHITECTURE.md)
- **Sicherheitsrichtlinie:** [`SECURITY.md`](SECURITY.md)
- **Beitragswesen:** [`CONTRIBUTING.md`](CONTRIBUTING.md)

---

## Lizenzierung

Contextra ist unter folgender Dual-Lizenz freigegeben (§0.2):

- **Apache License, Version 2.0** ([`LICENSE-APACHE`](LICENSE-APACHE))
- **MIT License** ([`LICENSE-MIT`](LICENSE-MIT))
