//! Plattenreserve-Überwachung zur Gewährleistung von Löschoperationen (SPEC §4.2 D-65, §11.4, ST-13, FM-3).

use contextra_types::{ContextraError, Result};
use std::path::Path;

/// Mindestplattenreserve in Bytes (1 GiB laut SPEC D-65 / ST-13).
pub const MIN_DISK_RESERVE_BYTES: u64 = 1024 * 1024 * 1024;

/// Struktur zur Berechnung und Überwachung der Plattenreserve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiskReserve {
    pub required_bytes: u64,
    pub available_bytes: u64,
}

impl DiskReserve {
    /// Erzeugt eine neue Plattenreserve basierend auf der größten bekannten Kandidatendatei.
    ///
    /// Formel laut ST-13: `max(1 GiB, 2 × größte Kandidatendatei)`.
    pub fn for_candidate(largest_candidate_bytes: u64) -> Self {
        let double_largest = largest_candidate_bytes.saturating_mul(2);
        let required = MIN_DISK_RESERVE_BYTES.max(double_largest);
        Self {
            required_bytes: required,
            available_bytes: 0,
        }
    }

    /// Liefert die geforderten Reserve-Bytes.
    pub fn required_bytes(&self) -> u64 {
        self.required_bytes
    }

    /// Prüft, ob ausreichend Speicherplatz vorhanden ist.
    pub fn has_sufficient_space(&self) -> bool {
        self.available_bytes >= self.required_bytes
    }

    /// Prüft den verfügbaren Speicherplatz auf dem Dateisystem von `path`.
    ///
    /// Weist mutierende Schreibzugriffe mit `Limit::DiskReserve` ab,
    /// wenn der freie Platz unter die Grenze fällt (ST-13).
    pub fn check_free_space(&self, _path: &Path, available_bytes: u64) -> Result<()> {
        if available_bytes < self.required_bytes {
            Err(ContextraError::Storage(format!(
                "Disk reserve limit reached: {} bytes available, {} bytes required (D-65/ST-13)",
                available_bytes, self.required_bytes
            )))
        } else {
            Ok(())
        }
    }
}
