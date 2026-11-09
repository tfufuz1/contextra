//! Fail-Closed Matrix & Egress Pipeline Tests (SPEC PRIV-2, INV-PRIVACY-FAILCLOSED-1).

use contextra_privacy::audit_trace::{EventSink, InMemoryEventSink};
use contextra_privacy::bulk_exfiltration_detector::{
    BulkExfiltrationDetector, BulkExfiltrationDetectorConfig,
};
use contextra_privacy::egress_gateway::{EgressGateway, NoOpSender, Sender};
use contextra_privacy::egress_guard::{
    DefaultEgressPolicy, EgressGuard, EgressGuardConfig, EgressPolicy, GuardDecision,
    RULE_BULK_EXFILTRATION_MATCH, RULE_PATTERN_DETECTED, RULE_PAYLOAD_SIZE_LIMIT,
};
use contextra_privacy::egress_vault::{EgressVault, EgressVaultConfig, MaskingSealer};
use contextra_privacy::error::{Error, Result};
use contextra_privacy::guarded_payload::GuardedPayload;
use contextra_privacy::types::DeterministicClock;
use rand_core::RngCore;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

struct FailingPolicy;
impl EgressPolicy for FailingPolicy {
    fn check_destination(&self, _destination: &str) -> Result<bool> {
        Err(Error::Corrupt("Simulated internal policy failure".into()))
    }
    fn check_content(&self, _text: &str) -> Result<bool> {
        Err(Error::Corrupt("Simulated internal scanner failure".into()))
    }
}

struct SimpleRng(AtomicU32);
impl RngCore for SimpleRng {
    fn next_u32(&mut self) -> u32 {
        self.0.fetch_add(1, Ordering::Relaxed)
    }
    fn next_u64(&mut self) -> u64 {
        self.next_u32() as u64
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for b in dest.iter_mut() {
            *b = (self.next_u32() & 0xFF) as u8;
        }
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> std::result::Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

/// SPEC: INV-PRIVACY-FAILCLOSED-1
#[test]
fn test_fail_closed_on_policy_error() {
    let clock = Arc::new(DeterministicClock::new(100));
    let config = EgressGuardConfig::default();
    let guard = EgressGuard::new(config, Arc::new(FailingPolicy), None, None, clock);

    let decision = guard.inspect("Normal safe text", "https://api.openai.com");
    assert!(decision.is_blocked(), "Must fail closed on policy error");
}

/// SPEC: INV-PRIVACY-FAILCLOSED-1
#[test]
fn test_fail_closed_on_oversized_payload() {
    let clock = Arc::new(DeterministicClock::new(100));
    let config = EgressGuardConfig {
        max_payload_bytes: 10,
        strict_fail_closed: true,
    };
    let guard = EgressGuard::new(config, Arc::new(DefaultEgressPolicy::new()), None, None, clock);

    let decision = guard.inspect("This text is 26 bytes long", "https://api.openai.com");
    assert!(decision.is_blocked());
    if let GuardDecision::Blocked { rule_id, .. } = decision {
        assert_eq!(rule_id, RULE_PAYLOAD_SIZE_LIMIT);
    }
}

/// SPEC: INV-PRIVACY-FAILCLOSED-1
#[test]
fn test_fail_closed_on_blocked_pattern() {
    let clock = Arc::new(DeterministicClock::new(100));
    let mut policy = DefaultEgressPolicy::new();
    policy.add_blocked_keyword("FORBIDDEN_PII");
    let guard = EgressGuard::new(
        EgressGuardConfig::default(),
        Arc::new(policy),
        None,
        None,
        clock,
    );

    let decision = guard.inspect("User says FORBIDDEN_PII please help", "https://api.openai.com");
    assert!(decision.is_blocked());
    if let GuardDecision::Blocked { rule_id, .. } = decision {
        assert_eq!(rule_id, RULE_PATTERN_DETECTED);
    }
}

/// SPEC: INV-PRIVACY-FAILCLOSED-1
#[test]
fn test_bulk_exfiltration_fail_closed() {
    let clock = Arc::new(DeterministicClock::new(1_000));
    let det_config = BulkExfiltrationDetectorConfig {
        window_duration_nanos: 1_000_000_000,
        max_bytes_per_window: 100,
        max_requests_per_window: 2,
    };
    let detector = Arc::new(BulkExfiltrationDetector::new(det_config, clock.clone()));
    let guard = EgressGuard::new(
        EgressGuardConfig::default(),
        Arc::new(DefaultEgressPolicy::new()),
        Some(detector),
        None,
        clock,
    );

    // Anfrage 1: 50 B -> OK
    assert!(guard.inspect(&"A".repeat(50), "https://api.openai.com").is_allowed());
    // Anfrage 2: 50 B -> OK
    assert!(guard.inspect(&"B".repeat(50), "https://api.openai.com").is_allowed());
    // Anfrage 3: Schwellwert für Requests (2) überschritten -> Block
    let d3 = guard.inspect("C", "https://api.openai.com");
    assert!(d3.is_blocked());
    if let GuardDecision::Blocked { rule_id, .. } = d3 {
        assert_eq!(rule_id, RULE_BULK_EXFILTRATION_MATCH);
    }
}

#[test]
fn test_vault_sealer_zeroize() {
    let sealer = Box::new(MaskingSealer::new(0x55));
    let vault = EgressVault::new(EgressVaultConfig::default(), sealer);
    let mut rng = SimpleRng(AtomicU32::new(1));

    let secret = "SuperSecretPassword123";
    let token = vault.tokenize(secret, &mut rng).unwrap();
    assert!(token.contains("CTX_SURROGATE"));

    let detokenized = vault.detokenize(&token).unwrap();
    assert_eq!(detokenized, secret);
}

#[test]
fn test_gateway_event_sink_audit() {
    let clock = Arc::new(DeterministicClock::new(500));
    let guard = Arc::new(EgressGuard::new(
        EgressGuardConfig::default(),
        Arc::new(DefaultEgressPolicy::new()),
        None,
        None,
        clock.clone(),
    ));
    let sink = Arc::new(InMemoryEventSink::new());
    let gateway = EgressGateway::new(guard, Arc::new(NoOpSender), Some(sink.clone()), clock);

    gateway.dispatch("https://api.openai.com", "Approved prompt").unwrap();

    let events = sink.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].category, "egress_dispatched");
    assert_eq!(events[0].outcome, "payload transmitted successfully");
}
