# FlatBuffers Code Generation & Governance

## Overview
This document describes the auto-generated FlatBuffers IPC code stored in `crates/contextra-wire/src/contextra_generated.rs` and managed by the `generated` module (`crates/contextra-wire/src/generated.rs`).

## Generator Tooling & Version
- **Compiler:** `flatc` (FlatBuffers Compiler)
- **Version:** `24.12.23` (FlatBuffers 24.3 Rust runtime)
- **Schema Source:** `schemas/contextra.fbs`

## Regeneration Command
To regenerate `src/contextra_generated.rs` from `schemas/contextra.fbs`, run:
```bash
cargo xtask regenerate-flatbuffers
```
Alternatively, using `flatc` directly:
```bash
flatc --rust -o crates/contextra-wire/src schemas/contextra.fbs
```

## Safety Governance & Unsafe Baseline
- **Absence of `// SAFETY:` Comments:** Code produced by `flatc` contains `unsafe` blocks for low-level byte table access without generating inline `// SAFETY:` comments.
- **Strict Drift Protection:** To preserve schema consistency and ensure `check-flatbuffers-drift` passes, hand editing of `src/contextra_generated.rs` is strictly prohibited.
- **Module Isolation:** All generated code is encapsulated within `crates/contextra-wire/src/generated.rs`, which applies module-level lints and suppression (`@generated, siehe check-flatbuffers-drift`).
- **Baseline Exception (GV-1/XT-1):** The unsafe block count in `contextra_generated.rs` (43 unsafe blocks) is covered under the baseline exception requested via GV-1/XT-1 in `.agents/governance/unsafe-safety-baseline.toml`.
