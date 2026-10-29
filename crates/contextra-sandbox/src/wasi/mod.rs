// FILE-CONTEXT
// STAND: 2026-10-30T00:00:00Z
// ZWECK: WASI preview1 host function registration (§4.18)
// INVARIANTEN: Zero unsafe code, strict bounds checking, host call limits

pub(crate) mod fd;
pub(crate) mod sys;

use crate::executor::SandboxState;
use wasmtime::Linker;

/// Error thrown on WASI `proc_exit`.
#[derive(Debug, Clone)]
pub struct ProcessExitError {
    pub code: i32,
}

impl std::fmt::Display for ProcessExitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Process exited with code {}", self.code)
    }
}

impl std::error::Error for ProcessExitError {}

/// Error thrown when stdout or stderr exceeds `max_output_bytes`.
#[derive(Debug, Clone)]
pub struct OutputLimitExceededError {
    pub stream: &'static str,
    pub limit: usize,
}

impl std::fmt::Display for OutputLimitExceededError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Output stream {} limit exceeded ({} bytes)",
            self.stream, self.limit
        )
    }
}

impl std::error::Error for OutputLimitExceededError {}

pub(crate) const ERRNO_SUCCESS: i32 = 0;
pub(crate) const ERRNO_ACCES: i32 = 2;
pub(crate) const ERRNO_BADF: i32 = 8;
pub(crate) const ERRNO_INVAL: i32 = 28;
pub(crate) const ERRNO_NOSYS: i32 = 52;
pub(crate) const ERRNO_NOTSUP: i32 = 58;

/// Registers WASI preview1 host functions into the provided Linker.
pub(crate) fn register(linker: &mut Linker<SandboxState>) -> Result<(), anyhow::Error> {
    fd::register_fd(linker)?;
    sys::register_sys(linker)?;
    Ok(())
}
