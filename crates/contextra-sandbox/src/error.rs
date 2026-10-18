//! WASM-Sandbox-Fehlertypen (§4.18).

use thiserror::Error;

/// Fehlertypen für Module Admission Control.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdmissionError {
    #[error("Unsigned WASM module: missing signature header or metadata")]
    UnsignedModule,

    #[error("Invalid WASM module signature: {0}")]
    InvalidSignature(String),

    #[error("Module digest mismatch: expected {expected}, got {actual}")]
    DigestMismatch { expected: String, actual: String },

    #[error("Invalid module payload or format: {0}")]
    InvalidModule(String),

    #[error("Module verification failed: {0}")]
    VerificationFailed(String),
}

/// Fehlertypen der WASM-Ausführungsgrenze.
#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("WASM-Execution timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("WASM module exceeded memory limit ({pages} pages, max {max_pages})")]
    MemoryExceeded { pages: u32, max_pages: u32 },

    #[error("WASM fuel budget exhausted ({consumed} fuel units consumed)")]
    FuelExhausted { consumed: u64 },

    #[error("WASM capability violation: {capability}")]
    CapabilityViolation { capability: String },

    #[error("WASM trap: {0}")]
    WasmTrap(String),

    #[error("Invalid WASM module: {0}")]
    InvalidModule(String),

    #[error("Output stream {stream} limit exceeded ({limit} bytes)")]
    OutputLimitExceeded { stream: &'static str, limit: usize },

    #[error("Input too large ({len} bytes, max {limit} bytes)")]
    InputTooLarge { len: usize, limit: usize },

    #[error("Process exited with code {code}")]
    ProcessExit { code: i32 },

    #[error("Module admission error: {0}")]
    Admission(#[from] AdmissionError),

    #[error("WASM runtime error: {0}")]
    Runtime(String),
}
