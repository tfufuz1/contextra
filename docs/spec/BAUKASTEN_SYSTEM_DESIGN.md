# Contextra Baukasten: Systemdesign- & Architektur-Spezifikation

> **Dokument-Status:** Normativ & Verbindlich  
> **Rolle:** Principal Rust Systems Engineer & Software-Architekt  
> **Zielgruppe:** Zukünftige Entwickler, Modul-Implementierer, Drittanwender  
> **Gültigkeit:** Gesamtes Repository `contextra`

---

## 1. Executive Summary & Zielbild

Dieses Dokument definiert die verbindlichen Systemdesign- und Architekturprinzipien für die Transformation des ehemaligen, monolithisch gekoppelten Contextra-Ökosystems in ein **vollständig modulares, gekapseltes Baukastensystem (Building Blocks)**.

### Das Kernversprechen
Zukünftige Entwickler können einzelne Bausteine (z. B. HNSW-Vektorsuche, BM25-Volltextsuche, Personalized PageRank, LinUCB-Bandit-Routing, Rule-based DLP / PII-Vaulting oder LSM-Tree-Persistenz) **isoliert** und **ohne transitive Abhängigkeitslast** in eigene Produktivlösungen (LLM-Anwendungen, Robotik-Steuerungen, Enterprise-Backends) einbinden.

---

## 2. Die 6 Goldenen Architektur- und Entkopplungsprinzipien

### Prinzip 1: Strikte Schichtenhierarchie (E0 bis E3)
Abhängigkeiten fließen ausnahmslos top-down ($E_n \to E_{<n}$). Querverbindungen zwischen unabhängigen Bausteinen derselben Ebene sind verboten, Rückwärtspfeile nach oben sind ausgeschlossen (INV-8).

* **E0 (Foundation & Leaf-Bausteine):** Völlig leere Kern-Closure. Null interne Abhängigkeiten.
  * Bausteine: `ids`, `env`, `sys`, `simd`, `wire`, `durable-fs`, `privacy`, `prompt-guard`, `sandbox`, `erasure`, `license`, `avv-generator`, `audit-export`, `ingest`.
* **E1a (Core Algorithmic Capabilities):** Mathematische Kerne, Inferenz und Graph-Algorithmen. Kern-Closure $\le 4$ E0-Bausteine (typisch `ids`, `env`).
  * Bausteine: `mvcc`, `crypto`, `text`, `graph`, `rank`, `checkpoint`, `kvcache`, `vault`, `infer-ollama`, `infer-candle`.
* **E1b (Core State & Persistence):** Speichersysteme und Indizes. Kern-Closure baut auf E0 und E1a auf.
  * Bausteine: `store` (LSM-Tree), `vector` (HNSW).
* **E2 (Integration & Assembly):** Bindeglieder, Orchestrierung und Fassaden.
  * Bausteine: `engine`, `db`, `cognition`, `agent`, `router`, `contextra`.
* **E3 (Applications & Bindings):** Endnutzer-Tools und Testsuiten.
  * Bausteine: `mcp`, `py`, `testkit`, `bench`.

### Prinzip 2: Autarker Kern (Zero-Transitive-Overhead)
Der funktionale Kern jedes Bausteins muss mit `--no-default-features` kompilierbar sein, **ohne** monolithische Integrationscrates wie `contextra-types` oder `contextra-ports` vorauszusetzen.
* Sämtliche Anbindungen an das übergeordnete Contextra-Ökosystem befinden sich ausnahmslos in `src/compat/` hinter dem Feature `workspace`.
* Im isolierten Zustand nutzt das Modul ausschließlich Standard-Rust-Typen (`&[f32]`, `&str`, generische IDs oder `ids::DocId`).

### Prinzip 3: Micro-Traits & Port-Inversion (Ports & Adapters)
Kein Baustein unterhalb von E2 darf ein anderes konkretes I/O-, Persistenz- oder Indexmodul importieren.
* Umweltanforderungen (Uhrzeit, Zufall, Persistence-Hooks, Filter) werden über minimale, lokale Micro-Traits in einem internen Modul (`src/contract.rs` bzw. `src/traits.rs`) definiert.
* Beispiel: `contextra-text` hängt nicht von `contextra-store` ab, sondern deklariert `trait PostingStore`. Die konkrete Implementierung liefert die Integrationsschicht (E2).

### Prinzip 4: Domänenspezifische Fehlerkapselung (`thiserror`)
Jeder Baustein deklariert seinen eigenen Fehlertyp via `#[derive(thiserror::Error)] #[non_exhaustive] pub enum <Crate>Error`.
* Der monolithische `ContextraError` DARF NICHT in E0- oder E1-Kernen importiert werden.
* Konvertierungen (`From<LocalError> for ContextraError`) existieren ausschließlich im Modul `src/compat/error.rs` hinter dem Feature `workspace`.

### Prinzip 5: Strikte Isolation der Löschkette (GDPR/Compliance)
Alle Gesetzes-, Deletion-Proof-, Saga- und Audit-Logiken gehören ausnahmslos in `contextra-erasure`.
* Kernbausteine wie `store`, `vector`, `text`, `graph` oder `crypto` enthalten keinerlei DSGVO- oder Deletion-Proof-Begriffe.
* Physische Bereinigungen werden neutral über generische Wartungsmethoden (`purge`, `vacuum`) angebunden.

### Prinzip 6: Zero-Panic & Unsafe-Inseln
* In sämtlichen Bausteinen gilt `#![forbid(unsafe_code)]`.
* Einzige Ausnahme sind die drei hardwarenahen Unsafe-Inseln: `contextra-simd`, `contextra-sys` und `contextra-wire`. Diese kapseln Unsafe-Operationen vollständig hinter sicheren öffentlichen APIs und belegen jeden internen Block mit `// SAFETY:`.
* Kein `.unwrap()`, `.expect()`, `panic!()`, `todo!()` oder `unimplemented!()` in Produktionspfaden.

---

## 3. Architektur- und Modul-Festlegung der 6 Kern-Domänen

---

### Domäne 1: Hybrid Retrieval & Suche

#### 1.1 Vektorsuche (`contextra-vector`, `contextra-simd`)
* **Verantwortung:** Hochperformante approximative Nächste-Nachbarn-Suche (ANN), Quantisierung und Vektordistanzberechnung.
* **Algorithmen & Funktionen:**
  - HNSW (Hierarchical Navigable Small World Graph) mit konfigurierbarem $M$, $M_0$ und $ef$.
  - Skalare Quantisierung (SQ8) und RaBitQ zur Reduzierung des RAM-Bedarfs.
  - SIMD-Distanzkerne (L2, Cosine, Dot-Product) für AVX2, AVX-512 und ARM NEON mit skalarem Fallback.
* **Entkopplungs-Architektur:**
  - `contextra-simd` ist Level E0 (autark, keine internen Dependencies, sichere API).
  - `contextra-vector` ist Level E1b. Der Kern benötigt nur `contextra-ids`, `contextra-env`, `contextra-mvcc` und optional `contextra-simd`.
  - Integration: `VectorIndex`-Port-Implementierungen liegen in `src/compat/vector_index.rs`.
* **Minimal-Beispiel für Drittentwickler (unter 30 Zeilen):**
```rust
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use contextra_vector::Metric;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = HnswConfig {
        max_elements: 10_000,
        m: 16,
        ef_construction: 64,
        metric: Metric::Cosine,
    };
    let index = HnswIndex::new(config)?;
    index.insert(1, &[0.1, 0.5, 0.9, -0.2])?;

    let hits = index.search(&[0.12, 0.48, 0.88, -0.19], 5, 32)?;
    for hit in hits {
        println!("Treffer ID: {}, Score: {}", hit.id, hit.score);
    }
    Ok(())
}
```

#### 1.2 Volltextsuche (`contextra-text`)
* **Verantwortung:** Morphologische Textanalyse, Tokenisierung und deterministisches BM25-Scoring.
* **Algorithmen & Funktionen:**
  - Okapi BM25 mit Block-Max WAND (Weak AND) Pruning.
  - Deutsche Morphologie: Compound Splitting, Umlaut-Normalisierung, Stoppwort-Filterung.
  - Inverted Index mit lock-freien Snapshots.
* **Entkopplungs-Architektur:**
  - Level E1a. Benötigt im Kern nur `contextra-ids` und `contextra-env`.
  - Speicherunabhängig: Definiert `trait PostingStore` in `src/posting_store.rs`.
* **Minimal-Beispiel für Drittentwickler:**
```rust
use contextra_text::bm25::Bm25Scorer;
use contextra_text::tokenizer::GermanTokenizer;

fn main() {
    let tokenizer = GermanTokenizer::default();
    let tokens = tokenizer.tokenize("Datenschutzfolgenabschätzung für KI-Systeme");
    println!("Extrahierte Terme: {:?}", tokens);

    let scorer = Bm25Scorer::new(1.2, 0.75);
    let score = scorer.score_term(2, 100, 10, 50, 1000);
    println!("BM25 Term-Score: {:.4}", score);
}
```

#### 1.3 Graph-Analyse & GraphRAG (`contextra-graph`)
* **Verantwortung:** Graphbasierte Relevanzfortpflanzung und Strukturanalyse.
* **Algorithmen & Funktionen:**
  - Personalized PageRank (PPR) via Andersen-Chung-Lang Forward-Push.
  - Compressed Sparse Row (CSR) Graphspeicher.
  - Community Detection & Perkolation.
* **Entkopplungs-Architektur:**
  - Level E1a. Keine internen Abhängigkeiten außer `contextra-ids`.
  - Trennung: Stabile CSR-Graph-Strukturen und PPR im Kern; experimentelle K-Path Diffusion hinter Feature `exp-hyperedge`.
* **Minimal-Beispiel für Drittentwickler:**
```rust
use contextra_graph::csr::CsrGraph;
use contextra_graph::ppr::{ForwardPushPpr, PprConfig};
use contextra_ids::EntityId;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut graph = CsrGraph::new();
    graph.add_edge(EntityId(1), EntityId(2), 1.0);
    graph.add_edge(EntityId(2), EntityId(3), 0.8);

    let ppr = ForwardPushPpr::new(PprConfig::default());
    let ranks = ppr.compute(&graph, &[EntityId(1)])?;
    for (entity, score) in ranks {
        println!("Entity: {:?}, Relevanz: {:.4}", entity, score);
    }
    Ok(())
}
```

#### 1.4 Multi-Signal Rangfusion (`contextra-rank`)
* **Verantwortung:** Gewichtete Fusion heterogener Suchsignale und statistische Score-Kalibrierung.
* **Algorithmen & Funktionen:**
  - Reciprocal Rank Fusion (RRF) & Weighted RRF mit striktem deterministischem Tie-Breaking (aufsteigende ID).
  - Isotonische Regression (PAVA) und Platt-Scaling.
  - Conformal Prediction für finiten Konfidenzbereich.
* **Entkopplungs-Architektur:**
  - Level E1a. Mathematischer Kern arbeitet auf generischen Typen/Slices.
* **Minimal-Beispiel für Drittentwickler:**
```rust
use contextra_rank::fusion::{rrf_fuse, ScoredItem};

fn main() {
    let vector_results = vec![ScoredItem::new(1, 0.95), ScoredItem::new(2, 0.80)];
    let text_results = vec![ScoredItem::new(2, 12.4), ScoredItem::new(3, 8.1)];

    let fused = rrf_fuse(&[&vector_results, &text_results], 60);
    for hit in fused {
        println!("Dokument ID: {}, Fused Score: {:.5}", hit.id, hit.score);
    }
}
```

---

### Domäne 2: Autonome Agenten & Kognition

#### 2.1 Agenten-Zustandsmaschine (`contextra-agent`)
* **Verantwortung:** Deterministische Multi-Step-Workflow-Steuerung.
* **Algorithmen & Funktionen:**
  - StateGraph FSM mit deterministischem Zyklus: `checkpoint` $\to$ `execute` $\to$ `commit` $\to$ `audit`.
  - RAII `CheckpointGuard` für automatischen Rollback bei Fehlern.
  - `TokenBudget` mit atomarer Vorreservierung zur Verhinderung von Endlosschleifen.
* **Entkopplungs-Architektur:**
  - Level E2. Kapselt Speicherzugriffe über Ports (`ContextStore`, `ScratchpadStore`).

#### 2.2 Langzeitgedächtnis & Konsolidierung (`contextra-cognition`)
* **Verantwortung:** Verdichtung und semantische Deduplizierung von Konversationshistorien.
* **Algorithmen & Funktionen:**
  - Near-Duplicate Clustering via Kosinus-Schwellenwert.
  - Dialog-Verdichtung (Truncate, Summary, Selective).
  - Transitivity Veto zur Verhinderung ungültiger Wissensverschmelzungen.
* **Entkopplungs-Architektur:**
  - Level E2. Konsolidierungslogik operiert über generische Slices `&[(DocId, Vec<f32>)]`.

---

### Domäne 3: Adaptives Routing & Modellsteuerung

#### 3.1 Bandit-Routing & Regelung (`contextra-adapt`, `contextra-router`)
* **Verantwortung:** Intelligente Modellauswahl nach Güte, Latenz und Kosten.
* **Algorithmen & Funktionen:**
  - LinUCB Contextual Bandits mit Sherman-Morrison $O(d^2)$ Updates oder $O(d)$ Diagonal-Approximation.
  - Flow-Corrected Thompson Sampling (FC-TS) mit Drift-Kompensation.
  - PID-Latenzregler zur Einhaltung von SLAs.
* **Entkopplungs-Architektur:**
  - `contextra-adapt` ist **Level E0** (vollkommen autark, null interne Abhängigkeiten).
  - Rein mathematische Formulierung auf `f32`/`f64`-Vektoren, unabhängig von konkreten LLM-Anbietern.
* **Minimal-Beispiel für Drittentwickler:**
```rust
use contextra_adapt::bandit::{LinUcbBandit, LinUcbConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = LinUcbConfig { dimension: 4, alpha: 0.2 };
    let mut bandit = LinUcbBandit::new(config)?;

    let context = [0.2, -0.5, 0.8, 0.1];
    let selected_arm = bandit.select_arm(&context, &[1, 2, 3])?;
    println!("Ausgewähltes Modell (Arm): {}", selected_arm);

    bandit.update(selected_arm, &context, 0.95)?; // Belohnung rückmelden
    Ok(())
}
```

---

### Domäne 4: Datenschutz, Sicherheit & Compliance

#### 4.1 Egress-Prüfung & Data Loss Prevention (`contextra-privacy`, `contextra-prompt-guard`)
* **Verantwortung:** Erkennung und Maskierung sensibler Daten vor dem Verlassen des Systems; Injektionsabwehr.
* **Algorithmen & Funktionen:**
  - Regelbasierte Entity Recognition (IBAN, E-Mail, Kreditkarten) mit Prüfziffervalidierung.
  - Surrogate Vaulting: Deterministische Token-Maskierung mit reversibler Rehydrierung.
  - Sliding-Window Rate-Limiter für Bulk-Exfiltration.
  - Prompt-Injection-Heuristik.
* **Entkopplungs-Architektur:**
  - Beide Crates sind **Level E0** (100% autark).
  - Universell einsetzbar in API-Gateways, Proxies und Middleware.
* **Minimal-Beispiel für Drittentwickler:**
```rust
use contextra_privacy::entity_recognizer::EntityRecognizer;
use contextra_privacy::vault::SurrogateVault;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let recognizer = EntityRecognizer::default();
    let text = "Überweise an DE02100100100123456789 bitte.";
    let matches = recognizer.find_entities(text);

    let mut vault = SurrogateVault::new();
    let masked = vault.mask_entities(text, &matches)?;
    println!("Maskierter Text: {}", masked);

    let restored = vault.unmask_entities(&masked)?;
    println!("Rehydrierter Text: {}", restored);
    Ok(())
}
```

#### 4.2 Löschnachweise & Kryptografie (`contextra-erasure`, `contextra-crypto`, `contextra-vault`, `contextra-sys`)
* **Verantwortung:** Unwiederbringliche Schlüsselvernichtung (Crypto Shredding), digitale Löschnachweise und OS-Memory-Locking.
* **Algorithmen & Funktionen:**
  - Crypto Shredding via Argon2id / AES-256-GCM-SIV DEK-Zerstörung.
  - Ed25519-signierte Löschnachweise.
  - `mlock`/`munlock` Systemaufrufe via `contextra-sys` und flüchtige Schlüssel via `ZeroizeOnDrop`.
* **Entkopplungs-Architektur:**
  - `contextra-erasure` kapselt alle DSGVO- und Deletion-Proof-Prozesse autark (Level E0).
  - `contextra-sys` ist die Ring-0 Unsafe-Island für POSIX-Primitives.

---

### Domäne 5: Speichersystem, Transaktionen & Isolation

#### 5.1 LSM-Tree Speicherkern (`contextra-store`)
* **Verantwortung:** Durabler, crash-konsistenter Key-Value-Speicher.
* **Algorithmen & Funktionen:**
  - LSM-Tree mit SkipList-MemTable, hierarchischer Kompaktierung und SSTables (V4).
  - 2-Stufen-Bloom-Filter für latenzfreie Negativprüfungen.
  - Write-Ahead-Log (WAL) mit BLAKE3/HMAC-Integritätskette.
* **Entkopplungs-Architektur:**
  - Level E1b. Kern hängt ausschließlich von `contextra-ids`, `contextra-env`, `contextra-sys`, `contextra-mvcc` und `contextra-crypto` ab.
  - Eigener Fehlertyp `StoreError`. Keine Abhängigkeit zu `types` oder `ports` im Basiskern.

#### 5.2 Transaktionsisolation (`contextra-mvcc`)
* **Verantwortung:** Serializable Snapshot Isolation (SSI) und Lock-freie Commit-Validierung.
* **Entkopplungs-Architektur:**
  - Level E1a. Hängt ausschließlich an `contextra-ids` und `contextra-env`.

#### 5.3 Ausführungs-Isolation (`contextra-sandbox`)
* **Verantwortung:** Gekapselte Ausführung ungesicherter Plugins/Tools mittels deterministischem Fuel-Budgeting (WASM).
* **Entkopplungs-Architektur:**
  - Level E0 (vollkommen autark).

---

### Domäne 6: Ingest, Inferenz & Anwendungen

#### 6.1 Dokumenten-Ingest (`contextra-ingest`)
* **Verantwortung:** Formatbewusstes Chunking und strukturierte Text-Extraktion (Markdown, Text, PDF).
* **Entkopplungs-Architektur:**
  - Level E0. Formate modular gegated (`fmt-text`, `fmt-pdf`).

#### 6.2 Inferenz-Konnektoren (`contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`)
* **Verantwortung:** Lokale und externe Modellanbindung für Embeddings und Textgenerierung.
* **Entkopplungs-Architektur:**
  - Level E1a. Enthalten Fakes und Fallbacks, sodass Builds auch ohne GPU-Treiber oder C-Libraries autonom lauffähig sind.

#### 6.3 Model Context Protocol Server (`contextra-mcp`)
* **Verantwortung:** Standardisiertes JSON-RPC 2.0 / STDIO Interface für Agenten, Cursor und Claude Desktop.
* **Entkopplungs-Architektur:**
  - Level E3. Verdrahtet die darunterliegenden Bausteine nach außen.

---

## 4. Leitfaden für Entwickler: Einen Baustein in ein Drittprojekt einbinden

1. **`Cargo.toml` minimal halten:**
   Drittprojekte binden stets nur den benötigten Crate mit `default-features = false` ein und wählen benötigte Features modular:
   ```toml
   [dependencies]
   contextra-vector = { version = "0.3", default-features = false, features = ["simd"] }
   contextra-privacy = { version = "0.2" }
   ```
2. **Keine Ökosystem-Altlasten mitschleppen:**
   Das Feature `workspace` wird im Drittprojekt niemals aktiviert. Dadurch bleibt der transitive Abhängigkeitsbaum minimal und frei von internen Orchestrierungs-Layern.
3. **Eigene Umwelt-Typen injizieren:**
   Falls ein Baustein Zeiterfassung oder Entropie benötigt, werden die Micro-Traits aus `contextra-env` oder dem lokalen `contract.rs` implementiert (oder mit Standard-Typen aus `env/std` bedient).

---

## 5. Checkliste für Architektur-Reviews & Refactorings

- [x] Ist der Baustein in der korrekten Ebene (E0, E1a, E1b, E2, E3) verortet?
- [x] Baut der Baustein fehlerfrei mit `cargo check -p <crate> --no-default-features`?
- [x] Sind im Basiskern keine Importe von `contextra_types` oder `contextra_ports` vorhanden?
- [x] Kapselt der Crate alle Fehler in einem lokalen `#[derive(thiserror::Error)]`?
- [x] Wurden alle DSGVO- und Deletion-Proof-Logiken in `contextra-erasure` isoliert?
- [x] Wird `#![forbid(unsafe_code)]` strikt eingehalten (außer in `simd`, `sys`, `wire`)?
- [x] Liegt für jeden Baustein ein kompilierbares Minimal-Beispiel vor?
