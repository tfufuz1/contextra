use contextra_storage::erase::tickets::{
    CompletionEvidence, EraseLevel, EraseScope, EraseTicket, Residual,
    TicketEvidence, TicketStage, TicketStatus,
};
use contextra_storage::ports::{Error, Protection, Ticket};

fn ticket() -> Ticket { Ticket::from_bytes([7; 16]) }

fn complete_evidence() -> CompletionEvidence {
    CompletionEvidence {
        anchor_advanced: true,
        verified: true,
        owners_attested: true,
        graveyard_empty: true,
    }
}

#[test]
fn cryptographic_ticket_requires_gate_keys_and_purge_before_done() {
    let mut erase = EraseTicket::new(ticket(), EraseScope::Cluster, Protection::Sealed, EraseLevel::Cryptographic).unwrap();
    assert!(matches!(erase.apply(1, TicketEvidence::Completed { evidence: complete_evidence(), residual: None }), Err(Error::Corrupt { .. })));
    erase.apply(1, TicketEvidence::GateApplied { stamp: 9 }).unwrap();
    erase.apply(2, TicketEvidence::KeyDestroyIntent).unwrap();
    erase.apply(3, TicketEvidence::KeysDestroyed).unwrap();
    erase.apply(4, TicketEvidence::PurgeStarted).unwrap();
    erase.apply(5, TicketEvidence::Purged).unwrap();
    erase.apply(6, TicketEvidence::Completed { evidence: complete_evidence(), residual: None }).unwrap();
    assert_eq!(erase.stage(), TicketStage::Done);
    assert_eq!(erase.status(), TicketStatus::Complete);
}

#[test]
fn logical_cluster_completion_carries_logical_only_residual() {
    let mut erase = EraseTicket::new(ticket(), EraseScope::Cluster, Protection::Plain, EraseLevel::Logical).unwrap();
    erase.apply(1, TicketEvidence::GateApplied { stamp: 3 }).unwrap();
    erase.apply(2, TicketEvidence::Completed { evidence: complete_evidence(), residual: None }).unwrap();
    assert_eq!(erase.status(), TicketStatus::CompleteWithResidual(Residual::LogicalOnly));
}

#[test]
fn incomplete_evidence_never_claims_completion() {
    let mut erase = EraseTicket::new(ticket(), EraseScope::Range, Protection::Plain, EraseLevel::Logical).unwrap();
    erase.apply(1, TicketEvidence::GateApplied { stamp: 1 }).unwrap();
    erase.apply(2, TicketEvidence::Completed {
        evidence: CompletionEvidence { verified: true, ..CompletionEvidence::default() },
        residual: None,
    }).unwrap();
    assert_eq!(erase.stage(), TicketStage::Gated);
    assert_eq!(erase.status(), TicketStatus::Incomplete);
}

#[test]
fn sealed_ranges_cannot_request_physical_or_crypto_erasure() {
    assert!(matches!(
        EraseTicket::new(ticket(), EraseScope::Range, Protection::Sealed, EraseLevel::Physical),
        Err(Error::Unsupported { .. })
    ));
}
