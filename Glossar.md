# Contextra — Projekt-Glossar

> Umfassendes Fach-, Architektur- und Codebase-Glossar für **Contextra** (Embedded Hybrid-Search & LLM-natives, versioniertes Storage-System in Rust).  
> Umfasst sowohl die **aktuell im Workspace implementierten Crates (Ring 0–4)** als auch die **normativen Architekturbegriffe der Spezifikation (L0–L6, v2.1)**.

---

## Inhaltsverzeichnis

1. [Schichten- und Crate-Übersicht](#1-schichten--und-crate-übersicht)
2. [Alphabetisches Glossar (A–Z)](#2-alphabetisches-glossar-a-z)
   - [A](#a) · [B](#b) · [C](#c) · [D](#d) · [E](#e) · [F](#f) · [G](#g) · [H](#h) · [I](#i) · [K](#k) · [L](#l) · [M](#m) · [N](#n) · [O](#o) · [P](#p) · [Q](#q) · [R](#r) · [S](#s) · [T](#t) · [U](#u) · [V](#v) · [W](#w) · [X](#x) · [Z](#z)

---

## 1. Schichten- und Crate-Übersicht

Contextra gliedert sich im aktuellen Codebase-Layout in **Ring 0–4** (plus Adapter/Werkzeuge) und in der Zielarchitektur (`SPEC-01`) in die **Schichten L0–L6**:

| Ring / Schicht | Crates | Verantwortung |
|---|---|---|
| **Ring 0 (Fundament & Unsafe Islands)** | `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-sys`, `contextra-simd`, `contextra-wire`, `contextra-core` *(deprecated Facade)* | Kanonische Typen/IDs, Port-Traits, MVCC/SSI, Low-Level-System-FFI, SIMD-Distanzkerne, Zero-Copy FlatBuffers IPC |
| **Ring 1 (Storage, Durability & Cache)** | `contextra-durable-fs`, `contextra-store`, `contextra-checkpoint`, `contextra-kvcache`, `contextra-crypto` | Crash-sichere Dateisystem-Ops, LSM-Tree (`MemTable`/`SSTable`/WAL/Compaction), Snapshots/Checkpoints, KV-Block-Cache & Prefix-Radix, AEAD-Verschlüsselung |
| **Ring 2 (Indizes & Retrieval-Signale)** | `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-rank`, `contextra-adapt` | HNSW/DiskANN/ACORN-Vektorindex, BM25/BM25F/FST-Volltext, CSR-Graph & PPR, 4/5-Signal-Fusion & Kalibrierung, PID/Bandit/Drift-Controller |
| **Ring 3 (Engine, Kognition, Privacy & Agenten)** | `contextra-engine`, `contextra-cognition`, `contextra-agent`, `contextra-privacy`, `contextra-router`, `contextra-sandbox`, `contextra-license` | Storage-/Index-/Txn-Orchestrierung, Konsolidierung, persistente Agent-Workflows, Cloud-Egress/DLP, SLM/LLM-Routing, WASM-Sandbox, Lizenz-Gate |
| **Ring 4 (Fassade, Ingest, Inferenz & Schnittstellen)** | `contextra`, `contextra-db`, `contextra-ingest`, `contextra-mcp`, `contextra-py`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-audit-export`, `contextra-avv-generator` | Öffentliche Rust-Fassade, Embedded DB, Dokument-Extraktoren/Chunker, MCP-Server, PyO3-Bindings, lokale/externe Inferenz-Backends, Compliance-Exporte |
| **Test, Bench & Werkzeuge** | `contextra-testkit`, `contextra-bench`, `xtask`, `xtask-heavy` | Deterministische Test-Utilities (`ManualClock`, `FaultVfs`), Retrieval-Benchmarks, CI-/Audit-/Verify-Tasks |

---

## 2. Alphabetisches Glossar (A–Z)

### A

- **ACORN** (`contextra-vector`): Algorithmus für gefilterte HNSW-Vektorsuche (*Filtered ANN*), der Prädikat-Filter direkt während der Graph-Traversierung berücksichtigt (`FilteredIndex`, `AcornError`), statt nur Post-Filtering anzuwenden.
- **Adaptive Decay / Controller** (`contextra-adapt`): Regelkreis (`AdaptiveDecayController`, `DecayControllerConfig`), der anhand von `DecaySignalInputs` den zeitlichen Relevanzverfall (Memory Decay) dynamisch anpasst.
- **AEAD (AES-256-GCM-SIV)** (`contextra-crypto`): *Authenticated Encryption with Associated Data* — Nonce-missbrauchsresistente Verschlüsselung at Rest für WAL, SSTables, Packs und KV-Caches.
- **AgentMemory** (`contextra`): High-Level-Fassadentyp für strukturiertes Agenten-Gedächtnis mit `MemoryId`, `RelationId` und hybrider Abfrage.
- **APPRH (Adaptive Personalized PageRank Heuristic)** (`contextra-graph`): Adaptive Steuerung für *Personalized PageRank*-Traversierungen (`ApprhSelector`, `ApprhGateMonitor`, `ApprhFlipGate`), um zwischen exakter und heuristischer Graph-Expansion umzuschalten.
- **AuditChain / Art. 30 Export** (`contextra-crypto`, `contextra-audit-export`): Kryptographisch verkettete Audit-Protokollierung (`AuditChain`, `AuditChainEntry`) sowie Export von Verarbeitungsverzeichnissen nach DSGVO Art. 30 (`AuditRegisterExporter`). *(Hinweis: Gemäß Zielarchitektur `SPEC-01` §1.4 / `C-15` entfallen DSGVO-Löschketten im Kern zugunsten von `retire`/`purge`.)*
- **AVV-Generator** (`contextra-avv-generator`): Generator für Auftragsverarbeitungsvertrag-Vorlagen (`AvvContext`, `TechnicalMeasure`), die technische Garantien von Contextra referenzieren.

### B

- **Bandit-Routing** (`contextra-adapt`, `contextra-router`): Multi-Armed-Bandit-Verfahren (u. a. Contextual Bandits mit `SketchMatrix`, Thompson Sampling, `egress-sherman-morrison`) zur dynamischen Modell- und Pfadauswahl.
- **BeliefBeta** (`SPEC-02` §2.6): Evidenzbasierte Konfidenzrepräsentation als Beta-Verteilung `Beta(α, β)` mit `source_root`-Deduplizierung (Schutz vor Echokammern) und 10-%-Lower-Confidence-Bound (LCB).
- **Bitemporalität** (`SPEC-02`, `contextra-types`): Trennung zweier Zeitachsen pro Fakt: Gültigkeitszeit (*Valid Time*: `valid_from`, `valid_to`) in der realen Welt und Transaktionszeit (*Transaction Time*: `tx_time`) im System.
- **BLAKE3 (Keyed Hash)** (`contextra-crypto`, `SPEC-03` §8.3): Kryptographische Hashfunktion für Content-Addressing (`ChunkId`). In verschlüsselten Repositories wird `BLAKE3::keyed_hash(repo_hash_key, plaintext)` über kanonischen Klartext statt über Nonce-Ciphertext berechnet, um Cross-Repo-Orakel zu verhindern und Deduplizierung zu erhalten.
- **Block-Max WAND (BMW)** (`contextra-text`, `SPEC-05` §3): Effizienter Top-k-Retrieval-Algorithmus für invertierte Indizes, der Block-Obergrenzen nutzt, um irrelevante Postings-Blöcke zu überspringen.
- **BM25 / BM25F / BM25MorphIndex** (`contextra-text`): Probabilistische Volltext-Relevanzbewertung (Signal 2 in der Hybrid-Suche). `BM25F` gewichtet Felder (`FieldId`, `FieldWeight`) mit globalen Korpus-Statistiken; `BM25MorphIndex` berücksichtigt morphologische Varianten.
- **Buffer Pool (W-TinyLFU)** (`SPEC-03`, `contextra-store`): Partitionierter In-Memory-Seitencache mit *Windowed TinyLFU*-Eviction-Strategie zum Schutz vor Scan-Pollution.

### C

- **Capability / `capabilities.toml`** (`contextra-types`, `contextra-sandbox`, `SPEC-07`): Deklaratives Berechtigungsmodell für Tools, MCP-Endpunkte und Speicheroperationen (z. B. `WasmCapabilities`, `MergeOperatorCapabilities`).
- **Catoni Drift Detector** (`contextra-adapt`, `contextra-rank`): Robuster statistischer Drift-Detektor (`CatoniDriftDetector`, `DriftDetector`) auf Basis des Catoni-M-Schätzers zur Erkennung von Score- oder Verteilungs-Drift.
- **Cell (`CellContent`, `CellMeta`, `CellId`)** (`SPEC-02` §2.2): Entkoppelte Grund-Wissenseinheit in Contextra. Trennt den unveränderlichen Rohinhalt (`CellContent`, identifiziert durch `content_hash`) von leichtgewichtigen, versionierten Metadaten (`CellMeta`: Attribute, Trust, Taint, Belief, Gültigkeitszeit) und Graph-Kanten.
- **CellLocator vs. ObjectLocator** (`SPEC-03` §3): Zweistufige Adressierung physischer Daten: `CellLocator` adressiert `(PackId, ExtentNo, RowIndex)` pro Cell in PAX-Extents; `ObjectLocator` adressiert `(PackId, Offset, Len)` für Prolly-Knoten, Blobs und Segmente.
- **Checkpoint & PinGuard** (`contextra-checkpoint`): MVCC-basiertes Snapshotting und Backup-Management (`CheckpointManifest`, `StateCheckpoint`, `CheckpointHardlinkCloner`). `PinGuard` verhindert die Garbage Collection aktiv genutzter Snapshots.
- **ChunkId / Chunker** (`contextra-ingest`, `contextra-types`, `SPEC-02`): Inhaltsadressierte ID eines Text-/Datenabschnitts (`ChunkId`). Strukturierte Zerlegung erfolgt formatabhängig in `contextra-ingest` (z. B. `MarkdownChunker`, PDF/HTML/CSV/Mail-Extraktoren).
- **Circuit Breaker** (`contextra-infer-ollama`): Resilienz-Muster (`CircuitBreaker`, `BreakerStatus`) zur Absicherung externer Inferenz-Aufrufe gegen kaskadierende Ausfälle.
- **Cognition / Consolidation** (`contextra-cognition`): Hintergrund-Konsolidierung von Erinnerungen (`ConsolidationEngine`, `AggregationNode`, `SuperEdgeDraft`), Komprimierung und Verdichtung von Wissensgraphen.
- **CollectionProfile** (`contextra`): Vorkonfiguriertes Profil für eine Sammlung (`CollectionProfile`, `DeploymentTier`, `PerformanceProfile`), das Speicherbudgets und Index-Parameter bündelt.
- **Compaction (Adaptive / Leveled / Tiered)** (`contextra-store`): Hintergrundprozess im LSM-Tree (`CompactionEngine`, `AdaptiveCompactionPlanner`, `CostBasedAdaptivePlanner`), der SSTables zusammenführt, Tombstones bereinigt und Leseverstärkung reduziert.
- **Conformal Calibration** (`contextra-rank`, `contextra-router`): Kalibrierung von Retrieval- und Routing-Scores mit statistischen Konfidenzgarantien (`ConformalCalibrator`, `AdaptiveConformalCalibrator`, `IsotonicCalibrator`, `PlattScaler`).
- **ConflictCell** (`SPEC-04` §6.2): Explizite, versionierte Darstellung eines Merge-Konflikts auf einem Merge-Request-Branch (`mr/*`). Auf geschützten Branches (`main`) dürfen niemals unaufgelöste `ConflictCell`s existieren (`INV-7`).
- **ContextraBuilder** (`contextra`): Zentraler Builder zum Öffnen und Konfigurieren einer eingebetteten Contextra-Datenbankinstanz.
- **CSR-Graph (Compressed Sparse Row)** (`contextra-graph`): Speicher- und cache-effiziente Repräsentation des Entity-Relation-Graphen (Signal 3 der Hybrid-Suche) für schnelle Nachbarschafts-Traversierung und PPR.

### D

- **DataClass** (`contextra-crypto`): Klassifizierung von Daten nach Schutzbedarf für Schlüsselwahl, Verschlüsselung und Egress-Regeln.
- **DeletionProof** (`contextra-crypto`, `contextra-audit-export`): Kryptographischer Löschnachweis (`DeletionProofKeyPair`, `DeletionProofSummary`) im bestehenden Compliance-Code. *(In der Zielarchitektur `SPEC-01` §1.4 ausdrücklich durch `retire` + `purge` ersetzt.)*
- **Derivation Cache** (`SPEC-06` §7.1): Zweigübergreifend memoisierter Cache für abgeleitete Artefakte (Embeddings, Token-Counts, Summaries) indiziert per `(content_hash, producer_id)`.
- **DiBud-Fusion** (`contextra-rank`): *Budgeted Channel Fusion* (`DiBudFusionState`, `BudgetedChannel`, `FusionBudget`) zur Kombination mehrerer Retrieval-Signale unter expliziten Latenz-/Kostenbudgets.
- **DiskANN / Vamana** (`contextra-vector`, `SPEC-05` §2): SSD-gestützter Graph-Vektorindex für große Datenmengen (`DiskAnnConfig`, `DiskAnnFallbackPolicy`), kombinierbar mit komprimierten Vektoren im RAM.
- **DLP (Data Loss Prevention)** (`contextra-privacy`): Schutzmechanismus vor unbefugtem Datenabfluss bei Cloud-Egress (`BulkExfiltrationDetector`, `InjectionDetector`, `DefaultEgressClassifier`).
- **DocId (32-bit vs. `docid-128`)** (`contextra-types`, `contextra-vector`, `contextra-graph`): Segmentlokaler Dokument-Bezeichner (`u32` bzw. optional 128-bit via Feature-Flag `docid-128`), verwendet in Roaring-Bitmaps und Postings-Listen.
- **DurableFs** (`contextra-durable-fs`, `contextra-sys`): Abstraktion für crash-sichere Dateisystemoperationen (`DurableFile`, `JournalSyncPolicy`, `RecordRecoveryPolicy`) inkl. `fsync`/`fdatasync`-Semantik und Verzeichnis-Sync.

### E

- **Edge Reinforcement Learning** (`contextra-graph`, Feature `physio-synaptic-edges` / `edge-reinforcement-learning`): Verstärkungslernen auf Graph-Kanten zur Anpassung von Kantengewichten anhand erfolgreicher Retrieval-Pfade.
- **Egress-Guard** (`contextra-privacy`, `contextra-crypto`, `SPEC-07`): Prüfschicht (`EgressGuardCheck`), die ausgehende Datenströme (z. B. an Cloud-LLMs) klassifiziert, filtert und gegen Exfiltration sowie Prompt-Injection absichert.
- **Embedder / EmbeddingProvider** (`contextra-ports`, `contextra-infer-onnx`, `contextra-infer-candle`, `contextra-infer-ollama`): Port-Trait (`TextEmbeddingEngine`, `EmbeddingProvider`) und konkrete Backends zur Berechnung dichter Vektor-Embeddings.
- **EvidenceSufficiency & Abstention** (`SPEC-06` §5): Bewertung der Beleglage einer Query (`Sufficient`, `Thin`, `Conflicting`, `None`). Bei `None` liefert die Engine `NO_EVIDENCE_FOUND` (Abstention), statt Halluzinationen zu erzwingen.
- **Extent (PAX)** (`SPEC-03` §2.2): 256 KiB–1 MiB große I/O-Einheit im Pack-Format (`.cxp`). Organisiert Daten spaltenorientiert (*Partition Attributes Across*) in einem kompakten `Hot-Head` (für Ranking) und einem `Cold-Tail` (Volltexte, f16-Vektoren).

### F

- **FaultVfs / FaultInjector** (`contextra-durable-fs`, `contextra-testkit`): Deterministische Fehlerinjektion (`FaultInjectingVfs`, `FaultConfig`, `InjectedFault`) zur Simulation von Stromausfällen, I/O-Fehlern und Crash-Recovery.
- **FlatBuffers IPC** (`contextra-wire`): Zero-Copy-Serialisierungsformat für Inter-Prozess-Kommunikation und FFI-Grenzen (`WireBuffer`, `VerifiableRoot`, `Embedding`, `ScoredDocument`).
- **FST (Finite State Transducer)** (`contextra-text`, `SPEC-05` §3): Speicherplatzsparender Präfix-/Term-Index für schnelle Wörterbuch-Lookups und Fuzzy-/Präfix-Suche im Volltext-Subsystem.
- **Fusion (4-/5-Signal-Fusion, RRF)** (`contextra-rank`, `contextra-db`, `contextra-engine`): Kombination heterogener Retrieval-Signale (Vektor, BM25/Volltext, CSR-Graph, zeitlicher Decay/Metadaten) mittels *Reciprocal Rank Fusion* (RRF), gewichteter Kombination oder `DiBud`.

### G

- **GASP** (`contextra-infer-candle`): Validator (`GaspConfig`, `GaspValidator`) im Candle-Inferenz-Backend für strukturierte bzw. validierte Ausgaben.
- **GGUF** (`contextra-infer-candle`): Quantisiertes Modellformat für lokale LLM-Inferenz über Candle (`GgufMetadata`, `QuantizedLlamaModel`).
- **Gid (Global Dense ID)** (`SPEC-02` §2.1): Global eindeutige, dichte und monotone `u64`-ID pro Cell für gemeinsame Bitmaps über Vektor-, Volltext- und Graph-Indizes hinweg.
- **GroundingLevel** (`SPEC-02` §2.6, `contextra-cognition`): Belegstufe einer Aussage von `G0` (direkt verifizierte Textspanne) bis `G3` (unbelegte Hypothese / Quarantäne).
- **Group Commit** (`SPEC-07` §1, `contextra-store`): Bündelung mehrerer gleichzeitiger Transaktions-Commits in einem gemeinsamen `fsync`-Aufruf des WAL für hohen Schreibdurchsatz.

### H

- **Handle (MAC-geschützt)** (`SPEC-02` §5): Token-sparsamer, für LLMs gedachter Kurzbezeichner im Format `c:<base32>-<mac2>` (z. B. `c:k7qd-3f`). Das 10-Bit-BLAKE3-MAC-Tag über `(repo_seed, gid)` schließt zufällige Treffer durch LLM-Halluzinationen zu 99,90 % aus.
- **HLC (Hybrid Logical Clock)** (`contextra-types`, `contextra-ports`, `SPEC-01`): Kombination aus physischer Zeit und logischem Zähler zur deterministischen Kausalitäts- und Zeitstempel-Ordnung.
- **HNSW (Hierarchical Navigable Small World)** (`contextra-vector`): Mehrschichtiger Proximity-Graph für approximative Nächste-Nachbarn-Suche (ANN) im RAM (Signal 1 der Hybrid-Suche).
- **Hot-Head / Cold-Tail** (`SPEC-03` §2.2): Layout-Prinzip in PAX-Extents: Häufig für Filterung und Ranking gelesene Spalten liegen vorne (`Hot-Head`), große Payloads wie Volltext hinten (`Cold-Tail`).

### I

- **Idempotenz-Journal (Sidecar)** (`SPEC-04` §8, `INV-6`): Nicht-versionierte, WAL-gesicherte Tabelle (`sidecar/idempotency.cxs`), die wiederholte Schreibanfragen mit gleichem `IdempotencyKey` und Payload-Hash dedupliziert.
- **IndexManifestRef** (`SPEC-04` §2, `SPEC-05` §1): Separater Zeiger auf physische Index-Segmente (Vamana, FTS, CSR) inkl. `source_truth_hash` und `watermark_seq`. Bewusst vom deterministischen `RootSet`-Hash getrennt (`INV-1`, `INV-4`).
- **Ingest** (`contextra-ingest`): Subsystem zur Dokumentextraktion und strukturtreuen Zerlegung (Markdown, PDF via `lopdf`, Office/XML/ZIP, HTML via `html5ever`, CSV, E-Mail via `mail-parser`, YAML/TOML).
- **InjectionDetector** (`contextra-privacy`): Detektor zur Erkennung von Prompt-Injection-Mustern in unstrusted Dokumenten oder Nutzereingaben.
- **Isotonic Calibrator** (`contextra-rank`): Nicht-parametrische, monotone Kalibrierung von Ähnlichkeits-Scores auf Wahrscheinlichkeiten (`IsotonicCalibrator`).

### K

- **KEK / DEK** (`SPEC-03` §8.1, `contextra-crypto`): Zweistufige Schlüsselhierarchie aus *Key Encryption Key* (KEK) und *Data Encryption Key* (DEK), die eine O(1)-Rotation des Master-Schlüssels ohne Neuverschlüsselung aller Daten ermöglicht.
- **Kernel-Sidecars (`.cxs`)** (`SPEC-04` §8): Nicht-versionierte, lokal WAL-gesicherte Hilfsdatenbanken für ephemeren Laufzeitzustand (`work_queue.cxs`, `idempotency.cxs`), um Commit-Spam im `RootSet` zu vermeiden.
- **KV-Cache & Prefix-Radix-Baum** (`contextra-kvcache`): Verwaltung und Wiederverwendung von LLM-Key-Value-Tensoren entlang gemeinsamer Prompt-Präfixe via Radix-Baum, inkl. Eviction-Worker, Tenant-Isolation und Tiering.
- **KV-Encryption / Crypto-Shredding** (`contextra-crypto`, `contextra-kvcache`): Verschlüsselung gecachter KV-Blöcke mit flüchtigen Schlüsseln (`VolatileEncryptionKey`), sodass ein Schlüsselverlust den Cache unlesbar macht.

### L

- **Leasing / Work Queue** (`SPEC-06` §7.2): Koordinationsmechanismus für asynchrone Hintergrund-Worker (z. B. Embedding-Berechnung) über `sidecar/work_queue.cxs` mit Lease-Timeouts und Retry-Zählern.
- **Leiden / Community Detection** (`SPEC-05`, `contextra-graph`): Algorithmus zur hierarchischen Cluster-Bildung auf Wissensgraphen für Zoom-Aggregationen und Community-Summaries.
- **License Gate** (`contextra-license`): Lizenz- und Feature-Ring-Prüfung zur Durchsetzung von Aktivierungs- und Editionsschranken.
- **Loom / Miri** (`contextra-checkpoint`, `contextra-vector`, `xtask`): Verifikationswerkzeuge für nebenläufige Rust-Datenstrukturen (Exhaustive Thread-Interleaving-Tests mit `loom`, Undefined-Behavior-Prüfung mit `miri`).
- **LSM-Tree (Log-Structured Merge-Tree)** (`contextra-store`): Schreiboptimierte Speicherarchitektur aus WAL, In-Memory-`MemTable` und unveränderlichen `SSTable`-Ebenen mit Hintergrund-Compaction.
- **Lyapunov Drift Watcher** (`contextra-adapt`): Stabilitätswächter auf Basis von Lyapunov-Funktionen zur Überwachung von Warteschlangen-, Latenz- oder Verteilungs-Drift.

### M

- **ManualClock** (`contextra-testkit`): Deterministisch steuerbare Uhr-Implementierung des `Clock`-Traits für reproduzierbare Zeit- und TTL-Tests.
- **MCP (Model Context Protocol)** (`contextra-mcp`): Standardisierte JSON-RPC/stdio-Schnittstelle für KI-Agenten. In Contextra als *Layer-7-Rand-Crate* ohne `unsafe`-Code für untrusted Eingaben implementiert.
- **MemDelta / MemTable** (`contextra-store`, `SPEC-04` §1.4): In-Memory-Schreibpuffer vor dem Persistieren auf Disk (`MemTable` im LSM-Tree bzw. branch-lokale Skip-List `MemDelta` vor dem Prolly-Fold).
- **Merge-Gates (`G1`–`G7`)** (`SPEC-04` §6.3): Deterministische Qualitäts- und Sicherheitsprüfungen, die ein Agenten-Session-Branch bestehen muss, bevor er nach `main` gemerged werden darf.
- **MergeOperator** (`contextra-store`, `contextra-sandbox`): Programmierbarer Operator zur konfliktfreien Zusammenführung von Werten bei Compaction oder Branch-Merge (optional in WASM-Sandbox).
- **MVCC (Multi-Version Concurrency Control)** (`contextra-mvcc`): Gleichzeitigkeitskontrolle über unveränderliche Versionen und Sequenznummern (`SequenceLog`), sodass Leser konsistente Snapshots ohne Schreibblockaden sehen.

### N

- **Namespace (`NsId`)** (`SPEC-02`): Logische Mandanten- oder Domänentrennung innerhalb eines Contextra-Repositories.
- **Noop-Adapter / Port-Traits** (`contextra-ports`, `contextra-agent`, `contextra-engine`): Abstrakte Interfaces (`Clock`, `Vfs`, `ObjectStore`, `EmbeddingProvider`, `LlmTextGenerator`) und zugehörige No-Op-Implementierungen für entkoppelte Tests und Simulationen.

### O

- **OCC (Optimistic Concurrency Control)** (`contextra-mvcc`, `SPEC-04`): Transaktionsverfahren ohne vorab gesetzte Schreibsperren; Konflikte werden erst beim Commit gegen die Lese-/Schreibmengen geprüft.
- **Ollama / ONNX / Candle** (`contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-infer-candle`): Die drei austauschbaren Inferenz-Adapter-Crates für lokale GGUF-Modelle (Candle), In-Process-Embeddings (ONNX Runtime) und externe Modellserver (Ollama).
- **Orphan Cleanup** (`contextra-engine`): Bereinigung verwaister Verweise oder Hyperkanten (`OrphanCleanupIndex`, `OrphanCleanupBackoffConfig`).

### P

- **Pack (`.cxp`)** (`SPEC-03`): Unveränderliche Container-Datei auf Disk, die mehrere PAX-Extents, Prolly-Knoten oder Index-Blöcke bündelt.
- **PAX-Extent**: Siehe *Extent (PAX)*.
- **PID-Regler** (`contextra-adapt`): Proportional-Integral-Derivative-Regler zur dynamischen Steuerung von Systemparametern (z. B. Compaction-Rate, Cache-Größen, Backoff).
- **Platt Scaler** (`contextra-rank`): Parametrische logistische Kalibrierung (`PlattScaler`) zur Umwandlung roher Ähnlichkeits-Scores in kalibrierte Wahrscheinlichkeiten.
- **PPR (Personalized PageRank, Forward-Push)** (`contextra-graph`): Schneller, lokaler Graph-Random-Walk-Algorithmus (*Forward-Push-PPR*) zur Bestimmung kontextuell relevanter Entitäten ausgehend von Seed-Knoten.
- **Prolly-Tree** (`SPEC-04` §1): *Probabilistic B-Tree* — inhaltsadressierter, deterministischer Suchbaum, dessen Struktur ausschließlich vom Schlüssel-Wert-Inhalt abhängt (history-unabhängig, ideal für O(1)-Branching und strukturelles Diffing).
- **Provenienz (`source_root`)** (`SPEC-02` §2.6): Pflichtangabe zur Herkunft einer Information (Quelle, Extraktor, Zeitstempel), um Zirkelschlüsse und Echokammern bei Konfidenzberechnungen zu verhindern.
- **Purge / Retire** (`SPEC-01` §1.4, `SPEC-06` §7.3, `contextra-db`): Explizite Löschmechanismen ohne kryptographische Löschnachweise: `retire` setzt einen versionierten Tombstone; `purge` entfernt Daten nach Retention-Ablauf im Best-Effort-Verfahren.

### Q

- **Quantisierung (SQ8, RaBitQ, PQ, f16)** (`contextra-vector`, `contextra-simd`): Verlustbehaftete Vektorkompression zur RAM-Reduktion bei hoher Recall-Erhaltung (8-Bit-Skalarquantisierung `SQ8`, binäre `RaBitQ`-Projektion, Produktquantisierung `PQ`, Half-Precision `f16`).

### R

- **RaBitQ** (`contextra-vector`, `quantize_rabitq.rs`): Hochkomprimierende, fehlerkorrigierte Vektorquantisierung mit kalibrierten Distanzschätzern für große Vektorbestände.
- **Re-Layout (Affinity)** (`SPEC-03` §4): Physische Neuanordnung häufig gemeinsam gelesener Cells in neue PAX-Extents unter Aktualisierung des `CellLocator`, ohne `ChunkId` oder `CommitId` zu verändern.
- **Replicator Dynamics Weights** (`contextra-rank`): Evolutionäre Gewichtsanpassung (`replicator-dynamics-weights`) für die Gewichtung der Retrieval-Signale in der Fusion.
- **Ring-Architektur (Ring 0–4 / Unsafe Islands)** (`CLAUDE.md`, `contextra-core`, `contextra-sys`, `contextra-simd`, `contextra-wire`): Strikt geschichtete Abhängigkeitsordnung des aktuellen Workspace. `unsafe`-Code ist workspace-weit verboten (`unsafe_code = "deny"`) und ausschließlich in drei dedizierten *Ring-0 Unsafe Islands* (`contextra-sys`, `contextra-simd`, `contextra-wire`) gekapselt.
- **Roaring Bitmap** (`roaring`, `SPEC-05`): Komprimierte Bitmaps über `Gid`/`DocId` für schnelle Mengenoperationen (ACL-Filter, Namespace-Filter, Tombstones) *vor* bzw. *während* des Rankings (`INV-5`).
- **RootSet** (`SPEC-04` §2): Die Menge aller logischen, deterministischen Prolly-Tree-Wurzeln eines Commits (`primary`, `sequences`, `graph`, `aliases`, `policies`, `conflicts`, `provenance`).
- **Router** (`contextra-router`): Intelligente Anfrage-Verteilung zwischen Small Language Models (SLMs) und größeren Modellen (`ArmRegistry`, `SlmProfile`, `DecisionId`, `RoutingOutcome`).
- **RRF (Reciprocal Rank Fusion)** (`contextra-rank`, `contextra-db`): Rangbasierte Fusionsmethode `1 / (k + rank)`, die verschiedene Retrieval-Listen ohne vorherige Score-Normierung robust kombiniert.

### S

- **Sandbox (WASM Execution Boundary)** (`contextra-sandbox`): Isolierte WebAssembly-Ausführungsumgebung (`WasmExecutor`, `WasmMergeFunction`) mit strikten Capability- und Fuel-Limits für untrusted Code-Execution oder Merge-Operatoren.
- **SATB + Remark GC** (`SPEC-07` §4): *Snapshot-at-the-Beginning* Garbage Collection mit Write-Barrier, die garantiert, dass kein von einem aktiven Root (Branch, Tag, Pin, Savepoint) erreichbares Objekt entfernt wird (`INV-9`).
- **Savepoint** (`SPEC-04`, `contextra-checkpoint`): Leichtgewichtiger Rücksetzpunkt innerhalb eines Agenten-Session-Branches (z. B. vor jedem Tool-Call).
- **Sequence Log** (`contextra-mvcc`): Monotones Sequenzprotokoll zur Vergabe von Transaktions- und Watermark-Sequenznummern.
- **Session-Branch & Session-DAG** (`contextra-graph`, `SPEC-04` §4): Isolierter Schreibzweig für einen Agenten oder eine Konversation (`INV-7`: Agenten schreiben nie direkt nach `main`).
- **SIMD-Kernels** (`contextra-simd`): Hardware-beschleunigte Vektor-Distanzfunktionen (Cosine, Dot-Product, L2, SQ8/U8-Varianten) mit Laufzeit-CPU-Feature-Dispatch (AVX2, AVX-512, NEON).
- **Small-to-Big-Expansion** (`SPEC-06` §3.2): Retrieval-Strategie, bei der zunächst kleine, präzise Chunks gesucht und anschließend für den LLM-Kontext auf ihre übergeordneten Eltern-Abschnitte expandiert werden.
- **Snapshot (`Snapshot<'db>`)** (`SPEC-04` §3.1, `INV-2`): Lebenszeit-gebundener, konsistenter Lesezustand `(head, mem_delta_seq, now)`, den alle Modalitäten einer Query gemeinsam nutzen.
- **SQ8 (Scalar Quantization 8-bit)** (`contextra-vector`): Komprimierung von `f32`-Vektorkomponenten auf `u8` inkl. Bias-Kalibrierung (`sq8_bias.rs`) und Reranking.
- **SSI (Serializable Snapshot Isolation)** (`contextra-mvcc`, `SequenceLogSsiValidator`): Stärkste Isolationsstufe für Transaktionen ohne Phantom- oder Write-Skew-Anomalien.
- **SSTable (Sorted String Table)** (`contextra-store`): Unveränderliche, sortierte Schlüssel-Wert-Datei auf Disk im LSM-Tree mit Block-Index und Bloom-/Filterstrukturen.
- **Strangler Facade (`contextra-core`)** (`contextra-core`): Übergangs-Crate im Strangler-Pattern, das Legacy-Re-Exports während der Migration auf die Ring-0-Crates (`contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-wire`) bereitstellt.

### T

- **TaintFlags & TrustTier** (`SPEC-07` §7.1): Orthogonale Sicherheitsmetadaten: `TrustTier` beschreibt die Vertrauenswürdigkeit einer Quelle (wird per Minimum vererbt), `TaintFlags` markieren Kontaminationen wie untrusted Web-Input (werden per Union vererbt).
- **TenantScoped / Tenant-Isolation** (`contextra-types`, `contextra-kvcache`): Strikte Mandantentrennung auf Typ- und Cache-Ebene (`TenantScopeViolation`) gegen Cross-Tenant-Datenlecks.
- **Testkit (`contextra-testkit`)** (`contextra-testkit`): Gemeinsame Test-Infrastruktur mit `ManualClock`, `InMemoryStorageEngine`, `FaultVfs` und `ReferenceModel` für Differential- und Property-Tests.
- **Tiering** (`contextra-kvcache`, `SPEC-01`): Automatische Datenplatzierung über Speicherebenen hinweg (RAM ↔ NVMe ↔ Objektspeicher/S3).
- **Tombstone** (`contextra-store`, `contextra-vector`, `SPEC-06`): Löschmarkierung (`KvDeleteMode`, `TombstoneSet`, `retire`) in unveränderlichen Strukturen, bis eine Compaction bzw. ein Rebuild den Platz freigibt.
- **Token-Budget & Zoom-Knapsack** (`contextra-types`, `SPEC-06` §3.2, `INV-8`): Harte Obergrenze (`max_tokens`) für den zusammengestellten LLM-Kontext. Der Packer löst ein Multiple-Choice-Knapsack-Problem über Zoom-Stufen mit Redundanzabzug (MMR/Submodularität).
- **Txn-Typestate** (`SPEC-04` §3.1): Compile-Zeit-Unterscheidung zwischen `Txn<ReadOnly>` und `Txn<ReadWrite>` zur Verhinderung unregistrierter SSI-Lesezugriffe.

### U

- **Unsafe Island** (`contextra-sys`, `contextra-simd`, `contextra-wire`): Die exakt drei Ring-0-Crates, in denen `unsafe` für FFI, SIMD-Intrinsics oder Zero-Copy-Deserialisierung unter strengen Sicherheitsinvarianten gekapselt ist.

### V

- **Vamana**: Siehe *DiskANN / Vamana*.
- **VerifiableRoot (`Unverified` / `Verified`)** (`contextra-wire`): Typestate-Muster zur Validierung untrusted FlatBuffers-Payloads vor dem Zero-Copy-Zugriff.
- **Vfs-Port** (`contextra-ports`, `contextra-durable-fs`, `contextra-testkit`): Injizierbares Dateisystem-Interface (`Vfs`), das echte Disk-I/O, In-Memory-Tests und Fehlerinjektion austauschbar macht.
- **Volatile Context Vault** (`contextra-db`, `contextra-crypto`): Flüchtiger, verschlüsselter Zwischenspeicher (`VolatileContextVault`, `VaultChunk`, `PurgeReceipt`) für temporäre Arbeitsdaten mit kontrolliertem `Purge`.

### W

- **WAL (Write-Ahead Log, `.cxw`)** (`contextra-store`, `SPEC-07`): Sequenzielles, crash-sicheres Änderungsprotokoll auf Disk. Ein Commit ist dauerhaft, sobald sein WAL-Frame geflusht ist.
- **Watermark (`watermark_seq` & Fallback)** (`SPEC-05` §1.2, `INV-4`): Sequenznummer im `IndexManifestRef`, bis zu der ein Index-Segment aktuell ist. Neuere Einträge (`> watermark_seq`) werden transparent über exakte Delta-/MemDelta-Scans ergänzt.
- **Work Queue**: Siehe *Leasing / Work Queue*.

### X

- **`xtask` / `xtask-heavy`** (`xtask`, `xtask-heavy`): Cargo-Automatisierungs-Crates im Workspace für Architektur-Gates, Feature-Katalog-Generierung (`gen-feature-catalog`), Audit-Freshness-Checks, Benchmarks und schwere Verifikationsläufe (Mutation-Testing, Fuzzing).

### Z

- **Zero-Copy (FlatBuffers)** (`contextra-wire`): Direkter Lesezugriff auf serialisierte Puffer ohne Allokation oder Parsing-Kopien nach vorheriger Strukturverifikation.
- **Zoom-Stufen (L0–Ln) & Zoom-Knapsack** (`SPEC-02`, `SPEC-06` §3.2): Hierarchische Mehrfachauflösungen derselben Wissenseinheit (z. B. Titel/Abstract bis Volltext), aus denen der Context Packer unter einem Token-Budget die optimale Detailtiefe auswählt.
