#![cfg(not(loom))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::evidence::WitLayerEvidence;
use contextra_crypto::deletion_proof::probe::WitStoreGeneration;
use contextra_crypto::deletion_proof::{DeletionLayer, LayerCleanupProof};
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs, WitnessEvidence};
use contextra_types::TxId;

#[test]
fn from_witness_rejects_invalid_witness() -> contextra_types::Result<()> {
    // Case 1: Empty / unverified evidence is rejected (WitnessVerification::None)
    let empty_wit = LayerWitness::issue_from_storage_layer(
        DeletionLayer::LsmMemtable,
        &[b"key1".to_vec()],
        TxId(100),
        WitnessEvidence::MemtableScan {
            memtables_scanned: 0,
        },
    )?;
    assert!(
        LayerCleanupProof::from_witness(&empty_wit).is_err(),
        "from_witness must reject witness with WitnessVerification::None"
    );

    // Case 2: Witness with remaining live entries fails witness issuance/verification
    let non_empty_wit_res = LayerWitness::issue(WitIssueArgs {
        layer: DeletionLayer::LsmMemtable,
        scope_hash: [0u8; 32],
        scanned_at_tx: TxId(100),
        as_of_tx: TxId(100),
        assurance: WitAssurance::S1,
        claim: WitClaimText::S1,
        footprint_hash: [0u8; 32],
        scope_write_seq: 0,
        scanner_name: "memtable_scanner",
        scanner_version: 1,
        store_generation: WitStoreGeneration(1),
        layer_evidence: WitLayerEvidence::Memtable {
            memtables_scanned: 1,
            entries_found: 5,
            tombstones_found: 2,
        },
        key_identifiers_claimed: false,
    });
    assert!(
        non_empty_wit_res.is_err(),
        "Witness issuance must fail when live entries exist"
    );

    // Case 3: Valid witness with 0 remaining entries succeeds
    let valid_wit = LayerWitness::issue_from_storage_layer(
        DeletionLayer::LsmMemtable,
        &[b"key1".to_vec()],
        TxId(100),
        WitnessEvidence::MemtableScan {
            memtables_scanned: 1,
        },
    )?;
    let proof = LayerCleanupProof::from_witness(&valid_wit)?;
    assert_eq!(proof.layer(), &DeletionLayer::LsmMemtable);

    Ok(())
}
