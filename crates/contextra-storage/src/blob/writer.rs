//! # Blob-Writer
//!
//! Schreibt Blob-Dateien nach SPEC §9.6 append-only bis zum Abschluss per Footer.

use crate::blob::record::{
    BlobFooter, BlobHeader, BlobRecord, BlobRef, BLOB_FOOTER_LEN, BLOB_HEADER_LEN,
};
use crate::codec::crc32c::crc32c;
use crate::keys::nonce::{blob_body_nonce, blob_record_nonce};
use contextra_storage_ports::{Cipher, File, Kdf, Mac};
use std::fmt;
use std::io;

/// Fehler bei Schreiboperationen auf Blob-Dateien.
#[derive(Debug)]
pub enum BlobWriteError {
    /// E/A-Fehler.
    Io(io::Error),
    /// Unbekannter oder ungültiger Integritätsmodus.
    InvalidIntegrityMode(u8),
    /// Fehlendes kryptografisches Primitiv oder Schlüssel.
    CryptoMissing(&'static str),
    /// Arithmetischer Überlauf.
    Overflow,
}

impl fmt::Display for BlobWriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "Blob E/A-Fehler: {e}"),
            Self::InvalidIntegrityMode(m) => write!(f, "Ungültiger Integritätsmodus: {m}"),
            Self::CryptoMissing(msg) => write!(f, "Kryptografischer Port/Schlüssel fehlt: {msg}"),
            Self::Overflow => write!(f, "Überlauf bei Offset- oder Längenberechnung"),
        }
    }
}

impl std::error::Error for BlobWriteError {}

impl From<io::Error> for BlobWriteError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

/// Writer für das sequenzielle Erstellen einer Blob-Datei.
pub struct BlobWriter {
    blob_no: u64,
    generation: u32,
    space: u16,
    integrity_mode: u8,
    offset: u64,
    record_count: u32,
    header_written: bool,
}

impl BlobWriter {
    /// Erstellt einen neuen Writer.
    pub fn new(blob_no: u64, generation: u32, space: u16, integrity_mode: u8) -> Self {
        Self {
            blob_no,
            generation,
            space,
            integrity_mode,
            offset: 0,
            record_count: 0,
            header_written: false,
        }
    }

    /// Schreibt den 48-Byte-Header an den Anfang der Datei.
    pub fn write_header(&mut self, file: &dyn File) -> Result<(), BlobWriteError> {
        let header = BlobHeader::new(self.blob_no, self.generation, self.space);
        let bytes = header.encode();
        file.write_all_at(&bytes, 0)?;
        self.offset = u64::try_from(BLOB_HEADER_LEN).map_err(|_| BlobWriteError::Overflow)?;
        self.header_written = true;
        Ok(())
    }

    /// Hängt einen Record an die Blob-Datei an und liefert die eindeutige BlobRef.
    #[allow(clippy::too_many_arguments)]
    pub fn append(
        &mut self,
        file: &dyn File,
        record: &BlobRecord,
        space_dek: Option<&[u8; 32]>,
        domain_key: Option<&[u8; 32]>,
        cipher: Option<&dyn Cipher>,
        mac: Option<&dyn Mac>,
        kdf: Option<&dyn Kdf>,
    ) -> Result<BlobRef, BlobWriteError> {
        if !self.header_written {
            self.write_header(file)?;
        }

        let record_offset = self.offset;

        // Domänenversiegelung, falls gefordert
        let mut final_record = record.clone();
        if let Some(dk) = domain_key {
            let c = cipher.ok_or(BlobWriteError::CryptoMissing("Cipher für Domänenversiegelung"))?;
            let offset_u32 = u32::try_from(record_offset & 0xFFFF_FFFF).unwrap_or(0);
            let nonce = blob_body_nonce(self.blob_no, offset_u32);

            let mut aad = Vec::new();
            aad.extend_from_slice(&self.space.to_le_bytes());
            aad.extend_from_slice(&record.domain.to_le_bytes());
            aad.extend_from_slice(&record.key);

            let sealed_body = c.seal(dk, &nonce, &aad, &record.body);
            final_record.body = sealed_body;
        }

        let content = final_record.encode_content();
        let rlen_u32 = u32::try_from(content.len()).map_err(|_| BlobWriteError::Overflow)?;
        let content_crc = crc32c(0, &content);

        let mut record_bytes = Vec::new();
        record_bytes.extend_from_slice(&rlen_u32.to_le_bytes());

        match self.integrity_mode {
            0 => {
                // Plain: content || crc32c
                let chk = crc32c(0, &content);
                record_bytes.extend_from_slice(&content);
                record_bytes.extend_from_slice(&chk.to_le_bytes());
            }
            1 => {
                // Authenticated: content || mac16
                let m = mac.ok_or(BlobWriteError::CryptoMissing("Mac"))?;
                let dek = space_dek.ok_or(BlobWriteError::CryptoMissing("Space DEK"))?;
                let blob_no_b = self.blob_no.to_le_bytes();
                let offset_b = record_offset.to_le_bytes();
                let tag = m.mac16(dek, &[&content, &blob_no_b, &offset_b]);
                record_bytes.extend_from_slice(&content);
                record_bytes.extend_from_slice(&tag);
            }
            2 => {
                // Encrypted: Cipher.seal(Kdf(space_dek, ...))
                let c = cipher.ok_or(BlobWriteError::CryptoMissing("Cipher"))?;
                let k = kdf.ok_or(BlobWriteError::CryptoMissing("Kdf"))?;
                let dek = space_dek.ok_or(BlobWriteError::CryptoMissing("Space DEK"))?;

                let blob_key = k.derive(dek, &[b"ctxs blob v1", &self.blob_no.to_le_bytes()]);
                let nonce = blob_record_nonce(record_offset);

                let mut aad = Vec::new();
                aad.extend_from_slice(&self.space.to_le_bytes());
                aad.extend_from_slice(&self.blob_no.to_le_bytes());
                aad.extend_from_slice(&record_offset.to_le_bytes());

                let sealed = c.seal(&blob_key, &nonce, &aad, &content);
                record_bytes.extend_from_slice(&sealed);
            }
            other => return Err(BlobWriteError::InvalidIntegrityMode(other)),
        }

        file.write_all_at(&record_bytes, record_offset)?;

        let bytes_written_u64 =
            u64::try_from(record_bytes.len()).map_err(|_| BlobWriteError::Overflow)?;
        self.offset = self
            .offset
            .checked_add(bytes_written_u64)
            .ok_or(BlobWriteError::Overflow)?;
        self.record_count = self
            .record_count
            .checked_add(1)
            .ok_or(BlobWriteError::Overflow)?;

        // INV-BLOB-2: Kein Teilen: ein BlobRef <-> ein Record
        Ok(BlobRef {
            blob_no: self.blob_no,
            offset: record_offset,
            len: rlen_u32,
            crc32c: content_crc,
            domain: record.domain,
        })
    }

    /// Schließt die Blob-Datei mit dem 20-Byte-Footer ab und synchronisiert sie.
    pub fn finish(&mut self, file: &dyn File) -> Result<u64, BlobWriteError> {
        if !self.header_written {
            self.write_header(file)?;
        }

        let footer = BlobFooter::new(self.record_count, self.offset);
        let footer_bytes = footer.encode();
        file.write_all_at(&footer_bytes, self.offset)?;

        let footer_len_u64 =
            u64::try_from(BLOB_FOOTER_LEN).map_err(|_| BlobWriteError::Overflow)?;
        let total_bytes = self
            .offset
            .checked_add(footer_len_u64)
            .ok_or(BlobWriteError::Overflow)?;

        file.sync_all()?;
        Ok(total_bytes)
    }
}
