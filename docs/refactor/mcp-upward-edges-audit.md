# Audit: `memfuse-mcp` Aufwärtskanten-Analyse für Welle 3

**Date:** 2026-09-11
**Author:** Google-Jules (Principal Rust Systems Engineer)
**Status:** APPROVED (Audit Deliverable)
**Target Crate:** `crates/memfuse-mcp` (Ring 4 Application Services)
**Scope:** Direct dependencies on Ring 1–3 internal crates (`memfuse-db`, `memfuse-router`, `memfuse-ollama`/`memfuse-infer-ollama`, `memfuse-calibration`)

---

## 1. Executive Summary & Context

Under the MemFuse Ring Architecture (Rings 0–4):
* **Ring 0 (Foundation):** `memfuse-core`, `memfuse-types`, `memfuse-wire`
* **Ring 1 (Storage & Base Engines):** `memfuse-store`, `memfuse-index`, `memfuse-text`, `memfuse-graph`, `memfuse-crypto`, `memfuse-mvcc`
* **Ring 2 (Core Business & Integration):** `memfuse-db`, `memfuse-router`, `memfuse-calibration`, `memfuse-ollama`, `memfuse-embed`, `memfuse-candle`
* **Ring 3 (Abstraction Ports & Workflows):** `memfuse-ports`, `memfuse-agent`, `memfuse-checkpoint`, `memfuse-adapt`
* **Ring 4 (Applications & Facades):** `memfuse` (Primary Facade / Composition Root), `memfuse-mcp` (MCP Stdio Server), `memfuse-py`

### Rationale
Currently, `memfuse-mcp` directly imports types and functions from internal Ring 2 crates (`memfuse-db`, `memfuse-router`, `memfuse-calibration`, `memfuse-ollama`). While Ring 4 applications are permitted to depend on lower rings, good architectural isolation dictates that applications like `memfuse-mcp` should consume core database and inference services through the unified `memfuse` facade (or dedicated abstraction ports in `memfuse-ports`), rather than linking directly to individual internal engine implementations.

This audit report identifies every upward/direct dependency edge from `memfuse-mcp` to internal Ring 2 crates, categorizes their functional roles, evaluates whether each should transition to the `memfuse` facade or a port in `memfuse-ports`, and provides a detailed Wave 3 execution plan with sequencing and safety verification criteria.

---

## 2. Crate Renaming Status Note

During Wave 2/Wave 3 refactoring, inference crates undergo structural standardization:
* **Current Repo State:** The crate directory is `crates/memfuse-ollama` and package name in `Cargo.toml` is `memfuse-ollama`.
* **Planned Standardized State:** Renaming to `memfuse-infer-ollama`.
* **Audit Compatibility:** This audit accounts for both `memfuse-ollama` (active in current branch) and `memfuse-infer-ollama` (future state). All recommendations apply identically regardless of the prefix.

---

## 3. Comprehensive Inventory of Internal Ring 2 Usages in `memfuse-mcp`

Below is the file-by-file and line-by-line breakdown of every usage of `memfuse_db::`, `memfuse_router::`, `memfuse_ollama::` / `memfuse_infer_ollama::`, and `memfuse_calibration::` across `crates/memfuse-mcp`.

### 3.1 `crates/memfuse-mcp/src/config.rs`

| Line | Code Symbol / Expression | Crate | Purpose |
|------|--------------------------|-------|---------|
| 31 | `memfuse_ollama::DEFAULT_BASE_URL.to_string()` | `memfuse-ollama` | Default base URL for embedding provider config |
| 32 | `memfuse_ollama::DEFAULT_EMBED_MODEL.to_string()` | `memfuse-ollama` | Default embedding model name |
| 47 | `memfuse_ollama::DEFAULT_BASE_URL.to_string()` | `memfuse-ollama` | Fallback env value for Ollama URL |
| 50 | `memfuse_ollama::DEFAULT_EMBED_MODEL.to_string()` | `memfuse-ollama` | Fallback env value for embed model |
| 91 | `memfuse_ollama::OllamaEmbedder::new(...)` | `memfuse-ollama` | Instantiation of Ollama embedding engine |
| 164 | `memfuse_ollama::DEFAULT_BASE_URL.to_string()` | `memfuse-ollama` | Default base URL for LLM config |
| 179 | `memfuse_ollama::DEFAULT_BASE_URL.to_string()` | `memfuse-ollama` | Fallback env value for LLM URL |
| 218 | `memfuse_ollama::OllamaConfig { ... }` | `memfuse-ollama` | Ollama client configuration struct |
| 223 | `memfuse_ollama::OllamaClient::with_config(...)` | `memfuse-ollama` | Instantiation of Ollama LLM text generator |
| 263 | `pub profiles: Vec<memfuse_router::SlmProfile>` | `memfuse-router` | Router configuration profile list field |
| 285 | `serde_json::from_slice::<Vec<memfuse_router::SlmProfile>>(&bytes)` | `memfuse-router` | Deserialization of router SLM profiles from file |
| 291 | `serde_json::from_str::<Vec<memfuse_router::SlmProfile>>(&json_str)` | `memfuse-router` | Deserialization of router SLM profiles from JSON env |
| 326 | `assert_eq!(config.ollama_url, memfuse_ollama::DEFAULT_BASE_URL)` | `memfuse-ollama` | Test assertion for Ollama URL |
| 327 | `assert_eq!(config.embed_model, memfuse_ollama::DEFAULT_EMBED_MODEL)` | `memfuse-ollama` | Test assertion for embed model |
| 336 | `assert_eq!(config.ollama_url, memfuse_ollama::DEFAULT_BASE_URL)` | `memfuse-ollama` | Test assertion for LLM Ollama URL |

### 3.2 `crates/memfuse-mcp/src/routing.rs`

| Line | Code Symbol / Expression | Crate | Purpose |
|------|--------------------------|-------|---------|
| 3 | `use memfuse_db::MemFuse;` | `memfuse-db` | Database handle reference for attaching router |
| 8 | `pub router: Arc<memfuse_router::DefaultRouterEngine>` | `memfuse-router` | RoutingHandle field for RouterEngine instance |
| 9 | `pub calibrator: Arc<parking_lot::Mutex<memfuse_calibration::IsotonicCalibrator>>` | `memfuse-calibration` | RoutingHandle field for IsotonicCalibrator |
| 10 | `pub pid_controller: Arc<parking_lot::Mutex<memfuse_calibration::PidController>>` | `memfuse-calibration` | RoutingHandle field for PidController |
| 25 | `memfuse_router::RouterEngine::new(...)` | `memfuse-router` | RouterEngine instantiation |
| 32 | `memfuse_calibration::IsotonicCalibrator::with_defaults()` | `memfuse-calibration` | IsotonicCalibrator instantiation |
| 36 | `memfuse_calibration::PidController::default()` | `memfuse-calibration` | PidController instantiation |
| 40 | `Arc::downgrade(&router) as std::sync::Weak<dyn memfuse_db::DriftStatusProvider>` | `memfuse-db` | Casting router to `DriftStatusProvider` trait for DB |

### 3.3 `crates/memfuse-mcp/src/egress_guard.rs`

| Line | Code Symbol / Expression | Crate | Purpose |
|------|--------------------------|-------|---------|
| 11 | `use memfuse_db::Collection;` | `memfuse-db` | EgressGuard struct holds `Arc<Collection>` |
| 24 | `/// Baut auf dem lokalen HNSW-Vektorindex einer memfuse_db::Collection auf.` | `memfuse-db` | Doc comment reference |
| 140 | `use memfuse_db::MemFuse;` | `memfuse-db` | Test helper fixture creating MemFuse DB |

### 3.4 `crates/memfuse-mcp/src/server_tools.rs`

| Line | Code Symbol / Expression | Crate | Purpose |
|------|--------------------------|-------|---------|
| 7 | `use memfuse_db::chunker::{ChunkerConfig, MarkdownChunker};` | `memfuse-db` | Markdown document auto-chunker in `memfuse_insert` |
| 430 | `memfuse_db::execute_background_consolidation(...)` | `memfuse-db` | Background memory consolidation execution in `memfuse_consolidate` |
| 433 | `&memfuse_db::memory_consolidation::ConsolidationConfig::default()` | `memfuse-db` | Consolidation config default struct |

### 3.5 `crates/memfuse-mcp/src/server.rs`

| Line | Code Symbol / Expression | Crate | Purpose |
|------|--------------------------|-------|---------|
| 11 | `use memfuse_db::MemFuse;` | `memfuse-db` | `McpServer` struct field `pub db: Arc<MemFuse>` |

### 3.6 `crates/memfuse-mcp/src/bin/memfuse-mcp-server.rs`

| Line | Code Symbol / Expression | Crate | Purpose |
|------|--------------------------|-------|---------|
| 8 | `use memfuse_db::MemFuse;` | `memfuse-db` | Entrypoint database instantiation via `MemFuse::open(...)` |
| 64 | `serde_json::from_slice::<Vec<memfuse_router::SlmProfile>>(&bytes)` | `memfuse-router` | Deserialization of router profiles in main binary |

### 3.7 `crates/memfuse-mcp/src/tests.rs`

| Line | Code Symbol / Expression | Crate | Purpose |
|------|--------------------------|-------|---------|
| 5 | `use memfuse_db::MemFuse;` | `memfuse-db` | Unit test fixture creating `MemFuse::open(...)` |

### 3.8 `crates/memfuse-mcp/tests/mcp_test.rs`

| Line | Code Symbol / Expression | Crate | Purpose |
|------|--------------------------|-------|---------|
| 2 | `use memfuse_db::MemFuse;` | `memfuse-db` | Integration test fixture creating `MemFuse::open(...)` |
| 973 | `let profile = memfuse_router::SlmProfile::new(...)` | `memfuse-router` | Integration test constructing `SlmProfile` |

---

## 4. Functional Classification & Architectural Recommendations

We classify the internal usages into four distinct functional clusters and determine the target architecture for each.

```
+-------------------------------------------------------------------------------+
|                             memfuse-mcp (Ring 4)                            |
+-------------------------------------------------------------------------------+
       |                                       |                         |
       | (Primary DB Operations)               | (Inference & Routing)   | (Chunker & Consolidation)
       v                                       v                         v
+-----------------------+              +-----------------------+ +-----------------------+
|  memfuse Facade Crate |              |   memfuse-ports Crate | |  memfuse Facade /     |
|   (MemFuse, Builder)  |              |  (Inference/Router    | |  memfuse-ports        |
|  [Re-exports MemFuse] |              |      Traits & Ports)  | |  [Re-exports Chunk/   |
+-----------------------+              +-----------------------+ |   Consolidation]      |
                                                                 +-----------------------+
```

### Cluster 1: Core Database Handle & Collections (`MemFuse`, `Collection`)
* **Current Usage:** `src/server.rs`, `src/routing.rs`, `src/egress_guard.rs`, `src/bin/memfuse-mcp-server.rs`, `src/tests.rs`, `tests/mcp_test.rs` directly import `memfuse_db::{MemFuse, Collection}`.
* **Architectural Target:** **`memfuse` Facade Crate**.
* **Rationale:** The `memfuse` crate (Ring 4) is designed as the canonical facade and re-exports `MemFuse`, `Collection`, `MemFuseConfig`, and domain types (`DocId`, `ScoredDocument`). Re-exporting `Collection` from `memfuse` allows `memfuse-mcp` to import `memfuse::{MemFuse, Collection}` instead of `memfuse_db`.

### Cluster 2: SLM Router & Calibration (`RouterEngine`, `SlmProfile`, `IsotonicCalibrator`, `PidController`)
* **Current Usage:** `src/routing.rs`, `src/config.rs`, `src/bin/memfuse-mcp-server.rs`, `tests/mcp_test.rs` directly import `memfuse_router::*` and `memfuse_calibration::*`.
* **Architectural Target:** **`memfuse` Facade / `memfuse-ports` Port Abstraction**.
* **Rationale:** Router configuration and calibration parameters are application-level extensions. `memfuse` facade should re-export `SlmProfile` and routing initialization helpers, or expose a builder method `MemFuseBuilder::with_router_config(...)`. `memfuse-ports` provides the `RouterPort` trait.

### Cluster 3: Ollama / Inference Engine Instantiation (`OllamaEmbedder`, `OllamaClient`, `OllamaConfig`)
* **Current Usage:** `src/config.rs` directly instantiates `memfuse_ollama::OllamaEmbedder` and `memfuse_ollama::OllamaClient`.
* **Architectural Target:** **`memfuse-ports` / `memfuse` Facade Feature Gate**.
* **Rationale:** Embedding providers implement `memfuse_core::EmbeddingProvider` and `memfuse_core::LlmTextGenerator`. Dynamic provider creation in `config.rs` should either be moved to factory methods re-exported by `memfuse` facade under feature flags (`ollama`, `candle`, `onnx`), or dispatched via port adapters in `memfuse-ports`.

### Cluster 4: Text Chunking & Memory Consolidation (`MarkdownChunker`, `execute_background_consolidation`)
* **Current Usage:** `src/server_tools.rs` directly calls `memfuse_db::chunker::MarkdownChunker` and `memfuse_db::execute_background_consolidation`.
* **Architectural Target:** **`memfuse` Facade Re-export / `MemFuse` High-Level Methods**.
* **Rationale:** Text chunking and memory consolidation are core features of MemFuse. High-level methods should be exposed directly on `MemFuse` or `Collection` (e.g. `col.consolidate(...)`), or `MarkdownChunker` and `execute_background_consolidation` should be re-exported through `memfuse::chunker` and `memfuse::consolidation`.

---

## 5. Wave 3 Implementation Plan & Affected File Matrix

To remove direct dependencies on internal crates from `memfuse-mcp` without breaking functionality or introducing cyclic dependencies, Wave 3 should execute in the following three phased steps:

### Phase 3a: Extend `memfuse` Facade & `memfuse-ports`
Before modifying `memfuse-mcp`, update the Ring 4 facade crate (`crates/memfuse`):
1. **`crates/memfuse/Cargo.toml`:**
   - Add feature gates for `ollama`, `router`, `calibration` if needed.
2. **`crates/memfuse/src/lib.rs`:**
   - Re-export `Collection`, `CollectionConfig`, `chunker`, `memory_consolidation` from `memfuse_db`.
   - Re-export `SlmProfile`, `RouterConfig` from `memfuse_router` (when `router` feature enabled).
   - Re-export `OllamaEmbedder`, `OllamaClient`, `OllamaConfig` from `memfuse_ollama` (when `ollama` feature enabled).

### Phase 3b: Refactor `crates/memfuse-mcp` Source Files

| Affected File | Target Changes | Replacement Imports |
|---------------|----------------|---------------------|
| `crates/memfuse-mcp/Cargo.toml` | Replace `memfuse-db`, `memfuse-router`, `memfuse-ollama`, `memfuse-calibration` dependencies with `memfuse = { workspace = true, features = ["ollama", "router"] }` and `memfuse-ports`. | `memfuse = { workspace = true }` |
| `crates/memfuse-mcp/src/server.rs` | Update `db` field type and imports. | `use memfuse::{MemFuse, Collection};` |
| `crates/memfuse-mcp/src/egress_guard.rs` | Update `Collection` and test `MemFuse` imports. | `use memfuse::{Collection, MemFuse};` |
| `crates/memfuse-mcp/src/routing.rs` | Use facade/port re-exports for `RouterEngine`, `IsotonicCalibrator`, `PidController`. | `use memfuse::routing::*;` |
| `crates/memfuse-mcp/src/config.rs` | Use facade re-exports for `OllamaEmbedder`, `OllamaClient`, `SlmProfile`. | `use memfuse::inference::ollama::*;` |
| `crates/memfuse-mcp/src/server_tools.rs` | Use facade re-exports for `MarkdownChunker` and `execute_background_consolidation`. | `use memfuse::chunker::*;` |
| `crates/memfuse-mcp/src/bin/memfuse-mcp-server.rs` | Update `MemFuse` and `SlmProfile` imports. | `use memfuse::{MemFuse, SlmProfile};` |
| `crates/memfuse-mcp/src/tests.rs` | Update unit test imports. | `use memfuse::MemFuse;` |
| `crates/memfuse-mcp/tests/mcp_test.rs` | Update integration test imports. | `use memfuse::{MemFuse, SlmProfile};` |

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
  ├── Update `crates/memfuse/src/lib.rs` (Facade re-exports)
  └── Update `crates/memfuse-ports/src/lib.rs` (Port traits if required)
  │
  ▼
[Step 2: MCP Refactoring]
  │
  ├── Refactor `crates/memfuse-mcp/Cargo.toml`
  ├── Refactor `src/{config, routing, server, server_tools, egress_guard}.rs`
  └── Refactor `src/bin/memfuse-mcp-server.rs` & tests
  │
  ▼
[Step 3: Verification Gates]
  │
  ├── `just check`
  ├── `just dag-check`
  └── `cargo xtask jules-preflight`
```

### Conflict Avoidance Guidelines
1. **Facade First:** Phase 3a (`memfuse` facade re-exports) MUST be completed and merged before Phase 3b (`memfuse-mcp` import changes) is executed.
2. **Crate Lock Claim:** Any agent executing Wave 3 changes on `memfuse-mcp` MUST register a claim via `cargo xtask claim --crate memfuse-mcp`.
3. **Workspace Root Cargo.toml:** Do NOT touch workspace root `Cargo.toml` unless workspace dependency aliases change.

---

## 7. Safety Invariants & Preflight Verification Criteria

When executing the refactoring in Wave 3, the following strict repository invariants must be preserved:

1. **Zero-Panic Doctrine:** No `.unwrap()` or `.expect()` calls in production code paths. Propagate errors via `?` and `MemFuseError` or `McpError`.
2. **Zero-Copy & Alignment:** Maintain existing zero-copy `Bytes` buffers and 64-byte aligned SIMD vector structures in search pipelines.
3. **DAG Layering Integrity:** `just dag-check` must pass without warnings or errors.
4. **Unsafe Isolation:** `#![forbid(unsafe_code)]` must remain strictly enforced at crate roots of `memfuse-mcp` and `memfuse`.
5. **Preflight Gate:** `cargo xtask jules-preflight` must pass with zero errors.

---

*End of Audit Report.*
