//! Integration tests for Human Approval Workflow & SandboxWiring (§4.18).

use std::sync::Arc;

use contextra_sandbox::{
    approval::{classify_risk, ApprovalRequest, ApprovalRisk, ApprovalStatus, ApprovalTransitionError},
    capabilities::{ModulePolicy, WasmCapabilities},
    wiring::{SandboxWiring, SandboxWiringConfig},
    Ed25519ModuleVerifier,
};
use ed25519_dalek::SigningKey;

fn sample_wat_bytes() -> Vec<u8> {
    wat::parse_str("(module (func (export \"run\")))").expect("valid WAT payload")
}

#[test]
fn test_classify_risk_rule_matrix() {
    let default_caps = WasmCapabilities::default();
    assert_eq!(classify_risk(&default_caps), ApprovalRisk::Low);

    let fs_caps = WasmCapabilities {
        allow_filesystem: true,
        ..Default::default()
    };
    assert_eq!(classify_risk(&fs_caps), ApprovalRisk::Elevated);

    let net_caps = WasmCapabilities {
        allow_network: true,
        ..Default::default()
    };
    assert_eq!(classify_risk(&net_caps), ApprovalRisk::High);

    let egress_caps = WasmCapabilities {
        allow_cloud_egress: true,
        ..Default::default()
    };
    assert_eq!(classify_risk(&egress_caps), ApprovalRisk::High);
}

#[test]
fn test_approval_request_lifecycle_transitions() -> Result<(), ApprovalTransitionError> {
    let caps = WasmCapabilities::default();
    let request = ApprovalRequest::new("req-101".to_string(), &caps, 1000, 5000);

    assert_eq!(request.status, ApprovalStatus::Pending);

    // Pending -> Approved
    let approved = request.approve("sec_admin".to_string(), 2000)?;
    assert_eq!(
        approved.status,
        ApprovalStatus::Approved {
            approved_by: "sec_admin".to_string(),
            approved_at_unix_ms: 2000,
        }
    );

    // Approved -> Approved (illegal)
    let double_approve_err = approved.clone().approve("another_admin".to_string(), 2100);
    assert!(matches!(
        double_approve_err,
        Err(ApprovalTransitionError::InvalidStateTransition {
            target_action: "approve",
            ..
        })
    ));

    // Approved -> Rejected (illegal)
    let approve_to_reject_err = approved.reject("admin".to_string(), "revert".to_string(), 2200);
    assert!(matches!(
        approve_to_reject_err,
        Err(ApprovalTransitionError::InvalidStateTransition {
            target_action: "reject",
            ..
        })
    ));

    Ok(())
}

#[test]
fn test_approval_request_expired_transition_failures() {
    let caps = WasmCapabilities::default();
    let request = ApprovalRequest::new("req-102".to_string(), &caps, 1000, 500);

    // At expiration boundary (1000 + 500 = 1500)
    assert!(request.is_expired(1500));

    let approve_expired = request.clone().approve("admin".to_string(), 1500);
    assert_eq!(
        approve_expired,
        Err(ApprovalTransitionError::Expired {
            created_at_unix_ms: 1000,
            ttl_ms: 500,
            now_unix_ms: 1500,
        })
    );

    let reject_expired = request.reject("admin".to_string(), "too late".to_string(), 1501);
    assert_eq!(
        reject_expired,
        Err(ApprovalTransitionError::Expired {
            created_at_unix_ms: 1000,
            ttl_ms: 500,
            now_unix_ms: 1501,
        })
    );
}

#[test]
fn test_approval_request_rejection_and_expiration() -> Result<(), ApprovalTransitionError> {
    let caps = WasmCapabilities::default();
    let request_a = ApprovalRequest::new("req-103".to_string(), &caps, 1000, 5000);

    let rejected = request_a.reject("auditor".to_string(), "unapproved capabilities".to_string(), 2000)?;
    assert_eq!(
        rejected.status,
        ApprovalStatus::Rejected {
            rejected_by: "auditor".to_string(),
            reason: "unapproved capabilities".to_string(),
            rejected_at_unix_ms: 2000,
        }
    );

    let request_b = ApprovalRequest::new("req-104".to_string(), &caps, 1000, 5000);
    let expired = request_b.expire(2500)?;
    assert_eq!(
        expired.status,
        ApprovalStatus::Expired {
            expired_at_unix_ms: 2500,
        }
    );

    Ok(())
}

#[test]
fn test_sandbox_wiring_build_and_workflow_execution() -> Result<(), Box<dyn std::error::Error>> {
    let signing_key = SigningKey::from_bytes(&[1u8; 32]);
    let public_key = signing_key.verifying_key().to_bytes();

    let wasm_bytes = sample_wat_bytes();
    let signed_payload = Ed25519ModuleVerifier::sign_payload(&signing_key, &wasm_bytes);

    let caps = WasmCapabilities {
        module_policy: ModulePolicy::SignedBy(vec![public_key]),
        ..Default::default()
    };

    let config = SandboxWiringConfig {
        capabilities: caps.clone(),
        verifier: None, // Will auto-instantiate Ed25519ModuleVerifier for SignedBy
    };

    let wiring = SandboxWiring::build(config)?;

    // Verify module via wiring
    let admitted = wiring.verify_module(&signed_payload)?;
    assert_eq!(admitted.bytes(), &wasm_bytes[..]);

    // Risk classification
    assert_eq!(wiring.classify_risk(), ApprovalRisk::Low);

    // Auto-approve low risk execution
    let eval = wiring.evaluate_execution(
        "req-wired-low".to_string(),
        "mcp_system".to_string(),
        1000,
        5000,
        1050,
    )?;

    assert_eq!(
        eval.status,
        ApprovalStatus::Approved {
            approved_by: "mcp_system".to_string(),
            approved_at_unix_ms: 1050,
        }
    );

    Ok(())
}

#[test]
fn test_sandbox_wiring_high_risk_requires_explicit_human_approval() -> Result<(), Box<dyn std::error::Error>> {
    let high_risk_caps = WasmCapabilities {
        allow_network: true,
        ..Default::default()
    };

    let config = SandboxWiringConfig {
        capabilities: high_risk_caps,
        verifier: Some(Arc::new(Ed25519ModuleVerifier::default())),
    };

    let wiring = SandboxWiring::build(config)?;

    assert_eq!(wiring.classify_risk(), ApprovalRisk::High);

    // Auto-approve does NOT auto-approve High risk; leaves status Pending
    let eval = wiring.evaluate_execution(
        "req-wired-high".to_string(),
        "mcp_system".to_string(),
        1000,
        5000,
        1050,
    )?;

    assert_eq!(eval.status, ApprovalStatus::Pending);
    assert!(eval.requires_approval());

    // Explicit human approval transition
    let human_approved = eval.approve("security_officer".to_string(), 1200)?;
    assert_eq!(
        human_approved.status,
        ApprovalStatus::Approved {
            approved_by: "security_officer".to_string(),
            approved_at_unix_ms: 1200,
        }
    );

    Ok(())
}
