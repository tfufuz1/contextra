//! Modul
//!
//! # Zweck & Architektur
//! Komponente des  Subsystems (Ring 4 Adapter-Schicht).
//! Stellt Parser-, Extraktions- oder Hilfsfunktionen fuer die strukturierte Dokumentverarbeitung bereit.
//!
//! # Invarianten
//! - Zero-Panic: Alle Operationen sind defensiv aufgebaut.
//! - Fail-Closed: Fehlerhafte oder unvollstaendige Daten fuehren zu typisierten Fehlern.
//! - Kein externer I/O ueber Netzwerk oder unkontrollierte Dateisystempfade.

// FILE-CONTEXT
// ZWECK: Kodierungserkennung (BOM, UTF-8, UTF-16, Windows-1252) und Sanitization (WP M4).
// INVARIANTEN: Zero-Panic, typisierte Warnungen, Sanitization über canonical::sanitize_text.
// STAND: TS:2026-10-31T00:00:00Z

use crate::canonical::sanitize_text;
use crate::ir::{ExtractWarning, WarningCode};

/// Dekodiert Rohbytes unter Berücksichtigung von BOMs, UTF-8 Validierung, UTF-16 NUL-Muster
/// und Windows-1252 Fallback. Wendet anschließend `sanitize_text` an.
pub fn decode_text(input: &[u8]) -> (String, Vec<ExtractWarning>) {
    let mut warnings = Vec::new();

    let (raw, guessed) = decode_raw_bytes(input);
    if guessed {
        warnings.push(ExtractWarning {
            code: WarningCode::EncodingGuessed,
            detail: "Kodierung per Heuristik/Fallback erkannt".into(),
        });
    }

    let (clean, sanitized_count) = sanitize_text(&raw);
    if sanitized_count > 0 {
        warnings.push(ExtractWarning {
            code: WarningCode::TextSanitized,
            detail: format!("{} Zeichen bereinigt/entfernt", sanitized_count),
        });
    }

    (clean, warnings)
}

#[allow(clippy::indexing_slicing)]
fn decode_raw_bytes(input: &[u8]) -> (String, bool) {
    if input.is_empty() {
        return (String::new(), false);
    }

    // 1. BOM Check
    if input.len() >= 4 && input.starts_with(&[0xFF, 0xFE, 0x00, 0x00]) {
        return (decode_utf32_le(&input[4..]), false);
    }
    if input.len() >= 4 && input.starts_with(&[0x00, 0x00, 0xFE, 0xFF]) {
        return (decode_utf32_be(&input[4..]), false);
    }
    if input.len() >= 2 && input.starts_with(&[0xFF, 0xFE]) {
        return (decode_utf16_le(&input[2..]), false);
    }
    if input.len() >= 2 && input.starts_with(&[0xFE, 0xFF]) {
        return (decode_utf16_be(&input[2..]), false);
    }
    if input.len() >= 3 && input.starts_with(&[0xEF, 0xBB, 0xBF]) {
        if let Ok(s) = std::str::from_utf8(&input[3..]) {
            return (s.to_string(), false);
        }
    }

    // 2. Strikte UTF-8 Validierung
    if let Ok(s) = std::str::from_utf8(input) {
        return (s.to_string(), false);
    }

    // 3. UTF-16 ohne BOM (NUL-Muster Heuristik)
    if input.len() >= 4 {
        let mut even_nul = 0usize;
        let mut odd_nul = 0usize;
        let sample_len = input.len().min(1024);

        for (idx, &b) in input[..sample_len].iter().enumerate() {
            if b == 0 {
                if idx % 2 == 0 {
                    even_nul = even_nul.saturating_add(1);
                } else {
                    odd_nul = odd_nul.saturating_add(1);
                }
            }
        }

        let total_pairs = sample_len / 2;
        if total_pairs > 0 {
            let even_ratio = (even_nul * 100) / total_pairs;
            let odd_ratio = (odd_nul * 100) / total_pairs;

            if odd_ratio >= 30 && even_ratio <= 5 {
                // UTF-16LE ASCII text has 0x00 in odd byte positions
                return (decode_utf16_le(input), true);
            }
            if even_ratio >= 30 && odd_ratio <= 5 {
                // UTF-16BE ASCII text has 0x00 in even byte positions
                return (decode_utf16_be(input), true);
            }
        }
    }

    // 4. Windows-1252 Fallback über encoding_rs
    let (cow, _) = encoding_rs::WINDOWS_1252.decode_without_bom_handling(input);
    (cow.into_owned(), true)
}

#[allow(clippy::indexing_slicing, clippy::get_first)]
fn decode_utf16_le(bytes: &[u8]) -> String {
    let u16s: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| {
            u16::from_le_bytes([
                c.get(0).copied().unwrap_or(0),
                c.get(1).copied().unwrap_or(0),
            ])
        })
        .collect();
    String::from_utf16_lossy(&u16s)
}

#[allow(clippy::indexing_slicing, clippy::get_first)]
fn decode_utf16_be(bytes: &[u8]) -> String {
    let u16s: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| {
            u16::from_be_bytes([
                c.get(0).copied().unwrap_or(0),
                c.get(1).copied().unwrap_or(0),
            ])
        })
        .collect();
    String::from_utf16_lossy(&u16s)
}

#[allow(clippy::indexing_slicing, clippy::get_first)]
fn decode_utf32_le(bytes: &[u8]) -> String {
    bytes
        .chunks_exact(4)
        .filter_map(|c| {
            let b = [
                c.get(0).copied().unwrap_or(0),
                c.get(1).copied().unwrap_or(0),
                c.get(2).copied().unwrap_or(0),
                c.get(3).copied().unwrap_or(0),
            ];
            let val = u32::from_le_bytes(b);
            char::from_u32(val)
        })
        .collect()
}

#[allow(clippy::indexing_slicing, clippy::get_first)]
fn decode_utf32_be(bytes: &[u8]) -> String {
    bytes
        .chunks_exact(4)
        .filter_map(|c| {
            let b = [
                c.get(0).copied().unwrap_or(0),
                c.get(1).copied().unwrap_or(0),
                c.get(2).copied().unwrap_or(0),
                c.get(3).copied().unwrap_or(0),
            ];
            let val = u32::from_be_bytes(b);
            char::from_u32(val)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_with_and_without_bom() {
        let plain = "Hallo Welt".as_bytes();
        let (s1, w1) = decode_text(plain);
        assert_eq!(s1, "Hallo Welt");
        assert!(w1.is_empty());

        let bom_utf8 = [0xEF, 0xBB, 0xBF, b'H', b'a', b'l', b'l', b'o'];
        let (s2, w2) = decode_text(&bom_utf8);
        assert_eq!(s2, "Hallo");
        assert!(w2.is_empty());
    }

    #[test]
    fn test_utf16_le_bom() {
        let mut bytes = vec![0xFF, 0xFE];
        for u in "Hallo".encode_utf16() {
            bytes.extend_from_slice(&u.to_le_bytes());
        }
        let (s, w) = decode_text(&bytes);
        assert_eq!(s, "Hallo");
        assert!(w.is_empty());
    }

    #[test]
    fn test_windows1252_fallback() {
        // 0xE4 in Windows-1252 is 'ä'
        let bytes = [0xE4, 0xF6, 0xFC];
        let (s, w) = decode_text(&bytes);
        assert_eq!(s, "äöü");
        assert!(w
            .iter()
            .any(|warning| warning.code == WarningCode::EncodingGuessed));
    }
}
