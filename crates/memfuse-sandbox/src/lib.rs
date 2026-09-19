#![forbid(unsafe_code)]
//! MemFuse WASM Execution Boundary (§4.18).
//!
//! Stellt eine eng begrenzte WASM-Ausführungsgrenze für die `CodeExecution`-Permission bereit.
//! Kein WASM-Compiler — nur Ausführung vorab kompilierter `.wasm`-Binaries.
//!
//! # Sicherheitsgrundsätze
//! - `#![forbid(unsafe_code)]`: Alle Interaktionen via wasmtime's sichere Rust-API
//! - Fuel + Wall-Clock-Timeout: beide aktiv (§4.18)
//! - `WasmOutput.stdout` ist `ZeroizeOnDrop` (P9)
//! - Kein Dateisystem-/Netzwerkzugriff per Default

pub mod capabilities;
pub mod error;
pub mod executor;
pub mod output;

pub use capabilities::WasmCapabilities;
pub use error::SandboxError;
pub use executor::{SandboxedTool, WasmExecutor};
pub use output::WasmOutput;
