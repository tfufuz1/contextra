//! # Modul: config.rs
//!
//! **Zweck:** Konfigurationsparameter für Chunk-Größen, Overlap und Grenzzeichen.
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, deterministische Textverarbeitung.

// FILE-CONTEXT
// ZWECK: Konfiguration des Markdown-Chunkers (WP-7.1 / Teil 6.12 Spezifikation).
// INVARIANTEN: Standardwerte entsprechen Ingestion-Vorgaben; max_tokens > 0.
// NICHT-OFFENSICHTLICH: split_levels steuert, bei welchen Überschriftsebenen gesplittet wird.
// STAND: TS:2026-10-29T00:00:00Z

/// Configuration for the Markdown chunker.
#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    /// Maximum tokens per chunk (soft limit, hard limit is this * 1.2).
    /// Default: 512 tokens (aligned with standard dense embedding models).
    pub max_tokens: usize,
    /// Minimum tokens per chunk (merge threshold).
    pub min_tokens: usize,
    /// Include heading breadcrumb as metadata in resulting chunks.
    pub include_breadcrumbs: bool,
    /// Heading levels to split on (1 = H1, 2 = H2, ...).
    pub split_levels: Vec<u8>,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            min_tokens: 50,
            include_breadcrumbs: true,
            split_levels: vec![1, 2, 3],
        }
    }
}
