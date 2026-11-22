//! # KEYS-Dateiprotokoll nach SPEC §9.3
//!
//! Layout der Datei `KEYS`:
//! ```text
//! "CTXK" | version u16 | keys_generation u64 | nonce [12] | sealed_len u32 | sealed[sealed_len]
//! sealed = Cipher.seal( key = Kdf(KEK, "ctxs keys v1"), nonce, aad = instance_id || version || keys_generation, plain )
//! plain  = root_key [32] | count u16 | ( space u16 | dek_gen u32 | dek [32] ) * count
//! ```
//!
//! KEK-Rotation ist O(1): KEYS wird mit neuem KEK-Schlüssel und neuer Nonce ersetzt.

use super::hierarchy::derive_keys_key;
use super::secret::Secret32;
use contextra_storage_ports::{Cipher, Entropy, Kdf};
use std::fmt;

/// Magische Konstante für KEYS-Dateien.
pub const KEYS_MAGIC: &[u8; 4] = b"CTXK";
/// Version 1 des KEYS-Protokolls.
pub const KEYS_VERSION_1: u16 = 1;

/// Fehler beim Kodieren oder Dekodieren der KEYS-Datei.
#[derive(Debug, PartialEq, Eq)]
pub enum KeysError {
    /// Magische Bytes stimmen nicht überein.
    BadMagic,
    /// Unbekannte oder nicht unterstützte Protokollversion.
    UnsupportedVersion(u16),
    /// Datenstrom ist unvollständig oder kürzer als erwartet.
    Truncated,
    /// Ungültiges Format des Klartext-Inhalts.
    InvalidPlain,
    /// Fehler bei kryptografischer Operation (z. B. MAC-Verletzung bei Cipher::open).
    Cipher(String),
    /// E/A- oder Entropiefehler.
    Io(String),
}

impl fmt::Display for KeysError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // INV-API-1: Keine Schlüsselbytes in Fehlermeldungen
        match self {
            Self::BadMagic => write!(f, "Ungültiges Magic in KEYS-Datei"),
            Self::UnsupportedVersion(v) => write!(f, "Nicht unterstützte KEYS-Version: {v}"),
            Self::Truncated => write!(f, "KEYS-Datei ist abgeschnitten"),
            Self::InvalidPlain => write!(f, "Klartextstruktur von KEYS ist ungültig"),
            Self::Cipher(msg) => write!(f, "Kryptografiefehler in KEYS: {msg}"),
            Self::Io(msg) => write!(f, "E/A-Fehler in KEYS: {msg}"),
        }
    }
}

impl std::error::Error for KeysError {}

/// DEK-Eintrag für einen Space innerhalb der KEYS-Datei.
pub struct SpaceDekRecord {
    /// Space-Kennung.
    pub space: u16,
    /// DEK-Generation (hochzählend bei Epochenwechsel oder Re-Keying).
    pub dek_gen: u32,
    /// Der eigentliche Space-DEK.
    pub dek: Secret32,
}

/// Klartext-Nutzlast der KEYS-Datei vor der Verschlüsselung.
pub struct KeysPlain {
    /// Einmaliger Wurzelschlüssel für Datei- und Redo-Schlüssel.
    pub root_key: Secret32,
    /// Zugeordnete Space-DEKs.
    pub space_deks: Vec<SpaceDekRecord>,
}

impl KeysPlain {
    /// Kodiert den Klartext in Bytes.
    pub fn encode(&self) -> Vec<u8> {
        let count_u16 = u16::try_from(self.space_deks.len()).unwrap_or(u16::MAX);
        let mut buf = Vec::with_capacity(34 + self.space_deks.len() * 38);
        buf.extend_from_slice(self.root_key.expose());
        buf.extend_from_slice(&count_u16.to_le_bytes());

        for rec in &self.space_deks {
            buf.extend_from_slice(&rec.space.to_le_bytes());
            buf.extend_from_slice(&rec.dek_gen.to_le_bytes());
            buf.extend_from_slice(rec.dek.expose());
        }
        buf
    }

    /// Dekodiert den Klartext aus Bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, KeysError> {
        if bytes.len() < 34 {
            return Err(KeysError::Truncated);
        }

        let mut root_bytes = [0u8; 32];
        if let (Some(target), Some(src)) = (root_bytes.get_mut(..), bytes.get(0..32)) {
            target.copy_from_slice(src);
        } else {
            return Err(KeysError::Truncated);
        }
        let root_key = Secret32::new(root_bytes);

        let count = match bytes.get(32..34) {
            Some(slice) => {
                let mut b = [0u8; 2];
                if let Some(t) = b.get_mut(..) {
                    t.copy_from_slice(slice);
                }
                u16::from_le_bytes(b) as usize
            }
            None => return Err(KeysError::Truncated),
        };

        let expected_len = match 34usize.checked_add(count.saturating_mul(38)) {
            Some(l) => l,
            None => return Err(KeysError::InvalidPlain),
        };

        if bytes.len() < expected_len {
            return Err(KeysError::Truncated);
        }

        let mut space_deks = Vec::with_capacity(count);
        let mut offset = 34;
        for _ in 0..count {
            let space = match bytes.get(offset..offset + 2) {
                Some(s) => {
                    let mut b = [0u8; 2];
                    if let Some(t) = b.get_mut(..) {
                        t.copy_from_slice(s);
                    }
                    u16::from_le_bytes(b)
                }
                None => return Err(KeysError::Truncated),
            };
            offset = offset.saturating_add(2);

            let dek_gen = match bytes.get(offset..offset + 4) {
                Some(s) => {
                    let mut b = [0u8; 4];
                    if let Some(t) = b.get_mut(..) {
                        t.copy_from_slice(s);
                    }
                    u32::from_le_bytes(b)
                }
                None => return Err(KeysError::Truncated),
            };
            offset = offset.saturating_add(4);

            let mut dek_bytes = [0u8; 32];
            if let (Some(target), Some(src)) = (dek_bytes.get_mut(..), bytes.get(offset..offset + 32)) {
                target.copy_from_slice(src);
            } else {
                return Err(KeysError::Truncated);
            }
            offset = offset.saturating_add(32);

            space_deks.push(SpaceDekRecord {
                space,
                dek_gen,
                dek: Secret32::new(dek_bytes),
            });
        }

        Ok(Self {
            root_key,
            space_deks,
        })
    }
}

/// Vollständige `KEYS`-Datei (versiegelt).
pub struct KeysFile {
    /// Protokollversion (aktuell 1).
    pub version: u16,
    /// Fortlaufende Generation bei KEK-Rotation oder DEK-Aktualisierung.
    pub keys_generation: u64,
    /// Zufällige Nonce für diesen Schreibvorgang.
    pub nonce: [u8; 12],
    /// Verschlüsselte Nutzlast.
    pub sealed_payload: Vec<u8>,
}

impl KeysFile {
    /// Baut AAD für die Versiegelung: `instance_id || version || keys_generation` (26 Bytes).
    pub fn build_aad(instance_id: &[u8; 16], version: u16, keys_generation: u64) -> [u8; 26] {
        let mut aad = [0u8; 26];
        if let Some(target) = aad.get_mut(0..16) {
            target.copy_from_slice(instance_id);
        }
        if let Some(target) = aad.get_mut(16..18) {
            target.copy_from_slice(&version.to_le_bytes());
        }
        if let Some(target) = aad.get_mut(18..26) {
            target.copy_from_slice(&keys_generation.to_le_bytes());
        }
        aad
    }

    /// Versiegelt den Klartext und erstellt ein `KeysFile`.
    pub fn seal(
        plain: &KeysPlain,
        kek: &Secret32,
        instance_id: &[u8; 16],
        keys_generation: u64,
        kdf: &dyn Kdf,
        cipher: &dyn Cipher,
        entropy: &dyn Entropy,
    ) -> Result<Self, KeysError> {
        let mut nonce = [0u8; 12];
        entropy
            .fill(&mut nonce)
            .map_err(|e| KeysError::Io(e.to_string()))?;

        let keys_key = derive_keys_key(kdf, kek);
        let aad = Self::build_aad(instance_id, KEYS_VERSION_1, keys_generation);
        let plain_bytes = plain.encode();
        let sealed_payload = cipher.seal(keys_key.expose(), &nonce, &aad, &plain_bytes);

        Ok(Self {
            version: KEYS_VERSION_1,
            keys_generation,
            nonce,
            sealed_payload,
        })
    }

    /// Öffnet und entschlüsselt die versiegelte `KEYS`-Datei.
    pub fn open(
        &self,
        kek: &Secret32,
        instance_id: &[u8; 16],
        kdf: &dyn Kdf,
        cipher: &dyn Cipher,
    ) -> Result<KeysPlain, KeysError> {
        let keys_key = derive_keys_key(kdf, kek);
        let aad = Self::build_aad(instance_id, self.version, self.keys_generation);
        let plain_bytes = cipher
            .open(keys_key.expose(), &self.nonce, &aad, &self.sealed_payload)
            .map_err(|e| KeysError::Cipher(e.to_string()))?;

        KeysPlain::decode(&plain_bytes)
    }

    /// Serialisiert das `KeysFile` in das binäre Dateiformat.
    pub fn encode(&self) -> Vec<u8> {
        let sealed_len = match u32::try_from(self.sealed_payload.len()) {
            Ok(l) => l,
            Err(_) => u32::MAX,
        };
        let mut buf = Vec::with_capacity(30 + self.sealed_payload.len());
        buf.extend_from_slice(KEYS_MAGIC);
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.extend_from_slice(&self.keys_generation.to_le_bytes());
        buf.extend_from_slice(&self.nonce);
        buf.extend_from_slice(&sealed_len.to_le_bytes());
        buf.extend_from_slice(&self.sealed_payload);
        buf
    }

    /// Deserialisiert ein `KeysFile` aus dem binären Dateiformat.
    pub fn decode(bytes: &[u8]) -> Result<Self, KeysError> {
        if bytes.len() < 30 {
            return Err(KeysError::Truncated);
        }

        if bytes.get(0..4) != Some(KEYS_MAGIC) {
            return Err(KeysError::BadMagic);
        }

        let version = match bytes.get(4..6) {
            Some(s) => {
                let mut b = [0u8; 2];
                if let Some(t) = b.get_mut(..) {
                    t.copy_from_slice(s);
                }
                u16::from_le_bytes(b)
            }
            None => return Err(KeysError::Truncated),
        };

        if version != KEYS_VERSION_1 {
            return Err(KeysError::UnsupportedVersion(version));
        }

        let keys_generation = match bytes.get(6..14) {
            Some(s) => {
                let mut b = [0u8; 8];
                if let Some(t) = b.get_mut(..) {
                    t.copy_from_slice(s);
                }
                u64::from_le_bytes(b)
            }
            None => return Err(KeysError::Truncated),
        };

        let mut nonce = [0u8; 12];
        if let (Some(target), Some(src)) = (nonce.get_mut(..), bytes.get(14..26)) {
            target.copy_from_slice(src);
        } else {
            return Err(KeysError::Truncated);
        }

        let sealed_len = match bytes.get(26..30) {
            Some(s) => {
                let mut b = [0u8; 4];
                if let Some(t) = b.get_mut(..) {
                    t.copy_from_slice(s);
                }
                u32::from_le_bytes(b) as usize
            }
            None => return Err(KeysError::Truncated),
        };

        let payload_slice = match bytes.get(30..30 + sealed_len) {
            Some(s) => s,
            None => return Err(KeysError::Truncated),
        };

        Ok(Self {
            version,
            keys_generation,
            nonce,
            sealed_payload: payload_slice.to_vec(),
        })
    }
}
