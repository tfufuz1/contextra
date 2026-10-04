# AGENTS.md — contextra-sandbox
> Ring 2 · stable · Quelle: capabilities.toml · Spec: K.24, L.6

## 1. Zweck
Stellt eine isolierte WASM-Ausführungsgrenze für die `CodeExecution`-Berechtigung des MCP-Servers (§4.18) sowie für benutzerdefinierte WASM-Merge-Operatoren bereit. Führt vorkompilierte WASM-Module in einer gesicherten Sandbox mit striktem CPU-Fuel-Budget und Wall-Clock-Timeout aus.

## 2. Modul-Karte

| Datei / Verzeichnis | Verantwortung |
|---|---|
| `src/lib.rs` | `#![forbid(unsafe_code)]`, Crate-Dokumentation und Modul-Exporte |
| `src/approval.rs` | `ApprovalRequest`, `ApprovalRisk` und `ApprovalStatus` zur Risikobewertung und Freigabe |
| `src/capabilities.rs` | `WasmCapabilities` (Guest-Whitelist) und `MergeOperatorCapabilities::pure()` für pure Operatoren |
| `src/error.rs` | `SandboxError` Fehlertypen (§4.18: `Timeout`, `FuelExhausted`, `CapabilityViolation` etc.) |
| `src/executor.rs` | `WasmExecutor` Ausführungs-Engine und `CapabilityViolationError` |
| `src/merge.rs` | `WasmMergeFunction` Brücke für WASM Merge-Operatoren |
| `src/output.rs` | `WasmOutput` mit `ZeroizeOnDrop` für stdout (P9 Security Invariante) |
| `src/wasi.rs` | Handgeschriebener WASI-preview1 Host auf Wasmtime-Linker (`ProcessExitError`, `OutputLimitExceededError`) |

## 3. Invarianten

- **Strict Safe Rust:** `#![forbid(unsafe_code)]` erzwingt 100 % Safe Rust; Interaktion mit Wasmtime nutzt ausschließlich safe APIs.
- **Doppelte Isolierung (§4.18):** Jede WASM-Ausführung MUSS durch ein CPU-Fuel-Budget (`max_fuel`) UND ein Wall-Clock-Timeout (`tokio::time::timeout_at`) begrenzt sein (`cargo test -p contextra-sandbox --test wasm_boundary_tests`).
- **Zeroize on Drop (P9):** `WasmOutput.stdout` wird über `ZeroizeOnDrop` beim Verlassen des Scopes sicher aus dem RAM gelöscht (`output.rs`).
- **Fail-Closed Capabilities:** In `WasiHost` / `WasmCapabilities` sind Netzwerk, Filesystem und Cloud-Egress standardmäßig deaktiviert (`capabilities.rs`).

## 4. Verboten / Anti-Patterns

- **Verboten:** `unsafe`-Code in `contextra-sandbox` verwenden.
- **Verboten:** WASM-Ausführungen ohne Fuel-Begrenzung oder ohne Wall-Clock-Timeout zu starten.
- **Verboten:** Wasmtime `Store`- oder `Instance`-Zustände über mehrere Execution-Aufrufe hinweg wiederzuverwenden (erfordert State Isolation).

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- Jeder `execute()`-Aufruf erzeugt eine frische Wasmtime `Store`- und `Instance`-Instanz zur vollkommenen Zustandstrennung.
- Timeouts werden asynchron über `tokio::time::timeout_at` durchgesetzt, ohne den Thread zu blockieren.

## 6. Verifikation

- `cargo test -p contextra-sandbox --locked`
- `cargo test -p contextra-sandbox --test wasm_boundary_tests`
- `cargo test -p contextra-sandbox --test pure_merge_operator_capabilities_test`
- `cargo test -p contextra-sandbox --test wasm_wasi_io`

## 7. Bekannte Lücken / SOLL

- Handgeschriebener WASI-preview1 Host (`wasi.rs`) implementiert eine gezielte Minimalauswahl an System-Calls (`fd_read`, `fd_write`, `proc_exit`, `clock_time_get`, `random_get`); ununterstützte Host-Calls liefern `ERRNO_NOSYS` / `ERRNO_NOTSUP`.
