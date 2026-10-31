# AGENTS.md — contextra-sandbox
> Layer 6.5 | WASM Execution Boundary for MCP CodeExecution Permission | ~500 LOC

## 1. Zweck & Architekturrolle

Stellt eine eng begrenzte WASM-Ausführungsgrenze für die `CodeExecution`-Permission des MCP-Servers bereit (§4.18).
Kein WASM-Compiler im Guest — nur Ausführung vorab kompilierter `.wasm`-Binaries.

## 2. Modul-Karte

| Datei | Verantwortung |
|---|---|
| `lib.rs` | `#![forbid(unsafe_code)]`, Modul-Exporte und Crate-Dokumentation |
| `capabilities.rs` | `WasmCapabilities` Whitelist für WASM-Guest-Capabilities |
| `executor.rs` | `WasmExecutor` Execution Engine mit Store, Fuel, Limiter und Timeout |
| `output.rs` | `WasmOutput` mit `ZeroizeOnDrop` für stdout (P9 Invariante) |
| `error.rs` | `SandboxError` Fehlertypen für Sandbox-Verletzungen |

## 3. Kritische Invarianten

### Strict Safe Rust
Strict `#![forbid(unsafe_code)]` — alle WASM-Interaktionen nutzen ausschließlich safe APIs von Wasmtime.

### Fuel & Timeout Isolation (§4.18)
Jede Execution MUSS durch ein Fuel-Budget (CPU, deterministisch) UND ein Wall-Clock-Timeout (tokio::time::timeout) beschränkt sein.

### Zeroize on Drop (P9)
`WasmOutput.stdout` ist als `ZeroizeOnDrop` / `Zeroizing<Vec<u8>>` abgesichert. Sensitiver Host/Guest-Output verbleibt nach Drop nicht im RAM.

### State Isolation per Execution
Jeder `execute()`-Aufruf erzeugt eine frische `Store` + `Instance`. Keinerlei Zustandsübertrag oder Memory-Leakage zwischen Execution-Aufrufen.

## 4. Public API Quick-Reference

```rust
pub struct WasmCapabilities { ... }
pub struct WasmExecutor { ... }
impl WasmExecutor {
    pub fn new() -> Result<Self, SandboxError>;
    pub async fn execute(
        &self,
        wasm_bytes: &[u8],
        input: &[u8],
        capabilities: &WasmCapabilities,
        timeout: Duration,
    ) -> Result<WasmOutput, SandboxError>;
}
```

## 5. Anti-Patterns & LLM-Fallstricke

- **Niemals unsafe code in contextra-sandbox verwenden**.
- **Niemals Fuel-Setups oder Wall-Clock-Timeouts weglassen**.
- **Niemals Stores über execute()-Aufrufe hinweg wiederverwenden**.

## 6. Relevante ADRs & Rules

| ADR/Rule | Relevanz |
|---|---|
| §4.18 Spec | WASM Execution Boundary & Fuel Budgeting |
| P9 Security Invariant | Zeroize-On-Drop für Exec Output |
