# ADR-0XX: memfuse-sandbox — WASM Execution Boundary

**Status:** Proposed
**Datum:** 2026-09-13
**Blocker für Merge:** Dieses ADR MUSS auf "Accepted" gesetzt werden vor dem Merge in main.

## Kontext
`memfuse-mcp` benötigt eine sichere WASM-Ausführungsgrenze für die `CodeExecution`-Permission.

## Entscheidung
Neues Crate `memfuse-sandbox` (Layer 6.5) mit `wasmtime` als Backend.
`#![forbid(unsafe_code)]`. Fuel + Wall-Clock-Timeout beide aktiv.

## Konsequenzen
+ Echte Execution-Isolation für WASM-Guests
+ Keine C-FFI-Erweiterung (wasmtime ist Pure-Rust-nutzbar)
- `wasmtime` erhöht Compile-Zeit und Binary-Größe
- Layer-6.5-Sublayer muss in DAG-Check konfiguriert werden
