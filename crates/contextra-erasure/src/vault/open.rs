//! Tresor-Open: Kopf-, Metablock- und Stichprobenprüfung (SPEC §11.5, D-111 Abs. 8, F.3, ER-08).
//!
//! Ablauf Open nach D-111 Abs. 8:
//! 16 Köpfe und Metablöcke lesen und prüfen (`vault_id`, MAC);
//! je Shard 256 zufällige Slots auf MAC prüfen.
//! **Kein Vollscan.**
//! Metablock (Offset 64, 64 Byte):
//!   0..16:  vault_id (16 B)
//!   16:     shard_no (1 B)
//!   17:     cap_log2 (1 B)
//!   18..26: live (u64 be, 8 B)
//!   26..34: tombstones (u64 be, 8 B)
//!   34..42: generation (u64 be, 8 B)
//!   42..50: rsv (u64 be, 8 B, Cursor für vault_scrub)
//!   50..58: mac (8 B, HMAC-SHA256(K_vm, Bytes[0..50))[0..8])
//!   58..64: pad (6 B, 0)

use crate::error::ErasureError;
use crate::vault::slot::{decode_slot, VAULT_SLOT_SIZE};
use crate::vault::table::VAULT_SLOT_BASE;
use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Größe des Metablocks nach SPEC F.3.
pub const VAULT_METABLOCK_SIZE: usize = 64;

/// Ergebnis der Open-Prüfung für einen Tresor-Shard (D-111 Abs. 8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultOpenOutcome {
    /// Vault-ID laut Metablock.
    pub vault_id: [u8; 16],
    /// Shard-Nummer (0..15).
    pub shard_no: u8,
    /// Kapazität (2^cap_log2).
    pub capacity: usize,
    /// Anzahl aktiver Slots (Hinweiszähler aus Metablock).
    pub live_estimate: u64,
    /// Anzahl Grabsteine (Hinweiszähler aus Metablock).
    pub tombstone_estimate: u64,
    /// Generation.
    pub generation: u64,
    /// Persistierter Cursor für `vault_scrub` (Feld `rsv`).
    pub scrub_cursor: u64,
}

/// Berechnet den Metablock-MAC nach SPEC F.3:
/// `HMAC-SHA256(K_vm, Bytes[0..50))[0..8]`.
pub fn compute_metablock_mac(mac_key: &[u8; 32], meta_pre_mac: &[u8]) -> [u8; 8] {
    let mut out = [0u8; 8];
    if let Ok(mut hmac) = <Hmac<Sha256> as Mac>::new_from_slice(mac_key) {
        <Hmac<Sha256> as Mac>::update(&mut hmac, meta_pre_mac);
        let full = <Hmac<Sha256> as Mac>::finalize(hmac).into_bytes();
        if let (Some(src), Some(dest)) = (full.get(..8), out.get_mut(..8)) {
            dest.copy_from_slice(src);
        }
    }
    out
}

/// Parameter für einen Metablock (64 Bytes) nach SPEC F.3.
#[derive(Debug, Clone, Copy)]
pub struct MetablockParams<'a> {
    /// 16-Byte Tresor-ID.
    pub vault_id: &'a [u8; 16],
    /// Shard-Nummer (0..15).
    pub shard_no: u8,
    /// Zweierlogarithmus der Shard-Kapazität.
    pub cap_log2: u8,
    /// Anzahl lebender Einträge.
    pub live: u64,
    /// Anzahl Grabsteine.
    pub tombstones: u64,
    /// Shard-Generation.
    pub generation: u64,
    /// Scrub-Cursor.
    pub scrub_cursor: u64,
    /// 32-Byte MAC-Schlüssel.
    pub mac_key: &'a [u8; 32],
}

/// Encodiert einen Metablock (64 Bytes) nach SPEC F.3.
pub fn encode_metablock(params: MetablockParams<'_>) -> [u8; VAULT_METABLOCK_SIZE] {
    let vault_id = params.vault_id;
    let shard_no = params.shard_no;
    let cap_log2 = params.cap_log2;
    let live = params.live;
    let tombstones = params.tombstones;
    let generation = params.generation;
    let scrub_cursor = params.scrub_cursor;
    let mac_key = params.mac_key;
    let mut buf = [0u8; VAULT_METABLOCK_SIZE];
    if let Some(dest) = buf.get_mut(0..16) {
        dest.copy_from_slice(vault_id);
    }
    buf[16] = shard_no;
    buf[17] = cap_log2;
    if let Some(dest) = buf.get_mut(18..26) {
        dest.copy_from_slice(&live.to_be_bytes());
    }
    if let Some(dest) = buf.get_mut(26..34) {
        dest.copy_from_slice(&tombstones.to_be_bytes());
    }
    if let Some(dest) = buf.get_mut(34..42) {
        dest.copy_from_slice(&generation.to_be_bytes());
    }
    if let Some(dest) = buf.get_mut(42..50) {
        dest.copy_from_slice(&scrub_cursor.to_be_bytes());
    }
    let pre = &buf[0..50];
    let mac = compute_metablock_mac(mac_key, pre);
    if let Some(dest) = buf.get_mut(50..58) {
        dest.copy_from_slice(&mac);
    }
    // 58..64 bleibt 0 (pad)
    buf
}

/// Öffnet und prüft einen Tresor-Shard (Kopf + Metablock + Stichprobe, kein Vollscan, D-111 Abs. 8).
pub fn open_vault_shard(
    raw_shard: &[u8],
    expected_vault_id: &[u8; 16],
    mac_key: &[u8; 32],
) -> Result<VaultOpenOutcome, ErasureError> {
    if raw_shard.len() < VAULT_SLOT_BASE {
        return Err(ErasureError::Corrupt);
    }

    // 1. F.1-Kopf prüfen (Offset 0..64): kind muss 10 sein (FileKind::Vault)
    if raw_shard.first().copied() != Some(10) {
        return Err(ErasureError::Corrupt);
    }

    // 2. Metablock prüfen (Offset 64..128)
    let meta = raw_shard.get(64..128).ok_or(ErasureError::Corrupt)?;

    // MAC des Metablocks prüfen (über die ersten 50 Bytes)
    let pre = meta.get(0..50).ok_or(ErasureError::Corrupt)?;
    let expected_mac = compute_metablock_mac(mac_key, pre);
    let actual_mac = meta.get(50..58).ok_or(ErasureError::Corrupt)?;
    if actual_mac != expected_mac {
        return Err(ErasureError::Corrupt);
    }

    // Padding muss 0 sein
    let pad = meta.get(58..64).ok_or(ErasureError::Corrupt)?;
    if pad.iter().any(|&b| b != 0) {
        return Err(ErasureError::Corrupt);
    }

    // vault_id binden (D-112 Abs. 2)
    let mut vault_id = [0u8; 16];
    if let (Some(dest), Some(src)) = (vault_id.get_mut(..), meta.get(0..16)) {
        dest.copy_from_slice(src);
    }
    if &vault_id != expected_vault_id {
        return Err(ErasureError::Corrupt);
    }

    let shard_no = meta.get(16).copied().unwrap_or(0);
    let cap_log2 = meta.get(17).copied().unwrap_or(0);
    if cap_log2 > 40 {
        return Err(ErasureError::Corrupt);
    }
    let capacity = 1usize
        .checked_shl(cap_log2 as u32)
        .ok_or(ErasureError::Corrupt)?;

    let mut live_bytes = [0u8; 8];
    if let (Some(dest), Some(src)) = (live_bytes.get_mut(..), meta.get(18..26)) {
        dest.copy_from_slice(src);
    }
    let live_estimate = u64::from_be_bytes(live_bytes);

    let mut tomb_bytes = [0u8; 8];
    if let (Some(dest), Some(src)) = (tomb_bytes.get_mut(..), meta.get(26..34)) {
        dest.copy_from_slice(src);
    }
    let tombstone_estimate = u64::from_be_bytes(tomb_bytes);

    let mut gen_bytes = [0u8; 8];
    if let (Some(dest), Some(src)) = (gen_bytes.get_mut(..), meta.get(34..42)) {
        dest.copy_from_slice(src);
    }
    let generation = u64::from_be_bytes(gen_bytes);

    let mut rsv_bytes = [0u8; 8];
    if let (Some(dest), Some(src)) = (rsv_bytes.get_mut(..), meta.get(42..50)) {
        dest.copy_from_slice(src);
    }
    let scrub_cursor = u64::from_be_bytes(rsv_bytes);

    // 3. Stichprobe von Slots prüfen (D-111 Abs. 8: bis zu 256 Slots prüfen)
    let samples = capacity.min(256);
    let step = (capacity / samples).max(1);
    for s in 0..samples {
        let slot_idx = (s.saturating_mul(step)) % capacity;
        let off = VAULT_SLOT_BASE.saturating_add(slot_idx.saturating_mul(VAULT_SLOT_SIZE));
        if off.saturating_add(VAULT_SLOT_SIZE) <= raw_shard.len() {
            if let Some(slot_slice) = raw_shard.get(off..off + VAULT_SLOT_SIZE) {
                if let Ok(slot_arr) = slot_slice.try_into() {
                    // Decodiert und prüft MAC (aktive Slots und Grabsteine)
                    decode_slot(slot_arr, expected_vault_id, mac_key)?;
                }
            }
        }
    }

    Ok(VaultOpenOutcome {
        vault_id,
        shard_no,
        capacity,
        live_estimate,
        tombstone_estimate,
        generation,
        scrub_cursor,
    })
}

/// Abwärtskompatible Hilfsfunktion für Header-Öffnung.
pub fn open_vault(header: &[u8]) -> Result<VaultOpenOutcome, ErasureError> {
    if header.len() < VAULT_SLOT_BASE {
        return Err(ErasureError::Corrupt);
    }
    if header.first().copied() != Some(10) {
        return Err(ErasureError::Corrupt);
    }
    Ok(VaultOpenOutcome {
        vault_id: [0u8; 16],
        shard_no: 0,
        capacity: 64,
        live_estimate: 0,
        tombstone_estimate: 0,
        generation: 0,
        scrub_cursor: 0,
    })
}
