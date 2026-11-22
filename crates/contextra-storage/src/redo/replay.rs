//! # Redo-Replay und Wiederherstellung
//!
//! SPEC §9.4, §13.1 Schritt 6 und §18:
//! Liest Redo-Segmente ab `redo_floor` in ID-Reihenfolge, validiert Siegel und Invarianten,
//! schneidet Torn Tails im letzten ungesiegelten Segment ab und siegelt sie.

use contextra_storage_ports::{Fs, OpenMode};
use crate::redo::error::{check_redo_hwm, RedoError};
use crate::redo::frame::RedoFrame;
use crate::redo::op::RedoOp;
use crate::redo::replay_sm::{ReplayEffect, ReplayEvent, ReplayStateMachine};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

/// Ergebnis des Redo-Replays nach SPEC §13.1 Schritt 6.
#[derive(Debug, Clone, Default)]
pub struct ReplayResult {
    /// Alle erfolgreich wiederhergestellten Frames mit ihren Operationen.
    pub committed_frames: Vec<(RedoFrame, Vec<RedoOp>)>,
    /// Höchster wiederhergestellter Commit-Stamp.
    pub max_stamp: u64,
    /// Anzahl gelesener Segmente.
    pub segments_count: usize,
    /// ID des vorallokierten Reservesegments (falls vorhanden, INV-LOG-7).
    pub reserve_segment_id: Option<u64>,
}

/// Extrahiert die Segment-ID aus einem Dateinamen wie `redo-0000000000000012.log`.
pub fn parse_segment_id(file_name: &str) -> Option<u64> {
    let name = file_name.strip_prefix("redo-")?.strip_suffix(".log")?;
    if name.len() != 16 {
        return None;
    }
    u64::from_str_radix(name, 16).ok()
}

/// Führt das vollständige Redo-Replay über alle Segmente ab `redo_floor` durch.
pub fn recover_redo<F: Fs>(
    fs: Arc<F>,
    base_dir: &Path,
    redo_floor: u64,
    redo_floor_stamp: u64,
    redo_hwm: u64,
) -> Result<ReplayResult, RedoError> {
    let mut result = ReplayResult::default();

    // 1. Verzeichnis auflisten und Redo-Segmente finden
    let entries = match fs.list_dir(base_dir) {
        Ok(e) => e,
        Err(_) => return Ok(result), // Verzeichnis existiert noch nicht
    };

    let mut segments = BTreeMap::new();
    for entry in entries {
        if let Some(seg_id) = parse_segment_id(&entry.name) {
            // Nur Segmente ab redo_floor betrachten (INV-LOG-7)
            if seg_id >= redo_floor {
                segments.insert(seg_id, entry);
            }
        }
    }

    result.segments_count = segments.len();

    // 2. Segmente in aufsteigender ID-Reihenfolge abarbeiten
    let mut unsealed_segment_with_frames: Option<u64> = None;
    let mut last_valid_stamp: Option<u64> = None;

    let segment_ids: Vec<u64> = segments.keys().copied().collect();
    let num_segments = segment_ids.len();

    for (seg_idx, &seg_id) in segment_ids.iter().enumerate() {
        let seg_path = base_dir.join(crate::redo::driver::segment_file_name(seg_id));
        let file = fs.open(&seg_path, OpenMode::ReadWrite)?;

        let mut rsm = ReplayStateMachine::new();
        let mut buf = [0u8; 64 * 1024];
        let mut offset = 0u64;

        let mut seg_frames = Vec::new();
        let mut has_defect = false;
        let mut defect_good_offset = 0u64;
        let mut defect_cause = String::new();

        loop {
            let bytes_read = file.read_at(&mut buf, offset)?;
            if bytes_read == 0 {
                let effects = rsm.step(ReplayEvent::Eof);
                for eff in effects {
                    if let ReplayEffect::TornTail { good_offset, cause } = eff {
                        has_defect = true;
                        defect_good_offset = good_offset;
                        defect_cause = cause;
                    }
                }
                break;
            }

            let slice = match buf.get(..bytes_read) {
                Some(s) => s,
                None => break,
            };

            let effects = rsm.step(ReplayEvent::Chunk(slice));
            let bytes_read_u64 = match u64::try_from(bytes_read) {
                Ok(v) => v,
                Err(_) => 0,
            };
            offset = offset.saturating_add(bytes_read_u64);

            for eff in effects {
                match eff {
                    ReplayEffect::ValidFrame { frame, ops, next_offset: _ } => {
                        // INV-LOG-2: Beim Replay ist ein Frame vollständig gültig oder beendet den Log
                        last_valid_stamp = Some(frame.stamp);
                        seg_frames.push((frame, ops));
                    }
                    ReplayEffect::TornTail { good_offset, cause } => {
                        has_defect = true;
                        defect_good_offset = good_offset;
                        defect_cause = cause;
                    }
                    ReplayEffect::CorruptDefect { offset: _, cause } => {
                        has_defect = true;
                        defect_good_offset = rsm.current_offset;
                        defect_cause = cause;
                    }
                    _ => {}
                }
            }

            if rsm.is_terminated {
                break;
            }
        }

        let is_last_segment = seg_idx.saturating_add(1) == num_segments;

        // Prüfe Reserve-Segment (INV-LOG-7)
        // INV-LOG-7: Ein Segment ohne Frames mit gültigem Header und id >= redo_floor ist lebendig (Reserve).
        if seg_frames.is_empty() && !rsm.is_sealed && !has_defect {
            result.reserve_segment_id = Some(seg_id);
            continue;
        }

        // Prüfe INV-LOG-6: Jedes Segment außer dem neuesten mit Frames ist gesiegelt
        if let Some(prev_unsealed_id) = unsealed_segment_with_frames {
            if !seg_frames.is_empty() {
                return Err(RedoError::Corrupt(format!(
                    "INV-LOG-6: Ungesiegeltes Segment {prev_unsealed_id} hat nachfolgendes Segment {seg_id} mit Frames"
                )));
            }
        }

        if rsm.is_sealed {
            // Gesiegeltes Segment: Jeder Defekt ist Corrupt (fail-closed, INV-LOG-6)
            if has_defect {
                return Err(RedoError::Corrupt(format!(
                    "INV-LOG-6: Defekt in gesiegeltem Segment {seg_id}: {defect_cause}"
                )));
            }
        } else {
            // Ungesiegeltes Segment
            if !seg_frames.is_empty() {
                unsealed_segment_with_frames = Some(seg_id);
            }

            if is_last_segment {
                // Letztes ungesiegeltes Segment: Auf letztes gültiges Frame abschneiden und siegeln (SPEC §13.1 Schritt 6)
                let trunc_offset = if has_defect {
                    defect_good_offset
                } else {
                    rsm.current_offset
                };

                file.set_len(trunc_offset)?;
                file.sync_data()?;

                // Siegel anhängen
                let stamp = last_valid_stamp.unwrap_or(0);
                let (seal_frame, _) = RedoFrame::new_seal_plain(
                    stamp,
                    rsm.frame_count,
                    rsm.last_chain,
                    rsm.last_chain,
                );
                let seal_bytes = seal_frame.to_bytes();
                file.write_all_at(&seal_bytes, trunc_offset)?;
                file.sync_data()?;
            } else if !seg_frames.is_empty() {
                // Ein früheres Segment mit Frames ist nicht gesiegelt => Corrupt (INV-LOG-6)
                return Err(RedoError::Corrupt(format!(
                    "INV-LOG-6: Früheres Segment {seg_id} mit Frames ist nicht gesiegelt"
                )));
            }
        }

        // Frames in Gesamtergebnis übernehmen
        for (f, ops) in seg_frames {
            if f.stamp > result.max_stamp {
                result.max_stamp = f.stamp;
            }
            result.committed_frames.push((f, ops));
        }
    }

    // 3. INV-LOG-8: Redo-HWM-Prüfung nach Schritt 6
    // INV-LOG-8: letzter gültiger Stamp (bzw. redo_floor_stamp) < redo_hwm => Err(Corrupt{RedoTruncated})
    check_redo_hwm(last_valid_stamp, redo_floor_stamp, redo_hwm)?;

    Ok(result)
}
