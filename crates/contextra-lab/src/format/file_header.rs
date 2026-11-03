//! F.1 Dateikopf (alle Datendateien und Steuerdateien).
//!
//! SPEC Anhang F.1:
//! - 0..8: magic "CTXFILE\0"
//! - 8..10: format_version u16 (v1 = 1)
//! - 10..11: kind u8 (1..=10)
//! - 11..15: alg_ids u8[4] = [aead, kdf_or_0, mac, hash]; v1: [1, 0, 1, 1]
//! - 15..31: file_id [u8; 16]
//! - 31..39: created_seq u64
//! - 39..40: key_mode u8 (0 = zufälliger Dateischlüssel, 1 = abgeleitet)
//! - 40..48: reserved [u8; 8] = 0
//! - 48..64: header_tag [u8; 16] (AEAD-Tag über [0..48) als AAD, leerer Klartext,
//!   Nonce 0xFFFFFFFF || u64(0), Dateischlüssel)
//! - 64..: Körper

use aes_gcm_siv::aead::{Aead, KeyInit, Payload};
use aes_gcm_siv::{Aes256GcmSiv, Nonce};

use crate::error::LabError;

/// Konstante Dateikopfgröße.
pub const FILE_HEADER_SIZE: usize = 64;

/// Magische Bytes für F.1 Dateikopf.
pub const FILE_MAGIC: &[u8; 8] = b"CTXFILE\0";

/// Zulässige Version v1.
pub const FORMAT_VERSION_V1: u16 = 1;

/// Standard Algorithmen-Kombination in v1: [AES-256-GCM-SIV, kein KDF, HMAC-SHA256, BLAKE3/SHA256].
pub const STANDARD_ALG_IDS: [u8; 4] = [1, 0, 1, 1];

/// Dateikategorie (kind).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FileKind {
    /// 1: WAL-Segment
    WalSegment = 1,
    /// 2: SSTable
    SsTable = 2,
    /// 3: Manifest-Log
    ManifestLog = 3,
    /// 4: Textsegment
    TextSegment = 4,
    /// 5: Vektor-Snapshot
    VectorSnapshot = 5,
    /// 6: Journal
    Journal = 6,
    /// 7: Gate-Log
    GateLog = 7,
    /// 8: Ledger
    Ledger = 8,
    /// 9: Scope-Datei
    ScopeFile = 9,
    /// 10: Tresor
    Vault = 10,
}

impl FileKind {
    /// Parst das kind-Byte.
    pub fn from_u8(val: u8) -> Result<Self, LabError> {
        match val {
            1 => Ok(Self::WalSegment),
            2 => Ok(Self::SsTable),
            3 => Ok(Self::ManifestLog),
            4 => Ok(Self::TextSegment),
            5 => Ok(Self::VectorSnapshot),
            6 => Ok(Self::Journal),
            7 => Ok(Self::GateLog),
            8 => Ok(Self::Ledger),
            9 => Ok(Self::ScopeFile),
            10 => Ok(Self::Vault),
            other => Err(LabError::UnsupportedKind(other)),
        }
    }

    /// Erwarteter key_mode für dieses kind nach SPEC F.1.
    pub fn expected_key_mode(self) -> u8 {
        match self {
            Self::WalSegment
            | Self::SsTable
            | Self::TextSegment
            | Self::VectorSnapshot
            | Self::ScopeFile => 0,
            Self::ManifestLog | Self::Journal | Self::GateLog | Self::Ledger | Self::Vault => 1,
        }
    }
}

/// Strukturierter F.1 Dateikopf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHeader {
    /// Formatversion (v1 = 1).
    pub format_version: u16,
    /// Dateikategorie.
    pub kind: FileKind,
    /// Krypto-Algorithmen-IDs.
    pub alg_ids: [u8; 4],
    /// Eindeutige Dateikennung.
    pub file_id: [u8; 16],
    /// Erstellungssequenz.
    pub created_seq: u64,
    /// Schlüsselmodus (0 = zufällig, 1 = abgeleitet).
    pub key_mode: u8,
    /// Authentisierungs-Tag des Headers.
    pub header_tag: [u8; 16],
}

impl FileHeader {
    /// Parst und validiert die 64 Byte des Dateikopfs ohne Krypto-Tag-Prüfung.
    pub fn parse(bytes: &[u8]) -> Result<Self, LabError> {
        if bytes.len() < FILE_HEADER_SIZE {
            return Err(LabError::UnexpectedEof {
                expected: FILE_HEADER_SIZE,
                actual: bytes.len(),
            });
        }

        let magic = bytes.get(0..8).ok_or(LabError::InvalidMagic)?;
        if magic != FILE_MAGIC {
            return Err(LabError::InvalidMagic);
        }

        let ver_bytes: [u8; 2] = bytes
            .get(8..10)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("Formatversion fehlt"))?;
        let format_version = u16::from_be_bytes(ver_bytes);
        if format_version != FORMAT_VERSION_V1 {
            return Err(LabError::UnsupportedVersion(format_version));
        }

        let kind_byte = *bytes.get(10).ok_or(LabError::Corrupted("Kind fehlt"))?;
        let kind = FileKind::from_u8(kind_byte)?;

        let algs: [u8; 4] = bytes
            .get(11..15)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("Algorithmen-IDs fehlen"))?;
        if algs != STANDARD_ALG_IDS {
            return Err(LabError::UnsupportedAlgIds(algs));
        }

        let file_id: [u8; 16] = bytes
            .get(15..31)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("FileId fehlt"))?;

        let seq_bytes: [u8; 8] = bytes
            .get(31..39)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("CreatedSeq fehlt"))?;
        let created_seq = u64::from_be_bytes(seq_bytes);

        let key_mode = *bytes.get(39).ok_or(LabError::Corrupted("KeyMode fehlt"))?;
        if key_mode != kind.expected_key_mode() {
            return Err(LabError::Corrupted(
                "KeyMode widerspricht SPEC F.1 Spezifikation für diesen Kind",
            ));
        }

        let reserved = bytes
            .get(40..48)
            .ok_or(LabError::Corrupted("Reserved fehlt"))?;
        if reserved.iter().any(|&b| b != 0) {
            return Err(LabError::Corrupted("Reserved-Bytes sind nicht 0"));
        }

        let header_tag: [u8; 16] = bytes
            .get(48..64)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("HeaderTag fehlt"))?;

        Ok(Self {
            format_version,
            kind,
            alg_ids: algs,
            file_id,
            created_seq,
            key_mode,
            header_tag,
        })
    }

    /// Verifiziert das AEAD-Header-Tag unter dem gegebenen Dateischlüssel.
    /// Nonce: 0xFFFFFFFF || u64(0). AAD: Bytes 0..48. Leerer Klartext.
    pub fn verify_tag(&self, raw_header_64: &[u8], file_key: &[u8; 32]) -> Result<(), LabError> {
        if raw_header_64.len() < FILE_HEADER_SIZE {
            return Err(LabError::UnexpectedEof {
                expected: FILE_HEADER_SIZE,
                actual: raw_header_64.len(),
            });
        }
        let aad = raw_header_64
            .get(0..48)
            .ok_or(LabError::Corrupted("Header AAD ungültig"))?;

        let mut nonce_bytes = [0u8; 12];
        if let Some(prefix) = nonce_bytes.get_mut(0..4) {
            prefix.copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
        }

        let cipher = Aes256GcmSiv::new_from_slice(file_key)
            .map_err(|_| LabError::Corrupted("Schlüssellänge ungültig"))?;

        let mut ct_and_tag = Vec::with_capacity(16);
        ct_and_tag.extend_from_slice(&self.header_tag);

        let payload = Payload {
            msg: &ct_and_tag,
            aad,
        };

        cipher
            .decrypt(Nonce::from_slice(&nonce_bytes), payload)
            .map_err(|_| LabError::VerificationFailed("Dateikopf-AEAD-Tag ungültig"))?;

        Ok(())
    }

    /// Hilfsfunktion zum Erzeugen eines gültigen Dateikopfs inklusive AEAD-Tag.
    pub fn create_bytes(
        kind: FileKind,
        file_id: [u8; 16],
        created_seq: u64,
        file_key: &[u8; 32],
    ) -> [u8; 64] {
        let mut buf = [0u8; 64];
        if let Some(m) = buf.get_mut(0..8) {
            m.copy_from_slice(FILE_MAGIC);
        }
        if let Some(v) = buf.get_mut(8..10) {
            v.copy_from_slice(&FORMAT_VERSION_V1.to_be_bytes());
        }
        if let Some(k) = buf.get_mut(10) {
            *k = kind as u8;
        }
        if let Some(a) = buf.get_mut(11..15) {
            a.copy_from_slice(&STANDARD_ALG_IDS);
        }
        if let Some(f) = buf.get_mut(15..31) {
            f.copy_from_slice(&file_id);
        }
        if let Some(s) = buf.get_mut(31..39) {
            s.copy_from_slice(&created_seq.to_be_bytes());
        }
        if let Some(km) = buf.get_mut(39) {
            *km = kind.expected_key_mode();
        }

        let mut nonce_bytes = [0u8; 12];
        if let Some(prefix) = nonce_bytes.get_mut(0..4) {
            prefix.copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
        }

        if let Ok(cipher) = Aes256GcmSiv::new_from_slice(file_key) {
            let empty_msg: &[u8] = b"";
            let payload = Payload {
                msg: empty_msg,
                aad: buf.get(0..48).unwrap_or(&[]),
            };
            if let Ok(tag) = cipher.encrypt(Nonce::from_slice(&nonce_bytes), payload) {
                if let Some(dest) = buf.get_mut(48..64) {
                    dest.copy_from_slice(&tag);
                }
            }
        }

        buf
    }
}
