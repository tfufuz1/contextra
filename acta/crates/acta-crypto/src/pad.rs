//! Größenklassen-Padding (Spec 4.5).
//!
//! Der gerahmte Klartext `n` ist `[len u32 big-endian][Nutzlast][Nullfüllung]`. Seine Zielgröße:
//! - bis 4096 Byte: Vielfaches von 256,
//! - darüber: Vielfaches von `2^(floor(log2 n) - 3)`, höchstens 12,5 % Mehrbedarf.
//!
//! Exakte Längen verraten damit weder Dokument noch Betroffene.

use crate::error::CryptoError;
use zeroize::Zeroizing;

/// Länge des Längenpräfixes in Byte.
pub const LEN_PREFIX: usize = 4;
const SMALL_LIMIT: usize = 4096;
const SMALL_STEP: usize = 256;

/// Zielgröße des gerahmten Klartexts der Länge `n` (inkl. Längenpräfix).
///
/// # Errors
/// [`CryptoError::TooLarge`], wenn die Rundung überläuft.
pub fn target_len(n: usize) -> Result<usize, CryptoError> {
    if n <= SMALL_LIMIT {
        return round_up(n, SMALL_STEP);
    }
    let k = (usize::BITS - 1) - n.leading_zeros();
    let step = 1_usize
        .checked_shl(k.checked_sub(3).ok_or(CryptoError::TooLarge)?)
        .ok_or(CryptoError::TooLarge)?;
    round_up(n, step)
}

fn round_up(n: usize, step: usize) -> Result<usize, CryptoError> {
    n.div_ceil(step)
        .checked_mul(step)
        .ok_or(CryptoError::TooLarge)
}

/// Rahmt und füllt die Nutzlast auf ihre Größenklasse auf.
///
/// # Errors
/// [`CryptoError::TooLarge`], wenn die Nutzlast das Längenfeld (u32) überschreitet.
pub fn pad(payload: &[u8]) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    let len = u32::try_from(payload.len()).map_err(|_| CryptoError::TooLarge)?;
    let target = target_len(LEN_PREFIX + payload.len())?;
    let mut out = Zeroizing::new(Vec::with_capacity(target));
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(payload);
    out.resize(target, 0);
    Ok(out)
}

/// Entfernt Rahmen und Füllung. Verlangt die kanonische Zielgröße und Nullfüllung.
///
/// # Errors
/// [`CryptoError::Malformed`] bei falscher Länge, falscher Zielgröße oder Nicht-Null-Füllung.
pub fn unpad(framed: &[u8]) -> Result<&[u8], CryptoError> {
    let head = framed.get(..LEN_PREFIX).ok_or(CryptoError::Malformed)?;
    let head: [u8; LEN_PREFIX] =
        <[u8; LEN_PREFIX]>::try_from(head).map_err(|_| CryptoError::Malformed)?;
    let len = usize::try_from(u32::from_be_bytes(head)).map_err(|_| CryptoError::Malformed)?;
    let end = LEN_PREFIX.checked_add(len).ok_or(CryptoError::Malformed)?;
    if target_len(end).map_err(|_| CryptoError::Malformed)? != framed.len() {
        return Err(CryptoError::Malformed);
    }
    let payload = framed.get(LEN_PREFIX..end).ok_or(CryptoError::Malformed)?;
    let fill = framed.get(end..).ok_or(CryptoError::Malformed)?;
    if fill.iter().any(|b| *b != 0) {
        return Err(CryptoError::Malformed);
    }
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_classes_match_the_specification() -> Result<(), CryptoError> {
        assert_eq!(target_len(4)?, 256);
        assert_eq!(target_len(256)?, 256);
        assert_eq!(target_len(257)?, 512);
        assert_eq!(target_len(4096)?, 4096);
        assert_eq!(target_len(4097)?, 4608); // Schritt 512
        assert_eq!(target_len(8192)?, 8192);
        assert_eq!(target_len(8193)?, 9216); // Schritt 1024
        Ok(())
    }

    #[test]
    fn unpad_rejects_non_canonical_frames() -> Result<(), CryptoError> {
        let mut framed = pad(b"abc")?.to_vec();
        assert_eq!(unpad(&framed)?, b"abc");
        // Falsche Länge.
        let mut short = framed.clone();
        short.truncate(255);
        assert!(matches!(unpad(&short), Err(CryptoError::Malformed)));
        // Nicht-Null-Füllung.
        if let Some(last) = framed.last_mut() {
            *last = 1;
        }
        assert!(matches!(unpad(&framed), Err(CryptoError::Malformed)));
        Ok(())
    }
}
