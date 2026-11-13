# AGENTS.md — `contextra-sandbox`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-sandbox` · **Ring:** 2 · **Version:** 0.1.0
- **Beschreibung:** WASM Execution Boundary for Contextra MCP CodeExecution Permission
- **src:** 13 Dateien, 1905 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 0):** –
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 6):** `anyhow`, `thiserror`, `tokio`, `tracing`, `wasmtime`, `zeroize`
- **Reverse-Abhängigkeiten (1):** `contextra-db`
- **Kopplung an Hub-Crates:** ausgehend –; eingehend `contextra-db`
- **Features (2):** `default`, `wasm`

### Öffentliche API (grob, aus src/)
- struct (10): `ApprovalRequest`, `CapabilityViolationError`, `HostCallLimitExceededError`, `MergeOperatorCapabilities`, `OutputLimitExceededError`, `ProcessExitError`, `WasmCapabilities`, `WasmExecutor`, `WasmMergeFunction`, `WasmOutput`
- enum (4): `ApprovalRisk`, `ApprovalStatus`, `ApprovalTransitionError`, `SandboxError`
- fn (16): `approve`, `check_and_expire`, `classify_risk`, `execute`, `expire`, `is_expired`, `merge`, `new`, `new_pure`, `new_with_capabilities`, `pure`, `pure_merge_operator`, `pure_with_result_channel`, `reject`, `requires_approval` … (+1)
- mod (7): `approval`, `capabilities`, `error`, `executor`, `merge`, `output`, `wasi`
- pub use: `approval::{ classify_risk, ApprovalRequest, ApprovalRisk, ApprovalStatus, ApprovalTransitionError, }`, `capabilities::{MergeOperatorCapabilities, WasmCapabilities}`, `error::SandboxError`, `executor::WasmExecutor`, `merge::WasmMergeFunction`, `output::WasmOutput`, `state::CapabilityViolationError`

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->`.

<!-- END GENERATED: crate_inventory.py -->
