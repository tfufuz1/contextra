// FILE-CONTEXT
// STAND: 2026-09-25T00:00:00Z
// ZWECK: Human Approval Data Model & Pure State Machine for WASM Guest Execution (§4.18)
// INVARIANTEN: No internal timestamps (all time injected), zero panic, pure state machine

//! Human Approval Workflow and State Machine for WASM Execution Boundary.
//!
//! Defined data model and deterministic state machine for human approval decisions
//! regarding WASM guest module executions with expanded capabilities (`WasmCapabilities`).

use crate::capabilities::WasmCapabilities;
use thiserror::Error;

/// Risk level classification for a requested WASM execution configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApprovalRisk {
    /// Low risk: Execution within standard safe defaults (no filesystem/network/cloud egress).
    Low,
    /// Elevated risk: File system access requested without network or egress.
    Elevated,
    /// High risk: Network connectivity or cloud egress capabilities requested.
    High,
}

/// Derives the execution risk level deterministically from `WasmCapabilities`.
///
/// # Rule Matrix
///
/// | Capability Condition | Derived Risk Level | Rationale |
/// |---|---|---|
/// | `allow_network == true` \|\| `allow_cloud_egress == true` | `ApprovalRisk::High` | Egress/Network allows potential data exfiltration or external side-effects. |
/// | `allow_filesystem == true` (without network/egress) | `ApprovalRisk::Elevated` | Filesystem access permits persistent read/write operations locally. |
/// | Default safe capabilities or non-elevated overrides | `ApprovalRisk::Low` | Sandboxed in-memory execution capped by fuel/memory/timeout constraints. |
pub fn classify_risk(caps: &WasmCapabilities) -> ApprovalRisk {
    if caps.allow_network || caps.allow_cloud_egress {
        ApprovalRisk::High
    } else if caps.allow_filesystem {
        ApprovalRisk::Elevated
    } else {
        ApprovalRisk::Low
    }
}

/// Status of a human approval decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalStatus {
    /// Pending decision.
    Pending,
    /// Approved by an authorized entity.
    Approved {
        approved_by: String,
        approved_at_unix_ms: u64,
    },
    /// Rejected with an explicit reason.
    Rejected {
        rejected_by: String,
        reason: String,
        rejected_at_unix_ms: u64,
    },
    /// Request expired before a decision was reached.
    Expired { expired_at_unix_ms: u64 },
}

/// Errors occurring during approval state transitions.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ApprovalTransitionError {
    /// Transition rejected because the approval request has expired.
    #[error(
        "Approval request expired (created_at: {created_at_unix_ms}ms, ttl: {ttl_ms}ms, evaluated_at: {now_unix_ms}ms)"
    )]
    Expired {
        created_at_unix_ms: u64,
        ttl_ms: u64,
        now_unix_ms: u64,
    },

    /// Invalid state transition from current status to the requested action.
    #[error("Invalid state transition from {current_status:?} to target action '{target_action}'")]
    InvalidStateTransition {
        current_status: ApprovalStatus,
        target_action: &'static str,
    },
}

/// Human approval request state machine for a WASM execution run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalRequest {
    /// Unique identifier for the approval request.
    pub request_id: String,
    /// Classification risk level.
    pub risk: ApprovalRisk,
    /// Human-readable summary of capability fields that deviate from defaults.
    pub requested_capabilities_summary: String,
    /// Current approval status.
    pub status: ApprovalStatus,
    /// Unix timestamp in milliseconds when the request was created.
    pub created_at_unix_ms: u64,
    /// Time-to-live duration in milliseconds.
    pub ttl_ms: u64,
}

impl ApprovalRequest {
    /// Creates a new approval request in `Pending` status.
    ///
    /// Computes `risk` via `classify_risk` and generates `requested_capabilities_summary`
    /// from fields differing from `WasmCapabilities::default()`.
    pub fn new(
        request_id: String,
        caps: &WasmCapabilities,
        created_at_unix_ms: u64,
        ttl_ms: u64,
    ) -> Self {
        let risk = classify_risk(caps);
        let summary = summarize_capabilities_diff(caps);

        Self {
            request_id,
            risk,
            requested_capabilities_summary: summary,
            status: ApprovalStatus::Pending,
            created_at_unix_ms,
            ttl_ms,
        }
    }

    /// Checks if the request is expired relative to `now_unix_ms`.
    ///
    /// Uses `saturating_add` to prevent overflow panic on boundary values.
    pub fn is_expired(&self, now_unix_ms: u64) -> bool {
        let expiration_time = self.created_at_unix_ms.saturating_add(self.ttl_ms);
        now_unix_ms >= expiration_time
    }

    /// Transitions request from `Pending` to `Approved`.
    ///
    /// Fails with `ApprovalTransitionError` if not in `Pending` state or if expired at `now_unix_ms`.
    pub fn approve(
        mut self,
        approved_by: String,
        now_unix_ms: u64,
    ) -> Result<Self, ApprovalTransitionError> {
        if self.status != ApprovalStatus::Pending {
            return Err(ApprovalTransitionError::InvalidStateTransition {
                current_status: self.status,
                target_action: "approve",
            });
        }

        if self.is_expired(now_unix_ms) {
            return Err(ApprovalTransitionError::Expired {
                created_at_unix_ms: self.created_at_unix_ms,
                ttl_ms: self.ttl_ms,
                now_unix_ms,
            });
        }

        self.status = ApprovalStatus::Approved {
            approved_by,
            approved_at_unix_ms: now_unix_ms,
        };
        Ok(self)
    }

    /// Transitions request from `Pending` to `Rejected`.
    ///
    /// Fails with `ApprovalTransitionError` if not in `Pending` state or if expired at `now_unix_ms`.
    pub fn reject(
        mut self,
        rejected_by: String,
        reason: String,
        now_unix_ms: u64,
    ) -> Result<Self, ApprovalTransitionError> {
        if self.status != ApprovalStatus::Pending {
            return Err(ApprovalTransitionError::InvalidStateTransition {
                current_status: self.status,
                target_action: "reject",
            });
        }

        if self.is_expired(now_unix_ms) {
            return Err(ApprovalTransitionError::Expired {
                created_at_unix_ms: self.created_at_unix_ms,
                ttl_ms: self.ttl_ms,
                now_unix_ms,
            });
        }

        self.status = ApprovalStatus::Rejected {
            rejected_by,
            reason,
            rejected_at_unix_ms: now_unix_ms,
        };
        Ok(self)
    }

    /// Explicitly transitions request from `Pending` to `Expired`.
    pub fn expire(mut self, now_unix_ms: u64) -> Result<Self, ApprovalTransitionError> {
        if self.status != ApprovalStatus::Pending {
            return Err(ApprovalTransitionError::InvalidStateTransition {
                current_status: self.status,
                target_action: "expire",
            });
        }

        self.status = ApprovalStatus::Expired {
            expired_at_unix_ms: now_unix_ms,
        };
        Ok(self)
    }

    /// Determines whether this execution requires human approval before proceeding.
    ///
    /// # Product Rationale
    /// Low-risk executions (default sandboxed capabilities without network, cloud egress,
    /// or filesystem privileges) are allowed to run automatically (`false`) to ensure low-latency
    /// operation. Higher risk levels (`Elevated` or `High`) require explicit human approval (`true`).
    pub fn requires_approval(&self) -> bool {
        matches!(self.risk, ApprovalRisk::Elevated | ApprovalRisk::High)
    }
}

/// Generates a human-readable summary string of capability fields that differ from `WasmCapabilities::default()`.
fn summarize_capabilities_diff(caps: &WasmCapabilities) -> String {
    let default_caps = WasmCapabilities::default();
    let mut diffs = Vec::new();

    if caps.allow_stdout != default_caps.allow_stdout {
        diffs.push(format!("allow_stdout: {}", caps.allow_stdout));
    }
    if caps.allow_stderr != default_caps.allow_stderr {
        diffs.push(format!("allow_stderr: {}", caps.allow_stderr));
    }
    if caps.max_memory_pages != default_caps.max_memory_pages {
        diffs.push(format!("max_memory_pages: {}", caps.max_memory_pages));
    }
    if caps.max_fuel != default_caps.max_fuel {
        diffs.push(format!("max_fuel: {}", caps.max_fuel));
    }
    if caps.allow_filesystem != default_caps.allow_filesystem {
        diffs.push(format!("allow_filesystem: {}", caps.allow_filesystem));
    }
    if caps.allow_network != default_caps.allow_network {
        diffs.push(format!("allow_network: {}", caps.allow_network));
    }
    if caps.allow_clock != default_caps.allow_clock {
        diffs.push(format!("allow_clock: {}", caps.allow_clock));
    }
    if caps.allow_cloud_egress != default_caps.allow_cloud_egress {
        diffs.push(format!("allow_cloud_egress: {}", caps.allow_cloud_egress));
    }
    if caps.max_wall_clock_ms != default_caps.max_wall_clock_ms {
        diffs.push(format!("max_wall_clock_ms: {}", caps.max_wall_clock_ms));
    }
    if caps.max_module_size_bytes != default_caps.max_module_size_bytes {
        diffs.push(format!(
            "max_module_size_bytes: {}",
            caps.max_module_size_bytes
        ));
    }
    if caps.max_table_entries != default_caps.max_table_entries {
        diffs.push(format!("max_table_entries: {}", caps.max_table_entries));
    }
    if caps.max_output_bytes != default_caps.max_output_bytes {
        diffs.push(format!("max_output_bytes: {}", caps.max_output_bytes));
    }
    if caps.max_stdin_bytes != default_caps.max_stdin_bytes {
        diffs.push(format!("max_stdin_bytes: {}", caps.max_stdin_bytes));
    }
    if caps.random_seed != default_caps.random_seed {
        diffs.push(format!("random_seed: {:?}", caps.random_seed));
    }

    if diffs.is_empty() {
        "default capabilities".to_string()
    } else {
        diffs.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_risk_levels() {
        let default_caps = WasmCapabilities::default();
        assert_eq!(classify_risk(&default_caps), ApprovalRisk::Low);

        let mut fs_caps = WasmCapabilities::default();
        fs_caps.allow_filesystem = true;
        assert_eq!(classify_risk(&fs_caps), ApprovalRisk::Elevated);

        let mut net_caps = WasmCapabilities::default();
        net_caps.allow_network = true;
        assert_eq!(classify_risk(&net_caps), ApprovalRisk::High);

        let mut egress_caps = WasmCapabilities::default();
        egress_caps.allow_cloud_egress = true;
        assert_eq!(classify_risk(&egress_caps), ApprovalRisk::High);

        let mut all_caps = WasmCapabilities::default();
        all_caps.allow_filesystem = true;
        all_caps.allow_network = true;
        all_caps.allow_cloud_egress = true;
        assert_eq!(classify_risk(&all_caps), ApprovalRisk::High);
    }

    #[test]
    fn test_approve_transition_success_and_double_approve_fails() {
        let caps = WasmCapabilities::default();
        let request = ApprovalRequest::new("req-1".to_string(), &caps, 1000, 5000);

        assert_eq!(request.status, ApprovalStatus::Pending);

        let approved_req = request
            .approve("admin_user".to_string(), 2000)
            .expect("Approval should succeed");

        assert_eq!(
            approved_req.status,
            ApprovalStatus::Approved {
                approved_by: "admin_user".to_string(),
                approved_at_unix_ms: 2000,
            }
        );

        let second_approval_err = approved_req
            .approve("second_admin".to_string(), 2500)
            .expect_err("Double approval must fail");

        match second_approval_err {
            ApprovalTransitionError::InvalidStateTransition {
                current_status,
                target_action,
            } => {
                assert_eq!(target_action, "approve");
                assert!(matches!(current_status, ApprovalStatus::Approved { .. }));
            }
            _ => panic!("Expected InvalidStateTransition error"),
        }
    }

    #[test]
    fn test_expired_request_cannot_be_approved_or_rejected() {
        let caps = WasmCapabilities::default();
        let request = ApprovalRequest::new("req-2".to_string(), &caps, 1000, 500);

        // Before TTL expiration
        assert!(!request.is_expired(1499));
        // At TTL expiration boundary
        assert!(request.is_expired(1500));
        // After TTL expiration
        assert!(request.is_expired(1600));

        // Attempting to approve after expiration fails
        let err_approve = request
            .clone()
            .approve("admin".to_string(), 1500)
            .expect_err("Approval on expired request must fail");

        assert_eq!(
            err_approve,
            ApprovalTransitionError::Expired {
                created_at_unix_ms: 1000,
                ttl_ms: 500,
                now_unix_ms: 1500,
            }
        );

        // Attempting to reject after expiration fails
        let err_reject = request
            .reject("admin".to_string(), "too late".to_string(), 1600)
            .expect_err("Rejection on expired request must fail");

        assert_eq!(
            err_reject,
            ApprovalTransitionError::Expired {
                created_at_unix_ms: 1000,
                ttl_ms: 500,
                now_unix_ms: 1600,
            }
        );
    }

    #[test]
    fn test_reject_transition_success() {
        let caps = WasmCapabilities::default();
        let request = ApprovalRequest::new("req-3".to_string(), &caps, 1000, 5000);

        let rejected_req = request
            .reject(
                "security_officer".to_string(),
                "untrusted binary".to_string(),
                2000,
            )
            .expect("Rejection should succeed");

        assert_eq!(
            rejected_req.status,
            ApprovalStatus::Rejected {
                rejected_by: "security_officer".to_string(),
                reason: "untrusted binary".to_string(),
                rejected_at_unix_ms: 2000,
            }
        );
    }

    #[test]
    fn test_requires_approval_logic() {
        let default_caps = WasmCapabilities::default();
        let low_req = ApprovalRequest::new("req-low".to_string(), &default_caps, 1000, 5000);
        assert!(
            !low_req.requires_approval(),
            "Default capabilities should not require approval"
        );

        let mut net_caps = WasmCapabilities::default();
        net_caps.allow_network = true;
        let high_net_req = ApprovalRequest::new("req-high-net".to_string(), &net_caps, 1000, 5000);
        assert!(
            high_net_req.requires_approval(),
            "Network access requires approval"
        );

        let mut egress_caps = WasmCapabilities::default();
        egress_caps.allow_cloud_egress = true;
        let high_egress_req =
            ApprovalRequest::new("req-high-egress".to_string(), &egress_caps, 1000, 5000);
        assert!(
            high_egress_req.requires_approval(),
            "Cloud egress requires approval"
        );

        let mut fs_caps = WasmCapabilities::default();
        fs_caps.allow_filesystem = true;
        let elevated_fs_req =
            ApprovalRequest::new("req-elevated-fs".to_string(), &fs_caps, 1000, 5000);
        assert!(
            elevated_fs_req.requires_approval(),
            "Filesystem access requires approval"
        );
    }

    #[test]
    fn test_summary_formatting() {
        let default_caps = WasmCapabilities::default();
        let req_default =
            ApprovalRequest::new("req-default".to_string(), &default_caps, 1000, 5000);
        assert_eq!(
            req_default.requested_capabilities_summary,
            "default capabilities"
        );

        let mut custom_caps = WasmCapabilities::default();
        custom_caps.allow_network = true;
        custom_caps.max_memory_pages = 64;
        let req_custom = ApprovalRequest::new("req-custom".to_string(), &custom_caps, 1000, 5000);
        assert_eq!(
            req_custom.requested_capabilities_summary,
            "max_memory_pages: 64, allow_network: true"
        );
    }

    #[test]
    fn test_saturating_add_overflow_handling() {
        let default_caps = WasmCapabilities::default();
        let overflow_req = ApprovalRequest::new(
            "req-overflow".to_string(),
            &default_caps,
            u64::MAX - 10,
            100,
        );

        assert!(overflow_req.is_expired(u64::MAX));
        assert!(!overflow_req.is_expired(u64::MAX - 11));
    }
}
