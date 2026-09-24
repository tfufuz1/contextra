# Audit: `contextra-mcp` Aufwärtskanten-Analyse für Welle 3

**Date:** 2026-09-11
**Author:** Google-Jules (Principal Rust Systems Engineer)
**Status:** APPROVED (Audit Deliverable)
**Target Crate:** `crates/contextra-mcp` (Ring 4 Application Services)
**Scope:** Direct dependencies on Ring 1–3 internal crates (`contextra-db`, `contextra-router`, `contextra-ollama`/`contextra-infer-ollama`, `contextra-calibration`)

---

## 1. Executive Summary & Context

Under the Contextra Ring Architecture (Rings 0–4):
* **Ring 0 (Foundation):** `contextra-core`, `contextra-types`, `contextra-wire`
* **Ring 1 (Storage & Base Engines):** `contextra-store`, `contextra-index`, `contextra-text`, `contextra-graph`, `contextra-crypto`, `contextra-mvcc`
* **Ring 2 (Core Business & Integration):** `contextra-db`, `contextra-router`, `contextra-calibration`, `contextra-ollama`, `contextra-embed`, `contextra-candle`
* **Ring 3 (Abstraction Ports & Workflows):** `contextra-ports`, `contextra-agent`, `contextra-checkpoint`, `contextra-adapt`
* **Ring 4 (Applications & Facades):** `contextra` (Primary Facade / Composition Root), `contextra-mcp` (MCP Stdio Server), `contextra-py`

### Rationale
Currently, `contextra-mcp` directly imports types and functions from internal Ring 2 crates (`contextra-db`, `contextra-router`, `contextra-calibration`, `contextra-ollama`). While Ring 4 applications are permitted to depend on lower rings, good architectural isolation dictates that applications like `contextra-mcp` should consume core database and inference services through the unified `contextra` facade (or dedicated abstraction ports in `contextra-ports`), rather than linking directly to individual internal engine implementations.

This audit report identifies every upward/direct dependency edge from `contextra-mcp` to internal Ring 2 crates, categorizes their functional roles, evaluates whether each should transition to the `contextra` facade or a port in `contextra-ports`, and provides a detailed Wave 3 execution plan with sequencing and safety verification criteria.

---

## 2. Crate Renaming Status Note

During Wave 2/Wave 3 refactoring, inference crates undergo structural standardization:
* **Current Repo State:** The crate directory is `crates/contextra-ollama` and package name in `Cargo.toml` is `contextra-ollama`.
* **Planned Standardized State:** Renaming to `contextra-infer-ollama`.
* **Audit Compatibility:** This audit accounts for both `contextra-ollama` (active in current branch) and `contextra-infer-ollama` (future state). All recommendations apply identically regardless of the prefix.

---

## 3. Comprehensive Inventory of Internal Ring 2 Usages in `contextra-mcp`

Below is the file-by-file and line-by-line breakdown of every usage of `contextra_db::`, `contextra_router::`, `contextra_ollama::` / `contextra_infer_ollama::`, and `contextra_calibration::` across `crates/contextra-mcp`.

### 3.1 `crates/contextra-mcp/src/config.rs`

| Line | Code Symbol / Expression | Crate | Purpose |
|------|--------------------------|-------|---------|
| 31 | `contextra_ollama::DEFAULT_BASE_URL.to_string()` | `contextra-ollama` | Default base URL for embedding provider config |
| 32 | `contextra_ollama::DEFAULT_EMBED_MODEL.to_string()` | `contextra-ollama` | Default embedding model name |
| 47 | `contextra_ollama::DEFAULT_BASE_URL.to_string()` | `contextra-ollama` | Fallback env value for Ollama URL |
| 50 | `contextra_ollama::DEFAULT_EMBED_MODEL.to_string()` | `contextra-ollama` | Fallback env value for embed model |
| 91 | `contextra_ollama::OllamaEmbedder::new(...)` | `contextra-ollama` | Instantiation of Ollama embedding engine |
| 164 | `contextra_ollama::DEFAULT_BASE_URL.to_string()` | `contextra-ollama` | Default base URL for LLM config |
| 179 | `contextra_ollama::DEFAULT_BASE_URL.to_string()` | `contextra-ollama` | Fallback env value for LLM URL |
| 218 | `contextra_ollama::OllamaConfig { ... }` | `contextra-ollama` | Ollama client configuration struct |
| 223 | `contextra_ollama::OllamaClient::with_config(...)` | `contextra-ollama` | Instantiation of Ollama LLM text generator |
| 263 | `pub profiles: Vec<contextra_router::SlmProfile>` | `contextra-router` | Router configuration profile list field |
| 285 | `serde_json::from_slice::<Vec<contextra_router::SlmProfile>>(&bytes)` | `contextra-router` | Deserialization of router SLM profiles from file |
| 291 | `serde_json::from_str::<Vec<contextra_router::SlmProfile>>(&json_str)` | `contextra-router` | Deserialization of router SLM profiles from JSON env |
| 326 | `assert_eq!(config.ollama_url, contextra_ollama::DEFAULT_BASE_URL)` | `contextra-ollama` | Test assertion for Ollama URL |
| 327 | `assert_eq!(config.embed_model, contextra_ollama::DEFAULT_EMBED_MODEL)` | `contextra-ollama` | Test assertion for embed model |
| 336 | `assert_eq!(config.ollama_url, contextra_ollama::DEFAULT_BASE_URL)` | `contextra-ollama` | Test assertion for LLM Ollama URL |

### 3.2 `crates/contextra-mcp/src/routing.rs`

| Line | Code Symbol / Expression | Crate | Purpose |
|------|--------------------------|-------|---------|
| 3 | `use contextra_db::Contextra;` | `contextra-db` | Database handle reference for attaching router |
| 8 | `pub router: Arc<contextra_router::DefaultRouterEngine>` | `contextra-router` | RoutingHandle field for RouterEngine instance |
| 9 | `pub calibrator: Arc<parking_lot::Mutex<contextra_calibration::IsotonicCalibrator>>` | `contextra-calibration` | RoutingHandle field for IsotonicCalibrator |
| 10 | `pub pid_controller: Arc<parking_lot::Mutex<contextra_calibration::PidController>>` | `contextra-calibration` | RoutingHandle field for PidController |
| 25 | `contextra_router::RouterEngine::new(...)` | `contextra-router` | RouterEngine instantiation |
| 32 | `contextra_calibration::IsotonicCalibrator::with_defaults()` | `contextra-calibration` | IsotonicCalibrator instantiation |
| 36 | `contextra_calibration::PidController::default()` | `contextra-calibration` | PidController instantiation |
| 40 | `Arc::downgrade(&router) as std::sync::Weak<dyn contextra_db::DriftStatusProvider>` | `contextra-db` | Casting router to `DriftStatusProvider` trait for DB |

### 3.3 `crates/contextra-mcp/src/egress_guard.rs`

| Line | Code Symbol / Expression | Crate | Purpose |
|------|--------------------------|-------|---------|
| 11 | `use contextra_db::Collection;` | `contextra-db` | EgressGuard struct holds `Arc<Collection>` |
| 24 | `/// Baut auf dem lokalen HNSW-Vektorindex einer contextra_db::Collection auf.` | `contextra-db` | Doc comment reference |
| 140 | `use contextra_db::Contextra;` | `contextra-db` | Test helper fixture creating Contextra DB |

### 3.4 `crates/contextra-mcp/src/server_tools.rs`

| Line | Code Symbol / Expression | Crate | Purpose |
|------|--------------------------|-------|---------|
| 7 | `use contextra_db::chunker::{ChunkerConfig, MarkdownChunker};` | `contextra-db` | Markdown document auto-chunker in `contextra_insert` |
| 430 | `contextra_db::execute_background_consolidation(...)` | `contextra-db` | Background memory consolidation execution in `contextra_consolidate` |
| 433 | `&contextra_db::memory_consolidation::ConsolidationConfig::default()` | `contextra-db` | Consolidation config default struct |

### 3.5 `crates/contextra-mcp/src/server.rs`

| Line | Code Symbol / Expression | Crate | Purpose |
|------|--------------------------|-------|---------|
| 11 | `use contextra_db::Contextra;` | `contextra-db` | `McpServer` struct field `pub db: Arc<Contextra>` |

### 3.6 `crates/contextra-mcp/src/bin/contextra-mcp-server.rs`

| Line | Code Symbol / Expression | Crate | Purpose |
|------|--------------------------|-------|---------|
| 8 | `use contextra_db::Contextra;` | `contextra-db` | Entrypoint database instantiation via `Contextra::open(...)` |
| 64 | `serde_json::from_slice::<Vec<contextra_router::SlmProfile>>(&bytes)` | `contextra-router` | Deserialization of router profiles in main binary |

### 3.7 `crates/contextra-mcp/src/tests.rs`

| Line | Code Symbol / Expression | Crate | Purpose |
|------|--------------------------|-------|---------|
| 5 | `use contextra_db::Contextra;` | `contextra-db` | Unit test fixture creating `Contextra::open(...)` |

### 3.8 `crates/contextra-mcp/tests/mcp_test.rs`

| Line | Code Symbol / Expression | Crate | Purpose |
|------|--------------------------|-------|---------|
| 2 | `use contextra_db::Contextra;` | `contextra-db` | Integration test fixture creating `Contextra::open(...)` |
| 973 | `let profile = contextra_router::SlmProfile::new(...)` | `contextra-router` | Integration test constructing `SlmProfile` |

---

## 4. Functional Classification & Architectural Recommendations

We classify the internal usages into four distinct functional clusters and determine the target architecture for each.

```
+-------------------------------------------------------------------------------+
|                             contextra-mcp (Ring 4)                            |
+-------------------------------------------------------------------------------+
       |                                       |                         |
       | (Primary DB Operations)               | (Inference & Routing)   | (Chunker & Consolidation)
       v                                       v                         v
+-----------------------+              +-----------------------+ +-----------------------+
|  contextra Facade Crate |              |   contextra-ports Crate | |  contextra Facade /     |
|   (Contextra, Builder)  |              |  (Inference/Router    | |  contextra-ports        |
|  [Re-exports Contextra] |              |      Traits & Ports)  | |  [Re-exports Chunk/   |
+-----------------------+              +-----------------------+ |   Consolidation]      |
                                                                 +-----------------------+
```

### Cluster 1: Core Database Handle & Collections (`Contextra`, `Collection`)
* **Current Usage:** `src/server.rs`, `src/routing.rs`, `src/egress_guard.rs`, `src/bin/contextra-mcp-server.rs`, `src/tests.rs`, `tests/mcp_test.rs` directly import `contextra_db::{Contextra, Collection}`.
* **Architectural Target:** **`contextra` Facade Crate**.
* **Rationale:** The `contextra` crate (Ring 4) is designed as the canonical facade and re-exports `Contextra`, `Collection`, `ContextraConfig`, and domain types (`DocId`, `ScoredDocument`). Re-exporting `Collection` from `contextra` allows `contextra-mcp` to import `contextra::{Contextra, Collection}` instead of `contextra_db`.

### Cluster 2: SLM Router & Calibration (`RouterEngine`, `SlmProfile`, `IsotonicCalibrator`, `PidController`)
* **Current Usage:** `src/routing.rs`, `src/config.rs`, `src/bin/contextra-mcp-server.rs`, `tests/mcp_test.rs` directly import `contextra_router::*` and `contextra_calibration::*`.
* **Architectural Target:** **`contextra` Facade / `contextra-ports` Port Abstraction**.
* **Rationale:** Router configuration and calibration parameters are application-level extensions. `contextra` facade should re-export `SlmProfile` and routing initialization helpers, or expose a builder method `ContextraBuilder::with_router_config(...)`. `contextra-ports` provides the `RouterPort` trait.

### Cluster 3: Ollama / Inference Engine Instantiation (`OllamaEmbedder`, `OllamaClient`, `OllamaConfig`)
* **Current Usage:** `src/config.rs` directly instantiates `contextra_ollama::OllamaEmbedder` and `contextra_ollama::OllamaClient`.
* **Architectural Target:** **`contextra-ports` / `contextra` Facade Feature Gate**.
* **Rationale:** Embedding providers implement `contextra_core::EmbeddingProvider` and `contextra_core::LlmTextGenerator`. Dynamic provider creation in `config.rs` should either be moved to factory methods re-exported by `contextra` facade under feature flags (`ollama`, `candle`, `onnx`), or dispatched via port adapters in `contextra-ports`.

### Cluster 4: Text Chunking & Memory Consolidation (`MarkdownChunker`, `execute_background_consolidation`)
* **Current Usage:** `src/server_tools.rs` directly calls `contextra_db::chunker::MarkdownChunker` and `contextra_db::execute_background_consolidation`.
* **Architectural Target:** **`contextra` Facade Re-export / `Contextra` High-Level Methods**.
* **Rationale:** Text chunking and memory consolidation are core features of Contextra. High-level methods should be exposed directly on `Contextra` or `Collection` (e.g. `col.consolidate(...)`), or `MarkdownChunker` and `execute_background_consolidation` should be re-exported through `contextra::chunker` and `contextra::consolidation`.

---

## 5. Wave 3 Implementation Plan & Affected File Matrix

To remove direct dependencies on internal crates from `contextra-mcp` without breaking functionality or introducing cyclic dependencies, Wave 3 should execute in the following three phased steps:

### Phase 3a: Extend `contextra` Facade & `contextra-ports`
Before modifying `contextra-mcp`, update the Ring 4 facade crate (`crates/contextra`):
1. **`crates/contextra/Cargo.toml`:**
   - Add feature gates for `ollama`, `router`, `calibration` if needed.
2. **`crates/contextra/src/lib.rs`:**
   - Re-export `Collection`, `CollectionConfig`, `chunker`, `memory_consolidation` from `contextra_db`.
   - Re-export `SlmProfile`, `RouterConfig` from `contextra_router` (when `router` feature enabled).
   - Re-export `OllamaEmbedder`, `OllamaClient`, `OllamaConfig` from `contextra_ollama` (when `ollama` feature enabled).

### Phase 3b: Refactor `crates/contextra-mcp` Source Files

| Affected File | Target Changes | Replacement Imports |
|---------------|----------------|---------------------|
| `crates/contextra-mcp/Cargo.toml` | Replace `contextra-db`, `contextra-router`, `contextra-ollama`, `contextra-calibration` dependencies with `contextra = { workspace = true, features = ["ollama", "router"] }` and `contextra-ports`. | `contextra = { workspace = true }` |
| `crates/contextra-mcp/src/server.rs` | Update `db` field type and imports. | `use contextra::{Contextra, Collection};` |
| `crates/contextra-mcp/src/egress_guard.rs` | Update `Collection` and test `Contextra` imports. | `use contextra::{Collection, Contextra};` |
| `crates/contextra-mcp/src/routing.rs` | Use facade/port re-exports for `RouterEngine`, `IsotonicCalibrator`, `PidController`. | `use contextra::routing::*;` |
| `crates/contextra-mcp/src/config.rs` | Use facade re-exports for `OllamaEmbedder`, `OllamaClient`, `SlmProfile`. | `use contextra::inference::ollama::*;` |
| `crates/contextra-mcp/src/server_tools.rs` | Use facade re-exports for `MarkdownChunker` and `execute_background_consolidation`. | `use contextra::chunker::*;` |
| `crates/contextra-mcp/src/bin/contextra-mcp-server.rs` | Update `Contextra` and `SlmProfile` imports. | `use contextra::{Contextra, SlmProfile};` |
| `crates/contextra-mcp/src/tests.rs` | Update unit test imports. | `use contextra::Contextra;` |
| `crates/contextra-mcp/tests/mcp_test.rs` | Update integration test imports. | `use contextra::{Contextra, SlmProfile};` |

### Phase 3c: Verification & Gate Checks
Run standard preflight verification commands:
```bash
just check
just dag-check
cargo xtask jules-preflight
```

---

## 6. Sequence & Dependency Matrix for Wave 3 Parallelization

To prevent build breaks and git merge conflicts when multiple VMs execute Wave 3 tasks in parallel, the following sequence rules MUST be respected:

```
[Step 1: Facade/Ports Preparation]
  │
  ├── Update `crates/contextra/src/lib.rs` (Facade re-exports)
  └── Update `crates/contextra-ports/src/lib.rs` (Port traits if required)
  │
  ▼
[Step 2: MCP Refactoring]
  │
  ├── Refactor `crates/contextra-mcp/Cargo.toml`
  ├── Refactor `src/{config, routing, server, server_tools, egress_guard}.rs`
  └── Refactor `src/bin/contextra-mcp-server.rs` & tests
  │
  ▼
[Step 3: Verification Gates]
  │
  ├── `just check`
  ├── `just dag-check`
  └── `cargo xtask jules-preflight`
```

### Conflict Avoidance Guidelines
1. **Facade First:** Phase 3a (`contextra` facade re-exports) MUST be completed and merged before Phase 3b (`contextra-mcp` import changes) is executed.
2. **Crate Lock Claim:** Any agent executing Wave 3 changes on `contextra-mcp` MUST register a claim via `cargo xtask claim --crate contextra-mcp`.
3. **Workspace Root Cargo.toml:** Do NOT touch workspace root `Cargo.toml` unless workspace dependency aliases change.

---

## 7. Safety Invariants & Preflight Verification Criteria

When executing the refactoring in Wave 3, the following strict repository invariants must be preserved:

1. **Zero-Panic Doctrine:** No `.unwrap()` or `.expect()` calls in production code paths. Propagate errors via `?` and `ContextraError` or `McpError`.
2. **Zero-Copy & Alignment:** Maintain existing zero-copy `Bytes` buffers and 64-byte aligned SIMD vector structures in search pipelines.
3. **DAG Layering Integrity:** `just dag-check` must pass without warnings or errors.
4. **Unsafe Isolation:** `#![forbid(unsafe_code)]` must remain strictly enforced at crate roots of `contextra-mcp` and `contextra`.
5. **Preflight Gate:** `cargo xtask jules-preflight` must pass with zero errors.

---

*End of Audit Report.*
