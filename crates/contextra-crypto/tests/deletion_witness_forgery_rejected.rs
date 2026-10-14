//! Integration test for LayerWitness forgery rejection and verification (Auftrag B01 / F7).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_crypto::deletion_proof::{
    claim_text::{WitAssurance, WitClaimText},
    evidence::WitLayerEvidence,
    probe::WitStoreGeneration,
    DeletionLayer,
};
use contextra_crypto::deletion_witness::{
    LayerWitness, WitIssueArgs, WitnessEvidence, WitnessVerification,
};
use contextra_types::TxId;

#[test]
fn test_manipulated_counter_digest_mismatch() {
    let args = WitIssueArgs {
        layer: DeletionLayer::LsmMemtable,
        scope_hash: [0xAAu8; 32],
        scanned_at_tx: TxId(100),
        as_of_tx: TxId(90),
        assurance: WitAssurance::S1,
        claim: WitClaimText::S1,
        footprint_hash: [0xBBu8; 32],
        scope_write_seq: 1,
        scanner_name: "test_scanner",
        scanner_version: 1,
        store_generation: WitStoreGeneration(1),
        layer_evidence: WitLayerEvidence::Memtable {
            memtables_scanned: 5,
            entries_found: 0,
            tombstones_found: 2,
        },
        key_identifiers_claimed: false,
    };

    let witness = LayerWitness::issue(args).expect("issue should succeed");

    // Initially valid
    let ver = witness.verify().expect("verify should succeed");
    assert_eq!(ver, WitnessVerification::Verified(WitAssurance::S1));

    // Mismatch between layer (SsTables) and layer_evidence (Memtable)
    let args_mismatch = WitIssueArgs {
        layer: DeletionLayer::SsTableAllLevels,
        scope_hash: [0xAAu8; 32],
        scanned_at_tx: TxId(100),
        as_of_tx: TxId(90),
        assurance: WitAssurance::S2,
        claim: WitClaimText::S2V,
        footprint_hash: [0xBBu8; 32],
        scope_write_seq: 1,
        scanner_name: "test_scanner",
        scanner_version: 1,
        store_generation: WitStoreGeneration(1),
        layer_evidence: WitLayerEvidence::Memtable {
            memtables_scanned: 5,
            entries_found: 0,
            tombstones_found: 2,
        },
        key_identifiers_claimed: false,
    };

    let witness_bad = LayerWitness::issue(args_mismatch).expect("issue succeeds");
    let res = witness_bad.verify();
    assert!(
        res.is_err(),
        "mismatch between layer (SsTables) and layer_evidence (Memtable) must be rejected"
    );
}

#[test]
fn test_invalid_layer_claim_rejected() {
    // Memtable layer claiming Assurance S4
    let args = WitIssueArgs {
        layer: DeletionLayer::LsmMemtable,
        scope_hash: [0xAAu8; 32],
        scanned_at_tx: TxId(100),
        as_of_tx: TxId(90),
        assurance: WitAssurance::S4,
        claim: WitClaimText::S4,
        footprint_hash: [0xBBu8; 32],
        scope_write_seq: 1,
        scanner_name: "test_scanner",
        scanner_version: 1,
        store_generation: WitStoreGeneration(1),
        layer_evidence: WitLayerEvidence::Memtable {
            memtables_scanned: 5,
            entries_found: 0,
            tombstones_found: 2,
        },
        key_identifiers_claimed: false,
    };

    let witness = LayerWitness::issue(args).expect("issue succeeds");
    let res = witness.verify();
    assert!(
        res.is_err(),
        "Memtable claiming S4 assurance must be rejected during verification"
    );
}

#[test]
fn test_no_op_scanner_evidence_returns_none() {
    // Everything 0
    let args = WitIssueArgs {
        layer: DeletionLayer::LsmMemtable,
        scope_hash: [0x00u8; 32],
        scanned_at_tx: TxId(100),
        as_of_tx: TxId(90),
        assurance: WitAssurance::S0,
        claim: WitClaimText::S0,
        footprint_hash: [0x00u8; 32],
        scope_write_seq: 0,
        scanner_name: "noop_scanner",
        scanner_version: 1,
        store_generation: WitStoreGeneration(0),
        layer_evidence: WitLayerEvidence::Memtable {
            memtables_scanned: 0,
            entries_found: 0,
            tombstones_found: 0,
        },
        key_identifiers_claimed: false,
    };

    let witness = LayerWitness::issue(args).expect("issue succeeds");
    let ver = witness.verify().expect("verify should succeed");
    assert_eq!(
        ver,
        WitnessVerification::None,
        "No-op scanner with 0 scanned memtables must return WitnessVerification::None"
    );

    // Legacy no-op scanner
    let witness_legacy = LayerWitness::issue_from_storage_layer(
        DeletionLayer::LsmMemtable,
        &[b"k1".to_vec()],
        TxId(100),
        WitnessEvidence::MemtableScan {
            memtables_scanned: 0,
        },
    )
    .expect("issue succeeds");

    let ver_legacy = witness_legacy.verify().expect("verify should succeed");
    assert_eq!(
        ver_legacy,
        WitnessVerification::None,
        "Legacy no-op scanner with 0 scanned memtables must return WitnessVerification::None"
    );
}
