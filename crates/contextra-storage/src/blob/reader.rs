//! # Blob-Reader
//!
//! Liest und verifiziert Records aus Blob-Dateien nach SPEC §9.6.

use crate::blob::record::{
    BlobFooter, BlobHeader, BlobRecord, BlobRef, BLOB_FOOTER_LEN, BLOB_HEADER_LEN,
    RECORD_KIND_HOLE,
};
use crate::codec::crc32c::crc32c;
use crate::keys::nonce::{blob_body_nonce, blob_record_nonce};
use contextra_storage_ports::{Cipher, File, Kdf, Mac};
use std::fmt;
use std::io;

/// Fehler beim Lesen eines Blob-Records.
#[derive(Debug)]
pub enum BlobReadError {
    /// E/A-Fehler.
    Io(io::Error),
    /// Referenz zeigt auf ein Hole (INV-BLOB-3).
    HoleRef,
    /// Datenbeschädigung oder Verifikationsfehler (INV-BLOB-1).
    Corrupt(&'static str),
    /// Schlüssel- oder Prüfwertfehler.
    CryptoFailed(&'static str),
    /// Nicht unterstützter Modus.
    UnsupportedMode(u8),
    /// Arithmetischer Überlauf.
    Overflow,
}

impl fmt::Display for BlobReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "Blob-Lese-E/A-Fehler: {e}"),
            Self::HoleRef => write!(f, "INV-BLOB-3: BlobRef zeigt auf ein gelöschtes Hole"),
            Self::Corrupt(msg) => write!(f, "INV-BLOB-1: Blob-Daten beschädigt: {msg}"),
            Self::CryptoFailed(msg) => write!(f, "Blob-Entschlüsselung/MAC fehlgeschlagen: {msg}"),
            Self::UnsupportedMode(m) => write!(f, "Nicht unterstützter Integritätsmodus: {m}"),
            Self::Overflow => write!(f, "Überlauf bei Lese-Offset-Berechnung"),
        }
    }
}

impl std::error::Error for BlobReadError {}

impl From<io::Error> for BlobReadError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

/// Reader für den wahlfreien Zugriff auf Blob-Records.
pub struct BlobReader {
    header: BlobHeader,
    integrity_mode: u8,
}

impl BlobReader {
    /// Öffnet die Blob-Datei und liest den Header.
    pub fn open(file: &dyn File, integrity_mode: u8) -> Result<Self, BlobReadError> {
        let mut header_buf = [0u8; BLOB_HEADER_LEN];
        file.read_at(&mut header_buf, 0)?;
        let header = BlobHeader::decode(&header_buf)
            .map_err(|_| BlobReadError::Corrupt("Ungültiger Blob-Header"))?;

        Ok(Self {
            header,
            integrity_mode,
        })
    }

    /// Liefert den gelesenen Datei-Header.
    pub fn header(&self) -> &BlobHeader {
        &self.header
    }

    /// Liest den Footer der Blob-Datei am Dateiende.
    pub fn read_footer(&self, file: &dyn File) -> Result<BlobFooter, BlobReadError> {
        let file_len = file.len()?;
        let footer_len_u64 =
            u64::try_from(BLOB_FOOTER_LEN).map_err(|_| BlobReadError::Overflow)?;
        let footer_offset = file_len
            .checked_sub(footer_len_u64)
            .ok_or(BlobReadError::Corrupt("Datei zu kurz für Blob-Footer"))?;

        let mut buf = [0u8; BLOB_FOOTER_LEN];
        file.read_at(&mut buf, footer_offset)?;
        BlobFooter::decode(&buf).map_err(|_| BlobReadError::Corrupt("Ungültiger Blob-Footer"))
    }

    /// Liest und prüft einen Record anhand von `BlobRef`, `expected_key` und `expected_stamp`.
    ///
    /// INV-BLOB-1: Jeder BlobRef löst auf genau einen Record gleichen Schlüssels/Stamps auf.
    /// INV-BLOB-3: Ein BlobRef auf ein Hole liefert HoleRef.
    #[allow(clippy::too_many_arguments)]
    pub fn read(
        &self,
        file: &dyn File,
        blob_ref: &BlobRef,
        expected_key: &[u8],
        expected_stamp: u64,
        space_dek: Option<&[u8; 32]>,
        domain_key: Option<&[u8; 32]>,
        cipher: Option<&dyn Cipher>,
        mac: Option<&dyn Mac>,
        kdf: Option<&dyn Kdf>,
    ) -> Result<Vec<u8>, BlobReadError> {
        if blob_ref.blob_no != self.header.blob_no {
            // INV-BLOB-1: Falsche Blob-Nummer
            return Err(BlobReadError::Corrupt("BlobRef blob_no stimmt nicht überein"));
        }

        // 1. rlen lesen (4 Bytes)
        let mut rlen_b = [0u8; 4];
        file.read_at(&mut rlen_b, blob_ref.offset)?;
        let rlen = u32::from_le_bytes(rlen_b);
        if rlen != blob_ref.len {
            // INV-BLOB-1: Längenabweichung
            return Err(BlobReadError::Corrupt("Record-Länge weicht von BlobRef ab"));
        }

        let rlen_usize = usize::try_from(rlen).map_err(|_| BlobReadError::Overflow)?;
        let rlen_offset = blob_ref
            .offset
            .checked_add(4)
            .ok_or(BlobReadError::Overflow)?;

        // 2. Ersten Byte des Payloads lesen zur Hole-Prüfung
        let mut kind_buf = [0u8; 1];
        file.read_at(&mut kind_buf, rlen_offset)?;
        let first_byte = kind_buf.first().copied().unwrap_or(0);

        if first_byte == RECORD_KIND_HOLE {
            // INV-BLOB-3: Hole-Records sind nie verschlüsselt; BlobRef auf ein Hole liefert HoleRef
            return Err(BlobReadError::HoleRef);
        }

        // 3. Nutzinhalt und Prüfwert lesen je nach Modus
        let content = match self.integrity_mode {
            0 => {
                // Plain: content (rlen) || crc32c (4B)
                let mut data = vec![0u8; rlen_usize];
                file.read_at(&mut data, rlen_offset)?;

                let chk_offset = rlen_offset
                    .checked_add(u64::from(rlen))
                    .ok_or(BlobReadError::Overflow)?;
                let mut chk_b = [0u8; 4];
                file.read_at(&mut chk_b, chk_offset)?;
                let expected_chk = u32::from_le_bytes(chk_b);

                let actual_chk = crc32c(0, &data);
                if actual_chk != expected_chk || actual_chk != blob_ref.crc32c {
                    return Err(BlobReadError::Corrupt("CRC32C-Prüfsummenfehler im Record"));
                }
                data
            }
            1 => {
                // Authenticated: content (rlen) || mac16 (16B)
                let m = mac.ok_or(BlobReadError::CryptoFailed("Mac-Port fehlt"))?;
                let dek = space_dek.ok_or(BlobReadError::CryptoFailed("Space DEK fehlt"))?;

                let mut data = vec![0u8; rlen_usize];
                file.read_at(&mut data, rlen_offset)?;

                let tag_offset = rlen_offset
                    .checked_add(u64::from(rlen))
                    .ok_or(BlobReadError::Overflow)?;
                let mut tag = [0u8; 16];
                file.read_at(&mut tag, tag_offset)?;

                let blob_no_b = self.header.blob_no.to_le_bytes();
                let offset_b = blob_ref.offset.to_le_bytes();
                if !m.verify16(dek, &[&data, &blob_no_b, &offset_b], &tag) {
                    return Err(BlobReadError::Corrupt("MAC-Verifikation fehlgeschlagen"));
                }
                data
            }
            2 => {
                // Encrypted: Ciphertext + 16B Tag = rlen + 16
                let c = cipher.ok_or(BlobReadError::CryptoFailed("Cipher-Port fehlt"))?;
                let k = kdf.ok_or(BlobReadError::CryptoFailed("Kdf-Port fehlt"))?;
                let dek = space_dek.ok_or(BlobReadError::CryptoFailed("Space DEK fehlt"))?;

                let total_sealed_len = rlen_usize
                    .checked_add(16)
                    .ok_or(BlobReadError::Overflow)?;
                let mut sealed = vec![0u8; total_sealed_len];
                file.read_at(&mut sealed, rlen_offset)?;

                let blob_key =
                    k.derive(dek, &[b"ctxs blob v1", &self.header.blob_no.to_le_bytes()]);
                let nonce = blob_record_nonce(blob_ref.offset);

                let mut aad = Vec::new();
                aad.extend_from_slice(&self.header.space.to_le_bytes());
                aad.extend_from_slice(&self.header.blob_no.to_le_bytes());
                aad.extend_from_slice(&blob_ref.offset.to_le_bytes());

                c.open(&blob_key, &nonce, &aad, &sealed)
                    .map_err(|_| BlobReadError::Corrupt("AEAD-Entschlüsselung fehlgeschlagen"))?
            }
            other => return Err(BlobReadError::UnsupportedMode(other)),
        };

        // 4. Record-Inhalt dekodieren
        let record = BlobRecord::decode_content(&content)
            .map_err(|_| BlobReadError::Corrupt("Record-Dekodierung fehlgeschlagen"))?;

        // 5. INV-BLOB-1: Schlüssel und Stamp gegen BlobRef prüfen
        if record.stamp != expected_stamp {
            return Err(BlobReadError::Corrupt("Record-Stamp stimmt nicht mit Erwartung überein"));
        }
        if record.key != expected_key {
            return Err(BlobReadError::Corrupt("Record-Schlüssel stimmt nicht mit Erwartung überein"));
        }

        // 6. Domänenentsiegelung, falls Domänenschlüssel übergeben
        let final_body = if let Some(dk) = domain_key {
            let c = cipher.ok_or(BlobReadError::CryptoFailed("Cipher für Domänenentsiegelung fehlt"))?;
            let offset_u32 = u32::try_from(blob_ref.offset & 0xFFFF_FFFF).unwrap_or(0);
            let nonce = blob_body_nonce(self.header.blob_no, offset_u32);

            let mut aad = Vec::new();
            aad.extend_from_slice(&self.header.space.to_le_bytes());
            aad.extend_from_slice(&record.domain.to_le_bytes());
            aad.extend_from_slice(&record.key);

            c.open(dk, &nonce, &aad, &record.body)
                .map_err(|_| BlobReadError::Corrupt("Domänenentsiegelung fehlgeschlagen"))?
        } else {
            record.body
        };

        Ok(final_body)
    }
}
