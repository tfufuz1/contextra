//! Prüfung von Aufbewahrungspflichten und Restriktionen (SPEC §7.3, §11.2, D-15, ER-19).

use crate::types_local::{RetentionAttestation, RetentionCode};

/// Prüft ob die Attestierung eine aktive Aufbewahrungspflicht anzeigt.
pub fn is_retention_active(att: &RetentionAttestation) -> bool {
    matches!(
        att.code,
        RetentionCode::ObligationActive | RetentionCode::LegalClaimsPending
    )
}

/// Prüft ob der Modus `Restrict` zulässig ist.
pub fn is_restrict_allowed(att: &RetentionAttestation) -> bool {
    matches!(
        att.code,
        RetentionCode::DataSubjectRequestedRestriction
            | RetentionCode::ObligationActive
            | RetentionCode::LegalClaimsPending
            | RetentionCode::OperatorOverride
    )
}
