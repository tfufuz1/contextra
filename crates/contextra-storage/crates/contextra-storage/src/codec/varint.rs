//! Kanonisches unsigned LEB128 für Wire-Formate (SP-CODEC, INV-FMT-1).

use core::fmt;

/// Fehler beim Lesen eines kanonischen varint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarintError {
    Truncated,
    Overflow,
    NonCanonical,
}

impl fmt::Display for VarintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => f.write_str("truncated varint"),
            Self::Overflow => f.write_str("varint exceeds u64"),
            Self::NonCanonical => f.write_str("non-canonical varint"),
        }
    }
}

impl std::error::Error for VarintError {}

/// Hängt `value` als kleinstmögliche unsigned LEB128-Kodierung an.
pub fn varint_put(out: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        out.push((value as u8) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

/// Liest genau ein kanonisches unsigned LEB128 und liefert Wert sowie Bytezahl.
pub fn varint_get(input: &[u8]) -> Result<(u64, usize), VarintError> {
    let mut value = 0u64;
    for (index, byte) in input.iter().copied().take(10).enumerate() {
        let payload = (byte & 0x7f) as u64;
        if index == 9 && (byte & 0xfe) != 0 {
            return Err(VarintError::Overflow);
        }
        value |= payload << (index * 7);
        if byte & 0x80 == 0 {
            if index > 0 && payload == 0 {
                return Err(VarintError::NonCanonical);
            }
            return Ok((value, index + 1));
        }
    }

    if input.len() >= 10 {
        Err(VarintError::Overflow)
    } else {
        Err(VarintError::Truncated)
    }
}
