//! # Manifest-Eintraege (Ring 1)
//!
//! Modelliert Hinzufuegen und Entfernen von SSTables sowie Snapshot-Zustaende.

// FILE-CONTEXT
// STAND: 2026-09-29 (S-01 / T-04)
// ZWECK: Datenstruktur und (De-)Serialisierungslogik für SSTable-Manifest-Einträge (ManifestEntry).
// INVARIANTEN: Binäre Abwärtskompatibilität der Serialisierung; CRC32-Integritätsprüfung für jeden Eintrag.
// HOTSPOTS: to_bytes, from_bytes
// FORMATABKÜNDIGUNG (S-01 / T-04): Der Schreibpfad schreibt kein rank mehr in Replace-Einträge (schreibt stets 0u64 für binäre Nutzlastkompatibilität).
// Der Reader akzeptiert bestehende Einträge mit rank weiterhin und ignoriert das Feld für Sortierung oder Sichtbarkeit.

use contextra_types::{ContextraError, Result};
use std::path::PathBuf;

use super::MAX_MANIFEST_ENTRY_SIZE;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn checked_u32_len(len: usize, field: &str) -> Result<u32> {
    u32::try_from(len).map_err(|_| {
        ContextraError::Serialization(format!("Manifest {field} exceeds u32 length"))
    })
}

fn encode_name(path: &Path, out: &mut Vec<u8>) -> Result<()> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| ContextraError::Serialization("SST name is not valid UTF-8".into()))?;
    if !name.ends_with(".sst") || name.contains('/') || name.contains('\\') {
        return Err(ContextraError::Serialization(format!(
            "Invalid SST name: {name}"
        )));
    }
    let bytes = name.as_bytes();
    let len = u16::try_from(bytes.len())
        .map_err(|_| ContextraError::Serialization("SST name exceeds u16 length".into()))?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(bytes);
    Ok(())
}

fn encode_v2_entry(entry: &ManifestEntry, out: &mut Vec<u8>) -> Result<()> {
    match entry {
        ManifestEntry::Add { path, max_tx } => {
            out.push(1);
            encode_name(path, out)?;
            out.extend_from_slice(&max_tx.to_le_bytes());
        }
        ManifestEntry::Replace { removed, added, added_max_tx, .. } => {
            out.push(2);
            out.extend_from_slice(&checked_u32_len(removed.len(), "removed count")?.to_le_bytes());
            for path in removed {
                encode_name(path, out)?;
            }
            encode_name(added, out)?;
            out.extend_from_slice(&added_max_tx.to_le_bytes());
        }
        ManifestEntry::Remove { path } => {
            out.push(3);
            encode_name(path, out)?;
        }
        ManifestEntry::RollbackComplete { target_tx } => {
            out.push(4);
            out.extend_from_slice(&target_tx.to_le_bytes());
        }
        ManifestEntry::WalCheckpoint { hmac } => {
            out.push(5);
            out.extend_from_slice(hmac);
        }
    }
    Ok(())
}

fn finish_v2_frame(tag: u8, body: Vec<u8>) -> Result<Vec<u8>> {
    let payload_len = body
        .len()
        .checked_add(1)
        .ok_or_else(|| ContextraError::Serialization("Manifest frame length overflow".into()))?;
    let frame_len = payload_len
        .checked_add(4)
        .ok_or_else(|| ContextraError::Serialization("Manifest frame length overflow".into()))?;
    if frame_len > MAX_MANIFEST_ENTRY_SIZE as usize {
        return Err(ContextraError::Serialization("Manifest frame exceeds max size".into()));
    }
    let frame_len = checked_u32_len(frame_len, "frame")?;
    let mut crc_payload = Vec::with_capacity(payload_len);
    crc_payload.push(tag);
    crc_payload.extend_from_slice(&body);
    let crc = crc32fast::hash(&crc_payload);
    let mut frame = Vec::with_capacity(8 + payload_len);
    frame.extend_from_slice(&frame_len.to_le_bytes());
    frame.extend_from_slice(&crc.to_le_bytes());
    frame.extend_from_slice(&crc_payload);
    Ok(frame)
}

pub(crate) fn encode_batch_frame(entries: &[ManifestEntry]) -> Result<Vec<u8>> {
    let mut body = Vec::new();
    body.extend_from_slice(&checked_u32_len(entries.len(), "batch count")?.to_le_bytes());
    for entry in entries {
        encode_v2_entry(entry, &mut body)?;
    }
    finish_v2_frame(1, body)
}

pub(crate) fn encode_snapshot_frame(
    live: &BTreeMap<std::path::PathBuf, u64>,
    dead: &BTreeSet<std::path::PathBuf>,
    rollbacks: &BTreeSet<u64>,
    wal_hwm: Option<[u8; 32]>,
) -> Result<Vec<u8>> {
    let mut body = Vec::new();
    body.extend_from_slice(&checked_u32_len(live.len(), "live count")?.to_le_bytes());
    for (name, max_tx) in live {
        encode_name(name, &mut body)?;
        body.extend_from_slice(&max_tx.to_le_bytes());
    }
    body.extend_from_slice(&checked_u32_len(dead.len(), "dead count")?.to_le_bytes());
    for name in dead {
        encode_name(name, &mut body)?;
    }
    body.extend_from_slice(&checked_u32_len(rollbacks.len(), "rollback count")?.to_le_bytes());
    for target_tx in rollbacks {
        body.extend_from_slice(&target_tx.to_le_bytes());
    }
    match wal_hwm {
        Some(hmac) => {
            body.push(1);
            body.extend_from_slice(&hmac);
        }
        None => body.push(0),
    }
    finish_v2_frame(2, body)
}

fn take<'a>(data: &'a [u8], pos: &mut usize, len: usize) -> Result<&'a [u8]> {
    let end = pos
        .checked_add(len)
        .ok_or_else(|| ContextraError::Serialization("Manifest offset overflow".into()))?;
    let result = data
        .get(*pos..end)
        .ok_or_else(|| ContextraError::Serialization("Truncated v2 manifest frame".into()))?;
    *pos = end;
    Ok(result)
}

fn take_u16(data: &[u8], pos: &mut usize) -> Result<u16> {
    Ok(u16::from_le_bytes(take(data, pos, 2)?.try_into().map_err(|_| {
        ContextraError::Serialization("Invalid u16 in manifest frame".into())
    })?))
}

fn take_u32(data: &[u8], pos: &mut usize) -> Result<u32> {
    Ok(u32::from_le_bytes(take(data, pos, 4)?.try_into().map_err(|_| {
        ContextraError::Serialization("Invalid u32 in manifest frame".into())
    })?))
}

fn take_u64(data: &[u8], pos: &mut usize) -> Result<u64> {
    Ok(u64::from_le_bytes(take(data, pos, 8)?.try_into().map_err(|_| {
        ContextraError::Serialization("Invalid u64 in manifest frame".into())
    })?))
}

fn take_name(data: &[u8], pos: &mut usize) -> Result<std::path::PathBuf> {
    let len = usize::from(take_u16(data, pos)?);
    let bytes = take(data, pos, len)?;
    let name = std::str::from_utf8(bytes)
        .map_err(|error| ContextraError::Serialization(format!("Invalid SST name UTF-8: {error}")))?;
    if !name.ends_with(".sst") || name.contains('/') || name.contains('\\') {
        return Err(ContextraError::Serialization(format!("Invalid SST name: {name}")));
    }
    Ok(std::path::PathBuf::from(name))
}

fn decode_v2_entry(data: &[u8], pos: &mut usize) -> Result<ManifestEntry> {
    let tag = *take(data, pos, 1)?
        .first()
        .ok_or_else(|| ContextraError::Serialization("Missing manifest entry tag".into()))?;
    match tag {
        1 => Ok(ManifestEntry::Add {
            path: take_name(data, pos)?,
            max_tx: take_u64(data, pos)?,
        }),
        2 => {
            let count = usize::try_from(take_u32(data, pos)?).map_err(|_| {
                ContextraError::Serialization("Removed count not representable".into())
            })?;
            if count > data.len().saturating_sub(*pos) / 6 {
                return Err(ContextraError::Serialization(
                    "Removed count exceeds remaining manifest frame".into(),
                ));
            }
            let mut removed = Vec::with_capacity(count);
            for _ in 0..count {
                removed.push(take_name(data, pos)?);
            }
            let added = take_name(data, pos)?;
            let added_max_tx = take_u64(data, pos)?;
            Ok(ManifestEntry::Replace {
                removed,
                added,
                added_max_tx,
                rank: 0,
            })
        }
        3 => Ok(ManifestEntry::Remove {
            path: take_name(data, pos)?,
        }),
        4 => Ok(ManifestEntry::RollbackComplete {
            target_tx: take_u64(data, pos)?,
        }),
        5 => Ok(ManifestEntry::WalCheckpoint {
            hmac: take(data, pos, 32)?.try_into().map_err(|_| {
                ContextraError::Serialization("Invalid WAL HWM size".into())
            })?,
        }),
        _ => Err(ContextraError::Serialization(format!(
            "Unknown v2 manifest entry tag {tag}"
        ))),
    }
}

pub(crate) fn decode_v2_frame(raw: &[u8]) -> Result<Vec<ManifestEntry>> {
    if raw.len() < 5 {
        return Err(ContextraError::Serialization("v2 manifest frame too short".into()));
    }
    let stored_crc = u32::from_le_bytes(raw.get(..4).ok_or_else(|| {
        ContextraError::Serialization("Missing v2 manifest CRC".into())
    })?.try_into().map_err(|_| ContextraError::Serialization("Invalid v2 manifest CRC".into()))?);
    let payload = raw.get(4..).ok_or_else(|| {
        ContextraError::Serialization("Missing v2 manifest payload".into())
    })?;
    if crc32fast::hash(payload) != stored_crc {
        return Err(ContextraError::Serialization("v2 manifest CRC mismatch".into()));
    }
    let mut pos = 1usize;
    let tag = *payload.first().ok_or_else(|| {
        ContextraError::Serialization("Missing v2 manifest frame tag".into())
    })?;
    let mut entries = Vec::new();
    match tag {
        1 => {
            let count = usize::try_from(take_u32(payload, &mut pos)?).map_err(|_| {
                ContextraError::Serialization("Batch count not representable".into())
            })?;
            if count > payload.len().saturating_sub(pos) {
                return Err(ContextraError::Serialization(
                    "Batch count exceeds remaining manifest frame".into(),
                ));
            }
            for _ in 0..count {
                entries.push(decode_v2_entry(payload, &mut pos)?);
            }
        }
        2 => {
            let live_count = usize::try_from(take_u32(payload, &mut pos)?).map_err(|_| {
                ContextraError::Serialization("Live count not representable".into())
            })?;
            if live_count > payload.len().saturating_sub(pos) / 14 {
                return Err(ContextraError::Serialization(
                    "Live count exceeds remaining snapshot frame".into(),
                ));
            }
            for _ in 0..live_count {
                entries.push(ManifestEntry::Add {
                    path: take_name(payload, &mut pos)?,
                    max_tx: take_u64(payload, &mut pos)?,
                });
            }
            let dead_count = usize::try_from(take_u32(payload, &mut pos)?).map_err(|_| {
                ContextraError::Serialization("Dead count not representable".into())
            })?;
            if dead_count > payload.len().saturating_sub(pos) / 6 {
                return Err(ContextraError::Serialization(
                    "Dead count exceeds remaining snapshot frame".into(),
                ));
            }
            for _ in 0..dead_count {
                entries.push(ManifestEntry::Remove {
                    path: take_name(payload, &mut pos)?,
                });
            }
            let rollback_count = usize::try_from(take_u32(payload, &mut pos)?).map_err(|_| {
                ContextraError::Serialization("Rollback count not representable".into())
            })?;
            if rollback_count > payload.len().saturating_sub(pos) / 8 {
                return Err(ContextraError::Serialization(
                    "Rollback count exceeds remaining snapshot frame".into(),
                ));
            }
            for _ in 0..rollback_count {
                entries.push(ManifestEntry::RollbackComplete {
                    target_tx: take_u64(payload, &mut pos)?,
                });
            }
            let has_hwm = *take(payload, &mut pos, 1)?.first().ok_or_else(|| {
                ContextraError::Serialization("Missing snapshot HWM flag".into())
            })?;
            match has_hwm {
                0 => {}
                1 => entries.push(ManifestEntry::WalCheckpoint {
                    hmac: take(payload, &mut pos, 32)?.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid snapshot HWM".into())
                    })?,
                }),
                _ => return Err(ContextraError::Serialization("Invalid snapshot HWM flag".into())),
            }
        }
        _ => return Err(ContextraError::Serialization(format!("Unknown v2 manifest frame tag {tag}"))),
    }
    if pos != payload.len() {
        return Err(ContextraError::Serialization("Trailing bytes in v2 manifest frame".into()));
    }
    Ok(entries)
}

/// An entry in the SSTable manifest log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestEntry {
    /// A new SSTable file was created and fully fsynced.
    Add { path: PathBuf, max_tx: u64 },
    /// An SSTable file was removed or superseded.
    Remove { path: PathBuf },
    /// A transaction rollback operation completed.
    RollbackComplete { target_tx: u64 },
    /// An atomic replacement of multiple input SSTables by a single compacted output SSTable.
    Replace {
        removed: Vec<PathBuf>,
        added: PathBuf,
        added_max_tx: u64,
        /// Existenzprüfung, keine Sortierung
        ///
        /// Dieses Feld dient ausschließlich einer historischen Existenz-/Gültigkeitszähler-Funktion
        /// während des Manifest-Replays (siehe `core::reconstruct_valid_sstables`) und wird NICHT für die
        /// Bestimmung der Lese-Sichtbarkeitsreihenfolge verwendet — diese wird ausschließlich über
        /// `SstableMetadata::max_seq` bestimmt (siehe `lsm/ops/compaction.rs`,
        /// `sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT)`). Zukünftige Wartende dürfen `rank`
        /// NICHT als Sortierkriterium interpretieren oder heranziehen.
        rank: u64,
    },
    /// A WAL high-water-mark entry recording the valid WAL-tail HMAC at flush/checkpoint time.
    WalCheckpoint { hmac: [u8; 32] },
}

impl ManifestEntry {
    /// Serializes the entry into binary format with a 4-byte total size prefix and a 4-byte CRC32.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut payload = Vec::new();
        match self {
            ManifestEntry::Add { path, max_tx } => {
                payload.push(0u8); // op_tag = 0
                payload.extend_from_slice(&max_tx.to_le_bytes());
                let path_str = path.to_string_lossy();
                let path_bytes = path_str.as_bytes();
                payload.extend_from_slice(&checked_u32_len(path_bytes.len(), "path")?.to_le_bytes());
                payload.extend_from_slice(path_bytes);
            }
            ManifestEntry::Remove { path } => {
                payload.push(1u8); // op_tag = 1
                let path_str = path.to_string_lossy();
                let path_bytes = path_str.as_bytes();
                payload.extend_from_slice(&checked_u32_len(path_bytes.len(), "path")?.to_le_bytes());
                payload.extend_from_slice(path_bytes);
            }
            ManifestEntry::RollbackComplete { target_tx } => {
                payload.push(2u8); // op_tag = 2
                payload.extend_from_slice(&target_tx.to_le_bytes());
            }
            ManifestEntry::Replace {
                removed,
                added,
                added_max_tx,
                rank: _,
            } => {
                payload.push(3u8); // op_tag = 3
                payload.extend_from_slice(&added_max_tx.to_le_bytes());
                payload.extend_from_slice(&0u64.to_le_bytes()); // Abgekündigtes rank-Feld (S-01): stets 0u64
                let added_str = added.to_string_lossy();
                let added_bytes = added_str.as_bytes();
                payload.extend_from_slice(&checked_u32_len(added_bytes.len(), "added path")?.to_le_bytes());
                payload.extend_from_slice(added_bytes);
                payload.extend_from_slice(&checked_u32_len(removed.len(), "removed count")?.to_le_bytes());
                for p in removed {
                    let p_str = p.to_string_lossy();
                    let p_bytes = p_str.as_bytes();
                    payload.extend_from_slice(&checked_u32_len(p_bytes.len(), "removed path")?.to_le_bytes());
                    payload.extend_from_slice(p_bytes);
                }
            }
            ManifestEntry::WalCheckpoint { hmac } => {
                payload.push(4u8); // op_tag = 4
                payload.extend_from_slice(hmac);
            }
        }

        let entry_len = payload.len().checked_add(4).ok_or_else(|| {
            ContextraError::Serialization("Manifest entry length overflow".into())
        })?;
        let total_payload_size = checked_u32_len(entry_len, "entry")?;
        if total_payload_size > MAX_MANIFEST_ENTRY_SIZE {
            return Err(ContextraError::Serialization(format!(
                "Manifest entry exceeds max size: {} bytes",
                total_payload_size
            )));
        }

        let crc = crc32fast::hash(&payload);

        let mut buf = Vec::with_capacity(4 + total_payload_size as usize);
        buf.extend_from_slice(&total_payload_size.to_le_bytes());
        buf.extend_from_slice(&crc.to_le_bytes());
        buf.extend_from_slice(&payload);

        Ok(buf)
    }

    /// Deserializes a manifest entry from a slice containing `[crc32: u32 LE] [payload...]`.
    #[deny(clippy::indexing_slicing)]
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 5 {
            return Err(ContextraError::Serialization(
                "Manifest entry too short".into(),
            ));
        }

        let stored_crc_bytes = data.get(0..4).ok_or_else(|| {
            ContextraError::Serialization("Failed to read stored CRC bytes".into())
        })?;
        let stored_crc = u32::from_le_bytes(
            stored_crc_bytes
                .try_into()
                .map_err(|_| ContextraError::Serialization("Invalid CRC format".into()))?,
        );
        let payload = data
            .get(4..)
            .ok_or_else(|| ContextraError::Serialization("Failed to read payload bytes".into()))?;
        let computed_crc = crc32fast::hash(payload);

        if stored_crc != computed_crc {
            return Err(ContextraError::Serialization(format!(
                "CRC mismatch: stored={:#010x}, computed={:#010x}",
                stored_crc, computed_crc
            )));
        }

        let op_tag = *payload
            .first()
            .ok_or_else(|| ContextraError::Serialization("Failed to read op_tag".into()))?;
        let remaining = payload.get(1..).ok_or_else(|| {
            ContextraError::Serialization("Failed to read remaining payload".into())
        })?;

        match op_tag {
            0 => {
                // Add
                if remaining.len() < 12 {
                    return Err(ContextraError::Serialization(
                        "Add payload too short".into(),
                    ));
                }
                let max_tx_bytes = remaining.get(0..8).ok_or_else(|| {
                    ContextraError::Serialization("Add max_tx bytes missing".into())
                })?;
                let max_tx =
                    u64::from_le_bytes(max_tx_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid max_tx format".into())
                    })?);
                let path_len_bytes = remaining.get(8..12).ok_or_else(|| {
                    ContextraError::Serialization("Add path_len bytes missing".into())
                })?;
                let path_len =
                    u32::from_le_bytes(path_len_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid path_len format".into())
                    })?) as usize;
                let expected_len = 12 + path_len;
                if remaining.len() != expected_len {
                    return Err(ContextraError::Serialization(format!(
                        "Add payload length mismatch or trailing unparsed bytes: expected {}, got {}",
                        expected_len,
                        remaining.len()
                    )));
                }
                let path_bytes = remaining.get(12..12 + path_len).ok_or_else(|| {
                    ContextraError::Serialization("Add path bytes missing".into())
                })?;
                let path_str = std::str::from_utf8(path_bytes).map_err(|e| {
                    ContextraError::Serialization(format!("Invalid path UTF-8: {}", e))
                })?;
                Ok(ManifestEntry::Add {
                    path: PathBuf::from(path_str),
                    max_tx,
                })
            }
            1 => {
                // Remove
                if remaining.len() < 4 {
                    return Err(ContextraError::Serialization(
                        "Remove payload too short".into(),
                    ));
                }
                let path_len_bytes = remaining.get(0..4).ok_or_else(|| {
                    ContextraError::Serialization("Remove path_len bytes missing".into())
                })?;
                let path_len =
                    u32::from_le_bytes(path_len_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid path_len format".into())
                    })?) as usize;
                let expected_len = 4 + path_len;
                if remaining.len() != expected_len {
                    return Err(ContextraError::Serialization(format!(
                        "Remove payload length mismatch or trailing unparsed bytes: expected {}, got {}",
                        expected_len,
                        remaining.len()
                    )));
                }
                let path_bytes = remaining.get(4..4 + path_len).ok_or_else(|| {
                    ContextraError::Serialization("Remove path bytes missing".into())
                })?;
                let path_str = std::str::from_utf8(path_bytes).map_err(|e| {
                    ContextraError::Serialization(format!("Invalid path UTF-8: {}", e))
                })?;
                Ok(ManifestEntry::Remove {
                    path: PathBuf::from(path_str),
                })
            }
            2 => {
                // RollbackComplete
                if remaining.len() != 8 {
                    return Err(ContextraError::Serialization(format!(
                        "RollbackComplete payload length mismatch: expected 8, got {}",
                        remaining.len()
                    )));
                }
                let target_tx_bytes = remaining.get(0..8).ok_or_else(|| {
                    ContextraError::Serialization("target_tx bytes missing".into())
                })?;
                let target_tx = u64::from_le_bytes(target_tx_bytes.try_into().map_err(|_| {
                    ContextraError::Serialization("Invalid target_tx format".into())
                })?);
                Ok(ManifestEntry::RollbackComplete { target_tx })
            }
            3 => {
                // Replace
                if remaining.len() < 24 {
                    return Err(ContextraError::Serialization(
                        "Replace payload header too short".into(),
                    ));
                }
                let added_max_tx_bytes = remaining.get(0..8).ok_or_else(|| {
                    ContextraError::Serialization("added_max_tx bytes missing".into())
                })?;
                let added_max_tx =
                    u64::from_le_bytes(added_max_tx_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid added_max_tx format".into())
                    })?);
                let rank_bytes = remaining
                    .get(8..16)
                    .ok_or_else(|| ContextraError::Serialization("rank bytes missing".into()))?;
                let rank =
                    u64::from_le_bytes(rank_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid rank format".into())
                    })?);
                let added_path_len_bytes = remaining.get(16..20).ok_or_else(|| {
                    ContextraError::Serialization("added_path_len bytes missing".into())
                })?;
                let added_path_len =
                    u32::from_le_bytes(added_path_len_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid added_path_len format".into())
                    })?) as usize;
                let mut offset = 20;
                if remaining.len() < offset + added_path_len {
                    return Err(ContextraError::Serialization(
                        "Replace added path data truncated".into(),
                    ));
                }
                let added_path_bytes =
                    remaining
                        .get(offset..offset + added_path_len)
                        .ok_or_else(|| {
                            ContextraError::Serialization("added_path bytes missing".into())
                        })?;
                let added_str = std::str::from_utf8(added_path_bytes).map_err(|e| {
                    ContextraError::Serialization(format!("Invalid added path UTF-8: {}", e))
                })?;
                let added = PathBuf::from(added_str);
                offset += added_path_len;

                if remaining.len() < offset + 4 {
                    return Err(ContextraError::Serialization(
                        "Replace removed count truncated".into(),
                    ));
                }
                let removed_count_bytes = remaining.get(offset..offset + 4).ok_or_else(|| {
                    ContextraError::Serialization("removed_count bytes missing".into())
                })?;
                let removed_count =
                    u32::from_le_bytes(removed_count_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid removed_count format".into())
                    })?) as usize;
                offset += 4;

                let max_possible_removed = remaining.get(offset..).map_or(0, |r| r.len() / 4);
                if removed_count > max_possible_removed {
                    return Err(ContextraError::Serialization(format!(
                        "Invalid removed_count {} exceeds remaining buffer capacity {}",
                        removed_count, max_possible_removed
                    )));
                }

                let mut removed = Vec::with_capacity(removed_count);
                for _ in 0..removed_count {
                    if remaining.len() < offset + 4 {
                        return Err(ContextraError::Serialization(
                            "Replace removed path length truncated".into(),
                        ));
                    }
                    let r_len_bytes = remaining.get(offset..offset + 4).ok_or_else(|| {
                        ContextraError::Serialization("r_len bytes missing".into())
                    })?;
                    let r_len = u32::from_le_bytes(r_len_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid r_len format".into())
                    })?) as usize;
                    offset += 4;
                    if remaining.len() < offset + r_len {
                        return Err(ContextraError::Serialization(
                            "Replace removed path data truncated".into(),
                        ));
                    }
                    let r_bytes = remaining.get(offset..offset + r_len).ok_or_else(|| {
                        ContextraError::Serialization("r_path bytes missing".into())
                    })?;
                    let r_str = std::str::from_utf8(r_bytes).map_err(|e| {
                        ContextraError::Serialization(format!("Invalid removed path UTF-8: {}", e))
                    })?;
                    removed.push(PathBuf::from(r_str));
                    offset += r_len;
                }

                if offset != remaining.len() {
                    return Err(ContextraError::Serialization(format!(
                        "Replace payload trailing unparsed bytes: consumed {}, total {}",
                        offset,
                        remaining.len()
                    )));
                }

                Ok(ManifestEntry::Replace {
                    removed,
                    added,
                    added_max_tx,
                    rank,
                })
            }
            4 => {
                // WalCheckpoint
                if remaining.len() != 32 {
                    return Err(ContextraError::Serialization(format!(
                        "WalCheckpoint payload length mismatch: expected 32, got {}",
                        remaining.len()
                    )));
                }
                let hmac_bytes = remaining.get(0..32).ok_or_else(|| {
                    ContextraError::Serialization("WalCheckpoint hmac bytes missing".into())
                })?;
                let hmac: [u8; 32] = hmac_bytes
                    .try_into()
                    .map_err(|_| ContextraError::Serialization("Invalid hmac format".into()))?;
                Ok(ManifestEntry::WalCheckpoint { hmac })
            }
            _ => Err(ContextraError::Serialization(format!(
                "Unknown Manifest op tag: {}",
                op_tag
            ))),
        }
    }
}
