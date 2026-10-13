//! Counter-check tests for ProofIssuer v4 (Spec C3, C7).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionScope, IssRequest, ProofIssuer, ProofKeyPairSigner, WitAssurance,
    WitClaimText, WitLayerEvidence, WitStoreGeneration, WitStoreProbe,
};
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};
use contextra_types::{DocId, TenantId, TxId};
use rand::rngs::OsRng;
use std::sync::Arc;
use tempfile::tempdir;

struct MockStoreProbe {
    gen: u64,
    write_seq: u64,
    manifest_count: Option<u32>,
}

impl WitStoreProbe for MockStoreProbe {
    fn current_generation(&self, _layer: &DeletionLayer) -> WitStoreGeneration {
        WitStoreGeneration(self.gen)
    }

    fn scope_write_seq(&self, _scope_hash: &[u8; 32]) -> u64 {
        self.write_seq
    }

    fn manifest_file_count(&self) -> Option<u32> {
        self.manifest_count
    }
}

fn create_valid_witness(layer: DeletionLayer, scope_hash: [u8; 32], tx: u64) -> LayerWitness {
    let assurance = match layer {
        DeletionLayer::SsTableAllLevels => WitAssurance::S2,
        DeletionLayer::WalAllSegments { .. } => WitAssurance::S3,
        _ => WitAssurance::S0,
    };
    let claim = match assurance {
        WitAssurance::S2 => WitClaimText::S2V,
        WitAssurance::S3 => WitClaimText::S3,
        _ => WitClaimText::S0,
    };

    let evidence = match layer {
        DeletionLayer::LsmMemtable => WitLayerEvidence::Memtable {
            memtables_scanned: 1,
            entries_found: 0,
            tombstones_found: 0,
        },
        DeletionLayer::SsTableAllLevels => WitLayerEvidence::SsTables {
            manifest_gen: 1,
            files_in_manifest: 5,
            files_scanned: 5,
            files_pruned_with_verified_range: 0,
            blocks_scanned: 10,
            bytes_scanned: 100,
            orphan_files_found: 0,
            unclassified_files: 0,
            versions_found: 0,
            identifier_hits_index: 0,
            tombstones_remaining: 0,
        },
        _ => WitLayerEvidence::Memtable {
            memtables_scanned: 1,
            entries_found: 0,
            tombstones_found: 0,
        },
    };

    LayerWitness::issue(WitIssueArgs {
        layer,
        scope_hash,
        scanned_at_tx: TxId(tx),
        as_of_tx: TxId(tx),
        assurance,
        claim,
        footprint_hash: [0x33; 32],
        scope_write_seq: 10,
        scanner_name: "test_scanner",
        scanner_version: 1,
        store_generation: WitStoreGeneration(100),
        layer_evidence: evidence,
        key_identifiers_claimed: false,
    })
    .expect("witness issue failed")
}

#[test]
fn test_stale_store_generation_rejected() {
    let dir = tempdir().expect("tempdir failed");
    let signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));
    let (issuer, _) = ProofIssuer::open(dir.path(), signer, [1u8; 16]).expect("open issuer");

    let scope_hash = [0x11; 32];
    let witness = create_valid_witness(DeletionLayer::LsmMemtable, scope_hash, 50);

    let probe = MockStoreProbe {
        gen: 999, // Stale: witness has 100
        write_seq: 10,
        manifest_count: None,
    };

    let req = IssRequest {
        scope: DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: TenantId::try_new(1).expect("tenant"),
        },
        storage_scope: "doc:1".to_string(),
        deleted_keys_hash: scope_hash,
        footprint_hash: [0x33; 32],
        observed_empty: false,
        deleted_after_tx: 50,
        stage: 1,
        supersedes: None,
        assurance: WitAssurance::S0,
        witnesses: vec![witness],
        extra_exclusions: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
    };

    let probe_ref: &dyn WitStoreProbe = &probe;
    let res = issuer.issue(req, &[probe_ref]);
    assert!(res.is_err(), "Stale store generation MUST be rejected");
}

#[test]
fn test_scope_write_seq_deviation_rejected() {
    let dir = tempdir().expect("tempdir failed");
    let signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));
    let (issuer, _) = ProofIssuer::open(dir.path(), signer, [1u8; 16]).expect("open issuer");

    let scope_hash = [0x11; 32];
    let witness = create_valid_witness(DeletionLayer::LsmMemtable, scope_hash, 50);

    let probe = MockStoreProbe {
        gen: 100,
        write_seq: 99, // Deviates: witness has 10
        manifest_count: None,
    };

    let req = IssRequest {
        scope: DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: TenantId::try_new(1).expect("tenant"),
        },
        storage_scope: "doc:1".to_string(),
        deleted_keys_hash: scope_hash,
        footprint_hash: [0x33; 32],
        observed_empty: false,
        deleted_after_tx: 50,
        stage: 1,
        supersedes: None,
        assurance: WitAssurance::S0,
        witnesses: vec![witness],
        extra_exclusions: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
    };

    let probe_ref: &dyn WitStoreProbe = &probe;
    let res = issuer.issue(req, &[probe_ref]);
    assert!(res.is_err(), "Scope write seq deviation MUST be rejected");
}

#[test]
fn test_scope_hash_mismatch_rejected() {
    let dir = tempdir().expect("tempdir failed");
    let signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));
    let (issuer, _) = ProofIssuer::open(dir.path(), signer, [1u8; 16]).expect("open issuer");

    let scope_hash = [0x11; 32];
    let witness = create_valid_witness(DeletionLayer::LsmMemtable, scope_hash, 50);

    let probe = MockStoreProbe {
        gen: 100,
        write_seq: 10,
        manifest_count: None,
    };

    let req = IssRequest {
        scope: DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: TenantId::try_new(1).expect("tenant"),
        },
        storage_scope: "doc:1".to_string(),
        deleted_keys_hash: [0x22; 32], // Mismatch with witness (0x11)
        footprint_hash: [0x33; 32],
        observed_empty: false,
        deleted_after_tx: 50,
        stage: 1,
        supersedes: None,
        assurance: WitAssurance::S0,
        witnesses: vec![witness],
        extra_exclusions: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
    };

    let probe_ref: &dyn WitStoreProbe = &probe;
    let res = issuer.issue(req, &[probe_ref]);
    assert!(res.is_err(), "Scope hash mismatch MUST be rejected");
}

#[test]
fn test_sstable_manifest_file_count_mismatch_rejected() {
    let dir = tempdir().expect("tempdir failed");
    let signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));
    let (issuer, _) = ProofIssuer::open(dir.path(), signer, [1u8; 16]).expect("open issuer");

    let scope_hash = [0x11; 32];
    let witness = create_valid_witness(DeletionLayer::SsTableAllLevels, scope_hash, 50);

    let probe = MockStoreProbe {
        gen: 100,
        write_seq: 10,
        manifest_count: Some(10), // Witness evidence has 5 files_in_manifest
    };

    let req = IssRequest {
        scope: DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: TenantId::try_new(1).expect("tenant"),
        },
        storage_scope: "doc:1".to_string(),
        deleted_keys_hash: scope_hash,
        footprint_hash: [0x33; 32],
        observed_empty: false,
        deleted_after_tx: 50,
        stage: 1,
        supersedes: None,
        assurance: WitAssurance::S2,
        witnesses: vec![witness],
        extra_exclusions: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
    };

    let probe_ref: &dyn WitStoreProbe = &probe;
    let res = issuer.issue(req, &[probe_ref]);
    assert!(res.is_err(), "Manifest file count mismatch MUST be rejected");
}

#[test]
fn test_missing_mandatory_layer_for_assurance_rejected() {
    let dir = tempdir().expect("tempdir failed");
    let signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));
    let (issuer, _) = ProofIssuer::open(dir.path(), signer, [1u8; 16]).expect("open issuer");

    let scope_hash = [0x11; 32];
    // Only Memtable witness provided, but Assurance S1 requires TextIndex, HnswIndex, Relations, ErasureLedger
    let witness = create_valid_witness(DeletionLayer::LsmMemtable, scope_hash, 50);

    let probe = MockStoreProbe {
        gen: 100,
        write_seq: 10,
        manifest_count: None,
    };

    let req = IssRequest {
        scope: DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: TenantId::try_new(1).expect("tenant"),
        },
        storage_scope: "doc:1".to_string(),
        deleted_keys_hash: scope_hash,
        footprint_hash: [0x33; 32],
        observed_empty: false,
        deleted_after_tx: 50,
        stage: 1,
        supersedes: None,
        assurance: WitAssurance::S1, // Requires S1 layers
        witnesses: vec![witness],
        extra_exclusions: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
    };

    let probe_ref: &dyn WitStoreProbe = &probe;
    let res = issuer.issue(req, &[probe_ref]);
    assert!(res.is_err(), "Missing required layers for S1 MUST be rejected");
}

#[test]
fn test_empty_footprint_without_observed_empty_rejected() {
    let dir = tempdir().expect("tempdir failed");
    let signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));
    let (issuer, _) = ProofIssuer::open(dir.path(), signer, [1u8; 16]).expect("open issuer");

    let scope_hash = [0x11; 32];
    let witness = create_valid_witness(DeletionLayer::LsmMemtable, scope_hash, 50);

    let probe = MockStoreProbe {
        gen: 100,
        write_seq: 10,
        manifest_count: None,
    };

    let req = IssRequest {
        scope: DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: TenantId::try_new(1).expect("tenant"),
        },
        storage_scope: "doc:1".to_string(),
        deleted_keys_hash: scope_hash,
        footprint_hash: [0u8; 32], // Empty footprint
        observed_empty: false,      // But observed_empty is false
        deleted_after_tx: 50,
        stage: 1,
        supersedes: None,
        assurance: WitAssurance::S0,
        witnesses: vec![witness],
        extra_exclusions: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
    };

    let probe_ref: &dyn WitStoreProbe = &probe;
    let res = issuer.issue(req, &[probe_ref]);
    assert!(res.is_err(), "Empty footprint without observed_empty MUST be rejected");
}
