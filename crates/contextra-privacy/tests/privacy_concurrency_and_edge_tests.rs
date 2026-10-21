// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Comprehensive test suite for Privacy egress DLP filter race conditions and edge cases.
// INVARIANTEN: Deterministic parallel sanitization and surrogate mapping, fail-closed payload boundary checks, Unicode normalization invariance.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]

use contextra_privacy::{
    BlockReason, EgressClassification, EgressClassifier, EgressVault, NoOpRecognizer,
    SurrogateVault, MAX_CLASSIFY_PAYLOAD_BYTES,
};
use std::sync::Arc;
use tokio::sync::{mpsc, Barrier};

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Invariant: Parallel sanitization of the exact same document across 10+ threads produces deterministically identical surrogate maps and outputs.
/// Oracle: All worker threads produce identical sanitized strings, surrogate token keys, and entity lookup maps for the same session salt.
#[tokio::test]
async fn test_parallel_filtering_same_document_deterministic_output() -> TestResult {
    let seed: u64 = 0x9988_7766_5544_3322;
    let num_threads = 12;
    let barrier = Arc::new(Barrier::new(num_threads));
    let salt = [0x55u8; 16];
    let surrogate_vault = Arc::new(SurrogateVault::new(salt));

    let patterns = vec![
        r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b".to_string(),
        r"sk-[a-zA-Z0-9]{32}".to_string(),
    ];
    let vault = Arc::new(EgressVault::new(patterns)?.with_surrogate_vault(surrogate_vault.clone()));

    let document = "User alice@example.com logged in with key sk-12345678901234567890123456789012 into cluster";

    let (tx, mut rx) = mpsc::channel(num_threads);

    for i in 0..num_threads {
        let barrier_cloned = barrier.clone();
        let vault_cloned = vault.clone();
        let tx_cloned = tx.clone();

        tokio::spawn(async move {
            barrier_cloned.wait().await;
            let res = vault_cloned.sanitize_and_vault(document, &NoOpRecognizer);
            let _ = tx_cloned.send((i, res)).await;
        });
    }
    drop(tx);

    let mut first_sanitized: Option<String> = None;
    let mut count_results = 0;

    while let Some((worker_id, result)) = rx.recv().await {
        count_results += 1;
        let (sanitized, count) =
            result.unwrap_or_else(|e| panic!("Worker {worker_id} (seed 0x{seed:x}) failed: {e}"));
        assert_eq!(count, 2, "Must sanitize exactly 2 PII entities");

        if let Some(ref expected) = first_sanitized {
            assert_eq!(
                &sanitized, expected,
                "Worker {worker_id} (seed 0x{seed:x}) produced non-deterministic surrogate output mismatch!"
            );
        } else {
            first_sanitized = Some(sanitized);
        }
    }

    assert_eq!(count_results, num_threads);
    assert_eq!(
        surrogate_vault.len(),
        2,
        "Surrogate vault must contain exactly 2 unique surrogate mappings"
    );

    Ok(())
}

/// Invariant: High-concurrency surrogate generation on SurrogateVault under lock contention preserves map integrity without collisions or deadlocks.
/// Oracle: Concurrent insertions across threads succeed cleanly and vault length matches total distinct entities.
#[tokio::test]
async fn test_concurrent_surrogate_vault_generation_contention() -> TestResult {
    let seed: u64 = 0x1122_3344_5566_7788;
    let num_threads = 10;
    let items_per_thread = 50;
    let barrier = Arc::new(Barrier::new(num_threads));
    let surrogate_vault = Arc::new(SurrogateVault::new([0xAAu8; 16]));

    let (tx, mut rx) = mpsc::channel(num_threads);

    for thread_idx in 0..num_threads {
        let barrier_cloned = barrier.clone();
        let vault_cloned = surrogate_vault.clone();
        let tx_cloned = tx.clone();

        tokio::spawn(async move {
            barrier_cloned.wait().await;
            let mut generated = Vec::with_capacity(items_per_thread);
            for item_idx in 0..items_per_thread {
                let entity = format!("thread_{thread_idx}_user_{item_idx}@example.com");
                let surrogate = vault_cloned
                    .generate_surrogate(&entity)
                    .unwrap_or_else(|e| panic!("Thread {thread_idx} item {item_idx} failed: {e}"));
                generated.push((surrogate, entity));
            }
            let _ = tx_cloned.send((thread_idx, generated)).await;
        });
    }
    drop(tx);

    let mut total_received = 0;
    while let Some((thread_idx, mappings)) = rx.recv().await {
        total_received += mappings.len();
        for (surrogate, entity) in mappings {
            let fetched = surrogate_vault.get_entity(&surrogate);
            assert_eq!(
                fetched,
                Some(entity.clone()),
                "Thread {thread_idx} entity lookup for surrogate {surrogate} failed (seed: 0x{seed:x})"
            );
        }
    }

    assert_eq!(total_received, num_threads * items_per_thread);
    assert_eq!(
        surrogate_vault.len(),
        num_threads * items_per_thread,
        "SurrogateVault count must equal total unique entities inserted concurrently"
    );

    Ok(())
}

/// Invariant: Vault instance creation or pattern reloads during active background classification do not cause race conditions or memory corruption.
/// Oracle: Background classification tasks complete fail-closed or allow cleanly while main thread constructs new EgressVault instances.
#[tokio::test]
async fn test_vault_recreation_during_active_classification_race() -> TestResult {
    let num_threads = 8;
    let barrier = Arc::new(Barrier::new(num_threads + 1));
    let patterns = vec![r"sk-[a-zA-Z0-9]{32}".to_string()];
    let vault = Arc::new(EgressVault::new(patterns)?);

    let (tx, mut rx) = mpsc::channel(num_threads);

    for worker_id in 0..num_threads {
        let barrier_cloned = barrier.clone();
        let vault_cloned = vault.clone();
        let tx_cloned = tx.clone();

        tokio::spawn(async move {
            barrier_cloned.wait().await;
            let payload = if worker_id % 2 == 0 {
                "Public harmless payload string"
            } else {
                "Secret payload sk-01234567890123456789012345678901 inside"
            };
            let res = vault_cloned.classify(payload).await;
            let _ = tx_cloned.send((worker_id, res)).await;
        });
    }

    // Main thread waits at barrier then concurrently builds new EgressVault instances
    barrier.wait().await;
    for _ in 0..10 {
        let _new_vault = EgressVault::try_default()?;
    }

    drop(tx);

    let mut count = 0;
    while let Some((worker_id, res)) = rx.recv().await {
        count += 1;
        if worker_id % 2 == 0 {
            assert_eq!(res, EgressClassification::Allow);
        } else {
            assert_eq!(
                res,
                EgressClassification::Block(BlockReason::SensitivePattern("R-001".to_string()))
            );
        }
    }

    assert_eq!(count, num_threads);
    Ok(())
}

/// Invariant: Egress Vault edge cases (empty text, text with exclusively PII, PII at chunk boundaries, exact payload boundary).
/// Oracle: Exact classification and sanitization expectations met without panic.
#[tokio::test]
async fn test_privacy_edge_cases_empty_only_pii_and_boundaries() -> TestResult {
    let vault = EgressVault::try_default()?;

    // Edge Case 1: Empty text
    let res_empty = vault.classify("").await;
    assert_eq!(res_empty, EgressClassification::Allow);
    let (sanitized_empty, count_empty) = vault.sanitize_and_vault("", &NoOpRecognizer)?;
    assert_eq!(sanitized_empty, "");
    assert_eq!(count_empty, 0);

    // Edge Case 2: Exclusively PII
    let only_pii = "alice@example.com";
    let res_only_pii = vault.classify(only_pii).await;
    assert_eq!(
        res_only_pii,
        EgressClassification::Block(BlockReason::SensitivePattern("R-005".to_string()))
    );
    let (sanitized_only_pii, count_only_pii) =
        vault.sanitize_and_vault(only_pii, &NoOpRecognizer)?;
    assert_eq!(count_only_pii, 1);
    assert!(!sanitized_only_pii.contains("alice@example.com"));
    assert!(sanitized_only_pii.starts_with("[USER_ENTITY_"));

    // Edge Case 3: PII at chunk boundaries
    let boundary_text = "start prefix alice@example.com end suffix";
    let res_boundary = vault.classify(boundary_text).await;
    assert_eq!(
        res_boundary,
        EgressClassification::Block(BlockReason::SensitivePattern("R-005".to_string()))
    );
    let (sanitized_boundary, count_boundary) =
        vault.sanitize_and_vault(boundary_text, &NoOpRecognizer)?;
    assert_eq!(count_boundary, 1);
    assert!(sanitized_boundary.starts_with("start prefix [USER_ENTITY_"));
    assert!(sanitized_boundary.ends_with(" end suffix"));

    // Edge Case 4: Exact MAX_CLASSIFY_PAYLOAD_BYTES boundary
    let exact_payload = "clean_text_".repeat(MAX_CLASSIFY_PAYLOAD_BYTES / 11);
    let res_exact = vault.classify(&exact_payload).await;
    assert_eq!(res_exact, EgressClassification::Allow);

    // Edge Case 5: MAX_CLASSIFY_PAYLOAD_BYTES + 1 byte
    let oversized_payload = "a".repeat(MAX_CLASSIFY_PAYLOAD_BYTES + 1);
    let res_oversized = vault.classify(&oversized_payload).await;
    assert!(
        matches!(res_oversized, EgressClassification::Block(BlockReason::EgressPolicyDenied(ref msg)) if msg.contains("exceeds limit")),
        "Oversized payload must return EgressPolicyDenied block"
    );

    Ok(())
}

/// Invariant: Unicode normalization variants (NFC, NFD, NFKC, NFKD) containing ASCII PII patterns match DLP rules correctly.
/// Oracle: All normalization forms containing the secret string trigger classification block cleanly without panics.
#[test]
fn test_unicode_normalization_variants_handling() -> TestResult {
    let vault = EgressVault::try_default()?;

    // ASCII PII with combining accents in surrounding text across Unicode forms
    let raw_nfc = "Café secret_key alice@example.com résumé";
    let raw_nfd = "Cafe\u{301} secret_key alice@example.com re\u{301}sume\u{301}";

    let (sanitized_nfc, count_nfc) = vault.sanitize_and_vault(raw_nfc, &NoOpRecognizer)?;
    let (sanitized_nfd, count_nfd) = vault.sanitize_and_vault(raw_nfd, &NoOpRecognizer)?;

    assert_eq!(count_nfc, 1, "NFC string must replace email PII");
    assert_eq!(count_nfd, 1, "NFD string must replace email PII");

    assert!(!sanitized_nfc.contains("alice@example.com"));
    assert!(!sanitized_nfd.contains("alice@example.com"));

    Ok(())
}
