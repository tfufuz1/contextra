# Governance Rule: Asynchrones I/O & Non-Blocking Bounds (`rules/async-io.md`)

## Gilt für
- `crates/contextra-engine`
- `crates/contextra-ports`
- `crates/contextra-infer-onnx`
- Ring 0 Crate Async-Purity

## Pflichtregeln
1. **Ring 0 Async-Reinheit**: Ring-0-Crates (z.B. `contextra-types`, `contextra-ports`, `contextra-adapt`) dürfen keine direkten Abhängigkeiten zu Asynchronous Runtimes wie `tokio` aufweisen, sofern diese nicht explizit in `xtask/ring0-async-exceptions.toml` mit gültiger ADR-Referenz genehmigt sind (Quelle: `xtask/src/check_ring0_async_purity.rs`).
2. **Blocking Storage Trait Partitioning**: Speicher-Interfaces sind strikt in synchrone Lese-Pfade (`StorageRead`) und asynchrone Schreib-Pfade (`StorageWrite` via `BoxFuture`) getrennt (Quelle: `crates/contextra-ports/src/storage.rs`).
3. **No Thread Starvation in CPU Heavy Tasks**: Intensive Rechenoperationen (z.B. ONNX-Inferenz) in async Handlern müssen zwingend auf Blocking-Threadpools entladen werden via `tokio::task::spawn_blocking` (Quelle: `crates/contextra-infer-onnx/src/embedder.rs`).

## Häufige Fehler
- Mischung von synchronen Blockaden in async Task-Kontexten.
- Einschleusen von `tokio`-Traits in Ring 0 Kern-Module.
- Unbegrenzte Future-Instantiierung ohne Backpressure-Steuerung.

## Verweise
- `crates/contextra-ports/src/storage.rs`
- `xtask/src/check_ring0_async_purity.rs`
- `xtask/ring0-async-exceptions.toml`
