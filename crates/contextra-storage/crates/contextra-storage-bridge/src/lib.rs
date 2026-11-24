//! Storage Bridge: Adapter for synchronous and legacy callers (SP-BRIDGE 18, SPEC §13).
#![forbid(unsafe_code)]

use contextra_storage::erase::receipt::{
    ErasureReceipt, MeasureReport, MeasureStatus, ReceiptVerdict, verify_receipt,
};
use contextra_storage::erase::tickets::{Residual, TicketStatus};
use contextra_storage::Storage;
use contextra_storage_ports::{Error, ErrorClass, Ticket};

/// Versioned v15 DeletionProof projected from canonical R8.2/R9.1 ErasureReceipt.
/// Caps and incomplete measurements are preserved verbatim and never upgraded (E-77, E-80).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletionProof {
    pub version: u16,
    pub ticket: Ticket,
    pub status: TicketStatus,
    pub residual: Option<Residual>,
    pub measures: Vec<MeasureReport>,
    pub canonical_receipt_bytes: Vec<u8>,
}

pub struct StorageBridge {
    storage: Storage,
}

impl StorageBridge {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn inner(&self) -> &Storage {
        &self.storage
    }

    pub fn catalog_state(&self) -> &contextra_storage::catalog::state::CatalogState {
        self.storage.catalog()
    }

    pub fn check_healthy(&self) -> Result<(), Error> {
        Ok(())
    }

    /// Projects an offline-verified `ErasureReceipt` into a v15 `DeletionProof`.
    /// Preserves all residual caps and enforces `INV-VER-1` (no Incomplete -> Complete upgrade).
    pub fn project_deletion_proof(
        receipt: &ErasureReceipt,
        ledger_slice: &[contextra_storage::erase::ledger::LedgerRecord],
        anchor: &contextra_storage::erase::receipt::AnchorEvidence,
    ) -> Result<DeletionProof, Error> {
        let verdict = verify_receipt(receipt, ledger_slice, anchor)?;
        let status = match verdict {
            ReceiptVerdict::Valid { status } => status,
            ReceiptVerdict::Invalid { .. } => {
                return Err(Error::Corrupt {
                    place: "bridge deletion proof receipt invalid",
                })
            }
        };

        let effective_status = if receipt
            .measures
            .iter()
            .any(|m| m.status != MeasureStatus::Complete)
        {
            TicketStatus::Incomplete
        } else {
            status
        };

        Ok(DeletionProof {
            version: 15,
            ticket: receipt.ticket,
            status: effective_status,
            residual: receipt.residual,
            measures: receipt.measures.clone(),
            canonical_receipt_bytes: receipt.canonical_bytes.clone(),
        })
    }

    /// Maps core error classes cleanly without stripping semantic categories.
    pub fn classify_error(err: &Error) -> ErrorClass {
        err.class()
    }
}
