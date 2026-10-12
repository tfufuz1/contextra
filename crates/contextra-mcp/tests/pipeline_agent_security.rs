#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mcp::sandbox::ToolCategory;
use contextra_mcp::server_dispatch::{
    cancellation::{CancelObserver, ConcurrencyGovernor},
    channel_kind::ChannelKind,
    confirm_handler::handle_confirm,
    forget_guard::{ForgetGuard, ForgetRequest, ForgetScope},
    recall_truncator::RecallTruncator,
    tool_hints::annotate_tool_schema,
    trust_frame::{TrustFrame, TrustLevel},
};
use contextra_ports::SystemClock;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_forget_guard_thresholds_and_token_expiration() {
    let clock = Arc::new(SystemClock::new());
    let guard = ForgetGuard::new(10, clock); // Threshold = 10

    // 1. Single ID forget with dry_run = false -> Direct
    let req_direct = ForgetRequest {
        scope: ForgetScope::SingleDoc("doc_1".into()),
        collection: "default".into(),
        matched_count: 1,
        dry_run: false,
    };
    let decision1 = guard.evaluate("principal_a", req_direct);
    assert!(matches!(
        decision1,
        contextra_mcp::server_dispatch::forget_guard::ForgetDecision::Direct
    ));

    // 2. Forget with count 11 (> threshold 10) -> ConfirmationRequired
    let req_confirm = ForgetRequest {
        scope: ForgetScope::Selector("category == 'temp'".into()),
        collection: "default".into(),
        matched_count: 11,
        dry_run: false,
    };
    let decision2 = guard.evaluate("principal_a", req_confirm);
    let token = match decision2 {
        contextra_mcp::server_dispatch::forget_guard::ForgetDecision::ConfirmationRequired {
            token,
            preview,
        } => {
            assert_eq!(preview.matched_count, 11);
            token
        }
        _ => panic!("Expected ConfirmationRequired for 11 docs"),
    };

    // 3. Confirming via wrong channel (HostStdio) -> Denied
    let confirm_res_stdio = handle_confirm(
        &guard,
        "principal_a",
        &token,
        ChannelKind::HostStdio,
        |_scope, _coll| Ok(11),
    );
    assert!(confirm_res_stdio.is_err());
    assert_eq!(
        confirm_res_stdio.unwrap_err().code,
        contextra_mcp::server_dispatch::error_codes::McpErrorCode::Denied
    );

    // 4. Confirming via LocalIpc -> Success
    let confirm_res_ipc = handle_confirm(
        &guard,
        "principal_a",
        &token,
        ChannelKind::LocalIpc,
        |_scope, _coll| Ok(11),
    );
    assert!(confirm_res_ipc.is_ok());
    assert_eq!(confirm_res_ipc.unwrap()["deleted"], 11);

    // 5. Token is single-use: Re-using token -> Denied
    let confirm_res_reuse = handle_confirm(
        &guard,
        "principal_a",
        &token,
        ChannelKind::LocalIpc,
        |_scope, _coll| Ok(11),
    );
    assert!(confirm_res_reuse.is_err());
}

#[tokio::test]
async fn test_cancellation_pre_wal_checkpoint() {
    let cancel = CancelObserver::new();
    assert!(!cancel.is_cancelled());

    // Before WAL commit: trigger cancellation
    cancel.trigger_cancellation();
    assert!(cancel.is_cancelled());

    let wal_check = cancel.check_pre_wal_commit();
    assert!(wal_check.is_err());
    assert_eq!(
        wal_check.unwrap_err().code,
        contextra_mcp::server_dispatch::error_codes::McpErrorCode::Denied
    );
}

#[tokio::test]
async fn test_concurrency_governor_max_in_flight() {
    let governor = ConcurrencyGovernor::new(2); // limit 2

    let _g1 = governor.acquire_permit("user_1").unwrap();
    let _g2 = governor.acquire_permit("user_1").unwrap();

    // 3rd request for same principal exceeds limit -> Busy
    let err3 = governor.acquire_permit("user_1").unwrap_err();
    assert_eq!(
        err3.code,
        contextra_mcp::server_dispatch::error_codes::McpErrorCode::Busy
    );
    assert!(err3.code.is_retryable());

    // Different principal is still allowed
    let _g_other = governor.acquire_permit("user_2").unwrap();
}

#[tokio::test]
async fn test_recall_truncation_marker() {
    let truncator = RecallTruncator::new(10); // 10 tokens ~ 40 chars
    let long_text = "Word ".repeat(100); // 500 chars -> ~125 tokens

    let (truncated_str, marker) = truncator.truncate(&long_text);
    assert!(marker.truncated);
    assert!(marker.omitted > 0);
    assert!(truncated_str.ends_with("[TRUNCATED: max_response_tokens reached]"));
}

#[tokio::test]
async fn test_trust_frame_escalation_rejection() {
    let external_caller_level = TrustLevel::ExternalSystem;

    // Caller attempts to claim higher trust "User" -> rejected
    let res = TrustFrame::<()>::validate_write_trust(external_caller_level, TrustLevel::User);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(
        err.code,
        contextra_mcp::server_dispatch::error_codes::McpErrorCode::Denied
    );

    // Caller writes at or below caller trust level -> allowed
    assert!(TrustFrame::<()>::validate_write_trust(TrustLevel::User, TrustLevel::Agent).is_ok());
}

#[test]
fn test_tool_annotations_from_category() {
    let schema = json!({
        "type": "object",
        "properties": {}
    });

    let read_annotated = annotate_tool_schema(schema.clone(), ToolCategory::DatabaseRead);
    assert_eq!(read_annotated["readOnlyHint"], true);
    assert_eq!(read_annotated["destructiveHint"], false);

    let dest_annotated = annotate_tool_schema(schema, ToolCategory::DatabaseWrite);
    assert_eq!(dest_annotated["readOnlyHint"], false);
    assert_eq!(dest_annotated["destructiveHint"], true);
}
