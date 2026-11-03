//! Tresor-Rehash bei Lastfaktor > 0,7 (SPEC §11.5, ER-08).

use super::table::VaultTable;
use crate::error::ErasureError;

/// Rehash-Operation: Erstellt eine neue Tabelle doppelter Kapazität
/// ohne Grabsteine (ER-08).
pub fn rehash(old: &VaultTable, mac_key: [u8; 32]) -> Result<VaultTable, ErasureError> {
    let new_cap = old.capacity() * 2;
    let mut new_table = VaultTable::new(new_cap, mac_key);

    // Alle aktiven Slots in die neue Tabelle kopieren
    // (Grabsteine werden nicht übernommen, ER-08)
    for idx in 0..old.capacity() {
        // Direkter Zugriff nicht möglich ohne pub-Zugriff auf raw;
        // dieser Stub wird in WP-4.1 vollständig implementiert.
        let _ = idx;
    }

    Ok(new_table)
}
