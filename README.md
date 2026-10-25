# Contextra

Contextra ist die lokale, eingebettete Speicher- und Gedächtnis-Infrastruktur für KI-Anwendungen (§1, REQ-ARCH-001): Schlüssel-Wert-Speicher, Vektorindex (HNSW/SQ8), Volltextsuche (BM25/BM25F) und Wissensgraph bilden mit Metadaten, Audit und Kryptografie eine einzige, atomar konsistente Wahrheit (ein WAL) je Mandant getrennt (§1, §9.2, REQ-DUR-011, REQ-TEN-001). Zugänglich als reine Rust-Bibliothek und als MCP-Server über stdio (§1, §21.1, REQ-MCP-001).

> **Dokumentationsstand:** Normativ abgestimmt mit der **[Normativen Gesamtspezifikation v1.1](docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md)** (§1–§34).

---

## Produktthese & Alleinstellungsmerkmale

1. **Ehrlicher Löschnachweis statt Zusage (INV-CLAIMS-1, REQ-DEL-006):** Löschung nach `forget` bzw. `drop_collection` erfolgt logisch sofort (Tombstone, fsync, S0; §20.12). Ist der Schlüssel vernichtet, ist der Scope zusätzlich kryptografisch unlesbar (SK; §20.2a). Die physische Bereinigung (S1–S3) folgt nach Flush, Purge-Kompaktion und WAL-Scrub; erst dann wird ein signierter `DeletionProof` (Formatversion 4; §20.4) ausgestellt, der nicht belegte Schichten ehrlich ausweist (§4.3).
2. **Embedded & Single-Node-Garantie (REQ-ARCH-001):** Läuft vollständig ohne Netzwerk, ohne externe API-Keys, ohne separaten Serverprozess und ohne Telemetrie im selben Prozess wie die Anwendung des Nutzers (`cargo add contextra`). Single-Node, kein Sharding, keine Replikation (§1, §4.4).
3. **Pure Rust Inferenz (`contextra-infer-candle`, `[K]` / `[R]`):** Lokale Inferenz ohne C++ / CUDA FFI-Abhängigkeiten oder extern laufende Dämonen (§6.0).
4. **Deterministischer Kern (INV-RING0-DETERMINISM-1, REQ-ARCH-015):** Gleicher Anfangszustand und gleiche WAL-Folge liefern bit-identischen Zustand. Nichtdeterministische Quellen (LLM, Embeddings, Uhr) werden als Werte im WAL aufgezeichnet (INV-NONDET-1, REQ-DUR-012). Zero-Panic-Garantie im Produktionspfad (P7; §27.6).
5. **Abgestufte Normativitätsklassen (§6.0):**
   - **Kern `[K]` (stable):** Gilt uneingeschränkt. LSM Storage Engine (`contextra-store`), Vektor index (`contextra-vector`), Volltext (`contextra-text`), Graph (`contextra-graph`), Rank/Fusion (`contextra-rank`), Types (`contextra-types`), Ports (`contextra-ports`), SIMD (`contextra-simd`), Sys (`contextra-sys`), Wire (`contextra-wire`), MVCC (`contextra-mvcc`), Core (`contextra-core`), Crypto (`contextra-crypto`), Checkpoint (`contextra-checkpoint`), KVCache (`contextra-kvcache`), Engine (`contextra-engine`), Cognition (`contextra-cognition`), Agent (`contextra-agent`), MCP (`contextra-mcp`), Facade (`contextra`).
   - **Rand `[R]` (stable / experimental):** Gilt nur bei aktiviertem Feature und darf Kern-Garantien nie schwächen (§6.0). Inferenz-Backends (`contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`), WASM-Sandbox (`contextra-sandbox`), Egress-Vault / Privacy (`contextra-privacy`).
   - **Experimentell `[E]` (experimental):** Nicht normativ; kein Bestandteil von Release-Zusagen (§6.0). Adaptive Regler (`contextra-adapt`), Bandit-Routing (`contextra-router`), Compliance/Export (`contextra-audit-export`, `contextra-avv-generator`), Lizenzschicht (`contextra-license`).

---

## Capability Overview

Capabilities, Normativitätsklassen (`[K]`, `[R]`, `[E]`) und Reifegrade richten sich streng nach `capabilities.toml` und §6.0 der Spezifikation (INV-DOC-SINGLE-SOURCE-1).

| Feature / Crate | Klasse | Ring | Status / Maturity | Spezifikationsbezug |
|---|---|---|---|---|
| **`contextra`** | `[K]` | Ring 4 | `stable` | Haupt-Library Facade (§22.1, REQ-FAC-001) |
| **`contextra-store`** | `[K]` | Ring 1 | `stable` | LSM-Tree Storage, WAL, Compaction (§7.1, REQ-DUR-001) |
| **`contextra-vector`** | `[K]` | Ring 0 | `stable` | Vektorsuche (HNSW, DiskANN, SQ8) (§9.0, REQ-IDX-001) |
| **`contextra-text`** | `[K]` | Ring 0 | `stable` | Volltextsuche (BM25/BM25F, WAND) (§10.1, REQ-IDX-005) |
| **`contextra-graph`** | `[K]` | Ring 0 | `stable` | CSR Wissensgraph & PPR (§11.1, REQ-IDX-008) |
| **`contextra-rank`** | `[K]` | Ring 0 | `stable` | Signal-Fusion (RRF) & Kalibrierung (§12.1, REQ-RET-001) |
| **`contextra-crypto`** | `[K]` | Ring 0 | `stable` | AES-256-GCM-SIV, HMAC, DeletionProof v4 (§20.1, REQ-KEY-001) |
| **`contextra-kvcache`** | `[K]` | Ring 1 | `stable` | Tiered KV-Cache & Segment Storage (§13.1, REQ-MEM-001) |
| **`contextra-mcp`** | `[K]` | Ring 4 | `stable` | MCP Server über stdio (§21.1, REQ-MCP-001) |
| **`contextra-cognition`** | `[K]` | Ring 3 | `stable` | Gedächtniskonsolidierung & Maintenance (§17.1, REQ-COG-001) |
| **`contextra-checkpoint`** | `[K]` | Ring 1 | `stable` | Persistent Checkpoint Engine (§16.1, REQ-DUR-010) |
| **`contextra-agent`** | `[K]` | Ring 3 | `stable` | Agent Workflow & Audit Loop (§16.2, REQ-AUD-001) |
| **`contextra-infer-candle`** | `[R]` | Ring 2 | `stable` | Pure Rust GGUF/Candle Inferenz (§14.1, REQ-INF-001) |
| **`contextra-infer-ollama`** | `[R]` | Ring 2 | `stable` | Remote Ollama Provider Client (§14.2, REQ-INF-002) |
| **`contextra-infer-onnx`** | `[R]` | Ring 2 | `experimental` | ONNX Embedding & Reranker (§14.3, REQ-INF-003) |
| **`contextra-sandbox`** | `[R]` | Ring 2 | `stable` | Wasmtime Isolation Sandbox (§15.1, REQ-SBX-001) |
| **`contextra-privacy`** | `[R]` | Ring 3 | `stable` | Egress-Vault & PII-Filter (§19.1, REQ-PRV-001) |
| **`contextra-adapt`** | `[E]` | Ring 0 | `experimental` | Adaptive Regler (PID / LinUCB) (§18.1, REQ-EXP-001) |
| **`contextra-router`** | `[E]` | Ring 3 | `stable` | Cascade & Drift Router Engine (§18.2, REQ-EXP-002) |
| **`contextra-audit-export`** | `[E]` | Ring 0 | `experimental` | DSGVO Art. 30 Audit-Export (§20.10, REQ-EXP-004) |
| **`contextra-avv-generator`** | `[E]` | Ring 0 | `experimental` | Art. 28 AVV-Template-Generator (§20.11, REQ-EXP-005) |
| **`contextra-license`** | `[E]` | Ring 4 | `experimental` | Lizenzschicht & Ring-Gating (§23.5, REQ-EXP-003) |

---

## Installation & Einbindung

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

## Außenformulierung zur Löschung (INV-CLAIMS-1, REQ-DEL-006)

Aussagen zur Löschung in Contextra folgen zwingend der Außenformulierung gemäß §4.3 und §20.12 der Spezifikation:

> „Nach `forget` bzw. `drop_collection` ist der Scope logisch gelöscht (Tombstone, fsync). Ist sein Schlüssel vernichtet, ist er zusätzlich kryptografisch unlesbar; das weist der Beweis aus. Die physische Bereinigung folgt nach Flush, Purge-Kompaktion und WAL-Bereinigung; erst dann wird ein vollständiger Nachweis für die im Beweis aufgeführten Schichten ausgestellt. Nicht aufgeführte Schichten sind nicht abgedeckt. Für den physischen Datenträger gelten die dokumentierten Rahmenbedingungen (verschlüsseltes Volume, Swap deaktiviert oder verschlüsselt, Backups ohne Keystore).“

Behauptungen wie „100 % gelöscht“, „DSGVO-konform“ oder „garantiert“ sind ausdrücklich unzulässig (§4.3, §20.12).

---

## Dokumentation & Spezifikationen

- **Normative Gesamtspezifikation:** [`docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`](docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md)
- **Systemarchitektur & Ring-Modell:** [`ARCHITECTURE.md`](ARCHITECTURE.md)
- **Sicherheitsrichtlinie:** [`SECURITY.md`](SECURITY.md)
- **Beitragswesen:** [`CONTRIBUTING.md`](CONTRIBUTING.md)

---

## Lizenzierung

Contextra ist unter folgender Dual-Lizenz freigegeben:

- **Apache License, Version 2.0** ([`LICENSE-APACHE`](LICENSE-APACHE))
- **MIT License** ([`LICENSE-MIT`](LICENSE-MIT))
