// FILE-CONTEXT
// ZWECK: Ingestion-Hook fuer LLM- und regelbasierte Entitaetsextraktion (OpenIE / Rule-Based).
// INVARIANTEN: Default enabled: true (sofern nicht auto-extraction-opt-out gesetzt); Best-Effort Fehlerbehandlung; Keine Panics.

#![forbid(unsafe_code)]

use crate::collection::Collection;
#[cfg(feature = "entity-extraction")]
pub use crate::extraction::{EntityExtractionConfig, ExtractorMode};
use contextra_ports::{BoxFuture, LlmTextGenerator, StorageEngine, VectorIndex};
pub use contextra_types::AutoExtractionMode;
use contextra_types::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[cfg(not(feature = "entity-extraction"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtractorMode {
    OpenIe,
    RuleBased,
}

#[cfg(not(feature = "entity-extraction"))]
impl Default for ExtractorMode {
    fn default() -> Self {
        Self::OpenIe
    }
}

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

/// Fallback / Dummy LLM Generator fuer reine RuleBased Extraktion ohne externes LLM.
#[derive(Debug, Clone)]
struct NoopLlmGenerator;

impl LlmTextGenerator for NoopLlmGenerator {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, Result<String>> {
        Box::pin(async move { Ok("[]".to_string()) })
    }
}

/// Liefert den empfohlenen `AutoExtractionMode` fuer das gegebene "regulated"-Flag.
///
/// Gemäss v17 Teil 10.5 soll in EnterpriseRegulated / Regulierungs-Kontexten
/// die automatische Extraktion standardmäßig deaktiviert werden (`Disabled`), um Datenminimierung
/// sicherzustellen.
///
/// **Hinweis:** Diese Funktion ändert NICHT den globalen Standardwert von `AutoExtractionMode::default()`.
/// Die Entscheidung über eine globale Anpassung obliegt der Produktverantwortung und bleibt als offener
/// Punkt im PR-Text dokumentiert.
#[inline]
pub fn recommended_mode_for_regulated(is_regulated: bool) -> AutoExtractionMode {
    if is_regulated {
        AutoExtractionMode::Disabled
    } else {
        AutoExtractionMode::Enabled
    }
}

/// Konfiguration fuer die automatische Entitaetsextraktion beim Einfuegen von Dokumenten.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoExtractionConfig {
    /// Laufzeit-Modus fuer die automatische Extraktion (Default: Enabled, bzw. Disabled wenn Feature `auto-extraction-opt-out` aktiv ist).
    #[serde(default)]
    pub mode: AutoExtractionMode,
    /// Ob die automatische Extraktion aktiviert ist (Default: true, bzw. false wenn Feature `auto-extraction-opt-out` aktiv ist).
    pub enabled: bool,
    /// Konfiguration fuer die unterliegende Entitaetsextraktion.
    pub entity_config: EntityExtractionConfig,
}

impl AutoExtractionConfig {
    /// Erstellt eine neue `AutoExtractionConfig` mit dem angegebenen Modus.
    pub fn new(mode: AutoExtractionMode) -> Self {
        Self {
            mode,
            enabled: mode.is_enabled(),
            entity_config: EntityExtractionConfig::default(),
        }
    }

    /// Erstellt eine neue `AutoExtractionConfig` unter Verwendung der Empfehlung fuer Regulierungs-Kontexte.
    pub fn for_regulated(is_regulated: bool) -> Self {
        Self::new(recommended_mode_for_regulated(is_regulated))
    }

    /// Setzt den Laufzeit-Modus (Builder Pattern).
    pub fn with_mode(mut self, mode: AutoExtractionMode) -> Self {
        self.mode = mode;
        self.enabled = mode.is_enabled();
        self
    }

    /// Prueft, ob die automatische Extraktion unter Beruecksichtigung aller Abschaltwege aktiv ist.
    ///
    /// Precedence / Abschaltkette:
    /// 1. Compile-Zeit feature `auto-extraction-opt-out` -> erzwungen `false`
    /// 2. Laufzeit `mode == AutoExtractionMode::Disabled` -> `false`
    /// 3. Laufzeit `enabled == false` -> `false`
    #[inline]
    pub fn is_enabled(&self) -> bool {
        if cfg!(feature = "auto-extraction-opt-out") {
            return false;
        }
        self.enabled && self.mode.is_enabled()
    }
}

impl Default for AutoExtractionConfig {
    fn default() -> Self {
        let mode = if cfg!(feature = "auto-extraction-opt-out") {
            AutoExtractionMode::Disabled
        } else {
            AutoExtractionMode::default()
        };
        let enabled = mode.is_enabled() && !cfg!(feature = "auto-extraction-opt-out");
        Self {
            mode,
            enabled,
            entity_config: EntityExtractionConfig::default(),
        }
    }
}

#[allow(clippy::type_complexity)]
static AUTO_EXTRACTION_REGISTRY: std::sync::LazyLock<
    parking_lot::RwLock<
        ahash::AHashMap<
            usize,
            (
                Arc<dyn LlmTextGenerator>,
                AutoExtractionConfig,
                ExtractorMode,
            ),
        >,
    >,
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
        self.set_auto_extraction_with_mode(generator, config, ExtractorMode::OpenIe);
    }

    /// Sets or updates auto extraction with an explicit extractor mode.
    pub fn set_auto_extraction_with_mode(
        &self,
        generator: Arc<dyn LlmTextGenerator>,
        config: AutoExtractionConfig,
        extractor_mode: ExtractorMode,
    ) {
        let key = Arc::as_ptr(&self.kv_locks) as usize;
        AUTO_EXTRACTION_REGISTRY
            .write()
            .insert(key, (generator, config, extractor_mode));
    }

    /// Sets or updates auto extraction without an external LLM generator (suitable for RuleBased mode).
    pub fn set_auto_extraction_config(&self, config: AutoExtractionConfig) {
        let dummy_gen: Arc<dyn LlmTextGenerator> = Arc::new(NoopLlmGenerator);
        self.set_auto_extraction_with_mode(dummy_gen, config, ExtractorMode::RuleBased);
    }

    /// Returns the configured auto extraction generator and config, if present.
    pub fn auto_extraction_config(
        &self,
    ) -> Option<(Arc<dyn LlmTextGenerator>, AutoExtractionConfig)> {
        let key = Arc::as_ptr(&self.kv_locks) as usize;
        AUTO_EXTRACTION_REGISTRY
            .read()
            .get(&key)
            .map(|(g, c, _)| (g.clone(), c.clone()))
    }

    /// Returns the configured auto extraction generator, config, and extractor mode.
    pub fn auto_extraction_config_with_extractor(
        &self,
    ) -> Option<(
        Arc<dyn LlmTextGenerator>,
        AutoExtractionConfig,
        ExtractorMode,
    )> {
        let key = Arc::as_ptr(&self.kv_locks) as usize;
        AUTO_EXTRACTION_REGISTRY.read().get(&key).cloned()
    }
}

/// Extrahiert Wissens-Tripel aus `text` mittels konfiguriertem Extraktor und verknuepft Subjekt und Objekt in `collection`.
///
/// # Kostenschutz
/// Das Limit `max_llm_calls_per_cycle` (bei OpenIE) bzw. `MAX_EXTRACTION_TEXT_LENGTH`/`MAX_EXTRACTED_TRIPLES`
/// (bei RuleBased) wird im Extraktor durchgesetzt und dadurch hier automatisch eingehalten.
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
    if !cfg.is_enabled() {
        return Ok(0);
    }

    let extractor_mode = collection
        .auto_extraction_config_with_extractor()
        .map(|(_, _, m)| m)
        .unwrap_or(ExtractorMode::OpenIe);

    #[cfg(feature = "entity-extraction")]
    {
        let triples = match crate::extraction::extract_triples_for_mode(
            extractor_mode,
            text,
            Some(generator),
            &cfg.entity_config,
        )
        .await
        {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(doc_id = %doc_id, error = %e, "extract_triples_for_mode failed during auto extraction");
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
                .relate_with_provenance(
                    &triple.subject,
                    &triple.object,
                    &triple.predicate,
                    Some(doc_id),
                )
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
        let _ = (collection, doc_id, text, generator, extractor_mode);
        Ok(0)
    }
}
