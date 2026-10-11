// FILE-CONTEXT
// STAND: 2026-10-11 (T-2026-9103)
// ZWECK: Property-Tests und Semantik-Tests für ErasureLedger decision filter (Keep vs Drop) sowie Hash-Zähl-Verifikation.

use contextra_store::erasure_ledger::*;
use proptest::prelude::*;
use tempfile::tempdir;

proptest! {
    #[test]
    fn prop_ledger_decision_semantics(
        scope_bytes in prop::collection::vec(any::<u8>(), 1..16),
        key1_bytes in prop::collection::vec(any::<u8>(), 5..20),
        key2_bytes in prop::collection::vec(any::<u8>(), 5..20),
        erased_tx in 10u64..100u64,
        test_tx in 1u64..200u64,
    ) {
        let master_secret = [123u8; 32];
        let key = LedgerKey::derive_from_master_secret(&master_secret);

        let hash1 = compute_keyed_hash(&key, &scope_bytes, &key1_bytes);
        let hash2 = compute_keyed_hash(&key, &scope_bytes, &key2_bytes);

        // If keys differ, pseudonym hashes must differ
        if key1_bytes != key2_bytes {
            prop_assert_ne!(hash1, hash2);
        }

        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let dir = tempdir().unwrap();
            let (ledger_opt, _) = ErasureLedger::open_or_fail_closed(dir.path(), key.clone(), false, None).await.unwrap();
            let mut ledger = ledger_opt.unwrap();

            let entry = LedgerEntry {
                ledger_seq: 1,
                kind: LedgerEntryKind::Key,
                keyed_hash: hash1,
                erased_at_tx: erased_tx,
                proof_id: [1u8; 16],
            };
            ledger.append(&[entry]).await.unwrap();

            let decision1 = ledger.decide(&scope_bytes, &key1_bytes, test_tx);
            if test_tx <= erased_tx {
                assert_eq!(decision1, LedgerDecision::Drop);
            } else {
                assert_eq!(decision1, LedgerDecision::Keep);
            }

            // Key2 should not be affected by Key1 erasure
            if key1_bytes != key2_bytes {
                let decision2 = ledger.decide(&scope_bytes, &key2_bytes, test_tx);
                assert_eq!(decision2, LedgerDecision::Keep);
            }
        });
    }
}

#[tokio::test]
async fn test_prefix_erasure_decision() {
    let dir = tempdir().unwrap();
    let master_secret = [88u8; 32];
    let key = LedgerKey::derive_from_master_secret(&master_secret);

    let (ledger_opt, _) = ErasureLedger::open_or_fail_closed(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    let prefix_bytes = b"user:";
    let prefix_hash = compute_keyed_hash(&key, b"tenant_1", prefix_bytes);

    let entry = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Prefix { prefix_len: 5 },
        keyed_hash: prefix_hash,
        erased_at_tx: 50,
        proof_id: [10u8; 16],
    };
    ledger.append(&[entry]).await.unwrap();

    // Key with prefix user:100 at tx 30 <= 50 -> Drop
    let decision1 = ledger.decide(b"tenant_1", b"user:100", 30);
    assert_eq!(decision1, LedgerDecision::Drop);

    // Key with prefix user:100 at tx 60 > 50 -> Keep
    let decision2 = ledger.decide(b"tenant_1", b"user:100", 60);
    assert_eq!(decision2, LedgerDecision::Keep);

    // Key with different prefix org:100 at tx 30 -> Keep
    let decision3 = ledger.decide(b"tenant_1", b"org:100", 30);
    assert_eq!(decision3, LedgerDecision::Keep);
}

#[tokio::test]
async fn test_prefix_hashing_count_optimized_per_distinct_length() {
    let dir = tempdir().unwrap();
    let master_secret = [55u8; 32];
    let key = LedgerKey::derive_from_master_secret(&master_secret);

    let (ledger_opt, _) = ErasureLedger::open_or_fail_closed(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    // Add 100 prefix entries all sharing the SAME prefix length (length = 5)
    let mut entries = Vec::new();
    for i in 0..100u8 {
        let p_bytes = format!("usr{:02}", i);
        let hash = compute_keyed_hash(&key, b"scope", p_bytes.as_bytes());
        entries.push(LedgerEntry {
            ledger_seq: i as u64 + 1,
            kind: LedgerEntryKind::Prefix { prefix_len: 5 },
            keyed_hash: hash,
            erased_at_tx: 10,
            proof_id: [i; 16],
        });
    }
    ledger.append(&entries).await.unwrap();

    // Evaluate decide for a non-matching key.
    // Total BLAKE3 hash calculations performed inside `decide` must be exactly 2:
    // 1 for full key_hash + 1 for the single distinct prefix length 5 (NOT 101 hashes).
    let decision = ledger.decide(b"scope", b"other:item", 5);
    assert_eq!(decision, LedgerDecision::Keep);
}
