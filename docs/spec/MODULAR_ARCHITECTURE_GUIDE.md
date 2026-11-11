# Contextra Baukastensystem: Architektur-, Systemdesign- & Entkopplungs-Leitfaden

> **Status:** Normativ & Verbindlich
> **Rolle:** Principal Rust Systems Engineer
> **Fokus:** Bausteinsystem, Entkopplung, Wiederverwendbarkeit in Drittlösungen, Zero-Cost Abstraktionen

---

## 1. Executive Summary & Zielbild

Dieses Repository wird von einem monolithisch verdrahteten Gesamtsystem zu einem **hochgradig modularen, gekapselten Baukastensystem (Building Blocks)** umstrukturiert.

Ziel ist es, dass externe Softwareingenieure und interne Teams einzelne funktionale Module (z. B. HNSW-Vektorsuche, BM25-Textsuche, Personalized PageRank, Contextual Bandit Routing, Rule-Based DLP oder LSM-Storage) **isoliert** und **ohne transitive Abhängigkeitslast** in eigene Applikationen einbinden können.

### Die 5 Goldenen Architekturprinzipien des Baukastens:
1. **Strikte Ebenen-Hierarchie (E0 bis E3):** Abhängigkeiten fließen immer nur abwärts ($E_n \to E_{<n}$); Rückwärts- oder Querverbindungen sind verboten (INV-8).
2. **Autarke Kerne (Zero-Transitive-Overhead):** Der Kern eines Bausteins (`--no-default-features`) enthält *keine* Workspace-weiten Monster-Abhängigkeiten (`contextra-types`, `contextra-ports`). Alle Integrationen in das Contextra-Ökosystem liegen isoliert im Brücken-Modul `src/compat/` hinter dem Feature `workspace`.
3. **Micro-Traits & Lokale Abstraktionen (Ports & Adapters):** Bausteine definieren ihre Umwelt-Abstraktionen (wie Zeitmessung, Zufallsgeneratoren, Storage-Backends) als minimale, lokale Micro-Traits (`traits.rs` / `ext.rs`). Der Baustein hängt nicht von konkreten Implementierungen ab.
4. **Fehler-Kapselung mit `thiserror`:** Jeder Baustein deklariert seinen eigenen domänenspezifischen Fehlertyp (z. B. `VectorError`, `TextError`, `GraphError`, `StoreError`). Keine externen Typen lecken in die öffentliche Signatur. Konvertierungen (`From<LocalError> for ContextraError`) existieren ausschließlich im `compat/`-Modul.
5. **Single Responsibility & Feature-Hygiene (R10):** Features wie `std`, `simd` oder experimentelle Algorithmen (`exp-*`) sind modular zuschaltbar. Das `default`-Feature ist minimal und enthält niemals `workspace` oder Test-Utilities.

---

## 2. Das Ebenen-Modell (Levels E0 – E3)

| Ebene | Definition | Enthaltene Bausteine |
|---|---|---|
| **E0 (Foundation & Leaf)** | Kern-Closure ist völlig leer. Keine internen Abhängigkeiten. Maximale Autarkie. | `ids`, `env`, `sys`, `durable-fs`, `simd`, `wire`, `adapt`, `ingest`, `privacy`, `sandbox`, `prompt-guard`, `infer-onnx`, `erasure`, `license`, `audit-export`, `avv-generator` |
| **E1a (Core Algorithmic)** | Kern hängt maximal von E0 ab ($\le 4$ Abhängigkeiten, z. B. `ids`, `env`). Reiner Algorithmus/Inferenz. | `mvcc`, `crypto`, `text`, `graph`, `rank`, `checkpoint`, `kvcache`, `vault`, `infer-ollama`, `infer-candle` |
| **E1b (Core State & Index)**| Kern hängt von E0 und E1a ab (z. B. `store` benötigt `mvcc`, `crypto`, `sys`; `vector` benötigt `mvcc`). | `store`, `vector` |
| **E2 (Integration & Assembly)**| Verbindet Bausteine über `ports` und `types` zu Subsystemen oder Engines. | `types`, `ports`, `router`, `agent`, `cognition`, `engine`, `db`, `contextra` |
| **E3 (Applications & Tooling)**| Endbenutzer-Frontends, Server, Bindings und Test-Suiten. | `mcp`, `py`, `testkit`, `bench` |

---

## 3. Detaillierte Analyse & Architektur-Festlegung der 6 Domänen

---

### Domäne 1: Hybrid Retrieval & Suche

Die Domäne vereint semantische Vektorsuche, klassische Volltextsuche, Graph-Traversierung und Ergebnisse-Rangfusion.

#### 1.1 Vektorsuche (`contextra-vector`, `contextra-simd`)
* **Funktionen & Algorithmen:**
  - **HNSW (Hierarchical Navigable Small World):** Graphbasierter ANN-Index mit konfigurierbaren Parametern ($M, M_0, ef\_construction, ef\_search$).
  - **SQ8 (Skalare Quantisierung):** 8-Bit-Kompression von Float-Vektoren für minimierten RAM-Footprint.
  - **RaBitQ & DiskANN:** Out-of-Core Graphindex für Datensätze größer als der Hauptspeicher mit binaurer Quantisierung.
  - **Prädikat-augmentierte Vektorsuche (Acorn):** Vektor-Graph-Filterung mit dynamischem Pruning.
  - **SIMD-Distanzkerne (`contextra-simd`):** AVX2-, AVX-512- und ARM NEON-Dispatch für L2, Cosine und Dot Product mit sicherem skalarem Fallback.
* **Architektur & Moduldesign:**
  - `contextra-simd` ist ein E0-Baustein mit Unsafe-Island-Garantie (`#![deny(unsafe_op_in_unsafe_fn)]`) und reiner Safe-Rust-API.
  - `contextra-vector` ist E1b. Der Kern benötigt nur `contextra-ids`, `contextra-env`, `contextra-mvcc` und optional `contextra-simd`.
  - Direkte Kopplungen an `contextra-ports` und `contextra-types` werden im Kern eliminiert. Die Port-Implementierung `contextra_ports::VectorIndex` liegt strikt in `src/compat/vector_index.rs` hinter `feature = "workspace"`.
  - Der Vektorindex bietet generische Typ-Signaturen: `Index<ID, Vector>` mit `Vector: AsRef<[f32]>`.

#### 1.2 Volltextsuche (`contextra-text`)
* **Funktionen & Algorithmen:**
  - **BM25 Scoring:** Okapi BM25 mit konfigurierbarem $k_1$ und $b$.
  - **Block-Max WAND (Weak AND):** Dynamisches Score-Pruning über Posting-Block-Metadaten (`PostingBlockInfo`) für drastisch reduzierte Traversierungszeit.
  - **Deutscher Morphologie-Algorithmus:** Regelbasiertes Compound Splitting, Umlaut-Normalisierung, Stoppwort-Filter.
  - **Inverted Index:** Lock-freie CoW-Snapshots für Leseisolation.
* **Architektur & Moduldesign:**
  - E1a-Baustein: Hängt nur von `contextra-ids` und `contextra-env` ab.
  - Frühere tiefe Abhängigkeit zu `StorageEngine` wird durch ein schmales, lokales Trait `PostingStore` in `src/posting_store.rs` ersetzt.
  - Die Brücke zu `contextra-store` implementiert `PostingStore for Store` in `contextra-engine` oder via `compat/`.
  - Eigener Fehlertyp `TextError`. Keine Panics bei Indexierungsfehlern.

#### 1.3 Graph-Suche & GraphRAG (`contextra-graph`)
* **Funktionen & Algorithmen:**
  - **Personalized PageRank (PPR):** Forward-Push Algorithmus nach Andersen-Chung-Lang mit garantierter Rest-Schranke $\epsilon$.
  - **CSR (Compressed Sparse Row):** Speichereffiziente Adjazenzstruktur mit parallelen Eigenschafts-Arrays.
  - **K-Path Diffusion & APPRH:** Multi-Hop-Traversierung und adaptive Hyperkanten-Expansion.
  - **Community Detection & Percolation:** Physio-Perkolationsschwellen und dynamisches Kanten-Reinforcement.
* **Architektur & Moduldesign:**
  - E1a-Baustein: Keine internen Abhängigkeiten im Kern außer `contextra-ids` und `contextra-env`.
  - Trennung: Stabile CSR-Graph-Strukturen und Standard-PPR verbleiben im Kern. Komplexe experimentelle Algorithmen (K-Path Diffusion, Perkolation) werden hinter `exp-*`-Features (`exp-hyperedge`, `exp-diffusion`) isoliert.
  - `PprConfig`, `PprAlgorithm` und `TlHfdParams` werden direkt in `contextra-graph` definiert (Besitzer-Prinzip R4) statt in `contextra-types`.
  - Eigener Fehlertyp `GraphError`.

#### 1.4 Rangfusion & Kalibrierung (`contextra-rank`)
* **Funktionen & Algorithmen:**
  - **Weighted RRF & mRRF:** Multi-Signal Rangfusion mit parametrisierbarem $k$ (Standard 60) und Randabstands-Modulation.
  - **Resonanz-Bonus:** Multi-Signal Koinzidenz-Bonus.
  - **Isotonic Regression (PAVA) & Platt Scaling:** Wahrscheinlichkeitskalibrierung von Konfidenzwerten via Pool-Adjacent Violators.
  - **Conformal Prediction:** Exakte Konfidenzintervalle auf finiten Stichproben.
  - **Drift Detection:** Kolmogorov-Smirnov / PSI-basierte Verteilungsprüfung.
* **Architektur & Moduldesign:**
  - E1a-Baustein: Mathematische Algorithmen arbeiten direkt auf generischen Iteratoren oder Slices `&[Candidate<T>]`.
  - Autarkie: Völlig unabhängig von Search-Engines; kann zur Fusion beliebiger Signale in Empfehlungssystemen, Rankings und Klassifikatoren eingesetzt werden.
  - Eigener Fehlertyp `RankError`.

---

### Domäne 2: Autonome Agenten & Kognition

Diese Domäne kapselt die deterministische Steuerung von Multi-Step-Workflows und die semantische Konsolidierung von Langzeit-Erinnerungen.

#### 2.1 Agenten-Engine (`contextra-agent`)
* **Funktionen & Algorithmen:**
  - **StateGraph FSM:** Deterministischer Zustandsautomat: `checkpoint` $\to$ `execute` $\to$ `commit` $\to$ `audit`.
  - **RAII Auto-Checkpointing & Rollback:** `CheckpointGuard` sorgt bei Fehlern für deterministisches Zurücksetzen des Transaktions- und Graph-Zustands.
  - **Token-Budget-Steuerung (`TokenBudget`):** Atomare Vorreservierung und strikte Begrenzung von Schritten und Tokens zur Schleifenverhinderung.
* **Architektur & Moduldesign:**
  - E2-Baustein: Entkopplung von konkreten Datenbanken durch lokale Ports `ContextStore` und `ScratchpadStore`.
  - Fallible Konstruktoren (`try_new`, `try_add_node`) gemäß Zero-Panic-Doktrin.
  - Eigener Fehlertyp `AgentError`.

#### 2.2 Langzeitgedächtnis & Kognition (`contextra-cognition`)
* **Funktionen & Algorithmen:**
  - **Memory Consolidation:** Near-Duplicate Detection, Sliding-Window Clustering, Strukturanalyse.
  - **Context Compaction:** Truncate-, Summary- und Selective-Verdichtung für Dialoghistorien.
  - **LeanRAG Stage 3:** Diagonal-GMM Clustering, Typ-Denoising, Alpha-Knoten-Synthese.
  - **Transitivity Veto:** Mathematischer Schutz vor ungültigen Assoziationsverschmelzungen.
* **Architektur & Moduldesign:**
  - E2-Baustein: Konsumiert `contextra-graph` und `contextra-types`.
  - Bisherige zirkuläre Rückkopplung zu `contextra-engine` wird invertiert: `contextra-cognition` definiert schmale Ports (`DocumentSource`, `ConsolidationSink`), welche von Applikationsschichten implementiert werden.
  - Eigener Fehlertyp `CognitionError`.

---

### Domäne 3: Adaptives Routing & Modellsteuerung

Steuert die dynamische Auslastung, Selektion und Güteüberwachung von KI-Modellen.

#### 3.1 Bandit-Routing & Regler (`contextra-adapt`, `contextra-router`)
* **Funktionen & Algorithmen:**
  - **LinUCB Contextual Bandits:** Kontextuelle Banditen mit Sherman-Morrison $O(d^2)$ Rang-1 Updates oder $O(d)$ Diagonal-Approximation.
  - **Flow-Corrected Thompson Sampling (FC-TS):** Bayessches Routing mit Drift-Kompensation.
  - **RIE-Greedy:** Regularisierungsinduzierte Exploration.
  - **PID-Latenzregler (`PidController`):** Regelkreis zur Anpassung von Candidate-Pools gegen Latenz-SLA.
  - **Homeostat & Decay-Controller:** Exponentielle Halbwertszeit-Entwertung und Gleichgewichtsregelung.
* **Architektur & Moduldesign:**
  - `contextra-adapt` ist **Level E0** (100% autarker Baustein). Besitzt *null* interne Abhängigkeiten.
  - Alle Algorithmen basieren auf reinem Rust mit `f64`/`f32` Vektoren. Keine Bindung an spezifische Tokenizer oder LLM-Clients.
  - `contextra-router` (E2) baut auf `contextra-adapt` auf und mappt HTTP-/IPC-Anfragen über `contextra-wire`.

---

### Domäne 4: Datenschutz, Sicherheit & Compliance

Gewährleistet Egress-Prüfung, Data Loss Prevention (DLP), kryptografische Nachweise und physische Speicherisolation.

#### 4.1 Egress-Schutz & DLP (`contextra-privacy`, `contextra-prompt-guard`)
* **Funktionen & Algorithmen:**
  - **Rule-Based Entity Recognition & DLP:** Erkennung von IBAN, E-Mail, Kreditkarten, Steuer-IDs über Regex und Prüfsummen.
  - **Surrogate Vaulting:** Umwandlung von PII in deterministische Surrogat-Token mit verschlüsselter Rehydrierung nach LLM-Inferenz.
  - **k-NN Egress Guard:** Vektorähnlichkeitsprüfung gegen gesperrte vertrauliche Dokumente.
  - **Bulk Exfiltration Detector:** Sliding-Window Rate-Limiter für Token- und Datenvolumina.
  - **Prompt-Injection Detection:** Heuristische und modellbasierte Injektions-Abwehr.
  - **Context Edit Audit:** BLAKE3-kryptografische Hashketten zur Auditierung.
* **Architektur & Moduldesign:**
  - Beide Crates sind **Level E0** Bausteine (Score 100/100). Vollständig autark nutzbar in API-Gateways, Proxies und Middleware-Lösungen.
  - Keine Abhängigkeit zu LLM-Frameworks oder Storage-Engines. Eigene Fehlertypen (`GuardError`).

#### 4.2 Compliance, Verschlüsselung & Vaulting (`contextra-erasure`, `contextra-crypto`, `contextra-vault`, `contextra-sys`)
* **Funktionen & Algorithmen:**
  - **Cryptographic Shredding:** Unwiederbringliches Löschen durch Vernichtung von AES-256-GCM-SIV DEKs; KDF via Argon2id / HKDF.
  - **Ed25519 Deletion Proofs:** Kryptografisch signierte Löschnachweise.
  - **RAM-Locking & Vaulting:** `mlock` / `VirtualLock` via `contextra-sys` zur Verhinderung von OS-Paging; `ZeroizeOnDrop` für flüchtige Schlüssel im RAM.
* **Architektur & Moduldesign:**
  - `contextra-sys` ist die Ring-0 Unsafe-Island (E0). Kapselt Low-Level Systemaufrufe (`mmap`, `mlock`, `DurableFile`).
  - `contextra-vault` (E1a) bietet sichere Speicherpuffer über `contextra-sys`.
  - `contextra-erasure` ist vollständig autark (E0). Alle DSGVO- und Deletion-Proof-Logiken sind hier isoliert. Kernkomponenten wie `store` oder `vector` implementieren lediglich neutrale Datenbereinigungsmethoden (`purge`, `vacuum`).

---

### Domäne 5: Speichersystem, Transaktionen & Isolation

Stellt fundamentale Persistenz, ACID-Garantien und geschützte Ausführungsumgebungen bereit.

#### 5.1 LSM-Tree Speicherkern (`contextra-store`)
* **Funktionen & Algorithmen:**
  - **LSM-Tree (Log-Structured Merge-Tree):** MemTable mit SkipList, unveränderliche SSTables (V4) mit hierarchischer Kompaktierung.
  - **2-Stufen Bloom-Filter:** Schnelle Nicht-Existenz-Prüfungen ohne Block-I/O.
  - **Write-Ahead-Log (WAL):** Crash-konsistentes Append-Only Log mit BLAKE3/HMAC-Integritätskette.
* **Architektur & Moduldesign:**
  - E1b-Baustein: Baut auf `contextra-mvcc`, `contextra-crypto`, `contextra-sys`, `contextra-ids` und `contextra-env` auf.
  - Der Kern wird von `contextra-types` und `contextra-ports` befreit.
  - Eigener Fehlertyp `StoreError` mit klarer Fehlerabbildung (Storage, Io, WalCorruption, ChecksumMismatch, Conflict).
  - Strikte Invariante INV-6: Persistenz- und Durability-Pfade bleiben unberührt.

#### 5.2 Transaktions-Isolation (`contextra-mvcc`)
* **Funktionen & Algorithmen:**
  - **Serializable Snapshot Isolation (SSI):** Mathematische Transaktionsvalidierung gegen Write-Skew und Anti-Dependencies.
  - **SequenceLog & ReadSet-Tracking:** Lock-freie Commit-Validierung.
* **Architektur & Moduldesign:**
  - E1a-Baustein: Hängt nur an `contextra-ids` und `contextra-env`.
  - Eigener Fehlertyp `MvccError`. Getestet unter extremen Concurrency-Bedingungen via Loom (`loom_sequence_log`, `loom_snapshot_registry`, `loom_tx_buffer`).

#### 5.3 Ausführungs-Isolation (`contextra-sandbox`)
* **Funktionen & Algorithmen:**
  - **WASM Fuel-Budgeting:** Deterministische Rechenschritt-Zählung (Fuel) zur Ausführung nicht vertrauenswürdiger Werkzeuge.
* **Architektur & Moduldesign:**
  - E0-Baustein (Score 100/100). Kann überall dort integriert werden, wo Plugin-Code oder Sandboxed Tool-Calling benötigt wird.

---

### Domäne 6: Ingest, Inferenz & Anwendungen

Bereitstellung von Parsing, Multi-Provider-Inferenz und standardisierten Schnittstellen.

#### 6.1 Dokumenten-Ingest (`contextra-ingest`)
* **Funktionen & Algorithmen:**
  - **Prose & Structure Chunking:** Formatbewusstes Parsing (Markdown, Text, optional PDF/DOCX) mit Breadcrumbs und semantischer Segmentierung.
* **Architektur & Moduldesign:**
  - E0-Baustein. Feature-gated nach Dokumentformaten (`fmt-text`, `fmt-pdf` etc.).

#### 6.2 Inferenz-Anbindung (`contextra-infer-candle`, `contextra-infer-onnx`, `contextra-infer-ollama`)
* **Funktionen & Algorithmen:**
  - **In-Process Inferenz & GGUF-Loader:** Candle- und ONNX-Kerne für lokale Embeddings und Textgenerierung.
  - **Remote LLM Client:** HTTP-Client mit Circuit-Breaker und Exponential-Backoff für Ollama / OpenAI-kompatible APIs.
* **Architektur & Moduldesign:**
  - Alle Inferenz-Crates bieten fallbacks (z. B. `FakeEmbedder` / `--no-default-features` in `contextra-infer-candle`), sodass Builds auch ohne CUDA/C-Libs lauffähig sind.
  - Eigene Fehlertypen (`CandleError`, `OllamaError`).

#### 6.3 Model Context Protocol (`contextra-mcp`)
* **Funktionen & Algorithmen:**
  - **JSON-RPC 2.0 / STDIO Server:** Standardisiertes MCP-Interface für Cursor, Claude Desktop und autonome Agenten.
* **Architektur & Moduldesign:**
  - E3-Anwendungs-Crate. Verdrahtet die darunterliegenden Bausteine (Privacy, Prompt-Guard, Engine, Retrieval) sauber nach außen.

---

## 4. Baukasten-Verwendungsbeispiele für Entwickler

### Beispiel 1: Isolierte HNSW-Vektorsuche in einem Drittprojekt

```rust
// In Cargo.toml des Drittprojekts:
// contextra-vector = { version = "0.3", default-features = false, features = ["simd"] }

use contextra_vector::hnsw::{HnswIndex, HnswConfig};
use contextra_vector::Metric;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = HnswConfig {
        max_elements: 10_000,
        m: 16,
        ef_construction: 64,
        metric: Metric::Cosine,
    };
    let index = HnswIndex::new(config)?;

    let vector_a = vec![0.1, 0.5, 0.9, -0.2];
    index.insert(1, &vector_a)?;

    let query = vec![0.12, 0.48, 0.88, -0.19];
    let hits = index.search(&query, 5, 32)?;
    for hit in hits {
        println!("Doc: {}, Score: {}", hit.id, hit.score);
    }
    Ok(())
}
```

### Beispiel 2: Autarkes PII-Vaulting & Prompt-Guard in einer API-Middleware

```rust
// In Cargo.toml:
// contextra-privacy = "0.2"
// contextra-prompt-guard = "0.1"

use contextra_privacy::dlp::DlpScanner;
use contextra_privacy::vault::SurrogateVault;
use contextra_prompt_guard::PromptGuard;

fn sanitize_user_input(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let guard = PromptGuard::default();
    guard.validate_prompt(prompt)?; // Abwehr von Injection-Angriffen

    let scanner = DlpScanner::default();
    let matches = scanner.scan(prompt);

    let mut vault = SurrogateVault::new();
    let sanitized_prompt = vault.mask_pii(prompt, &matches)?;

    Ok(sanitized_prompt)
}
```

---

## 5. Checkliste für zukünftige Baustein-Entwicklungen

- [ ] Ist der Crate in **Level E0 oder E1** platziert?
- [ ] Baut der Crate fehlerfrei mit `cargo check -p <crate> --no-default-features`?
- [ ] Sind keine `contextra-types` oder `contextra-ports` im Kern vorhanden (nur in `compat/`)?
- [ ] Verwendet der Crate `thiserror` mit einem lokalen Fehlertyp und `#[non_exhaustive]`?
- [ ] Sind alle persistierten Formate 버전iert und prüfen die ID-Breite (`id_width_bits`)?
- [ ] Wurden alle neuen Features nach dem Schema `exp-<name>` benannt und sind nicht in `default`?
- [ ] Wurden keine Compliance-Aussagen („DSGVO-garantiert“) im Code oder den Docstrings hinterlegt?
