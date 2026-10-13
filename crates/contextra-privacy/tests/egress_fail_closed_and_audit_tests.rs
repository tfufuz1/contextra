use contextra_ports::Clock;
use contextra_privacy::{
    compute_audit_trace, extract_rule_id, BlockReason, BoxFuture,
    EgressClassification, EgressClassifierTrace, EgressGuard,
    EgressVault, TextSearchEngine, TextSearchResult, DEFAULT_EGRESS_GUARD_MIN_BYTES,
    DEFAULT_EGRESS_GUARD_THRESHOLD,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

struct TestClock {
    nanos: AtomicU64,
}

impl TestClock {
    fn new(nanos: u64) -> Self {
        Self {
            nanos: AtomicU64::new(nanos),
        }
    }
}

impl Clock for TestClock {
    fn now_unix_nanos(&self) -> u64 {
        self.nanos.load(Ordering::Relaxed)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.nanos.load(Ordering::Relaxed)
    }
}

struct TimingOutSearchEngine {
    delay: Duration,
}

impl TextSearchEngine for TimingOutSearchEngine {
    fn search_text<'a>(
        &'a self,
        _text: &'a str,
        _limit: usize,
    ) -> BoxFuture<'a, Result<Vec<TextSearchResult>, String>> {
        let delay = self.delay;
        Box::pin(async move {
            tokio::time::sleep(delay).await;
            Ok(vec![])
        })
    }
}

struct ErrorSearchEngine {
    message: String,
}

impl TextSearchEngine for ErrorSearchEngine {
    fn search_text<'a>(
        &'a self,
        _text: &'a str,
        _limit: usize,
    ) -> BoxFuture<'a, Result<Vec<TextSearchResult>, String>> {
        let msg = self.message.clone();
        Box::pin(async move { Err(msg) })
    }
}

#[tokio::test]
async fn test_egress_guard_fail_closed_timeout_and_error_with_inv_egress_audit_1() {
    let clock = TestClock::new(1_800_000_000_000_000_000);
    let sensitive_payload =
        "CONFIDENTIAL_CUSTOMER_RECORD_EXFILTRATION_TEST_PAYLOAD_STRING_EXCEEDING_MIN_BYTES. "
            .repeat(3);
    assert!(sensitive_payload.len() >= DEFAULT_EGRESS_GUARD_MIN_BYTES);

    // 1. Timeout -> Fail-Closed (Block)
    let timeout_engine = Arc::new(TimingOutSearchEngine {
        delay: Duration::from_millis(100),
    });
    let guard_timeout = EgressGuard::new(
        timeout_engine,
        DEFAULT_EGRESS_GUARD_THRESHOLD,
        DEFAULT_EGRESS_GUARD_MIN_BYTES,
    )
    .with_timeout(Duration::from_millis(5));

    let (class_timeout, trace_timeout) = guard_timeout
        .classify_with_trace(&sensitive_payload, &clock)
        .await;
    assert_eq!(
        class_timeout,
        EgressClassification::Block(BlockReason::ClassificationTimeout)
    );

    let rule_timeout = extract_rule_id(&class_timeout);
    assert_eq!(rule_timeout, "CLASSIFICATION_TIMEOUT");

    let expected_trace_timeout =
        compute_audit_trace(&sensitive_payload, &rule_timeout, 1_800_000_000_000_000_000);
    assert_eq!(trace_timeout, expected_trace_timeout);

    // 2. Index Error -> Fail-Closed (Block)
    let error_engine = Arc::new(ErrorSearchEngine {
        message: "Vector index offline or connection reset".to_string(),
    });
    let guard_error = EgressGuard::new(
        error_engine,
        DEFAULT_EGRESS_GUARD_THRESHOLD,
        DEFAULT_EGRESS_GUARD_MIN_BYTES,
    );

    let (class_error, trace_error) = guard_error
        .classify_with_trace(&sensitive_payload, &clock)
        .await;
    assert_eq!(
        class_error,
        EgressClassification::Block(BlockReason::InternalError(
            "egress guard index unavailable — fail-closed".to_string()
        ))
    );

    let rule_error = extract_rule_id(&class_error);
    let expected_trace_error =
        compute_audit_trace(&sensitive_payload, &rule_error, 1_800_000_000_000_000_000);
    assert_eq!(trace_error, expected_trace_error);

    // Verify audit trace contains NO raw payload bytes (INV-EGRESS-AUDIT-1)
    let trace_str = format!("{:?}", trace_timeout);
    assert!(!trace_str.contains("CONFIDENTIAL_CUSTOMER_RECORD"));
}

#[tokio::test]
async fn test_egress_vault_fail_closed_oversized_payload_audit_trace() {
    let clock = TestClock::new(1_850_000_000_000_000_000);
    let vault = EgressVault::try_default().expect("valid egress vault default");

    let oversized_payload = "X".repeat(contextra_privacy::MAX_CLASSIFY_PAYLOAD_BYTES + 1024);

    let (classification, trace) = vault.classify_with_trace(&oversized_payload, &clock).await;
    assert!(matches!(
        classification,
        EgressClassification::Block(BlockReason::EgressPolicyDenied(_))
    ));

    let rule_id = extract_rule_id(&classification);
    let expected_trace =
        compute_audit_trace(&oversized_payload, &rule_id, 1_850_000_000_000_000_000);
    assert_eq!(trace, expected_trace);
}
