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
// ZWECK: Erkennung und Dekodierung der Textkodierung für Webdokumente (BOM, Meta-Charset, UTF-8-Fallback).
// INVARIANTEN: Zero-Panic, nur get/checked_*, keine std::collections::{HashMap,HashSet}.
// STAND: TS:2026-10-31T00:00:00Z

use encoding_rs::Encoding;

/// Ergebnis der Dekodierung eines HTML/XHTML-Byte-Streams.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DecodedHtml {
    /// Der dekodierte UTF-8-String.
    pub text: String,
    /// Angabe, ob die Kodierung geraten/Fallback war.
    pub is_guessed: bool,
}

/// Dekodiert Eingabe-Bytes unter Berücksichtigung von BOM, `<meta charset>` und UTF-8-Fallback.
pub fn decode_html_bytes(input: &[u8]) -> DecodedHtml {
    // 1. BOM-Prüfung
    if let Some((encoding, bom_len)) = Encoding::for_bom(input) {
        let payload = input.get(bom_len..).unwrap_or(b"");
        let (cow, _, had_errors) = encoding.decode(payload);
        return DecodedHtml {
            text: cow.into_owned(),
            is_guessed: had_errors,
        };
    }

    // 2. <meta charset> oder <meta http-equiv="Content-Type"> Durchsuchung in den ersten 4096 Bytes
    let scan_len = input.len().min(4096);
    let slice = input.get(..scan_len).unwrap_or(input);
    let mut detected_encoding = None;

    if let Some(label) = extract_meta_charset(slice) {
        if let Some(enc) = Encoding::for_label(label.as_bytes()) {
            detected_encoding = Some(enc);
        }
    }

    let (encoding, is_guessed) = match detected_encoding {
        Some(enc) => (enc, false),
        None => (encoding_rs::UTF_8, true),
    };

    let (cow, _, had_errors) = encoding.decode(input);
    DecodedHtml {
        text: cow.into_owned(),
        is_guessed: is_guessed || had_errors,
    }
}

/// Extrahiert den Charset-Namen aus `<meta>`-Tags im gegebenen Byte-Ausschnitt.
fn extract_meta_charset(slice: &[u8]) -> Option<String> {
    let text = slice
        .iter()
        .map(|&b| {
            if b.is_ascii() {
                (b as char).to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>();

    if let Some(pos) = text.find("charset=") {
        let after = text.get(pos.saturating_add(8)..)?;
        let trimmed = after.trim_start();
        let quote = trimmed.chars().next()?;
        let value = if quote == '"' || quote == '\'' {
            trimmed.get(1..).and_then(|s| s.split(quote).next())
        } else {
            trimmed
                .split(|c: char| c.is_whitespace() || c == '>' || c == '/' || c == ';')
                .next()
        };

        if let Some(val) = value {
            let clean = val.trim();
            if !clean.is_empty() {
                return Some(clean.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_utf8_default() {
        let bytes = b"<html><body>Hello World</body></html>";
        let res = decode_html_bytes(bytes);
        assert_eq!(res.text, "<html><body>Hello World</body></html>");
        assert!(res.is_guessed);
    }

    #[test]
    fn test_decode_meta_charset() {
        let bytes =
            b"<html><head><meta charset=\"iso-8859-1\"></head><body>\xE4\xF6\xFC</body></html>";
        let res = decode_html_bytes(bytes);
        assert!(!res.is_guessed);
        assert!(res.text.contains("äöü"));
    }

    #[test]
    fn test_decode_bom() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(b"<html><body>BOM</body></html>");
        let res = decode_html_bytes(&bytes);
        assert_eq!(res.text, "<html><body>BOM</body></html>");
        assert!(!res.is_guessed);
    }
}
