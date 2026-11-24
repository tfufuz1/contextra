//! Contract tests for StorageBridge DeletionProof projection and error classification (SP-BRIDGE, E-77, E-80).

use contextra_storage::erase::receipt::{
    AnchorEvidence, ErasureReceipt, MeasureKind, MeasureReport, MeasureStatus, ProviderAttestation,
    ReceiptStageEntry,
};
use contextra_storage::erase::tickets::{EraseLevel, Residual, TicketStatus};
use contextra_storage_bridge::StorageBridge;
use contextra_storage_ports::traits::{AnchorStrength, AnchorValue};
use contextra_storage_ports::{Error, ErrorClass, Ticket};

#[test]
fn test_bridge_deletion_proof_preserves_caps_and_incomplete_inv_ver_1() {
    let anchor = AnchorEvidence {
        strength: AnchorStrength::Strong,
        value: AnchorValue {
            erase_epoch: 2,
            ledger_head: 9,
            catalog_epoch: 1,
        },
    };
    let mut receipt = ErasureReceipt {
        instance_id: [1u8; 16],
        ticket: Ticket::from_bytes([9u8; 16]),
        scope_digest: [2u8; 32],
        level: EraseLevel::Cryptographic,
        t_requested: 100,
        t_logical: 110,
        erase_epoch: 2,
        t_crypto: Some(120),
        t_physical: Some(130),
        t_verified: Some(140),
        stages: vec![ReceiptStageEntry {
            stage_name: "Completed",
            ledger_seq: 9,
            evidence_digest: [0u8; 32],
        }],
        provider: ProviderAttestation {
            destroyed: true,
            keys_in_backups: Some(9999),
            weak_key_assurance: false,
        },
        measures: vec![
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
        ],
        verifier_report_digest: [0u8; 32],
        ledger_head: [3u8; 32],
        anchor: anchor.clone(),
        restrictions_encountered: false,
        status: TicketStatus::CompleteWithResidual(Residual::BoundedBackupKeys { until_utc: 9999 }),
        residual: Some(Residual::BoundedBackupKeys { until_utc: 9999 }),
        canonical_bytes: Vec::new(),
    };
    receipt.canonical_bytes = receipt.build_canonical_bytes();

    let proof = StorageBridge::project_deletion_proof(&receipt, &[], &anchor).unwrap();
    assert_eq!(proof.version, 15);
    assert_eq!(
        proof.status,
        TicketStatus::CompleteWithResidual(Residual::BoundedBackupKeys { until_utc: 9999 })
    );
    assert_eq!(
        proof.residual,
        Some(Residual::BoundedBackupKeys { until_utc: 9999 })
    );

    // Incomplete receipt remains Incomplete in DeletionProof (INV-VER-1)
    let mut incomplete = receipt.clone();
    incomplete.status = TicketStatus::Incomplete;
    incomplete.residual = None;
    incomplete.measures[0].status = MeasureStatus::NotMeasured;
    incomplete.canonical_bytes = incomplete.build_canonical_bytes();

    let inc_proof = StorageBridge::project_deletion_proof(&incomplete, &[], &anchor).unwrap();
    assert_eq!(inc_proof.status, TicketStatus::Incomplete);
    assert_eq!(
        StorageBridge::classify_error(&Error::Rollback),
        ErrorClass::Rollback
    );
}
