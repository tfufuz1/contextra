# AGENTS.md — contextra — Top-Level Client-Façade

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §1 (Produktdefinition), §2 (Architektur), §14 (Konfiguration)  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E2 (Top-Level Client-Façade)  
**Kern-Closure:** contextra-db, contextra-types, contextra-erasure  
**Unsafe-Status:** Verboten (`#![forbid(unsafe_code)]`)  

---

## 1. Zweck & Systemrolle

Offizielle primäre Rust-API für Anwendungsentwickler (`Contextra::open`, `Collection`, Compliance-Profile).

---

## 2. Kern-Richtlinien für Agenten

- **Ergonomische High-Level API:** `Contextra::open`, `collection()`, `insert_k()`, `insert_p()`, `hybrid_search()`, `erase_subject()`, `drop_collection()`.
- **Compliance-Profile:** Vordefinierte Profile wie `ContextraConfig::enterprise_regulated(data_dir, passphrase)`.
- **Dünne Schicht:** Keine eigene Speicher- oder Indexierungslogik; reine Delegation an `contextra-db` und `contextra-erasure`.

---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra` · **Ring:** 4 · **Version:** 0.1.0
- **Beschreibung:** Contextra — Embedded hybrid-search for AI agents (Facade)
- **src:** 7 Dateien, 1195 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 3):** `contextra-db`, `contextra-ports`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 7):** `serde`, `serde_json`, `thiserror`, `tokio`, `tracing`, `uuid`, `zeroize`
- **Reverse-Abhängigkeiten (1):** `contextra-mcp`
- **Kopplung an Hub-Crates:** ausgehend `contextra-db`, `contextra-ports`, `contextra-types`; eingehend –
- **Features (2):** `default`, `embedded`

### Öffentliche API (grob, aus src/)
- struct (9): `AgentMemory`, `CollectionProfile`, `ContextraBuilder`, `LlmQueryRewriter`, `LsmTuning`, `Memory`, `MemoryId`, `RelationId`, `ResolvedProfileConfig`
- enum (9): `BootPhase`, `CollectionProfileError`, `DeploymentTier`, `DurabilityConfigError`, `DurabilityMode`, `EncryptionMode`, `KvDeleteMode`, `PerformanceProfile`, `PerformanceProfileError`
- trait (1): `ContextraConfigExt`
- fn (37): `as_str`, `build`, `builder`, `enforce_license`, `engine`, `execute_boot_sequence`, `forget`, `is_encrypted`, `new`, `new_arc`, `new_with_id_gen`, `open`, `open_with_config`, `recall`, `relate` … (+22)
- const (2): `DEFAULT_AUTO_EXTRACTION_MODE`, `ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT`
- mod (6): `agent_memory`, `boot`, `builder`, `collection_profile`, `performance_profile`, `query_rewriter`
- pub use: `agent_memory::{AgentMemory, Memory, MemoryId}`, `builder::{ContextraBuilder, ContextraConfigExt, EncryptionMode}`, `collection_profile::{CollectionProfile, DeploymentTier, KvDeleteMode, LsmTuning}`, `contextra_db::{ chunker, Collection, CollectionConfig, Contextra, ContextraConfig, ContextraStats, Document, DriftStatusProvider, EmbeddingBackend, SearchResult, TextEmbeddingEngine, }`, `contextra_ports::license::VectorDeleteMode`, `contextra_types::DistanceMetric`, `contextra_types::error::ContextraError`, `contextra_types::types::domain::AutoExtractionMode`, `contextra_types::types::domain::{DocId, ScoredDocument}`, `performance_profile::{DurabilityConfigError, DurabilityMode, PerformanceProfile}` … (+1)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
