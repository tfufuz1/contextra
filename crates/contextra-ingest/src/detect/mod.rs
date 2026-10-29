// FILE-CONTEXT
// ZWECK: Stub für Format-Erkennung anhand Dateiendung und MIME-Type (WP M3).
// INVARIANTEN: Zero-Panic, deterministisch, signature-fest für spätere P01 Erweiterung.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::format::FormatId;

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

/// Stub für Formaterkennung: Wertet vorrangig Dateiendung und MIME-Type aus.
pub fn detect(_bytes: &[u8], hints: &DetectHints<'_>) -> Result<Detection, IngestError> {
    if let Some(filename) = hints.filename {
        if let Some(pos) = filename.rfind('.') {
            let ext = &filename[pos + 1..];
            if let Some(format) = FormatId::from_extension(ext) {
                return Ok(Detection {
                    format,
                    confidence: Confidence::Medium,
                    reason: "extension match",
                });
            }
        }
    }

    if let Some(mime) = hints.mime {
        let clean_mime = mime.trim().to_lowercase();
        for &fmt in ALL_FORMATS {
            if fmt.mime() == clean_mime {
                return Ok(Detection {
                    format: fmt,
                    confidence: Confidence::Low,
                    reason: "mime match",
                });
            }
        }
    }

    Err(IngestError::UnsupportedFormat {
        format: None,
        reason: "Format could not be detected from hints or content".into(),
    })
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
    fn test_detect_stub() {
        let hints = DetectHints {
            filename: Some("document.pdf"),
            mime: None,
        };
        let det = detect(b"dummy", &hints).unwrap();
        assert_eq!(det.format, FormatId::Pdf);
        assert_eq!(det.confidence, Confidence::Medium);
    }
}
