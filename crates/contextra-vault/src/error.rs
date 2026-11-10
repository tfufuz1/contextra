//! Lokaler Fehlertyp für das Vault-Crate.

/// Fehlerzustände des flüchtigen Speicher-Puffers.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum VaultError {
    /// Kapazitätsgrenze im RAM wurde überschritten (Fail-Closed, kein Disk-Spillover).
    #[error("Vault-Kapazität überschritten: {current_bytes} + {new_bytes} > {max_bytes}")]
    CapacityExceeded {
        /// Aktuell belegter Speicher im Vault in Bytes.
        current_bytes: usize,
        /// Größe des neuen Chunks in Bytes.
        new_bytes: usize,
        /// Konfigurierte Höchstgrenze in Bytes.
        max_bytes: usize,
    },
    /// Der Vault wurde bereits geleert oder final übergeben.
    #[error("Vault bereits konsumiert (purge oder commit bereits aufgerufen)")]
    AlreadyConsumed,
}
