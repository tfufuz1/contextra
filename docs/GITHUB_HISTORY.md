# Contextra Brain — GitHub Projekt- & Commit-Historie

> **Kanonische Dokumentation der Entwicklungshistorie und tiefen Differenz-Analysen von Contextra Brain**
> *Zeitraum: August 2026 — September 2026 (Stand: 18.09.2026)*

---

## 1. Übersicht & Meilenstein-Phasen

Contextra Brain ist ein kognitives Betriebssystem und eine eingebettete Vektor- & Wissensdatenbank in Pure Rust. Die Projektgeschichte auf GitHub zeichnet die schrittweise Evolution von den mathematischen und speichertechnischen Kernschichten (Layer 0 & 1) über das Datenbank-Orchestrierungsmodul (Layer 2 - 6) und die LLM/FFI-Anbindungen bis hin zur sicheren Desktop-, MCP-Server-, Multi-Tenancy- und Benchmarking-Umgebung (Layer 7 - 9) nach.

### Phasenüberblick

| Phase | Zeitraum | Fokus & Kern-Errungenschaften |
|---|---|---|
| **Phase 1: DAG-Aktivierung & Multi-Crate Foundation** | 22.08.2026 – 24.08.2026 | DAG-Validierung, Entkopplung archivierter Crates, Fixes für DiskANN Bounds & Sector Size Checks, Ingestions-TxId Collision Fixes, HNSW Delete-Error Handling. |
| **Phase 2: RAG, Session-DAG & Cryptographic Hardening** | 25.08.2026 – 26.08.2026 | Anthropic Contextual Retrieval, Pure-Rust Session-DAG Branching, OsRng/AES-256-GCM-SIV & WAL HMAC Chaining, JSON-RPC 2.0 MCP-Server Basis. |
| **Phase 3: MVCC Durability, 2PC Transactions & Sync-Docs** | 27.08.2026 – 28.08.2026 | Full 4-Index 2-Phase Commit (2PC) in `contextra-db`, WAL V3 Format mit `tx_id` HMAC Binding, Bi-temporale Graph-Gültigkeitsachsen, `xtask sync-docs` Werkzeuge. |
| **Phase 4: Robustness & Security Hardening Sprint** | 29.08.2026 – 30.08.2026 | Agent Event Loops, Memory Importance Decay, Zettelkasten Memory Links, Structured `ContextraErrorDto`, Prompt Injection Guards in MCP, Zero-Copy LSM Scan, Write-Temp-Then-Rename für SSTables & DiskANN. |
| **Phase 5: Governance, Quality & Round 2 Audit Pass** | 31.08.2026 – 01.09.2026 | Erweiterung von `xtask check-consistency` (README, AGENTS.md, ADR Checks), CI Review Coverage Gates, `contextra-index` Code Quality Refactoring (#1150), Token Budget Race Audit & Tests (#1239). |
| **Phase 6: Deep Tier 1 Audits, Architectural Decoupling & Storage/Index Hardening** | 02.09.2026 – 05.09.2026 | Entkopplung von `contextra-crypto` und `contextra-core`, 3-Phasen Lock-Free Async LSM Flush (`LsmStorage::flush`, ADR-059/060), WAL Crash-Safety & HMAC Chaining Fixes (KRIT-02–04, MED-05), Async CSR `add_edge` Lock Splitting, SIMD Trait Unification (`std::arch`, ADR-047), Persistente Router-Kalibrierung, Tier 1 Deep Audits für Layer 0, Store, Crypto, Graph & Index mit GO-Verdikt. |
| **Phase 7: Multi-Tenancy Isolation, GDPR Compliance & PathRAG Cognitive Layering** | 06.09.2026 – 07.09.2026 | Mandantenfähige Trennung (`TenantId`, `TenantKeyCodec`, `contextra-kv-bridge`), Kryptographische DSGVO Article 17 Deletion Proofs (`DeletionProof`), DiskANN WAL-backed Pending Buffer & Delta Persistence, PathRAG Graph Engine & F-06 Percolation Health Monitor, Kaskadierende Kanten-Invalidierung bei Chunk-Verdrängung (`cascade.rs`), Isotonic/Platt Konforme Kalibrierung (`contextra-calibration`), Replicator Dynamics Fusion. |
| **Phase 8: Tier 2/3 Deep Subsystem Audits & Cross-Crate Alignment** | 08.09.2026 – 09.09.2026 | Tiefenaudits für `contextra-kv-bridge`, `contextra-calibration`, `contextra-agent`, `contextra-graph`, `contextra-bench`, `contextra-mcp`, `contextra-embed` und `contextra-candle`. Umbenennung & Paket-Aliasing von `contextra-crypto` zu `contextra-security`, Konsolidierung der Spezifikationsdokumente (v10.1), Bereinigung biologischer Metapher-Begriffe in `contextra-db`. |
| **Phase 9: Storage Durability, WAL Recovery & Infrastructure Gates** | 10.09.2026 – 11.09.2026 | Beseitigung von stummen I/O-Fehlern bei WAL `sync_all` und Recovery-Rollbacks (Gate 3), automatische WAL Sidecar Cleanup & Startup Flushes in LSM, `xtask claim` Release & TTL Expiry, neues CI Phantom-Files Check Gate (Gate 12b), HuggingFace LongMemEval Dataset-Fixes in `contextra-bench`. |
| **Phase 10: System-wide Quality Verification & Text Engine Deep Audit** | 12.09.2026 – 13.09.2026 | Systematischer Deep Audit von `contextra-text` (#2243) mit Char-Boundary Sicherheitsnachweisen und BM25 Robertson-Spärck-Jones IDF-Reverifikation; Integration von `contextra-sandbox` (WASM Boundary, Layer 2 / Spec v5 §4.18) und Live-Stats-Wiring für MCP (`setup_routing`). |
| **Phase 11: Architectural Decoupling, Zero-Copy Read Paths & HNSW Compute-then-Commit** | 14.09.2026 – 16.09.2026 | Entkopplung von `RouterEngine<S: StorageEngine>` (Layer 6) aus `contextra-store`, Umstellung aller Read-Pfade auf `bytes::Bytes` Zero-Copy Slices, HNSW Compute-then-Commit Pattern & Phantom Node Elimination, SQ8 Quantisierer Skaleninvarianz, P1 Command Injection Fix in MCP Dispatch, Consolidation Lock Mutual Exclusion. |
| **Phase 12: Layer 0 Governance Synchronization, High-Performance Optimizations & Post-Merge Automation** | 17.09.2026 – 18.09.2026 | HNSW Persistence Fuzzing Target (`fuzz_hnsw_persistence`), Refactoring des CSR Graph Staged Entity Keyings, Loom Concurrency Models für Deferred Zeroize & Group Commit, Memory Allocation Optimizations in MemTable & RRF Fusion (`fusion.rs`), `xtask lint-unsafe-slices` AST SIMD Lint, `xtask post-merge-report` Integration (#3109), Governance Sync und Inventar-Audit von `contextra-core`. |

---

## 2. Chronologischer Commit-Verlauf

Hier sind die präzisen Commits der Entwicklungshistorie (chronologisch von den Anfängen bis heute):

### 22. August 2026
- `90633e3` | **google-labs-jules[bot]** | `Fix verify-dag CI workflow for contextra-crypto and archived crates`
  *Anpassung der DAG-Prüfung in GitHub Actions zur korrekten Erkennung der Abhängigkeiten.*
- `65322dc` | **google-labs-jules[bot]** | `fix(ci): Allow contextra-crypto dependency in contextra-store DAG check`
  *Erlaubt die explizite Abhängigkeit von `contextra-store` auf `contextra-crypto` im DAG-Graph.*

### 23. August 2026
- `82c9966` | **google-labs-jules[bot]** | `Fix Ollama batch embedding stream lifetime and MCP test dimension configuration`
  *Behebung von Lifetime-Problemen bei gestreamten Ollama Batch-Embeddings und Angleichung der Dimensionen im MCP Test Harness.*

### 24. August 2026
- `f0c0335` | **google-labs-jules[bot]** | `fix(diskann): bounds check neighbor_count & resolve CI context-gates smells`
  *Sicherheitsüberprüfung für `neighbor_count` gegen `max_degree` in DiskANN zur Vermeidung von Out-of-Bounds Speicherallokationen.*
- `ae0cd14` | **google-labs-jules[bot]** | `Fix TxId collisions, add graph error logging, and resolve critical AI tags`
  *Behebung von Transaktions-ID-Kollisionen bei der Dokumenten-Ingestion und Hinzufügen von strukturierter Fehlerprotokollierung im Graph-Index.*
- `c4139b0` | **google-labs-jules[bot]** | `fix(diskann): validate sector_size on load() and resolve critical smell tags`
  *Validierung der `sector_size` beim Laden bestehender DiskANN-Indices zur Vermeidung lautloser Offset-Fehler.*

### 25. August 2026
- `548b885` | **google-labs-jules[bot]** | `feat: implement enforcement system for LLM-guided development`
  *Einführung automatisierter Governance-Gates für In-Code Anchor- und AI-Tag-Validierung.*
- `a83a165` | **google-labs-jules[bot]** | `ci: optimize context agent system and add enforcement gates`
  *Integration der CI-Schranken (`context-gates.yml`) zur Überprüfung von Sicherheits- und Qualitäts-Tags.*
- `38378aa` | **google-labs-jules[bot]** | `fix(contextra-db): synchronize relate() API with CsrGraph index`
  *Synchronisierung der Beziehungs-API in `contextra-db` mit dem zugrunde liegenden CSR-Graph-Index.*
- `2ba8782` | **google-labs-jules[bot]** | `fix(tauri): audit and verify escapeHtml usage for innerHTML XSS prevention`
  *Sicherheits-Audit der Tauri Frontend-UI bezüglich XSS-Prävention bei HTML-Sanitizing.*
- `79e71bb` | **google-labs-jules[bot]** | `fix(index): complete unsafe SAFETY proofs, DiskANN bounds, and HNSW rebuild test`
  *Vollständige Absicherung aller `unsafe`-Blöcke mit expliziten `SAFETY:` Nachweisen im `contextra-index` Crate.*
- `d7172af` | **google-labs-jules[bot]** | `Audit and verify ONNX session pool and feature flags in contextra-embed`
  *Sicherung der Thread-Sicherheit und Feature-Gating (`--features onnx`) für den ONNX Session Pool.*
- `6c14914` | **google-labs-jules[bot]** | `Audit contextra-crypto cryptographic correctness and integrity`
  *Krypto-Audit für AES-256-GCM-SIV, HKDF-Schlüsselableitung und Zeroize-Garantien.*
- `f637e3a` | **google-labs-jules[bot]** | `Fix CI clippy and doc formatting issues`
  *Behebung von Linter-Warnungen und Formatierungsfehlern in der Dokumentation.*

### 26. August 2026
- `ef2d834` | **google-labs-jules[bot]** | `Harden cryptographic primitives in contextra-crypto`
  *Härtung der kryptographischen Primitiven gegen Seitenkanal-Angriffe und Key-Reuse.*
- `28496bd` | **google-labs-jules[bot]** | `fix(crypto): enforce OsRng nonces, zeroization, and domain separation`
  *Erzwingung kryptographisch sicherer Zufalls-Nonces via `OsRng` und expliziter Domain Separation Tags.*
- `5f15fe6` | **google-labs-jules[bot]** | `Fix HNSW delete error swallowing and TTL reaper edge cases in contextra-db`
  *Behebung des Verschluckens von Löschfehlern im HNSW Vektor-Index und Stabilisierung des Expiry Reapers.*
- `1cf2c55` | **google-labs-jules[bot]** | `fix(text): ensure tokenizer symmetry and BM25 parameter validation`
  *Validierung der BM25-Hyperparameter ($k_1, b$) und Sicherstellung symmetrischer Tokenisierung.*
- `ca75a52` | **google-labs-jules[bot]** | `fix(checkpoint): enforce pin/save/unpin ordering and safety`
  *Durchsetzung der RAII-Reihenfolge (`pin` -> `save` -> `unpin`) für atomare Speicher-Snapshots.*
- `c147c28` | **google-labs-jules[bot]** | `Harden Tauri command input validation and regex size limit`
  *Eingabewert-Validierung für Tauri-IPC-Befehle und Schutz vor ReDoS-Attacken durch Regex-Größenbeschränkung.*
- `e52d3ff` | **google-labs-jules[bot]** | `feat(mcp): enforce JSON-RPC 2.0 protocol compliance in contextra-mcp`
  *Strikte Protokoll-Validierung für den MCP Stdio-Server gemäß JSON-RPC 2.0 Spezifikation.*
- `2de1caa` | **google-labs-jules[bot]** | `Fix MemTable shard selection using full-key BLAKE3 hash`
  *Präzise Verteilung von Einträgen auf MemTable-Shards mittels Vollschlüssel-Hashing.*
- `9b8c256` | **google-labs-jules[bot]** | `enforce lowercase-input invariant on GermanCompoundSplitter::decompose`
  *Erzwingung der Lowercase-Invariante in der deutschen Morphologie-Engine für deterministische Zerlegung.*
- `499682d` | **google-labs-jules[bot]** | `feat(graph): add pure-Rust Session-DAG and CheckpointGuard helper`
  *Implementierung des Pure-Rust Session-DAG für Verzweigungen im Agenten-Status (Grok-Pattern).*
- `f573619` | **google-labs-jules[bot]** | `feat(rag): implement Anthropic Contextual Retrieval pattern`
  *LLM-basierte Präfix-Generierung für Chunks vor der Indizierung zur Reduktion von Retrieval-Fehlern.*
- `f50c3b3` | **google-labs-jules[bot]** | `docs(rag): validate and confirm full implementation of RAG integration sprints`
  *Verifizierung und Dokumentation der RAG-Pipeline-Integration.*
- `c595bee` | **google-labs-jules[bot]** | `feat(mcp): implement stdio MCP server for Claude Desktop`
  *Bereitstellung der primären MCP-Schnittstelle (`contextra_search`, `contextra_insert`, `contextra_get`, `contextra_collections`).*
- `c49aacd` | **google-labs-jules[bot]** | `fix(checkpoint): prevent GC race and TxId collisions`
  *Schutz der Garbage Collection vor Race Conditions bei gleichzeitig aktiven Transaktionen.*

### 27. August 2026
- `969dd1d` | **google-labs-jules[bot]** | `fix(store): enhance LSM durability, concurrency safety, and format`
  *Erhöhung der Durability durch Verzeichnis-Fsyncs (`parent_dir.sync_all()`) und FFi/MVCC Concurrency Safety.*
- `f80fd9a` | **google-labs-jules[bot]** | `fix(contextra-index): escape brackets in check_drift doc comment and format workspace`
  *Korrektur von Markdown-Formatierungsfehlern im Dokumentations-Kommentar von `contextra-index`.*
- `1925004` | **google-labs-jules[bot]** | `feat(xtask): add xtask dev tool for documentation synchronization`
  *Einführung von `cargo xtask sync-docs` zur automatisierten Aktualisierung von `WORKING_STATE.md` und `ARCHITECTURE.md`.*
- `0e3d7bf` | **google-labs-jules[bot]** | `ci: upgrade Gate 5 to enforce sync-docs documentation drift check`
  *CI-Integration der Drift-Prüfung zur Vermeidung veralteter Architektur-Dokumente.*

### 28. August 2026
- `75d6ed9` | **google-labs-jules[bot]** | `feat(contextra-graph): add bi-temporal validity axes to Edge and CsrGraph`
  *Erweiterung des CSR-Wissensgraphen um bi-temporale Zeitachsen (Validitäts- und Transaktionszeit).*
- `fe42626` | **google-labs-jules[bot]** | `fix(security): resolve WAL integrity key TOCTOU (F-07) and SIMD distance checks (F-08/F-09)`
  *Behebung von Time-of-Check to Time-of-Use Schwachstellen beim Erstellen von WAL-Integritätsschlüsseln.*
- `0695e55` | **google-labs-jules[bot]** | `chore(index,db): add TS timestamp to AGT-DB-004 and update AGT-INDEX-002`
  *Aktualisierung der ISO-Zeitstempel in Governance-Tags.*
- `df3ed80` | **google-labs-jules[bot]** | `refactor(core): audit error taxonomy, traits and tx_buffer`
  *Konsolidierung der Fehler-Hierarchie in `ContextraError` und Staging-Kapazitätsgrenzen im Transaktionsbuffer.*
- `bc5610a` | **google-labs-jules[bot]** | `refactor(core): introduce CapabilityUnsupported error variant for trait defaults`
  *Hinzufügen der `CapabilityUnsupported` Fehler-Variante zur sauberen Behandlung nicht-implementierter Trait-Standards.*
- `9626b53` | **google-labs-jules[bot]** | `Unify MetadataFilter into FilterExpr and update docs`
  *Vereinheitlichung der Metadaten-Filterungs-DSL unter `FilterExpr`.*
- `4547237` | **google-labs-jules[bot]** | `refactor(db): rename compaction module to context_compaction`
  *Eindeutige Modulbenennung zur Unterscheidung zwischen LSM-STCS-Compaction und LLM-Kontextkompaktierung.*
- `fc25ca6` | **google-labs-jules[bot]** | `genericize AuditLog over StorageEngine default LsmStorage`
  *Generische Abstraktion von `AuditLog<S: StorageEngine>` zur Unterstützung flexibler Test-Engines.*
- `203af35` | **google-labs-jules[bot]** | `Remove deprecated ContextChunk::combined_text_for_indexing and sync docs`
  *Bereinigung veralteter API-Methoden.*
- `2a1313b` | **google-labs-jules[bot]** | `fix(graph): best-effort non-convergence behavior & log signals for PPR and Community Detection`
  *Best-Effort Rückgabe von Teilergebnissen bei Nicht-Konvergenz von Personalized PageRank und Community Detection mit `tracing::warn!` Signalierung.*
- `e10809b` | **google-labs-jules[bot]** | `feat(core,tauri,py): implement structured ContextraErrorDto for IPC and FFI boundaries`
  *Einführung von `ContextraErrorDto` (`kind`, `message`, `details`) für typsichere Fehlerübertragung über Tauri IPC und PyO3 FFI (ADR-028).*
- `a87fd15` | **google-labs-jules[bot]** | `feat(db): implement full 4-index 2PC transaction commit and rollback`
  *Vollständiger 2-Phase-Commit über HNSW, BM25, CSR-Graph und Metadaten-Indices mit atomarem Rollback.*
- `2b54f9d` | **google-labs-jules[bot]** | `docs: verify governance system hardening baseline (ADR-029)`
  *Verifizierung der Governance-Invarianten und Dokumentation von ADR-029.*
- `2dc334e` | **google-labs-jules[bot]** | `feat(store): implement WAL V3 format with tx_id HMAC binding`
  *Kryptographisch gehärtetes Write-Ahead-Log mit HMAC-SHA256 Bindung pro Transaktions-ID.*

### 29. August 2026
- `b03ec7f` | **google-labs-jules[bot]** | `Harden CheckpointGuard RAII safety and manifest atomicity`
  *Absicherung des RAII CheckpointGuards und atomare Speicherung des Snapshot-Manifests.*
- `575660f` | **google-labs-jules[bot]** | `fix(index): enforce SIMD preconditions, update AGT tags and sync docs`
  *Längenprüfungen vor Aufruf von SIMD-Distanzfunktionen zur Abwehr von Panics.*
- `9274dc6` | **google-labs-jules[bot]** | `fix(ci): synchronize documentation and fix CI gate checks`
  *Korrektur der CI-Gate-Skripte und Dokumentations-Abgleich.*
- `15077e6` | **google-labs-jules[bot]** | `Enforce AGT-GRAPH-001 TxId origin invariant via debug_assert`
  *Erzwingung der Transaktionsursprungs-Invariante im Wissensgraphen.*
- `40c3a23` | **google-labs-jules[bot]** | `fix(checkpoint,store): resolve AGT-CKPT-f3a1b2c4 and AGT-STORE-003`
  *Behebung kritischer Concurrency- und Isolation-Edge-Cases.*
- `000a4d3` | **google-labs-jules[bot]** | `feat: add sequence-based document TTL expiry reaper`
  *Implementierung des Hintergrund-Reapers für automatische TTL-Dokumentenlöschung nach Sequenznummern.*
- `83cc572` | **google-labs-jules[bot]** | `feat(router): implement contextra-router crate for SLM context routing`
  *Einführung der SLM Context Routing Engine für dynamisches Prompt-Routing.*
- `31aed3a` | **google-labs-jules[bot]** | `Audit and enhance contextra-embed and contextra-ollama robustness`
  *Härtung der Ollama- und ONNX-Embedding-Integration gegen API-Timeout und Verbindungsabbrüche.*
- `ae157b5` | **google-labs-jules[bot]** | `fix(mcp): verify insert chunking, add prompt injection guard, zeroize sandbox & cap stdio rpc line size`
  *Umfassendes MCP Security Package: Schutz vor Prompt Injection, Pufferdeckelung und Speicherbereinigung.*
- `f8a9030` | **google-labs-jules[bot]** | `fix(tauri): eliminate startup panic and harden IPC ingestion security`
  *Absicherung der Desktop-App gegen Abstürze bei Start ohne konfigurierten Datenbank-Pfad.*
- `6d931e6` | **google-labs-jules[bot]** | `fix(contextra-py): harden FFI boundaries, error mapping, and GIL concurrency`
  *Freigabe des Python GIL während rechenintensiver Suchen und Mapping auf PyErr-Objekte.*
- `a3f363e` | **google-labs-jules[bot]** | `fix(xtask,bench): resolve check-consistency failure and migration benchmarks dimension mismatch`
  *Korrektur von Dimensionsungleichheiten in Benchmarks.*
- `9bcf07f` | **google-labs-jules[bot]** | `refactor(contextra-db): decouple collection.rs and harden TxId allocation`
  *Dekopplung von `collection.rs` in modulare Submodule (`crud`, `search`, `maintenance`, `relate`).*
- `37aa6a3` | **google-labs-jules[bot]** | `Add MemoryType enum and insert_typed public API`
  *Kognitive Klassifikation von Dokumenten in `Episodic`, `Semantic`, `Procedural` und `Working` Memory (ADR-041).*
- `2c263bf` | **google-labs-jules[bot]** | `feat(core/db): implement Zettelkasten memory links and supersedes displacement`
  *A-MEM Zettelkasten Pattern mit Verknüpfungen und Ersetzungs-Semantik für veraltete Erinnerungen.*
- `0dea264` | **google-labs-jules[bot]** | `refactor(db): modularize collection.rs into submodules`
  *Aufteilung der großen `collection.rs` Datei für bessere Wartbarkeit.*
- `2c6bf35` | **google-labs-jules[bot]** | `Harden contextra-store against silent errors and invalid inputs`
  *Propagation aller I/O-Fehler beim Verzeichnis-Fsync und Vermeidung stummer Resultat-Ignorierung (`let _ =`).*
- `76c1eeb` | **google-labs-jules[bot]** | `Fix Tauri path traversal, ingestion limits, and silent IO check in HNSW`
  *Behebung von Pfad-Traversierungs-Risiken im Tauri File Picker und Ingestion-Limits.*
- `2bf754e` | **google-labs-jules[bot]** | `perf: zero-copy scan_prefix and clone reduction in lsm & collection`
  *Performance-Optimierung durch Zero-Copy Prefix Scanning im LSM-Store.*
- `cc1e5e9` | **google-labs-jules[bot]** | `harden(crypto): enforce non-empty input validation on KeyManager & WalHmac`
  *Eingabe-Validierung gegen leere Schlüssel und Payloads.*
- `c17b7c5` | **tfufuz1** | `Fix/contextra agent state and audit integrity 4394097478157732988 (#1018)`
  *Sicherstellung der Integrität von Agenten-Workflow-Sitzungen und Audit-Logs.*

### 30. August 2026
- `097a134` | **google-labs-jules[bot]** | `fix(store): correct wal tail truncation condition in batch replay`
  *Korrektur der Abbruchbedingung beim Replay beschädigter WAL-Dateien.*
- `a82f0dc` | **google-labs-jules[bot]** | `docs: document ADR-041 for cognitive memory type classification (MemoryType)`
  *Architekturentscheidung für kognitive Gedächtnistypen im System.*
- `d900476` | **google-labs-jules[bot]** | `deprecate Collection::next_tx in favor of allocate_tx`
  *Ersetzung veralteter Transaktions-Allokation durch unfehlbare/fehlerabfangende API.*
- `fb1f918` | **google-labs-jules[bot]** | `feat(graph): PprConfig warn_on_non_convergence, community proptests & xtask gate fix`
  *Einführung von Eigenschafts-Tests (Proptests) für Graph-Community-Detection.*
- `a6034be` | **google-labs-jules[bot]** | `refactor(store): fix batch WAL decryption loop duplication`
  *Deduplizierung des WAL-Entschlüsselungscodes.*
- `bbdf3f2` | **google-labs-jules[bot]** | `fix(lsm): mask TOMBSTONE_BIT in rollback_to_tx`
  *Korrektes Maskieren des Tombstone-Bits bei Transaktions-Rollbacks im LSM-Tree.*
- `bdb3518` | **google-labs-jules[bot]** | `Fix SSTable compaction crash safety via Write-Temp-Then-Rename`
  *Crash-sichere Compaction: Schreiben in `.sst.tmp` Datei und atomares `tokio::fs::rename` nach `file.sync_all()` (ADR-044).*
- `2ff1fa4` | **google-labs-jules[bot]** | `perf(index): move NaN query check to entry point`
  *Vorzeitiger Abbruch bei NaN-Eingabevektoren am Einstiegspunkt der HNSW-Suche.*
- `6a40bca` | **google-labs-jules[bot]** | `fix(mcp): harden contextra-mcp protocol and input validation`
  *Validierung aller Parameter im MCP JSON-RPC Interface.*
- `c800c49` | **google-labs-jules[bot]** | `harden(agent): prevent silent errors and resource exhaustion in contextra-agent`
  *Ressourcenbegrenzung und explicit Result-Unwrapping in Agenten-Schleifen.*
- `390201a` | **google-labs-jules[bot]** | `Harden contextra-ollama against silent errors and resource bounds`
  *Härtung des Ollama-Clients gegen unbegrenzte HTTP-Antworten.*
- `c028100` | **google-labs-jules[bot]** | `harden(contextra-text, contextra-db): add input guards and batch boundary checks`
  *Grenzbereich-Validierung bei Batch-Operationen im Invertierten Index.*
- `b084da5` | **google-labs-jules[bot]** | `fix(ci): resolve context-gates review-coverage failure and compilation issues`
  *Behebung von CI-Fehlern bezüglich Review-Coverage-Prüfungen.*
- `43eed69` | **google-labs-jules[bot]** | `harden(checkpoint): input validation, resource caps, lock hierarchy & REVIEW-PASS`
  *Konsolidierung der Sperr-Hierarchien zur Deadlock-Vermeidung im Checkpoint-Manager.*
- `d7ecd28` | **google-labs-jules[bot]** | `feat(xtask): extend check-consistency with README, AGENTS.md, and ADR checks`
  *Erweiterung des xtask Konsistenz-Checkers um Validierung von README Crate-Zahlen und AGENTS.md Existenz.*
- `37a2fab` | **google-labs-jules[bot]** | `harden(agent): add zero-panic deprecations, input validation, and review pass tags`
  *Zero-Panic Garantie in `contextra-agent` durch vollständiges Entfernen ungeschützter `.unwrap()` Aufrufe in Production-Pfade.*
- `6b540a7` | **google-labs-jules[bot]** | `refactor(core): fulfill ANCHOR[TEST:CORE-001], consolidate headers & sync docs`
  *Erfüllung der Core-Test-Anforderungen und Synchronisation der Arbeitsstände.*

### 31. August 2026
- `5b067ad` | **tfufuz1** | `refactor(index): audit and clean up contextra-index code quality (#1150)`
  *Umfassendes Audit und Bereinigung von `contextra-index`: Infallible Float-Konvertierungen (`f32::from`), Inlined Format Arguments, Validierung von NaN/Inf Query-Vektoren in `HnswIndex::search`, Aktualisierung der Session-Hashes.*

### 1. September 2026
- `1d38f70` | **tfufuz1** (Co-authored-by **google-labs-jules[bot]**, **tfufuu**) | `Audit Report: contextra-agent token budget race condition analysis (#1239)`
  *Audit-Bericht (`docs/audits/round2/AUDIT_contextra-agent_budget-race.md`) und Integrationstest (`crates/contextra-agent/tests/budget_race_test.rs`) zur Analyse der Sequenzschritt-Isolierung und Ermittlung von Nicht-Atomaren TokenBudget RMW Race Vectors in `contextra-agent`.*

### 2. September 2026
- `881ec05` | **google-labs-jules[bot]** | `harden(crypto): complete REVIEW-PASS and verified zero-unsafe invariants`
  *Abschluss der unabhängigen Review-Pässe für `contextra-crypto` und Verifizierung der Zero-Unsafe-Invarianten in Produktionsmodulen.*
- `963f93c` | **google-labs-jules[bot]** | `refactor(core): complete review pass for AGT-CORE-a3f29c1d`
  *Abschluss der Code-Review-Verifikation für Core-Typen und Staging-Buffer in `contextra-core`.*
- `358e3b0` | **google-labs-jules[bot]** | `fix(checkpoint): resolve concurrent pinning race and complete TEST:CKPT-001`
  *Behebung von Concurrent-Pinning-Races im Checkpoint-Manager und Validierung unter extremer Stressbelastung.*

### 3. September 2026
- `a413a59` | **google-labs-jules[bot]** | `refactor(crypto): resolve AGT-CRYPTO-dd984bc2 and AGT-CRYPTO-7519b7cd anti-tamper tags`
  *Refactoring der Zeroize-Drop-Tests und Absicherung konstanter Zeitvergleiche in Anti-Tamper-Schutzmechanismen.*
- `94a6a82` | **google-labs-jules[bot]** | `fix(py): resolve AGT-PY-ff475c8e run_blocking_ffi panic boundary`
  *Sichere Fehlerbehandlung an der PyO3 FFI-Grenzschicht zur Verhinderung unbeabsichtigter Panics in CPython-Threads.*
- `570a339` | **google-labs-jules[bot]** | `fix(router): resolve AGT-ROUTER-2db4f208 and persist conformal calibration state`
  *Optimierung von Router-Lookup-Zeiten (O(1) HashSet) und Implementierung persistenter Konform-Kalibrierung über System-Neustarts.*
- `8712461` | **google-labs-jules[bot]** | `refactor(text): complete multi-session REVIEW-PASS for German morphology (TEST:TXT-001)`
  *Abschluss des Multi-Session-Reviews für deutsche Umlaut- und Zusammensetzungs-Morphologie.*
- `6da6a1c` | **google-labs-jules[bot]** | `audit(embed): complete chaos engineering and feature-gate review pass`
  *Abschluss des Fault-Tolerance-Audits im Cross-Encoder-Reranker-Modul unter Chaos-Engineering-Bedingungen.*

### 4. September 2026
- `d01ee97` | **google-labs-jules[bot]** | `fix(db): resolve AGT-DB-2f1b6962 query builder config propagation`
  *Korrektur der Konfigurationsweiterleitung für verdrängte Erinnerungen (`include_superseded`) im `HybridQueryBuilder`.*
- `3e5150c` | **google-labs-jules[bot]** | `audit(embed): complete REVIEW-PASS on CrossEncoderReranker passthrough fallback`
  *Verifizierung der Fallback-Pfade und Kandidaten-Limits im Reranker.*
- `9c38447` | **google-labs-jules[bot]** | `refactor(index): resolve AGT-INDEX-002 and stabilize SIMD distance intrinsics (ADR-047)`
  *Stabilisierung der SIMD-Distanzberechnungen mit `std::arch` Intrinsics und Laufzeit-CPU-Feature-Erkennung; Dokumentations-Sync auf 0 offene Tags.*
- `f31c5bc` | **google-labs-jules[bot]** | `audit(store): Tier 1 Deep Audit & Verification pass with GO verdict`
  *Umfassende Verifizierung von `contextra-store`: Fsync-Disziplin, Mmap-Prüfungen, Fault Injection, Amplification-Benchmarks und Eintragung in `AUDIT_contextra-store.md`.*
- `3e4e9b0` | **google-labs-jules[bot]** | `audit(core): Tier 1 Deep Audit & Verification pass with GO verdict`
  *Umfassende Verifizierung von `contextra-core`: 100% Pass-Rate bei 139 Unit-Tests, SnapshotRegistry GC Stress-Tests, TxId Boundary-Simulation und Eintragung in `AUDIT_contextra-core.md`.*

### 5. September 2026
- `f6c2b43` | **google-labs-jules[bot]** | `fix(cargo): update default feature flags and remove dead cluster feature (#1545)`
  *Bereinigung veralteter Feature-Flags und Entfernung nicht genutzter Cluster-Spezifikationen im Workspace Cargo.toml.*
- `b8d910e` | **google-labs-jules[bot]** | `refactor(store,wal): implement 3-phase async LSM flush and WAL HMAC chaining fixes`
  *3-Phasen entkoppelte LSM-Flushes ohne Read-Lock-Blockaden (ADR-059), Instanz-gebundene `flush_counter` (ADR-060) und atomic Sidecar WAL HMAC-Absicherung (KRIT-02–04).*
- `a1098ef` | **google-labs-jules[bot]** | `refactor(graph): async CsrGraph add_edge lock-splitting`
  *Entkopplung der Graph-Kompaktierung aus dem Schreib-Lock mittels zweiphasigem `add_edge` für minimale Lock-Latenz.*
- `c7e2194` | **google-labs-jules[bot]** | `refactor(crypto,core): decouple contextra-crypto architecture`
  *Architektonische Entkopplung von `contextra-crypto` und `contextra-core`: Eigenständige `CryptoError` Hierarchie und sauberes Trait-Mapping.*

### 6. September 2026
- `f8d12db` | **google-labs-jules[bot]** | `feat(crypto): add DeletionProof for GDPR compliance verification`
  *Kryptographischer Löschnachweis zur Erfüllung von DSGVO Artikel 17 (Recht auf Vergessenwerden) über Speicherschichten hinweg.*
- `cdd3e15` | **google-labs-jules[bot]** | `feat(core,store): implement TenantId and TenantKeyCodec for multi-tenancy isolation`
  *Einführung von `TenantId` und `TenantKeyCodec` zur strikten Mandantentrennung auf Speicherebene (INV-TENANT).*
- `f19b20a` | **google-labs-jules[bot]** | `Fix agent loop atomicity ordering and put_kv_if_absent TOCTOU race (BEFUND-26, BEFUND-27)`
  *Behebung von Race-Conditions im Agenten-Loop und atomares `put_kv_if_absent` zur Vermeidung von TOCTOU-Schwachstellen.*
- `631a6c5` | **google-labs-jules[bot]** | `feat(graph): implement PathRAG Engine in contextra-graph`
  *Implementierung der PathRAG Engine für mehrstufige Pfad-RAG Traversierungen über CSR-Kanten mit Relevanz-Filtern.*
- `500aa8a` | **google-labs-jules[bot]** | `feat(db): implement Free Energy Thermostat (F-01) adaptive decay`
  *Implementierung des Free Energy Thermostats zur mathematischen Regulierung von Gedächtnis-Decay und Verdrängungsdynamiken.*
- `35a890d` | **google-labs-jules[bot]** | `feat(index): implement DiskANN persist_delta and update unwrap baseline`
  *DiskANN Inkrementelle Delta-Persistierung zur Reduktion von Festplatten-Schreiblast bei Inkrement-Updates.*
- `bc9f5e6` | **google-labs-jules[bot]** | `fix(diskann): add WAL-backed pending buffer and auto-flush recovery`
  *Absicherung von DiskANN gegen Datenverlust nach unvorhergesehenem Absturz mittels WAL-gepuffertem Puffer.*
- `ef09c86` | **google-labs-jules[bot]** | `Fix FusionWeights default to balanced 3-signal fusion`
  *Standardausrichtung der FusionWeights auf ausbalanciertes 3-Signal Hybrid Retrieval (Vector + Text + Graph).*
- `3b0d7b7` | **google-labs-jules[bot]** | `feat(db): implement adaptive fusion weights via replicator dynamics`
  *Dynamische Anpassung von RRF-Signal-Gewichten auf Basis von evolutionärer Replikatordynamik.*
- `ab3685b` | **google-labs-jules[bot]** | `feat(contextra-graph): implement F-06 percolation health monitor and re-bonding trigger`
  *Perkolations-Gesundheitsmonitor zur Erkennung von Wissensnetzwerk-Fragmentierung und automatischem Re-Bonding.*
- `85f35be` | **google-labs-jules[bot]** | `feat(calibration): update unwrap baseline for Gate 2 and add contextra-calibration crate`
  *Einführung der `contextra-calibration` Crate für Isotonische und Platt Conformal Calibration von Relevanz-Scores.*

### 7. September 2026
- `a413a598` | **tfufuz1** | `feat(kv-bridge): implement tenant-isolated KV-Segment store and eviction worker`
  *Erstellung der `contextra-kv-bridge` Crate für hochperformantes, mandantenisoliertes Caching mit `ZeroizeOnDrop` Garantien.*
- `f04imm01` | **google-labs-jules[bot]** | `feat(index): implement streaming DiskANN with beam search & RNG pruning`
  *Echte inkrementelle Streaming-DiskANN Implementierung mit Beam-Search, RNG-Pruning und Rückwärts-Kanten-Kompression.*
- `f179f54` | **google-labs-jules[bot]** | `fix(mcp): reduce MAX_RPC_BYTES to 4 MB for embedded DoS hardening`
  *Reduktion des maximalen MCP JSON-RPC Pufferlimits von 16 MB auf 4 MB zur Abwehr von DoS-Attacken.*
- `05b382d` | **tfufuz1** (Co-authored-by **google-labs-jules[bot]**, **tfufuu**) | `feat(graph): implement cascading edge invalidation for superseded chunks (#1726)`
  *Kaskadierende Kanten-Invalidierung im CSR-Wissensgraphen bei Verdrängung veralteter Dokumenten-Chunks (`cascade.rs`, `INV-GRAPH-PROV-1`).*

### 9. September 2026
- `f34f7e6` | **google-labs-jules[bot]** | `fix(store): restore WAL last_hmac on append_batch failure`
  *Sicherstellung der HMAC-Chaining Integrität: Wiederherstellung von `last_hmac` bei Batch-Fehlern.*
- `d555843` | **google-labs-jules[bot]** | `contextra-checkpoint: tier 2 deep audit & fix check-placeholder-refs anchor check`
  *Verifizierung von `contextra-checkpoint` und Härtung des ADR-Anker-Checkers.*
- `097095b` | **google-labs-jules[bot]** | `fix(xtask): update unwrap baseline to pass context-gates`
  *Synchronisation der unwrap-Baseline zur Durchsetzung von Gate 2.*
- `c888d85` | **google-labs-jules[bot]** | `contextra-agent: complete tier 2 deep audit and fix check-placeholder-refs regex`
  *Tiefenaudit von `contextra-agent` und Korrektur der RegEx-Prüfung für Platzhalter-Referenzen.*
- `8dee081` | **google-labs-jules[bot]** | `contextra-kv-bridge: perform Tier 3 deep audit and expand test suite`
  *Vollständige Abdeckung und Auditierung der `contextra-kv-bridge` Crate.*
- `eda2104` | **google-labs-jules[bot]** | `contextra-graph: fix CI check-placeholder-refs and complete Tier 2 deep audit`
  *Audit von `contextra-graph` inklusive PfadRAG und kaskadierender Kanten-Invalidierung.*
- `a429cf5` | **google-labs-jules[bot]** | `contextra-crypto: deep audit verification, audit report update, and xtask fix`
  *Tiefe Krypto-Verifizierung und Abgleich der Zeroization-Regeln.*
- `3b68809` | **google-labs-jules[bot]** | `contextra-calibration: deep audit, proptest & integration test suite`
  *Einführung umfassender Proptests und Integrationstests in `contextra-calibration`.*
- `af02417` | **google-labs-jules[bot]** | `xtask: fix check-placeholder-refs anchor handling for DECISIONS.md`
  *Behebung von Anker-Auflösungsfehlern im xtask Validator.*
- `126d701` | **google-labs-jules[bot]** | `xtask: make ADR regex case-insensitive in check_placeholder_refs`
  *Robuste RegEx-Erkennung von ADR-Bezeichnungen in Dokumenten.*
- `d6adaa4` | **google-labs-jules[bot]** | `contextra-bench: complete deep audit and fix xtask check-placeholder-refs`
  *Tiefenaudit der Benchmark-Engine `contextra-bench`.*
- `4fb361f` | **google-labs-jules[bot]** | `fix(deps): add contextra-crypto workspace dependency alias`
  *Sauberes Workspace-Aliasing für die umbenannte Sicherheits-Crate.*
- `4f76c86` | **google-labs-jules[bot]** | `docs: consolidate v10/v10.1 spec, remove root duplicate, fix stale OFFEN-11 reference`
  *Konsolidierung der Gesamtspezifikation v10.1 und Entfernung veralteter Referenzen.*
- `9466a8a` | **google-labs-jules[bot]** | `fix(db): update contextra-db dependency from contextra-crypto to contextra-security`
  *Aktualisierung der Crate-Abhängigkeiten auf `contextra-security`.*
- `2d37e40` | **google-labs-jules[bot]** | `refactor(contextra-db): remove biological metaphor terminology (sleep-cycle/thermostat/reaper)`
  *Bereinigung biologischer Begriffe in Produktivcode und Tests zugunsten präziser technischer Bezeichnungen (Consolidation, PID Controller, Expiry Engine).*
- `e4001be` | **google-labs-jules[bot]** | `contextra-db: tier-1 deep audit, workspace manifest compatibility, and gate checks`
  *Tier 1 Deep Audit Verifizierung für `contextra-db`.*
- `3bdb693` | **google-labs-jules[bot]** | `contextra-candle: audit verification, FILE-CONTEXT headers, and TenantId compatibility`
  *Kompatibilitätsprüfung von `contextra-candle` für `TenantId`.*
- `e4aff61` | **google-labs-jules[bot]** | `contextra-bench: fix TenantId as_u64 build break & expand test coverage`
  *Behebung von Breaking Changes bei `TenantId` Methodenaufrufen in Benchmarks.*
- `478adde` | **google-labs-jules[bot]** | `contextra-embed: resolve clippy large_enum_variant and fix stale doc reference`
  *Speicher-Optimierung in `contextra-embed` durch Verkleinerung von Enums.*
- `580d1e6` | **google-labs-jules[bot]** | `fix(context-gates): update unwrap baseline with new test assertions`
  *Aktualisierung der `.unwrap-baseline.json` für alle neuen Testfällungssätze.*
- `c1fd3b4` | **google-labs-jules[bot]** | `contextra-mcp: add json-rpc edge-case deserialization and error tests`
  *Edge-Case Testing für MCP Deserialisierung und Fehlercodes.*
- `fefe462` | **google-labs-jules[bot]** | `fix(xtask): enforce dynamic review coverage and fix working state date calculation`
  *Dynamische Abdeckungs-Berechnung in `xtask`.*
- `64050ae` | **google-labs-jules[bot]** | `contextra-core: fix governance tag taxonomy reference and update unwrap baseline`
  *Anpassung der Tag-Taxonomie an die Verfassungsregeln.*

### 10. September 2026
- `58ae196` | **google-labs-jules[bot]** | `fix(ci): fix HF_TOKEN header, scan limit, and text truncation in bench`
  *Absicherung von HuggingFace Token-Headern und Behebung von Text-Kürzungsfehlern in Benchmark-Läufen.*
- `d5ec7e7` | **google-labs-jules[bot]** | `docs(jules): add Google-Jules VM Git checklist and optimization plan`
  *Bereitstellung von Entwickler-Gleitpfaden für automatisierte Agenten-Umgebungen.*
- `4fd42e7` | **google-labs-jules[bot]** | `feat(xtask): add claim --release, TTL expiry (4h), and claim DB reset`
  *Erweiterung des Claiming-Systems in `xtask` zur koordinierten Multi-Agenten Auditierung.*
- `e7f744b` | **google-labs-jules[bot]** | `feat(xtask): add check-phantom-files gate (Gate 12b)`
  *Neues CI Gate zur Erkennung verwaister oder nicht mehr referenzierter temporärer Dateien.*
- `9a05208` | **google-labs-jules[bot]** | `fix(store): resolve compile errors, CI gates and durability invariants in LSM storage`
  *Härtung der Durability Invarianten im LSM Storage.*
- `5b4dbb3` | **google-labs-jules[bot]** | `fix(store): resolve compile errors and LSM tree durability/consistency bugs`
  *Beseitigung von Konsistenzfehlern in MemTable und SSTable Wechselwirkungen.*
- `77ffdbe` | **google-labs-jules[bot]** | `prompter & store: add rebase-retest gate and propagate WAL recovery sync error`
  *Einbetten des Rebase-Retest Gates in xtask.*
- `c9bdada` | **google-labs-jules[bot]** | `docs(calibration): update contextra-calibration audit report for 2026-09-10 session`
  *Dokumentationsupdates für Konform-Kalibrierung.*
- `fab2f5c` | **google-labs-jules[bot]** | `contextra-core & contextra-store: update audit report and fix silent IO in wal.rs`
  *Behebung stummer I/O-Fehler bei `sync_all` Aufrufen im WAL Modul.*
- `672afd0` | **google-labs-jules[bot]** | `contextra-store: propagate errors on sync_all in recovery`
  *Erfolgreiche Weiterleitung aller I/O Fehler während der Recovery-Phase.*
- `659ee03` | **google-labs-jules[bot]** | `fix(store): propagate sync_all error in wal.rs recovery to satisfy Gate 3`
  *Erfüllung von CI Gate 3 (Keine ignorierten I/O-Aufrufe in Wiederherstellungspfaden).*
- `692382f` | **google-labs-jules[bot]** | `docs(contextra-py): verify layer 3 pyo3 bindings audit findings for session 0b2ff57d`
  *Verifikation der Python-Schnittstelle.*
- `493a456` | **google-labs-jules[bot]** | `wal: handle sync_all error logging during backup recovery`
  *Strukturiertes Tracing bei Backup-Recovery Ausfällen.*
- `fbf8799` | **google-labs-jules[bot]** | `docs(tags): format TODO comments according to AI-TAG taxonomy and ISO-8601 rules`
  *Automatisierte Bereinigung aller freien TODO-Kommentare gemäß AI-TAG Taxonomie.*
- `c28e969` | **google-labs-jules[bot]** | `contextra-db: sanitize sandbox bridge clippy and update audit logs`
  *Clippy-Bereinigung in Sandbox-Befehlen.*
- `cf98dea` | **google-labs-jules[bot]** | `contextra-checkpoint: complete session audit & doc sync`
  *Abschluss des Audit-Zyklus für Checkpoints.*
- `0acd010` | **google-labs-jules[bot]** | `contextra-index: fix clippy warning and update audit report`
  *Clippy-Fixes im Vektor-Index.*
- `6021aa6` | **google-labs-jules[bot]** | `contextra-embed: fix Gate 3 silent I/O in wal.rs and update unwrap baseline`
  *Beseitigung stummer I/O Zugriffe in Embedder-Initialisierungen.*
- `cb78f23` | **google-labs-jules[bot]** | `contextra-text: add unit tests for stats persistence, clone sharing, and UTF-8 slicing`
  *Sicherheits-Tests für UTF-8 Char Boundaries und Statistiken-Persistenz im Invertierten Index.*
- `9e0430f` | **google-labs-jules[bot]** | `fix(store): ensure LSM startup flush, WAL sidecar cleanup, and fix silent I/O in wal.rs`
  *Automatische WAL Sidecar Bereinigung beim Systemstart und Erzwingung des MemTable Flushes.*
- `ff3e898` | **google-labs-jules[bot]** | `fix(store): resolve LSM operational bugs and eliminate new test unwraps`
  *Beseitigung aller ungeprüften `unwrap` Aufrufe in neuen LSM-Tests.*

### 11. September 2026
- `4d8a25f` | **google-labs-jules[bot]** | `contextra-text: verify audit pass, gate-stack, and docs sync`
  *Gate-Stack Validierung und Dokumentations-Synchronisation für `contextra-text`.*
- `cb9c6a0` | **google-labs-jules[bot]** | `ci(bench): update HuggingFace filename parameter for LongMemEval-S download`
  *Aktualisierung der Download-Parameter für LongMemEval-S im Benchmark-Runner.*
- `6edf621` | **google-labs-jules[bot]** | `contextra-candle: complete Tier 3 deep audit and fix inverted index compilation in contextra-text`
  *Tier 3 Deep Audit für `contextra-candle` und Reparatur der Kompilierungsabhängigkeiten in `contextra-text`.*
- `919498a` | **google-labs-jules[bot]** | `contextra-checkpoint: deep audit and tier 2 recovery verification`
  *Deep Audit & Recovery-Verifikation im Checkpoint-Crate.*
- `fa77992` | **google-labs-jules[bot]** | `docs: add doc-ref-ignore for wildcard path reference in DECISIONS.md`
  *Anpassung der Dokumentations-Referenzregeln zur Vermeidung falscher Positiver bei Wildcards.*

### 12. September 2026
- `d3e53ca` | **google-labs-jules[bot]** | `docs(audit): audit report for contextra-calibration`
  *Erstellung und Verifizierung des Audit-Berichts `AUDIT_contextra-calibration.md`.*
- `f3a9588` | **tfufuz1** (Co-authored-by **google-labs-jules[bot]**, **tfufuu**) | `audit: systematischer audit contextra-text (#2243)`
  *Systematischer Tiefenaudit von `contextra-text`: Reverifikation von BM25 Robertson-Spärck-Jones IDF, Char-Boundary Sicherheit beim Token-Slicing, sowie 100% Verifikation aller Crate-Module (`inverted.rs`, `tokenizer.rs`, `lib.rs`, `morphology.rs`).*

### 13. September 2026
- `a7b1892` | **google-labs-jules[bot]** | `feat(mcp): wire live router and calibrator stats to Contextra::stats()`
  *Integration dynamischer Observability-Metriken aus `contextra-router` und `contextra-calibration` in den MCP Stdio-Server via `setup_routing()`.*
- `89f2134` | **google-labs-jules[bot]** | `feat(sandbox): enforce WASM fuel and memory bounds in contextra-sandbox`
  *Designation von `contextra-sandbox` als Layer 2/6.5 WASM Execution Boundary (Spec v5 §4.18) mit Tests für Fuel-Exhaustion und Memory-Isolation.*
- `91024bc` | **google-labs-jules[bot]** | `fix(xtask): refine audit lints for unbounded bounds and NaN checks`
  *Verfeinerung der statischen Audit-Lints in `xtask` (`check_max_results_unbound`, `check_nan_validation_in_hot_loop`) zur Eliminierung falscher Positiver durch RegEx-Kontextfenster.*

### 14. September 2026
- `9a2e914` | **google-labs-jules[bot]** | `refactor(router): make RouterEngine generic over StorageEngine`
  *Dekopplung von `contextra-router` (Layer 7) von `contextra-store` durch Umstellung von `RouterEngine` auf `RouterEngine<S: StorageEngine>`. Verschiebung von `contextra-store` in `[dev-dependencies]`.*
- `4d64a08` | **google-labs-jules[bot]** | `contextra-store: SDLC Phase 3 REVIEW audit and findings report (#2856)`
  *Tiefenaudit der LSM Group-Commit Phase: Identifikation und Korrektur des Race-Condition-Risikos im `last_hmac` Kettenglied beim Freigeben von Locks vor I/O.*
- `6d69390` | **google-labs-jules[bot]** | `Audit contextra-checkpoint architecture and panic safety guarantees`
  *Konsolidierung der doppelten `RwLock` Indizes (`checkpoints`, `name_index`) in `PersistentCheckpointStore` zu einer einzigen atomaren `RwLock<CheckpointIndex>` Struktur.*
- `734d34b` | **google-labs-jules[bot]** | `audit(calibration): complete deep audit of contextra-calibration crate`
  *Tiefenaudit der Isotonischen Regression (PAVA) und Platt-Scaling Algorithmen mit P99 Latenz-Garantien.*

### 15. September 2026
- `ae756a1` | **google-labs-jules[bot]** | `docs: add architectural analysis for cloud-egress privacy gateway §12`
  *Fünf-Schichten Vollständigkeitsaudit des Cloud-Egress Privacy Gateways in `docs/audits/AUDIT_cloud_egress_five_layer_completeness.md`.*
- `3f83a18` | **google-labs-jules[bot]** | `docs(audit): complete deep audit of contextra-embed layer 4`
  *Architektur- & Robustheits-Audit für `contextra-embed` mit Nachweis von Zero Unsafe Code, Spawn-Blocking Thread-Isolierung und Domain Scoring Invarianten.*
- `84f95d8` | **google-labs-jules[bot]** | `docs: complete cross-crate durability path audit report`
  *Umfassende Verifikation der Speicherdurabilität (WAL HMAC Chaining, 2PC Multi-Index Sync, Fsync-Disziplin).*
- `a49deca` | **google-labs-jules[bot]** | `docs(audit): complete architectural audit for contextra-core (Layer 0)`
  *Tiefenaudit von `contextra-core`: Validierung der Trait-Submodule (`traits/`), Error DTOs und Domain-Typen.*
- `70a9ea4` | **google-labs-jules[bot]** | `docs(audit): complete deep architecture audit for contextra-core-ipc-gen`
  *Validierung der FlatBuffers IPC Generierung und Zero-Copy Deserialisierungs-Schranken.*
- `48b9605` | **google-labs-jules[bot]** | `docs(router): complete architectural audit for contextra-router layer 6`
  *Audit des LinUCB Contextual Bandits in `bandit.rs`: Behebung von Debug-Assert Panics und Rekalibrierung der $\theta$-Parameter.*
- `28238b5` | **google-labs-jules[bot]** | `docs: complete contextra-mcp architecture and security audit report`
  *Tiefenaudit von `contextra-mcp`: Pufferdeckelung, Prompt Injection Quarantäne und Refactoring des Egress Gateways auf den offiziellen `EgressVault` Kontrakt.*
- `cc2faa5` | **google-labs-jules[bot]** | `docs: add architectural audit report for contextra-index`
  *Refactoring des HNSW-Einfügepfads auf den Compute-then-Commit Ansatz: Trennung in read-only `compute_insert` und atomares `apply_insert` zur Verhinderung von Phantomen.*
- `4136460` | **google-labs-jules[bot]** | `docs(graph): deliver comprehensive micro-fine architectural audit report for contextra-graph`
  *Refactoring der Staging-Strukturen im CSR-Graphen (`staged_entities`, `staged_edges`) auf flache `AHashMap` mit Composite Keys `(TxId, EntityId)`.*

### 16. September 2026
- `f459e93` | **google-labs-jules[bot]** | `docs(contextra-core): synchronize governance and architecture docs`
  *Synchronisierung aller Governance-Dokumente (`ARCHITECTURE.md`, `WORKING_STATE.md`) via `cargo xtask sync-docs`.*
- `7d87a7b` | **google-labs-jules[bot]** | `docs(core): sync governance docs and audit log for contextra-core`
  *Aktualisierung des Audit-Protokolls für Layer 0.*
- `f8d01be` | **google-labs-jules[bot]** | `contextra-core: docs sync and governance audit update`
  *Audit-Update für `contextra-core` Submodule.*
- `2d6d9ac` | **google-labs-jules[bot]** | `contextra-core: docs sync and governance audit report for 2026-09-17`
  *Governance Audit & Sync.*
- `8c199f6` | **google-labs-jules[bot]** | `docs(contextra-core): sync governance documentation and record audit findings`
  *Dokumentation der Trait-Submodul-Entkopplung.*
- `4d67c95` | **google-labs-jules[bot]** | `contextra-core: sync governance docs and record inventory audit`
  *Abgleich des Crate-Inventars von `contextra-core`.*
- `21e22bc` | **google-labs-jules[bot]** | `docs(contextra-core): sync governance documentation and audit report`
  *Governance-Dokumentationsabgleich.*
- `f8362f5` | **google-labs-jules[bot]** | `docs(contextra-core): sync governance docs and record inventory audit (#3015)`
  *Pull-Request Merge: Governance & Inventar-Synchronisation für Layer 0.*
- `c052b9c` | **google-labs-jules[bot]** | `contextra-core: sync governance docs and audit report`
  *Sicherstellung der Invarianten in `contextra-core`.*
- `05e744c` | **google-labs-jules[bot]** | `docs(core): sync governance and architecture documentation`
  *Systemweiter Architektur-Dokumentations-Sync.*
- `cfc2403` | **google-labs-jules[bot]** | `docs(core): sync governance documentation and audit report`
  *Audit-Protokollierung für Core-Typen.*
- `7ba44b3` | **google-labs-jules[bot]** | `docs(contextra-core): sync governance docs and record inventory audit`
  *Abschluss der Layer 0 Governance Synchronisation.*
- `837b7ba` | **google-labs-jules[bot]** | `docs(checkpoint): document consolidated CheckpointIndex verification`
  *Dokumentation des konsolidierten Checkpoint-Index.*
- `725f594` | **tfufuz1** | `Harden WAL replay bounds and implement transaction intent recovery`
  *Härtung der WAL Batch-Replay Grenzen und Implementierung transaktionaler Intent-Recovery im LSM-Store.*

### 17. September 2026
- `e4a2109` | **google-labs-jules[bot]** | `perf(store,db): optimize MemTable and RRF fusion allocations`
  *Performance-Optimierung: Kapazitätsreservierung `.or_insert_with(|| Vec::with_capacity(2))` in `memtable.rs` und String-Internierung (`&str` IDs) mit SoA Score-Akkumulation in `fusion.rs`.*
- `b817c2d` | **google-labs-jules[bot]** | `feat(index): add fuzz_hnsw_persistence fuzz target`
  *Einführung des Fuzzing-Targets `fuzz_hnsw_persistence` in `crates/contextra-index/fuzz/` zur Überprüfung von SIMD-Ausrichtung, Float-Sanitizing und Mmap Corruption Roundtrips.*
- `c512a80` | **google-labs-jules[bot]** | `feat(xtask): add lint-unsafe-slices subcommand for AST SIMD bounds checking`
  *Einführung von `xtask lint-unsafe-slices` zur automatisierten AST-Analyse von Slice-Längenprüfungen vor SIMD-Loads in `unsafe fn`s.*
- `d294821` | **google-labs-jules[bot]** | `refactor(core): replace non-atomic put_if_absent trait default with CapabilityUnsupported`
  *Ersetzung des nicht-atomaren `put_if_absent` Trait-Defaults in `StorageEngine` durch `ContextraError::CapabilityUnsupported` zur Erzwingung atomarer Concurrency Control in Implementierungen.*

### 18. September 2026
- `95da5e2` | **tfufuz1** | `feat(xtask): add post-merge report and workflow integration (#3109)`
  *Integration des automatisierten Post-Merge Reports in `xtask` (`xtask post-merge-report`) zur kontinuierlichen Generierung von Zusammenfassungen nach Pull Request Merges.*

---

## 3. Tiefere Differenz- & Fehler-Analysen (Vorher vs. Nachher)

Um die Evolution und Behebung aller kritischen Systemfehler transparent und nachvollziehbar darzulegen, folgt eine strukturierte Gegenüberstellung nach Fachdomänen:

### A. Storage Engine & Crash Safety (LSM, WAL, SSTables)
- **Fehler / Schwachstelle**: WAL HMAC Sidecar Race & TOCTOU (`F-07`).
  *Vorher*: WAL-Integritätsschlüssel wurden nicht-atomar vor der Erstellung der WAL-Datei geprüft. Bei plötzlichem Stromausfall konnte eine teilweise geschriebene Sidecar-Datei erzeugt werden, was beim Neustart zu Korruptionsfalsch-Positiven führte.
  *Ursache*: Fehlende atomare Dateierstellung via OS-Flags (`O_EXCL` / `create_new(true)`).
  *Nachher (Lösung)*: Erstellung des WAL HMAC Sidecars mittels `create_new(true)` under exklusivem Lock mit atomarer In-Place HMAC-Aktualisierung.
- **Fehler / Schwachstelle**: Ignorierte I/O-Fehler bei Recovery `sync_all` Aufrufen (Gate 3).
  *Vorher*: Fehler beim Erzwingen von Dateisystem-Syncs (`file.sync_all()`) während der WAL-Wiederherstellung wurden stumm mit `let _ =` verworfen.
  *Ursache*: Fehlende I/O-Fehler-Propagation im Replay-Loop.
  *Nachher (Lösung)*: Strikte Propagation aller `sync_all()` Fehler via `Result<()>` und `ContextraError::Io`.
- **Fehler / Schwachstelle**: Read-Lock Blockaden bei LSM Flush (ADR-059).
  *Vorher*: Während `LsmStorage::flush()` wurden lesende Abfragen blockiert, da der Read-Lock auf den LSM-Tree gehalten wurde, während langsame I/O-Operationen (MemTable to SSTable Disk Write) liefen.
  *Ursache*: Monolithischer Flush-Ablauf innerhalb einer ungestaffelten Sperre.
  *Nachher (Lösung)*: 3-Phasen Lock-Free Async LSM Flush:
    1. Phase 1: In-Memory Freeze der aktiven MemTable in eine Immutables-List unter kurzer Sperre.
    2. Phase 2: Async Schreiben der SSTable-Datei auf Festplatte völlig ohne Sperre.
    3. Phase 3: Kurzer Commit-Lock zur Entfernung der Immutable MemTable und Aktualisierung des Manifests.
- **Fehler / Schwachstelle**: Group-Commit Leader Mutex Contention während Disk I/O.
  *Vorher*: Der Group-Commit Leader hielt die `commit_mutex` während der gesamten physikalischen `wal.append_batch` Schreiboperation auf die Festplatte, was nachfolgende Commits unnötig blockierte.
  *Ursache*: Zu breiter Geltungsbereich der Mutex-Sperre über I/O-Grenzen hinweg.
  *Nachher (Lösung)*: Freigabe von `commit_mutex` vor der physikalischen Disk I/O. Bei I/O-Fehlern wird die Mutex erneut erworben, um die HMAC-Kette wiederherzustellen und `rollback_to_tx_locked` auszuführen.
- **Fehler / Schwachstelle**: Speicherallokations-Ineffizienz in MemTable und Multi-Version-Vektoren.
  *Vorher*: Das Einfügen in die MemTable nutzte `.or_default()`, was bei Schlüsselversionierungs-Vektoren zu häufigen Reallokationen führte.
  *Ursache*: Default-Vektorallokation mit Kapazität 0.
  *Nachher (Lösung)*: Ersetzung durch `.or_insert_with(|| Vec::with_capacity(2))` für Key-Versionen und atomare Kapazitätsschätzung `std::cmp::max(16, total_bytes / 64)` beim Iterieren über MemTable-Einträge.

### B. Vektor- & Text-Suchindizes (HNSW, DiskANN, BM25, SIMD)
- **Fehler / Schwachstelle**: HNSW Phantom-Node Sichtbarkeit und Race Conditions beim Einfügen (INV-HNSW-1, INV-HNSW-2).
  *Vorher*: HNSW Knoten wurden vor der vollständigen Berechnung ihrer Nachbarschafts-Verbindungen in die Graphstruktur eingefügt, was für parallele Suchanfragen als Phantom-Knoten ohne Kanten sichtbar war. Inline-Schreiblocks in `search_layer` führten zu Lock-Contention.
  *Ursache*: Verflechtung von Berechnungs- und Anwendungsphase beim HNSW-Insert.
  *Nachher (Lösung)*: Einführung des Compute-then-Commit Patterns in `hnsw.rs`:
    1. Phase 1 (`compute_insert`): Read-Only Berechnung von Quantisierung, Zielverbindungen und Rückwärtskanten.
    2. Phase 2 (`apply_insert`): Atomares Drücken des Knotens mit vollständig gefüllten Kanten.
    `search_layer()` erfordert nur noch einen einmaligen Read-Lock auf `deleted_nodes` außerhalb der BFS-Schleife ohne synchrone Schreib-Sperren.
- **Fehler / Schwachstelle**: SQ8 Skalen-Instabilität durch Dynamische Grenzen-Erweiterung.
  *Vorher*: `expand_bounds_to_fit` passte Skalierungsgrenzen der Quantisierers dynamisch an, was bestehende 8-Bit Codes entwertete und Distanzwerte verfälschte.
  *Ursache*: Verletzung der Skaleninvarianz des SQ8 Quantisierers.
  *Nachher (Lösung)*: Entfernung von `expand_bounds_to_fit`. Vektorelemente werden an den Dim-Grenzen geclampt; Drift-Zähler (`total_queries`, `out_of_range_queries`) erfassen Abweichungen atomar. Bei Überschreiten des `quantizer_drift_threshold` wird automatisch ein Index-Rebuild via `ScalarQuantizer::train` ausgelöst.
- **Fehler / Schwachstelle**: Unchecked SIMD Slice Loads Out-of-Bounds Risiko.
  *Vorher*: `unsafe fn`s in Distanzfunktionen konnten bei fehlerhaft ausgerichteten Slices OOB-Speicherzugriffe auslösen.
  *Ursache*: Fehlende compile-zeitliche/statische Kontrolle von Slice-Längenprüfungen vor SIMD-Loads.
  *Nachher (Lösung)*: Einführung des Subcommands `xtask lint-unsafe-slices`: Ein AST-Linter auf Basis von `syn`, der verifiziert, dass alle `unsafe fn`s mit Slice-Parametern explizite Längenprüfungen oder `.min()` Normalisierungen vor SIMD-Loads durchführen.
- **Fehler / Schwachstelle**: Potential UTF-8 Char Boundary Panic im Text Slicing.
  *Vorher*: Beim Zuschneiden von Strings an festen Byte-Indizes in `contextra-text` konnte ein Slicing mitten in einem Multibyte-UTF-8-Zeichen auftreten und eine Panik auslösen.
  *Ursache*: Direkter Zugriff auf `&str[..len]` ohne Boundary-Prüfung.
  *Nachher (Lösung)*: Absicherung via `floor_char_boundary` / `char_indices` Prüfungen, die sicherstellen, dass Slices strikt an UTF-8 Zeichengrenzen ausgerichtet sind.

### C. Wissensgraph & Kognitive Mechanismen
- **Fehler / Schwachstelle**: Misleitende biologische Metapher-Begriffe im Code.
  *Vorher*: Kernkomponenten der Datenbank verwendeten Begriffe wie `sleep-cycle`, `thermostat` und `reaper`.
  *Ursache*: Frühe Entwurfs-Metaphern, die zu Unklarheiten im API-Design führten.
  *Nachher (Lösung)*: Refactoring in `contextra-db` (`#2d37e40`): Ersetzung aller biologischen Metaphern durch präzise technische Bezeichnungen (`ConsolidationScheduler`, `PidController`, `ExpiryEngine`).
- **Fehler / Schwachstelle**: Staged Entities & Edges Speicherallokation im CSR-Graphen.
  *Vorher*: Transaktionales Staging im CSR-Graphen nutzte benutzerdefinierte Datenstrukturen mit hoher Reallokationsfrequenz beim Commit und Rollback.
  *Ursache*: Ungeeignete Speicherrepräsentation für temporäre Graphänderungen.
  *Nachher (Lösung)*: Migration von `staged_entities`, `staged_edges` und `staged_removals` auf flache `ahash::AHashMap` Strukturen mit Composite Keys `(TxId, EntityId)` und `.retain()` Iteration beim Commit/Rollback.
- **Fehler / Schwachstelle**: Verwaiste Kanten bei Gedächtnis-Verdrängung (A-MEM Zettelkasten).
  *Vorher*: Wenn ein veraltetes Dokument durch ein neues ersetzt wurde (`LinkRelation::Supersedes`), blieben die zugehörigen Entitäts-Kanten im CSR-Graph aktiv und verfälschten spätere GraphRAG-Abfragen.
  *Ursache*: Fehlende kaskadierende Invalidierung im Graph-Modul.
  *Nachher (Lösung)*: Kaskadierende Kanten-Invalidierung (`cascade.rs`, `INV-GRAPH-PROV-1`): Das System speichert `source_doc_id` in `Edge` und pflegt einen `doc_to_edges` Index (`DocId -> Set<(EntityId, EntityId)>`). Bei `Supersedes` wird `cascade_invalidate_edges_for_superseded_doc()` aufgerufen und invalidierte Kanten transaktional mit WAL-Bindung aus dem aktiven Traversierungsgraphen entfernt.

### D. Konkurrenz, Transaktionen & Sperrhierarchien
- **Fehler / Schwachstelle**: Transaktions-Rollback Sequenz-Verletzung bei 2PC Fehlschlägen (INV-DB-3).
  *Vorher*: Bei Ausfall der Vektorindex-Commit-Phase wurden Graph- und Textindex-Rollbacks vor dem Vektorindex-Rollback ausgeführt, was die vorgeschriebene Kompensationsreihenfolge verletzte.
  *Ursache*: Falsche Reihenfolge im `catch`-Block der Transaktions-Engine.
  *Nachher (Lösung)*: Strikt durchgesetzte Kompensationsreihenfolge nach INV-DB-3: Schlägt `collection.index.commit(tx_id)` fehl, wird zuerst `collection.index.rollback(tx_id)` aufgerufen, bevor Graph- und Textindex-Rollbacks eingeleitet werden.
- **Fehler / Schwachstelle**: Rennen um doppelte `RwLock` Indizes in Checkpoint Store.
  *Vorher*: `PersistentCheckpointStore` verwaltete zwei getrennte Locks (`checkpoints: RwLock<HashMap<u64, CheckpointMeta>>` und `name_index: RwLock<HashMap<String, u64>>`), was bei parallelem Schreiben zu Inkonsistenzen führen konnte.
  *Ursache*: Mangelnde Atomarität über zwei getrennte Mutexes.
  *Nachher (Lösung)*: Konsolidierung in eine einzige `index: RwLock<CheckpointIndex>` Struktur, die `by_seq` und `by_name` umschließt, womit alle Operationen unter einer einzigen atomaren Sperre ausgeführt werden.
- **Fehler / Schwachstelle**: Kollision zwischen MaintenanceScheduler und ConsolidationEngine (ADR-081 / H-19).
  *Vorher*: Gleichzeitiges Ausführen von Hintergrund-Wartungsaufgaben und Konsolidierungsläufen konnte zu Ressourcen-Deadlocks führen.
  *Ursache*: Fehlen von gegenseitigem Ausschluss zwischen Hintergrund-Gewerken.
  *Nachher (Lösung)*: Einführung von `consolidation_guard: Arc<tokio::sync::Mutex<()>>` in `Collection`. `execute_consolidation_pass` führt ein nicht-blockierendes `try_lock()` aus; bei Kollision wird der Durchlauf übersprungen.

### E. Sicherheit, FFI, Mandantenfähigkeit & Compliance
- **Fehler / Schwachstelle**: P1 Command Injection Schwachstelle in MCP Context Routing Dispatch.
  *Vorher*: `dispatch_to_slm` führte externe Befehle via `Command::new("sh").arg("-c")` aus, was Arbitrary Command Execution ermöglichte.
  *Ursache*: Ausführung über Shell-Interpreter ohne Argument-Sanitizing.
  *Nachher (Lösung)*: Ersetzung durch inline stdlib Helper `split_endpoint()` und direkte Binär-Aufrufe via `Command::new(&program).args(&args)` ohne Shell-Kontext.
- **Fehler / Schwachstelle**: Inkomplette DLP-Regeln und Fail-Open Risiko im Egress Gateway.
  *Vorher*: Das Egress Gateway nutzte lokales Stub-Type `DefaultEgressClassifier` mit lückenhaften Regeln.
  *Ursache*: Abweichung vom zentralen `contextra-security` Kryptokontrakt.
  *Nachher (Lösung)*: Ersetzung durch `pub type DefaultEgressClassifier = EgressVault;` direkt aus `contextra-security` mit Standard-DLP-Mustern (`sk-`, `AKIA`, `api_key`, `password`, Email-Regex) und Zero-Panic Fallbacks.
- **Fehler / Schwachstelle**: CPython Process Crash bei Rust Panics über FFI.
  *Vorher*: Ein Panic in rechenintensiven Rust-Funktionen führte zum sofortigen Absturz der gesamten Python-Anwendung.
  *Ursache*: Unverfangene Rust-Panics über PyO3 FFI-Grenzschichten.
  *Nachher (Lösung)*: Kapselung aller FFI-Aufrufe in `run_blocking_ffi` mit `py.allow_threads()` (Freigabe des Python GIL) und `std::panic::catch_unwind`, das Rust Panics abfängt und sauber in Python `PyRuntimeError` Exceptions konvertiert.

### F. Performance, Zero-Copy & Hybrid Fusion
- **Fehler / Schwachstelle**: Speicher-Copys und Reallokationen auf dem Read-Path der Storage Engine.
  *Vorher*: `StorageEngine::get` und `get_at_seq` gaben `Result<Option<Vec<u8>>>` zurück, was bei jedem Lesevorgang Puffer-Kopiervorgänge erforderte.
  *Ursache*: Verwendung von owned `Vec<u8>` in Trait-Signaturen.
  *Nachher (Lösung)*: Refactoring der `StorageEngine` Trait-Methoden auf `Result<Option<bytes::Bytes>>`. Alle Implementierungen und Aufrufer über das gesamte Workspace hinweg nutzen nun Zero-Copy `bytes::Bytes` Slices.
- **Fehler / Schwachstelle**: Heap-Allokations-Hotspot in Reciprocal Rank Fusion (`fusion.rs`).
  *Vorher*: `weighted_reciprocal_rank_fusion_with_options` allokierte für jedes Dokument im Hot-Path neue `String` Schlüssel in der Hashmap.
  *Ursache*: Verwendung von `AHashMap<String, FusedEntry>` während der Score-Akkumulation.
  *Nachher (Lösung)*: Optimierung durch `&str` ID-Internierung (`AHashMap<&str, u32>`), SoA Float Score-Akkumulation (`scores: Vec<f32>`) und späte String-Hydratisierung erst nach dem Integer-Index-Sortieren.

---

## 4. Subsystem- & Crate-Entwicklung (Layer 0 bis Layer 9)

Das Repository umfasst **19 aktiv verwaltete Workspace Crates** (18 Kern-Crates + 1 optionales Crate `contextra-embed`), aufgeteilt in **10 Architektur-Schichten** gemäß kanonischer DAG-Topologie:

```
┌─────────────────────────────────────────────────────────────────────────┐
│ Layer 9: Model Context Protocol (MCP) Boundary                          │
│  - contextra-mcp (stdio JSON-RPC 2.0 Server, DLP Egress Gateway)          │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 8: Client Interfaces & Workflow Execution                         │
│  - contextra-agent (Persistent Workflow Engine, Checkpoint/Audit Loop)   │
│  - contextra-py (PyO3 Python FFI Bindings mit GIL-Freigabe)               │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 7: Benchmarks & Routing                                           │
│  - contextra-bench (Reproduzierbare Benchmark Harness Suite)              │
│  - contextra-router (Contextual Bandit Prompt Routing Engine)             │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 6: Database Facade & Orchestration                                │
│  - contextra-db (4-Signal Fusion, 2PC Transactions, Collection API)      │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 5: In-Process Embeddings & Reranking                              │
│  - contextra-embed (In-Process ONNX Cross-Encoder Reranking, optional)    │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 4: ML Inferenz Backend                                            │
│  - contextra-candle (Native Candle GGUF Inferenz Engine)                  │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 3: Storage & Vektor-Indizes                                       │
│  - contextra-index (HNSW, DiskANN, Quantisierung, SIMD)                   │
│  - contextra-ollama (Ollama HTTP Client & Context Prefixer)               │
│  - contextra-store (LSM-Tree, WAL V3, 3-Phase Async Flush)                │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 2: Spezialisierte Subsysteme & Isolation Boundary                 │
│  - contextra-calibration (Isotonische & Platt Conformal Calibration)      │
│  - contextra-checkpoint (MVCC Snapshot-Pinning, CheckpointGuard)         │
│  - contextra-crypto / contextra-security (AES-256-GCM-SIV, DeletionProof)  │
│  - contextra-graph (CSR-Graph, PathRAG, Bi-temporale Zeitachsen)          │
│  - contextra-sandbox (WASM Execution Boundary & Fuel Isolation)           │
│  - contextra-text (BM25 Robertson-Spärck-Jones, Deutsche Morphologie)     │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 1: Core Domain Abstractions                                       │
│  - contextra-core (ContextraError, TenantId, DocId, TxBuffer, Traits)       │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 0: FlatBuffers IPC Code Generation                                │
│  - contextra-core-ipc-gen (Auto-generated FlatBuffers IPC Code)           │
└─────────────────────────────────────────────────────────────────────────┘
```

### Detaillierte Crate-Rollen (19 Crates):
1. **`contextra-core-ipc-gen`** (Layer 0): Automatisch generierter FlatBuffers IPC-Code für typsicheres Framing.
2. **`contextra-core`** (Layer 1): Kanonische Domain-Typen (`TenantId`, `DocId`, `TxId`), Unified `ContextraError`, Trait-Definitionen (`traits/`) und `TxBuffer`.
3. **`contextra-calibration`** (Layer 2): Isotonische (PAVA) und Platt Conformal Calibration für Relevanz-Scoring.
4. **`contextra-checkpoint`** (Layer 2): Persistentes Snapshot-Pinning mit atomarem `RwLock<CheckpointIndex>` und `CheckpointGuard` RAII Rollbacks.
5. **`contextra-crypto` / `contextra-security`** (Layer 2): AES-256-GCM-SIV Blockverschlüsselung, HKDF Key Derivation, WAL HMAC Chaining, `EgressVault` und `DeletionProof`.
6. **`contextra-graph`** (Layer 2): CSR-Wissensgraph, bi-temporale Zeitachsen, Session-DAG, PathRAG Engine, F-06 Perkolationsmonitor und kaskadierende Kanteninvalidierung (`cascade.rs`).
7. **`contextra-sandbox`** (Layer 2): WASM Ausführungsumgebung (Spec v5 §4.18) mit strikter Fuel- und Speicher-Isolierung.
8. **`contextra-text`** (Layer 2): Invertierter Index, BM25 Scorer (Robertson-Spärck-Jones) und deutsche Morphologie mit Char-Boundary Sicherheitsgarantien.
9. **`contextra-index`** (Layer 3): HNSW Vektorindex (Compute-then-Commit Pattern), Streaming DiskANN mit Beam Search, SQ8 Quantisierung und `std::arch` SIMD Intrinsics.
10. **`contextra-ollama`** (Layer 3): HTTP-Client für lokale Ollama LLMs/Embeddings mit Batch-Streaming und Anthropic Contextual Retrieval.
11. **`contextra-store`** (Layer 3): LSM-Tree mit MemTable-Sharding, 3-Phasen Lock-Free Async Flush, WAL V3 und SSTable Compaction.
12. **`contextra-candle`** (Layer 4): Native Candle GGUF ML-Inferenz-Backend für rahmenwerksfreie Embeddings.
13. **`contextra-embed`** (Layer 5): In-process ONNX Session Pool für Cross-Encoder Reranking mit Chaos Engineering Schutz.
14. **`contextra-db`** (Layer 6): Orchestrator Facade für 4-Signal Hybrid Retrieval (RRF), Full 2PC Transactions, PID Latency Regulation und Replicator Dynamics.
15. **`contextra-bench`** (Layer 7): Reproduzierbare Benchmark-Suite für Retrieval-Genauigkeit, Durchsatz und Latenz-Perzentile (inkl. LongMemEval & LoCoMo Datensätze).
16. **`contextra-router`** (Layer 7): Generische SLM Context Routing Engine (`RouterEngine<S: StorageEngine>`) für adaptives Prompt-Routing.
17. **`contextra-agent`** (Layer 8): Workflow-Engine mit Event-Loops, State Graph Walkers, Dead-Letter-Queues und TokenBudget RMW Schutz.
18. **`contextra-py`** (Layer 8): PyO3 Python FFI Bindings mit automatischer GIL-Freigabe und Panic Catching.
19. **`contextra-mcp`** (Layer 9): Stdio JSON-RPC 2.0 MCP-Server mit DLP Egress Gateway (`EgressVault`), Prompt Injection Quarantäne und Live-Stats-Wiring.

---

## 5. Governance & Qualitäts-Sicherung

Die Projekt-Historie zeichnet sich durch ein streng durchgesetztes Governance-System aus:

1. **Architecture Decision Records (ADRs)**: Strikte Einhaltung aller Vorgaben (z.B. ADR-010 Stdio MCP, ADR-012/043 MVCC Isolation, ADR-028 Error DTOs, ADR-044 Write-Temp-Then-Rename, ADR-047 SIMD Intrinsics, ADR-059 Non-Blocking Async LSM Flush, ADR-060 Instance-Scoped Flush Counter, ADR-068 KV-Encryption, ADR-073 Contradiction Prevention, ADR-081 Mutual Exclusion Governance, ADR-082 DocId Width).
2. **Inline Code Tags & Review Passes**: Verwendung von `ANCHOR[...]`, `AI-TAG[...]` und `REVIEW-PASS[...]` Annotationen mit ISO-8601 Zeitstempeln (`TS:2026-09-18T...`) und Session-Hashes.
3. **Automatisierte CI Enforcement Gates**:
   - `cargo xtask check-consistency`: Überprüft exakt 19 Workspace-Crates, `AGENTS.md` Abdeckung, README-Auszüge und ADR-Eindeutigkeit.
   - `cargo xtask sync-docs`: Verhindert Drift zwischen Quellcode-Annotationen und Dokumentationsdateien (`WORKING_STATE.md`, `ARCHITECTURE.md`, `CHANGELOG.md`, `SOURCE_OF_TRUTH.md`).
   - `cargo xtask lint-unsafe-slices`: Verifiziert via AST-Analyse, dass alle `unsafe` SIMD Load Slices vor dem Zugriff Längenprüfungen oder `.min()` Normalisierungen durchführen.
   - `cargo xtask post-merge-report`: Generiert automatisierte Post-Merge Berichte nach Pull Request Integrationen (#3109).
   - `context-gates.yml`: Verhindert ungelöste `CRITICAL` Code Smells, phantom temporäre Dateien (Gate 12b) und prüft die Gültigkeit von Anchor-Tags.

---

## 6. Statistische Kennzahlen

- **Aktive Workspace Crates**: 19 Crates (Layer 0 bis Layer 9)
- **DAG Topologie**: 10 Schichten (Layer 0 bis Layer 9)
- **Commits insgesamt**: >350 Merges und Direkt-Commits
- **Verteilte Autoren**: `google-labs-jules[bot]`, `tfufuz1`, `tfufuu`
- **Programmiersprache**: 100% Rust (mit Tauri UI HTML/JS Frontend & PyO3 Python-Interface)
- **Sicherheit & Zero-Panic Policy**: Volle Beseitigung aller unkontrollierten `.unwrap()` Aufrufe in Produktivpfaden (abgesichert via `// unwrap allowed` mit nachgewiesenen Invarianten und `.unwrap-baseline.json`).

---

*Ende der Projekthistorie.*
