# Contextra — Gesamtspezifikation (Fassung v17.0 Master)

**Stand:** 01.10.2026 / 02.10.2026
**Verantwortlich:** Principal Senior Rust Architect
**Status:** Normativ & Verbindlich (Einzige Master-Quelle der Wahrheit für das Contextra-Multi-Crate-Ökosystem)
**Code-Basis:** `https://github.com/tfufuz1/contextra`, HEAD-Stand (34 Crates in `crates/` + `xtask/`, Rust 1.89.0)

---

## Inhaltsverzeichnis
- [Teil 0 — Vision, Positionierung, Scope & Governance](#teil-0--vision-positionierung-scope--governance)
- [Teil 1 — Produktformen & Schnittstellen (IST & SOLL)](#teil-1--produktformen--schnittstellen-ist--soll)
- [Teil 2 — Multi-Crate-Architektur & Ring-Modell (Ring 0 bis Ring 4)](#teil-2--multi-crate-architektur--ring-modell-ring-0-bis-ring-4)
- [Teil 3 — Speicher-, Vektor-, Text-, Graph- & KV-Cache-Schichten](#teil-3--speicher--vektor--text--graph---kv-cache-schichten)
- [Teil 4 — Retrieval-Pipeline, Fusion, Kalibrierung & Adaptive Steuerung](#teil-4--retrieval-pipeline-fusion-kalibrierung--adaptive-steuerung)
- [Teil 5 — Sicherheit, Mandanten-Isolierung, Privacy & Kryptografischer Löschbeweis](#teil-5--sicherheit-mandanten-isolierung-privacy--kryptografischer-löschbeweis)
- [Teil 6 — Mikrofeingranularer Schnittstellen- & API-Katalog](#teil-6--mikrofeingranularer-schnittstellen---api-katalog)
- [Teil 7 — IST- vs. SOLL-Zustand: Lückenanalyse & Ursachen-Taxonomie (U1–U7)](#teil-7--ist--vs-soll-zustand-lückenanalyse--ursachen-taxonomie-u1u7)
- [Teil 8 — Gesamtroadmap & Migrationsplan (60-Tage-Roadmap)](#teil-8--gesamtroadmap--migrationsplan-60-tage-roadmap)

---

## Teil 0 — Vision, Positionierung, Scope & Governance

### 0.1 Ein-Satz-Definition
Contextra ist eine **eingebettete, air-gap-fähige Rust-Gedächtnis- und Wissensschicht für LLM-Agenten**: `cargo add contextra`, kein separater Server, kein zweiter Prozess. Sie liefert LLMs reproduzierbaren, auditierten, mandantengetrennten Kontext und kann Daten **kryptografisch nachweisbar löschen**.

### 0.2 Zielgruppe & Markt-Positionierung
Rust-Entwickler, die autonome KI-Agenten bauen, sowie Systemhäuser und Integratoren für regulierte Branchen im DACH-Raum (B2B2G / Enterprise Regulated). Relevant im Kontext des Vergabebeschleunigungsgesetzes und des BSI-C3A-Kriterienkatalogs (April 2026).

### 0.3 Produkt-Umfang (Contextra ist)
- Eine rein prozessinterne Rust-Bibliothek ohne externe Cloud-Abhängigkeiten im Kernlaufzeit-Pfad;
- Ein optionale MCP-Server (`contextra-mcp`, stdio-only) für IDEs und Agenten-Clients (Claude Desktop, Cursor, VS Code);
- Eine Engine mit kryptografisch verifizierbarer Löschung (Ed25519-`DeletionProof` v3, ohne Datenbankzustand extern verifizierbar);
- Eine hybride Retrieval-Engine (Sparse BM25/BM25F, Dense Vector HNSW/DiskANN, GraphRAG CSR Hyperedges, KV-Cache Context-Bridge);
- Ein strikt deterministisches System (Zeit und Zufall sind abstrahiert über die injizierbaren Ports `Clock` und `Rng`).

### 0.4 Scope-Ausschlüsse (❌, Verbindlich)
- **Tauri Desktop-App** (ADR-077: Im September 2026 entfernt; endgültig ausgeschlossen);
- **Multi-Node / Horizontales Sharding / Peer-to-Peer-Sync** (Contextra bleibt eine eingebettete Single-Node-Engine);
- **Framework-Adapter als Kernbestandteil** (LangChain, LangGraph, LlamaIndex werden nur als dünne Hilfs-Adapter in `contextra-adapters` gepflegt);
- **Interne Entwicklungs-Tools als Produktbestandteil** (Prompter-Dashboard, Jules-Harness sind CI/Governance-Infrastruktur).

### 0.5 Governance-Regeln & Prinzipien
1. **Unsafe Isolation**: `unsafe` Rust-Code ist strikt auf genau drei Unsafe Islands beschränkt: `contextra-simd` (SIMD-Instruktionen), `contextra-sys` (OS/mmap/Win32-ACL) und `contextra-wire` (FlatBuffers IPC). Alle anderen 31 Workspace-Crates erzwingen `#![forbid(unsafe_code)]` oder `#![deny(unsafe_code)]`.
2. **P28 Determinismus**: Kein direkter Aufruf von `SystemTime::now()` oder `rand::thread_rng()` im Logikcode. Zeit- und Zufallsoperationen erfolgen ausschließlich über `Arc<dyn Clock>` und `Arc<dyn Rng>`. Einzige Ausnahme ist die Initialisierung von unverhersehbarem kryptografischem Schlüssel- und Salt-Material (ADR-098).
3. **Ring-DAG-Integrität**: Abhängigkeiten fließen strikt von Ring 0 bis Ring 4. Zirkuläre Abhängigkeiten oder Rückwärts-Referenzen werden durch `cargo xtask check-ring-layering` und `tests/layering.rs` im CI-Gate blockiert.
4. **48h-Abhängigkeitsregel**: Keine neue externe Crate-Abhängigkeit ohne schriftliche Architektur-Begründung und 48 Stunden Wartezeit.
5. **Ein Benchmark, eine Wahrheit**: Performance-Angaben beziehen sich rein auf Speicher- und Index-Retrieval-Latenz (ohne externe LLM/Embedding-Netzwerk-Latenzen).

---

## Teil 1 — Produktformen & Schnittstellen (IST & SOLL)

### 1.1 Übersicht der Schnittstellen

| Produktform | Schnittstelle / Crate | Primäre Client-Zielgruppe | Status |
| :--- | :--- | :--- | :--- |
| **Rust Primary Facade** | `contextra` (`builder`, `open`, `open_with_config`) | Rust-Anwendungsentwickler | ✅ Stable |
| **AgentMemory Facade** | `contextra::AgentMemory` (`remember`, `recall`, `forget`, `relate`) | High-Level Agenten-Entwickler | ✅ Stable |
| **Low-Level Database Engine** | `contextra-db` (`Database`, `Collection`) | Systems-Engineers & Storage-Integratoren | ✅ Stable |
| **MCP Protocol Server** | `contextra-mcp` (stdio-only JSON-RPC 2.0) | Claude Desktop, Cursor, Agenten-Runtimes | ✅ Stable |
| **Framework Adapters** | `contextra-adapters` (LangChain, LangGraph, LlamaIndex) | Python/Rust Hybrid-Frameworks | 🟡 Experimental |
| **Python Bindings** | `contextra-py` (PyO3 `cdylib`) | Python Agent Runtimes | 🟡 Experimental |

### 1.2 Hauptfassade (`contextra`)
Die primäre Endanwender-Schnittstelle ist unter `<= 20 pub fn` gehalten:
- `contextra::builder(dim: usize) -> ContextraBuilder`: Erstellt den Fluent Builder;
- `contextra::open(path: impl AsRef<Path>) -> Result<Database>`: Öffnet eine Datenbank mit Standardkonfiguration;
- `contextra::open_with_config(path: impl AsRef<Path>, config: ContextraConfig) -> Result<Database>`: Öffnet eine Datenbank mit benutzerdefinierter Konfiguration.

---

## Teil 2 — Multi-Crate-Architektur & Ring-Modell (Ring 0 bis Ring 4)

Contextra ist in 34 Workspace-Crates unterteilt, die strikt in 5 Ringen (Ring 0 bis Ring 4) angeordnet sind.

```
Ring 4: [contextra] [contextra-agent] [contextra-mcp] [contextra-adapters] [contextra-bench] [contextra-py]
           │              │                   │
Ring 3: [contextra-engine] [contextra-cognition] [contextra-sandbox] [contextra-infer-onnx]
           │              │                   │
Ring 2: [contextra-store] [contextra-graph] [contextra-checkpoint] [contextra-router] [contextra-infer-candle] [contextra-infer-ollama]
           │              │                   │
Ring 1: [contextra-core] [contextra-kvcache] [contextra-privacy] [contextra-rank] [contextra-adapt] [contextra-license] [contextra-testkit] [contextra-audit-export] [contextra-avv-generator]
           │              │                   │
Ring 0: [contextra-types] [contextra-ports] [contextra-mvcc] [contextra-wire] [contextra-sys] [contextra-simd] [contextra-crypto] [contextra-vector] [contextra-text]
```

### 2.1 Ring 0 — Fundament & Invarianten
- **`contextra-types`**: Kanonische Domain-Typen (`DocId`, `TxId`, `TenantId`, `TenantScoped<T>`, `FusionWeights`, `FilterExpr`, `ContextraError`). `#![forbid(unsafe_code)]`.
- **`contextra-ports`**: Trait-Abstraktionen (`StorageEngine`, `VectorIndex`, `TextEmbeddingEngine`, `Clock`, `Rng`, `IdGen`, `MetricsSink`). `#![forbid(unsafe_code)]`.
- **`contextra-mvcc`**: In-Memory MVCC Sequence-Log & Snapshot-Registry. `#![forbid(unsafe_code)]`.
- **`contextra-wire`**: FlatBuffers Serialisierungs-Adapter (`WireBuffer`). **Unsafe Island 1** (Parsing-Grenzen).
- **`contextra-sys`**: Platform-Systemnahe Bindings (`mmap`, `mlock`, Win32-ACL). **Unsafe Island 2**.
- **`contextra-simd`**: Vektor-Distanzkerne (Cosine, L2, Dot) mit Laufzeit-AVX2/NEON-Dispatch. **Unsafe Island 3**.
- **`contextra-crypto`**: Schlüssel-Hierarchie, AEAD (AES-256-GCM-SIV), Audit-Chain, Ed25519 `DeletionProof` v3. `#![forbid(unsafe_code)]`.
- **`contextra-vector`**: HNSW & ACORN-Indexierung, DiskANN, PQ/SQ-Quantisierung. `#![forbid(unsafe_code)]`.
- **`contextra-text`**: BM25 & BM25F Inverted Index, Deutsche Morphologie. `#![forbid(unsafe_code)]`.

### 2.2 Ring 1 — Kernel-Dienste & Transformationen
- **`contextra-core`**: Kerndatenstrukturen, Invarianten-Prüfung, Tombstone-Bit-Disziplin.
- **`contextra-kvcache`**: Stufenmodell KV-Cache Storage & Offloading.
- **`contextra-privacy`**: `EgressGuard` L4 Bulk-Exfiltrationsschutz, Anonymisierung.
- **`contextra-rank`**: Reciprocal Rank Fusion (RRF k=60), Isotonische Kalibrierung (PAVA).
- **`contextra-adapt`**: PID Latency Controller, Bandit-Feedback.
- **`contextra-license`**: Deployment-gebundener `SignedLicenseGate` (`FeatureRing`).
- **`contextra-testkit`**: In-Memory `ReferenceModel`, Chaos-Matrix Fixtures.
- **`contextra-audit-export`**: GDPR Art. 30 Verarbeitungsverzeichnis & BSI TR-02102 Export.
- **`contextra-avv-generator`**: AVV Template Generator gemäß Art. 28 DSGVO.

### 2.3 Ring 2 — Persistenz & Subsysteme
- **`contextra-store`**: LSM-Tree Storage Engine, WAL Durability (AES-256-GCM-SIV), Compaction Engine, Adaptive Compaction.
- **`contextra-graph`**: CSR Graph-Engine, Hyperedge Support, GraphRAG Community Detection.
- **`contextra-checkpoint`**: Persistent Checkpoint Engine & Blake3 Manifest-Verifikation.
- **`contextra-router`**: Lyapunov Drift Control & Multi-Candidate Routing.
- **`contextra-infer-candle`**: Candle Local Inference Bridge.
- **`contextra-infer-ollama`**: Ollama External Provider Client.

### 2.4 Ring 3 — Execution & High-Level Services
- **`contextra-infer-onnx`**: ONNX Embeddings (`ort`) & Cross-Encoder Reranking.
- **`contextra-sandbox`**: WASM Execution Boundary & Isolations-Sandbox.
- **`contextra-engine`**: Compute Pool, Async Storage Execution Layer, Collection Query Builder.
- **`contextra-cognition`**: Background Sleep & Memory Compactor Passes.

### 2.5 Ring 4 — Facades & Client-Integrationen
- **`contextra-db`**: Main Database Abstraction (`Database`, `Collection`).
- **`contextra`**: Enduser Library Facade.
- **`contextra-agent`**: Agent Execution Pipeline & Audit Engine.
- **`contextra-mcp`**: Model Context Protocol stdio Server & Safety Boundary.
- **`contextra-adapters`**: Framework-Integrationen (LangChain, LangGraph, LlamaIndex).
- **`contextra-bench`**: Benchmark Suite & Reproducibility Harness.
- **`contextra-py`**: PyO3 Python Bindings (`cdylib`).

---

## Teil 3 — Speicher-, Vektor-, Text-, Graph- & KV-Cache-Schichten

### 3.1 LSM-Tree & WAL Persistenz (`contextra-store`)
- **WAL-HMAC & Verschlüsselung**: Jedes WAL-Segment wird per AES-256-GCM-SIV verschlüsselt und mit HMAC-SHA256 gesichert.
- **Legacy Migration**: Replay von Legacy WAL-Segmenten mit dem statischen Fallback-Schlüssel erfordert die explizite Methode `Wal::open_for_legacy_migration()`, welche `allow_legacy_integrity_key_fallback = true` setzt und eine `tracing::warn!` Warnung ausgibt (INV-WAL-LEGACY-KEY-1).
- **Adaptive Compaction**: `CostBasedAdaptivePlanner` bewertet Workload-Muster über deterministische Operationszähler und Transaktions-Sequenznummern (P28-konform). Tombstones, die von aktiven Snapshots referenziert werden (`SnapshotRegistry::min_active_seqno()`), bleiben unangetastet (INV-COMPACTION-ADAPTIVE-1).

### 3.2 Vektor-Indexierung & Nachbarschaftsreparatur (`contextra-vector`)
- **HNSW & ACORN**: HNSW ermöglicht schwach-skalierende NNS-Suchen. ACORN (`FilteredIndex`) skaliert die Kanten-Budgets dynamisch nach Invers-Selektivität ($\gamma$-Budgeting).
- **Löschung & Nachbarschaftsreparatur (ADR-097)**: Bei Dokument-Löschungen wird der HNSW-Graph im Betroffenen Layer 2-phasig repariert (Rewiring), anstatt nur gelöschte Knoten zu maskieren (Tombstone Pruning). Dadurch wird Recall-Kollaps bei hohen Lösch-Raten verhindert.

### 3.3 Text-Indexierung & Morphologie (`contextra-text`)
- **BM25 / BM25F Engine**: Unterstützt strukturierte Feld-Gewichtung (BM25F) über das Cargo-Feature `bm25f`.
- **Wörterbuch-Ressourcen**: Eingebettete Fachvokabulare (`data/legal_de.txt`, `data/medical_de.txt`) werden beim Start über `include_str!` geladen und in-memory verarbeitet. Formatterzugene Abschnitte (`[compound_stems]`, `[protected_terms]`) erzwingen alphabetische Sortierung und strikte Disjunktheit.

### 3.4 Wissensgraph & CSR-Hyperkanten (`contextra-graph`)
- **CSR-Struktur**: `CsrGraph` veraltet adjazente Kanten in komprimierten Sparse-Row-Arrays für speichereffiziente $O(1)$-Traversierung.
- **Hyperkanten & Rollen**: `relate_n_ary` verbindet N Entitäten mit Rollen-Bindungen (`RoleBinding`) über automatisch inkrementierte `HyperEdgeId`s.

### 3.5 KV-Cache Bridge & Crypto-Shredding (`contextra-kvcache`)
- **Crypto-Shredding**: KV-Cache-Segmente werden über abgeleitete Sub-Keys (HKDF-SHA256 über Master-Key, `DEFAULT_SHRED_KEY_GROUP_SIZE = 64`) verschlüsselt. Bei Löschung wird der Sub-Key in `KeyRegistry` entzogen (`revoke_subkey`).

---

## Teil 4 — Retrieval-Pipeline, Fusion, Kalibrierung & Adaptive Steuerung

### 4.1 Hybrid Retrieval Architecture
Das Retrieval führt bis zu 4 Signale zusammen:
1. **Dense Vector Signal**: HNSW / DiskANN Ähnlichkeitsscore;
2. **Sparse Text Signal**: BM25 / BM25F Relevanzscore;
3. **Graph Community Signal**: GraphRAG PPR / Community Density Boost;
4. **KV-Cache Re-use Signal**: Dynamic Context Reuse.

```
Query ──┬──> Vector Index Search ──┐
        ├──> Text BM25 Search    ──┼──> RRF Fusion (k=60) ──> Isotonic PAVA ──> PID Pool Sizing ──> Top-K Results
        └──> Graph Traversal     ──┘
```

### 4.2 Reciprocal Rank Fusion (RRF) & Kalibrierung (`contextra-rank`)
- **RRF Constant**: RRF verschmilzt Einzel-Signale mit der kanonischen Konstante $k = 60.0$:
  $$RRF\_Score(d) = \sum_{s \in Signals} \frac{w_s}{60 + Rank_s(d)}$$
- **Isotonische Kalibrierung**: Verwendet den Pool-Adjacent-Violators-Algorithmus (PAVA), um Fusions-Scores in verlässliche Wahrscheinlichkeitswerte zu transformieren.

### 4.3 Adaptive Kandidatenpool-Skalierung (`contextra-adapt`)
- **PID Controller**: Passt die Pre-Reranking-Kandidatenpoolgröße dynamisch im Intervall $[50, 200]$ an, basierend auf den p95-Retrieval-Latenzmessungen des `Clock`-Ports.
- **Signal Failure Policy**:
  - `SignalFailurePolicy::Fail` (Standard): Bricht die gesamte Abfrage ab, wenn ein Signal ausfällt;
  - `SignalFailurePolicy::Degrade`: Überspringt fehlgeschlagene Signale, liefert partielle Ergebnisse zurück und befüllt `SearchReport::degraded_signals`.

---

## Teil 5 — Sicherheit, Mandanten-Isolierung, Privacy & Kryptografischer Löschbeweis

### 5.1 Type-Level Mandanten-Isolierung (`contextra-types`)
- **`TenantScoped<T>`**: Bindet Datenwerte typ-sicher an eine `TenantId`. Entpackung über `into_inner_checked(&expected_tenant)` gibt `T` nur bei exakter Übereinstimmung frei. Bei Mismatch wird `TenantScopeViolation::Mismatch` zurückgegeben, ohne den gekapselten Wert im Speicher freizugeben.

### 5.2 Egress Guard Bulk-Exfiltrationsschutz (`contextra-privacy`)
- **`EgressGuard`**: Überwacht ausgehende Antworten auf L4-Ebene. Standard-Schwellenwert: Similarity 0.85, Timeout: 200 ms, Min-Payload-Schwelle: 128 Bytes. Bei Ausfällen oder Timeouts greift Fail-Closed (`BlockReason::InternalError` bzw. `BlockReason::ClassificationTimeout`). Jede Entscheidung erzeugt einen auditierten `EgressClassifierTrace` (INV-EGRESS-AUDIT-1).

### 5.3 Kryptografischer Löschbeweis (`contextra-crypto`)
- **`DeletionProof` v3**: Nachweis über die physische Vernichtung von Daten in allen 7 Schichten (LSM SSTables, WAL, Vector Index, Inverted Index, Graph CSR, KV-Cache, In-Memory Snapshots).
- **Zustandsfreie Verifikation**: Dritte können die Ed25519-Signatur eines `DeletionProof` mittels `verify()` prüfen, ohne Zugriff auf den Datenbankzustand zu benötigen. Enthält eine optionale `audit_chain_position` Referenz auf die fälschungssichere Blake3 `AuditChain`.

### 5.4 Key Governance & Revocation Log (`contextra-crypto`)
- **`RevocationLog`**: Persistent, Ed25519-signiertes, append-only Log (v17 Teil 16.1) mit SHA-256-Hash-Kette. `KeyRegistry` verweigert Operationen auf widerrufenen Schlüsseln mit `CryptoError::KeyRevoked`.

---

## Teil 6 — Mikrofeingranularer Schnittstellen- & API-Katalog

### 6.1 Ring 0 Trait APIs (`contextra-ports`)

```rust
#[async_trait]
pub trait StorageEngine: Send + Sync {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    async fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    async fn delete(&self, key: &[u8]) -> Result<()>;
    async fn scan_prefix(&self, prefix: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
}

pub trait VectorIndex: Send + Sync {
    fn search(&self, vector: &[f32], k: usize) -> Result<Vec<(DocId, f32)>>;
    fn insert(&self, doc_id: DocId, vector: &[f32]) -> Result<()>;
    fn delete(&self, doc_id: DocId) -> Result<()>;
}

pub trait Clock: Send + Sync {
    fn now_unix_nanos(&self) -> u64;
}

pub trait Rng: Send + Sync {
    fn next_u64(&self) -> u64;
}
```

### 6.2 High-Level Agent Memory Facade (`contextra::AgentMemory`)

```rust
impl AgentMemory {
    pub async fn remember(&self, content: &str, metadata: Metadata) -> Result<DocId>;
    pub async fn recall(&self, query: &str, k: usize) -> Result<Vec<ScoredDocument>>;
    pub async fn forget(&self, doc_id: DocId) -> Result<DeletionProof>;
    pub async fn relate(&self, source: EntityId, target: EntityId, rel_type: &str) -> Result<()>;
}
```

### 6.3 MCP Protocol Tools (`contextra-mcp`)
All write/delete operations enforce strict explicit confirmation parameters (`confirm: true` as boolean):
- `contextra_search`: Dynamic hybrid retrieval (`DatabaseRead`);
- `contextra_get`: Key lookup (`DatabaseRead`);
- `contextra_insert` / `contextra_upsert`: Key-value insertion (`DatabaseWrite`);
- `contextra_delete`: Specific key deletion (`DatabaseWrite`);
- `contextra_forget`: Document deletion with proof generation (`DatabaseWrite`, requires `confirm: true`);
- `contextra_create_collection`: Collection creation (`DatabaseWrite`);
- `contextra_drop_collection`: Collection drop (`DatabaseWrite`, requires `confirm: true`).

---

## Teil 7 — IST- vs. SOLL-Zustand: Lückenanalyse & Ursachen-Taxonomie (U1–U7)

### 7.1 Ursachen-Taxonomie (Root Cause Classification)
- **U1 (Spezifikationsabweichung)**: Code implementiert andere Schnittstelle als Alt-Spezifikation;
- **U2 (Asynchronitäts-/Concomitant-Bug)**: Race-Condition oder Lock-Order Violation;
- **U3 (Sicherheits- / Isolation-Lücke)**: Ungeschützte Entpackung oder Fehlen von Fail-Closed;
- **U4 (Leistungs- / Speicher-Regression)**: Unbeabsichtigter O(N)-Scan oder Re-Allocations;
- **U5 (Determinismus-Verletzung)**: Direkte Nutzung von System-Uhr oder OS-Entropy;
- **U6 (Veraltete Test-Infrastruktur)**: Mismatch zwischen Mocks und Crate-Signatur;
- **U7 (Dokumentations- / Capability-Drift)**: Inkompatibilität zwischen `capabilities.toml` und Code.

### 7.2 Lücken-Katalog & Priorisierung

| Subsystem / Crate | Beschreibung der Lücke | Ursache | Priorität | Soll-Zustand |
| :--- | :--- | :---: | :---: | :--- |
| `contextra-engine` | Doppelte Felddeklaration `on_signal_failure` in `HybridQueryBuilder` | U1 | **P0** | Vereinheitlichung auf `OnSignalFailure` / `SignalFailurePolicy` |
| `contextra-store` | SSTable-`mmap` Null-Copy Path zeigt `UNIMPLEMENTED` Marker | U4 | **P1** | Vollständige `mmap`-Integration über `contextra-sys` |
| `contextra-crypto` | Argon2id Key Derivation Header `MFKD` v1 Migration ausstehend | U3 | **P1** | Erzwingung des `MFKD` v1 Binary Headers |
| `contextra-vector` | DiskANN Staging Buffer Flush unter hoher Ingestion-Last | U4 | **P2** | Asynchroner Background-Flush mit Credit-Backpressure |
| `contextra-mcp` | `contextra_plugin_status` gibt leeres Default-Registry-Metadata zurück | U7 | **P2** | Anbindung an echtes `PluginRegistry` aus `contextra-ports` |

---

## Teil 8 — Gesamtroadmap & Migrationsplan (60-Tage-Roadmap)

```
[Tag 1-15: Core Stabilization] ──> [Tag 16-30: BSI Compliance] ──> [Tag 31-45: SOTA Retrieval] ──> [Tag 46-60: Final Launch]
```

### 8.1 Meilensteine

#### Phase 1: Core Stabilization & API Harmonisierung (Tage 1–15)
- [x] Bereinigung aller Typparitätsprobleme (`OnSignalFailure` vs `SignalFailurePolicy`);
- [x] Durchsetzung der Ring-DAG-Architektur in allen 34 Crates;
- [x] Abdeckung aller Unsafe Islands mit expliziten Verification Tests.

#### Phase 2: BSI TR-02102 Compliance & Crypto-Hardening (Tage 16–30)
- [ ] Abschluss der Argon2id `MFKD` v1 Header-Migration in `contextra-crypto`;
- [ ] Vollständige Audit-Chain Blake3 Head Signatur-Verifikation im `contextra-audit-export`;
- [ ] Validierung der DSGVO Art. 30 Verarbeitungsverzeichnis-Generierung.

#### Phase 3: SOTA Retrieval & Performance Optimierung (Tage 31–45)
- [ ] Fertigstellung des SSTable Zero-Copy `mmap` Readers in `contextra-store`;
- [ ] Dynamische Tuning-Optimierung des PID Latency Controllers unter 100k Vektor-Skalierung;
- [ ] HNSW Nachbarschaftsreparatur-Performance-Benchmarking unter kontinuierlicher Löschlast.

#### Phase 4: Final Launch & Public Documentation (Tage 46–60)
- [ ] 100%ige Abdeckung der `doc-truth` Prüf-Gates über das gesamte Repository;
- [ ] Veröffentlichung des reproduzierbaren Benchmark-Reports (`contextra-bench`);
- [ ] Freigabe der v1.0.0 Stabil-Spezifikation und Crate Release.
