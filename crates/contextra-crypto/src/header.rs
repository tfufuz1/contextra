//! Pure in-memory HEADER encoding and decoding according to F.2 and CR-13, CR-14.

use crate::error::{CryptoError, Result};
use crate::hierarchy::HdrKey;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

pub const HEADER_SIZE: usize = 1024;
pub const HEADER_MAGIC: &[u8; 8] = b"CTXHDR\x00\x01";
pub const HEADER_KIND: u8 = 0x20;
pub const HEADER_FORMAT_VERSION: u16 = 1;
pub const HEADER_ALG_IDS: [u8; 4] = [1, 0, 1, 1]; // [aead=1, kdf_or_0=0, mac=1, hash=1]

/// Slot representation inside HEADER (128 bytes).
#[derive(Debug, Clone, Copy)]
pub struct HeaderSlot {
    pub slot_id: u8,
    pub anchor_id: u8,
    pub kdf_id: u8,
    pub status: u8, // 0 = empty, 1 = active, 2 = removed
    pub m_kib: u32,
    pub t: u8,
    pub p: u8,
    pub slot_salt: [u8; 32],
    pub nonce: [u8; 12],
    pub wrapped_master: [u8; 48],
}

impl HeaderSlot {
    pub fn empty() -> Self {
        Self {
            slot_id: 0,
            anchor_id: 0,
            kdf_id: 0,
            status: 0,
            m_kib: 0,
            t: 0,
            p: 0,
            slot_salt: [0u8; 32],
            nonce: [0u8; 12],
            wrapped_master: [0u8; 48],
        }
    }

    pub fn encode(&self, out: &mut [u8; 128]) {
        out.fill(0);
        out[0] = self.slot_id;
        out[1] = self.anchor_id;
        out[2] = self.kdf_id;
        out[3] = self.status;
        if let Some(slot) = out.get_mut(4..8) {
            slot.copy_from_slice(&self.m_kib.to_be_bytes());
        }
        out[8] = self.t;
        out[9] = self.p;
        // reserved 2 bytes (out[10..12]) = 0
        if let Some(slot) = out.get_mut(12..44) {
            slot.copy_from_slice(&self.slot_salt);
        }
        if let Some(slot) = out.get_mut(44..56) {
            slot.copy_from_slice(&self.nonce);
        }
        if let Some(slot) = out.get_mut(56..104) {
            slot.copy_from_slice(&self.wrapped_master);
        }
        // zero 24 bytes (out[104..128]) = 0
    }

    pub fn decode(bytes: &[u8; 128]) -> Self {
        let mut slot_salt = [0u8; 32];
        if let Some(src) = bytes.get(12..44) {
            slot_salt.copy_from_slice(src);
        }
        let mut nonce = [0u8; 12];
        if let Some(src) = bytes.get(44..56) {
            nonce.copy_from_slice(src);
        }
        let mut wrapped_master = [0u8; 48];
        if let Some(src) = bytes.get(56..104) {
            wrapped_master.copy_from_slice(src);
        }

        let m_kib = bytes
            .get(4..8)
            .and_then(|s| s.try_into().ok())
            .map(u32::from_be_bytes)
            .unwrap_or(0);

        Self {
            slot_id: bytes.first().copied().unwrap_or(0),
            anchor_id: bytes.get(1).copied().unwrap_or(0),
            kdf_id: bytes.get(2).copied().unwrap_or(0),
            status: bytes.get(3).copied().unwrap_or(0),
            m_kib,
            t: bytes.get(8).copied().unwrap_or(0),
            p: bytes.get(9).copied().unwrap_or(0),
            slot_salt,
            nonce,
            wrapped_master,
        }
    }
}

/// Parsed HEADER representation.
#[derive(Debug, Clone)]
pub struct Header {
    pub format_version: u16,
    pub kind: u8,
    pub alg_ids: [u8; 4],
    pub store_id: [u8; 16],
    pub generation: u64,
    pub slot_count: u8,
    pub report_pub: [u8; 32],
    pub slots: [HeaderSlot; 4],
}

impl Header {
    pub fn check16(bytes584: &[u8]) -> [u8; 16] {
        let hash = Sha256::digest(bytes584);
        let mut out = [0u8; 16];
        if let Some(src) = hash.get(..16) {
            out.copy_from_slice(src);
        }
        out
    }

    pub fn mac(hdr_key: &HdrKey, bytes584: &[u8]) -> [u8; 32] {
        let mut mac = match Hmac::<Sha256>::new_from_slice(hdr_key.0.as_bytes()) {
            Ok(m) => m,
            Err(_) => return [0u8; 32],
        };
        mac.update(bytes584);
        mac.finalize().into_bytes().into()
    }

    pub fn encode(&self, hdr_key: &HdrKey) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        if let Some(dest) = buf.get_mut(0..8) {
            dest.copy_from_slice(HEADER_MAGIC);
        }
        if let Some(dest) = buf.get_mut(8..10) {
            dest.copy_from_slice(&self.format_version.to_be_bytes());
        }
        if let Some(b) = buf.get_mut(10) {
            *b = self.kind;
        }
        if let Some(dest) = buf.get_mut(11..15) {
            dest.copy_from_slice(&self.alg_ids);
        }
        if let Some(dest) = buf.get_mut(15..31) {
            dest.copy_from_slice(&self.store_id);
        }
        if let Some(dest) = buf.get_mut(31..39) {
            dest.copy_from_slice(&self.generation.to_be_bytes());
        }
        if let Some(b) = buf.get_mut(39) {
            *b = self.slot_count;
        }
        if let Some(dest) = buf.get_mut(40..72) {
            dest.copy_from_slice(&self.report_pub);
        }

        for (i, slot) in self.slots.iter().enumerate() {
            let offset = 72 + i * 128;
            let mut slot_buf = [0u8; 128];
            slot.encode(&mut slot_buf);
            if let Some(dest) = buf.get_mut(offset..offset + 128) {
                dest.copy_from_slice(&slot_buf);
            }
        }

        // check: SHA-256(Bytes[0..584))[0..16]
        let (chk, hmac_val) = match buf.get(..584) {
            Some(header_bytes) => (
                Self::check16(header_bytes),
                Self::mac(hdr_key, header_bytes),
            ),
            None => ([0u8; 16], [0u8; 32]),
        };
        if let Some(dest) = buf.get_mut(584..600) {
            dest.copy_from_slice(&chk);
        }
        if let Some(dest) = buf.get_mut(600..632) {
            dest.copy_from_slice(&hmac_val);
        }

        buf
    }

    pub fn decode_checked(raw: &[u8; HEADER_SIZE]) -> Result<Self> {
        let magic = match raw.get(0..8) {
            Some(m) => m,
            None => return Err(CryptoError::Corrupt),
        };
        if magic != HEADER_MAGIC {
            return Err(CryptoError::Corrupt);
        }

        let format_version = raw
            .get(8..10)
            .and_then(|s| s.try_into().ok())
            .map(u16::from_be_bytes)
            .ok_or(CryptoError::Corrupt)?;
        if format_version != HEADER_FORMAT_VERSION {
            return Err(CryptoError::Corrupt);
        }

        let kind = raw.get(10).copied().ok_or(CryptoError::Corrupt)?;
        if kind != HEADER_KIND {
            return Err(CryptoError::Corrupt);
        }

        let alg_ids: [u8; 4] = raw
            .get(11..15)
            .and_then(|s| s.try_into().ok())
            .ok_or(CryptoError::Corrupt)?;
        // CR-14: unknown values, especially alg_id = 2, return Corrupt
        if alg_ids[0] == 2 || alg_ids != HEADER_ALG_IDS {
            return Err(CryptoError::Corrupt);
        }

        let bytes584 = raw.get(..584).ok_or(CryptoError::Corrupt)?;
        let expected_check = Self::check16(bytes584);
        let actual_check = raw.get(584..600).ok_or(CryptoError::Corrupt)?;
        if !bool::from(subtle::ConstantTimeEq::ct_eq(actual_check, &expected_check)) {
            return Err(CryptoError::Corrupt);
        }

        let store_id: [u8; 16] = raw
            .get(15..31)
            .and_then(|s| s.try_into().ok())
            .ok_or(CryptoError::Corrupt)?;
        let generation = raw
            .get(31..39)
            .and_then(|s| s.try_into().ok())
            .map(u64::from_be_bytes)
            .ok_or(CryptoError::Corrupt)?;
        let slot_count = raw.get(39).copied().ok_or(CryptoError::Corrupt)?;
        if slot_count == 0 || slot_count > 4 {
            return Err(CryptoError::Corrupt);
        }

        let report_pub: [u8; 32] = raw
            .get(40..72)
            .and_then(|s| s.try_into().ok())
            .ok_or(CryptoError::Corrupt)?;

        let mut slots = [
            HeaderSlot::empty(),
            HeaderSlot::empty(),
            HeaderSlot::empty(),
            HeaderSlot::empty(),
        ];
        for (i, slot) in slots.iter_mut().enumerate() {
            let offset = 72 + i * 128;
            if let Some(src) = raw.get(offset..offset + 128) {
                if let Ok(slot_buf) = <&[u8; 128]>::try_from(src) {
                    *slot = HeaderSlot::decode(slot_buf);
                }
            }
        }

        Ok(Self {
            format_version,
            kind,
            alg_ids,
            store_id,
            generation,
            slot_count,
            report_pub,
            slots,
        })
    }

    pub fn verify_mac(&self, hdr_key: &HdrKey, raw: &[u8; HEADER_SIZE]) -> Result<()> {
        let bytes584 = raw.get(..584).ok_or(CryptoError::Corrupt)?;
        let expected = Self::mac(hdr_key, bytes584);
        let actual_mac = raw.get(600..632).ok_or(CryptoError::Corrupt)?;
        if bool::from(subtle::ConstantTimeEq::ct_eq(actual_mac, &expected)) {
            Ok(())
        } else {
            Err(CryptoError::Auth)
        }
    }
}
