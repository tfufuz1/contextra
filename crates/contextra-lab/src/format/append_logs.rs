//! F.9 Journal, Gate-Log, Ledger Parser.
//!
//! SPEC Anhang F.9:
//! - Alle drei: Appendlogs mit abgeleitetem Dateischlüssel (`kind = 6, 7, 8`)
//! - Eintrag: `len(u32) || ct || aead_tag(16) || chain(32)`
//! - `chain = SHA-256(chain_prev || len || ct || aead_tag)`
//! - Journal-Klartext:
//!   `erasure_id(16) || state(u8) || time(u64) || commitment(16) || target_type(u8) || retention_code(u8) || scope_tags(list lo/hi) || counters || witness_hashes`
//!   - Zustände: 1 Requested+Intent, 2 Gated, 3 KeyDestroyed, 4 Purging, 5 Verifying, 6 Completed, 7 Pending, 8 Refused
//! - Gate-Log-Klartext:
//!   `Add{gate_id(16), variant(u8: 1 Erased, 2 Restricted), lo(65), hi(65), gated_seq(u64), erasure_id(16)}`
//!   oder `Remove{gate_id(16)}`
//! - Ledger-Eintrag:
//!   `commitment(16) || gated_seq(u64) || completed_at(u64) || range_count(u16) || (lo || hi)*`
//!   - `ledger_mac = HMAC(K_ledger, chain)` je Eintrag

use hmac::Hmac;
use sha2::digest::Mac;
use sha2::{Digest, Sha256};

use crate::error::LabError;
use crate::format::file_header::{FileHeader, FileKind, FILE_HEADER_SIZE};

/// Ein logischer Append-Log-Eintrag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendLogEntry {
    /// Verschlüsselte Nutzlast.
    pub ciphertext: Vec<u8>,
    /// AEAD-Tag (16 Byte).
    pub aead_tag: [u8; 16],
    /// Kette (SHA-256).
    pub chain: [u8; 32],
}

/// Strukturierte Repräsentation eines Append-Logs (Journal / GateLog / Ledger).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendLogFile {
    /// F.1 Dateikopf.
    pub header: FileHeader,
    /// Einträge.
    pub entries: Vec<AppendLogEntry>,
}

impl AppendLogFile {
    /// Parst ein beliebiges Append-Log und validiert die kryptografische SHA-256 Kette.
    pub fn parse(bytes: &[u8], expected_kind: FileKind) -> Result<Self, LabError> {
        let header = FileHeader::parse(bytes)?;
        if header.kind != expected_kind {
            return Err(LabError::UnsupportedKind(header.kind as u8));
        }

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

            let total_len = 4 + len + 16 + 32;
            if offset + total_len > bytes.len() {
                return Err(LabError::UnexpectedEof {
                    expected: offset + total_len,
                    actual: bytes.len(),
                });
            }

            let ct = bytes
                .get(offset + 4..offset + 4 + len)
                .ok_or(LabError::Corrupted("ct fehlt"))?
                .to_vec();

            let aead_tag: [u8; 16] = bytes
                .get(offset + 4 + len..offset + 4 + len + 16)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("tag fehlt"))?;

            let chain: [u8; 32] = bytes
                .get(offset + 4 + len + 16..offset + total_len)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("chain fehlt"))?;

            let mut hasher = Sha256::new();
            hasher.update(chain_prev);
            let pre = bytes
                .get(offset..offset + 4 + len + 16)
                .ok_or(LabError::Corrupted("pre fehlt"))?;
            hasher.update(pre);
            let expected_chain = hasher.finalize();

            if chain != expected_chain.as_slice() {
                return Err(LabError::VerificationFailed("Hash-Kette unterbrochen"));
            }

            chain_prev = chain;
            offset += total_len;

            entries.push(AppendLogEntry {
                ciphertext: ct,
                aead_tag,
                chain,
            });
        }

        Ok(Self { header, entries })
    }

    /// Verifiziert den Ledger-MAC eines Eintrags.
    /// ledger_mac = HMAC-SHA256(K_ledger, chain)
    pub fn verify_ledger_entry_mac(
        chain: &[u8; 32],
        ledger_mac: &[u8; 32],
        k_ledger: &[u8; 32],
    ) -> Result<(), LabError> {
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = <HmacSha256 as hmac::digest::KeyInit>::new_from_slice(k_ledger)
            .map_err(|_| LabError::Corrupted("HMAC Init fehlgeschlagen"))?;
        mac.update(chain);
        mac.verify_slice(ledger_mac)
            .map_err(|_| LabError::VerificationFailed("Ledger HMAC Verifikation fehlgeschlagen"))
    }
}
