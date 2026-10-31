// FILE-CONTEXT
// STAND: 2026-10-30T00:00:00Z
// ZWECK: Sandbox execution state and resource limits enforcement
// INVARIANTEN: #![forbid(unsafe_code)], zero panics, strict memory and table limits

use std::sync::{Arc, Mutex};
use tokio::time::Instant;

/// Custom error type returned by host functions on capability violation (INV-SBX-3).
#[derive(Debug, Clone)]
pub struct CapabilityViolationError {
    pub capability: &'static str,
}

impl std::fmt::Display for CapabilityViolationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WASM capability violation: {}", self.capability)
    }
}

impl std::error::Error for CapabilityViolationError {}

/// Custom error type returned when host calls exceed maximum allowed limit.
#[derive(Debug, Clone)]
pub struct HostCallLimitExceededError {
    pub limit: u32,
}

impl std::fmt::Display for HostCallLimitExceededError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Host call limit exceeded ({})", self.limit)
    }
}

impl std::error::Error for HostCallLimitExceededError {}

pub(crate) struct SandboxState {
    pub(crate) max_pages: u32,
    pub(crate) max_table_entries: u32,
    pub(crate) allow_cloud_egress: bool,
    pub(crate) allow_stdout: bool,
    pub(crate) allow_stderr: bool,
    pub(crate) allow_clock: bool,
    pub(crate) max_output_bytes: usize,
    pub(crate) host_calls_count: u32,
    pub(crate) max_host_calls: u32,
    pub(crate) stdin_input: Vec<u8>,
    pub(crate) stdin_pos: usize,
    pub(crate) stdout_buf: Arc<Mutex<Vec<u8>>>,
    pub(crate) stderr_buf: Arc<Mutex<Vec<u8>>>,
    pub(crate) start_instant: Instant,
    pub(crate) rng_state: Option<u64>,
}

impl SandboxState {
    pub(crate) fn check_and_inc_host_call(&mut self) -> Result<(), HostCallLimitExceededError> {
        if self.host_calls_count >= self.max_host_calls {
            Err(HostCallLimitExceededError {
                limit: self.max_host_calls,
            })
        } else {
            self.host_calls_count += 1;
            Ok(())
        }
    }
}

impl wasmtime::ResourceLimiter for SandboxState {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _max: Option<usize>,
    ) -> anyhow::Result<bool> {
        // INV-SBX-2: Rounding up with div_ceil ensures exact page limits without page under-counting
        let pages = desired.div_ceil(65536);
        let desired_pages = match u32::try_from(pages) {
            Ok(p) => p,
            Err(_) => return Ok(false),
        };
        Ok(desired_pages <= self.max_pages)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _max: Option<usize>,
    ) -> anyhow::Result<bool> {
        // INV-SBX-2: Enforce strict ceiling on table elements
        let desired_entries = match u32::try_from(desired) {
            Ok(d) => d,
            Err(_) => return Ok(false),
        };
        Ok(desired_entries <= self.max_table_entries)
    }
}
