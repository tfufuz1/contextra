use contextra_ports::Clock;
use contextra_privacy::{
    compute_audit_trace, extract_rule_id, BlockReason, EgressClassification, EgressClassifier,
    EgressClassifierTrace, EgressVault, MAX_CLASSIFY_PAYLOAD_BYTES,
};
use std::sync::atomic::{AtomicU64, Ordering};

struct FixedClock {
    nanos: AtomicU64,
}

impl FixedClock {
    fn new(nanos: u64) -> Self {
        Self {
            nanos: AtomicU64::new(nanos),
        }
    }
}

impl Clock for FixedClock {
    fn now_unix_nanos(&self) -> u64 {
        self.nanos.load(Ordering::Relaxed)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.nanos.load(Ordering::Relaxed)
    }
}

#[tokio::test]
async fn test_audit_trace_determinism() -> Result<(), Box<dyn std::error::Error>> {
    let patterns = vec![r"sk-[a-zA-Z0-9]{32}".to_string()];
    let vault = EgressVault::new(patterns)?;
    let clock = FixedClock::new(1_700_000_000_000_000_000);

    let payload = "This contains secret key sk-01234567890123456789012345678901 inside";

    let (class1, trace1) = vault.classify_with_trace(payload, &clock).await;
    let (class2, trace2) = vault.classify_with_trace(payload, &clock).await;

    assert_eq!(class1, class2);
    assert_eq!(trace1, trace2);

    let expected_rule_id = extract_rule_id(&class1);
    assert_eq!(expected_rule_id, "R-001");

    let manual_trace = compute_audit_trace(payload, &expected_rule_id, 1_700_000_000_000_000_000);
    assert_eq!(trace1, manual_trace);
    Ok(())
}

#[tokio::test]
async fn test_audit_trace_negatives() -> Result<(), Box<dyn std::error::Error>> {
    let patterns = vec![
        r"sk-[a-zA-Z0-9]{32}".to_string(),
        r"AKIA[0-9A-Z]{16}".to_string(),
    ];
    let vault = EgressVault::new(patterns)?;
    let clock1 = FixedClock::new(100);
    let clock2 = FixedClock::new(200);

    let payload_sk = "Secret sk-01234567890123456789012345678901";
    let payload_aws = "Secret AKIA1234567890123456";

    let (class_sk, trace_sk_t1) = vault.classify_with_trace(payload_sk, &clock1).await;
    let (_, trace_sk_t2) = vault.classify_with_trace(payload_sk, &clock2).await;
    assert_ne!(trace_sk_t1, trace_sk_t2, "Different timestamps must yield different traces");

    let (class_aws, trace_aws_t1) = vault.classify_with_trace(payload_aws, &clock1).await;
    assert_ne!(trace_sk_t1, trace_aws_t1, "Different payloads/rules must yield different traces");

    let rule_sk = extract_rule_id(&class_sk);
    let rule_aws = extract_rule_id(&class_aws);
    assert_ne!(rule_sk, rule_aws);
    Ok(())
}

#[tokio::test]
async fn test_audit_trace_parity_with_classify() -> Result<(), Box<dyn std::error::Error>> {
    let patterns = vec![r"password".to_string()];
    let vault = EgressVault::new(patterns)?;
    let clock = FixedClock::new(1_234_567_890);

    let clean_payload = "Safe text payload";
    let block_payload = "Text with password inside";

    let direct_clean = vault.classify(clean_payload).await;
    let (traced_clean_class, _trace_clean) = vault.classify_with_trace(clean_payload, &clock).await;
    assert_eq!(direct_clean, traced_clean_class);
    assert_eq!(traced_clean_class, EgressClassification::Allow);

    let direct_block = vault.classify(block_payload).await;
    let (traced_block_class, _trace_block) = vault.classify_with_trace(block_payload, &clock).await;
    assert_eq!(direct_block, traced_block_class);
    assert!(matches!(
        traced_block_class,
        EgressClassification::Block(BlockReason::SensitivePattern(_))
    ));
    Ok(())
}

#[tokio::test]
async fn test_audit_trace_empty_and_max_boundary_no_panic() -> Result<(), Box<dyn std::error::Error>> {
    let vault = EgressVault::try_default()?;
    let clock = FixedClock::new(500);

    // Empty payload
    let (class_empty, trace_empty) = vault.classify_with_trace("", &clock).await;
    assert_eq!(class_empty, EgressClassification::Allow);
    assert_ne!(trace_empty, [0u8; 32]);

    // Exact boundary size payload
    let exact_payload = "a".repeat(MAX_CLASSIFY_PAYLOAD_BYTES);
    let (class_exact, trace_exact) = vault.classify_with_trace(&exact_payload, &clock).await;
    assert_eq!(class_exact, EgressClassification::Allow);
    assert_ne!(trace_exact, [0u8; 32]);

    // Oversized payload
    let oversized_payload = "a".repeat(MAX_CLASSIFY_PAYLOAD_BYTES + 1);
    let (class_over, trace_over) = vault.classify_with_trace(&oversized_payload, &clock).await;
    assert!(matches!(
        class_over,
        EgressClassification::Block(BlockReason::EgressPolicyDenied(_))
    ));
    assert_ne!(trace_over, [0u8; 32]);
    Ok(())
}
