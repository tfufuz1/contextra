//! # Katalog-Log und Persistenz
//!
//! SPEC §9.7: Datei-Header (20 B), Records mit Prüfsummen-Kette, Torn-Tail-Toleranz (INV-CAT-3).

use crate::catalog::durable::Synced;
use crate::catalog::edit::CatalogEdit;
pub use crate::catalog::rollover::rollover;
use crate::catalog::state::{apply_edit, CatalogState};
use crate::catalog::types::CatalogError;
use crate::codec::crc32c::crc32c;
use crate::keys::nonce::catalog_record_nonce;
use contextra_storage_ports::{Cipher, File, Kdf, Mac};

/// Magic-Bytes für Katalog-Dateien ("CTXC").
pub const CATALOG_MAGIC: &[u8; 4] = b"CTXC";

/// Länge des 20-Byte-Headers einer Katalog-Datei.
pub const CATALOG_HEADER_LEN: usize = 20;
/// Länge des 20-Byte-Headers einer Katalog-Datei als u64.
pub const CATALOG_HEADER_LEN_U64: u64 = 20;

/// Tag für Snapshot-Record (1).
pub const RECORD_TAG_SNAPSHOT: u8 = 1;

/// Tag für Edit-Record (2).
pub const RECORD_TAG_EDIT: u8 = 2;

/// Header einer Katalog-Datei nach SPEC §9.7 (20 Bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogHeader {
    /// Formatversion (aktuell 1).
    pub version: u16,
    /// Flags (aktuell 0).
    pub flags: u16,
    /// Fortlaufende Nummer der Katalog-Datei.
    pub catalog_no: u64,
}

impl CatalogHeader {
    /// Erstellt einen neuen Katalog-Header.
    pub fn new(catalog_no: u64) -> Self {
        Self { version: 1, flags: 0, catalog_no }
    }

    /// Kodiert den Header in ein 20-Byte-Array.
    pub fn encode(&self) -> [u8; CATALOG_HEADER_LEN] {
        let mut buf = [0u8; CATALOG_HEADER_LEN];
        let (data, crc_slot) = buf.split_at_mut(16);

        let (magic_slot, rest) = data.split_at_mut(4);
        magic_slot.copy_from_slice(CATALOG_MAGIC);

        let (ver_slot, rest) = rest.split_at_mut(2);
        ver_slot.copy_from_slice(&self.version.to_le_bytes());

        let (flags_slot, no_slot) = rest.split_at_mut(2);
        flags_slot.copy_from_slice(&self.flags.to_le_bytes());
        no_slot.copy_from_slice(&self.catalog_no.to_le_bytes());

        let chk = crc32c(0, data);
        crc_slot.copy_from_slice(&chk.to_le_bytes());

        buf
    }

    /// Dekodiert und verifiziert einen 20-Byte-Header.
    pub fn decode(buf: &[u8]) -> Result<Self, CatalogError> {
        if buf.len() < CATALOG_HEADER_LEN {
            return Err(CatalogError::UnexpectedEof);
        }
        let (slice, _) = buf.split_at(CATALOG_HEADER_LEN);

        let (magic_slot, rest) = slice.split_at(4);
        if magic_slot != CATALOG_MAGIC {
            return Err(CatalogError::Corrupt("Ungültige Katalog-Magic"));
        }

        let (ver_slot, rest) = rest.split_at(2);
        let version = u16::from_le_bytes(ver_slot.try_into().map_err(|_| CatalogError::UnexpectedEof)?);
        if version != 1 {
            return Err(CatalogError::Corrupt("Nicht unterstützte Katalog-Version"));
        }

        let (flags_slot, rest) = rest.split_at(2);
        let flags = u16::from_le_bytes(flags_slot.try_into().map_err(|_| CatalogError::UnexpectedEof)?);

        let (no_slot, crc_slot) = rest.split_at(8);
        let catalog_no = u64::from_le_bytes(no_slot.try_into().map_err(|_| CatalogError::UnexpectedEof)?);

        let expected_chk = u32::from_le_bytes(crc_slot.try_into().map_err(|_| CatalogError::UnexpectedEof)?);
        let (data, _) = slice.split_at(16);
        let actual_chk = crc32c(0, data);
        if actual_chk != expected_chk {
            return Err(CatalogError::Corrupt("Header-Prüfsummenfehler im Katalog"));
        }

        Ok(Self { version, flags, catalog_no })
    }
}

/// Faltet catalog_no (u64) zu einem 32-Bit-Seed (SPEC §9.7).
pub fn fold_u64_to_u32(val: u64) -> u32 {
    let low = u32::try_from(val & 0xFFFF_FFFF).unwrap_or(0);
    let high = u32::try_from((val >> 32) & 0xFFFF_FFFF).unwrap_or(0);
    low ^ high
}

/// Liefert den kanonischen Dateinamen für eine Katalog-Datei ("CATALOG-000007").
pub fn catalog_file_name(catalog_no: u64) -> String {
    format!("CATALOG-{catalog_no:06}")
}

/// Verwalter für das Lesen und Schreiben von Katalog-Records.
#[derive(Debug)]
pub struct CatalogLog {
    catalog_no: u64,
    integrity_mode: u8,
    offset: u64,
    record_index: u64,
    chk_vorher: u32,
    head_mac: [u8; 16],
}

impl CatalogLog {
    /// Erstellt einen neuen CatalogLog-Zustand für eine gegebene Katalognummer.
    pub fn new(catalog_no: u64) -> Self {
        Self {
            catalog_no,
            integrity_mode: 0,
            offset: CATALOG_HEADER_LEN_U64,
            record_index: 0,
            chk_vorher: fold_u64_to_u32(catalog_no),
            head_mac: [0u8; 16],
        }
    }

    /// Öffnet eine bestehende Katalog-Datei, liest alle Records und baut den Zustand auf.
    ///
    /// INV-CAT-3: Nur der letzte Record darf Torn Write sein und wird verworfen.
    /// Jeder frühere Defekt führt zu CatalogError::Corrupt.
    #[allow(clippy::too_many_arguments)]
    pub fn read_and_recover(
        file: &dyn File,
        catalog_no: u64,
        integrity_mode: u8,
        root_key: Option<&[u8; 32]>,
        cipher: Option<&dyn Cipher>,
        mac: Option<&dyn Mac>,
        kdf: Option<&dyn Kdf>,
    ) -> Result<(CatalogState, Self), CatalogError> {
        let file_len = file.len().map_err(CatalogError::from)?;
        let mut header_buf = [0u8; CATALOG_HEADER_LEN];
        file.read_at(&mut header_buf, 0).map_err(CatalogError::from)?;
        let header = CatalogHeader::decode(&header_buf)?;

        if header.catalog_no != catalog_no {
            return Err(CatalogError::Corrupt("Catalog-Nummer weicht ab"));
        }

        let mut state = CatalogState::default();
        let mut offset = CATALOG_HEADER_LEN_U64;
        let mut record_index = 0u64;
        let mut chk_vorher = fold_u64_to_u32(catalog_no);
        let mut head_mac = [0u8; 16];

        let tag_len_u64: u64 = match integrity_mode {
            0 => 4,
            _ => 16,
        };

        while offset < file_len {
            let remaining_file_bytes = file_len.saturating_sub(offset);

            // 1. Prüfen, ob noch 4 Bytes für die Länge lesbar sind
            if remaining_file_bytes < 4 {
                // INV-CAT-3: Torn Write am Ende wird verworfen
                break;
            }

            let mut len_b = [0u8; 4];
            file.read_at(&mut len_b, offset).map_err(CatalogError::from)?;
            let len = u32::from_le_bytes(len_b);
            let len_u64 = u64::from(len);

            let total_record_len = match 4u64.checked_add(len_u64).and_then(|l| l.checked_add(tag_len_u64)) {
                Some(l) => l,
                None => return Err(CatalogError::Corrupt("Record-Länge überläuft")),
            };

            if remaining_file_bytes < total_record_len {
                // INV-CAT-3: Unvollständiger Record am Ende wird verworfen
                break;
            }

            let payload_len_u64 = len_u64.saturating_sub(1);
            let payload_len = usize::try_from(payload_len_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;

            let mut tag_b = [0u8; 1];
            file.read_at(&mut tag_b, offset.saturating_add(4)).map_err(CatalogError::from)?;
            let tag = tag_b.first().copied().unwrap_or(0);

            let payload_offset = offset.saturating_add(5);
            let mut payload = vec![0u8; payload_len];
            file.read_at(&mut payload, payload_offset).map_err(CatalogError::from)?;

            let chk_offset = payload_offset.saturating_add(payload_len_u64);
            let is_last_in_file = offset.saturating_add(total_record_len) >= file_len;

            // Prüfwert verifizieren
            let decode_res = (|| -> Result<CatalogEdit, CatalogError> {
                match integrity_mode {
                    0 => {
                        let mut chk_b = [0u8; 4];
                        file.read_at(&mut chk_b, chk_offset).map_err(CatalogError::from)?;
                        let expected_chk = u32::from_le_bytes(chk_b);

                        let mut check_data = Vec::with_capacity(1 + payload.len());
                        check_data.push(tag);
                        check_data.extend_from_slice(&payload);

                        let actual_chk = crc32c(chk_vorher, &check_data);
                        if actual_chk != expected_chk {
                            return Err(CatalogError::Corrupt("CRC32C-Kettenfehler im Catalog-Record"));
                        }
                        chk_vorher = actual_chk;
                        CatalogEdit::decode(&payload)
                    }
                    1 => {
                        let m = mac.ok_or(CatalogError::Corrupt("Mac fehlt"))?;
                        let rk = root_key.ok_or(CatalogError::Corrupt("Root-Key fehlt"))?;

                        let mut tag_buf = [0u8; 16];
                        file.read_at(&mut tag_buf, chk_offset).map_err(CatalogError::from)?;

                        let cv_b = chk_vorher.to_le_bytes();
                        let l_b = len.to_le_bytes();
                        let t_b = [tag];
                        let c_no_b = catalog_no.to_le_bytes();
                        let idx_b = record_index.to_le_bytes();

                        if !m.verify16(rk, &[&cv_b, &l_b, &t_b, &payload, &c_no_b, &idx_b], &tag_buf) {
                            return Err(CatalogError::Corrupt("MAC-Kettenfehler im Catalog-Record"));
                        }
                        head_mac = tag_buf;
                        chk_vorher = fold_u64_to_u32(u64::from_le_bytes(tag_buf[..8].try_into().unwrap_or([0; 8])));
                        CatalogEdit::decode(&payload)
                    }
                    2 => {
                        let c = cipher.ok_or(CatalogError::Corrupt("Cipher fehlt"))?;
                        let k = kdf.ok_or(CatalogError::Corrupt("Kdf fehlt"))?;
                        let rk = root_key.ok_or(CatalogError::Corrupt("Root-Key fehlt"))?;

                        let mut sealed = vec![0u8; payload_len.saturating_add(16)];
                        file.read_at(&mut sealed, payload_offset).map_err(CatalogError::from)?;

                        let cat_key = k.derive(rk, &[b"ctxs cat v1", &catalog_no.to_le_bytes()]);
                        let nonce = catalog_record_nonce(record_index);

                        let mut aad = Vec::new();
                        aad.extend_from_slice(&chk_vorher.to_le_bytes());
                        aad.extend_from_slice(&len.to_le_bytes());
                        aad.extend_from_slice(&catalog_no.to_le_bytes());
                        aad.extend_from_slice(&record_index.to_le_bytes());

                        let plain = c.open(&cat_key, &nonce, &aad, &sealed)
                            .map_err(|_| CatalogError::Corrupt("Catalog-Record-Entschlüsselung fehlgeschlagen"))?;
                        CatalogEdit::decode(&plain)
                    }
                    _ => Err(CatalogError::Corrupt("Ungültiger Integritätsmodus")),
                }
            })();

            match decode_res {
                Ok(edit) => {
                    // P-2 / INV-CAT-2: Edit atomar anwenden
                    state = apply_edit(&state, &edit)?;
                    offset = offset.saturating_add(total_record_len);
                    record_index = record_index.saturating_add(1);
                }
                Err(err) => {
                    if is_last_in_file {
                        // INV-CAT-3: Letzter defekter Record wird verworfen
                        break;
                    } else {
                        // INV-CAT-3: Früherer Defekt führt zwingend zu Corrupt
                        return Err(err);
                    }
                }
            }
        }

        let log = Self {
            catalog_no,
            integrity_mode,
            offset,
            record_index,
            chk_vorher,
            head_mac,
        };

        Ok((state, log))
    }

    /// Schreibt einen neuen atomaren Edit in die Katalog-Datei (M-5).
    ///
    /// M-5: Der Katalog-Commit nimmt ausschließlich Synced entgegen.
    #[allow(clippy::too_many_arguments)]
    pub fn commit(
        &mut self,
        file: &dyn File,
        edit: Synced<CatalogEdit>,
        root_key: Option<&[u8; 32]>,
        cipher: Option<&dyn Cipher>,
        mac: Option<&dyn Mac>,
        kdf: Option<&dyn Kdf>,
    ) -> Result<(), CatalogError> {
        let edit_val = edit.into_inner();
        let payload = edit_val.encode();

        let payload_len_u32 = u32::try_from(payload.len())
            .map_err(|_| CatalogError::InvalidFormat("Payload-Überlauf"))?;
        let len = payload_len_u32.checked_add(1)
            .ok_or(CatalogError::InvalidFormat("Längen-Überlauf"))?;

        let tag = RECORD_TAG_EDIT;

        let mut record_bytes = Vec::new();
        record_bytes.extend_from_slice(&len.to_le_bytes());
        record_bytes.push(tag);

        match self.integrity_mode {
            0 => {
                let mut check_data = Vec::with_capacity(1 + payload.len());
                check_data.push(tag);
                check_data.extend_from_slice(&payload);

                let chk = crc32c(self.chk_vorher, &check_data);
                self.chk_vorher = chk;

                record_bytes.extend_from_slice(&payload);
                record_bytes.extend_from_slice(&chk.to_le_bytes());
            }
            1 => {
                let m = mac.ok_or(CatalogError::Corrupt("Mac fehlt"))?;
                let rk = root_key.ok_or(CatalogError::Corrupt("Root-Key fehlt"))?;

                let cv_b = self.chk_vorher.to_le_bytes();
                let l_b = len.to_le_bytes();
                let t_b = [tag];
                let c_no_b = self.catalog_no.to_le_bytes();
                let idx_b = self.record_index.to_le_bytes();

                let tag_out = m.mac16(rk, &[&cv_b, &l_b, &t_b, &payload, &c_no_b, &idx_b]);
                self.head_mac = tag_out;
                self.chk_vorher = fold_u64_to_u32(u64::from_le_bytes(tag_out[..8].try_into().unwrap_or([0; 8])));

                record_bytes.extend_from_slice(&payload);
                record_bytes.extend_from_slice(&tag_out);
            }
            2 => {
                let c = cipher.ok_or(CatalogError::Corrupt("Cipher fehlt"))?;
                let k = kdf.ok_or(CatalogError::Corrupt("Kdf fehlt"))?;
                let rk = root_key.ok_or(CatalogError::Corrupt("Root-Key fehlt"))?;

                let cat_key = k.derive(rk, &[b"ctxs cat v1", &self.catalog_no.to_le_bytes()]);
                let nonce = catalog_record_nonce(self.record_index);

                let mut aad = Vec::new();
                aad.extend_from_slice(&self.chk_vorher.to_le_bytes());
                aad.extend_from_slice(&len.to_le_bytes());
                aad.extend_from_slice(&self.catalog_no.to_le_bytes());
                aad.extend_from_slice(&self.record_index.to_le_bytes());

                let sealed = c.seal(&cat_key, &nonce, &aad, &payload);
                record_bytes.extend_from_slice(&sealed);
            }
            _ => return Err(CatalogError::Corrupt("Ungültiger Integritätsmodus")),
        }

        file.write_all_at(&record_bytes, self.offset).map_err(CatalogError::from)?;
        file.sync_data().map_err(CatalogError::from)?;

        let bytes_written = u64::try_from(record_bytes.len())
            .map_err(|_| CatalogError::InvalidFormat("Längenüberlauf"))?;
        self.offset = self.offset.saturating_add(bytes_written);
        self.record_index = self.record_index.saturating_add(1);

        Ok(())
    }
}
