//! Modul
//!
//! # Zweck & Architektur
//! Stellt Parser-, Extraktions- oder Hilfsfunktionen fuer die strukturierte Dokumentverarbeitung bereit.
//!
//! # Invarianten
//! - Zero-Panic: Alle Operationen sind defensiv aufgebaut.
//! - Fail-Closed: Fehlerhafte oder unvollstaendige Daten fuehren zu typisierten Fehlern.
//! - Kein externer I/O ueber Netzwerk oder unkontrollierte Dateisystempfade.

// Tabellen- & Spreadsheet-Extractor (Xlsx, Xls, Xlsb, Ods, Csv, Tsv).
// Invarianten: Zero-Panic, typisierte IngestError, kein unsafe, defensive Budgetierung.

pub mod csv_fmt;
pub mod excel;

use crate::error::IngestError;
use crate::extractor::{DocumentExtractor, ExtractContext};
use crate::format::FormatId;
use crate::ir::DocumentIr;

#[derive(Debug, Default)]
pub struct SheetsExtractor;

impl SheetsExtractor {
    pub fn new() -> Self {
        Self
    }
}

const FORMATS: &[FormatId] = &[
    FormatId::Xlsx,
    FormatId::Xls,
    FormatId::Xlsb,
    FormatId::Ods,
    FormatId::Csv,
    FormatId::Tsv,
];

impl DocumentExtractor for SheetsExtractor {
    fn formats(&self) -> &'static [FormatId] {
        FORMATS
    }

    fn extract(
        &self,
        format: FormatId,
        input: &[u8],
        ctx: &ExtractContext<'_>,
    ) -> Result<DocumentIr, IngestError> {
        match format {
            FormatId::Csv | FormatId::Tsv => csv_fmt::extract_csv_tsv(format, input, ctx),
            FormatId::Xlsx | FormatId::Xls | FormatId::Xlsb | FormatId::Ods => {
                excel::extract_excel(format, input, ctx)
            }
            _ => Err(IngestError::UnsupportedFormat {
                format: Some(format),
                reason: format!(
                    "Format {:?} wird von SheetsExtractor nicht unterstützt",
                    format
                ),
            }),
        }
    }
}
