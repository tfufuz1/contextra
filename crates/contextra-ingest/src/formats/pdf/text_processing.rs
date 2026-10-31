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
// ZWECK: Text-Nachbearbeitung für PDF-Extraktion (Ligaturen, Hyphenation, Absätze, Header/Footer).
// INVARIANTEN: FILE-CONTEXT Kopf, Zero-Panic, deterministisch (BTreeMap), keine unsafe operations.
// STAND: TS:2026-10-31T00:00:00Z

use std::collections::BTreeMap;

/// Dehnt Unicode-Ligaturen (U+FB00..=U+FB06) aus und entfernt Soft-Hyphens (U+00AD).
pub fn clean_page_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\u{00AD}' => {} // Soft hyphen entfernen
            '\u{FB00}' => out.push_str("ff"),
            '\u{FB01}' => out.push_str("fi"),
            '\u{FB02}' => out.push_str("fl"),
            '\u{FB03}' => out.push_str("ffi"),
            '\u{FB04}' => out.push_str("ffl"),
            '\u{FB05}' => out.push_str("st"),
            '\u{FB06}' => out.push_str("st"),
            other => out.push(other),
        }
    }
    out
}

/// Führt Silbentrennung am Zeilenende zusammen:
/// NUR wenn Buchstabe + "-" + Zeilenumbruch ('\n' oder '\r\n') + Kleinbuchstabe folgt.
/// Behält Bindestriche wie in "E-Mail" oder "Rhein-Main" bei.
pub fn join_end_of_line_hyphens(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = match chars.get(i) {
            Some(&c) => c,
            None => break,
        };

        if ch == '-' && i > 0 {
            let prev_char = match chars.get(i.saturating_sub(1)) {
                Some(&c) => c,
                None => '\0',
            };

            if prev_char.is_alphabetic() {
                let mut next_idx = i.saturating_add(1);

                if next_idx < len && chars.get(next_idx) == Some(&'\r') {
                    next_idx = next_idx.saturating_add(1);
                }

                if next_idx < len && chars.get(next_idx) == Some(&'\n') {
                    let mut after_newline_idx = next_idx.saturating_add(1);
                    while after_newline_idx < len {
                        match chars.get(after_newline_idx) {
                            Some(&' ') | Some(&'\t') => {
                                after_newline_idx = after_newline_idx.saturating_add(1);
                            }
                            _ => break,
                        }
                    }

                    if after_newline_idx < len {
                        if let Some(&next_char) = chars.get(after_newline_idx) {
                            if next_char.is_lowercase() {
                                i = after_newline_idx;
                                continue;
                            }
                        }
                    }
                }
            }
        }

        result.push(ch);
        i = i.saturating_add(1);
    }

    result
}

/// Baut Fließtextabsätze auf:
/// Leerzeilen und Einrückung dienen als Absatzgrenzen.
/// Harte Zeilenumbrüche innerhalb eines Absatzes werden zu Leerzeichen.
pub fn reconstruct_paragraphs(text: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let lines: Vec<&str> = text.lines().collect();

    let mut current_paragraph = String::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !current_paragraph.is_empty() {
                paragraphs.push(current_paragraph.clone());
                current_paragraph.clear();
            }
            continue;
        }

        let is_indented = line.starts_with("  ") || line.starts_with('\t');

        if is_indented && !current_paragraph.is_empty() {
            paragraphs.push(current_paragraph.clone());
            current_paragraph.clear();
        }

        if current_paragraph.is_empty() {
            current_paragraph.push_str(trimmed);
        } else {
            current_paragraph.push(' ');
            current_paragraph.push_str(trimmed);
        }
    }

    if !current_paragraph.is_empty() {
        paragraphs.push(current_paragraph);
    }

    paragraphs
}

/// Normalisiert Ziffern in einer Zeile zu '#' für die Wiedererkennung wiederkehrender Kopf-/Fußzeilen.
pub fn normalize_line_digits(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    for ch in line.chars() {
        if ch.is_ascii_digit() {
            out.push('#');
        } else {
            out.push(ch);
        }
    }
    out
}

/// Ergebnis der Header/Footer-Analyse pro Seite.
#[derive(Debug, Default)]
pub struct HeaderFooterAnalysis {
    pub detected_headers: Vec<String>,
    pub detected_footers: Vec<String>,
}

/// Analysiert Seitenzeilen auf wiederkehrende Kopf-/Fußzeilen.
/// Regel: Gleiche Zeile mit Ziffern-Normalisierung auf >= 3 Seiten und >= 50% aller Seiten.
pub fn analyze_headers_footers(page_lines: &[Vec<String>]) -> HeaderFooterAnalysis {
    let total_pages = page_lines.len();
    if total_pages < 3 {
        return HeaderFooterAnalysis::default();
    }

    let min_occurrences = 3.max((total_pages.saturating_add(1)) / 2);

    let mut top_line_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut bottom_line_counts: BTreeMap<String, usize> = BTreeMap::new();

    for lines in page_lines {
        if lines.len() == 1 {
            if let Some(first) = lines.first() {
                let norm = normalize_line_digits(first.trim());
                if !norm.is_empty() {
                    *top_line_counts.entry(norm.clone()).or_insert(0) += 1;
                    *bottom_line_counts.entry(norm).or_insert(0) += 1;
                }
            }
        } else {
            if let Some(first) = lines.first() {
                let norm = normalize_line_digits(first.trim());
                if !norm.is_empty() {
                    *top_line_counts.entry(norm).or_insert(0) += 1;
                }
            }
            if let Some(last) = lines.last() {
                let norm = normalize_line_digits(last.trim());
                if !norm.is_empty() {
                    *bottom_line_counts.entry(norm).or_insert(0) += 1;
                }
            }
        }
    }

    let mut detected_headers = Vec::new();
    for (norm, count) in top_line_counts {
        if count >= min_occurrences {
            detected_headers.push(norm);
        }
    }

    let mut detected_footers = Vec::new();
    for (norm, count) in bottom_line_counts {
        if count >= min_occurrences {
            detected_footers.push(norm);
        }
    }

    HeaderFooterAnalysis {
        detected_headers,
        detected_footers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_page_text() {
        let input = "O\u{00AD}\u{FB00}ent\u{FB01}ch \u{FB02}ug\u{FB00}e";
        let cleaned = clean_page_text(input);
        assert_eq!(cleaned, "Offentfich flugffe");
    }

    #[test]
    fn test_join_end_of_line_hyphens() {
        let input = "Das ist ein Multi-\nfunktionelles Gerät.\nE-Mail und Rhein-\nMain bleiben.";
        let joined = join_end_of_line_hyphens(input);
        assert!(joined.contains("Multifunktionelles"));
        assert!(joined.contains("E-Mail"));
        assert!(joined.contains("Rhein-\nMain") || joined.contains("Rhein-Main"));
    }

    #[test]
    fn test_reconstruct_paragraphs() {
        let input = "Erster Absatz\nZeile zwei.\n\nZweiter Absatz\nmit zwei Zeilen.";
        let paras = reconstruct_paragraphs(input);
        assert_eq!(paras.len(), 2);
        assert_eq!(paras[0], "Erster Absatz Zeile zwei.");
        assert_eq!(paras[1], "Zweiter Absatz mit zwei Zeilen.");
    }

    #[test]
    fn test_analyze_headers_footers() {
        let page1 = vec!["Dokument Titel".into(), "Inhalt 1".into(), "Seite 1".into()];
        let page2 = vec!["Dokument Titel".into(), "Inhalt 2".into(), "Seite 2".into()];
        let page3 = vec!["Dokument Titel".into(), "Inhalt 3".into(), "Seite 3".into()];
        let pages = vec![page1, page2, page3];

        let analysis = analyze_headers_footers(&pages);
        assert_eq!(
            analysis.detected_headers,
            vec!["Dokument Titel".to_string()]
        );
        assert_eq!(analysis.detected_footers, vec!["Seite #".to_string()]);
    }
}
