// FILE-CONTEXT
// ZWECK: Moduldefinition für Entitätsextraktion (OpenIE & Rule-Based) (Layer 3 - Engine).
// INVARIANTEN: Zero panic / unwrap / expect in non-test paths.
// STAND: 2026-10-01

//! Modul für OpenIE und regelbasierte Entitäts- und Relationsextraktion.

#![forbid(unsafe_code)]

pub mod open_ie;
pub mod rule_based;
#[cfg(test)]
pub mod tests;
pub mod types;

pub use open_ie::extract_triples;
pub use rule_based::RuleBasedExtractor;
pub use types::{EntityExtractionConfig, ExtractedTriple, ExtractorMode};

use contextra_ports::{BoxFuture, LlmTextGenerator};
use contextra_types::{ContextraError, Result};

/// Gemeinsamer Trait für alle Entitätsextraktoren in Contextra-Engine.
pub trait EntityExtractor: Send + Sync {
    /// Extrahiert Wissens-Tripel aus dem übergebenen Freitext.
    fn extract_triples<'a>(
        &'a self,
        text: &'a str,
        generator: Option<&'a dyn LlmTextGenerator>,
        config: &'a EntityExtractionConfig,
    ) -> BoxFuture<'a, Result<Vec<ExtractedTriple>>>;
}

/// Implementierung des OpenIE-Extraktors hinter `EntityExtractor`.
#[derive(Debug, Clone, Default)]
pub struct OpenIeExtractor;

impl EntityExtractor for OpenIeExtractor {
    fn extract_triples<'a>(
        &'a self,
        text: &'a str,
        generator: Option<&'a dyn LlmTextGenerator>,
        config: &'a EntityExtractionConfig,
    ) -> BoxFuture<'a, Result<Vec<ExtractedTriple>>> {
        Box::pin(async move {
            let gen = generator.ok_or_else(|| {
                ContextraError::InvalidInput(
                    "LlmTextGenerator missing for OpenIeExtractor".to_string(),
                )
            })?;
            open_ie::extract_triples(text, gen, config).await
        })
    }
}

impl EntityExtractor for RuleBasedExtractor {
    fn extract_triples<'a>(
        &'a self,
        text: &'a str,
        _generator: Option<&'a dyn LlmTextGenerator>,
        config: &'a EntityExtractionConfig,
    ) -> BoxFuture<'a, Result<Vec<ExtractedTriple>>> {
        Box::pin(async move { self.extract(text, config) })
    }
}

/// Erstellt eine Instanz des konfigurierten Extraktors.
pub fn create_extractor(mode: ExtractorMode) -> Box<dyn EntityExtractor> {
    match mode {
        ExtractorMode::OpenIe => Box::new(OpenIeExtractor),
        ExtractorMode::RuleBased => Box::new(RuleBasedExtractor::new()),
    }
}

/// Hilfsfunktion zur Ausführung der Extraktion für einen expliziten Extraktor-Modus.
pub async fn extract_triples_for_mode(
    mode: ExtractorMode,
    text: &str,
    generator: Option<&dyn LlmTextGenerator>,
    config: &EntityExtractionConfig,
) -> Result<Vec<ExtractedTriple>> {
    let extractor = create_extractor(mode);
    extractor.extract_triples(text, generator, config).await
}

/// Hilfsfunktion zur Ausführung der Extraktion basierend auf der Standard-Konfiguration (OpenIE).
pub async fn extract_triples_for_config(
    text: &str,
    generator: Option<&dyn LlmTextGenerator>,
    config: &EntityExtractionConfig,
) -> Result<Vec<ExtractedTriple>> {
    extract_triples_for_mode(ExtractorMode::OpenIe, text, generator, config).await
}
