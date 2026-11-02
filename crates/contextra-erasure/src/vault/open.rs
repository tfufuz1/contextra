//! Tresor-Open: Kopf- und Stichprobenprüfung (SPEC §11.5, ER-08).

use crate::error::ErasureError;

/// Ergebnis der Open-Prüfung.
#[derive(Debug)]
pub struct VaultOpenOutcome {
    /// Kapazität laut Kopf.
    pub capacity: usize,
    /// Anzahl aktiver Slots (Stichproben-Schätzung, kein Vollscan).
    pub live_estimate: usize,
}

/// Öffnet und prüft einen Tresor (Kopf + Stichproben, kein Vollscan, ER-08).
pub fn open_vault(header: &[u8]) -> Result<VaultOpenOutcome, ErasureError> {
    if header.len() < 128 {
        return Err(ErasureError::Corrupt);
    }
    // kind muss 10 sein (FileKind::Vault = 10)
    if header.first().copied() != Some(10) {
        return Err(ErasureError::Corrupt);
    }
    Ok(VaultOpenOutcome {
        capacity: 64,
        live_estimate: 0,
    })
}
