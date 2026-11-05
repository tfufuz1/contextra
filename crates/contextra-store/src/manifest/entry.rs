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

use contextra_ports::keys::FileId;
use contextra_types::{ContextraError, Result};
use std::path::PathBuf;

use super::MAX_MANIFEST_ENTRY_SIZE;

/// Op-Tag für WalFloor (SPEC F.4: 0x03).
pub const OP_TAG_WAL_FLOOR: u8 = 0x03;

/// Op-Tag für TextCommit (SPEC F.4: 0x06).
pub const OP_TAG_TEXT_COMMIT: u8 = 0x06;

/// Op-Tag für atomaren SSTable-Tausch (ADR-Vorschlag SL-1 aus P02).
///
/// SPEC-Änderung SL-1 ausstehend. Intern gehaltener Op-Tag 0x07 für einen
/// unteilbaren Tausch von hinzugefügten und entfernten SSTables in einem Eintrag.
pub const OP_TAG_SST_SWAP: u8 = 0x07;

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
        rank: u64,
    },
    /// A WAL high-water-mark entry recording the valid WAL-tail HMAC at flush/checkpoint time.
    WalCheckpoint { hmac: [u8; 32] },
    /// WAL-Floor-Eintrag (Typ 0x03, F.4): markiert dauerhaft geflushte WAL-Sequenz und entfernte Segmente.
    WalFloor {
        seq: u64,
        tail_mac: [u8; 32],
        removed_file_ids: Vec<FileId>,
    },
    /// Text-Commit-Eintrag (Typ 0x06, F.4 / D-121): Textsegment-Verwaltung und Floor-Punkt.
    TextCommit {
        add: Option<FileId>,
        remove: Vec<FileId>,
        floor_seq: u64,
    },
    /// Atomarer SSTable-Tausch (Typ 0x07, ADR-Vorschlag SL-1 aus P02).
    ///
    /// SPEC-Änderung SL-1 ausstehend: Verhindert das Zerreißen getrennter Add/Remove-Einträge bei Torn-Writes.
    SstSwap {
        add: Vec<PathBuf>,
        remove: Vec<PathBuf>,
    },
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
                let path_len = u32::try_from(path_bytes.len()).unwrap_or(u32::MAX);
                payload.extend_from_slice(&path_len.to_le_bytes());
                payload.extend_from_slice(path_bytes);
            }
            ManifestEntry::Remove { path } => {
                payload.push(1u8); // op_tag = 1
                let path_str = path.to_string_lossy();
                let path_bytes = path_str.as_bytes();
                let path_len = u32::try_from(path_bytes.len()).unwrap_or(u32::MAX);
                payload.extend_from_slice(&path_len.to_le_bytes());
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
                let added_len = u32::try_from(added_bytes.len()).unwrap_or(u32::MAX);
                payload.extend_from_slice(&added_len.to_le_bytes());
                payload.extend_from_slice(added_bytes);
                let removed_len = u32::try_from(removed.len()).unwrap_or(u32::MAX);
                payload.extend_from_slice(&removed_len.to_le_bytes());
                for p in removed {
                    let p_str = p.to_string_lossy();
                    let p_bytes = p_str.as_bytes();
                    let p_len = u32::try_from(p_bytes.len()).unwrap_or(u32::MAX);
                    payload.extend_from_slice(&p_len.to_le_bytes());
                    payload.extend_from_slice(p_bytes);
                }
            }
            ManifestEntry::WalCheckpoint { hmac } => {
                payload.push(4u8); // op_tag = 4
                payload.extend_from_slice(hmac);
            }
            ManifestEntry::WalFloor {
                seq,
                tail_mac,
                removed_file_ids,
            } => {
                payload.push(OP_TAG_WAL_FLOOR); // op_tag = 0x03
                payload.extend_from_slice(&seq.to_le_bytes());
                payload.extend_from_slice(tail_mac);
                let count = u32::try_from(removed_file_ids.len()).unwrap_or(u32::MAX);
                payload.extend_from_slice(&count.to_le_bytes());
                for fid in removed_file_ids {
                    payload.extend_from_slice(&fid.as_u64().to_le_bytes());
                }
            }
            ManifestEntry::TextCommit {
                add,
                remove,
                floor_seq,
            } => {
                payload.push(OP_TAG_TEXT_COMMIT); // op_tag = 0x06
                payload.extend_from_slice(&floor_seq.to_le_bytes());
                match add {
                    Some(fid) => {
                        payload.push(1u8);
                        payload.extend_from_slice(&fid.as_u64().to_le_bytes());
                    }
                    None => {
                        payload.push(0u8);
                        payload.extend_from_slice(&0u64.to_le_bytes());
                    }
                }
                let count = u32::try_from(remove.len()).unwrap_or(u32::MAX);
                payload.extend_from_slice(&count.to_le_bytes());
                for fid in remove {
                    payload.extend_from_slice(&fid.as_u64().to_le_bytes());
                }
            }
            ManifestEntry::SstSwap { add, remove } => {
                payload.push(OP_TAG_SST_SWAP); // op_tag = 0x07 (SPEC-Änderung SL-1 ausstehend)
                let add_count = u32::try_from(add.len()).unwrap_or(u32::MAX);
                payload.extend_from_slice(&add_count.to_le_bytes());
                for p in add {
                    let p_str = p.to_string_lossy();
                    let p_bytes = p_str.as_bytes();
                    let p_len = u32::try_from(p_bytes.len()).unwrap_or(u32::MAX);
                    payload.extend_from_slice(&p_len.to_le_bytes());
                    payload.extend_from_slice(p_bytes);
                }
                let count = u32::try_from(remove.len()).unwrap_or(u32::MAX);
                payload.extend_from_slice(&count.to_le_bytes());
                for p in remove {
                    let p_str = p.to_string_lossy();
                    let p_bytes = p_str.as_bytes();
                    let p_len = u32::try_from(p_bytes.len()).unwrap_or(u32::MAX);
                    payload.extend_from_slice(&p_len.to_le_bytes());
                    payload.extend_from_slice(p_bytes);
                }
            }
        }

        let total_payload_size = u32::try_from(4 + payload.len()).unwrap_or(u32::MAX);
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
            ContextraError::Serialization("Stored CRC bytes slice missing".into())
        })?;
        let stored_crc = u32::from_le_bytes(
            stored_crc_bytes
                .try_into()
                .map_err(|_| ContextraError::Serialization("Invalid stored CRC format".into()))?,
        );

        let payload = data
            .get(4..)
            .ok_or_else(|| ContextraError::Serialization("Payload missing".into()))?;
        let computed_crc = crc32fast::hash(payload);

        if stored_crc != computed_crc {
            return Err(ContextraError::Serialization(format!(
                "CRC mismatch in ManifestEntry: stored {:x}, computed {:x}",
                stored_crc, computed_crc
            )));
        }

        let op_tag = *payload
            .first()
            .ok_or_else(|| ContextraError::Serialization("Empty payload".into()))?;
        let remaining = payload
            .get(1..)
            .ok_or_else(|| ContextraError::Serialization("Corrupted payload".into()))?;

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
                // Op-Tag 3: Entweder WalFloor (SPEC F.4: 0x03) oder Replace (Legacy-Manifest-Eintrag)
                // WalFloor hat ein fixes Layout: seq (8 B) || tail_mac (32 B) || count (4 B) || count * 8 B.
                // Replace hat: added_max_tx (8 B) || rank (8 B, stets 0) || added_path_len (4 B) ...
                if remaining.len() >= 44 && (remaining.len() - 44) % 8 == 0 {
                    let count_bytes = remaining.get(40..44).ok_or_else(|| {
                        ContextraError::Serialization("WalFloor count missing".into())
                    })?;
                    let count = u32::from_le_bytes(count_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid WalFloor count format".into())
                    })?) as usize;
                    if remaining.len() == 44 + count * 8 {
                        let rank_check = remaining.get(8..16);
                        // Falls Bytes 8..16 nicht alles Nullen sind, ist es eindeutig WalFloor (tail_mac HMAC)
                        if rank_check != Some(&[0u8; 8]) {
                            return Self::parse_wal_floor(remaining);
                        }
                        // Falls Bytes 8..16 zufällig null sind, versuchen wir Replace; schlägt Replace fehl, parsen wir WalFloor
                        if let Ok(replace) = Self::parse_replace(remaining) {
                            return Ok(replace);
                        } else {
                            return Self::parse_wal_floor(remaining);
                        }
                    }
                }

                Self::parse_replace(remaining)
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
            OP_TAG_TEXT_COMMIT => {
                // 0x06: TextCommit { floor_seq: u64, has_add: u8, add: u64, remove_count: u32, remove: [u64] }
                if remaining.len() < 21 {
                    return Err(ContextraError::Serialization(
                        "TextCommit payload too short".into(),
                    ));
                }
                let floor_bytes = remaining.get(0..8).ok_or_else(|| {
                    ContextraError::Serialization("floor_seq bytes missing".into())
                })?;
                let floor_seq = u64::from_le_bytes(floor_bytes.try_into().map_err(|_| {
                    ContextraError::Serialization("Invalid floor_seq format".into())
                })?);

                let has_add = *remaining
                    .get(8)
                    .ok_or_else(|| ContextraError::Serialization("has_add missing".into()))?;
                let add_bytes = remaining
                    .get(9..17)
                    .ok_or_else(|| ContextraError::Serialization("add_fid bytes missing".into()))?;
                let add_val =
                    u64::from_le_bytes(add_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid add_fid format".into())
                    })?);
                let add = if has_add != 0 {
                    Some(FileId::new(add_val))
                } else {
                    None
                };

                let rem_cnt_bytes = remaining.get(17..21).ok_or_else(|| {
                    ContextraError::Serialization("remove_count bytes missing".into())
                })?;
                let rem_cnt = u32::from_le_bytes(rem_cnt_bytes.try_into().map_err(|_| {
                    ContextraError::Serialization("Invalid remove_count format".into())
                })?) as usize;

                let expected_len = rem_cnt
                    .checked_mul(8)
                    .and_then(|v| v.checked_add(21))
                    .ok_or_else(|| {
                        ContextraError::Serialization("TextCommit size overflow".into())
                    })?;
                if remaining.len() != expected_len {
                    return Err(ContextraError::Serialization(format!(
                        "TextCommit payload length mismatch: expected {}, got {}",
                        expected_len,
                        remaining.len()
                    )));
                }

                let mut remove = Vec::with_capacity(rem_cnt);
                let mut offset = 21;
                for _ in 0..rem_cnt {
                    let fid_bytes = remaining.get(offset..offset + 8).ok_or_else(|| {
                        ContextraError::Serialization("remove FileId missing".into())
                    })?;
                    let fid = u64::from_le_bytes(fid_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid remove FileId format".into())
                    })?);
                    remove.push(FileId::new(fid));
                    offset += 8;
                }

                Ok(ManifestEntry::TextCommit {
                    add,
                    remove,
                    floor_seq,
                })
            }
            OP_TAG_SST_SWAP => {
                // 0x07: SstSwap { add: [PathBuf], remove: [PathBuf] }
                if remaining.len() < 8 {
                    return Err(ContextraError::Serialization(
                        "SstSwap payload too short".into(),
                    ));
                }

                let mut offset = 0;
                let add_count_bytes = remaining.get(offset..offset + 4).ok_or_else(|| {
                    ContextraError::Serialization("add_count bytes missing".into())
                })?;
                let add_count = u32::from_le_bytes(add_count_bytes.try_into().map_err(|_| {
                    ContextraError::Serialization("Invalid add_count format".into())
                })?) as usize;
                offset += 4;

                let max_add = remaining.get(offset..).map_or(0, |r| r.len() / 4);
                if add_count > max_add {
                    return Err(ContextraError::Serialization(format!(
                        "Invalid add_count {} exceeds remaining buffer capacity {}",
                        add_count, max_add
                    )));
                }

                let mut add = Vec::with_capacity(add_count);
                for _ in 0..add_count {
                    if remaining.len() < offset + 4 {
                        return Err(ContextraError::Serialization(
                            "SstSwap add path length truncated".into(),
                        ));
                    }
                    let p_len_bytes = remaining.get(offset..offset + 4).ok_or_else(|| {
                        ContextraError::Serialization("p_len bytes missing".into())
                    })?;
                    let p_len = u32::from_le_bytes(p_len_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid p_len format".into())
                    })?) as usize;
                    offset += 4;
                    if remaining.len() < offset + p_len {
                        return Err(ContextraError::Serialization(
                            "SstSwap add path data truncated".into(),
                        ));
                    }
                    let p_bytes = remaining
                        .get(offset..offset + p_len)
                        .ok_or_else(|| ContextraError::Serialization("p_bytes missing".into()))?;
                    let p_str = std::str::from_utf8(p_bytes).map_err(|e| {
                        ContextraError::Serialization(format!("Invalid add path UTF-8: {}", e))
                    })?;
                    add.push(PathBuf::from(p_str));
                    offset += p_len;
                }

                if remaining.len() < offset + 4 {
                    return Err(ContextraError::Serialization(
                        "SstSwap remove count missing".into(),
                    ));
                }
                let rem_count_bytes = remaining.get(offset..offset + 4).ok_or_else(|| {
                    ContextraError::Serialization("rem_count bytes missing".into())
                })?;
                let rem_count = u32::from_le_bytes(rem_count_bytes.try_into().map_err(|_| {
                    ContextraError::Serialization("Invalid rem_count format".into())
                })?) as usize;
                offset += 4;

                let max_rem = remaining.get(offset..).map_or(0, |r| r.len() / 4);
                if rem_count > max_rem {
                    return Err(ContextraError::Serialization(format!(
                        "Invalid rem_count {} exceeds remaining buffer capacity {}",
                        rem_count, max_rem
                    )));
                }

                let mut remove = Vec::with_capacity(rem_count);
                for _ in 0..rem_count {
                    if remaining.len() < offset + 4 {
                        return Err(ContextraError::Serialization(
                            "SstSwap remove path length truncated".into(),
                        ));
                    }
                    let p_len_bytes = remaining.get(offset..offset + 4).ok_or_else(|| {
                        ContextraError::Serialization("p_len bytes missing".into())
                    })?;
                    let p_len = u32::from_le_bytes(p_len_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid p_len format".into())
                    })?) as usize;
                    offset += 4;
                    if remaining.len() < offset + p_len {
                        return Err(ContextraError::Serialization(
                            "SstSwap remove path data truncated".into(),
                        ));
                    }
                    let p_bytes = remaining
                        .get(offset..offset + p_len)
                        .ok_or_else(|| ContextraError::Serialization("p_bytes missing".into()))?;
                    let p_str = std::str::from_utf8(p_bytes).map_err(|e| {
                        ContextraError::Serialization(format!("Invalid remove path UTF-8: {}", e))
                    })?;
                    remove.push(PathBuf::from(p_str));
                    offset += p_len;
                }

                if offset != remaining.len() {
                    return Err(ContextraError::Serialization(format!(
                        "SstSwap payload trailing unparsed bytes: consumed {}, total {}",
                        offset,
                        remaining.len()
                    )));
                }

                Ok(ManifestEntry::SstSwap { add, remove })
            }
            _ => Err(ContextraError::Serialization(format!(
                "Unknown Manifest op tag: {}",
                op_tag
            ))),
        }
    }

    fn parse_wal_floor(remaining: &[u8]) -> Result<Self> {
        if remaining.len() < 44 {
            return Err(ContextraError::Serialization(
                "WalFloor payload too short".into(),
            ));
        }
        let seq_bytes = remaining
            .get(0..8)
            .ok_or_else(|| ContextraError::Serialization("seq bytes missing".into()))?;
        let seq = u64::from_le_bytes(
            seq_bytes
                .try_into()
                .map_err(|_| ContextraError::Serialization("Invalid seq format".into()))?,
        );

        let mac_bytes = remaining
            .get(8..40)
            .ok_or_else(|| ContextraError::Serialization("tail_mac bytes missing".into()))?;
        let tail_mac: [u8; 32] = mac_bytes
            .try_into()
            .map_err(|_| ContextraError::Serialization("Invalid tail_mac format".into()))?;

        let count_bytes = remaining
            .get(40..44)
            .ok_or_else(|| ContextraError::Serialization("count bytes missing".into()))?;
        let count = u32::from_le_bytes(
            count_bytes
                .try_into()
                .map_err(|_| ContextraError::Serialization("Invalid count format".into()))?,
        ) as usize;

        let expected_len = count
            .checked_mul(8)
            .and_then(|v| v.checked_add(44))
            .ok_or_else(|| ContextraError::Serialization("WalFloor size overflow".into()))?;
        if remaining.len() != expected_len {
            return Err(ContextraError::Serialization(format!(
                "WalFloor payload length mismatch: expected {}, got {}",
                expected_len,
                remaining.len()
            )));
        }

        let mut removed_file_ids = Vec::with_capacity(count);
        let mut offset = 44;
        for _ in 0..count {
            let fid_bytes = remaining
                .get(offset..offset + 8)
                .ok_or_else(|| ContextraError::Serialization("FileId missing".into()))?;
            let fid = u64::from_le_bytes(
                fid_bytes
                    .try_into()
                    .map_err(|_| ContextraError::Serialization("Invalid FileId format".into()))?,
            );
            removed_file_ids.push(FileId::new(fid));
            offset += 8;
        }

        Ok(ManifestEntry::WalFloor {
            seq,
            tail_mac,
            removed_file_ids,
        })
    }

    fn parse_replace(remaining: &[u8]) -> Result<Self> {
        if remaining.len() < 24 {
            return Err(ContextraError::Serialization(
                "Replace payload header too short".into(),
            ));
        }
        let added_max_tx_bytes = remaining
            .get(0..8)
            .ok_or_else(|| ContextraError::Serialization("added_max_tx bytes missing".into()))?;
        let added_max_tx =
            u64::from_le_bytes(added_max_tx_bytes.try_into().map_err(|_| {
                ContextraError::Serialization("Invalid added_max_tx format".into())
            })?);
        let rank_bytes = remaining
            .get(8..16)
            .ok_or_else(|| ContextraError::Serialization("rank bytes missing".into()))?;
        let rank = u64::from_le_bytes(
            rank_bytes
                .try_into()
                .map_err(|_| ContextraError::Serialization("Invalid rank format".into()))?,
        );
        let added_path_len_bytes = remaining
            .get(16..20)
            .ok_or_else(|| ContextraError::Serialization("added_path_len bytes missing".into()))?;
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
        let added_path_bytes = remaining
            .get(offset..offset + added_path_len)
            .ok_or_else(|| ContextraError::Serialization("added_path bytes missing".into()))?;
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
        let removed_count_bytes = remaining
            .get(offset..offset + 4)
            .ok_or_else(|| ContextraError::Serialization("removed_count bytes missing".into()))?;
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
            let r_len_bytes = remaining
                .get(offset..offset + 4)
                .ok_or_else(|| ContextraError::Serialization("r_len bytes missing".into()))?;
            let r_len = u32::from_le_bytes(
                r_len_bytes
                    .try_into()
                    .map_err(|_| ContextraError::Serialization("Invalid r_len format".into()))?,
            ) as usize;
            offset += 4;
            if remaining.len() < offset + r_len {
                return Err(ContextraError::Serialization(
                    "Replace removed path data truncated".into(),
                ));
            }
            let r_bytes = remaining
                .get(offset..offset + r_len)
                .ok_or_else(|| ContextraError::Serialization("r_path bytes missing".into()))?;
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
}
