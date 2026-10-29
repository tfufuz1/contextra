// FILE-CONTEXT
// ZWECK: Konfiguration für das Ingestion-Chunking und Fingerprinting (WP M3).
// INVARIANTEN: Zero-Panic, blake3-Fingerprint für Caching, SPEC-Abweichungen isoliert.
// STAND: TS:2026-10-31T00:00:00Z

use crate::format::ChunkStrategy;
use blake3::Hasher;
use serde::{Deserialize, Serialize};

/// Gesamtkonfiguration für das Ingestion-Chunking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChunkingConfig {
    pub strategy: Option<ChunkStrategy>,
    pub max_tokens: usize,
    pub min_tokens: usize,
    pub hard_limit_pct: u32,
    pub overlap_pct: u32,
    pub include_breadcrumbs: bool,
    pub split_levels: Vec<u8>,

    pub prose: ProseOptions,
    pub tabular: TabularOptions,
    pub paged: PagedOptions,
    pub structured: StructuredOptions,
    pub email: EmailOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProseOptions {
    pub sentence_aware_windows: bool,
    pub footnotes_with_section: bool,
    pub keep_lists_together: bool,
}

impl Default for ProseOptions {
    fn default() -> Self {
        Self {
            sentence_aware_windows: true,
            footnotes_with_section: true,
            keep_lists_together: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TabularMode {
    Auto,
    RowGroups,
    RecordRows,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TabularOptions {
    pub mode: TabularMode,
    pub wide_table_cols: u32,
    pub skip_empty_rows: bool,
    pub drop_empty_columns: bool,
}

impl Default for TabularOptions {
    fn default() -> Self {
        Self {
            mode: TabularMode::Auto,
            wide_table_cols: 12,
            skip_empty_rows: true,
            drop_empty_columns: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PagedOptions {
    pub merge_small_pages: bool,
    pub join_cross_page_paragraphs: bool,
    pub notes_with_slide: bool,
}

impl Default for PagedOptions {
    fn default() -> Self {
        Self {
            merge_small_pages: true,
            join_cross_page_paragraphs: false,
            notes_with_slide: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StructuredOptions {
    pub group_depth: u32,
    pub batch_small_records: bool,
}

impl Default for StructuredOptions {
    fn default() -> Self {
        Self {
            group_depth: 2,
            batch_small_records: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EmailOptions {
    pub include_quoted: bool,
    pub header_context: bool,
}

impl Default for EmailOptions {
    fn default() -> Self {
        Self {
            include_quoted: false,
            header_context: true,
        }
    }
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            strategy: None,
            max_tokens: 512,
            min_tokens: 50,
            hard_limit_pct: 120,
            overlap_pct: 20,
            include_breadcrumbs: true,
            split_levels: vec![1, 2, 3],
            prose: ProseOptions::default(),
            tabular: TabularOptions::default(),
            paged: PagedOptions::default(),
            structured: StructuredOptions::default(),
            email: EmailOptions::default(),
        }
    }
}

impl ChunkingConfig {
    /// Erzeugt einen deterministischen BLAKE3-Fingerprint der Konfiguration (Hex-String).
    pub fn fingerprint(&self) -> String {
        let json = serde_json::to_vec(self).unwrap_or_default();
        let mut hasher = Hasher::new();
        hasher.update(&json);
        hasher.finalize().to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunking_config_fingerprint() {
        let cfg1 = ChunkingConfig::default();
        let cfg2 = ChunkingConfig::default();
        assert_eq!(cfg1.fingerprint(), cfg2.fingerprint());

        let cfg3 = ChunkingConfig {
            max_tokens: 256,
            ..Default::default()
        };
        assert_ne!(cfg1.fingerprint(), cfg3.fingerprint());
    }
}
