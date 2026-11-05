//! Tresor-Rehash nach SPEC §11.5, D-111 Abs. 4..6.
//!
//! Lastfaktor λ = (live + tombstones) / capacity:
//!   λ > 0,60: Rehash im Hintergrund, abbrechbar durch erase.
//!   λ > 0,80: Rehash erzwungen, nicht abbrechbar.
//!   live / capacity < 0,30: Kapazität beibehalten (nur Grabsteine abräumen).
//!   sonst: Kapazität verdoppeln.
//! Protokoll: Neue Datei schreiben, alle aktiven Slots kopieren (Grabsteine nicht),
//! sync_data, rename, Verzeichnis-fsync, alte Datei vollständig mit Nullen überschreiben (D-111 Abs. 5).

use super::durable::DurableVaultShard;
use super::table::VaultTable;
use crate::error::ErasureError;
use contextra_ports::vfs::Vfs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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

/// Führt Rehash für einen In-Memory-Shard durch.
pub fn rehash_shard(
    old: &mut VaultTable,
    vault_id: [u8; 16],
    mac_key: [u8; 32],
) -> Result<(VaultTable, RehashOutcome), ErasureError> {
    let old_cap = old.capacity();
    let live = old.live_count();

    let new_cap = if (live as f64) / (old_cap as f64) < 0.30 {
        old_cap
    } else {
        old_cap.saturating_mul(2)
    };

    let mut new_table = VaultTable::new_shard(old.shard_no(), new_cap, vault_id, mac_key);
    new_table.set_generation(old.generation().saturating_add(1));
    new_table.set_rsv_cursor(old.rsv_cursor());

    let active_slots = old.collect_active_slots();
    let slots_copied = active_slots.len();

    for sd in active_slots {
        new_table.put(sd)?;
    }

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

/// Führt Rehash für einen dauerhaften Shard über VFS durch nach SPEC D-111 Abs. 4 bis 6:
/// - Prüft ob Abbruch durch wartendes erase angefordert wurde (`erase_pending`), falls λ <= 0.80.
/// - Schreibt temporäre Datei `<nn>.vlt.new` mit F.1-Kopf und Metablock, kopiert alle aktiven Slots, sync_data.
/// - Rename `<nn>.vlt.new` -> `<nn>.vlt`, sync_dir.
/// - Überschreibt Altdatei über das gehaltene Handle vollständig mit Nullen, sync_data, schließen.
pub fn rehash_durable_shard(
    old: &mut DurableVaultShard,
    vfs: &Arc<dyn Vfs>,
    erase_pending: Option<&AtomicBool>,
) -> Result<(DurableVaultShard, RehashOutcome), ErasureError> {
    let old_cap = old.capacity();
    let live = old.live_count();
    let load_factor = old.load_factor();

    if load_factor <= 0.80 {
        if let Some(pending) = erase_pending {
            if pending.load(Ordering::Acquire) {
                return Err(ErasureError::Refused);
            }
        }
    }

    let new_cap = if (live as f64) / (old_cap as f64) < 0.30 {
        old_cap
    } else {
        old_cap.saturating_mul(2)
    };

    let old_path = old.path().to_path_buf();
    let parent_dir = old_path.parent().ok_or(ErasureError::Corrupt)?;
    let temp_name = format!("{:02x}.vlt.new", old.shard_no());
    let temp_path = parent_dir.join(temp_name);

    if vfs.exists(&temp_path).unwrap_or(false) {
        if let Err(e) = vfs.remove_file(&temp_path) {
            return Err(ErasureError::Io(std::io::Error::other(e.to_string())));
        }
    }

    let mut new_shard = DurableVaultShard::create_file(
        vfs,
        temp_path.clone(),
        old.shard_no(),
        new_cap,
        *old.vault_id(),
        *old.mac_key_ref(),
        0,
    )?;

    new_shard.set_generation(old.generation().saturating_add(1));
    new_shard.set_rsv_cursor(old.rsv_cursor());

    let active_slots = old.collect_active_slots()?;
    let slots_copied = active_slots.len();

    for sd in active_slots {
        new_shard.put(sd)?;
    }

    new_shard.sync_data()?;

    vfs.rename(&temp_path, &old_path).map_err(|e| match e {
        contextra_ports::ContextraError::Io(io_err) => ErasureError::Io(io_err),
        other => ErasureError::Io(std::io::Error::other(other.to_string())),
    })?;

    vfs.sync_dir(parent_dir).map_err(|e| match e {
        contextra_ports::ContextraError::Io(io_err) => ErasureError::Io(io_err),
        other => ErasureError::Io(std::io::Error::other(other.to_string())),
    })?;

    old.overwrite_with_zeros()?;
    new_shard.set_path(old_path);

    Ok((
        new_shard,
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
