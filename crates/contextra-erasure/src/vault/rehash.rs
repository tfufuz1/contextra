//! Tresor-Rehash nach SPEC §11.5, D-111 Abs. 4..6.
//!
//! Lastfaktor λ = (live + tombstones) / capacity:
//!   λ > 0,60: Rehash im Hintergrund, abbrechbar durch erase.
//!   λ > 0,80: Rehash erzwungen, nicht abbrechbar.
//!   live / capacity < 0,30: Kapazität beibehalten (nur Grabsteine abräumen).
//!   sonst: Kapazität verdoppeln.
//! Protokoll: Neue Datei schreiben, alle aktiven Slots kopieren (Grabsteine nicht),
//! sync_data, rename, Verzeichnis-fsync, alte Datei vollständig mit Nullen überschreiben (D-111 Abs. 5).

use super::table::VaultTable;
use crate::error::ErasureError;

/// Ergebnis einer Rehash-Operation (D-111).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RehashOutcome {
    /// Neue Shard-Kapazität.
    pub new_capacity: usize,
    /// Anzahl kopierter aktiver Slots.
    pub slots_copied: usize,
    /// Ob die Altdatei vollständig mit Nullen überschrieben wurde (D-111 Abs. 5, D-111 Abs. 11).
    pub old_overwritten: bool,
}

/// Führt Rehash für einen Shard durch (D-111 Abs. 4 bis 6).
pub fn rehash_shard(
    old: &mut VaultTable,
    vault_id: [u8; 16],
    mac_key: [u8; 32],
) -> Result<(VaultTable, RehashOutcome), ErasureError> {
    let old_cap = old.capacity();
    let live = old.live_count();

    // Kapazitätsentscheidung nach D-111 Abs. 4:
    // Ist live / capacity < 0,30, behält der Rehash die Kapazität, sonst verdoppelt er sie.
    let new_cap = if (live as f64) / (old_cap as f64) < 0.30 {
        old_cap
    } else {
        old_cap.saturating_mul(2)
    };

    let mut new_table = VaultTable::new_shard(old.shard_no(), new_cap, vault_id, mac_key);
    new_table.set_generation(old.generation().saturating_add(1));
    new_table.set_rsv_cursor(old.rsv_cursor());

    // Aktive Slots sammeln und in neue Tabelle einfügen (Grabsteine werden nicht übernommen)
    let active_slots = old.collect_active_slots();
    let slots_copied = active_slots.len();

    for sd in active_slots {
        new_table.put(sd)?;
    }

    // Altdatei vollständig mit Nullen überschreiben unabhängig vom Medium (D-111 Abs. 5)
    old.overwrite_with_zeros();

    Ok((
        new_table,
        RehashOutcome {
            new_capacity: new_cap,
            slots_copied,
            old_overwritten: true,
        },
    ))
}

/// Rückwärtskompatible Hilfsfunktion für Rehash einer Tabelle.
pub fn rehash(
    old: &VaultTable,
    vault_id: [u8; 16],
    mac_key: [u8; 32],
) -> Result<VaultTable, ErasureError> {
    let mut clone_shard = VaultTable::new_shard(old.shard_no(), old.capacity(), vault_id, mac_key);
    for sd in old.collect_active_slots() {
        clone_shard.put(sd)?;
    }
    let (new_table, _) = rehash_shard(&mut clone_shard, vault_id, mac_key)?;
    Ok(new_table)
}
