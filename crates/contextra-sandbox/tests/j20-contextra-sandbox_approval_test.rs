//! Integration tests for Human Approval Workflow (`requires_approval` production path).

use contextra_sandbox::{
    ApprovalRequest, ApprovalRisk, ApprovalStatus, ApprovalTransitionError, WasmCapabilities,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_auto_approve_low_risk_execution_workflow() -> TestResult {
    let default_caps = WasmCapabilities::default();
    let request = ApprovalRequest::new("req-low-1".to_string(), &default_caps, 1000, 5000);

    assert_eq!(request.risk, ApprovalRisk::Low);
    assert!(!request.requires_approval());
    assert_eq!(request.status, ApprovalStatus::Pending);

    // Auto-approve low-risk request via production path
    let processed_req = request.auto_approve_if_low_risk("auto_approver_daemon".to_string(), 1200)?;

    assert_eq!(
        processed_req.status,
        ApprovalStatus::Approved {
            approved_by: "auto_approver_daemon".to_string(),
            approved_at_unix_ms: 1200,
        }
    );

    Ok(())
}

#[test]
fn test_auto_approve_bypasses_elevated_and_high_risk_requests() -> TestResult {
    let now_ms = 1000;
    let ttl_ms = 5000;

    // 1. Elevated Risk: Filesystem Access
    let fs_caps = WasmCapabilities {
        allow_filesystem: true,
        ..Default::default()
    };
    let fs_req = ApprovalRequest::new("req-fs-1".to_string(), &fs_caps, now_ms, ttl_ms);
    assert_eq!(fs_req.risk, ApprovalRisk::Elevated);
    assert!(fs_req.requires_approval());

    let fs_processed = fs_req.auto_approve_if_low_risk("auto_approver".to_string(), now_ms + 100)?;
    assert_eq!(
        fs_processed.status,
        ApprovalStatus::Pending,
        "Elevated risk request must remain Pending after auto_approve_if_low_risk"
    );

    // Transition elevated request manually
    let fs_approved = fs_processed.approve("security_lead".to_string(), now_ms + 200)?;
    assert!(matches!(
        fs_approved.status,
        ApprovalStatus::Approved { ref approved_by, .. } if approved_by == "security_lead"
    ));

    // 2. High Risk: Network Access
    let net_caps = WasmCapabilities {
        allow_network: true,
        ..Default::default()
    };
    let net_req = ApprovalRequest::new("req-net-1".to_string(), &net_caps, now_ms, ttl_ms);
    assert_eq!(net_req.risk, ApprovalRisk::High);
    assert!(net_req.requires_approval());

    let net_processed = net_req.auto_approve_if_low_risk("auto_approver".to_string(), now_ms + 100)?;
    assert_eq!(
        net_processed.status,
        ApprovalStatus::Pending,
        "High risk request must remain Pending after auto_approve_if_low_risk"
    );

    // 3. High Risk: Cloud Egress Access
    let egress_caps = WasmCapabilities {
        allow_cloud_egress: true,
        ..Default::default()
    };
    let egress_req = ApprovalRequest::new("req-egress-1".to_string(), &egress_caps, now_ms, ttl_ms);
    assert_eq!(egress_req.risk, ApprovalRisk::High);
    assert!(egress_req.requires_approval());

    let egress_processed = egress_req.auto_approve_if_low_risk("auto_approver".to_string(), now_ms + 100)?;
    assert_eq!(
        egress_processed.status,
        ApprovalStatus::Pending,
        "Cloud egress request must remain Pending after auto_approve_if_low_risk"
    );

    Ok(())
}

#[test]
fn test_expired_request_returns_error_on_auto_approve() {
    let default_caps = WasmCapabilities::default();
    let req = ApprovalRequest::new("req-expired".to_string(), &default_caps, 1000, 500);

    let err = req.auto_approve_if_low_risk("auto_approver".to_string(), 1600);
    assert_eq!(
        err,
        Err(ApprovalTransitionError::Expired {
            created_at_unix_ms: 1000,
            ttl_ms: 500,
            now_unix_ms: 1600,
        })
    );
}
