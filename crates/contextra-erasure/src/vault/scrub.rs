//! Tresor-Scrubbing und Waisen-Prüfung (SPEC §11.5, D-83, D-111 Abs. 8).
//!
//! Waisenbereinigung:
//! - Nicht beim Open (kein Vollscan, D-111 Abs. 8).
//! - Hintergrundaufgabe `vault_scrub` wandert schrittweise über Slots mit persistiertem Cursor `rsv`.
//! - D-Slot ohne Manifest-Record mit `gen < next_seq - 2^20` wird genullt.
//! - F-Slot ohne Datendatei wird genullt (D-85).

use crate::error::ErasureError;
use crate::vault::slot::SlotType;
use crate::vault::FileVault;

/// Kontext oder Callback-Prüfer für Manifest- und Dateiexistenz.
pub trait VaultScrubContext {
    /// Prüft ob die gegebene Datei existiert (für F-Slots).
    fn file_exists(&self, file_id: &[u8; 16]) -> bool;

    /// Prüft ob ein Dokument noch im Manifest aktiv ist (für D-Slots).
    fn document_in_manifest(&self, doc_tag: &[u8; 16]) -> bool;

    /// Aktuelle nächste Sequenznummer des Stores (next_seq).
    fn next_seq(&self) -> u64;
}

/// Ergebnis eines Scrub-Schritts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScrubOutcome {
    /// Anzahl geprüfter Slots.
    pub slots_scanned: usize,
    /// Anzahl genullter F-Slots (Waisen).
    pub f_orphans_cleaned: usize,
    /// Anzahl genullter D-Slots (Waisen).
    pub d_orphans_cleaned: usize,
    /// Neuer persistierter Cursor `rsv`.
    pub next_cursor: u64,
}

/// Führt einen Scrub-Schritt über bis zu `batch_size` Slots eines Shards durch.
pub fn scrub_shard_step<C: VaultScrubContext>(
    vault: &mut FileVault,
    shard_idx: usize,
    batch_size: usize,
    ctx: &C,
) -> Result<ScrubOutcome, ErasureError> {
    let next_seq = ctx.next_seq();
    let seq_floor = next_seq.saturating_sub(1 << 20);

    let (cap, rsv) = {
        if let Some(durable) = vault.durable_shard(shard_idx) {
            (durable.capacity(), durable.rsv_cursor())
        } else if let Some(in_mem) = vault.shard(shard_idx) {
            (in_mem.capacity(), in_mem.rsv_cursor())
        } else {
            return Err(ErasureError::Corrupt);
        }
    };

    if cap == 0 {
        return Ok(ScrubOutcome::default());
    }

    let start_idx = usize::try_from(rsv % (cap as u64)).unwrap_or(0);
    let mut slots_scanned: usize = 0;
    let mut f_orphans = Vec::new();
    let mut d_orphans = Vec::new();

    for i in 0..batch_size {
        let slot_idx = (start_idx.saturating_add(i)) % cap;
        slots_scanned = slots_scanned.saturating_add(1);

        let slot_data_opt = if let Some(durable) = vault.durable_shard(shard_idx) {
            let raw = durable.read_slot_raw_pub(slot_idx)?;
            crate::vault::slot::decode_slot(&raw, durable.vault_id(), durable.mac_key_ref())?
        } else if let Some(in_mem) = vault.shard(shard_idx) {
            if let Some(raw) = in_mem.slot_bytes_pub(slot_idx) {
                crate::vault::slot::decode_slot(raw, in_mem.vault_id(), in_mem.mac_key_ref())?
            } else {
                None
            }
        } else {
            None
        };

        if let Some(sd) = slot_data_opt {
            match sd.slot_type {
                SlotType::File => {
                    if !ctx.file_exists(&sd.tag) {
                        f_orphans.push(sd.tag);
                    }
                }
                SlotType::Document => {
                    if sd.gen < seq_floor && !ctx.document_in_manifest(&sd.tag) {
                        d_orphans.push(sd.tag);
                    }
                }
                _ => {}
            }
        }
    }

    let next_cursor = rsv.saturating_add(slots_scanned as u64);

    let mut f_cleaned: usize = 0;
    for tag in f_orphans {
        if vault.destroy_slot(SlotType::File, tag).is_ok() {
            f_cleaned = f_cleaned.saturating_add(1);
        }
    }

    let mut d_cleaned: usize = 0;
    for tag in d_orphans {
        if vault.destroy_slot(SlotType::Document, tag).is_ok() {
            d_cleaned = d_cleaned.saturating_add(1);
        }
    }

    if let Some(durable) = vault.durable_shard_mut(shard_idx) {
        durable.set_rsv_cursor(next_cursor);
        durable.flush_meta()?;
    } else if let Some(in_mem) = vault.shard_mut(shard_idx) {
        in_mem.set_rsv_cursor(next_cursor);
    }

    Ok(ScrubOutcome {
        slots_scanned,
        f_orphans_cleaned: f_cleaned,
        d_orphans_cleaned: d_cleaned,
        next_cursor,
    })
}
