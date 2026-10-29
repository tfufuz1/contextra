# Wurzelursachen-Analyse (Baseline Root Causes)

- **Commit:** `6202ecf39af06913fae226f75541a46ec1f4a9d7`
- **Toolchain:** `rustc 1.89.0 (29483883e 2025-08-04)`
- **Stand:** 2026-10-29 01:34:47 UTC

| Root Cause Crate | Lint / Fehlercode | Betroffene Crates | Fundstelle | Betroffene Crates Liste |
|---|---|---|---|---|
| `contextra-ports` | `clippy::cast_possible_truncation` | 27 | `crates/contextra-ports/src/clock.rs:45:22` | contextra, contextra-adapt, contextra-agent, contextra-audit-export, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-core, contextra-crypto, contextra-db, contextra-engine, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-mvcc, contextra-ports, contextra-privacy, contextra-py, contextra-rank, contextra-router, contextra-testkit, contextra-text, contextra-vector |
| `contextra-ports` | `clippy::disallowed_methods` | 27 | `crates/contextra-ports/src/clock.rs:29:28` | contextra, contextra-adapt, contextra-agent, contextra-audit-export, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-core, contextra-crypto, contextra-db, contextra-engine, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-mvcc, contextra-ports, contextra-privacy, contextra-py, contextra-rank, contextra-router, contextra-testkit, contextra-text, contextra-vector |
| `contextra-ports` | `clippy::indexing_slicing` | 27 | `crates/contextra-ports/src/rng.rs:92:13` | contextra, contextra-adapt, contextra-agent, contextra-audit-export, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-core, contextra-crypto, contextra-db, contextra-engine, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-mvcc, contextra-ports, contextra-privacy, contextra-py, contextra-rank, contextra-router, contextra-testkit, contextra-text, contextra-vector |
| `contextra-simd` | `clippy::cast_sign_loss` | 11 | `crates/contextra-simd/src/kernels/avx2.rs:397:19` | contextra, contextra-agent, contextra-bench, contextra-cognition, contextra-db, contextra-engine, contextra-mcp, contextra-py, contextra-router, contextra-simd, contextra-vector |
| `contextra-simd` | `clippy::indexing_slicing` | 11 | `crates/contextra-simd/src/kernels/scalar.rs:67:13` | contextra, contextra-agent, contextra-bench, contextra-cognition, contextra-db, contextra-engine, contextra-mcp, contextra-py, contextra-router, contextra-simd, contextra-vector |
| `contextra-engine` | `dead_code` | 9 | `crates/contextra-engine/src/lib.rs:350:5` | contextra, contextra-agent, contextra-bench, contextra-cognition, contextra-db, contextra-engine, contextra-mcp, contextra-py, contextra-router |
| `contextra-engine` | `unused_imports` | 9 | `crates/contextra-engine/src/contextra_impl/erasure.rs:10:19` | contextra, contextra-agent, contextra-bench, contextra-cognition, contextra-db, contextra-engine, contextra-mcp, contextra-py, contextra-router |
| `contextra-sandbox` | `clippy::cast_possible_truncation` | 2 | `crates/contextra-sandbox/src/executor.rs:63:29` | contextra-sandbox, contextra-store |
| `contextra-sandbox` | `clippy::cast_sign_loss` | 2 | `crates/contextra-sandbox/src/wasi.rs:79:30` | contextra-sandbox, contextra-store |
| `contextra-sandbox` | `clippy::disallowed_methods` | 2 | `crates/contextra-sandbox/src/executor.rs:218:32` | contextra-sandbox, contextra-store |
| `contextra-sandbox` | `clippy::indexing_slicing` | 2 | `crates/contextra-sandbox/src/wasi.rs:104:56` | contextra-sandbox, contextra-store |
| `contextra-avv-generator` | `clippy::disallowed_methods` | 1 | `crates/contextra-avv-generator/tests/campaign_j27_avv_test.rs:65:24` | contextra-avv-generator |
| `contextra-avv-generator` | `clippy::expect_used` | 1 | `crates/contextra-avv-generator/tests/campaign_j27_avv_test.rs:20:18` | contextra-avv-generator |
| `contextra-db` | `E0599` | 1 | `crates/contextra-db/tests/deletion_proof_integration.rs:232:42` | contextra-db |
| `contextra-db` | `deprecated` | 1 | `crates/contextra-db/examples/verify_deletion_proof_external.rs:83:31` | contextra-db |
| `contextra-engine` | `deprecated` | 1 | `crates/contextra-engine/tests/tenant_index_persistence_reload.rs:27:30` | contextra-engine |
| `contextra-ports` | `clippy::disallowed_types` | 1 | `crates/contextra-ports/src/id_gen.rs:50:5` | contextra-ports |
| `contextra-store` | `dead_code` | 1 | `crates/contextra-store/tests/differential/redb_operation_sequence.rs:13:4` | contextra-store |
| `external` | `compiler_error` | 1 | `clippy.toml:29:1` | contextra-types |
| `xtask` | `clippy::disallowed_methods` | 1 | `xtask/build.rs:12:25` | xtask |
| `xtask` | `clippy::unwrap_used` | 1 | `xtask/build.rs:87:24` | xtask |
| `xtask` | `dead_code` | 1 | `xtask/src/gates/code/code_gates.rs:21:5` | xtask |
| `xtask` | `unused_imports` | 1 | `xtask/tests/veto_deadline_gate_test.rs:13:62` | xtask |
| `xtask` | `unused_variables` | 1 | `xtask/src/gates/gov/meta_gates.rs:22:19` | xtask |
