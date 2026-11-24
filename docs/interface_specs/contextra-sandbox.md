# Schnittstellenspezifikation: `contextra-sandbox`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-sandbox` |
| Version | workspace |
| Pfad | `crates/contextra-sandbox` |
| Rolle | Sandbox (Wasm) |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> Contextra WASM Execution Boundary (§4.18).
>
> Stellt eine eng begrenzte WASM-Ausführungsgrenze für die `CodeExecution`-Permission bereit.
> Kein WASM-Compiler — nur Ausführung vorab kompilierter `.wasm`-Binaries.
>
> # Sicherheitsgrundsätze
> - `#![forbid(unsafe_code)]`: Alle Interaktionen via wasmtime's sichere Rust-API
> - Fuel + Wall-Clock-Timeout: beide aktiv (§4.18)
> - `WasmOutput.stdout` ist `ZeroizeOnDrop` (P9)
> - Kein Dateisystem-/Netzwerkzugriff per Default

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `anyhow` | normal | registry | ja |  |
| `criterion` | dev | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | dev | workspace | nein |  |
| `tracing` | normal | workspace | nein |  |
| `wasmtime` | normal | registry | ja |  |
| `wat` | dev | registry | nein |  |
| `zeroize` | normal | registry | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-db`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `default` | – |
| `wasm` | `dep:wasmtime`, `dep:tokio`, `dep:anyhow` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 13 |
| Quellzeilen | 2377 |
| Öffentliche Items (Modul-Ebene) | 22 |
| davon `fn` | 1 |
| davon `struct` | 10 |
| davon `enum` | 4 |
| davon `mod` | 7 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 7 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 2 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 5 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 40 / 40 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `approval` | `crates/contextra-sandbox/src/approval.rs` | 532 |
| `capabilities` | `crates/contextra-sandbox/src/capabilities.rs` | 183 |
| `error` | `crates/contextra-sandbox/src/error.rs` | 38 |
| `executor` | `crates/contextra-sandbox/src/executor/mod.rs` | 160 |
| `executor::runner` | `crates/contextra-sandbox/src/executor/runner.rs` | 151 |
| `executor::state` | `crates/contextra-sandbox/src/executor/state.rs` | 92 |
| `executor::tests` | `crates/contextra-sandbox/src/executor/tests.rs` | 327 |
| `crate` | `crates/contextra-sandbox/src/lib.rs` | 33 |
| `merge` | `crates/contextra-sandbox/src/merge.rs` | 118 |
| `output` | `crates/contextra-sandbox/src/output.rs` | 29 |
| `wasi::fd` | `crates/contextra-sandbox/src/wasi/fd.rs` | 325 |
| `wasi` | `crates/contextra-sandbox/src/wasi/mod.rs` | 52 |
| `wasi::sys` | `crates/contextra-sandbox/src/wasi/sys.rs` | 337 |

## 6. Re-Exports (`pub use`)

- `pub use state::CapabilityViolationError` — `crates/contextra-sandbox/src/executor/mod.rs:9` (Modul `executor`)
- `pub use approval::` — `crates/contextra-sandbox/src/lib.rs:24` (Modul `crate`)
- `pub use capabilities::` — `crates/contextra-sandbox/src/lib.rs:27` (Modul `crate`)
- `pub use error::SandboxError` — `crates/contextra-sandbox/src/lib.rs:28` (Modul `crate`)
- `pub use executor::WasmExecutor` — `crates/contextra-sandbox/src/lib.rs:30` (Modul `crate`)
- `pub use merge::WasmMergeFunction` — `crates/contextra-sandbox/src/lib.rs:32` (Modul `crate`)
- `pub use output::WasmOutput` — `crates/contextra-sandbox/src/lib.rs:33` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item

### Modul `approval`

#### `struct ApprovalRequest`
- Ort: `crates/contextra-sandbox/src/approval.rs:82`
```rust
pub struct ApprovalRequest
```
Dokumentation: Human approval request state machine for a WASM execution run.
- Impl-Block `crates/contextra-sandbox/src/approval.rs:97`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `request_id` | `String` | `crates/contextra-sandbox/src/approval.rs:84` |
| `risk` | `ApprovalRisk` | `crates/contextra-sandbox/src/approval.rs:86` |
| `requested_capabilities_summary` | `String` | `crates/contextra-sandbox/src/approval.rs:88` |
| `status` | `ApprovalStatus` | `crates/contextra-sandbox/src/approval.rs:90` |
| `created_at_unix_ms` | `u64` | `crates/contextra-sandbox/src/approval.rs:92` |
| `ttl_ms` | `u64` | `crates/contextra-sandbox/src/approval.rs:94` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( request_id: String, caps: &WasmCapabilities, created_at_unix_ms: u64, ttl_ms: u64, ) -> Self` | `crates/contextra-sandbox/src/approval.rs:102` | Creates a new approval request in `Pending` status. Computes `risk` via `classify_risk` and generates `requested_capabilities_summary` from fields differing fro |
| `pub fn is_expired(&self, now_unix_ms: u64) -> bool` | `crates/contextra-sandbox/src/approval.rs:124` | Checks if the request is expired relative to `now_unix_ms`. Uses `saturating_add` to prevent overflow panic on boundary values. |
| `pub fn approve( mut self, approved_by: String, now_unix_ms: u64, ) -> Result<Self, ApprovalTransitionError>` | `crates/contextra-sandbox/src/approval.rs:132` | Transitions request from `Pending` to `Approved`. Fails with `ApprovalTransitionError` if not in `Pending` state or if expired at `now_unix_ms`. |
| `pub fn reject( mut self, rejected_by: String, reason: String, now_unix_ms: u64, ) -> Result<Self, ApprovalTransitionError>` | `crates/contextra-sandbox/src/approval.rs:170` | Transitions request from `Pending` to `Rejected`. Fails with `ApprovalTransitionError` if not in `Pending` state or if expired at `now_unix_ms`. |
| `pub fn expire(mut self, now_unix_ms: u64) -> Result<Self, ApprovalTransitionError>` | `crates/contextra-sandbox/src/approval.rs:209` | Explicitly transitions request from `Pending` to `Expired`. |
| `pub fn check_and_expire(self, now_unix_ms: u64) -> Result<Self, ApprovalTransitionError>` | `crates/contextra-sandbox/src/approval.rs:231` | Checks if request is expired and transitions to `Expired` if pending. |
| `pub fn requires_approval(&self) -> bool` | `crates/contextra-sandbox/src/approval.rs:245` | Determines whether this execution requires human approval before proceeding. # Product Rationale Low-risk executions (default sandboxed capabilities without net |

#### `enum ApprovalRisk`
- Ort: `crates/contextra-sandbox/src/approval.rs:11`
```rust
pub enum ApprovalRisk
```
Dokumentation: Risk level classification for a requested WASM execution configuration.

Varianten:

- `Low` — `crates/contextra-sandbox/src/approval.rs:13`
- `Elevated` — `crates/contextra-sandbox/src/approval.rs:15`
- `High` — `crates/contextra-sandbox/src/approval.rs:17`

#### `enum ApprovalStatus`
- Ort: `crates/contextra-sandbox/src/approval.rs:41`
```rust
pub enum ApprovalStatus
```
Dokumentation: Status of a human approval decision.

Varianten:

- `Pending` — `crates/contextra-sandbox/src/approval.rs:43`
- `Approved{` — `crates/contextra-sandbox/src/approval.rs:45`
- `Rejected{` — `crates/contextra-sandbox/src/approval.rs:50`
- `Expired{ expired_at_unix_ms: u64 }` — `crates/contextra-sandbox/src/approval.rs:56`

#### `enum ApprovalTransitionError`
- Ort: `crates/contextra-sandbox/src/approval.rs:61`
```rust
pub enum ApprovalTransitionError
```
Dokumentation: Errors occurring during approval state transitions.

Varianten:

- `Expired{` — `crates/contextra-sandbox/src/approval.rs:66`
- `InvalidStateTransition{` — `crates/contextra-sandbox/src/approval.rs:74`

#### `fn classify_risk`
- Ort: `crates/contextra-sandbox/src/approval.rs:29`
```rust
pub fn classify_risk(caps: &WasmCapabilities) -> ApprovalRisk
```
Dokumentation: Derives the execution risk level deterministically from `WasmCapabilities`. # Rule Matrix | Capability Condition | Derived Risk Level | Rationale | |---|---|---| | `allow_network == true` \|\| `allow_cloud_egress == true` | `ApprovalRisk::High` | Egress/Network allows potential data exfiltration or external side-effects. | | `allow_filesystem == true` (without network/egress) | `ApprovalRisk::Elevated` | Filesystem access permits persistent read/write operations locally. | | Default safe capabilities or non-elevated overrides | `ApprovalRisk::Low` | Sandboxed in-memory execution capped by fuel

### Modul `capabilities`

#### `struct MergeOperatorCapabilities`
- Ort: `crates/contextra-sandbox/src/capabilities.rs:75`
```rust
pub struct MergeOperatorCapabilities
```
Dokumentation: Dedicated pure capabilities preset for I/O-free WASM Merge Operators (§4.18).
- Impl-Block `crates/contextra-sandbox/src/capabilities.rs:77`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn pure() -> WasmCapabilities` | `crates/contextra-sandbox/src/capabilities.rs:91` | Returns a strict `WasmCapabilities` preset for pure merge operators. # Security Profile - `allow_stdout`: `false` - `allow_stderr`: `false` - `allow_filesystem` |
| `pub fn pure_with_result_channel() -> WasmCapabilities` | `crates/contextra-sandbox/src/capabilities.rs:115` | Returns a strict `WasmCapabilities` preset for merge operators with stdout enabled for returning output bytes. Differs from `pure()` strictly by `allow_stdout = |

#### `struct WasmCapabilities`
- Ort: `crates/contextra-sandbox/src/capabilities.rs:9`
```rust
pub struct WasmCapabilities
```
Dokumentation: Whitelist für WASM-Guest-Capabilities. # Sicherheitsmodell Kein Dateisystem- und Netzwerkzugriff per Default. Monotone Uhr ist erlaubt (deterministisch, kein Side-Channel).
- Impl-Block (Trait `Default`) `crates/contextra-sandbox/src/capabilities.rs:44`
- Impl-Block `crates/contextra-sandbox/src/capabilities.rs:66`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `allow_stdout` | `bool` | `crates/contextra-sandbox/src/capabilities.rs:11` |
| `allow_stderr` | `bool` | `crates/contextra-sandbox/src/capabilities.rs:13` |
| `max_memory_pages` | `u32` | `crates/contextra-sandbox/src/capabilities.rs:15` |
| `max_fuel` | `u64` | `crates/contextra-sandbox/src/capabilities.rs:17` |
| `allow_filesystem` | `bool` | `crates/contextra-sandbox/src/capabilities.rs:19` |
| `allow_network` | `bool` | `crates/contextra-sandbox/src/capabilities.rs:21` |
| `allow_clock` | `bool` | `crates/contextra-sandbox/src/capabilities.rs:23` |
| `allow_cloud_egress` | `bool` | `crates/contextra-sandbox/src/capabilities.rs:25` |
| `max_wall_clock_ms` | `u64` | `crates/contextra-sandbox/src/capabilities.rs:29` |
| `max_module_size_bytes` | `usize` | `crates/contextra-sandbox/src/capabilities.rs:31` |
| `max_table_entries` | `u32` | `crates/contextra-sandbox/src/capabilities.rs:33` |
| `max_output_bytes` | `usize` | `crates/contextra-sandbox/src/capabilities.rs:35` |
| `max_stdin_bytes` | `usize` | `crates/contextra-sandbox/src/capabilities.rs:37` |
| `random_seed` | `Option<u64>` | `crates/contextra-sandbox/src/capabilities.rs:39` |
| `max_host_calls` | `u32` | `crates/contextra-sandbox/src/capabilities.rs:41` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn pure_merge_operator() -> Self` | `crates/contextra-sandbox/src/capabilities.rs:68` | Returns a strict `WasmCapabilities` preset for pure merge operators. |

### Modul `crate`

#### `mod approval`
- Ort: `crates/contextra-sandbox/src/lib.rs:13`
```rust
pub mod approval
```

#### `mod capabilities`
- Ort: `crates/contextra-sandbox/src/lib.rs:14`
```rust
pub mod capabilities
```

#### `mod error`
- Ort: `crates/contextra-sandbox/src/lib.rs:15`
```rust
pub mod error
```

#### `mod executor`
- Ort: `crates/contextra-sandbox/src/lib.rs:17`
```rust
pub mod executor
```
- Gate: `#[cfg(feature = "wasm")]`

#### `mod merge`
- Ort: `crates/contextra-sandbox/src/lib.rs:19`
```rust
pub mod merge
```
- Gate: `#[cfg(feature = "wasm")]`

#### `mod output`
- Ort: `crates/contextra-sandbox/src/lib.rs:20`
```rust
pub mod output
```

#### `mod wasi`
- Ort: `crates/contextra-sandbox/src/lib.rs:22`
```rust
pub mod wasi
```
- Gate: `#[cfg(feature = "wasm")]`

### Modul `error`

#### `enum SandboxError`
- Ort: `crates/contextra-sandbox/src/error.rs:8`
```rust
pub enum SandboxError
```
Dokumentation: Fehlertypen der WASM-Ausführungsgrenze.

Varianten:

- `Timeout{ timeout_ms: u64 }` — `crates/contextra-sandbox/src/error.rs:10`
- `MemoryExceeded{ pages: u32, max_pages: u32 }` — `crates/contextra-sandbox/src/error.rs:13`
- `FuelExhausted{ consumed: u64 }` — `crates/contextra-sandbox/src/error.rs:16`
- `CapabilityViolation{ capability: String }` — `crates/contextra-sandbox/src/error.rs:19`
- `WasmTrap(String)` — `crates/contextra-sandbox/src/error.rs:22`
- `InvalidModule(String)` — `crates/contextra-sandbox/src/error.rs:25`
- `OutputLimitExceeded{ stream: &'static str, limit: usize }` — `crates/contextra-sandbox/src/error.rs:28`
- `InputTooLarge{ len: usize, limit: usize }` — `crates/contextra-sandbox/src/error.rs:31`
- `ProcessExit{ code: i32 }` — `crates/contextra-sandbox/src/error.rs:34`
- `Runtime(String)` — `crates/contextra-sandbox/src/error.rs:37`

### Modul `executor`

#### `struct WasmExecutor`
- Ort: `crates/contextra-sandbox/src/executor/mod.rs:20`
```rust
pub struct WasmExecutor
```
Dokumentation: WASM-Ausführungsgrenze für die `CodeExecution`-Permission. # Designprinzip (§4.18) - Frische `Store` + `Instance` pro execute()-Aufruf: kein Zustandsüberlauf - Fuel-Budget (CPU, deterministisch) + Wall-Clock-Timeout (nicht-deterministisch) — beide pflicht - `#![forbid(unsafe_code)]` — ausschließlich wasmtime's sichere Rust-API
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-sandbox/src/executor/mod.rs:24`
- Impl-Block `crates/contextra-sandbox/src/executor/mod.rs:30`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Result<Self, SandboxError>` | `crates/contextra-sandbox/src/executor/mod.rs:35` | Erstellt einen neuen WasmExecutor. # Errors Wenn wasmtime::Engine nicht initialisiert werden kann. |
| `pub fn validate_module( &self, wasm_bytes: &[u8], max_module_size_bytes: usize, ) -> Result<(), SandboxError>` | `crates/contextra-sandbox/src/executor/mod.rs:49` | Validates a WASM binary without executing it. Checks binary size limits and attempts module compilation. |
| `pub async fn execute( &self, wasm_bytes: &[u8], input: &[u8], capabilities: &WasmCapabilities, timeout: Duration, ) -> Result<WasmOutput, SandboxError>` | `crates/contextra-sandbox/src/executor/mod.rs:73` | Führt ein WASM-Binary mit Capability-Einschränkungen aus. # Garantien - Fuel-Budget: CPU-Limit (deterministisch), verhindert Endlosschleifen - Wall-Clock-Timeou |

### Modul `executor::state`

#### `struct CapabilityViolationError`
- Ort: `crates/contextra-sandbox/src/executor/state.rs:6`
```rust
pub struct CapabilityViolationError
```
Dokumentation: Custom error type returned by host functions on capability violation (INV-SBX-3).
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-sandbox/src/executor/state.rs:10`
- Impl-Block (Trait `std::error::Error`) `crates/contextra-sandbox/src/executor/state.rs:16`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `capability` | `&'static str` | `crates/contextra-sandbox/src/executor/state.rs:7` |

#### `struct HostCallLimitExceededError`
- Ort: `crates/contextra-sandbox/src/executor/state.rs:20`
```rust
pub struct HostCallLimitExceededError
```
Dokumentation: Custom error type returned when host calls exceed maximum allowed limit.
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-sandbox/src/executor/state.rs:24`
- Impl-Block (Trait `std::error::Error`) `crates/contextra-sandbox/src/executor/state.rs:30`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `limit` | `u32` | `crates/contextra-sandbox/src/executor/state.rs:21` |

### Modul `merge`

#### `struct WasmMergeFunction`
- Ort: `crates/contextra-sandbox/src/merge.rs:22`
```rust
pub struct WasmMergeFunction
```
Dokumentation: Standalone, pure, deterministic WASM merge operator executor. Encapsulates a pre-validated WASM module and executes pure merge logic over binary inputs via stdin/stdout.
- Impl-Block `crates/contextra-sandbox/src/merge.rs:28`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(wasm_bytes: impl Into<Arc<[u8]>>) -> Result<Self, SandboxError>` | `crates/contextra-sandbox/src/merge.rs:35` | Creates a new `WasmMergeFunction` and validates the WASM binary once upon creation. Uses default pure merge capabilities (`MergeOperatorCapabilities::pure_with_ |
| `pub fn new_pure(wasm_bytes: impl Into<Arc<[u8]>>) -> Result<Self, SandboxError>` | `crates/contextra-sandbox/src/merge.rs:43` | Creates a new `WasmMergeFunction` with strict `pure()` capabilities (no stdout/stderr/FS/net/clock/PRNG). |
| `pub fn new_with_capabilities( wasm_bytes: impl Into<Arc<[u8]>>, caps: WasmCapabilities, ) -> Result<Self, SandboxError>` | `crates/contextra-sandbox/src/merge.rs:48` | Creates a new `WasmMergeFunction` with custom capability limits (e.g. `max_fuel` or `max_wall_clock_ms`). |
| `pub async fn merge(&self, existing: &[u8], new: &[u8]) -> Result<Vec<u8>, SandboxError>` | `crates/contextra-sandbox/src/merge.rs:76` | Merges two binary values (`existing` and `new`) deterministically using the WASM module. # Input Encoding Binary stream passed to guest stdin: `u32 LE len(exist |

### Modul `output`

#### `struct WasmOutput`
- Ort: `crates/contextra-sandbox/src/output.rs:11`
```rust
pub struct WasmOutput
```
Dokumentation: Ausgabe einer WASM-Execution. # Sicherheits-Invariante (P9) `stdout` ist `ZeroizeOnDrop` — der Speicher wird beim Drop überschrieben. Sensitive WASM-Ausgaben verlassen den Speicher nicht ohne Zeroization.
- Impl-Block `crates/contextra-sandbox/src/output.rs:21`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `stdout` | `Zeroizing<Vec<u8>>` | `crates/contextra-sandbox/src/output.rs:14` |
| `stderr` | `Vec<u8>` | `crates/contextra-sandbox/src/output.rs:16` |
| `fuel_consumed` | `u64` | `crates/contextra-sandbox/src/output.rs:18` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(stdout: Vec<u8>, stderr: Vec<u8>, fuel_consumed: u64) -> Self` | `crates/contextra-sandbox/src/output.rs:22` |  |

### Modul `wasi`

#### `struct OutputLimitExceededError`
- Ort: `crates/contextra-sandbox/src/wasi/mod.rs:23`
```rust
pub struct OutputLimitExceededError
```
Dokumentation: Error thrown when stdout or stderr exceeds `max_output_bytes`.
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-sandbox/src/wasi/mod.rs:28`
- Impl-Block (Trait `std::error::Error`) `crates/contextra-sandbox/src/wasi/mod.rs:38`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `stream` | `&'static str` | `crates/contextra-sandbox/src/wasi/mod.rs:24` |
| `limit` | `usize` | `crates/contextra-sandbox/src/wasi/mod.rs:25` |

#### `struct ProcessExitError`
- Ort: `crates/contextra-sandbox/src/wasi/mod.rs:9`
```rust
pub struct ProcessExitError
```
Dokumentation: Error thrown on WASI `proc_exit`.
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-sandbox/src/wasi/mod.rs:13`
- Impl-Block (Trait `std::error::Error`) `crates/contextra-sandbox/src/wasi/mod.rs:19`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `code` | `i32` | `crates/contextra-sandbox/src/wasi/mod.rs:10` |
