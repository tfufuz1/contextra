//! Integrationstests für Witness / Evidence / ClaimText / Mandatory Exclusions v2 (W5-02).

#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_crypto::{
    deletion_proof::{
        claim_text::{WitAssurance, WitClaimText},
        evidence::WitLayerEvidence,
        exclusions::{mandatory_exclusions_for, mandatory_ids, WIT_MANDATORY_EXCLUSIONS},
        probe::WitStoreGeneration,
        types::{DeletionLayer, ExcludedScope},
    },
    deletion_witness::{LayerWitness, WitIssueArgs},
};
use contextra_types::TxId;

#[test]
fn test_claim_text_ids_frozen() {
    assert_eq!(WitClaimText::S0.id(), 1);
    assert_eq!(WitClaimText::S1.id(), 2);
    assert_eq!(WitClaimText::S2V.id(), 3);
    assert_eq!(WitClaimText::S2K.id(), 4);
    assert_eq!(WitClaimText::S3.id(), 5);
    assert_eq!(WitClaimText::S4.id(), 6);
    assert_eq!(WitClaimText::RamUnreferenced.id(), 7);
    assert_eq!(WitClaimText::RamZeroized.id(), 8);

    for claim in WitClaimText::all() {
        assert_eq!(WitClaimText::from_id(claim.id()), Some(*claim));
        assert!(!claim.text().is_empty());
    }

    assert_eq!(WitClaimText::from_id(0), None);
    assert_eq!(WitClaimText::from_id(9), None);
}

#[test]
fn test_mandatory_exclusions() {
    assert_eq!(WIT_MANDATORY_EXCLUSIONS.len(), 9);

    let s0_exclusions = mandatory_exclusions_for(WitAssurance::S0);
    assert!(s0_exclusions.contains(&ExcludedScope::StorageMediaRemanence));
    assert_eq!(s0_exclusions.len(), 9);

    let s3_exclusions = mandatory_exclusions_for(WitAssurance::S3);
    assert!(s3_exclusions.contains(&ExcludedScope::StorageMediaRemanence));
    assert_eq!(s3_exclusions.len(), 9);

    let s4_exclusions = mandatory_exclusions_for(WitAssurance::S4);
    assert!(!s4_exclusions.contains(&ExcludedScope::StorageMediaRemanence));
    assert_eq!(s4_exclusions.len(), 8);

    let ids = mandatory_ids();
    assert_eq!(ids.len(), 9);
}

#[test]
fn test_evidence_memtable_invariants() {
    let valid = WitLayerEvidence::Memtable {
        memtables_scanned: 2,
        entries_found: 5,
        tombstones_found: 5,
    };
    assert!(valid.check_invariants(WitAssurance::S0, false).is_ok());

    let invalid = WitLayerEvidence::Memtable {
        memtables_scanned: 2,
        entries_found: 6,
        tombstones_found: 5,
    };
    let err = match invalid.check_invariants(WitAssurance::S0, false) {
        Err(e) => e,
        Ok(_) => panic!("Expected WitInvariantViolation"),
    };
    assert_eq!(err.layer_name, "LsmMemtable");
    assert_eq!(err.field_name, "entries_found");
}

#[test]
fn test_evidence_sstables_invariants() {
    let valid = WitLayerEvidence::SsTables {
        manifest_gen: 10,
        files_in_manifest: 10,
        files_scanned: 8,
        files_pruned_with_verified_range: 2,
        blocks_scanned: 100,
        bytes_scanned: 4096,
        orphan_files_found: 0,
        unclassified_files: 0,
        versions_found: 0,
        identifier_hits_index: 0,
        tombstones_remaining: 0,
    };
    assert!(valid.check_invariants(WitAssurance::S2, false).is_ok());

    // file sum mismatch
    let mut invalid = valid.clone();
    if let WitLayerEvidence::SsTables { files_scanned, .. } = &mut invalid {
        *files_scanned = 7;
    }
    assert!(invalid.check_invariants(WitAssurance::S2, false).is_err());

    // orphan files found
    let mut invalid = valid.clone();
    if let WitLayerEvidence::SsTables { orphan_files_found, .. } = &mut invalid {
        *orphan_files_found = 1;
    }
    assert!(invalid.check_invariants(WitAssurance::S2, false).is_err());

    // versions found
    let mut invalid = valid.clone();
    if let WitLayerEvidence::SsTables { versions_found, .. } = &mut invalid {
        *versions_found = 1;
    }
    assert!(invalid.check_invariants(WitAssurance::S2, false).is_err());

    // identifier hits index for S2
    let mut invalid = valid.clone();
    if let WitLayerEvidence::SsTables { identifier_hits_index, .. } = &mut invalid {
        *identifier_hits_index = 1;
    }
    assert!(invalid.check_invariants(WitAssurance::S2, false).is_err());
}

#[test]
fn test_evidence_manifest_invariants() {
    let valid = WitLayerEvidence::Manifest {
        current_manifest_gen: 5,
        in_flight_commits: 0,
        unpersisted_edits: 0,
    };
    assert!(valid.check_invariants(WitAssurance::S0, false).is_ok());

    let invalid = WitLayerEvidence::Manifest {
        current_manifest_gen: 5,
        in_flight_commits: 1,
        unpersisted_edits: 0,
    };
    assert!(invalid.check_invariants(WitAssurance::S0, false).is_err());
}

#[test]
fn test_evidence_wal_invariants() {
    let valid = WitLayerEvidence::Wal {
        segments_scanned: 3,
        segments_total: 5,
        segments_removed: 2,
        bytes_scrubbed: 1024,
        unscrubbed_records_found: 0,
    };
    assert!(valid.check_invariants(WitAssurance::S3, false).is_ok());

    let invalid = WitLayerEvidence::Wal {
        segments_scanned: 3,
        segments_total: 5,
        segments_removed: 1,
        bytes_scrubbed: 1024,
        unscrubbed_records_found: 0,
    };
    assert!(invalid.check_invariants(WitAssurance::S3, false).is_err());
}

#[test]
fn test_evidence_hnsw_invariants() {
    let valid = WitLayerEvidence::HnswIndex {
        node_id: 42,
        incoming_edges: 0,
        outgoing_edges: 3,
        pinned_by_reader: false,
        marked_zeroed: true,
    };
    assert!(valid.check_invariants(WitAssurance::S1, false).is_ok());

    let invalid = WitLayerEvidence::HnswIndex {
        node_id: 42,
        incoming_edges: 1,
        outgoing_edges: 3,
        pinned_by_reader: false,
        marked_zeroed: true,
    };
    assert!(invalid.check_invariants(WitAssurance::S1, false).is_err());
}

#[test]
fn test_evidence_keyring_invariants() {
    let valid = WitLayerEvidence::Keyring {
        kek_epoch: 1,
        dek_present: false,
        kek_epoch_destroyed: true,
    };
    assert!(valid.check_invariants(WitAssurance::S4, false).is_ok());

    let invalid = WitLayerEvidence::Keyring {
        kek_epoch: 1,
        dek_present: true,
        kek_epoch_destroyed: true,
    };
    assert!(invalid.check_invariants(WitAssurance::S4, false).is_err());
}

#[test]
fn test_tlv_determinism_and_digest() {
    let ev1 = WitLayerEvidence::Wal {
        segments_scanned: 3,
        segments_total: 5,
        segments_removed: 2,
        bytes_scrubbed: 1024,
        unscrubbed_records_found: 0,
    };
    let ev2 = WitLayerEvidence::Wal {
        segments_scanned: 3,
        segments_total: 5,
        segments_removed: 2,
        bytes_scrubbed: 1024,
        unscrubbed_records_found: 0,
    };

    let tlv1 = ev1.to_canonical_tlv();
    let tlv2 = ev2.to_canonical_tlv();
    assert_eq!(tlv1, tlv2);

    let digest1 = ev1.witness_digest(4);
    let digest2 = ev2.witness_digest(4);
    assert_eq!(digest1, digest2);

    // Diff layer id -> diff digest
    let digest_other = ev1.witness_digest(5);
    assert_ne!(digest1, digest_other);
}

#[test]
fn test_issue_witness_scanned_at_tx_less_than_as_of_tx_rejected() {
    let evidence = WitLayerEvidence::Memtable {
        memtables_scanned: 1,
        entries_found: 0,
        tombstones_found: 0,
    };

    let args = WitIssueArgs {
        layer: DeletionLayer::LsmMemtable,
        scope_hash: [1u8; 32],
        scanned_at_tx: TxId(9),
        as_of_tx: TxId(10),
        assurance: WitAssurance::S0,
        claim: WitClaimText::S0,
        footprint_hash: [0u8; 32],
        scope_write_seq: 1,
        scanner_name: "memtable_scanner",
        scanner_version: 1,
        store_generation: WitStoreGeneration(1),
        layer_evidence: evidence,
        key_identifiers_claimed: false,
    };

    let res = LayerWitness::issue(args);
    assert!(res.is_err());
}

#[test]
fn test_issue_witness_success() {
    let evidence = WitLayerEvidence::Memtable {
        memtables_scanned: 1,
        entries_found: 0,
        tombstones_found: 0,
    };

    let args = WitIssueArgs {
        layer: DeletionLayer::LsmMemtable,
        scope_hash: [1u8; 32],
        scanned_at_tx: TxId(10),
        as_of_tx: TxId(10),
        assurance: WitAssurance::S0,
        claim: WitClaimText::S0,
        footprint_hash: [0u8; 32],
        scope_write_seq: 1,
        scanner_name: "memtable_scanner",
        scanner_version: 1,
        store_generation: WitStoreGeneration(1),
        layer_evidence: evidence,
        key_identifiers_claimed: false,
    };

    let witness = LayerWitness::issue(args).map_err(|e| format!("issue failed: {e}")).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(witness.assurance(), WitAssurance::S0);
    assert_eq!(witness.claim(), WitClaimText::S0);
    assert_eq!(witness.scanned_at_tx(), TxId(10));
    assert_eq!(witness.as_of_tx(), TxId(10));
    assert!(witness.layer_evidence().is_some());
}
