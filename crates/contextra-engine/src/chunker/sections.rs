// FILE-CONTEXT
// ZWECK: Interne Datenstruktur für rohe Markdown-Abschnitte mit Brotkrumen und Metadaten.
// INVARIANTEN: Hält Zeilen, Breadcrumb, Ebene, Zeilenbereich und Byte-Span eines Abschnitts.
// NICHT-OFFENSICHTLICH: Dient als Zwischenrepräsentation vor der Splitting- und Merge-Phase.
// STAND: TS:2026-10-29T00:00:00Z

#[derive(Debug, Clone)]
pub(crate) struct RawSection {
    pub(crate) lines: Vec<String>,
    pub(crate) breadcrumb: String,
    pub(crate) heading_level: u8,
    pub(crate) source_line: usize,
    pub(crate) source_line_end: usize,
    pub(crate) start_byte: usize,
    pub(crate) end_byte: usize,
    pub(crate) tokens: usize,
}
