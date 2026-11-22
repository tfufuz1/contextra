// FILE-CONTEXT
// ZWECK: Excel & ODS Extraktionslogik (Xlsx, Xls, Xlsb, Ods) über calamine.
// INVARIANTEN: Zero-Panic, typisierte IngestError, kein unsafe, DoS-Schutz über Dimensionen & Archive.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::{IngestError, LimitKind};
use crate::extractor::ExtractContext;
use crate::format::FormatId;
use crate::ir::{BlockKind, Boundary, DocMeta, DocumentIr, ExtractWarning, Locator, WarningCode};
use calamine::{CellErrorType, Data, Dimensions, Reader, SheetVisible};
use std::io::Cursor;

const CFB_MAGIC: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// Extrahiert Tabellen aus Xlsx, Xls, Xlsb und Ods Dokumenten.
pub fn extract_excel(
    format: FormatId,
    input: &[u8],
    ctx: &ExtractContext<'_>,
) -> Result<DocumentIr, IngestError> {
    if input.starts_with(CFB_MAGIC)
        && (format == FormatId::Xlsx || format == FormatId::Xlsb || format == FormatId::Ods)
    {
        return Err(IngestError::Encrypted {
            format: Some(format),
        });
    }

    if matches!(format, FormatId::Xlsx | FormatId::Xlsb | FormatId::Ods) {
        if let Err(err) = crate::safe::archive::validate_zip_archive(Cursor::new(input), ctx.limits)
        {
            match err {
                IngestError::Corrupt { .. } if input.starts_with(CFB_MAGIC) => {
                    return Err(IngestError::Encrypted {
                        format: Some(format),
                    });
                }
                other => return Err(other),
            }
        }
    }

    match format {
        FormatId::Xlsx => extract_xlsx(input, format, ctx),
        FormatId::Xlsb => extract_xlsb(input, format, ctx),
        FormatId::Ods => extract_ods(input, format, ctx),
        FormatId::Xls => extract_xls(input, format, ctx),
        _ => Err(IngestError::UnsupportedFormat {
            format: Some(format),
            reason: format!("Unerwartetes Format {:?}", format),
        }),
    }
}

fn map_calamine_error(err_str: String, format: FormatId) -> IngestError {
    let lower = err_str.to_lowercase();
    if lower.contains("password") || lower.contains("encrypted") || lower.contains("protected") {
        IngestError::Encrypted {
            format: Some(format),
        }
    } else {
        IngestError::Corrupt {
            format: Some(format),
            detail: err_str,
        }
    }
}

fn extract_xlsx(
    input: &[u8],
    format: FormatId,
    ctx: &ExtractContext<'_>,
) -> Result<DocumentIr, IngestError> {
    let mut wb = calamine::open_workbook_from_rs::<calamine::Xlsx<_>, _>(Cursor::new(input))
        .map_err(|e| map_calamine_error(e.to_string(), format))?;

    let sheets_meta = wb.sheets_metadata().to_vec();
    let sheet_count = u32::try_from(sheets_meta.len()).unwrap_or(u32::MAX);
    let mut ir = create_ir(format, sheet_count);

    for sheet in sheets_meta {
        if !should_process_sheet(&sheet.visible, &sheet.name, &mut ir, ctx) {
            continue;
        }

        if let Ok(cell_reader) = wb.worksheet_cells_reader(&sheet.name) {
            let dim_len = cell_reader.dimensions().len();
            if dim_len > ctx.limits.max_table_cells {
                return Err(IngestError::LimitExceeded {
                    kind: LimitKind::TableCells,
                    limit: ctx.limits.max_table_cells,
                    observed: dim_len,
                });
            }
        }

        let mut range = match wb.worksheet_range(&sheet.name) {
            Ok(r) => r,
            Err(e) => return Err(map_calamine_error(format!("{e:?}"), format)),
        };

        check_range_size(&range, ctx)?;

        if let Some(Ok(merge_cells)) = wb.worksheet_merge_cells(&sheet.name) {
            apply_merge_cells(&mut range, &merge_cells);
        }

        process_range(&mut ir, &range, &sheet.name, ctx)?;
    }

    Ok(ir)
}

fn extract_xlsb(
    input: &[u8],
    format: FormatId,
    ctx: &ExtractContext<'_>,
) -> Result<DocumentIr, IngestError> {
    let mut wb = calamine::open_workbook_from_rs::<calamine::Xlsb<_>, _>(Cursor::new(input))
        .map_err(|e| map_calamine_error(e.to_string(), format))?;

    let sheets_meta = wb.sheets_metadata().to_vec();
    let sheet_count = u32::try_from(sheets_meta.len()).unwrap_or(u32::MAX);
    let mut ir = create_ir(format, sheet_count);

    for sheet in sheets_meta {
        if !should_process_sheet(&sheet.visible, &sheet.name, &mut ir, ctx) {
            continue;
        }

        if let Ok(cell_reader) = wb.worksheet_cells_reader(&sheet.name) {
            let dim_len = cell_reader.dimensions().len();
            if dim_len > ctx.limits.max_table_cells {
                return Err(IngestError::LimitExceeded {
                    kind: LimitKind::TableCells,
                    limit: ctx.limits.max_table_cells,
                    observed: dim_len,
                });
            }
        }

        let range = match wb.worksheet_range(&sheet.name) {
            Ok(r) => r,
            Err(e) => return Err(map_calamine_error(format!("{e:?}"), format)),
        };

        check_range_size(&range, ctx)?;
        process_range(&mut ir, &range, &sheet.name, ctx)?;
    }

    Ok(ir)
}

fn extract_ods(
    input: &[u8],
    format: FormatId,
    ctx: &ExtractContext<'_>,
) -> Result<DocumentIr, IngestError> {
    let mut wb = calamine::open_workbook_from_rs::<calamine::Ods<_>, _>(Cursor::new(input))
        .map_err(|e| map_calamine_error(e.to_string(), format))?;

    let sheets_meta = wb.sheets_metadata().to_vec();
    let sheet_count = u32::try_from(sheets_meta.len()).unwrap_or(u32::MAX);
    let mut ir = create_ir(format, sheet_count);

    for sheet in sheets_meta {
        if !should_process_sheet(&sheet.visible, &sheet.name, &mut ir, ctx) {
            continue;
        }

        let range = match wb.worksheet_range(&sheet.name) {
            Ok(r) => r,
            Err(e) => return Err(map_calamine_error(format!("{e:?}"), format)),
        };

        check_range_size(&range, ctx)?;
        process_range(&mut ir, &range, &sheet.name, ctx)?;
    }

    Ok(ir)
}

fn extract_xls(
    input: &[u8],
    format: FormatId,
    ctx: &ExtractContext<'_>,
) -> Result<DocumentIr, IngestError> {
    let mut wb = calamine::open_workbook_from_rs::<calamine::Xls<_>, _>(Cursor::new(input))
        .map_err(|e| map_calamine_error(e.to_string(), format))?;

    let sheets_meta = wb.sheets_metadata().to_vec();
    let sheet_count = u32::try_from(sheets_meta.len()).unwrap_or(u32::MAX);
    let mut ir = create_ir(format, sheet_count);

    for sheet in sheets_meta {
        if !should_process_sheet(&sheet.visible, &sheet.name, &mut ir, ctx) {
            continue;
        }

        let mut range = match wb.worksheet_range(&sheet.name) {
            Ok(r) => r,
            Err(e) => return Err(map_calamine_error(format!("{e:?}"), format)),
        };

        check_range_size(&range, ctx)?;

        if let Some(merge_cells) = wb.worksheet_merge_cells(&sheet.name) {
            apply_merge_cells(&mut range, &merge_cells);
        }

        process_range(&mut ir, &range, &sheet.name, ctx)?;
    }

    Ok(ir)
}

fn create_ir(format: FormatId, page_count: u32) -> DocumentIr {
    let mut ir = DocumentIr::new(format);
    ir.meta = DocMeta {
        page_count: Some(page_count),
        ..Default::default()
    };
    ir
}

fn should_process_sheet(
    visible: &SheetVisible,
    sheet_name: &str,
    ir: &mut DocumentIr,
    ctx: &ExtractContext<'_>,
) -> bool {
    if *visible != SheetVisible::Visible && !ctx.policy.include_hidden {
        ir.warnings.push(ExtractWarning {
            code: WarningCode::HiddenContentSkipped,
            detail: format!("Verstecktes Blatt '{sheet_name}' übersprungen"),
        });
        return false;
    }

    let next_index = match u32::try_from(ir.blocks.len()) {
        Ok(idx) => idx,
        Err(_) => u32::MAX,
    };

    ir.push(
        BlockKind::Boundary(Boundary::Sheet {
            name: sheet_name.to_string(),
            index: next_index,
        }),
        Locator {
            sheet: Some(sheet_name.to_string()),
            ..Default::default()
        },
    );

    true
}

fn check_range_size(
    range: &calamine::Range<Data>,
    ctx: &ExtractContext<'_>,
) -> Result<(), IngestError> {
    let (height, width) = range.get_size();
    let total_cells = (height as u64).saturating_mul(width as u64);
    if total_cells > ctx.limits.max_table_cells {
        Err(IngestError::LimitExceeded {
            kind: LimitKind::TableCells,
            limit: ctx.limits.max_table_cells,
            observed: total_cells,
        })
    } else {
        Ok(())
    }
}

fn apply_merge_cells(range: &mut calamine::Range<Data>, merge_cells: &[Dimensions]) {
    for merge_dim in merge_cells {
        let Dimensions { start, end } = *merge_dim;
        for r in start.0..=end.0 {
            for c in start.1..=end.1 {
                if (r, c) != start {
                    range.set_value((r, c), Data::Empty);
                }
            }
        }
    }
}

fn process_range(
    ir: &mut DocumentIr,
    range: &calamine::Range<Data>,
    sheet_name: &str,
    ctx: &ExtractContext<'_>,
) -> Result<(), IngestError> {
    let (height, width) = range.get_size();
    let mut rows: Vec<Vec<String>> = Vec::with_capacity(height);

    for row in range.rows() {
        let mut row_strings = Vec::with_capacity(width);
        for cell in row {
            row_strings.push(format_cell_data(cell));
        }
        rows.push(row_strings);
    }

    super::csv_fmt::build_tables_from_rows(ir, rows, sheet_name, ctx)
}

fn format_cell_data(data: &Data) -> String {
    match data {
        Data::Int(v) => v.to_string(),
        Data::Float(v) => v.to_string(),
        Data::String(v) => v.clone(),
        Data::Bool(v) => v.to_string(),
        Data::DateTime(dt) => excel_serial_to_iso8601(dt.as_f64(), false),
        Data::DateTimeIso(v) => v.clone(),
        Data::DurationIso(v) => v.clone(),
        Data::Error(e) => format_cell_error(e),
        Data::Empty => String::new(),
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn excel_serial_to_iso8601(val: f64, is_1904: bool) -> String {
    let days_offset = if is_1904 { 24107.0 } else { 0.0 };
    let mut total_days = (val.floor() + days_offset) as i64;
    let frac = val - val.floor();

    if !is_1904 && total_days < 60 {
        total_days += 1;
    }

    let jdn = total_days + 2415019;

    let l = jdn + 68569;
    let n = (4 * l) / 146097;
    let l = l - (146097 * n + 3) / 4;
    let i = (4000 * (l + 1)) / 1464101;
    let l = l - (1461 * i) / 4 + 31;
    let j = (80 * l) / 2447;
    let day = l - (2447 * j) / 80;
    let l = j / 11;
    let month = j + 2 - (12 * l);
    let year = 100 * (n - 49) + i + l;

    let secs_in_day = (frac * 86400.0).round() as u64;
    let hours = (secs_in_day / 3600) % 24;
    let mins = (secs_in_day % 3600) / 60;
    let secs = secs_in_day % 60;

    if secs_in_day == 0 {
        format!("{year:04}-{month:02}-{day:02}")
    } else {
        format!("{year:04}-{month:02}-{day:02}T{hours:02}:{mins:02}:{secs:02}")
    }
}

fn format_cell_error(e: &CellErrorType) -> String {
    match e {
        CellErrorType::Div0 => "#DIV/0!".to_string(),
        CellErrorType::NA => "#N/A".to_string(),
        CellErrorType::Name => "#NAME?".to_string(),
        CellErrorType::Null => "#NULL!".to_string(),
        CellErrorType::Num => "#NUM!".to_string(),
        CellErrorType::Ref => "#REF!".to_string(),
        CellErrorType::Value => "#VALUE!".to_string(),
        CellErrorType::GettingData => "#DATA!".to_string(),
    }
}
