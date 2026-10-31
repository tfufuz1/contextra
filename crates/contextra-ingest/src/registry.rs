//! Modul
//!
//! # Zweck & Architektur
//! Komponente des  Subsystems (Ring 4 Adapter-Schicht).
//! Stellt Parser-, Extraktions- oder Hilfsfunktionen fuer die strukturierte Dokumentverarbeitung bereit.
//!
//! # Invarianten
//! - Zero-Panic: Alle Operationen sind defensiv aufgebaut.
//! - Fail-Closed: Fehlerhafte oder unvollstaendige Daten fuehren zu typisierten Fehlern.
//! - Kein externer I/O ueber Netzwerk oder unkontrollierte Dateisystempfade.

// FILE-CONTEXT
// ZWECK: ExtractorRegistry zur Registrierung und Panik-Isolierung von Format-Extraktoren (WP M3).
// INVARIANTEN: Zero-Panic, catch_unwind Isolierung, kein HashMap/HashSet (BTreeMap), feature-basierte Defaults.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::extractor::{DocumentExtractor, ExtractContext};
use crate::format::FormatId;
use crate::ir::DocumentIr;
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

/// Registrierungstabelle für Format-Extraktoren.
#[derive(Clone, Default)]
pub struct ExtractorRegistry {
    extractors: BTreeMap<FormatId, Arc<dyn DocumentExtractor>>,
}

impl ExtractorRegistry {
    /// Erzeugt eine leere ExtractorRegistry.
    pub fn new_empty() -> Self {
        Self {
            extractors: BTreeMap::new(),
        }
    }

    /// Registriert einen Extraktor für alle von ihm unterstützten Formate.
    pub fn register(&mut self, extractor: Box<dyn DocumentExtractor>) {
        let arc: Arc<dyn DocumentExtractor> = Arc::from(extractor);
        for &fmt in arc.formats() {
            self.extractors.insert(fmt, Arc::clone(&arc));
        }
    }

    /// Registriert Standard-Extraktoren gemäß den aktivierten Feature-Flags (`fmt-*`).
    pub fn with_defaults() -> Self {
        let mut registry = Self::new_empty();

        #[cfg(feature = "fmt-text")]
        registry.register(Box::new(crate::formats::text::TextExtractor::new()));

        #[cfg(feature = "fmt-web")]
        registry.register(Box::new(crate::formats::web::WebExtractor::new()));

        #[cfg(feature = "fmt-docx")]
        registry.register(Box::new(crate::formats::docx::DocxExtractor::new()));

        #[cfg(feature = "fmt-pptx")]
        registry.register(Box::new(crate::formats::pptx::PptxExtractor::new()));

        #[cfg(feature = "fmt-sheets")]
        registry.register(Box::new(crate::formats::sheets::SheetsExtractor::new()));

        #[cfg(feature = "fmt-odf")]
        registry.register(Box::new(crate::formats::odf::OdfExtractor::new()));

        #[cfg(feature = "fmt-pdf")]
        registry.register(Box::new(crate::formats::pdf::PdfExtractor::new()));

        #[cfg(feature = "fmt-ole")]
        registry.register(Box::new(crate::formats::ole::OleExtractor::new()));

        #[cfg(feature = "fmt-email")]
        registry.register(Box::new(crate::formats::email::EmailExtractor::new()));

        #[cfg(feature = "fmt-structured")]
        registry.register(Box::new(
            crate::formats::structured::StructuredExtractor::new(),
        ));

        registry
    }

    /// Fuehrt die Extraktion fuer ein Format durch und fängt Panics des Parsers fail-closed ab.
    pub fn extract(
        &self,
        format: FormatId,
        bytes: &[u8],
        ctx: &ExtractContext<'_>,
    ) -> Result<DocumentIr, IngestError> {
        let extractor =
            self.extractors
                .get(&format)
                .ok_or_else(|| IngestError::UnsupportedFormat {
                    format: Some(format),
                    reason: format!(
                        "Feature fmt-{} not enabled or no extractor registered",
                        format.name()
                    ),
                })?;

        let res = catch_unwind(AssertUnwindSafe(|| extractor.extract(format, bytes, ctx)));

        match res {
            Ok(result) => result,
            Err(_) => Err(IngestError::ParserPanic { format }),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )]
    use super::*;
    use crate::limits::{ExtractPolicy, IngestLimits};

    struct PanickingExtractor;
    impl DocumentExtractor for PanickingExtractor {
        fn formats(&self) -> &'static [FormatId] {
            &[FormatId::Txt]
        }
        fn extract(
            &self,
            _format: FormatId,
            _input: &[u8],
            _ctx: &ExtractContext<'_>,
        ) -> Result<DocumentIr, IngestError> {
            panic!("Simulator panic in parser");
        }
    }

    #[test]
    fn test_panic_isolation_in_registry() {
        let mut reg = ExtractorRegistry::new_empty();
        reg.register(Box::new(PanickingExtractor));

        let limits = IngestLimits::default();
        let policy = ExtractPolicy::default();
        let budget = crate::extractor::ResourceBudget::new(limits.clone());
        let ctx = ExtractContext {
            limits: &limits,
            policy: &policy,
            budget: &budget,
            registry: &reg,
            filename: None,
        };

        let err = reg.extract(FormatId::Txt, b"hello", &ctx).unwrap_err();
        assert_eq!(
            err,
            IngestError::ParserPanic {
                format: FormatId::Txt
            }
        );
    }
}
