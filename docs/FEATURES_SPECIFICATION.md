# Contextra — Vollständige Feature- und Funktionsspezifikation

**Stand:** 2026-09-30 (Systemspezifikation v14)
**Quelle der Wahrheit:** Quellcode-Audit über alle 35 Workspace-Crates (HEAD)
**Status:** Verbindliche Spezifikation aller sichtbaren, opt-in, versteckten und internen Features

---

## 1. Executive Summary & Architektureinordnung

Contextra ist eine hochperformante, einbettbare (embedded), air-gapped-fähige Gedächtnis- und Wissensschicht für Rust- und Lokal-KI-Anwendungen. Die Systemarchitektur gliedert sich in ein striktes **5-Ring-Modell (Ring 0 bis Ring 4)** sowie dedizierte Tooling-Crates.

| Ring | Charakteristik | Enthaltene Crates |
| :--- | :--- | :--- |
| **Ring 0** | Synchrone Kern-Primitives, Zero-Copy-Typen, SIMD, Compliance-Templates, Audit-Exports, Mathematik | `contextra-types`, `contextra-core`, `contextra-wire`, `contextra-ports`, `contextra-sys`, `contextra-simd`, `contextra-mvcc`, `contextra-crypto`, `contextra-text`, `contextra-vector`, `contextra-graph`, `contextra-rank`, `contextra-adapt`, `contextra-avv-generator`, `contextra-audit-export` |
| **Ring 1** | Persistenz, LSM-Tree, WAL, Checkpoints, KV-Cache | `contextra-store`, `contextra-checkpoint`, `contextra-kvcache` |
| **Ring 2** | Isolation, Datenschutz, Inferenz-Provider | `contextra-sandbox`, `contextra-privacy`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx` |
| **Ring 3** | Compute Engine, Graph-Konsolidierung, Routing, DB & Agenten | `contextra-engine`, `contextra-cognition`, `contextra-router`, `contextra-db`, `contextra-agent` |
| **Ring 4** | Endanwender-Fassade, Protokolle & Bindings | `contextra`, `contextra-mcp`, `contextra-py`, `contextra-license` |
| **Tooling**| Test-Harnesses & Benchmarks | `contextra-testkit`, `contextra-bench` |

---

## 2. Detaillierte Feature-Spezifikation aller 35 Crates

### 2.1 Ring 0 — Synchrone Primitives & Algorithmen

#### `contextra-types`
- **Zero-Copy Typen:** Bounded Structs für `DocId`, `TxId`, `TenantId`, `SeqNum`.
- **Einstellbare DocId-Bitbreite (`docid-128`):** Unter dem Feature `docid-128` wird `DocIdRaw` von `u64` auf `u128` skaliert.
- **Fehlertaxonomie (`ContextraError`):** Typisierte Fehler für Storage, Crypto, Graph, Vector, MVCC, Limit-Exceeded und Configuration.

#### `contextra-core`
- **Tombstone-Semantik:** Explizite Marker für gelöschte Dokumente und Relationen im Speicher.
- **`TxBuffer`:** Transaktionaler In-Memory-Puffer für atomare Batch-Schreiboperationen.

#### `contextra-wire`
- **FlatBuffers Serialization:** Zero-Copy-Deserialisierung von Nachrichtenstrukturen (`schemas/contextra.fbs`).
- **Unsafe-Insel:** Effizienter Direct-Memory-Zugriff für Wire-Protokoll-Frames.

#### `contextra-ports`
- **Abstrakte Trait-Schnittstellen:** `Clock` (`SystemClock`), `Rng` (`SeededRng` via SplitMix64), `IdGen` (`SequentialIdGen`), `AttentionExporter`.
- **Zero-Dependency-Decoupling:** Ermöglicht deterministisches Injection-Testing ohne Async-Runtime.

#### `contextra-sys`
- **Low-Level System I/O:** Memory-Mapping (`mmap`), Direct-File-Access und OS-nahe Operationen.

#### `contextra-simd`
- **Hardware-Beschleunigte Distanzberechnungen:** SIMD-optimierter Dot Product, Cosine Distance und Euclidean (L2) Distance.
- **Laufzeit-Autodetection:** Automatische Auswahl von AVX2, FMA, ARM NEON oder WASM SIMD128.

#### `contextra-mvcc`
- **Snapshot Isolation:** Locklose Multi-Version Concurrency Control für Leseoperationen.
- **SequenceLog SSI-Validierung:** `SequenceLogSsiValidator` unterstützt Pruning alter Write-Sets via `prune_through(bound_seq)`.
- **`ReadSet` O(1) Cache:** Caching von `min_snapshot_seq` mit Grenzwert `DEFAULT_MAX_READ_SET_KEYS` (100.000) zur Vermeidung von Speicherexplosionen.
- **Zeit-Injiizierte Reaping-Methoden:** `reap_orphans_at`, `reap_orphans_bounded_at` für deterministische Transaktions-Bereinigung.

#### `contextra-crypto`
- **AEAD & MAC:** AES-256-GCM Verschlüsselung und HMAC-SHA256 Integrity Validation für WAL v3.
- **Crypto-Shredding (`KeyRegistry`):** Thread-sichere Subkey-Ableitung via HKDF-SHA256 und O(1) Gruppen-Key-Revokation (`revoke_subkey`).
- **V3 DeletionProof Verification:** `DeletionProof::verify_external` erlaubt KeyManager-unabhängige Verifikation von Ed25519-Löschbeweisen (AK-20).
- **Zeroize-on-Drop:** Automatische Bereinigung empfindlicher Schluesselpuffer im Speicher.
- **Tenant-Scoped Cryptography:** Isolierte Key-Derivation und Cipher-Instanziierung je `TenantId`.

#### `contextra-text`
- **BM25 & BM25F Inverted Index:** Volltextsuche mit Feld-Gewichtung.
- **Block-Max WAND Query Engine:** Effiziente TOP-K-Suche mit Postings-Max-Scores.
- **MVCC-Term-Leak-Schutz:** Sequenz-bewusste Posting-Schlüssel-Prüfung (`storage.get_at_seq(&pl_doc_key, seq)`) verhindert Term-Leaks aus zukünftigen MVCC-Revisionen.
- **Morphologisches Tokenizing (`BM25MorphIndex`):** Sprachspezifische Tokenisierung (z. B. Deutsch via `Language::German`).

#### `contextra-vector`
- **HNSW Vector Index:** Bounded-Graph-Vektorsuche.
- **DiskANN Engine (`experimental-diskann`):** SSD-optimierte Graphsuche für große Datenmengen (>100.000 Vektoren).
- **Predicate-Augmented Search (`experimental-predicate-augmented-search`):** ACORN-artige 2-Hop-Nachbar-Expansion während der Graph-Traversierung. Verhindert Recall-Kollaps bei hoher Prädikat-Selektivität (<5%).
- **RaBitQ Quantisierung (`experimental-rabitq`):** 1-Bit/4-Bit Quantisierung für reduzierten RAM-Bedarf.
- **Partial Index Rebuild (`partial-index-rebuild`):** Tombstone-Pruning im HNSW-Graph (unter Veto-Governance).

#### `contextra-graph`
- **CSR Graph Storage:** Compact Sparse Row Repräsentation für hochperformante Graph-Traversierung.
- **Reinforcement-Learning Kanten (`edge-reinforcement-learning`):** Feature-gated `cooccurrence_weight` und `traversal_weight` Felder auf `Edge`.
- **TL-HFD Diffusion:** Subgradienten-Updates über `PathGraph`-Strukturen mittels Lovász-Extension-Schnitten (`compute_lovasz_extension`, `truncate_participants`). Monoton fallende beste Zielwerte $F^*(t)$.
- **$N$-äre Hyperkanten:** Multi-Knoten-Hyperkanten via `HyperEdgeView` und `ArcSlice<T>`.
- **Diffusion & Community Detection:** GraphRAG, Label Propagation, PPR Forward Push, $k$-Path Diffusion, APPRH Diffusion.

#### `contextra-rank`
- **Rank Fusion & Normalisierung:** Reciprocal Rank Fusion (RRF), DiBud-Algorithmus (`dibud`), Coherence Bonus Scorer.
- **Score-Kalibrierung:** Platt Scaler (`PlattScaledSigmoid`) und Isotonische Kalibrierung (`isotonic-calibrator`).

#### `contextra-adapt`
- **PID Latency Controller:** PID-Regelkreis zur dynamischen Steuerung von Such-Latezen.
- **Sherman-Morrison Inverse Matrix Updates:** $O(d^2)$ Rank-1 Updates der Inversen Matrix $A^{-1}$ in `BanditProfileState`.
- **Flow-Corrected Thompson Sampling (`flow-corrected-thompson`):** Adaptives Routing unter Berücksichtigung von System-Flussraten.
- **Sketched Bandit & RIE Personalization:** Reduzierter Speicherbedarf durch Sketching und RIE-Greedy-Personalisierung.

---

### 2.2 Ring 1 — Persistenz, Storage & Cache

#### `contextra-store`
- **LSM-Tree Engine:** MemTable, SSTables, Block-Cache und Compaction-Engine.
- **WAL v3 mit HMAC-SHA256 Integrity:** Lückenloser Integritätsschutz gegen Stille Datenkorruption.
- **Löschmodi (`KvDeleteMode`):** `TombstoneOnly` und `CryptoShred` (Default).
- **Crash-Safe Rekeying Migration:** `Wal::open_for_legacy_migration()` führt Re-MAC auf v3 aus und setzt ein `.rekeyed`-Marker-File. Nach Vorhandensein wird der Legacy-Fallback dauerhaft gesperrt.
- **TOCTOU-Safe WAL Truncation:** `WalCommand::Truncate` aktualisiert zuerst die atomare In-Memory-Größe (`size.store`), bevor `file.set_len` ausgeführt wird.
- **SSTable Resurrection Protection:** `Manifest::reconstruct_dead_sstables()` berechnet explizit gelöschte SSTables und entfernt diese beim Start von der Festplatte.
- **WAL Tail HMAC Verification:** `verify_wal_chain_completeness` vergleicht beim Start den WAL-Tail-HMAC mit dem Manifest High-Water-Mark.

#### `contextra-checkpoint`
- **Blake3 Manifest Verification:** Prüfsummenvalidierung für Persistenz-Checkpoints.
- **Orphan Recovery:** Automatische Identifikation und Bereinigung verwaister Daten-Segments.

#### `contextra-kvcache`
- **Mandanten-Isolierter Prefix-Radix-Cache (`TenantPrefixKvStore`):** Strukturell isolierter KV-Cache mit `PrefixRadixTree` und Konfiguration des Byte-Budgets je Mandant (Default: 256 MiB). LRU-Blockgruppen-Eviction.
- **AttentionExporter-Gewichtete Eviction:** `EvictionWorker` verknüpft sich mit dem Ring 0 Trait `AttentionExporter` via `ExporterBackedScoreSource` für gewichtetes Cache-Pruning.
- **KiVi Quantisierung (`kivi-quantization`):** Komprimierte KV-Cache-Speicherung für LLM-Inferenz.

---

### 2.3 Ring 2 — Isolation, Privacy & Provider

#### `contextra-sandbox`
- **Entkoppelte WASI Preview 1 Sandbox:** Reine Rust-Implementierung der WASI-Host-Funktionen (`wasi.rs`) ohne Abhängigkeit von `wasmtime-wasi`.
- **Ressourcen- & Determinismus-Schranken (`WasmCapabilities`):** Begrenzung von Standard-Output/Input (`max_output_bytes`, `max_stdin_bytes`, Default 1 MiB) sowie PRNG-Seeding (`random_seed: Option<u64>`).
- **Trap-Mapping:** Direktes Trapping von `OutputLimitExceededError` und `ProcessExitError` auf `SandboxError`.

#### `contextra-privacy`
- **Cloud Egress Gateway & Guard:** Egress Filtering, Regex-Klassifizierung und PII-Anonymisierung.
- **Tenant-Scoped Safeguards:** additive `TenantScoped`-Methoden (`classify_scoped`, `sanitize_and_vault_scoped`, `check_scoped`) erzwingen strikte Mandanten-Validierung.
- **Prompt Injection Protection:** Erkennung und Blockade von Prompt-Injection-Versuchen in eingehenden Queries.

#### `contextra-infer-candle`
- **Reine Rust GGUF Inference Engine:** In-Process Inferenz via Candle Framework.
- **Hardware-Beschleunigung:** Apple Metal und CUDA Support.
- **AttentionExporter Adapter:** Bounded Ring-Buffer (`MAX_TRACKED_REQUESTS = 256`) zum Export von 1D-Prefill-Attention-Gewichten.

#### `contextra-infer-ollama`
- **Remote Model Provider Client:** Integration externer Ollama LLM- und Embedding-Endpunkte.

#### `contextra-infer-onnx`
- **ONNX Runtime Execution Provider:** Decomposed Module für Cross-Encoder Reranking (`RerankConfig`, `CrossEncoderReranker`) und Embeddings.

---

### 2.4 Ring 3 — Compute Pool, Routing & High-Level DB

#### `contextra-engine`
- **Async Execution Layer:** Entkoppelte Asynchron-Ausführung über den synchronen Ring 0 Primitives (ADR-N02).
- **ACORN Scoped Search Execution:** `ScopeConstraint` mit `allowed_doc_ids` (BTreeSet) und `gamma = 2`. Scoped Knn-Suche (`search_knn_acorn`) erfordert `execute_with_scope()`. Aufruf von `.execute()` mit aktiver `hard_scope` führt zu `CapabilityUnsupported`.
- **OpenIE Auto-Extraction mit Provenance:** `auto_extract_and_relate` speichert `source_doc_id` auf extrahierten Kanten für Löschbeweis-Kaskaden.
- **Zero-Panic Fallback Stubs (`no_crypto_stubs`):** Ermöglicht Klartext-Ausführung ohne `contextra-crypto`.

#### `contextra-cognition`
- **Sleep-Cycle Consolidation:** Hintergrund-Passes zur Konsolidierung von Epochen-Erinnerungen und Knotengruppierung.
- **Graph Connectivity Health:** Überwachung der Graph-Perkolation (`physio-percolation`).
- **Edge Reinforcement Learning Pass:** Dynamische Kanten-Gewichtung basierend auf Abrufhäufigkeit.

#### `contextra-router`
- **Lyapunov Drift Control:** Dynamische Balance zwischen Suchqualität (Recall/Precision) und Latenz/Kosten-Budget via `LyapunovDriftWatcher`.
- **Contextual Bandit Routing:** Sherman-Morrison und Flow-Corrected Thompson Sampling fürs Candidate-Routing.

#### `contextra-db`
- **Strangler Shell Facade:** Konsolidierte Fassade über `contextra-engine`, `contextra-cognition` und `contextra-adapt`.
- **VolatileContextVault:** Ephemerer Vault zur sicheren Verarbeitung flüchtiger Kontexte.
- **MultiStepEngine:** Mehrstufige Query-Ausführung und Planungs-Pipeline.

#### `contextra-agent`
- **Agent Memory Pipeline:** `remember()`, `recall()`, `forget()`, `relate()`.
- **Dead Letter Queue (DLQ):** Erfassung fehlgeschlagener Step-Ausführungen.
- **Execution Budget & Isolation:** Strenge Schritt- und Time-Out-Limits für Agenten-Aktionen.

---

### 2.5 Ring 4 & Facade — Public API, MCP & Compliance

#### `contextra` (Haupt-Facade)
- **Unified Public API:** Direkter Zugriff auf `AgentMemory`, High-Level Search & Storage.
- **Feature Rings:**
  - `fast` (Default): Minimale, synchrone Kernfunktionalität.
  - `sovereign`: Aktiviert Verschlüsselung (`contextra-crypto`) und Privacy Gateway (`contextra-privacy`).
  - `compliance`: Aktiviert zusätzlich DSGVO Art. 30 Audit-Export und AVV-Generator.

#### `contextra-mcp`
- **Model Context Protocol Server:** Standardisiertes Interface für KI-Agenten und LLMs.
- **`contextra_forget` Bestätigungspflicht:** Erfordert expliziten Sicherheitsparameter `confirm: true` zur Löschung.
- **Exfiltration & Injection Defense:** Integrierter Bulk Exfiltration Detector und Prompt Injection Guard.

#### `contextra-py`
- **PyO3 Bindings:** Python-Schnittstelle für `Collection`, `Search`, `Memory`.

#### `contextra-audit-export`
- **DSGVO Art. 30 Verarbeitungsverzeichnis:** Export-Funktionen (`render_register_json`, `render_register_markdown`) für Compliance-Audits.

#### `contextra-avv-generator`
- **DSGVO Art. 28 AVV Template Generator:** Automatische Erstellung strukturierter Auftragsverarbeitungsverträge.

#### `contextra-license`
- **Ring Feature Gate Control:** Lizenzschlüssel- und Aktivierungsprüfung für Enterprise-Module.

---

## 3. Versteckte, unaufgeforderte & interne Features (Code-Fundgrube)

| Subsystem | Verstecktes / Internes Feature | Funktionsweise / Detail im Quellcode |
| :--- | :--- | :--- |
| **`contextra-engine`** | ACORN Hard-Boundary Query Scoping | `ScopeConstraint` mit `allowed_doc_ids: BTreeSet<DocId>` erzwingt harte Suchgrenzen (`search_knn_acorn`). Regulärer `.execute()`-Aufruf schlägt bei gesetzter Scope fehl (`CapabilityUnsupported`). |
| **`contextra-vector`** | Predicate-Augmented 2-Hop Expansion | `experimental-predicate-augmented-search` führt ACORN-artige 2-Hop-Nachbar-Expansionen während Graph-Traversierungen aus. Verhindert Recall-Kollaps (<5% Selektivität). |
| **`contextra-text`** | BM25 WAND MVCC Sequence Filtering | `storage.get_at_seq(&pl_doc_key, seq)` prüft während Nicht-Latest-Suchen Postings gegen historische MVCC-Sequenzen. Verhindert Term-Leaks aus nachfolgenden Updates. |
| **`contextra-store`** | Crash-Safe WAL Rekeying Migration | `Wal::open_for_legacy_migration()` schreibt Re-MACed Segment-Dateien und ein Marker-File (`.rekeyed`). Schaltet den unsicheren Obfuscation-Fallback dauerhaft ab. |
| **`contextra-store`** | SSTable Resurrection Protection | `Manifest::reconstruct_dead_sstables()` ermittelt in der Historie gelöschte SSTables und löscht deren verbliebene `.sst`-Dateien beim Systemstart ab. |
| **`contextra-store`** | TOCTOU-Safe WAL Truncation | In `WalCommand::Truncate` wird der atomare Größenwert im RAM aktualisiert, *bevor* die physische Datei abgeschnitten wird. |
| **`contextra-crypto`** | KeyRegistry & O(1) Crypto Shredding | `KeyRegistry` unterstützt die dynamische Ableitung hierarchischer Subkeys via HKDF-SHA256 sowie O(1) Widerruf (`revoke_subkey`). |
| **`contextra-sandbox`** | Standalone WASI Host Boundary | Implementiert WASI Preview 1 manuell in `wasi.rs` ohne `wasmtime-wasi`. Erzwingt Stream-Size-Limits (`1 MiB`) und injiziert PRNG-Seeds. |
| **`contextra-graph`** | TL-HFD Diffusion & Lovász Extension | Berechnet Subgradienten-Schritte über `PathGraph` via Lovász-Extension-Schnitten (`compute_lovasz_extension`). Zielwerte sind nachweislich monoton nicht-steigend. |
| **`contextra-adapt`** | Sherman-Morrison Inverse Matrix Update | Pflege der inversen Kovarianzmatrix $A^{-1}$ in $O(d^2)$ Zeitaufwand pro Update via Rang-1 Sherman-Morrison Formalismus. |
| **`contextra-kvcache`** | TenantPrefixKvStore & Attention Eviction | Strukturelle Trennung je Mandant in `PrefixRadixTree` mit LRU-Blockgruppen-Eviction. EvictionWorker nutzt Prefill-Attention-Weights als Score-Quelle. |
| **`contextra-mcp`** | Confirmation Gate on MCP Forget | MCP Tool `contextra_forget` lehnt Aufrufe ohne `confirm: true` ab. |

---

## 4. Cargo Feature-Flag Matrix

| Crate | Default Features | Opt-In / Non-Default Features |
| :--- | :--- | :--- |
| `contextra` | `fast`, `candle` | `sovereign`, `compliance`, `audit-export`, `avv-generator`, `onnx`, `ollama`, `router` |
| `contextra-store` | `wal-integrity` | `encryption-at-rest`, `deletion-proof`, `memory-only-storage`, `fault-injection`, `block-cache-v2`, `sieve-cache`, `docid-128` |
| `contextra-vector` | *keine* | `docid-128`, `experimental-diskann`, `experimental-rabitq`, `experimental-predicate-augmented-search`, `graph`, `partial-index-rebuild` |
| `contextra-graph` | *keine* | `docid-128`, `graph-connectivity-health`, `edge-reinforcement-learning`, `ppr-forward-push`, `k-path-diffusion`, `apprh-diffusion` |
| `contextra-text` | *keine* | `bm25f`, `docid-128` |
| `contextra-rank` | *keine* | `dibud` |
| `contextra-adapt` | `bandit-routing`, `flow-corrected-thompson` | `egress-sherman-morrison`, `sketched-bandit`, `rie-greedy-personalization` |
| `contextra-router` | *keine* | `bandit-routing`, `cloud-egress-guard`, `egress-sherman-morrison`, `flow-corrected-thompson` |
| `contextra-kvcache` | *keine* | `content-addressed-kv-cache`, `kv-encryption`, `kvcache-attention-eviction`, `kvcache-kivi-quant`, `kivi-quantization` |
| `contextra-infer-candle` | *keine* | `candle`, `docid-128`, `kv-bridge`, `kv-stage-b`, `cuda` |
| `contextra-infer-onnx` | *keine* | `reranking`, `onnx`, `candle-backend` |
| `contextra-mcp` | *keine* | `agent-workflows`, `onnx`, `candle`, `ollama`, `kv-bridge`, `test-utils`, `docid-128` |
| `contextra-engine` | *keine* | `encryption-at-rest`, `bench`, `sandbox`, `reranking`, `onnx`, `experimental-diskann`, `background-maintenance`, `graph-connectivity-health`, `docid-128`, `edge-reinforcement-learning`, `coherence-bonus-fusion`, `adaptive-candidate-pool-sizing`, `entity-extraction` |
| `contextra-db` | *keine* | `bench`, `sandbox`, `reranking`, `onnx`, `experimental-diskann`, `background-maintenance`, `graph-connectivity-health`, `coherence-bonus-fusion`, `adaptive-candidate-pool-sizing`, `volatile-vault`, `docid-128`, `edge-reinforcement-learning` |
