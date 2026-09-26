// FILE-CONTEXT
// ZWECK: Ingestion-Hook fuer LLM-gestuetzte Entitaetsextraktion (OpenIE).
// INVARIANTEN: Default enabled: true (sofern nicht auto-extraction-opt-out gesetzt); Best-Effort Fehlerbehandlung; Keine Panics.

#![forbid(unsafe_code)]

use crate::collection::Collection;
#[cfg(feature = "entity-extraction")]
pub use crate::extraction::EntityExtractionConfig;
use contextra_ports::{LlmTextGenerator, StorageEngine, VectorIndex};
use contextra_types::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[cfg(not(feature = "entity-extraction"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityExtractionConfig {
    pub enabled: bool,
    pub max_llm_calls_per_cycle: usize,
    pub min_confidence: f32,
}

#[cfg(not(feature = "entity-extraction"))]
impl Default for EntityExtractionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_llm_calls_per_cycle: 10,
            min_confidence: 0.5,
        }
    }
}

/// Konfiguration fuer die automatische Entitaetsextraktion beim Einfuegen von Dokumenten.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoExtractionConfig {
    /// Ob die automatische Extraktion aktiviert ist (Default: true, bzw. false wenn Feature `auto-extraction-opt-out` aktiv ist).
    pub enabled: bool,
    /// Konfiguration fuer die unterliegende Entitaetsextraktion.
    pub entity_config: EntityExtractionConfig,
}

impl Default for AutoExtractionConfig {
    fn default() -> Self {
        Self {
            #[cfg(not(feature = "auto-extraction-opt-out"))]
            enabled: true,
            #[cfg(feature = "auto-extraction-opt-out")]
            enabled: false,
            entity_config: EntityExtractionConfig::default(),
        }
    }
}

#[allow(clippy::type_complexity)]
static AUTO_EXTRACTION_REGISTRY: std::sync::LazyLock<
    parking_lot::RwLock<ahash::AHashMap<usize, (Arc<dyn LlmTextGenerator>, AutoExtractionConfig)>>,
> = std::sync::LazyLock::new(|| parking_lot::RwLock::new(ahash::AHashMap::new()));

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Configures auto extraction with an LLM generator and config (builder pattern).
    pub fn with_auto_extraction(
        self,
        generator: Arc<dyn LlmTextGenerator>,
        config: AutoExtractionConfig,
    ) -> Self {
        self.set_auto_extraction(generator, config);
        self
    }

    /// Sets or updates the auto extraction generator and config for this collection.
    pub fn set_auto_extraction(
        &self,
        generator: Arc<dyn LlmTextGenerator>,
        config: AutoExtractionConfig,
    ) {
        let key = Arc::as_ptr(&self.kv_locks) as usize;
        AUTO_EXTRACTION_REGISTRY.write().insert(key, (generator, config));
    }

    /// Returns the configured auto extraction generator and config, if present.
    pub fn auto_extraction_config(
        &self,
    ) -> Option<(Arc<dyn LlmTextGenerator>, AutoExtractionConfig)> {
        let key = Arc::as_ptr(&self.kv_locks) as usize;
        AUTO_EXTRACTION_REGISTRY.read().get(&key).cloned()
    }
}

/// Extrahiert Wissens-Tripel aus `text` mittels `generator` und verknuepft Subjekt und Objekt in `collection`.
///
/// # Kostenschutz
/// Das Limit `max_llm_calls_per_cycle` aus `EntityExtractionConfig` wird direkt in `extract_triples` durchgesetzt
/// und dadurch hier automatisch eingehalten.
///
/// # Idempotenz
/// Die Erstellung von Beziehungen via `Collection::relate` ist idempotent. Bereits existierende Beziehungen
/// werden von `relate` ohne Fehler ueberschrieben/aktualisiert (`Ok(())`).
pub(crate) async fn auto_extract_and_relate<S: StorageEngine, V: VectorIndex>(
    collection: &Collection<S, V>,
    doc_id: &str,
    text: &str,
    generator: &dyn LlmTextGenerator,
    cfg: &AutoExtractionConfig,
) -> Result<usize> {
    if !cfg.enabled {
        return Ok(0);
    }

    #[cfg(feature = "entity-extraction")]
    {
        let triples = match crate::extraction::extract_triples(text, generator, &cfg.entity_config).await {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(doc_id = %doc_id, error = %e, "extract_triples failed during auto extraction");
                return Err(e);
            }
        };

        let min_conf = cfg.entity_config.min_confidence;
        let mut created_edges = 0;

        for triple in triples {
            // Defensiver Check fuer Mindestkonfidenz
            if triple.confidence < min_conf {
                continue;
            }

            match collection
                .relate_with_provenance(&triple.subject, &triple.object, &triple.predicate, Some(doc_id))
                .await
            {
                Ok(_) => {
                    created_edges += 1;
                }
                Err(e) => {
                    tracing::warn!(
                        doc_id = %doc_id,
                        subject = %triple.subject,
                        object = %triple.object,
                        predicate = %triple.predicate,
                        error = %e,
                        "Failed to relate auto-extracted triple"
                    );
                }
            }
        }

        Ok(created_edges)
    }

    #[cfg(not(feature = "entity-extraction"))]
    {
        let _ = (collection, doc_id, text, generator);
        Ok(0)
    }
}
