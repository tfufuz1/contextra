//! Integration tests for Erase Protocol, Receipt and Offline Verification (SPEC §9.7, §13.2.1, INV-VER-1).

use contextra_crypto_ref::mac::HmacSha256;
use contextra_crypto_ref::mem_key_provider::MemKeyProvider;
use contextra_storage::env::anchor::MemAnchor;
use contextra_storage::env::faultfs::{Fault, FaultFs, FaultPlan};
use contextra_storage::env::memfs::MemFs;
use contextra_storage::env::sim::SimEntropy;
use contextra_storage::erase::cluster_table::{ClusterState, ClusterTable};
use contextra_storage::erase::gate::GateIndex;
use contextra_storage::erase::ledger::{LedgerWriter, MAX_LEDGER_BYTES};
use contextra_storage::erase::links::LinkGraph;
use contextra_storage::erase::protocol::{EraseCoordinator, EraseRequest};
use contextra_storage::erase::receipt::{
    verify_receipt, AnchorEvidence, ErasureReceipt, MeasureKind, MeasureReport, MeasureStatus,
    ProviderAttestation, ReceiptStageEntry, ReceiptVerdict,
};
use contextra_storage::erase::tickets::{
    CompletionEvidence, EraseLevel, EraseScope, Residual, TicketStage, TicketStatus,
};
use contextra_storage_ports::traits::{Anchor, AnchorStrength, AnchorValue, FileRole, Fs, IoCtx};
use contextra_storage_ports::{ClusterId, Key32, KeyProvider, KeyState, Protection, Ticket};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

fn test_ctx() -> IoCtx {
    IoCtx {
        role: FileRole::Ledger,
        purpose: "test",
    }
}

fn all_complete_measures() -> Vec<MeasureReport> {
    vec![
        MeasureReport {
            kind: MeasureKind::NoLiveRunBlobOrRedo,
            status: MeasureStatus::Complete,
        },
        MeasureReport {
            kind: MeasureKind::MemtablesClean,
            status: MeasureStatus::Complete,
        },
        MeasureReport {
            kind: MeasureKind::KeyProbeAbsent,
            status: MeasureStatus::Complete,
        },
        MeasureReport {
            kind: MeasureKind::LedgerChainValid,
            status: MeasureStatus::Complete,
        },
    ]
}

#[test]
fn test_receipt_status_caps_and_measure_kinds() {
    let base = TicketStatus::Complete;
    let provider = ProviderAttestation {
        destroyed: true,
        keys_in_backups: Some(2000),
        weak_key_assurance: false,
    };
    let measures = all_complete_measures();

    // Case 1: Bounded backup keys until 2000 > current 1000
    let (status, residual) = ErasureReceipt::compute_status_caps(
        base,
        EraseLevel::Cryptographic,
        &provider,
        &measures,
        false,
        1000,
    );
    assert_eq!(
        status,
        TicketStatus::CompleteWithResidual(Residual::BoundedBackupKeys { until_utc: 2000 })
    );
    assert_eq!(
        residual,
        Some(Residual::BoundedBackupKeys { until_utc: 2000 })
    );

    // Case 1b: After backup expiration (current 3000 >= 2000), cap disappears
    let (status_expired, res_expired) = ErasureReceipt::compute_status_caps(
        base,
        EraseLevel::Cryptographic,
        &provider,
        &measures,
        false,
        3000,
    );
    assert_eq!(status_expired, TicketStatus::Complete);
    assert_eq!(res_expired, None);

    // Case 2: Logical only level always caps to CompleteWithResidual(LogicalOnly)
    let (status_logical, res_logical) = ErasureReceipt::compute_status_caps(
        base,
        EraseLevel::Logical,
        &provider,
        &measures,
        false,
        3000,
    );
    assert_eq!(
        status_logical,
        TicketStatus::CompleteWithResidual(Residual::LogicalOnly)
    );
    assert_eq!(res_logical, Some(Residual::LogicalOnly));

    // Case 3: Missing measure caps to Incomplete (INV-VER-1)
    let incomplete_measures = vec![
        MeasureReport {
            kind: MeasureKind::NoLiveRunBlobOrRedo,
            status: MeasureStatus::Complete,
        },
        MeasureReport {
            kind: MeasureKind::MemtablesClean,
            status: MeasureStatus::NotMeasured,
        },
    ];
    let (status_inc, res_inc) = ErasureReceipt::compute_status_caps(
        base,
        EraseLevel::Cryptographic,
        &provider,
        &incomplete_measures,
        false,
        3000,
    );
    assert_eq!(status_inc, TicketStatus::Incomplete);
    assert_eq!(res_inc, None);
}

#[test]
fn test_verify_receipt_offline() {
    let anchor_evidence = AnchorEvidence {
        strength: AnchorStrength::Strong,
        value: AnchorValue {
            erase_epoch: 1,
            ledger_head: 5,
            catalog_epoch: 1,
        },
    };

    let mut receipt = ErasureReceipt {
        instance_id: [1u8; 16],
        ticket: Ticket::from_bytes([2u8; 16]),
        scope_digest: [3u8; 32],
        level: EraseLevel::Cryptographic,
        t_requested: 1000,
        t_logical: 1010,
        erase_epoch: 1,
        t_crypto: Some(1020),
        t_physical: Some(1030),
        t_verified: Some(1040),
        stages: vec![ReceiptStageEntry {
            stage_name: "Completed",
            ledger_seq: 5,
            evidence_digest: [0u8; 32],
        }],
        provider: ProviderAttestation {
            destroyed: true,
            keys_in_backups: None,
            weak_key_assurance: false,
        },
        measures: all_complete_measures(),
        verifier_report_digest: [0u8; 32],
        ledger_head: [4u8; 32],
        anchor: anchor_evidence.clone(),
        restrictions_encountered: false,
        status: TicketStatus::Complete,
        residual: None,
        canonical_bytes: Vec::new(),
    };
    receipt.canonical_bytes = receipt.build_canonical_bytes();

    let verdict = verify_receipt(&receipt, &[], &anchor_evidence).expect("verification should succeed");
    assert_eq!(verdict, ReceiptVerdict::Valid { status: TicketStatus::Complete });

    // Tampered canonical bytes
    let mut tampered = receipt.clone();
    tampered.canonical_bytes[10] ^= 0xff;
    let bad_verdict = verify_receipt(&tampered, &[], &anchor_evidence).expect("verify");
    assert_eq!(bad_verdict, ReceiptVerdict::Invalid { reason: "canonical bytes mismatch" });

    // Incomplete measurements on a Complete cryptographic receipt must fail INV-VER-1
    let mut incomplete_receipt = receipt.clone();
    incomplete_receipt.measures[0].status = MeasureStatus::NotMeasured;
    incomplete_receipt.canonical_bytes = incomplete_receipt.build_canonical_bytes();
    let inv_verdict = verify_receipt(&incomplete_receipt, &[], &anchor_evidence).expect("verify");
    assert!(matches!(inv_verdict, ReceiptVerdict::Invalid { .. }));
}

#[test]
fn test_erase_coordinator_lifecycle() {
    let fs = FaultFs::new(
        MemFs::new(),
        FaultPlan {
            at_op: u64::MAX,
            fault: Fault::Eio,
        },
    );
    let ledger_path = Path::new("LEDGER");
    let file = fs.create_new(ledger_path, test_ctx()).unwrap();
    file.preallocate(MAX_LEDGER_BYTES, test_ctx()).unwrap();
    fs.sync_dir(Path::new("."), test_ctx()).ok();

    let mac = HmacSha256;
    let key = Key32::from_array([0x77u8; 32]);
    let (mut writer, _) = LedgerWriter::recover(file.as_ref(), &key, &mac).unwrap();

    let mut coordinator = EraseCoordinator::new();
    let mut clusters = ClusterTable::new();
    let c1 = ClusterId::from_wire([0x11u8; 16]);
    clusters.register(c1, Protection::Sealed).unwrap();

    let links = LinkGraph::default();
    let restrictions = BTreeMap::new();
    let ticket_id = Ticket::from_bytes([0xaa; 16]);

    let req = EraseRequest {
        ticket: ticket_id,
        scope: EraseScope::Cluster,
        protection: Protection::Sealed,
        level: EraseLevel::Cryptographic,
        clusters: vec![c1],
        purge_deadline_utc: 5000,
    };

    // Step 1: Record Intent
    coordinator
        .record_intent(req, &clusters, &links, &restrictions, &mut writer, 100, 1)
        .unwrap();
    assert_eq!(
        coordinator.ticket(ticket_id).unwrap().stage(),
        TicketStage::IntentRecorded
    );

    // Step 2: Apply Gate
    let mut gate_index = GateIndex::new();
    coordinator
        .apply_gate(
            ticket_id,
            &mut clusters,
            &links,
            &restrictions,
            &mut gate_index,
            &mut writer,
            1,
            110,
            &[c1],
            1,
        )
        .unwrap();
    assert!(gate_index.check_visible(c1, 1).is_err());

    // Step 3: Key destruction (Disabled key blocks until Absent)
    let entropy = Arc::new(SimEntropy::new(42));
    let key_provider = MemKeyProvider::new(entropy.clone());
    key_provider.create_cluster_key(c1, entropy.as_ref()).unwrap();
    key_provider.destroy_many(&[c1]).unwrap();
    assert_eq!(key_provider.probe(c1).unwrap(), KeyState::Absent);

    coordinator
        .destroy_keys(ticket_id, &[c1], &key_provider, &mut writer, 120, 1)
        .unwrap();
    assert_eq!(
        coordinator.ticket(ticket_id).unwrap().stage(),
        TicketStage::KeysDestroyed
    );

    // Step 4: Purge start + verified
    coordinator
        .start_purge(ticket_id, &mut writer, 125, 1)
        .unwrap();
    coordinator
        .complete_purge(ticket_id, &mut clusters, &[c1], [0x55; 32], &mut writer, 128, 1)
        .unwrap();
    assert_eq!(clusters.get(c1).unwrap().state, ClusterState::Purged);

    // Step 5: Finalize and verify receipt offline against replayed ledger slice
    let anchor = MemAnchor::default();
    let receipt = coordinator
        .finalize(
            ticket_id,
            &anchor,
            &mut writer,
            130,
            1,
            CompletionEvidence {
                anchor_advanced: true,
                verified: true,
                owners_attested: true,
                graveyard_empty: true,
            },
        )
        .unwrap();

    assert_eq!(receipt.status, TicketStatus::Complete);
    assert!(receipt.canonical_bytes.len() > 32);

    let (_, records) = LedgerWriter::recover(file.as_ref(), &key, &mac).unwrap();
    let anchor_evidence = AnchorEvidence {
        strength: anchor.strength(),
        value: anchor.read().unwrap(),
    };
    let verdict = verify_receipt(&receipt, &records, &anchor_evidence).unwrap();
    assert_eq!(
        verdict,
        ReceiptVerdict::Valid {
            status: TicketStatus::Complete
        }
    );

    // Replay into fresh coordinator produces Done ticket
    let mut replayed = EraseCoordinator::new();
    for rec in &records {
        replayed.recover_from_record(rec).unwrap();
    }
    assert_eq!(
        replayed.ticket(ticket_id).unwrap().stage(),
        TicketStage::Done
    );
}
