//! # Blob-Sweep und Generationstausch
//!
//! SPEC §9.6 und §14.3: Erzeugt Generation g+1 mit identischen Offsets.
//! Gelöschte Records werden durch Hole-Records mit Nullen ersetzt.

use crate::blob::record::{
    BlobFooter, BlobHeader, BLOB_FOOTER_LEN, BLOB_HEADER_LEN, RECORD_KIND_HOLE,
};
use crate::codec::crc32c::crc32c;
use contextra_storage_ports::{File, Mac};
use std::collections::HashSet;
use std::fmt;
use std::io;

/// Fehler beim Durchführen eines Blob-Sweeps.
#[derive(Debug)]
pub enum BlobSweepError {
    /// E/A-Fehler.
    Io(io::Error),
    /// Datenbeschädigung in der Quelldatei.
    Corrupt(&'static str),
    /// Fehlendes MAC-Primitiv für Hole-Tag.
    CryptoMissing(&'static str),
    /// Arithmetischer Überlauf.
    Overflow,
}

impl fmt::Display for BlobSweepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "Blob-Sweep E/A-Fehler: {e}"),
            Self::Corrupt(msg) => write!(f, "Blob-Sweep Quellbeschädigung: {msg}"),
            Self::CryptoMissing(msg) => write!(f, "Blob-Sweep Krypto fehlt: {msg}"),
            Self::Overflow => write!(f, "Überlauf bei Offset-Berechnung"),
        }
    }
}

impl std::error::Error for BlobSweepError {}

impl From<io::Error> for BlobSweepError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

/// Führt den Generationstausch g -> g+1 durch (SPEC §9.6).
///
/// INV-BLOB-3: Hole-Records werden nie verschlüsselt.
pub fn sweep_blob(
    src_file: &dyn File,
    dst_file: &dyn File,
    live_offsets: &HashSet<u64>,
    integrity_mode: u8,
    mac_key: Option<&[u8; 32]>,
    mac: Option<&dyn Mac>,
) -> Result<BlobHeader, BlobSweepError> {
    // 1. Quell-Header lesen und verifizieren
    let mut header_buf = [0u8; BLOB_HEADER_LEN];
    src_file.read_at(&mut header_buf, 0)?;
    let src_header = BlobHeader::decode(&header_buf)
        .map_err(|_| BlobSweepError::Corrupt("Ungültiger Quell-Header"))?;

    let new_generation = src_header
        .generation
        .checked_add(1)
        .ok_or(BlobSweepError::Overflow)?;

    // 2. Ziel-Header mit neuer Generation schreiben
    let dst_header = BlobHeader::new(src_header.blob_no, new_generation, src_header.space);
    let dst_header_buf = dst_header.encode();
    dst_file.write_all_at(&dst_header_buf, 0)?;

    let src_len = src_file.len()?;
    let footer_len_u64 =
        u64::try_from(BLOB_FOOTER_LEN).map_err(|_| BlobSweepError::Overflow)?;
    let footer_start = src_len
        .checked_sub(footer_len_u64)
        .ok_or(BlobSweepError::Corrupt("Datei zu kurz für Blob-Footer"))?;

    let mut current_offset =
        u64::try_from(BLOB_HEADER_LEN).map_err(|_| BlobSweepError::Overflow)?;

    let tag_len_usize: usize = match integrity_mode {
        0 => 4,
        _ => 16,
    };
    let tag_len_u64 =
        u64::try_from(tag_len_usize).map_err(|_| BlobSweepError::Overflow)?;

    // 3. Records sequenziell durchgehen bis zum Footer
    while current_offset < footer_start {
        let mut rlen_b = [0u8; 4];
        src_file.read_at(&mut rlen_b, current_offset)?;
        let rlen = u32::from_le_bytes(rlen_b);
        let rlen_usize = usize::try_from(rlen).map_err(|_| BlobSweepError::Overflow)?;
        let rlen_u64 = u64::from(rlen);

        let total_record_len = 4u64
            .checked_add(rlen_u64)
            .and_then(|l| l.checked_add(tag_len_u64))
            .ok_or(BlobSweepError::Overflow)?;

        let total_record_usize =
            usize::try_from(total_record_len).map_err(|_| BlobSweepError::Overflow)?;

        if live_offsets.contains(&current_offset) {
            // Lebendiger Record: Exakt gleiche Bytes an exakt gleichen Offset kopieren
            let mut record_bytes = vec![0u8; total_record_usize];
            src_file.read_at(&mut record_bytes, current_offset)?;
            dst_file.write_all_at(&record_bytes, current_offset)?;
        } else {
            // INV-BLOB-3: Gelöschter Record wird durch unverschlüsseltes Hole ersetzt
            let mut hole_bytes = Vec::with_capacity(total_record_usize);
            hole_bytes.extend_from_slice(&rlen.to_le_bytes());

            // Payload: 0x02 gefolgt von (rlen - 1) Nullen
            let mut payload = vec![0u8; rlen_usize];
            if let Some(first) = payload.first_mut() {
                *first = RECORD_KIND_HOLE;
            }

            hole_bytes.extend_from_slice(&payload);

            match integrity_mode {
                0 => {
                    // Plain: CRC32C über Nullen
                    let chk = crc32c(0, &payload);
                    hole_bytes.extend_from_slice(&chk.to_le_bytes());
                }
                _ => {
                    // Authenticated/Encrypted: Mac16(blob_no || offset || rlen || 0x02)
                    let m = mac.ok_or(BlobSweepError::CryptoMissing("Mac für Hole-Tag fehlt"))?;
                    let key = mac_key.ok_or(BlobSweepError::CryptoMissing("Schlüssel für Hole-Tag fehlt"))?;

                    let b_no = src_header.blob_no.to_le_bytes();
                    let off_b = current_offset.to_le_bytes();
                    let rlen_b = rlen.to_le_bytes();
                    let kind_b = [RECORD_KIND_HOLE];

                    let tag = m.mac16(key, &[&b_no, &off_b, &rlen_b, &kind_b]);
                    hole_bytes.extend_from_slice(&tag);
                }
            }

            dst_file.write_all_at(&hole_bytes, current_offset)?;
        }

        current_offset = current_offset
            .checked_add(total_record_len)
            .ok_or(BlobSweepError::Overflow)?;
    }

    // 4. Footer kopieren und synchronisieren
    let mut footer_buf = [0u8; BLOB_FOOTER_LEN];
    src_file.read_at(&mut footer_buf, footer_start)?;
    let footer = BlobFooter::decode(&footer_buf)
        .map_err(|_| BlobSweepError::Corrupt("Ungültiger Quell-Footer"))?;

    let dst_footer_buf = footer.encode();
    dst_file.write_all_at(&dst_footer_buf, footer_start)?;
    dst_file.sync_all()?;

    Ok(dst_header)
}
