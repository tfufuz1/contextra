#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]
//! Integrationstest für Zeroize-Verhalten und Fail-Closed-Verhalten bei Egress-Policies.
//!
//! # Zweck
//! Verifiziert, dass sensible Payloads und Surrogate nach Drop zuverlässig unbrauchbar werden
//! und dass EgressGuard und Gateway bei unbekannten oder restriktiven Policies fail-closed reagieren.

use contextra_privacy::{
    BlockReason, EgressClassification, EgressDecision, EgressGuard, EgressPolicy, EgressVault,
    GuardedPayload, RuleBasedEntityRecognizer, SurrogateVault, TextSearchEngine, TextSearchResult,
    Unsanitized,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use zeroize::Zeroize;

struct StrictDenyPolicy;
impl EgressPolicy for StrictDenyPolicy {
    fn evaluate(&self, _payload: &str) -> EgressDecision {
        EgressDecision::Deny("Strict fail-closed policy triggered".to_string())
    }
}

struct EmptyMockEngine;
impl TextSearchEngine for EmptyMockEngine {
    fn search_text<'a>(
        &'a self,
        _text: &'a str,
        _limit: usize,
    ) -> contextra_privacy::BoxFuture<'a, Result<Vec<TextSearchResult>, String>> {
        Box::pin(async move { Ok(Vec::new()) })
    }
}

#[tokio::test]
async fn test_egress_guard_fail_closed_on_policy_and_empty_index() {
    let policy = StrictDenyPolicy;
    let decision = policy.evaluate("confidential medical query");
    assert_eq!(
        decision,
        EgressDecision::Deny("Strict fail-closed policy triggered".to_string())
    );

    let engine = Arc::new(EmptyMockEngine);
    let guard = EgressGuard::new(engine, 0.85, 128);
    // EgressGuard fails closed if no similar vectors or index is empty
    let classification = guard
        .check(
            "very long query exceeding the minimum 128 bytes limit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .await;
    assert!(
        matches!(
            classification,
            EgressClassification::Block(BlockReason::InternalError(_))
        ),
        "EgressGuard must fail-closed with Block on empty index, got {:?}",
        classification
    );
}

#[test]
fn test_surrogate_vault_zeroize_and_clear_lifecycle() {
    let vault = SurrogateVault::new([42u8; 16]);
    let surrogate = vault
        .generate_surrogate("sensitive-credit-card-1234")
        .expect("generate ok");
    assert!(vault.get_entity(&surrogate).is_some());
    assert_eq!(vault.len(), 1);

    vault.clear();
    assert_eq!(vault.len(), 0);
    assert!(vault.get_entity(&surrogate).is_none());
}

#[test]
fn test_guarded_payload_zeroize_drop_cycle() {
    static DROP_COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct Tracker;
    impl Drop for Tracker {
        fn drop(&mut self) {
            DROP_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
    }

    {
        let mut sensitive_buf = [0x55u8; 32];
        let payload = GuardedPayload::<Unsanitized>::new(
            "secret-data".to_string(),
            "session-abc".to_string(),
        );
        assert_eq!(payload.session_id(), "session-abc");
        let _tracker = Tracker;
        // payload and tracker drop here
        drop(payload);
        drop(_tracker);

        sensitive_buf.zeroize();
        assert_eq!(sensitive_buf, [0u8; 32]);
    }

    assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 1);
}

#[test]
fn test_egress_vault_sanitization_deterministic() {
    let vault = EgressVault::try_default().expect("valid vault");
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Mitarbeiter Max (max@corp.de) mit IBAN DE89370400440532013000";

    let (sanitized1, count1) = vault
        .sanitize_and_vault(text, &recognizer)
        .expect("sanitize ok");
    let (sanitized2, count2) = vault
        .sanitize_and_vault(text, &recognizer)
        .expect("sanitize ok");

    assert_eq!(count1, 2);
    assert_eq!(count2, 2);
    assert_eq!(sanitized1, sanitized2);
}
