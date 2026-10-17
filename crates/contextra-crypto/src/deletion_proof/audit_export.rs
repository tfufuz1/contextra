//! Compliance audit export functionality for deletion proofs.

use super::proof::DeletionProof;
use crate::ed25519_proof::SignatureVersion;
use contextra_types::{ContextraError, Result};

impl DeletionProof {
    /// Exportiert Proof als JSON für Compliance-Dokumentation.
    /// ExcludedScope-Liste ist maschinenlesbar enthalten.
    pub fn export_for_audit(&self) -> Result<String> {
        // Referenz: Befund v17 Teil 10.1 & INVARIANTE INV-DELETION-1
        // Typisierte Versionsprüfung über SignatureVersion.
        let version = self
            .signature_version_typed()
            .map_err(|e| ContextraError::Internal(e.to_string()))?;

        if version == SignatureVersion::V1 {
            let mut clone = self.clone();
            clone.integrity_warning = Some(
                "covered_layers/excluded_scopes are not cryptographically signed in this legacy proof version"
                    .to_string(),
            );
            serde_json::to_string_pretty(&clone)
                .map_err(|e| ContextraError::Internal(e.to_string()))
        } else {
            serde_json::to_string_pretty(self).map_err(|e| ContextraError::Internal(e.to_string()))
        }
    }
}
