// FILE-CONTEXT
// ZWECK: Inhaltsbasierte Formaterkennung anhand Magic Bytes, Strukturmerkmalen und Hints (WP M3 / P01).
// INVARIANTEN: Zero-Panic, deterministisch, signature-fest, feste Obergrenze (max 64 KiB analysiert).
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::format::FormatId;

/// Maximale Anzahl an Bytes, die zur Formaterkennung analysiert werden (64 KiB).
pub const MAX_DETECT_BYTES: usize = 64 * 1024;

/// Konfidenz der Formaterkennung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Confidence {
    Low,
    Medium,
    High,
}

/// Hinweise zur Formaterkennung.
#[derive(Debug, Clone, Default)]
pub struct DetectHints<'a> {
    pub filename: Option<&'a str>,
    pub mime: Option<&'a str>,
}

/// Ergebnis der Formaterkennung.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detection {
    pub format: FormatId,
    pub confidence: Confidence,
    pub reason: &'static str,
}

/// Information über durch Inhaltsanalyse erzeugte Treffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ContentMatch {
    format: FormatId,
    reason: &'static str,
}

/// Formaterkennung anhand von Inhalts-Bytes (max 64 KiB) und Hinweisen.
pub fn detect(bytes: &[u8], hints: &DetectHints<'_>) -> Result<Detection, IngestError> {
    let analyze_len = bytes.len().min(MAX_DETECT_BYTES);
    let data = match bytes.get(..analyze_len) {
        Some(slice) => slice,
        None => bytes,
    };

    let (hint_fmt, is_from_ext) = resolve_hint_format(hints);
    let content_match = detect_content_format(data, hint_fmt);

    match (content_match, hint_fmt) {
        // Fall 1: Magic und Hint stimmen überein -> High Confidence
        (Some(c_match), Some(h_fmt)) if c_match.format == h_fmt => Ok(Detection {
            format: c_match.format,
            confidence: Confidence::High,
            reason: "magic and hint match",
        }),

        // Fall 2: Magic vorhanden, aber Widerspruch mit Hint -> Magic gewinnt, Medium Confidence
        (Some(c_match), Some(_h_fmt)) => Ok(Detection {
            format: c_match.format,
            confidence: Confidence::Medium,
            reason: if is_from_ext {
                "content magic overrides extension match"
            } else {
                "content magic overrides mime match"
            },
        }),

        // Fall 3: Nur Magic vorhanden -> Medium Confidence
        (Some(c_match), None) => Ok(Detection {
            format: c_match.format,
            confidence: Confidence::Medium,
            reason: c_match.reason,
        }),

        // Fall 4: Kein Magic, aber Hint vorhanden -> Nur Endung/MIME = Confidence::Low
        (None, Some(h_fmt)) => {
            // Prüfung auf Widerspruch: Wenn es unzweideutige Container-Magic gibt (z.B. PK\x03\x04 oder GZIP),
            // die aber kein spezifisches Subformat ergab, und der Hint ein Textformat fordert:
            if (is_zip_container(data) || is_gzip(data)) && is_text_format(h_fmt) {
                return Err(IngestError::UnsupportedFormat {
                    format: Some(h_fmt),
                    reason: "Content is a binary archive container but hint specifies text format"
                        .into(),
                });
            }

            Ok(Detection {
                format: h_fmt,
                confidence: Confidence::Low,
                reason: if is_from_ext {
                    "extension match"
                } else {
                    "mime match"
                },
            })
        }

        // Fall 5: Weder Magic noch Hint
        (None, None) => Err(IngestError::UnsupportedFormat {
            format: None,
            reason: "Format could not be detected from hints or content".into(),
        }),
    }
}

/// Löst Hint-Format aus Dateiendung oder MIME-Type auf.
fn resolve_hint_format(hints: &DetectHints<'_>) -> (Option<FormatId>, bool) {
    if let Some(filename) = hints.filename {
        if let Some(pos) = filename.rfind('.') {
            if let Some(ext) = filename.get(pos.saturating_add(1)..) {
                if let Some(format) = FormatId::from_extension(ext) {
                    return (Some(format), true);
                }
            }
        }
    }

    if let Some(mime) = hints.mime {
        let clean_mime = mime.trim().to_lowercase();
        for &fmt in ALL_FORMATS {
            if fmt.mime() == clean_mime {
                return (Some(fmt), false);
            }
        }
    }

    (None, false)
}

/// Führt inhaltsbasierte Erkennung über Magic Bytes und Strukturmerkmale durch.
fn detect_content_format(data: &[u8], hint_fmt: Option<FormatId>) -> Option<ContentMatch> {
    if data.is_empty() {
        return None;
    }

    // 1. PDF Check (%PDF-)
    if is_pdf(data) {
        return Some(ContentMatch {
            format: FormatId::Pdf,
            reason: "PDF magic header",
        });
    }

    // 2. RTF Check ({\rtf)
    if is_rtf(data) {
        return Some(ContentMatch {
            format: FormatId::Rtf,
            reason: "RTF magic header",
        });
    }

    // 3. ZIP-Container (PK\x03\x04) -> Docx, Pptx, Xlsx, Odt, Odp, Ods, Epub
    if is_zip_container(data) {
        if let Some((fmt, reason)) = inspect_zip_container(data) {
            return Some(ContentMatch {
                format: fmt,
                reason,
            });
        }
    }

    // 4. OLE2/CFB Container (D0 CF 11 E0 A1 B1 1A E1) -> Doc, Xls, Ppt, Msg
    if is_ole_container(data) {
        if let Some((fmt, reason)) = inspect_ole_container(data) {
            return Some(ContentMatch {
                format: fmt,
                reason,
            });
        } else if let Some(h_fmt) = hint_fmt {
            if is_ole_format(h_fmt) {
                return Some(ContentMatch {
                    format: h_fmt,
                    reason: "OLE2 header detected without specific directory stream",
                });
            }
        }
    }

    // 5. GZIP Check (0x1F 0x8B)
    if is_gzip(data) {
        if let Some(h_fmt) = hint_fmt {
            if !is_text_format(h_fmt) {
                return Some(ContentMatch {
                    format: h_fmt,
                    reason: "GZIP compressed archive matching hint",
                });
            }
        }
    }

    // 6. Text-, BOM- und Strukturheuristiken (UTF-8, UTF-16, HTML, XML, JSON, Markdown, EML, MBOX, TeX)
    detect_text_heuristics(data)
}

fn is_pdf(data: &[u8]) -> bool {
    if data.starts_with(b"%PDF-") {
        return true;
    }
    let limit = data.len().min(1024);
    if let Some(slice) = data.get(..limit) {
        slice.windows(5).any(|w| w == b"%PDF-")
    } else {
        false
    }
}

fn is_rtf(data: &[u8]) -> bool {
    data.starts_with(b"{\\rtf")
}

fn is_zip_container(data: &[u8]) -> bool {
    data.starts_with(&[0x50, 0x4b, 0x03, 0x04])
}

fn is_ole_container(data: &[u8]) -> bool {
    data.starts_with(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1])
}

fn is_gzip(data: &[u8]) -> bool {
    data.starts_with(&[0x1f, 0x8b])
}

#[inline]
fn byte_at(slice: &[u8], idx: usize) -> u8 {
    match slice.get(idx) {
        Some(&b) => b,
        None => 0,
    }
}

/// Untersucht ZIP-Lokaldateiheader ohne Entpacken.
fn inspect_zip_container(data: &[u8]) -> Option<(FormatId, &'static str)> {
    let mut offset = 0usize;
    let mut has_content_types = false;
    let mut has_word = false;
    let mut has_xl = false;
    let mut has_ppt = false;

    while let Some(header) = data.get(offset..offset.saturating_add(30)) {
        if header.get(..4) != Some(&[0x50, 0x4b, 0x03, 0x04]) {
            break;
        }

        let comp_method = u16::from_le_bytes([byte_at(header, 8), byte_at(header, 9)]);
        let comp_size = match usize::try_from(u32::from_le_bytes([
            byte_at(header, 18),
            byte_at(header, 19),
            byte_at(header, 20),
            byte_at(header, 21),
        ])) {
            Ok(s) => s,
            Err(_) => 0,
        };

        let fn_len = usize::from(u16::from_le_bytes([
            byte_at(header, 26),
            byte_at(header, 27),
        ]));
        let extra_len = usize::from(u16::from_le_bytes([
            byte_at(header, 28),
            byte_at(header, 29),
        ]));

        let fn_start = offset.saturating_add(30);
        let fn_end = fn_start.saturating_add(fn_len);

        if let Some(fn_bytes) = data.get(fn_start..fn_end) {
            let fn_str = String::from_utf8_lossy(fn_bytes);

            if fn_str == "mimetype" && comp_method == 0 {
                let mime_start = fn_end.saturating_add(extra_len);
                let mime_end = mime_start.saturating_add(comp_size);
                if let Some(mime_bytes) = data.get(mime_start..mime_end) {
                    let mime_str = String::from_utf8_lossy(mime_bytes);
                    let clean = mime_str.trim();
                    if clean.starts_with("application/vnd.oasis.opendocument.text") {
                        return Some((FormatId::Odt, "ODF text package mimetype"));
                    }
                    if clean.starts_with("application/vnd.oasis.opendocument.spreadsheet") {
                        return Some((FormatId::Ods, "ODF spreadsheet package mimetype"));
                    }
                    if clean.starts_with("application/vnd.oasis.opendocument.presentation") {
                        return Some((FormatId::Odp, "ODF presentation package mimetype"));
                    }
                    if clean.starts_with("application/epub+zip") {
                        return Some((FormatId::Epub, "EPUB package mimetype"));
                    }
                }
            }

            if fn_str == "[Content_Types].xml" {
                has_content_types = true;
            }
            if fn_str.starts_with("word/") {
                has_word = true;
            }
            if fn_str.starts_with("xl/") {
                has_xl = true;
            }
            if fn_str.starts_with("ppt/") {
                has_ppt = true;
            }
        }

        let next_offset = fn_end.saturating_add(extra_len).saturating_add(comp_size);
        if next_offset <= offset {
            break;
        }
        offset = next_offset;
    }

    if has_content_types || has_word || has_xl || has_ppt {
        if has_word {
            return Some((FormatId::Docx, "OOXML Word processing container"));
        }
        if has_xl {
            return Some((FormatId::Xlsx, "OOXML Spreadsheet container"));
        }
        if has_ppt {
            return Some((FormatId::Pptx, "OOXML Presentation container"));
        }
        return Some((FormatId::Docx, "OOXML container structure"));
    }

    None
}

/// Suchmuster für OLE2 / CFB Verzeichnisstream-Namen in UTF-16LE.
fn inspect_ole_container(data: &[u8]) -> Option<(FormatId, &'static str)> {
    const WORD_STREAM: &[u8] = &[
        b'W', 0, b'o', 0, b'r', 0, b'd', 0, b'D', 0, b'o', 0, b'c', 0, b'u', 0, b'm', 0, b'e', 0,
        b'n', 0, b't', 0,
    ];
    const WORKBOOK_STREAM: &[u8] = &[
        b'W', 0, b'o', 0, b'r', 0, b'k', 0, b'b', 0, b'o', 0, b'o', 0, b'k', 0,
    ];
    const BOOK_STREAM: &[u8] = &[b'B', 0, b'o', 0, b'o', 0, b'k', 0];
    const PPT_STREAM: &[u8] = &[
        b'P', 0, b'o', 0, b'w', 0, b'e', 0, b'r', 0, b'P', 0, b'o', 0, b'i', 0, b'n', 0, b't', 0,
        b' ', 0, b'D', 0, b'o', 0, b'c', 0, b'u', 0, b'm', 0, b'e', 0, b'n', 0, b't', 0,
    ];
    const MSG_SUBSTG: &[u8] = &[
        b'_', 0, b'_', 0, b's', 0, b'u', 0, b'b', 0, b's', 0, b't', 0, b'g', 0, b'1', 0, b'.', 0,
        b'0', 0,
    ];
    const MSG_PROPS: &[u8] = &[
        b'_', 0, b'_', 0, b'p', 0, b'r', 0, b'o', 0, b'p', 0, b'e', 0, b'r', 0, b't', 0, b'i', 0,
        b'e', 0, b's', 0,
    ];

    if contains_slice(data, WORD_STREAM) {
        return Some((FormatId::Doc, "OLE2 WordDocument stream"));
    }
    if contains_slice(data, WORKBOOK_STREAM) || contains_slice(data, BOOK_STREAM) {
        return Some((FormatId::Xls, "OLE2 Workbook stream"));
    }
    if contains_slice(data, PPT_STREAM) {
        return Some((FormatId::Ppt, "OLE2 PowerPoint stream"));
    }
    if contains_slice(data, MSG_SUBSTG) || contains_slice(data, MSG_PROPS) {
        return Some((FormatId::Msg, "OLE2 Outlook Message stream"));
    }

    None
}

fn contains_slice(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || haystack.len() < needle.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// Heuristiken für Text-basierte Formate (inklusive BOM- und UTF-16 Dekodierung).
fn detect_text_heuristics(data: &[u8]) -> Option<ContentMatch> {
    // 1. UTF-16 LE/BE BOM Dekodierung
    let (text_str, has_bom) = if data.starts_with(&[0xff, 0xfe]) {
        // UTF-16 LE BOM
        let decoded = decode_utf16_le(match data.get(2..) {
            Some(s) => s,
            None => return None,
        });
        if decoded.is_empty() {
            return None;
        }
        (decoded, true)
    } else if data.starts_with(&[0xfe, 0xff]) {
        // UTF-16 BE BOM
        let decoded = decode_utf16_be(match data.get(2..) {
            Some(s) => s,
            None => return None,
        });
        if decoded.is_empty() {
            return None;
        }
        (decoded, true)
    } else {
        // UTF-8
        let text_data = if data.starts_with(&[0xef, 0xbb, 0xbf]) {
            match data.get(3..) {
                Some(s) => s,
                None => data,
            }
        } else {
            data
        };

        match std::str::from_utf8(text_data) {
            Ok(s) => (s.to_string(), data.starts_with(&[0xef, 0xbb, 0xbf])),
            Err(e) => {
                let valid_len = e.valid_up_to();
                if valid_len < 16 {
                    return None;
                }
                match text_data.get(..valid_len) {
                    Some(valid_slice) => match std::str::from_utf8(valid_slice) {
                        Ok(s) => (s.to_string(), false),
                        Err(_) => return None,
                    },
                    None => return None,
                }
            }
        }
    };

    let trimmed = text_str.trim_start();

    // MBOX
    if trimmed.starts_with("From ") {
        return Some(ContentMatch {
            format: FormatId::Mbox,
            reason: "MBOX header line",
        });
    }

    // HTML / XHTML
    let trimmed_lower = trimmed.to_lowercase();
    if trimmed_lower.starts_with("<!doctype html")
        || trimmed_lower.starts_with("<html")
        || trimmed_lower.starts_with("<head")
        || trimmed_lower.starts_with("<body")
    {
        if trimmed_lower.contains("xmlns=\"http://www.w3.org/1999/xhtml\"") {
            return Some(ContentMatch {
                format: FormatId::Xhtml,
                reason: "XHTML namespace structure",
            });
        }
        return Some(ContentMatch {
            format: FormatId::Html,
            reason: "HTML element structure",
        });
    }

    // XML
    if trimmed.starts_with("<?xml") {
        return Some(ContentMatch {
            format: FormatId::Xml,
            reason: "XML declaration header",
        });
    }

    // JSON / JSONL
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
            return Some(ContentMatch {
                format: FormatId::Json,
                reason: "Valid JSON document structure",
            });
        }
        if let Some(first_line) = trimmed.lines().next() {
            if serde_json::from_str::<serde_json::Value>(first_line.trim()).is_ok() {
                return Some(ContentMatch {
                    format: FormatId::Jsonl,
                    reason: "Valid JSONL line structure",
                });
            }
        }
    }

    // Email (EML)
    if is_eml_header(trimmed) {
        return Some(ContentMatch {
            format: FormatId::Eml,
            reason: "RFC822 email header structure",
        });
    }

    // LaTeX / TeX
    if trimmed.contains("\\documentclass")
        || trimmed.contains("\\begin{document}")
        || trimmed.contains("\\section{")
    {
        return Some(ContentMatch {
            format: FormatId::Tex,
            reason: "TeX macro structure",
        });
    }

    // Markdown Heuristik
    if is_markdown(trimmed) {
        return Some(ContentMatch {
            format: FormatId::Markdown,
            reason: "Markdown heading or link structure",
        });
    }

    // Falls BOM vorhanden war, aber kein spezifisches Textformat -> Txt Fallback
    if has_bom {
        return Some(ContentMatch {
            format: FormatId::Txt,
            reason: "Text BOM detected",
        });
    }

    None
}

fn decode_utf16_le(bytes: &[u8]) -> String {
    let u16s: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| {
            let b0 = match c.get(0) {
                Some(&b) => b,
                None => 0,
            };
            let b1 = match c.get(1) {
                Some(&b) => b,
                None => 0,
            };
            u16::from_le_bytes([b0, b1])
        })
        .collect();
    String::from_utf16_lossy(&u16s)
}

fn decode_utf16_be(bytes: &[u8]) -> String {
    let u16s: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| {
            let b0 = match c.get(0) {
                Some(&b) => b,
                None => 0,
            };
            let b1 = match c.get(1) {
                Some(&b) => b,
                None => 0,
            };
            u16::from_be_bytes([b0, b1])
        })
        .collect();
    String::from_utf16_lossy(&u16s)
}

fn is_markdown(text: &str) -> bool {
    let mut has_heading = false;
    let mut has_link = false;
    let mut has_bullet = false;
    let mut has_frontmatter = false;

    if text.starts_with("---\n") || text.starts_with("---\r\n") {
        has_frontmatter = true;
    }

    for line in text.lines().take(30) {
        let l = line.trim();
        if l.starts_with("# ") || l.starts_with("## ") || l.starts_with("### ") {
            has_heading = true;
        }
        if l.starts_with("- ") || l.starts_with("* ") {
            has_bullet = true;
        }
        if l.contains('[') && l.contains("](") && l.contains(')') {
            has_link = true;
        }
    }

    has_frontmatter || has_heading || (has_bullet && has_link)
}

fn is_eml_header(text: &str) -> bool {
    let mut has_from = false;
    let mut has_subject_or_to_or_mime = false;

    for line in text.lines().take(20) {
        let l = line.trim();
        if l.starts_with("From:") || l.starts_with("from:") {
            has_from = true;
        }
        if l.starts_with("To:")
            || l.starts_with("Subject:")
            || l.starts_with("MIME-Version:")
            || l.starts_with("Message-ID:")
            || l.starts_with("Received:")
        {
            has_subject_or_to_or_mime = true;
        }
    }

    has_from && has_subject_or_to_or_mime
}

fn is_text_format(fmt: FormatId) -> bool {
    matches!(
        fmt,
        FormatId::Txt
            | FormatId::Markdown
            | FormatId::Rst
            | FormatId::AsciiDoc
            | FormatId::Tex
            | FormatId::Html
            | FormatId::Xhtml
            | FormatId::Csv
            | FormatId::Tsv
            | FormatId::Json
            | FormatId::Jsonl
            | FormatId::Xml
            | FormatId::Yaml
            | FormatId::Toml
    )
}

fn is_ole_format(fmt: FormatId) -> bool {
    matches!(
        fmt,
        FormatId::Doc | FormatId::Xls | FormatId::Ppt | FormatId::Msg
    )
}

const ALL_FORMATS: &[FormatId] = &[
    FormatId::Txt,
    FormatId::Markdown,
    FormatId::Rst,
    FormatId::AsciiDoc,
    FormatId::Tex,
    FormatId::Rtf,
    FormatId::Html,
    FormatId::Xhtml,
    FormatId::Epub,
    FormatId::Docx,
    FormatId::Pptx,
    FormatId::Xlsx,
    FormatId::Xls,
    FormatId::Xlsb,
    FormatId::Ods,
    FormatId::Csv,
    FormatId::Tsv,
    FormatId::Odt,
    FormatId::Odp,
    FormatId::Pdf,
    FormatId::Doc,
    FormatId::Ppt,
    FormatId::Msg,
    FormatId::Eml,
    FormatId::Mbox,
    FormatId::Json,
    FormatId::Jsonl,
    FormatId::Xml,
    FormatId::Yaml,
    FormatId::Toml,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_pdf_high_confidence() {
        let hints = DetectHints {
            filename: Some("doc.pdf"),
            mime: Some("application/pdf"),
        };
        let bytes = b"%PDF-1.7\n%...";
        let det = match detect(bytes, &hints) {
            Ok(d) => d,
            Err(e) => panic!("Detection failed: {e:?}"),
        };
        assert_eq!(det.format, FormatId::Pdf);
        assert_eq!(det.confidence, Confidence::High);
    }

    #[test]
    fn test_detect_rtf_no_hints() {
        let hints = DetectHints::default();
        let bytes = b"{\\rtf1\\ansi\\deff0}";
        let det = match detect(bytes, &hints) {
            Ok(d) => d,
            Err(e) => panic!("Detection failed: {e:?}"),
        };
        assert_eq!(det.format, FormatId::Rtf);
        assert_eq!(det.confidence, Confidence::Medium);
    }

    #[test]
    fn test_detect_contradiction_zip_vs_txt() {
        let hints = DetectHints {
            filename: Some("archive.txt"),
            mime: None,
        };
        let mut zip_bytes = vec![0x50, 0x4b, 0x03, 0x04];
        zip_bytes.extend_from_slice(&[0u8; 22]);
        zip_bytes.extend_from_slice(&10u16.to_le_bytes());
        zip_bytes.extend_from_slice(&0u16.to_le_bytes());
        zip_bytes.extend_from_slice(b"word/doc.xml");

        let det = match detect(&zip_bytes, &hints) {
            Ok(d) => d,
            Err(e) => panic!("Detection failed: {e:?}"),
        };
        assert_eq!(det.format, FormatId::Docx);
        assert_eq!(det.confidence, Confidence::Medium);
        assert!(det.reason.contains("override"));
    }

    #[test]
    fn test_detect_gzip_and_utf16() {
        let gz_bytes = vec![0x1f, 0x8b, 0x08, 0x00];
        let hints = DetectHints {
            filename: Some("doc.pdf"),
            mime: None,
        };
        let det = match detect(&gz_bytes, &hints) {
            Ok(d) => d,
            Err(e) => panic!("Detection failed: {e:?}"),
        };
        assert_eq!(det.format, FormatId::Pdf);

        let mut utf16_bytes = vec![0xff, 0xfe];
        for c in "# Title\n[link](http://x.y)".encode_utf16() {
            utf16_bytes.extend_from_slice(&c.to_le_bytes());
        }
        let det_u16 = match detect(&utf16_bytes, &DetectHints::default()) {
            Ok(d) => d,
            Err(e) => panic!("Detection failed: {e:?}"),
        };
        assert_eq!(det_u16.format, FormatId::Markdown);
    }
}
