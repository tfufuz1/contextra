//! Prüfung und Reservierung von Speicherplatz vor dem Purge-Rewrite (SPEC §11.4, D-65, ER-22).
//!
//! Fehlt Platz → `Pending(DiskReserve)`, Gate bleibt aktiv.
//! `erase` bis `Gated` braucht keinen nennenswerten Platz (ER-06).

use crate::error::ErasureError;

/// Ergebnis der Speicherplatzprüfung.
#[derive(Debug, PartialEq, Eq)]
pub enum ReserveCheck {
    /// Ausreichend Speicherplatz vorhanden.
    Sufficient {
        /// Freie Bytes.
        free_bytes: u64,
    },
    /// Zu wenig Speicherplatz (→ Pending(DiskReserve)).
    Insufficient {
        /// Freie Bytes.
        free_bytes: u64,
        /// Benötigte Bytes.
        required_bytes: u64,
    },
}

/// Prüft ob genug Speicherplatz für den Purge-Rewrite vorhanden ist.
pub fn check_disk_reserve(
    available_bytes: u64,
    required_bytes: u64,
) -> Result<ReserveCheck, ErasureError> {
    if available_bytes >= required_bytes {
        Ok(ReserveCheck::Sufficient {
            free_bytes: available_bytes,
        })
    } else {
        Ok(ReserveCheck::Insufficient {
            free_bytes: available_bytes,
            required_bytes,
        })
    }
}
