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
// ZWECK: CSV/TSV Extraktionslogik mit Kodierungserkennung, Delimiter-Erkennung & RFC 4180 support.
// INVARIANTEN: Zero-Panic, typisierte IngestError, kein unsafe, defensive Budgetierung.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::extractor::ExtractContext;
use crate::format::FormatId;
use crate::ir::{
    BlockKind, Boundary, DocMeta, DocumentIr, ExtractWarning, Locator, Table, WarningCode,
};

/// Parst CSV oder TSV Dokumente in eine `DocumentIr`.
pub fn extract_csv_tsv(
    format: FormatId,
    input: &[u8],
    ctx: &ExtractContext<'_>,
) -> Result<DocumentIr, IngestError> {
    if input.is_empty() {
        let mut ir = DocumentIr::new(format);
        ir.meta = DocMeta {
            page_count: Some(1),
            ..Default::default()
        };
        ir.push(
            BlockKind::Boundary(Boundary::Sheet {
                name: "csv".into(),
                index: 0,
            }),
            Locator {
                sheet: Some("csv".into()),
                ..Default::default()
            },
        );
        return Ok(ir);
    }

    let mut warnings = Vec::new();
    let text = decode_text(input, &mut warnings);

    let delimiter = if format == FormatId::Tsv {
        b'\t'
    } else {
        detect_delimiter(&text)
    };

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(delimiter)
        .flexible(true)
        .from_reader(text.as_bytes());

    let mut raw_rows: Vec<Vec<String>> = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| IngestError::Corrupt {
            format: Some(format),
            detail: format!("CSV Parsing-Fehler: {e}"),
        })?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        raw_rows.push(row);
    }

    let mut ir = DocumentIr::new(format);
    ir.meta = DocMeta {
        page_count: Some(1),
        ..Default::default()
    };
    ir.warnings = warnings;

    ir.push(
        BlockKind::Boundary(Boundary::Sheet {
            name: "csv".into(),
            index: 0,
        }),
        Locator {
            sheet: Some("csv".into()),
            ..Default::default()
        },
    );

    build_tables_from_rows(&mut ir, raw_rows, "csv", ctx)?;

    Ok(ir)
}

/// Dekodiert Text aus Raw-Bytes unter Berücksichtigung von BOM, UTF-8 und Windows-1252 Fallback.
fn decode_text(input: &[u8], warnings: &mut Vec<ExtractWarning>) -> String {
    if input.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let slice = match input.get(3..) {
            Some(s) => s,
            None => &[],
        };
        return String::from_utf8_lossy(slice).to_string();
    }

    if let Ok(valid_utf8) = std::str::from_utf8(input) {
        return valid_utf8.to_string();
    }

    let (cow, _, had_errors) = encoding_rs::WINDOWS_1252.decode(input);
    if had_errors || !cow.is_empty() {
        warnings.push(ExtractWarning {
            code: WarningCode::EncodingGuessed,
            detail: "Windows-1252 Kodierung erkannt/verwendet".to_string(),
        });
    }

    cow.to_string()
}

/// Bestimmt das wahrscheinlichste Trennzeichen (, ; \t |) über die ersten 100 Zeilen.
fn detect_delimiter(text: &str) -> u8 {
    let candidates = [b',', b';', b'\t', b'|'];
    let lines: Vec<&str> = text.lines().take(100).collect();
    if lines.is_empty() {
        return b',';
    }

    let mut best_delimiter = b',';
    let mut min_variance = f64::MAX;
    let mut max_avg_count = 0.0;

    for &cand in &candidates {
        let mut counts = Vec::new();
        for line in &lines {
            if line.trim().is_empty() {
                continue;
            }
            let count = count_delimiter_outside_quotes(line, cand);
            counts.push(count as f64);
        }

        if counts.is_empty() {
            continue;
        }

        let sum: f64 = counts.iter().sum();
        let avg = sum / (counts.len() as f64);

        if avg < 0.1 {
            continue;
        }

        let variance: f64 =
            counts.iter().map(|c| (c - avg).powi(2)).sum::<f64>() / (counts.len() as f64);

        if variance < min_variance || (variance == min_variance && avg > max_avg_count) {
            min_variance = variance;
            max_avg_count = avg;
            best_delimiter = cand;
        }
    }

    best_delimiter
}

/// Zählt Trennzeichen außerhalb von Anführungszeichen (RFC 4180).
fn count_delimiter_outside_quotes(line: &str, delimiter: u8) -> usize {
    let mut count = 0;
    let mut in_quotes = false;
    let del_char = delimiter as char;

    for ch in line.chars() {
        if ch == '"' {
            in_quotes = !in_quotes;
        } else if ch == del_char && !in_quotes {
            count += 1;
        }
    }

    count
}

/// Teilt Zeilen in Blöcke durch Leerzeilen, schneidet leere Ränder ab und erzeugt `Table`-Blöcke.
pub fn build_tables_from_rows(
    ir: &mut DocumentIr,
    rows: Vec<Vec<String>>,
    sheet_name: &str,
    ctx: &ExtractContext<'_>,
) -> Result<(), IngestError> {
    let mut current_block_rows: Vec<(usize, Vec<String>)> = Vec::new();

    for (row_idx, row) in rows.into_iter().enumerate() {
        let is_empty_row = row.iter().all(|c| c.trim().is_empty());
        if is_empty_row {
            if !current_block_rows.is_empty() {
                process_and_push_table(ir, current_block_rows, sheet_name, ctx)?;
                current_block_rows = Vec::new();
            }
        } else {
            current_block_rows.push((row_idx, row));
        }
    }

    if !current_block_rows.is_empty() {
        process_and_push_table(ir, current_block_rows, sheet_name, ctx)?;
    }

    Ok(())
}

fn process_and_push_table(
    ir: &mut DocumentIr,
    raw_block: Vec<(usize, Vec<String>)>,
    sheet_name: &str,
    ctx: &ExtractContext<'_>,
) -> Result<(), IngestError> {
    if raw_block.is_empty() {
        return Ok(());
    }

    let start_row_idx = match raw_block.first() {
        Some((r, _)) => *r,
        None => 0,
    };

    let mut min_col = usize::MAX;
    let mut max_col = 0;

    for (_, row) in &raw_block {
        for (c_idx, cell) in row.iter().enumerate() {
            if !cell.trim().is_empty() {
                if c_idx < min_col {
                    min_col = c_idx;
                }
                if c_idx > max_col {
                    max_col = c_idx;
                }
            }
        }
    }

    if min_col > max_col {
        return Ok(());
    }

    let mut table_rows = Vec::new();
    let mut non_empty_cell_count: u64 = 0;

    for (_, row) in raw_block {
        let mut trimmed_row = Vec::new();
        for col_idx in min_col..=max_col {
            let val = row.get(col_idx).cloned().unwrap_or_default();
            if !val.trim().is_empty() {
                non_empty_cell_count = non_empty_cell_count.saturating_add(1);
            }
            trimmed_row.push(val);
        }
        table_rows.push(trimmed_row);
    }

    ctx.budget.charge_cells(non_empty_cell_count)?;

    let header_rows = determine_header_rows(&table_rows);

    let origin_r = match u32::try_from(start_row_idx) {
        Ok(v) => v,
        Err(_) => u32::MAX,
    };
    let origin_c = match u32::try_from(min_col) {
        Ok(v) => v,
        Err(_) => u32::MAX,
    };

    let table = Table {
        caption: None,
        header_rows,
        rows: table_rows,
        origin: (origin_r, origin_c),
    };

    ir.push(
        BlockKind::Table(table),
        Locator {
            sheet: Some(sheet_name.to_string()),
            ..Default::default()
        },
    );

    Ok(())
}

/// Bestimmt, ob die erste Zeile eines Tabellenblocks eine Kopfzeile ist.
pub fn determine_header_rows(rows: &[Vec<String>]) -> u8 {
    let first_row = match rows.first() {
        Some(r) => r,
        None => return 0,
    };

    let non_empty_first = first_row.iter().filter(|c| !c.trim().is_empty()).count();
    if non_empty_first == 0 {
        return 0;
    }

    let non_numeric_first = first_row
        .iter()
        .filter(|c| !c.trim().is_empty() && c.parse::<f64>().is_err())
        .count();

    if non_numeric_first * 2 < non_empty_first {
        return 0;
    }

    if rows.len() < 2 {
        return 1;
    }

    if let Some(second_row) = rows.get(1) {
        let second_row_total = second_row.iter().filter(|c| !c.trim().is_empty()).count();
        let second_row_numeric = second_row
            .iter()
            .filter(|c| !c.trim().is_empty() && c.parse::<f64>().is_ok())
            .count();

        if second_row_total > 0 && second_row_numeric * 2 >= second_row_total {
            return 1;
        }
    }

    1
}
