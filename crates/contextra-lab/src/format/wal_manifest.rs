//! F.4 WAL und Manifest Parser.
//!
//! SPEC Anhang F.4:
//! - WAL-Segment (kind = 1):
//!   - Kopf nach F.1
//!   - Segmentkopf: prev_mac(32) || first_seq(u64)
//!   - Frames:
//!     - len(u32) || seq(u64) || ct(len) || aead_tag(16) || mac(32)
//!     - ct = AEAD mit Dateischlüssel, Nonce, AAD und Klartext = Batch
//!     - mac_n = HMAC-SHA256(K_wal, prev_mac || len || seq || ct || aead_tag)
//!   - Torn Write Prüfung: unvollständiger/ungültiger letzter Frame wird verworfen,
//!     wenn alle folgenden Bytes 0 sind; sonst Corrupt.
//!
//! - Manifest-Log (kind = 3):
//!   - Kopf nach F.1
//!   - Appendlog mit abgeleitetem Dateischlüssel
//!   - Eintrag: len(u32) || type(u8) || payload || aead_tag(16) || chain(32)
//!   - chain = SHA-256(chain_prev || Eintrag_ohne_chain)
//!   - Typen:
//!     - 0x01: AddFile { file_id(16), kind(u8), level(u8), lo(65), hi(65), min_seq(u64), max_seq(u64), size(u64) }
//!     - 0x02: RemoveFile { file_id(16) }
//!     - 0x03: WalFloor { seq(u64), tail_mac(32), removed_file_ids }
//!     - 0x04: SeqMark { next_seq(u64) }
//!     - 0x05: Collection { coll_tag(16), name_tag(16), options_envelope }

use aes_gcm_siv::aead::{Aead, KeyInit, Payload};
use aes_gcm_siv::{Aes256GcmSiv, Nonce};
use hmac::Hmac;
use sha2::digest::Mac;
use sha2::{Digest, Sha256};

use crate::error::LabError;
use crate::format::file_header::{FileHeader, FileKind, FILE_HEADER_SIZE};

/// Frame eines WAL-Segments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalFrame {
    /// Sequenznummer.
    pub seq: u64,
    /// Chiffrat.
    pub ciphertext: Vec<u8>,
    /// AEAD-Tag (16 Byte).
    pub aead_tag: [u8; 16],
    /// HMAC (32 Byte).
    pub mac: [u8; 32],
}

/// WAL-Segment Kopfdaten.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalSegmentHeader {
    /// Vorheriger MAC.
    pub prev_mac: [u8; 32],
    /// Erste Sequenznummer im Segment.
    pub first_seq: u64,
}

/// Strukturierte Repräsentation eines WAL-Segments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalSegment {
    /// F.1 Dateikopf.
    pub header: FileHeader,
    /// Segmentkopf.
    pub segment_header: WalSegmentHeader,
    /// Gültig geparste Frames.
    pub frames: Vec<WalFrame>,
    /// Ob ein sauberer Torn-Write am Ende verworfen wurde.
    pub had_torn_write: bool,
}

impl WalSegment {
    /// Parst ein WAL-Segment unter Berücksichtigung von HMAC-Kette und Torn-Writes.
    pub fn parse(bytes: &[u8], k_wal: &[u8; 32], file_key: &[u8; 32]) -> Result<Self, LabError> {
        let header = FileHeader::parse(bytes)?;
        if header.kind != FileKind::WalSegment {
            return Err(LabError::UnsupportedKind(header.kind as u8));
        }
        header.verify_tag(bytes, file_key)?;

        let seg_head_bytes =
            bytes
                .get(FILE_HEADER_SIZE..FILE_HEADER_SIZE + 40)
                .ok_or(LabError::UnexpectedEof {
                    expected: FILE_HEADER_SIZE + 40,
                    actual: bytes.len(),
                })?;

        let prev_mac: [u8; 32] = seg_head_bytes
            .get(0..32)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("Segment prev_mac fehlt"))?;
        let first_seq = u64::from_be_bytes(
            seg_head_bytes
                .get(32..40)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Segment first_seq fehlt"))?,
        );

        let segment_header = WalSegmentHeader {
            prev_mac,
            first_seq,
        };

        let mut frames = Vec::new();
        let mut offset = FILE_HEADER_SIZE + 40;
        let mut running_prev_mac = prev_mac;
        let mut frame_idx = 0u64;
        let mut had_torn_write = false;

        type HmacSha256 = Hmac<Sha256>;

        while offset < bytes.len() {
            let rem = bytes.len() - offset;
            if rem < 12 + 16 + 32 {
                let tail = bytes.get(offset..).unwrap_or(&[]);
                if tail.iter().all(|&b| b == 0) {
                    had_torn_write = true;
                    break;
                } else {
                    return Err(LabError::Corrupted("Torn Write mit Nicht-Null-Bytes"));
                }
            }

            let len_bytes = bytes
                .get(offset..offset + 4)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Frame len fehlt"))?;
            let len = u32::from_be_bytes(len_bytes) as usize;

            let total_frame_len = 4 + 8 + len + 16 + 32;
            if offset + total_frame_len > bytes.len() {
                let tail = bytes.get(offset..).unwrap_or(&[]);
                if tail.iter().all(|&b| b == 0) {
                    had_torn_write = true;
                    break;
                } else {
                    return Err(LabError::Corrupted(
                        "Abgeschnittener Frame ohne Null-Trailing",
                    ));
                }
            }

            let seq_bytes = bytes
                .get(offset + 4..offset + 12)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Frame seq fehlt"))?;
            let seq = u64::from_be_bytes(seq_bytes);

            let ct = bytes
                .get(offset + 12..offset + 12 + len)
                .ok_or(LabError::Corrupted("Frame ct fehlt"))?
                .to_vec();

            let tag: [u8; 16] = bytes
                .get(offset + 12 + len..offset + 12 + len + 16)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Frame tag fehlt"))?;

            let mac: [u8; 32] = bytes
                .get(offset + 12 + len + 16..offset + total_frame_len)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Frame mac fehlt"))?;

            let mut hmac = <HmacSha256 as hmac::digest::KeyInit>::new_from_slice(k_wal)
                .map_err(|_| LabError::Corrupted("HMAC Init fehlgeschlagen"))?;
            hmac.update(&running_prev_mac);
            hmac.update(&len_bytes);
            hmac.update(&seq_bytes);
            hmac.update(&ct);
            hmac.update(&tag);

            if hmac.verify_slice(&mac).is_err() {
                let tail = bytes.get(offset..).unwrap_or(&[]);
                if tail.iter().all(|&b| b == 0) {
                    had_torn_write = true;
                    break;
                } else {
                    return Err(LabError::VerificationFailed("WAL Frame MAC ungültig"));
                }
            }

            let mut nonce = [0u8; 12];
            if let Some(dest) = nonce.get_mut(4..12) {
                dest.copy_from_slice(&frame_idx.to_be_bytes());
            }

            let mut aad = Vec::with_capacity(3 + 16 + 8 + 32);
            aad.extend_from_slice(b"wal");
            aad.extend_from_slice(&header.file_id);
            aad.extend_from_slice(&frame_idx.to_be_bytes());
            aad.extend_from_slice(&running_prev_mac);

            let cipher = Aes256GcmSiv::new_from_slice(file_key)
                .map_err(|_| LabError::Corrupted("Cipher Init Fehler"))?;

            let mut ct_and_tag = ct.clone();
            ct_and_tag.extend_from_slice(&tag);

            let payload = Payload {
                msg: &ct_and_tag,
                aad: &aad,
            };

            cipher
                .decrypt(Nonce::from_slice(&nonce), payload)
                .map_err(|_| {
                    LabError::VerificationFailed("WAL Frame AEAD Entschlüsselung fehlgeschlagen")
                })?;

            running_prev_mac = mac;
            frame_idx = frame_idx.saturating_add(1);
            offset += total_frame_len;

            frames.push(WalFrame {
                seq,
                ciphertext: ct,
                aead_tag: tag,
                mac,
            });
        }

        Ok(Self {
            header,
            segment_header,
            frames,
            had_torn_write,
        })
    }
}

/// Manifest Log Eintrag Typen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestEntryPayload {
    /// 0x01 AddFile
    AddFile {
        /// Dateikennung.
        file_id: [u8; 16],
        /// Typ der Datei.
        kind: u8,
        /// Level.
        level: u8,
        /// Untere Grenze Schlüsselraum (65 Bytes).
        lo: [u8; 65],
        /// Obere Grenze Schlüsselraum (65 Bytes).
        hi: [u8; 65],
        /// Min Sequenz.
        min_seq: u64,
        /// Max Sequenz.
        max_seq: u64,
        /// Größe in Bytes.
        size: u64,
    },
    /// 0x02 RemoveFile
    RemoveFile {
        /// Dateikennung.
        file_id: [u8; 16],
    },
    /// 0x03 WalFloor
    WalFloor {
        /// Sequenz bis zu der WAL dauerhaft in SSTables ist.
        seq: u64,
        /// Erwarteter tail_mac des ersten verbleibenden Segments.
        tail_mac: [u8; 32],
        /// Entfernte Datei-IDs.
        removed_file_ids: Vec<[u8; 16]>,
    },
    /// 0x04 SeqMark
    SeqMark {
        /// Nächste Sequenznummer.
        next_seq: u64,
    },
    /// 0x05 Collection
    Collection {
        /// Collection Tag.
        coll_tag: [u8; 16],
        /// Name Tag.
        name_tag: [u8; 16],
        /// Optionen Chiffrathülle.
        options_envelope: Vec<u8>,
    },
    /// Sonstiges
    Unknown(u8, Vec<u8>),
}

/// Manifest-Log Eintrag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEntry {
    /// Typ-Byte.
    pub entry_type: u8,
    /// Dekodierte Nutzlast.
    pub payload: ManifestEntryPayload,
    /// AEAD-Tag.
    pub aead_tag: [u8; 16],
    /// Kette (SHA-256).
    pub chain: [u8; 32],
}

/// Strukturierte Repräsentation eines Manifest-Logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestLog {
    /// F.1 Dateikopf.
    pub header: FileHeader,
    /// Geparste Einträge.
    pub entries: Vec<ManifestEntry>,
}

impl ManifestLog {
    /// Parst ein Manifest-Log.
    pub fn parse(bytes: &[u8], file_key: &[u8; 32]) -> Result<Self, LabError> {
        let header = FileHeader::parse(bytes)?;
        if header.kind != FileKind::ManifestLog {
            return Err(LabError::UnsupportedKind(header.kind as u8));
        }
        header.verify_tag(bytes, file_key)?;

        let mut entries = Vec::new();
        let mut offset = FILE_HEADER_SIZE;
        let mut chain_prev = [0u8; 32];

        while offset < bytes.len() {
            if offset + 4 > bytes.len() {
                break;
            }
            let len_bytes = bytes
                .get(offset..offset + 4)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Entry len fehlt"))?;
            let len = u32::from_be_bytes(len_bytes) as usize;

            if len < 1 {
                return Err(LabError::Corrupted("Entry len kleiner 1"));
            }

            let total_entry_len = 4 + len + 16 + 32;
            if offset + total_entry_len > bytes.len() {
                return Err(LabError::UnexpectedEof {
                    expected: offset + total_entry_len,
                    actual: bytes.len(),
                });
            }

            let entry_type = *bytes
                .get(offset + 4)
                .ok_or(LabError::Corrupted("Type fehlt"))?;

            let raw_payload = bytes
                .get(offset + 5..offset + 4 + len)
                .ok_or(LabError::Corrupted("Payload fehlt"))?;

            let aead_tag: [u8; 16] = bytes
                .get(offset + 4 + len..offset + 4 + len + 16)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Tag fehlt"))?;

            let chain: [u8; 32] = bytes
                .get(offset + 4 + len + 16..offset + total_entry_len)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Chain fehlt"))?;

            let mut hasher = Sha256::new();
            hasher.update(chain_prev);
            let entry_pre = bytes
                .get(offset..offset + 4 + len + 16)
                .ok_or(LabError::Corrupted("Pre fehlt"))?;
            hasher.update(entry_pre);
            let expected_chain = hasher.finalize();

            if chain != expected_chain.as_slice() {
                return Err(LabError::VerificationFailed(
                    "Manifest-Log Hash-Kette unterbrochen",
                ));
            }

            let payload = match entry_type {
                0x01 => {
                    if raw_payload.len() < 172 {
                        return Err(LabError::Corrupted("AddFile Payload zu kurz"));
                    }
                    let file_id: [u8; 16] = raw_payload
                        .get(0..16)
                        .and_then(|s| s.try_into().ok())
                        .unwrap_or([0; 16]);
                    let k = *raw_payload.get(16).unwrap_or(&0);
                    let lvl = *raw_payload.get(17).unwrap_or(&0);
                    let lo: [u8; 65] = raw_payload
                        .get(18..83)
                        .and_then(|s| s.try_into().ok())
                        .unwrap_or([0; 65]);
                    let hi: [u8; 65] = raw_payload
                        .get(83..148)
                        .and_then(|s| s.try_into().ok())
                        .unwrap_or([0; 65]);
                    let min_seq = u64::from_be_bytes(
                        raw_payload
                            .get(148..156)
                            .and_then(|s| s.try_into().ok())
                            .unwrap_or([0; 8]),
                    );
                    let max_seq = u64::from_be_bytes(
                        raw_payload
                            .get(156..164)
                            .and_then(|s| s.try_into().ok())
                            .unwrap_or([0; 8]),
                    );
                    let size = u64::from_be_bytes(
                        raw_payload
                            .get(164..172)
                            .and_then(|s| s.try_into().ok())
                            .unwrap_or([0; 8]),
                    );

                    ManifestEntryPayload::AddFile {
                        file_id,
                        kind: k,
                        level: lvl,
                        lo,
                        hi,
                        min_seq,
                        max_seq,
                        size,
                    }
                }
                0x02 => {
                    if raw_payload.len() < 16 {
                        return Err(LabError::Corrupted("RemoveFile Payload zu kurz"));
                    }
                    let file_id: [u8; 16] = raw_payload
                        .get(0..16)
                        .and_then(|s| s.try_into().ok())
                        .unwrap_or([0; 16]);
                    ManifestEntryPayload::RemoveFile { file_id }
                }
                0x03 => {
                    if raw_payload.len() < 40 {
                        return Err(LabError::Corrupted("WalFloor Payload zu kurz"));
                    }
                    let seq = u64::from_be_bytes(
                        raw_payload
                            .get(0..8)
                            .and_then(|s| s.try_into().ok())
                            .unwrap_or([0; 8]),
                    );
                    let tail_mac: [u8; 32] = raw_payload
                        .get(8..40)
                        .and_then(|s| s.try_into().ok())
                        .unwrap_or([0; 32]);
                    let mut removed_file_ids = Vec::new();
                    let mut p_off = 40;
                    while p_off + 16 <= raw_payload.len() {
                        let fid: [u8; 16] = raw_payload
                            .get(p_off..p_off + 16)
                            .and_then(|s| s.try_into().ok())
                            .unwrap_or([0; 16]);
                        removed_file_ids.push(fid);
                        p_off += 16;
                    }
                    ManifestEntryPayload::WalFloor {
                        seq,
                        tail_mac,
                        removed_file_ids,
                    }
                }
                0x04 => {
                    if raw_payload.len() < 8 {
                        return Err(LabError::Corrupted("SeqMark Payload zu kurz"));
                    }
                    let next_seq = u64::from_be_bytes(
                        raw_payload
                            .get(0..8)
                            .and_then(|s| s.try_into().ok())
                            .unwrap_or([0; 8]),
                    );
                    ManifestEntryPayload::SeqMark { next_seq }
                }
                other => ManifestEntryPayload::Unknown(other, raw_payload.to_vec()),
            };

            chain_prev = chain;
            offset += total_entry_len;

            entries.push(ManifestEntry {
                entry_type,
                payload,
                aead_tag,
                chain,
            });
        }

        Ok(Self { header, entries })
    }
}
