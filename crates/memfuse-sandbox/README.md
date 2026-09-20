# memfuse-sandbox

`memfuse-sandbox` stellt eine isolierte WASM-Ausführungsgrenze bereit (Ring 2).

## Zweck

Führt vorab kompilierte WASM-Binaries für Agenten-Tools aus. Erzwingt strikte Fuel- (Rechenschritt) und Wall-Clock-Timeout-Grenzen.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 2 (Blatt-Crate / WASM Execution Boundary)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

- **Capabilities & Limits:** `WasmCapabilities`
- **Executor:** `WasmExecutor`
- **Output:** `WasmOutput`
- **Errors:** `SandboxError`

## Architektur & Verweise

Details zur WASM-Isolation (P23) finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) und `README.md` §10.2.
