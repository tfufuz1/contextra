//! # Kanonischer Varint (LEB128 für u64)
//!
//! Normative Regeln (INV-FMT-1, SPEC §9.1):
//! - Nur kanonische Kodierungen sind gültig.
//! - Keine überflüssigen Fortsetzungen (`0x80 ... 0x00`).
//! - Das zehnte Byte darf ausschließlich `0x01` sein.
//! - Höchstens 10 Bytes; Dekodierung ist total (P-4).
//! - Keine Integer-Casts (M-02 = 0).

use std::fmt;

/// Fehler bei der Dekodierung eines Varints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarintError {
    /// Unerwartetes Ende der Eingabe.
    UnexpectedEof,
    /// Nicht-kanonische Kodierung oder ungültige Bytes (INV-FMT-1).
    NonCanonical,
    /// Überlauf über den Wertebereich von `u64`.
    Overflow,
}

impl fmt::Display for VarintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "Unerwartetes Ende beim Dekodieren des Varints"),
            Self::NonCanonical => write!(f, "Nicht-kanonischer Varint (INV-FMT-1)"),
            Self::Overflow => write!(f, "Überlauf beim Dekodieren des Varints"),
        }
    }
}

impl std::error::Error for VarintError {}

/// Kodiert einen `u64` kanonisch in den Puffer und gibt die Anzahl der geschriebenen Bytes zurück.
pub fn encode_u64(mut val: u64, buf: &mut [u8]) -> usize {
    let mut i: usize = 0;
    while val >= 0x80 {
        if let Some(b) = buf.get_mut(i) {
            let lower = u8::try_from(val & 0x7F).unwrap_or(0);
            *b = lower | 0x80;
        }
        val >>= 7;
        i = i.saturating_add(1);
    }
    if let Some(b) = buf.get_mut(i) {
        *b = u8::try_from(val & 0xFF).unwrap_or(0);
    }
    i.saturating_add(1)
}

/// Dekodiert einen kanonischen Varint total aus dem Slice.
///
/// Liefert `Ok((wert, verbrauchte_bytes))` oder `Err(VarintError)`.
pub fn decode_u64(buf: &[u8]) -> Result<(u64, usize), VarintError> {
    if buf.is_empty() {
        return Err(VarintError::UnexpectedEof);
    }

    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    let mut consumed: usize = 0;

    for &byte in buf.iter().take(10) {
        consumed = consumed.saturating_add(1);
        let val = u64::from(byte & 0x7F);

        if consumed == 10 {
            // Das 10. Byte darf nur 0x01 sein (INV-FMT-1)
            if byte != 0x01 {
                return Err(VarintError::NonCanonical);
            }
            let shifted = val.checked_shl(shift).ok_or(VarintError::Overflow)?;
            result |= shifted;
            return Ok((result, consumed));
        }

        let shifted = val.checked_shl(shift).ok_or(VarintError::Overflow)?;
        result |= shifted;
        shift = shift.saturating_add(7);

        if (byte & 0x80) == 0 {
            // Ende erreicht. Prüfe Kanonizität:
            // Wenn mehr als 1 Byte gelesen wurde, darf das letzte Byte nicht 0x00 sein.
            if consumed > 1 && byte == 0x00 {
                return Err(VarintError::NonCanonical);
            }
            return Ok((result, consumed));
        }
    }

    // Wenn nach 10 Bytes noch das Continuation-Bit gesetzt war oder Puffer abgebrochen
    if consumed == 10 {
        Err(VarintError::NonCanonical)
    } else {
        Err(VarintError::UnexpectedEof)
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_a2_varint() {
        let vectors: &[(u64, &[u8])] = &[
            (0, &[0x00]),
            (1, &[0x01]),
            (127, &[0x7f]),
            (128, &[0x80, 0x01]),
            (300, &[0xac, 0x02]),
            (16384, &[0x80, 0x80, 0x01]),
            (1 << 32, &[0x80, 0x80, 0x80, 0x80, 0x10]),
            (u64::MAX, &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]),
        ];

        for &(val, expected) in vectors {
            let mut buf = [0u8; 10];
            let len = encode_u64(val, &mut buf);
            assert_eq!(&buf[..len], expected);

            let (decoded, consumed) = decode_u64(expected).expect("valid decode");
            assert_eq!(decoded, val);
            assert_eq!(consumed, expected.len());
        }

        // Ungültige Vektoren prüfen
        assert_eq!(decode_u64(&[0x80, 0x00]), Err(VarintError::NonCanonical));
        assert_eq!(decode_u64(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x02]), Err(VarintError::NonCanonical));
        assert_eq!(decode_u64(&[]), Err(VarintError::UnexpectedEof));
        assert_eq!(decode_u64(&[0x80]), Err(VarintError::UnexpectedEof));
    }
}
