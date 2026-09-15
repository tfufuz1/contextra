# Tier 2 Deep Audit Report: `memfuse-sandbox` (WASM Execution Boundary)

**Crate:** `memfuse-sandbox` (Layer 6.5)
**Date:** 2026-09-15
**Auditor:** Senior Rust WASM-Security Engineer (Jules)
**HEAD Commit:** `4eadcbe`
**Scope:** `crates/memfuse-sandbox/src/` (`lib.rs`, `capabilities.rs`, `executor.rs`, `output.rs`, `error.rs`) and `crates/memfuse-sandbox/tests/`

---

## Executive Summary

`memfuse-sandbox` provides an isolated, capability-gated WASM execution boundary for the MemFuse MCP server (`CodeExecution` permission).
The crate strictly enforces `#![forbid(unsafe_code)]` across all source files. Wasmtime configuration enforces dual isolation mechanisms: CPU fuel budgeting (`set_fuel`) and asynchronous wall-clock timeout wrapping (`tokio::time::timeout`).

All core security invariants specified in §4.18 and §10.14 are verified and covered by unit and integration tests.

---

## Detailed Invariant Audit Matrix

| Invariant / Requirement | Status | Verification Method & Reference |
|---|---|---|
| **Safe Rust Enforcement** (`#![forbid(unsafe_code)]`) | **VERIFIED** | Enforced in `lib.rs:1`, verified by `wasm_audit_checker.py`. |
| **Fail-Closed Default Capabilities** | **VERIFIED** | `WasmCapabilities::default()` sets `allow_cloud_egress: false`, `allow_filesystem: false`, `allow_network: false`, `max_memory_pages: 16`. Tested in `capabilities.rs:46`. |
| **WASM Memory Isolation** (§10.14) | **VERIFIED** | Enforced via `wasmtime::ResourceLimiter` in `executor.rs:13`. Verified in test `test_wasm_memory_isolation_property_variations` (`tests/wasm_boundary_tests.rs:7`). |
| **Fuel Exhaustion Error** (§10.14) | **VERIFIED** | Enforced via `store.set_fuel()` and `start_fn.call_async()`. Verified in test `test_wasm_fuel_exhaustion_returns_error_property` (`tests/wasm_boundary_tests.rs:52`). |
| **Wall-Clock Timeout Isolation** | **VERIFIED** | Enforced via `tokio::time::timeout` in `executor.rs:206`. Verified in `executor.rs:218`. |
| **Zeroize on Drop (P9)** | **VERIFIED** | `WasmOutput.stdout` wraps `zeroize::Zeroizing<Vec<u8>>` in `output.rs:13`. |
| **Cloud Egress Host Function Guard** | **VERIFIED** | `host_cloud_query` dynamic check in `executor.rs:168`. Verified in test `test_cloud_egress_strict_capability_isolation` (`tests/wasm_boundary_tests.rs:80`). |

---

## Audit Findings & Observations

- **Finding AGT-SANDBOX-12a4a39c (`AI-TAG[SMELL][MINOR]`)**:
  - **Location**: `crates/memfuse-sandbox/src/executor.rs:134`
  - **BEFUND**: The WASI `fd_write` linker function stub currently discards input buffer pointers (`iovs`) and returns success `0`.
  - **RISIKO**: WASM guest modules calling standard WASI `fd_write` (e.g. `println!`) execute cleanly without trapping, but outputs in `WasmOutput` remain empty.
  - **EMPFEHLUNG**: Integrate full WASI preview1 guest memory reader to route `fd_write` buffers into `stdout_buf` / `stderr_buf` when enabled.

---

## Tooling & Verification Log

- **Tooling Executed**:
  - `/home/jules/self_created_tools/wasm_audit_checker.py` -> Passed
  - `cargo run -p xtask -- check-agents-integrity` -> Passed
  - `cargo llvm-cov -p memfuse-sandbox --all-features` -> 88.49% Line Coverage
  - `cargo test -p memfuse-sandbox --all-features -- --test-threads=8` (10 repetitions) -> 0 Failures / Races detected.
