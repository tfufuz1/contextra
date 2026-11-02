//! F.2 HEADER.a und HEADER.b (1024 Byte) Parser.
//!
//! SPEC Anhang F.2:
//! - 0..8: magic "CTXHDR\0\1"
//! - 8..10: format_version u16 (v1 = 1)
//! - 10..11: kind = 0x20
//! - 11..15: alg_ids u8[4]
//! - 15..31: store_id [u8; 16]
//! - 31..39: generation u64
//! - 39..40: slot_count u8 (1..=4)
//! - 40..72: report_pub [u8; 32] (Ed25519)
//! - 72..584: 4 * 128 Byte Slots
//! - 584..600: check [u8; 16] = SHA-256(Bytes[0..584))[0..16]
//! - 600..632: header_mac [u8; 32] = HMAC-SHA256(K_hdr, Bytes[0..584)),
//!   wobei K_hdr = HKDF(K_master, "ctx/hdr/v1")
//! - 632..1024: padding 392 Bytes = 0

use aes_gcm_siv::aead::{Aead, KeyInit, Payload};
use aes_gcm_siv::{Aes256GcmSiv, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use hkdf::Hkdf;
use hmac::Hmac;
use sha2::digest::Mac;
use sha2::{Digest, Sha256};

use crate::error::LabError;

/// Feste Kopfgröße für HEADER.a / HEADER.b.
pub const HEADER_AB_SIZE: usize = 1024;

/// Magische Bytes für F.2.
pub const HEADER_AB_MAGIC: &[u8; 8] = &[0x43, 0x54, 0x58, 0x48, 0x44, 0x52, 0x00, 0x01];

/// Kind für HEADER.a/b.
pub const HEADER_AB_KIND: u8 = 0x20;

/// Slot (128 Byte).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderSlot {
    /// Slot-ID.
    pub slot_id: u8,
    /// Anker-ID.
    pub anchor_id: u8,
    /// KDF-ID.
    pub kdf_id: u8,
    /// Status: 0 leer, 1 aktiv, 2 entfernt.
    pub status: u8,
    /// Speicheraufwand m in KiB.
    pub m_kib: u32,
    /// Zeitaufwand t.
    pub t: u8,
    /// Parallelitätsgrad p.
    pub p: u8,
    /// Slot-Salt (32 Byte).
    pub slot_salt: [u8; 32],
    /// Nonce (12 Byte).
    pub nonce: [u8; 12],
    /// Verpackter Hauptschlüssel (48 Byte: 32 B Chiffrat + 16 B Tag).
    pub wrapped_master: [u8; 48],
}

impl HeaderSlot {
    /// Parst einen 128-Byte-Slot.
    pub fn parse(bytes: &[u8]) -> Result<Self, LabError> {
        if bytes.len() < 128 {
            return Err(LabError::UnexpectedEof {
                expected: 128,
                actual: bytes.len(),
            });
        }

        let slot_id = *bytes.first().ok_or(LabError::Corrupted("SlotId fehlt"))?;
        let anchor_id = *bytes.get(1).ok_or(LabError::Corrupted("AnchorId fehlt"))?;
        let kdf_id = *bytes.get(2).ok_or(LabError::Corrupted("KdfId fehlt"))?;
        let status = *bytes.get(3).ok_or(LabError::Corrupted("Status fehlt"))?;

        if status > 2 {
            return Err(LabError::Corrupted("Ungültiger Slot-Status"));
        }

        let m_kib = u32::from_be_bytes(
            bytes
                .get(4..8)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("m_kib fehlt"))?,
        );
        let t = *bytes.get(8).ok_or(LabError::Corrupted("t fehlt"))?;
        let p = *bytes.get(9).ok_or(LabError::Corrupted("p fehlt"))?;

        // Validierung der KDF-Grenzen nach D-06 für aktive Slots:
        // m in [65536, 4194304] KiB, t in [3, 16], p in [1, 16]
        if status == 1 {
            if !(65_536..=4_194_304).contains(&m_kib) {
                return Err(LabError::Corrupted("m_kib außerhalb D-06 Grenzen"));
            }
            if !(3..=16).contains(&t) {
                return Err(LabError::Corrupted("t außerhalb D-06 Grenzen"));
            }
            if !(1..=16).contains(&p) {
                return Err(LabError::Corrupted("p außerhalb D-06 Grenzen"));
            }
        }

        let reserved = bytes
            .get(10..12)
            .ok_or(LabError::Corrupted("Reserved fehlt"))?;
        if reserved.iter().any(|&b| b != 0) {
            return Err(LabError::Corrupted("Reserved im Slot ist nicht 0"));
        }

        let slot_salt: [u8; 32] = bytes
            .get(12..44)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("SlotSalt fehlt"))?;

        let nonce: [u8; 12] = bytes
            .get(44..56)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("Nonce fehlt"))?;

        let wrapped_master: [u8; 48] = bytes
            .get(56..104)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("WrappedMaster fehlt"))?;

        let zero_pad = bytes
            .get(104..128)
            .ok_or(LabError::Corrupted("Slot Padding fehlt"))?;
        if zero_pad.iter().any(|&b| b != 0) {
            return Err(LabError::Corrupted("Slot Padding ist nicht 0"));
        }

        Ok(Self {
            slot_id,
            anchor_id,
            kdf_id,
            status,
            m_kib,
            t,
            p,
            slot_salt,
            nonce,
            wrapped_master,
        })
    }

    /// Entschlüsselt den Hauptschlüssel aus einem aktiven Slot via Passphrase.
    /// KEK = Argon2id(passphrase, slot_salt, m, t, p) -> 32 B
    /// AAD = "ctx/slot/v1" || store_id || slot_id || anchor_id || kdf_id || u32(m) || u8(t) || u8(p) || slot_salt
    pub fn unwrap_master_key(
        &self,
        passphrase: &[u8],
        store_id: &[u8; 16],
    ) -> Result<[u8; 32], LabError> {
        if self.status != 1 {
            return Err(LabError::Corrupted("Slot ist nicht aktiv"));
        }

        let params = Params::new(self.m_kib, self.t as u32, self.p as u32, Some(32))
            .map_err(|_| LabError::Corrupted("Ungültige Argon2 Parameter"))?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut kek = [0u8; 32];
        argon2
            .hash_password_into(passphrase, &self.slot_salt, &mut kek)
            .map_err(|_| LabError::Corrupted("Argon2 KEK Ableitung fehlgeschlagen"))?;

        let mut aad = Vec::with_capacity(66);
        aad.extend_from_slice(b"ctx/slot/v1");
        aad.extend_from_slice(store_id);
        aad.push(self.slot_id);
        aad.push(self.anchor_id);
        aad.push(self.kdf_id);
        aad.extend_from_slice(&self.m_kib.to_be_bytes());
        aad.push(self.t);
        aad.push(self.p);
        aad.extend_from_slice(&self.slot_salt);

        let cipher = Aes256GcmSiv::new_from_slice(&kek)
            .map_err(|_| LabError::Corrupted("Ungültige KEK-Länge"))?;

        let payload = Payload {
            msg: &self.wrapped_master,
            aad: &aad,
        };

        let pt = cipher
            .decrypt(Nonce::from_slice(&self.nonce), payload)
            .map_err(|_| LabError::VerificationFailed("Master-Key Unwrap fehlgeschlagen"))?;

        if pt.len() != 32 {
            return Err(LabError::Corrupted(
                "Entschlüsselter Master-Key hat falsche Länge",
            ));
        }

        let mut out = [0u8; 32];
        if let Some(dest) = out.get_mut(..) {
            dest.copy_from_slice(&pt);
        }
        Ok(out)
    }
}

/// Strukturierter HEADER.a / HEADER.b Inhalt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderAb {
    /// Formatversion.
    pub format_version: u16,
    /// Store-ID.
    pub store_id: [u8; 16],
    /// Generationsnummer.
    pub generation: u64,
    /// Anzahl belegter Slots (1..=4).
    pub slot_count: u8,
    /// Öffentlicher Berichtsschlüssel (Ed25519).
    pub report_pub: [u8; 32],
    /// 4 Slots.
    pub slots: Vec<HeaderSlot>,
    /// SHA-256 Prüfsumme.
    pub check: [u8; 16],
    /// HMAC über Bytes [0..584).
    pub header_mac: [u8; 32],
}

impl HeaderAb {
    /// Parst die 1024 Bytes einer HEADER-Datei und validiert Checksumme und Padding.
    pub fn parse(bytes: &[u8]) -> Result<Self, LabError> {
        if bytes.len() < HEADER_AB_SIZE {
            return Err(LabError::UnexpectedEof {
                expected: HEADER_AB_SIZE,
                actual: bytes.len(),
            });
        }

        let magic = bytes.get(0..8).ok_or(LabError::InvalidMagic)?;
        if magic != HEADER_AB_MAGIC {
            return Err(LabError::InvalidMagic);
        }

        let ver = u16::from_be_bytes(
            bytes
                .get(8..10)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Version fehlt"))?,
        );
        if ver != 1 {
            return Err(LabError::UnsupportedVersion(ver));
        }

        let kind = *bytes.get(10).ok_or(LabError::Corrupted("Kind fehlt"))?;
        if kind != HEADER_AB_KIND {
            return Err(LabError::UnsupportedKind(kind));
        }

        let store_id: [u8; 16] = bytes
            .get(15..31)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("StoreId fehlt"))?;

        let generation = u64::from_be_bytes(
            bytes
                .get(31..39)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Generation fehlt"))?,
        );

        let slot_count = *bytes
            .get(39)
            .ok_or(LabError::Corrupted("SlotCount fehlt"))?;
        if !(1..=4).contains(&slot_count) {
            return Err(LabError::Corrupted("SlotCount nicht in 1..=4"));
        }

        let report_pub: [u8; 32] = bytes
            .get(40..72)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("ReportPub fehlt"))?;

        let mut slots = Vec::with_capacity(4);
        for i in 0..4 {
            let start = 72 + i * 128;
            let end = start + 128;
            let slot_bytes = bytes
                .get(start..end)
                .ok_or(LabError::Corrupted("Slot-Bereich ungültig"))?;
            slots.push(HeaderSlot::parse(slot_bytes)?);
        }

        let check: [u8; 16] = bytes
            .get(584..600)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("Check fehlt"))?;

        let header_pre = bytes
            .get(0..584)
            .ok_or(LabError::Corrupted("Header Pre-Data fehlt"))?;
        let digest = Sha256::digest(header_pre);
        let expected_check = digest
            .get(0..16)
            .ok_or(LabError::Corrupted("SHA256 Fehler"))?;
        if check != expected_check {
            return Err(LabError::VerificationFailed(
                "HEADER SHA-256 Checksumme fehlerhaft",
            ));
        }

        let header_mac: [u8; 32] = bytes
            .get(600..632)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("HeaderMac fehlt"))?;

        let padding = bytes
            .get(632..1024)
            .ok_or(LabError::Corrupted("Padding fehlt"))?;
        if padding.iter().any(|&b| b != 0) {
            return Err(LabError::Corrupted("HEADER Padding ist nicht 0"));
        }

        Ok(Self {
            format_version: ver,
            store_id,
            generation,
            slot_count,
            report_pub,
            slots,
            check,
            header_mac,
        })
    }

    /// Verifiziert den Header-MAC gegen den Master-Schlüssel.
    /// K_hdr = HKDF-Expand(K_master, "ctx/hdr/v1", 32)
    /// header_mac = HMAC-SHA256(K_hdr, Bytes[0..584))
    pub fn verify_mac(&self, raw_1024: &[u8], master_key: &[u8; 32]) -> Result<(), LabError> {
        if raw_1024.len() < HEADER_AB_SIZE {
            return Err(LabError::UnexpectedEof {
                expected: HEADER_AB_SIZE,
                actual: raw_1024.len(),
            });
        }

        let hk = Hkdf::<Sha256>::new(None, master_key);
        let mut k_hdr = [0u8; 32];
        hk.expand(b"ctx/hdr/v1", &mut k_hdr)
            .map_err(|_| LabError::Corrupted("HKDF Expand K_hdr fehlgeschlagen"))?;

        type HmacSha256 = Hmac<Sha256>;
        let mut mac = <HmacSha256 as hmac::digest::KeyInit>::new_from_slice(&k_hdr)
            .map_err(|_| LabError::Corrupted("HMAC Initialisierung fehlgeschlagen"))?;

        let pre = raw_1024
            .get(0..584)
            .ok_or(LabError::Corrupted("Pre-Bereich ungültig"))?;
        mac.update(pre);

        mac.verify_slice(&self.header_mac)
            .map_err(|_| LabError::VerificationFailed("HEADER HMAC Verifikation fehlgeschlagen"))?;

        Ok(())
    }
}
