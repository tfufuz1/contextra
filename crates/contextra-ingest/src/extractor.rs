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
// ZWECK: Trait-Definitionen und Ressourcen-Budgetierung für Format-Extraktoren (WP M2).
// INVARIANTEN: Send + Sync, atomare Budgetierung ohne Zeit, keine std::panic.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::{IngestError, LimitKind};
use crate::format::FormatId;
use crate::ir::DocumentIr;
use crate::limits::{ExtractPolicy, IngestLimits};
use std::sync::atomic::{AtomicU64, Ordering};

/// Abstrakter Extractor-Trait für ein oder mehrere Dokumentformate.
pub trait DocumentExtractor: Send + Sync {
    /// Liste der von diesem Extractor unterstützten Formate.
    fn formats(&self) -> &'static [FormatId];

    /// Extrahiert ein Dokument in die `DocumentIr`.
    fn extract(
        &self,
        format: FormatId,
        input: &[u8],
        ctx: &ExtractContext<'_>,
    ) -> Result<DocumentIr, IngestError>;
}

/// Atomares Ressourcen-Budget zur Absicherung gegen DoS/Zip-Bomben (arbeitsbasiert, ohne Zeit).
#[derive(Debug)]
pub struct ResourceBudget {
    limits: IngestLimits,
    decompressed_bytes: AtomicU64,
    xml_events: AtomicU64,
    text_bytes: AtomicU64,
    cells: AtomicU64,
    pages: AtomicU64,
    nesting_depth: AtomicU64,
}

impl ResourceBudget {
    /// Erstellt ein neues Budget basierend auf den konfigurierten `IngestLimits`.
    pub fn new(limits: IngestLimits) -> Self {
        Self {
            limits,
            decompressed_bytes: AtomicU64::new(0),
            xml_events: AtomicU64::new(0),
            text_bytes: AtomicU64::new(0),
            cells: AtomicU64::new(0),
            pages: AtomicU64::new(0),
            nesting_depth: AtomicU64::new(0),
        }
    }

    /// Verbucht entkomprimierte Bytes.
    pub fn charge_decompressed(&self, n: u64) -> Result<(), IngestError> {
        let prev = self.decompressed_bytes.fetch_add(n, Ordering::Relaxed);
        let current = prev.saturating_add(n);
        if current > self.limits.max_decompressed_bytes {
            Err(IngestError::LimitExceeded {
                kind: LimitKind::DecompressedBytes,
                limit: self.limits.max_decompressed_bytes,
                observed: current,
            })
        } else {
            Ok(())
        }
    }

    /// Verbucht XML-Events.
    pub fn charge_xml_events(&self, n: u64) -> Result<(), IngestError> {
        let prev = self.xml_events.fetch_add(n, Ordering::Relaxed);
        let current = prev.saturating_add(n);
        if current > self.limits.max_xml_events {
            Err(IngestError::LimitExceeded {
                kind: LimitKind::XmlEvents,
                limit: self.limits.max_xml_events,
                observed: current,
            })
        } else {
            Ok(())
        }
    }

    /// Verbucht Text-Bytes.
    pub fn charge_text(&self, n: u64) -> Result<(), IngestError> {
        let prev = self.text_bytes.fetch_add(n, Ordering::Relaxed);
        let current = prev.saturating_add(n);
        if current > self.limits.max_text_bytes {
            Err(IngestError::LimitExceeded {
                kind: LimitKind::TextBytes,
                limit: self.limits.max_text_bytes,
                observed: current,
            })
        } else {
            Ok(())
        }
    }

    /// Verbucht Tabellenzellen.
    pub fn charge_cells(&self, n: u64) -> Result<(), IngestError> {
        let prev = self.cells.fetch_add(n, Ordering::Relaxed);
        let current = prev.saturating_add(n);
        if current > self.limits.max_table_cells {
            Err(IngestError::LimitExceeded {
                kind: LimitKind::TableCells,
                limit: self.limits.max_table_cells,
                observed: current,
            })
        } else {
            Ok(())
        }
    }

    /// Verbucht Seiten.
    pub fn charge_pages(&self, n: u64) -> Result<(), IngestError> {
        let prev = self.pages.fetch_add(n, Ordering::Relaxed);
        let current = prev.saturating_add(n);
        if current > u64::from(self.limits.max_pages) {
            Err(IngestError::LimitExceeded {
                kind: LimitKind::Pages,
                limit: u64::from(self.limits.max_pages),
                observed: current,
            })
        } else {
            Ok(())
        }
    }

    /// Tritt in eine tiefere Dokument-Verschachtelung ein (RAII NestGuard).
    pub fn enter_nested(&self) -> Result<NestGuard<'_>, IngestError> {
        let current = self.nesting_depth.fetch_add(1, Ordering::Relaxed) + 1;
        if current > u64::from(self.limits.max_nesting_depth) {
            self.nesting_depth.fetch_sub(1, Ordering::Relaxed);
            Err(IngestError::LimitExceeded {
                kind: LimitKind::NestingDepth,
                limit: u64::from(self.limits.max_nesting_depth),
                observed: current,
            })
        } else {
            Ok(NestGuard { budget: self })
        }
    }
}

/// RAII-Guard für die Verschachtelungstiefe.
#[derive(Debug)]
pub struct NestGuard<'a> {
    budget: &'a ResourceBudget,
}

impl Drop for NestGuard<'_> {
    fn drop(&mut self) {
        self.budget.nesting_depth.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Extraktions-Kontext, der während des gesamten Parsing-Vorgangs mitgeführt wird.
pub struct ExtractContext<'a> {
    pub limits: &'a IngestLimits,
    pub policy: &'a ExtractPolicy,
    pub budget: &'a ResourceBudget,
    pub registry: &'a crate::registry::ExtractorRegistry,
    pub filename: Option<&'a str>,
}

impl ExtractContext<'_> {
    /// Hilfsmethode zum rekursiven Extrahesieren eines verschachtelten Dokuments (Anhangs).
    pub fn extract_nested(
        &self,
        name: &str,
        mime: Option<&str>,
        bytes: &[u8],
    ) -> Result<DocumentIr, IngestError> {
        let _guard = self.budget.enter_nested()?;

        // Erkennung des Formats
        let hints = crate::detect::DetectHints {
            filename: Some(name),
            mime,
        };
        let detection = crate::detect::detect(bytes, &hints)?;

        // Ausführen der Extraktion über die Registry
        self.registry.extract(detection.format, bytes, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_budget_limits() {
        let limits = IngestLimits {
            max_decompressed_bytes: 100,
            max_nesting_depth: 1,
            ..Default::default()
        };
        let budget = ResourceBudget::new(limits);

        assert!(budget.charge_decompressed(50).is_ok());
        assert!(budget.charge_decompressed(60).is_err());

        {
            let guard = budget.enter_nested();
            assert!(guard.is_ok());
            let second_guard = budget.enter_nested();
            assert!(second_guard.is_err());
        }

        // Nach Drop des guards ist erneutes Eintreten wieder möglich
        assert!(budget.enter_nested().is_ok());
    }
}
