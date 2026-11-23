//! # FORMAT-Kopf (SPEC §10.2, Modus `Plain`)
//!
//! Layout (53 Byte):
//! `"CTXS"[4] | format_ver u16 (=1) | feature_req u64 | feature_opt u64 | integrity_mode u8 |
//!  instance_id [16] | algo_set u16 (=1) | created_ms u64 | chk`
//!
//! - Länge vor `chk`: 49 Byte. `integrity_mode` steht an Offset 22 (§10.2).
//! - `chk` = CRC-32C über die 49 Byte vor `chk`, Seed 0 (Annahme A-1, `tests/format/README.md`).
//! - Der Prüfwert kommt über den `Checksum`-Port (SPEC §4): der Aufrufer reicht die Port-Funktion
//!   `crc32c(seed, data)` durch. Hier steht keine eigene CRC-Implementierung.
//! - `Authenticated`/`Encrypted` sind in dieser Schicht nicht kodiert: der Schlüssel für den
//!   FORMAT-MAC ist in der Spec nicht festgelegt (WP-2.4). Der Modus wird abgewiesen.
//!
//! Dekodierung ist total (P-4): jede Byte-Folge ergibt `Ok` oder `Err`, nie Panik.
//! Reihenfolge der Prüfungen: Magic, `integrity_mode` (F-15), Länge, `chk`,
//! Version, Pflicht-Bits, `algo_set`.

use std::fmt;

/// Magische Bytes der FORMAT-Datei.
pub const FORMAT_MAGIC: [u8; 4] = *b"CTXS";
/// Formatversion (SPEC §10.1: genau eine Version, `1`).
pub const FORMAT_VERSION: u16 = 1;
/// `algo_set` 1: XChaCha20-Poly1305, BLAKE3-keyed, HKDF-artig (§10.2).
pub const ALGO_SET_XCHACHA_BLAKE3: u16 = 1;
/// `integrity_mode` `Plain`.
pub const INTEGRITY_PLAIN: u8 = 0;

/// Pflicht-Bit `ENCRYPTED` (§10.2).
pub const FEATURE_ENCRYPTED: u64 = 1 << 0;
/// Pflicht-Bit `AUTHENTICATED` (§10.2).
pub const FEATURE_AUTHENTICATED: u64 = 1 << 1;
/// Pflicht-Bit `SEAL_CLUSTER` (§10.2).
pub const FEATURE_SEAL_CLUSTER: u64 = 1 << 2;
/// Pflicht-Bit `COLD_TIER` (§10.2).
pub const FEATURE_COLD_TIER: u64 = 1 << 3;
/// Alle Pflicht-Bits, die Version 1 kennt. Jedes andere Pflicht-Bit ist `Unsupported`.
pub const KNOWN_REQUIRED_FEATURES: u64 =
    FEATURE_ENCRYPTED | FEATURE_AUTHENTICATED | FEATURE_SEAL_CLUSTER | FEATURE_COLD_TIER;

/// Länge vor `chk` (§10.2).
pub const FORMAT_BODY_LEN: usize = 49;
/// Gesamtlänge im Modus `Plain` (§10.2).
pub const FORMAT_PLAIN_LEN: usize = FORMAT_BODY_LEN + 4;

const OFFSET_VERSION: usize = 4;
const OFFSET_FEATURE_REQ: usize = 6;
const OFFSET_FEATURE_OPT: usize = 14;
const OFFSET_MODE: usize = 22;
const OFFSET_INSTANCE: usize = 23;
const OFFSET_ALGO: usize = 39;
const OFFSET_CREATED: usize = 41;
const OFFSET_CHK: usize = FORMAT_BODY_LEN;

/// Felder des FORMAT-Kopfes ohne Magic, Version und `chk`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatFields {
    /// Gesetzte Pflicht-Bits.
    pub feature_req: u64,
    /// Gesetzte optionale Bits (unbekannte werden ignoriert).
    pub feature_opt: u64,
    /// `integrity_mode` (0 `Plain`, 1 `Authenticated`, 2 `Encrypted`).
    pub integrity_mode: u8,
    /// Instanzkennung (16 Byte).
    pub instance_id: [u8; 16],
    /// `algo_set` (1 bei Version 1).
    pub algo_set: u16,
    /// Erstellungszeit in Millisekunden seit Unix-Epoche.
    pub created_ms: u64,
}

/// Fehler beim Kodieren oder Dekodieren des FORMAT-Kopfes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatCodecError {
    /// Eingabe kürzer als erwartet.
    Truncated {
        /// Erwartete Länge.
        expected: usize,
        /// Tatsächliche Länge.
        actual: usize,
    },
    /// Eingabe länger als der feste Kopf (kein Restbereich erlaubt).
    TrailingBytes {
        /// Anzahl überschüssiger Bytes.
        extra: usize,
    },
    /// Magic stimmt nicht.
    BadMagic,
    /// `integrity_mode` wird in dieser Schicht nicht kodiert oder weicht ab (F-15).
    UnsupportedIntegrityMode(u8),
    /// Prüfwert stimmt nicht.
    ChecksumMismatch {
        /// Im Kopf gespeicherter Wert.
        expected: u32,
        /// Neu berechneter Wert.
        actual: u32,
    },
    /// Formatversion ist nicht 1.
    UnsupportedVersion(u16),
    /// Unbekannte Pflicht-Bits (nur die gesetzten unbekannten Bits werden genannt).
    UnsupportedRequiredFeature(u64),
    /// `algo_set` ist nicht 1.
    UnsupportedAlgoSet(u16),
}

impl fmt::Display for FormatCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { expected, actual } => {
                write!(f, "FORMAT-Kopf zu kurz: {actual} B, erwartet {expected} B")
            }
            Self::TrailingBytes { extra } => {
                write!(f, "FORMAT-Kopf hat {extra} überschüssige Bytes")
            }
            Self::BadMagic => write!(f, "Ungültiges Magic im FORMAT-Kopf"),
            Self::UnsupportedIntegrityMode(m) => {
                write!(f, "Integritätsmodus {m} wird nicht unterstützt (F-15)")
            }
            Self::ChecksumMismatch { expected, actual } => write!(
                f,
                "FORMAT-Prüfwert falsch: erwartet 0x{expected:08x}, berechnet 0x{actual:08x}"
            ),
            Self::UnsupportedVersion(v) => write!(f, "Nicht unterstützte FORMAT-Version {v}"),
            Self::UnsupportedRequiredFeature(bits) => {
                write!(f, "Unbekannte Pflicht-Bits gesetzt: 0x{bits:016x}")
            }
            Self::UnsupportedAlgoSet(a) => write!(f, "Nicht unterstütztes algo_set {a}"),
        }
    }
}

impl std::error::Error for FormatCodecError {}

/// Kodiert den FORMAT-Kopf im Modus `Plain` (53 Byte).
///
/// `checksum` ist die Port-Funktion `Checksum::crc32c(seed, data)` (SPEC §4).
pub fn encode_format_plain(
    fields: &FormatFields,
    checksum: &dyn Fn(u32, &[u8]) -> u32,
) -> Result<Vec<u8>, FormatCodecError> {
    if fields.integrity_mode != INTEGRITY_PLAIN {
        return Err(FormatCodecError::UnsupportedIntegrityMode(
            fields.integrity_mode,
        ));
    }
    let mut out = Vec::with_capacity(FORMAT_PLAIN_LEN);
    out.extend_from_slice(&FORMAT_MAGIC);
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&fields.feature_req.to_le_bytes());
    out.extend_from_slice(&fields.feature_opt.to_le_bytes());
    out.push(fields.integrity_mode);
    out.extend_from_slice(&fields.instance_id);
    out.extend_from_slice(&fields.algo_set.to_le_bytes());
    out.extend_from_slice(&fields.created_ms.to_le_bytes());
    let chk = checksum(0, &out);
    out.extend_from_slice(&chk.to_le_bytes());
    Ok(out)
}

/// Dekodiert einen FORMAT-Kopf im Modus `Plain`. Total: nie Panik.
pub fn decode_format_plain(
    buf: &[u8],
    checksum: &dyn Fn(u32, &[u8]) -> u32,
) -> Result<FormatFields, FormatCodecError> {
    let truncated = || FormatCodecError::Truncated {
        expected: FORMAT_PLAIN_LEN,
        actual: buf.len(),
    };
    // Magic und Modus zuerst: der Modus bestimmt die Länge von `chk` (§10.2, F-15).
    if buf.len() <= OFFSET_MODE {
        return Err(truncated());
    }
    if read::<4>(buf, 0)? != FORMAT_MAGIC {
        return Err(FormatCodecError::BadMagic);
    }
    let [mode] = read::<1>(buf, OFFSET_MODE)?;
    if mode != INTEGRITY_PLAIN {
        return Err(FormatCodecError::UnsupportedIntegrityMode(mode));
    }
    match buf.len().cmp(&FORMAT_PLAIN_LEN) {
        std::cmp::Ordering::Less => return Err(truncated()),
        std::cmp::Ordering::Greater => {
            return Err(FormatCodecError::TrailingBytes {
                extra: buf.len().saturating_sub(FORMAT_PLAIN_LEN),
            })
        }
        std::cmp::Ordering::Equal => {}
    }

    // Prüfwert vor jeder Auswertung der Felder (fail-closed).
    let body = buf.get(..FORMAT_BODY_LEN).ok_or_else(truncated)?;
    let expected = u32::from_le_bytes(read::<4>(buf, OFFSET_CHK)?);
    let actual = checksum(0, body);
    if expected != actual {
        return Err(FormatCodecError::ChecksumMismatch { expected, actual });
    }

    let version = u16::from_le_bytes(read::<2>(buf, OFFSET_VERSION)?);
    if version != FORMAT_VERSION {
        return Err(FormatCodecError::UnsupportedVersion(version));
    }
    let feature_req = u64::from_le_bytes(read::<8>(buf, OFFSET_FEATURE_REQ)?);
    let unknown_required = feature_req & !KNOWN_REQUIRED_FEATURES;
    if unknown_required != 0 {
        return Err(FormatCodecError::UnsupportedRequiredFeature(
            unknown_required,
        ));
    }
    let algo_set = u16::from_le_bytes(read::<2>(buf, OFFSET_ALGO)?);
    if algo_set != ALGO_SET_XCHACHA_BLAKE3 {
        return Err(FormatCodecError::UnsupportedAlgoSet(algo_set));
    }

    Ok(FormatFields {
        feature_req,
        feature_opt: u64::from_le_bytes(read::<8>(buf, OFFSET_FEATURE_OPT)?),
        integrity_mode: mode,
        instance_id: read::<16>(buf, OFFSET_INSTANCE)?,
        algo_set,
        created_ms: u64::from_le_bytes(read::<8>(buf, OFFSET_CREATED)?),
    })
}

/// Liest `N` Bytes ab `off`. Fehlt ein Teil, ist die Eingabe zu kurz.
fn read<const N: usize>(buf: &[u8], off: usize) -> Result<[u8; N], FormatCodecError> {
    buf.get(off..)
        .and_then(|s| s.get(..N))
        .and_then(|s| <[u8; N]>::try_from(s).ok())
        .ok_or(FormatCodecError::Truncated {
            expected: FORMAT_PLAIN_LEN,
            actual: buf.len(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::crc32c::crc32c;

    fn sample() -> FormatFields {
        FormatFields {
            feature_req: 0,
            feature_opt: 0,
            integrity_mode: INTEGRITY_PLAIN,
            instance_id: core::array::from_fn(|i| u8::try_from(i).unwrap_or(0)),
            algo_set: ALGO_SET_XCHACHA_BLAKE3,
            created_ms: 0x0102_0304_0506_0708,
        }
    }

    #[test]
    fn roundtrip_plain() {
        let f = sample();
        let bytes = encode_format_plain(&f, &crc32c).expect("Plain kodierbar");
        assert_eq!(bytes.len(), FORMAT_PLAIN_LEN);
        assert_eq!(decode_format_plain(&bytes, &crc32c), Ok(f));
    }

    #[test]
    fn kodieren_lehnt_nicht_plain_ab() {
        let mut f = sample();
        f.integrity_mode = 1;
        assert_eq!(
            encode_format_plain(&f, &crc32c),
            Err(FormatCodecError::UnsupportedIntegrityMode(1))
        );
    }

    #[test]
    fn leere_und_kurze_eingaben_sind_fehler_nicht_panik() {
        for n in 0..FORMAT_PLAIN_LEN {
            let bytes = vec![0u8; n];
            assert!(decode_format_plain(&bytes, &crc32c).is_err());
        }
    }
}
