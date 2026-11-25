# Contextra — Central Type Registry (`TYPE_REGISTRY.md`)

> **Regel (AGENTS.md — Non-Obvious Decisions)**: Vor dem Anlegen eines neuen Typs oder Traits MUSS diese Tabelle konsultiert werden. Bei semantischen Überschneidungen ist der bestehende Typ zu erweitern oder die Kollision explizit per ADR zu begründen.

---

## 🏛️ Domain Types & Structs

| Typ / Struct / Enum | Crate | Datei : Zeile | Zweck / Domäne |
|---|---|---|---|
| `DocId` | `contextra-core` | `crates/contextra-core/src/types.rs:18` | 64-Bit BLAKE3-getrunkierte Dokumenten-ID |
| `TxId` | `contextra-core` | `crates/contextra-core/src/types.rs:42` | Transaktions-ID (`[1, 10^12]` vs. `INTERNAL_BASE` System-Range) |
| `EntityId` | `contextra-core` | `crates/contextra-core/src/types.rs:65` | Wissensgraph Knoten-Entitäts-ID |
| `ContextraError` | `contextra-core` | `crates/contextra-core/src/error.rs:14` | Zentraler, abweisungsfreier Fehler-Enum |
| `ContextraErrorDto` | `contextra-core` | `crates/contextra-core/src/error_dto.rs:10` | Serialisierbare FFI/IPC DTO-Fehlerdarstellung |
| `ContextChunk` | `contextra-core` | `crates/contextra-core/src/types/saos.rs:15` | Dokument-Chunk mit optionalem `contextual_prefix` (ADR-019) |
| `HybridQuery` | `contextra-core` | `crates/contextra-core/src/types/saos.rs:75` | Query-Spezifikation für 4-Signal-Suche |
| `MetadataFilter` | `contextra-core` | `crates/contextra-core/src/types/saos.rs:120` | Metadaten-Filter-Prädikate (Eq, Ne, In, Range, Contains, And, Or) |
| `CheckpointGuard` | `contextra-checkpoint` | `crates/contextra-checkpoint/src/lib.rs:24` | RAII-Guard für automatischen WAL-Rollback bei Drop |
| `TenantId` | `contextra-core` | `crates/contextra-core/src/types/domain.rs:59` | Mandanten-Identifikator (INV-TENANT-1: 0 ist SYSTEM) |
| `CompactionStrategy` | `contextra-db` | `crates/contextra-db/src/compaction.rs:18` | Kontext-Kompaktierungs-Strategien (DropOld, Summarize, LlmSummarize) |
| `StoredDocument` | `contextra-db` | `crates/contextra-db/src/collection/mod.rs:34` | In-Storage Repräsentation eines Dokuments inklusive Embeddings |
| `StoredDocumentMeta` | `contextra-db` | `crates/contextra-db/src/collection/mod.rs:43` | In-Storage Repräsentation für schnelle Result-Hydration (ohne Vektoren) |
| `MemoryType` | `contextra-core` | `crates/contextra-core/src/types/domain.rs:535` | Klassifikation kognitiver Gedächtnistypen (Episodic, Semantic, Procedural, Working) (ADR-041) |
| `ModelFingerprint` | `contextra-core` | `crates/contextra-core/src/model_fingerprint.rs:12` | SHA-256-basierter Fingerabdruck über Modellgewichte und Quantisierungsstufe; Grundlage der KV-Cache-Schlüsselableitung |
| `HyperEdgeId` | `contextra-graph` | `crates/contextra-graph/src/hyperedge.rs:1` | Eindeutige ID einer Hyperkante |
| `RoleId` | `contextra-graph` | `crates/contextra-graph/src/hyperedge.rs:2` | Rollenbezeichner innerhalb einer Hyperkante |
| `RoleBinding` | `contextra-graph` | `crates/contextra-graph/src/hyperedge.rs:3` | Zuweisung einer Entität zu einer Rolle in einer Hyperkante |
| `HyperEdge` | `contextra-graph` | `crates/contextra-graph/src/hyperedge.rs:4` | N-äre Kantenstruktur für Fakten mit >2 Beteiligten |
| `GraphMutationError` | `contextra-graph` | `crates/contextra-graph/src/error.rs:10` | Fehler bei Graph-Mutationen (z.B. Fan-out überschritten) |
| `SieveNode` | `contextra-store` | `crates/contextra-store/src/block_cache/sieve.rs:1` | Knoten im lock-freien SIEVE-Cache |
| `SieveCacheBackend` | `contextra-store` | `crates/contextra-store/src/block_cache/sieve.rs:2` | Lock-freies Cache-Backend (Block-Cache v2) |
| `ShermanMorrisonBandit` | `contextra-router` | `crates/contextra-router/src/bandit.rs:1` | Contextual Bandit mit exakter inkrementeller Ridge-Regression |
| `AlignedVector` | `contextra-router` | `crates/contextra-router/src/bandit.rs:2` | Cache-Line-aligned SIMD-Vektor für Bandit-Matrix |
| `WalRingBuffer` | `contextra-store` | `crates/contextra-store/src/wal_ring_buffer.rs:1` | Lock-freier SPSC-Ring-Puffer für WAL-Schreiboperationen |
| `ProvenanceBuilder` | `contextra-db` | `crates/contextra-db/src/provenance.rs:1` | Konstruktion des RRF-Herkunftsnachweises |
| `PprParams` | `contextra-graph` | `crates/contextra-graph/src/ppr.rs:1` | Konfiguration für Forward-Push PPR |
| `StarExpansionIterator` | `contextra-graph` | `crates/contextra-graph/src/community.rs:1` | Projektion von Hyperkanten für Leiden |
| `CommunityDetectionConfig` | `contextra-graph` | `crates/contextra-graph/src/community.rs:2` | Konfiguration für Leiden inkl. `hyperedges_included` |
| `CommunityAssignment` | `contextra-graph` | `crates/contextra-graph/src/community.rs:3` | Ergebnis der Community Detection |
| `WasmCapabilities` | `contextra-mcp` | `crates/contextra-mcp/src/sandbox.rs:1` | Erlaubte Ausführungsrechte in der WASM-Sandbox |
| `EgressVault` | `contextra-crypto` | `crates/contextra-crypto/src/egress.rs:1` | Sicherheitskomponente für Cloud-Kommunikation |
| `BulkExfiltrationDetector` | `contextra-crypto` | `crates/contextra-crypto/src/egress.rs:2` | Detektor für Massen-Datenabfluss |
| `LyapunovDriftWatcher` | `contextra-calibration` | `crates/contextra-calibration/src/drift.rs:1` | Überwachung von Drift-Eskalation |

---

## 🔌 Central Traits

| Trait | Crate | Datei : Zeile | Zweck / Contract |
|---|---|---|---|
| `StorageEngine` | `contextra-core` | `crates/contextra-core/src/traits.rs:75` | LSM-Tree Speicherengine (MVCC, Transaktionen, Scans) |
| `VectorIndex` | `contextra-core` | `crates/contextra-core/src/traits.rs:175` | Vektor-Suchindex (HNSW/DiskANN) |
| `TextIndex` | `contextra-core` | `crates/contextra-core/src/traits.rs:305` | Lexikalischer Volltextindex (BM25) |
| `GraphIndex` | `contextra-core` | `crates/contextra-core/src/traits.rs:395` | CSR Entity-Relation-Wissensgraph |
| `TextEmbeddingEngine` | `contextra-core` | `crates/contextra-core/src/traits.rs:275` | Embedding-Provider Interface |
| `CheckpointCoordinator` | `contextra-core` | `crates/contextra-core/src/traits.rs:35` | Konsolidiertes Checkpoint-Management (ADR-011) |
| `BanditPolicy` | `contextra-router` | `crates/contextra-router/src/routing_strategy.rs:1` | Trait für Routing-Entscheidungen |
| `BlockCacheBackend` | `contextra-store` | `crates/contextra-store/src/block_cache/mod.rs:1` | Austauschbares Backend für den Block-Cache (LRU/SIEVE) |
| `Checkpointable` | `contextra-checkpoint` | `crates/contextra-checkpoint/src/lib.rs:1` | Eigenschaft von Snapshots/Strukturen |
| `BenchmarkHarness` | `contextra-bench` | `crates/contextra-bench/src/lib.rs:1` | Interface für reproduzierbare Benchmarks |
