// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: CopyStatus und Re-Exports von contextra-store CopyRegEntry/CopyKind für Checkpoint Kopienregister.
// INVARIANTEN: Zero-Panic, Fail-Closed.

pub use contextra_store::copy_registry::{file_id_for_name, CopyKind, CopyRegEntry};

/// Detailed status of a registered copy for scope / Article 17/19 verification.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CopyStatus {
    /// The copy has been physically removed / unlinked.
    Deleted,
    /// The copy destination or medium is unreachable.
    Unreachable,
    /// The copy is out of system scope (e.g. Art. 19 external recipients, Art. 28 processors).
    OutOfScope { reason: String },
    /// The copy is legally retained under statutory retention obligations.
    Retained { legal_basis: String },
    /// Default status for unknown or unverified copies. Blocks full proof assertions.
    #[default]
    Unknown,
}

impl CopyStatus {
    /// Creates an `OutOfScope` status for external recipients under Art. 19 GDPR.
    #[must_use]
    pub fn art19_recipient(recipient_notice: &str) -> Self {
        CopyStatus::OutOfScope {
            reason: format!(
                "Art. 19 DSGVO: Empfänger-Benachrichtigung erforderlich ({recipient_notice})"
            ),
        }
    }

    /// Creates an `OutOfScope` status for data processors under Art. 28 GDPR.
    #[must_use]
    pub fn art28_processor(processor_notice: &str) -> Self {
        CopyStatus::OutOfScope {
            reason: format!(
                "Art. 28 DSGVO: Auftragsverarbeiter-Weisung erforderlich ({processor_notice})"
            ),
        }
    }

    /// Returns `true` if the status represents verified deletion.
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        matches!(self, CopyStatus::Deleted)
    }

    /// Returns `true` if the status is unknown and blocks full deletion proof assertions.
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        matches!(self, CopyStatus::Unknown)
    }
}
