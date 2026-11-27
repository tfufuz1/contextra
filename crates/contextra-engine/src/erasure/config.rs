//! Engine-side configuration for the durable erasure pipeline.

use contextra_types::{CheckpointTaintPolicy, ErasureProfile};
use std::time::Duration;

/// Scheduling and retry limits for background physical erasure.
#[derive(Debug, Clone)]
pub struct ErasureConfig {
    pub coalesce_window: Duration,
    pub sla: Duration,
    pub erase_io_share: f32,
    pub max_units_per_tick: usize,
    pub retry_backoff: (Duration, Duration),
    pub quarantine_after_attempts: u32,
    pub checkpoint_policy: CheckpointTaintPolicy,
    pub checkpoint_grace: Duration,
    pub max_pin_age: Duration,
    pub profile: ErasureProfile,
    pub recheck_interval: Duration,
}

impl Default for ErasureConfig {
    fn default() -> Self {
        Self {
            coalesce_window: Duration::from_secs(5 * 60),
            sla: Duration::from_secs(24 * 60 * 60),
            erase_io_share: 0.5,
            max_units_per_tick: 1_024,
            retry_backoff: (Duration::from_secs(1), Duration::from_secs(5 * 60)),
            quarantine_after_attempts: 10,
            checkpoint_policy: CheckpointTaintPolicy::Drop,
            checkpoint_grace: Duration::ZERO,
            max_pin_age: Duration::from_secs(7 * 24 * 60 * 60),
            profile: ErasureProfile::Standard,
            recheck_interval: Duration::from_secs(5 * 60),
        }
    }
}

/// Per-request policy. Defaults to waiting until the read gate is durably active.
#[derive(Debug, Clone)]
pub struct EraseOptions {
    pub reason: contextra_types::ErasureReason,
    pub await_level: contextra_types::AwaitLevel,
    pub policy: Option<contextra_types::ErasurePolicy>,
}

impl Default for EraseOptions {
    fn default() -> Self {
        Self {
            reason: contextra_types::ErasureReason::UserRequest,
            await_level: contextra_types::AwaitLevel::default(),
            policy: None,
        }
    }
}
