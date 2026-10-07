#![forbid(unsafe_code)]
//! Contextra WASM Execution Boundary (§4.18).
//!
//! Stellt eine eng begrenzte WASM-Ausführungsgrenze für die `CodeExecution`-Permission bereit.
//! Kein WASM-Compiler — nur Ausführung vorab kompilierter `.wasm`-Binaries.
//!
//! # Sicherheitsgrundsätze
//! - `#![forbid(unsafe_code)]`: Alle Interaktionen via wasmtime's sichere Rust-API
//! - Fuel + Wall-Clock-Timeout: beide aktiv (§4.18)
//! - `WasmOutput.stdout` ist `ZeroizeOnDrop` (P9)
//! - Kein Dateisystem-/Netzwerkzugriff per Default

pub mod admission;
pub mod approval;
pub mod capabilities;
pub mod error;
pub mod executor;
pub mod merge;
pub mod output;
pub mod wasi;

pub use admission::{AdmittedModule, ModuleVerifier};
pub use approval::{
    classify_risk, ApprovalRequest, ApprovalRisk, ApprovalStatus, ApprovalTransitionError,
};
pub use capabilities::{MergeOperatorCapabilities, WasmCapabilities};
pub use error::{AdmissionError, SandboxError};
pub use executor::WasmExecutor;
pub use merge::WasmMergeFunction;
pub use output::WasmOutput;
