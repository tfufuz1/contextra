//! # FORMAT-Datei
//!
//! SPEC §9.2, Anhang A.4, F-15:
//! Dateilayout:
//! ```text
//! "CTXS" | version u16 | required u64 | optional u64 | integrity_mode u8 | instance_id [16] | created_ms u64 | chk
//! ```
//! - Offset 22: `integrity_mode` (0: Plain, 1: Authenticated, 2: Encrypted).
//! - Länge: 51 Bytes (`Plain`, 4 B CRC32C) bzw. 63 Bytes (`Authenticated`/`Encrypted`, 16 B MAC).
//! - Downgrade-Schutz (F-15): `Config::integrity` ist verbindlich; Abweichung -> `UnsupportedIntegrityMode`.
//! - Unbekanntes Pflicht-Bit -> `UnsupportedFeature`; unbekanntes optionales Bit wird ignoriert.

use crate::codec::crc32c::crc32c;
use crate::keys::secret::Secret32;
use contextra_storage_ports::Mac;
use std::fmt;

/// Magische Bytes der FORMAT-Datei ("CTXS").
pub const FORMAT_MAGIC: &[u8; 4] = b"CTXS";
/// Version 1 des FORMAT-Layouts.
pub const FORMAT_VERSION_1: u16 = 1;

/// Dateilänge für Plain (47 Bytes Header + 4 Bytes CRC32C).
pub const FORMAT_PLAIN_LEN: usize = 51;
/// Dateilänge für Authenticated / Encrypted (47 Bytes Header + 16 Bytes MAC).
pub const FORMAT_AUTH_ENC_LEN: usize = 63;

/// Pflicht-Bit: Verschlüsselung aktiv.
pub const FEATURE_ENCRYPTED: u64 = 1 << 0;
/// Pflicht-Bit: Authentisierung aktiv.
pub const FEATURE_AUTHENTICATED: u64 = 1 << 1;
/// Pflicht-Bit: Domänenversiegelung aktiv.
pub const FEATURE_DOMAIN_SEALED: u64 = 1 << 2;
/// Pflicht-Bit: Kalter Objektspeicher aktiv.
pub const FEATURE_COLD_TIER: u64 = 1 << 3;

/// Maske aller in Version 1 bekannten Pflicht-Features.
pub const KNOWN_REQUIRED_FEATURES: u64 =
    FEATURE_ENCRYPTED | FEATURE_AUTHENTICATED | FEATURE_DOMAIN_SEALED | FEATURE_COLD_TIER;

/// Integritätsmodus der Contextra-Storage-Instanz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntegrityMode {
    /// Plain: Nur CRC32C-Prüfsummen, keine Verschlüsselung (0).
    Plain = 0,
    /// Authenticated: BLAKE3-MACs mit AAD-Bindung (1).
    Authenticated = 1,
    /// Encrypted: ChaCha20-Poly1305 AEAD + BLAKE3-MACs (2).
    Encrypted = 2,
}

impl IntegrityMode {
    /// Parst den Modus aus einem u8.
    pub fn try_from_u8(val: u8) -> Result<Self, FormatError> {
        match val {
            0 => Ok(Self::Plain),
            1 => Ok(Self::Authenticated),
            2 => Ok(Self::Encrypted),
            other => Err(FormatError::InvalidMode(other)),
        }
    }

    /// Liefert den numerischen Wert des Modus.
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Fehler beim Lesen oder Validieren der FORMAT-Datei.
#[derive(Debug, PartialEq, Eq)]
pub enum FormatError {
    /// Magische Bytes stimmen nicht überein.
    BadMagic,
    /// Formatversion wird nicht unterstützt.
    UnsupportedVersion(u16),
    /// Unbekanntes Pflicht-Feature-Bit gesetzt.
    UnsupportedFeature(u64),
    /// F-15: Integritätsmodus weicht von der Konfiguration ab (Downgrade-Schutz).
    UnsupportedIntegrityMode(u8),
    /// CRC32C-Prüfsumme ist korrupt.
    CorruptChecksum {
        /// Erwartete Prüfsumme.
        expected: u32,
        /// Tatsächlich gelesene Prüfsumme.
        actual: u32,
    },
    /// MAC-Signatur ist ungültig.
    CorruptMac,
    /// Datei ist kürzer als erwartet.
    Truncated {
        /// Erwartete Mindestlänge.
        expected: usize,
        /// Tatsächliche Dateilänge.
        actual: usize,
    },
    /// Ungültiger Wert für Integritätsmodus.
    InvalidMode(u8),
    /// Allgemeiner E/A-Fehler.
    Io(String),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // INV-API-1: Keine Schlüsselbytes in Fehlermeldungen
        match self {
            Self::BadMagic => write!(f, "Ungültiges Magic in FORMAT-Datei"),
            Self::UnsupportedVersion(v) => write!(f, "Nicht unterstützte FORMAT-Version: {v}"),
            Self::UnsupportedFeature(bit) => {
                write!(f, "Nicht unterstütztes Pflicht-Feature-Bit: 0x{bit:016x}")
            }
            Self::UnsupportedIntegrityMode(m) => {
                write!(f, "Integritätsmodus {m} weicht von Konfiguration ab (F-15)")
            }
            Self::CorruptChecksum { expected, actual } => {
                write!(
                    f,
                    "FORMAT-Prüfsumme fehlerhaft: erwartet 0x{expected:08x}, gelesen 0x{actual:08x}"
                )
            }
            Self::CorruptMac => write!(f, "FORMAT-MAC ungültig"),
            Self::Truncated { expected, actual } => {
                write!(f, "FORMAT-Datei zu kurz: {actual} B, erwartet {expected} B")
            }
            Self::InvalidMode(m) => write!(f, "Ungültiger Integritätsmodus: {m}"),
            Self::Io(msg) => write!(f, "E/A-Fehler bei FORMAT: {msg}"),
        }
    }
}

impl std::error::Error for FormatError {}

/// Kopfdaten der FORMAT-Datei.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatHeader {
    /// Formatversion (immer 1).
    pub version: u16,
    /// Gesetzte Pflicht-Feature-Bits.
    pub required_features: u64,
    /// Gesetzte optionale Feature-Bits.
    pub optional_features: u64,
    /// Integritätsmodus (Plain, Authenticated, Encrypted).
    pub integrity_mode: IntegrityMode,
    /// Eindeutige 16-Byte-Instanzkennung.
    pub instance_id: [u8; 16],
    /// Erstellungszeitpunkt in Millisekunden seit UNIX-Epoche.
    pub created_ms: u64,
}

fn read_u16(slice: &[u8]) -> u16 {
    let mut b = [0u8; 2];
    if let (Some(t), Some(s)) = (b.get_mut(..), slice.get(..2)) {
        t.copy_from_slice(s);
    }
    u16::from_le_bytes(b)
}

fn read_u32(slice: &[u8]) -> u32 {
    let mut b = [0u8; 4];
    if let (Some(t), Some(s)) = (b.get_mut(..), slice.get(..4)) {
        t.copy_from_slice(s);
    }
    u32::from_le_bytes(b)
}

fn read_u64(slice: &[u8]) -> u64 {
    let mut b = [0u8; 8];
    if let (Some(t), Some(s)) = (b.get_mut(..), slice.get(..8)) {
        t.copy_from_slice(s);
    }
    u64::from_le_bytes(b)
}

impl FormatHeader {
    /// Erstellt eine neue FORMAT-Struktur.
    pub fn new(
        integrity_mode: IntegrityMode,
        instance_id: [u8; 16],
        created_ms: u64,
        required_features: u64,
        optional_features: u64,
    ) -> Self {
        Self {
            version: FORMAT_VERSION_1,
            required_features,
            optional_features,
            integrity_mode,
            instance_id,
            created_ms,
        }
    }

    /// Kodiert die 47 vorangestellten Bytes vor der Prüfsumme bzw. dem MAC.
    pub fn encode_prefix_47(&self) -> [u8; 47] {
        let mut buf = [0u8; 47];
        if let Some(target) = buf.get_mut(0..4) {
            target.copy_from_slice(FORMAT_MAGIC);
        }
        if let Some(target) = buf.get_mut(4..6) {
            target.copy_from_slice(&self.version.to_le_bytes());
        }
        if let Some(target) = buf.get_mut(6..14) {
            target.copy_from_slice(&self.required_features.to_le_bytes());
        }
        if let Some(target) = buf.get_mut(14..22) {
            target.copy_from_slice(&self.optional_features.to_le_bytes());
        }
        // Offset 22: integrity_mode
        if let Some(byte) = buf.get_mut(22) {
            *byte = self.integrity_mode.as_u8();
        }
        if let Some(target) = buf.get_mut(23..39) {
            target.copy_from_slice(&self.instance_id);
        }
        if let Some(target) = buf.get_mut(39..47) {
            target.copy_from_slice(&self.created_ms.to_le_bytes());
        }
        buf
    }

    /// Kodiert eine vollständige Plain-FORMAT-Datei (51 Bytes mit CRC32C).
    pub fn encode_plain(&self) -> [u8; FORMAT_PLAIN_LEN] {
        let prefix = self.encode_prefix_47();
        let chk = crc32c(0, &prefix);
        let mut full = [0u8; FORMAT_PLAIN_LEN];
        if let Some(target) = full.get_mut(0..47) {
            target.copy_from_slice(&prefix);
        }
        if let Some(target) = full.get_mut(47..51) {
            target.copy_from_slice(&chk.to_le_bytes());
        }
        full
    }

    /// Kodiert eine vollständige Authenticated- oder Encrypted-FORMAT-Datei (63 Bytes mit MAC).
    pub fn encode_authenticated(
        &self,
        mac: &dyn Mac,
        format_mac_key: &Secret32,
    ) -> [u8; FORMAT_AUTH_ENC_LEN] {
        let prefix = self.encode_prefix_47();
        let tag = mac.mac16(format_mac_key.expose(), &[&prefix]);
        let mut full = [0u8; FORMAT_AUTH_ENC_LEN];
        if let Some(target) = full.get_mut(0..47) {
            target.copy_from_slice(&prefix);
        }
        if let Some(target) = full.get_mut(47..63) {
            target.copy_from_slice(&tag);
        }
        full
    }

    /// Dekodiert und validiert eine FORMAT-Datei aus einem Byte-Puffer.
    ///
    /// Führt Downgrade-Schutz (F-15) und Prüfung auf unbekannte Pflicht-Bits durch.
    pub fn decode(
        bytes: &[u8],
        expected_mode: Option<IntegrityMode>,
        mac_context: Option<(&dyn Mac, &Secret32)>,
    ) -> Result<Self, FormatError> {
        if bytes.len() < 47 {
            return Err(FormatError::Truncated {
                expected: 47,
                actual: bytes.len(),
            });
        }

        if bytes.get(0..4) != Some(FORMAT_MAGIC) {
            return Err(FormatError::BadMagic);
        }

        let version = read_u16(bytes.get(4..6).unwrap_or(&[]));
        if version != FORMAT_VERSION_1 {
            return Err(FormatError::UnsupportedVersion(version));
        }

        let required_features = read_u64(bytes.get(6..14).unwrap_or(&[]));
        let optional_features = read_u64(bytes.get(14..22).unwrap_or(&[]));

        // Offset 22: integrity_mode
        let raw_mode = match bytes.get(22) {
            Some(&m) => m,
            None => {
                return Err(FormatError::Truncated {
                    expected: 23,
                    actual: bytes.len(),
                })
            }
        };
        let integrity_mode = IntegrityMode::try_from_u8(raw_mode)?;

        // F-15: Downgrade-Schutz VOR dem Lesen weiterer Dateien
        if let Some(exp) = expected_mode {
            if integrity_mode != exp {
                return Err(FormatError::UnsupportedIntegrityMode(raw_mode));
            }
        }

        let mut instance_id = [0u8; 16];
        if let (Some(target), Some(src)) = (instance_id.get_mut(..), bytes.get(23..39)) {
            target.copy_from_slice(src);
        }
        let created_ms = read_u64(bytes.get(39..47).unwrap_or(&[]));

        // Prüfe Prüfsumme / MAC
        let prefix = bytes.get(0..47).unwrap_or(&[]);
        match integrity_mode {
            IntegrityMode::Plain => {
                if bytes.len() < FORMAT_PLAIN_LEN {
                    return Err(FormatError::Truncated {
                        expected: FORMAT_PLAIN_LEN,
                        actual: bytes.len(),
                    });
                }
                let actual = read_u32(bytes.get(47..51).unwrap_or(&[]));
                let expected = crc32c(0, prefix);
                if actual != expected {
                    return Err(FormatError::CorruptChecksum { expected, actual });
                }
            }
            IntegrityMode::Authenticated | IntegrityMode::Encrypted => {
                if bytes.len() < FORMAT_AUTH_ENC_LEN {
                    return Err(FormatError::Truncated {
                        expected: FORMAT_AUTH_ENC_LEN,
                        actual: bytes.len(),
                    });
                }
                let mut actual_tag = [0u8; 16];
                if let (Some(target), Some(src)) = (actual_tag.get_mut(..), bytes.get(47..63)) {
                    target.copy_from_slice(src);
                }
                if let Some((mac, key)) = mac_context {
                    if !mac.verify16(key.expose(), &[prefix], &actual_tag) {
                        return Err(FormatError::CorruptMac);
                    }
                }
            }
        }

        // Unbekannte Pflicht-Features prüfen (unbekannte optionale Bits werden ignoriert)
        let unknown_required = required_features & !KNOWN_REQUIRED_FEATURES;
        if unknown_required != 0 {
            let lowest_bit = 1u64 << unknown_required.trailing_zeros();
            return Err(FormatError::UnsupportedFeature(lowest_bit));
        }

        Ok(Self {
            version,
            required_features,
            optional_features,
            integrity_mode,
            instance_id,
            created_ms,
        })
    }
}

/// Fügt ein Pflicht-Feature-Bit hinzu.
///
/// Pflicht-Bits werden gemäß SPEC §9.2 ausschließlich hinzugefügt (`|=`), nie entfernt.
pub fn add_required_feature(header: &mut FormatHeader, feature_bit: u64) {
    header.required_features |= feature_bit;
}
