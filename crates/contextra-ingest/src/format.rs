// FILE-CONTEXT
// ZWECK: Dokumentformat-IDs und Chunk-Strategie-Zuordnungen für contextra-ingest (WP M2).
// INVARIANTEN: Zero-Panic, #[non_exhaustive], kleingeschriebene stabile Namen, Dateiendungs- und MIME-Zuordnung.
// STAND: TS:2026-10-31T00:00:00Z

use serde::{Deserialize, Serialize};

/// Aufzählung aller unterstützen Dokumentformate in contextra-ingest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FormatId {
    Txt,
    Markdown,
    Rst,
    AsciiDoc,
    Tex,
    Rtf,
    Html,
    Xhtml,
    Epub,
    Docx,
    Pptx,
    Xlsx,
    Xls,
    Xlsb,
    Ods,
    Csv,
    Tsv,
    Odt,
    Odp,
    Pdf,
    Doc,
    Ppt,
    Msg,
    Eml,
    Mbox,
    Json,
    Jsonl,
    Xml,
    Yaml,
    Toml,
}

/// Strategie für die strukturierte Zerlegung (Chunking) eines Dokuments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ChunkStrategy {
    Prose,
    Tabular,
    Paged,
    Structured,
    Email,
}

impl FormatId {
    /// Stabile, kleingeschriebene Textidentifikation des Formats.
    pub fn name(self) -> &'static str {
        match self {
            FormatId::Txt => "txt",
            FormatId::Markdown => "markdown",
            FormatId::Rst => "rst",
            FormatId::AsciiDoc => "asciidoc",
            FormatId::Tex => "tex",
            FormatId::Rtf => "rtf",
            FormatId::Html => "html",
            FormatId::Xhtml => "xhtml",
            FormatId::Epub => "epub",
            FormatId::Docx => "docx",
            FormatId::Pptx => "pptx",
            FormatId::Xlsx => "xlsx",
            FormatId::Xls => "xls",
            FormatId::Xlsb => "xlsb",
            FormatId::Ods => "ods",
            FormatId::Csv => "csv",
            FormatId::Tsv => "tsv",
            FormatId::Odt => "odt",
            FormatId::Odp => "odp",
            FormatId::Pdf => "pdf",
            FormatId::Doc => "doc",
            FormatId::Ppt => "ppt",
            FormatId::Msg => "msg",
            FormatId::Eml => "eml",
            FormatId::Mbox => "mbox",
            FormatId::Json => "json",
            FormatId::Jsonl => "jsonl",
            FormatId::Xml => "xml",
            FormatId::Yaml => "yaml",
            FormatId::Toml => "toml",
        }
    }

    /// Liste bekannter Dateiendungen (ohne führenden Punkt).
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            FormatId::Txt => &["txt", "text"],
            FormatId::Markdown => &["md", "markdown", "mdown"],
            FormatId::Rst => &["rst"],
            FormatId::AsciiDoc => &["adoc", "asciidoc"],
            FormatId::Tex => &["tex", "latex"],
            FormatId::Rtf => &["rtf"],
            FormatId::Html => &["html", "htm"],
            FormatId::Xhtml => &["xhtml"],
            FormatId::Epub => &["epub"],
            FormatId::Docx => &["docx"],
            FormatId::Pptx => &["pptx"],
            FormatId::Xlsx => &["xlsx"],
            FormatId::Xls => &["xls"],
            FormatId::Xlsb => &["xlsb"],
            FormatId::Ods => &["ods"],
            FormatId::Csv => &["csv"],
            FormatId::Tsv => &["tsv"],
            FormatId::Odt => &["odt"],
            FormatId::Odp => &["odp"],
            FormatId::Pdf => &["pdf"],
            FormatId::Doc => &["doc"],
            FormatId::Ppt => &["ppt"],
            FormatId::Msg => &["msg"],
            FormatId::Eml => &["eml"],
            FormatId::Mbox => &["mbox"],
            FormatId::Json => &["json"],
            FormatId::Jsonl => &["jsonl", "ndjson"],
            FormatId::Xml => &["xml"],
            FormatId::Yaml => &["yaml", "yml"],
            FormatId::Toml => &["toml"],
        }
    }

    /// Standard-MIME-Type des Formats.
    pub fn mime(self) -> &'static str {
        match self {
            FormatId::Txt => "text/plain",
            FormatId::Markdown => "text/markdown",
            FormatId::Rst => "text/x-rst",
            FormatId::AsciiDoc => "text/asciidoc",
            FormatId::Tex => "text/x-tex",
            FormatId::Rtf => "application/rtf",
            FormatId::Html => "text/html",
            FormatId::Xhtml => "application/xhtml+xml",
            FormatId::Epub => "application/epub+zip",
            FormatId::Docx => {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            }
            FormatId::Pptx => {
                "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            }
            FormatId::Xlsx => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            FormatId::Xls => "application/vnd.ms-excel",
            FormatId::Xlsb => "application/vnd.ms-excel.sheet.binary.macroenabled.12",
            FormatId::Ods => "application/vnd.oasis.opendocument.spreadsheet",
            FormatId::Csv => "text/csv",
            FormatId::Tsv => "text/tab-separated-values",
            FormatId::Odt => "application/vnd.oasis.opendocument.text",
            FormatId::Odp => "application/vnd.oasis.opendocument.presentation",
            FormatId::Pdf => "application/pdf",
            FormatId::Doc => "application/msword",
            FormatId::Ppt => "application/vnd.ms-powerpoint",
            FormatId::Msg => "application/vnd.ms-outlook",
            FormatId::Eml => "message/rfc822",
            FormatId::Mbox => "application/mbox",
            FormatId::Json => "application/json",
            FormatId::Jsonl => "application/x-ndjson",
            FormatId::Xml => "application/xml",
            FormatId::Yaml => "application/x-yaml",
            FormatId::Toml => "application/toml",
        }
    }

    /// Erstellt `FormatId` anhand einer Dateiendung (ohne Punkt, Case-Insensitive).
    pub fn from_extension(ext: &str) -> Option<Self> {
        let clean = ext.trim().trim_start_matches('.').to_lowercase();
        match clean.as_str() {
            "txt" | "text" => Some(FormatId::Txt),
            "md" | "markdown" | "mdown" => Some(FormatId::Markdown),
            "rst" => Some(FormatId::Rst),
            "adoc" | "asciidoc" => Some(FormatId::AsciiDoc),
            "tex" | "latex" => Some(FormatId::Tex),
            "rtf" => Some(FormatId::Rtf),
            "html" | "htm" => Some(FormatId::Html),
            "xhtml" => Some(FormatId::Xhtml),
            "epub" => Some(FormatId::Epub),
            "docx" => Some(FormatId::Docx),
            "pptx" => Some(FormatId::Pptx),
            "xlsx" => Some(FormatId::Xlsx),
            "xls" => Some(FormatId::Xls),
            "xlsb" => Some(FormatId::Xlsb),
            "ods" => Some(FormatId::Ods),
            "csv" => Some(FormatId::Csv),
            "tsv" => Some(FormatId::Tsv),
            "odt" => Some(FormatId::Odt),
            "odp" => Some(FormatId::Odp),
            "pdf" => Some(FormatId::Pdf),
            "doc" => Some(FormatId::Doc),
            "ppt" => Some(FormatId::Ppt),
            "msg" => Some(FormatId::Msg),
            "eml" => Some(FormatId::Eml),
            "mbox" => Some(FormatId::Mbox),
            "json" => Some(FormatId::Json),
            "jsonl" | "ndjson" => Some(FormatId::Jsonl),
            "xml" => Some(FormatId::Xml),
            "yaml" | "yml" => Some(FormatId::Yaml),
            "toml" => Some(FormatId::Toml),
            _ => None,
        }
    }

    /// Standard-Chunking-Strategie für ein Format.
    pub fn default_strategy(self) -> ChunkStrategy {
        match self {
            FormatId::Pdf | FormatId::Pptx | FormatId::Odp | FormatId::Ppt => ChunkStrategy::Paged,
            FormatId::Xlsx
            | FormatId::Xls
            | FormatId::Xlsb
            | FormatId::Ods
            | FormatId::Csv
            | FormatId::Tsv => ChunkStrategy::Tabular,
            FormatId::Json | FormatId::Jsonl | FormatId::Xml | FormatId::Yaml | FormatId::Toml => {
                ChunkStrategy::Structured
            }
            FormatId::Eml | FormatId::Mbox | FormatId::Msg => ChunkStrategy::Email,
            _ => ChunkStrategy::Prose,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_mappings() {
        assert_eq!(FormatId::from_extension("md"), Some(FormatId::Markdown));
        assert_eq!(FormatId::from_extension(".PDF"), Some(FormatId::Pdf));
        assert_eq!(FormatId::Pdf.default_strategy(), ChunkStrategy::Paged);
        assert_eq!(FormatId::Csv.default_strategy(), ChunkStrategy::Tabular);
        assert_eq!(FormatId::Json.default_strategy(), ChunkStrategy::Structured);
        assert_eq!(FormatId::Eml.default_strategy(), ChunkStrategy::Email);
        assert_eq!(FormatId::Txt.default_strategy(), ChunkStrategy::Prose);
    }
}
