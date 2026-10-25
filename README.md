# Contextra

Contextra ist die lokale, eingebettete Speicher- und Gedächtnis-Infrastruktur für KI-Anwendungen (§1): Schlüssel-Wert-Speicher, Vektorindex (HNSW/SQ8), Volltextsuche (BM25/BM25F) und Wissensgraph bilden mit Metadaten, Audit und Kryptografie eine einzige, atomar konsistente Wahrheit (ein WAL) je Mandant getrennt (§1, §20.2, VETO-F10). Zugänglich als reine Rust-Bibliothek und als MCP-Server über stdio (§1, §21).

> **Dokumentationsstand:** Normativ abgestimmt mit der **[Normativen Gesamtspezifikation Fassung 1.2](docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md)** (§0–§34).

---

## Die fünf Produktmissionen (§2)

1. **M1 — Persistenter Prefix-Cache (`[R]`, §13):** Wiederverwendung von KV-Cache-Blöcken über Anfragen hinweg. *Hinweis zum Ist-Zustand:* M1 wird gemäß §13.1 erst dann produktiv behauptet, wenn die Bedingungen B1–B3 (Löschkette, `Generator`-Backend mit `external_kv` und veröffentlichtes Break-even-Gate) maschinell belegt sind.
2. **M2 — Atomare Suchraum-Konsistenz (`[K]`, §7, §8, §9, §10, §11):** WAL als einzige Wahrheit. Vektor-, Text- und Graph-Indexe werden atomar mit den Metadaten synchronisiert (INV-ATOMIC-IDX-1).
3. **M3 — Semantische Präzision mit Mandanten-Grenze (`[K]`, §9, §10, §12):** Strikte Mandantentrennung auf Vektor-, Text- und Graphebene (VETO-F10). ACL-Prüfung erfolgt vor der Distanzberechnung.
4. **M4 — Belegbare Antworten (`[K]`, §20):** Jedes Ergebnis ist auf einen WAL-Eintrag rückführbar. Kryptografischer Löschnachweis über einen signierten `DeletionProof` (Formatversion 4; §20.4).
5. **M5 — Deterministische Filterung neben den Daten (`[R]`, §14, §15):** WASM-basierte Filter am Daten-Port mit begrenztem Fuel-Budget.

---

## Produktvision & Missionen M1–M5 (§2.0)

1. **Ehrlicher Löschnachweis statt Zusage (INV-CLAIMS-1, §4.3):** Löschung nach `forget` bzw. `drop_collection` erfolgt logisch sofort (Tombstone, fsync, S0; §20.12). Ist der Schlüssel vernichtet, ist der Scope zusätzlich kryptografisch unlesbar (SK; §20.2a). Die physische Bereinigung (S1–S3) folgt nach Flush, Purge-Kompaktion und WAL-Scrub; erst dann wird ein signierter `DeletionProof` (Formatversion 4; §20.4) ausgestellt, der nicht belegte Schichten ehrlich ausweist (§4.3).
2. **Embedded & Single-Node-Garantie (§1, §4.1):** Läuft vollständig ohne Netzwerk, ohne externe API-Keys, ohne separaten Serverprozess und ohne Telemetrie im selben Prozess wie die Anwendung des Nutzers (`cargo add contextra`). Single-Node, kein Sharding, keine Replikation (§1, §4.1).
3. **Pure Rust Inferenz (`contextra-infer-candle`, `[R]`, §15):** Lokale Inferenz ohne C++ / CUDA FFI-Abhängigkeiten oder extern laufende Dämonen (§6.0).
4. **Deterministischer Kern (INV-RING0-DETERMINISM-1, §3, §5):** Gleicher Anfangszustand und gleiche WAL-Folge liefern bit-identischen Zustand. Nichtdeterministische Quellen (LLM, Embeddings, Uhr) werden als Werte im WAL aufgezeichnet (INV-NONDET-1). Zero-Panic-Garantie im Produktionspfad (§3, §27.6).
5. **Abgestufte Normativitätsklassen (§6.0):**
   - **Kern `[K]`:** Gilt uneingeschränkt. Storage Engine (`contextra-store`), Vektor (`contextra-vector`), Text (`contextra-text`), Types (`contextra-types`), Ports (`contextra-ports`), SIMD (`contextra-simd`), Sys (`contextra-sys`), Wire (`contextra-wire`), MVCC (`contextra-mvcc`), Core (`contextra-core`), Crypto (`contextra-crypto`), Checkpoint (`contextra-checkpoint`), Engine (`contextra-engine`), DB (`contextra-db`), MCP (`contextra-mcp`), Facade (`contextra`).
   - **Rand `[R]`:** Gilt nur bei aktiviertem Feature und darf Kern-Garantien nie schwächen (§6.0, §6.11). Graph (`contextra-graph`), Rank (`contextra-rank`), Adapt (`contextra-adapt`), KVCache (`contextra-kvcache`), Cognition (`contextra-cognition`), Privacy (`contextra-privacy`), Sandbox (`contextra-sandbox`), Inferenz-Backends (`contextra-infer-candle`, `contextra-infer-ollama`), Python-Bindings (`contextra-py`).
   - **Experimentell `[E]`:** Nicht normativ; kein Bestandteil von Release-Zusagen (§6.0). Router (`contextra-router`), Agent (`contextra-agent`), ONNX (`contextra-infer-onnx`), Compliance/Export (`contextra-audit-export`, `contextra-avv-generator`), Lizenzschicht (`contextra-license`).

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

| Feature / Crate | Klasse (§6.0) | Ring | Status / Maturity (`capabilities.toml`) | Spezifikationsbezug |
|---|---|---|---|---|
| **`contextra`** | `[K]` | Ring 4 | `stable` | Haupt-Library Facade (§22) |
| **`contextra-store`** | `[K]` | Ring 1 | `stable` | LSM-Tree Storage, WAL, Compaction (§7) |
| **`contextra-vector`** | `[K]` | Ring 0 | `stable` | Vektorsuche (HNSW, DiskANN, SQ8) (§9) |
| **`contextra-text`** | `[K]` | Ring 0 | `stable` | Volltextsuche (BM25/BM25F, WAND) (§10) |
| **`contextra-types`** | `[K]` | Ring 0 | `stable` | Typen, IDs, Budgets, Fehlerklassen (§6.1) |
| **`contextra-ports`** | `[K]` | Ring 0 | `stable` | Trait-Schnittstellen für Port-Entkopplung (§6.1, Anhang A) |
| **`contextra-simd`** | `[K]` | Ring 0 | `stable` | SIMD Distanzberechnungen (Unsafe Island; §6.7) |
| **`contextra-sys`** | `[K]` | Ring 0 | `stable` | Low-Level OS/IO Bindings (Unsafe Island; §6.7) |
| **`contextra-wire`** | `[K]` | Ring 0 | `stable` | FlatBuffers Wire-Protokolle (Unsafe Island; §6.7) |
| **`contextra-mvcc`** | `[K]` | Ring 0 | `stable` | MVCC, SSI Tracking & Read-Sets (§8) |
| **`contextra-core`** | `[K]` | Ring 0 | `stable` | Kern-Buffer & Deprecation-Reexporte (§6.1) |
| **`contextra-crypto`** | `[K]` | Ring 0 | `stable` | AES-256-GCM-SIV, HMAC, DeletionProof v4 (§20) |
| **`contextra-checkpoint`** | `[K]` | Ring 1 | `stable` | Checkpoint Engine & Manifests (§16) |
| **`contextra-engine`** | `[K]` | Ring 3 | `stable` | Compute Pool & Storage Execution (§16) |
| **`contextra-db`** | `[K]` | Ring 3 | `stable` | Main DB Abstraction & Collections (§22) |
| **`contextra-mcp`** | `[K]` | Ring 4 | `stable` | MCP Server über stdio (§21) |
| **`contextra-graph`** | `[R]` | Ring 0 | `stable` | CSR Wissensgraph & PPR (§11) |
| **`contextra-rank`** | `[R]` | Ring 0 | `stable` | Signal-Fusion (RRF) & Kalibrierung (§12) |
| **`contextra-adapt`** | `[R]` | Ring 0 | `experimental` | Adaptive Regler (PID / LinUCB) (§18) |
| **`contextra-kvcache`** | `[R]` | Ring 1 | `stable` | Tiered KV-Cache & Segment Storage (§13) |
| **`contextra-infer-candle`** | `[R]` | Ring 2 | `stable` | Pure Rust GGUF/Candle Inferenz (§15) |
| **`contextra-infer-ollama`** | `[R]` | Ring 2 | `stable` | Remote Ollama Provider Client (§15) |
| **`contextra-sandbox`** | `[R]` | Ring 2 | `stable` | Wasmtime Isolation Sandbox (§14) |
| **`contextra-cognition`** | `[R]` | Ring 3 | `stable` | Gedächtniskonsolidierung & Maintenance (§17) |
| **`contextra-privacy`** | `[R]` | Ring 3 | `stable` | Egress-Vault & PII-Filter (§19) |
| **`contextra-py`** | `[R]` | Ring 4 | `stable` | PyO3 Python Bindings (§22) |
| **`contextra-router`** | `[E]` | Ring 3 | `stable` | Cascade & Drift Router Engine (§18a) |
| **`contextra-agent`** | `[E]` | Ring 3 | `stable` | Persistent Agent Workflows & Audit Loop (§18) |
| **`contextra-infer-onnx`** | `[E]` | Ring 2 | `experimental` | ONNX Embedding & Reranker (§15) |
| **`contextra-audit-export`** | `[E]` | Ring 0 | `experimental` | Audit-Log & Art. 30 Export (§20) |
| **`contextra-avv-generator`** | `[E]` | Ring 0 | `experimental` | Art. 28 AVV-Template-Generator (§20) |
| **`contextra-license`** | `[E]` | Ring 4 | `experimental` | Lizenzschicht & Ring-Gating (§23) |

---

## Installation & Einbindung (§22.1)

Füge `contextra` zu deiner `Cargo.toml` hinzu (§22):

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
    // 1. Datenbankinstanz konfigurieren und öffnen (§22)
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

## Außenformulierung zur Löschung (INV-CLAIMS-1)

Aussagen zur Löschung in Contextra folgen zwingend der Außenformulierung gemäß §4.3 und §20.12 der Spezifikation:

> „Nach `forget` bzw. `drop_collection` ist der Scope logisch gelöscht (Tombstone, fsync). Ist sein Schlüssel vernichtet, ist er zusätzlich kryptografisch unlesbar; das weist der Beweis aus. Die physische Bereinigung folgt nach Flush, Purge-Kompaktion und WAL-Bereinigung; erst dann wird ein vollständiger Nachweis für die im Beweis aufgeführten Schichten ausgestellt. Nicht aufgeführte Schichten sind nicht abgedeckt. Für den physischen Datenträger gelten die dokumentierten Rahmenbedingungen (verschlüsseltes Volume, Swap deaktiviert oder verschlüsselt, Backups ohne Keystore).“

Werbe- oder Pauschalaussagen zur Löschwirkung oder Rechtskonformität sind unzulässig (§4.3, §20.12).

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
