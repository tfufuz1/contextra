// FILE-CONTEXT
// STAND: 2026-09-20T00:00:00Z
// ZWECK: Multi-step retrieval primitives (pure synchronous types / traits).

use crate::fusion::SearchResult;
use memfuse_core::{BoxFuture, Result};

/// Default target latency in milliseconds for multi-step search.
pub const DEFAULT_TARGET_LATENCY_MS: f64 = 100.0;

/// Konfiguration für Multi-Step Retrieval.
#[derive(Debug, Clone)]
pub struct MultiStepConfig {
    /// Maximale Iterationsrunden (Standard: 3).
    pub max_rounds: usize,
    /// Mindest-Score-Schwellenwert: unter diesem Wert gilt Runde als unzureichend.
    pub quality_threshold: f32,
    /// Minimale Anzahl an Treffern die den Threshold überschreiten müssen.
    pub min_quality_hits: usize,
    /// Maximale Latenz in Millisekunden für den Multi-Step-Zyklus (Standard: 100.0 ms).
    pub latency_budget_ms: f64,
}

impl Default for MultiStepConfig {
    fn default() -> Self {
        Self {
            max_rounds: 3,
            quality_threshold: 0.5,
            min_quality_hits: 2,
            latency_budget_ms: DEFAULT_TARGET_LATENCY_MS,
        }
    }
}

/// Ergebnis einer Multi-Step-Suche mit Audit-Informationen.
#[derive(Debug)]
pub struct MultiStepResult {
    /// Fusionierte Suchergebnisse.
    pub results: Vec<SearchResult>,
    /// Anzahl der tatsächlich durchgeführten Runden.
    pub rounds_executed: usize,
    /// Queries die in den Folgerunden verwendet wurden.
    pub sub_queries: Vec<String>,
}

/// Trait für Query-Rewriting (LLM-agnostisch).
pub trait QueryRewriter: Send + Sync {
    /// Generiert alternative Teil-Queries basierend auf bisherigen Ergebnissen.
    fn rewrite<'a>(
        &'a self,
        original_query: &'a str,
        current_results: &'a [SearchResult],
    ) -> BoxFuture<'a, Result<Vec<String>>>;
}
