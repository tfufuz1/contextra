//! Prüfung und Reservierung von Speicherplatz vor dem Purge-Rewrite (SPEC §11.4, D-65, ER-22).
//!
//! Fehlt Platz → `Pending(DiskReserve)`, Gate bleibt aktiv.
//! `erase` bis `Gated` braucht keinen nennenswerten Platz (ER-06).

use crate::error::ErasureError;

/// Ergebnis der Speicherplatzprüfung.
#[derive(Debug)]
pub enum ReserveCheck {
    /// Ausreichend Speicherplatz vorhanden.
    Sufficient { free_bytes: u64 },
    /// Zu wenig Speicherplatz (→ Pending(DiskReserve)).
    Insufficient { free_bytes: u64, required_bytes: u64 },
}

/// Prüft ob genug Speicherplatz für den Purge-Rewrite vorhanden ist.
///
/// In der Produktionsimplementierung fragt diese Funktion das VFS nach freiem Speicher.
/// Hier: Stub, der immer `Sufficient` zurückgibt (WP-4.2-Implementierung folgt).
pub fn check_disk_reserve(
    min_free_bytes: u64,
    estimated_rewrite_bytes: u64,
) -> Result<ReserveCheck, ErasureError> {
    // Stub: Produktiv via Vfs::available_space()
    let _ = estimated_rewrite_bytes;
    // Konservative Schätzung: Immer ausreichend im Stub
    Ok(ReserveCheck::Sufficient {
        free_bytes: min_free_bytes.saturating_mul(10),
    })
}
