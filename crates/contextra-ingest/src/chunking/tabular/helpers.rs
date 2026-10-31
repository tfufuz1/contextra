//! Hilfsfunktionen fuer Tabellenaggregation und Header-Wiederholung.
//!
//! # Zweck & Architektur
//! Gehoert zur Segmentierungsschicht (Ring 4) in `contextra-ingest`.
//! Zerlegt die hierarchische `DocumentIr` in semantisch zusammenhaengende,
//! budgetierte Einheiten (Chunks) fuer Embeddings und Volltext-Indexierung.
//!
// FILE-CONTEXT
// ZWECK: Hilfsfunktionen für Spaltenbuchstaben (A–XFD) und A1-Range-Berechnung (WP-ING-13).
// INVARIANTEN: Zero-Panic, deterministisch, reine Ganzzahlarithmetik.
// STAND: TS:2026-10-31T00:00:00Z

/// Konvertiert einen 0-basierten Spaltenindex in Excel-Spaltenbuchstaben (A, B, ..., Z, AA, ..., XFD).
/// Maximal unterstützter Spaltenindex in Excel ist 16383 (XFD).
// ANCHOR[contextra-ingest:CHK-0239]
// fn `col_to_letter` fuer Hilfsfunktionen fuer Tabellenaggregation und Header-Wiederholung.
pub fn col_to_letter(mut col: u32) -> String {
    let mut result = String::new();
    loop {
        let rem = col % 26;
        if let Some(ch) = char::from_u32(u32::from(b'A').saturating_add(rem)) {
            result.push(ch);
        }
        if col < 26 {
            break;
        }
        col = (col / 26).saturating_sub(1);
    }
    result.chars().rev().collect()
}

/// Formatiert den A1-Zellbereich aus `Table.origin`, Header-Anzahl, aktiven Spalten
/// und den 1-basierten Datenzeilen (first, last).
/// Beispiel: origin = (5, 3) [D6], header_rows = 1, active_cols = 0..3, data_rows = (1, 36) => "D6:F42"
// ANCHOR[contextra-ingest:CHK-0240]
// fn `format_a1_range` fuer Hilfsfunktionen fuer Tabellenaggregation und Header-Wiederholung.
pub fn format_a1_range(
    origin: (u32, u32),
    header_rows: u8,
    active_cols: &[usize],
    first_data_row_1based: usize,
    last_data_row_1based: usize,
) -> String {
    let (origin_row, origin_col) = origin;

    let first_rel_col = active_cols
        .first()
        .and_then(|&c| u32::try_from(c).ok())
        .unwrap_or(0);
    let last_rel_col = active_cols
        .last()
        .and_then(|&c| u32::try_from(c).ok())
        .unwrap_or(0);

    let start_col_idx = origin_col.saturating_add(first_rel_col);
    let end_col_idx = origin_col.saturating_add(last_rel_col);

    let start_col_letter = col_to_letter(start_col_idx);
    let end_col_letter = col_to_letter(end_col_idx);

    let first_row_u32 = u32::try_from(first_data_row_1based).unwrap_or(1);
    let last_row_u32 = u32::try_from(last_data_row_1based).unwrap_or(1);

    // Wenn der Chunk bei Datenzeile 1 beginnt und header_rows > 0, beginnt der Bereich bei der Kopfzeile
    let start_sheet_row = if first_data_row_1based == 1 && header_rows > 0 {
        origin_row.saturating_add(1)
    } else {
        origin_row
            .saturating_add(u32::from(header_rows))
            .saturating_add(first_row_u32)
    };

    let end_sheet_row = origin_row
        .saturating_add(u32::from(header_rows))
        .saturating_add(last_row_u32);

    if start_col_letter == end_col_letter && start_sheet_row == end_sheet_row {
        format!("{}{}", start_col_letter, start_sheet_row)
    } else {
        format!(
            "{}{}:{}{}",
            start_col_letter, start_sheet_row, end_col_letter, end_sheet_row
        )
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]
mod tests {
    use super::*;

    #[test]
    fn test_col_to_letter_mapping() {
        assert_eq!(col_to_letter(0), "A");
        assert_eq!(col_to_letter(1), "B");
        assert_eq!(col_to_letter(25), "Z");
        assert_eq!(col_to_letter(26), "AA");
        assert_eq!(col_to_letter(27), "AB");
        assert_eq!(col_to_letter(701), "ZZ");
        assert_eq!(col_to_letter(702), "AAA");
        assert_eq!(col_to_letter(16383), "XFD");
    }

    #[test]
    fn test_format_a1_range_origin_5_3() {
        // origin (5, 3) -> Row 6, Col D
        // 3 active cols (0, 1, 2) -> D, E, F
        // 1 header row, data_rows 1..36 -> end sheet row = 5 + 1 + 36 = 42
        let range1 = format_a1_range((5, 3), 1, &[0, 1, 2], 1, 36);
        assert_eq!(range1, "D6:F42");

        // Chunk 2: data_rows 37..72 -> start row = 5 + 1 + 37 = 43, end row = 5 + 1 + 72 = 78
        let range2 = format_a1_range((5, 3), 1, &[0, 1, 2], 37, 72);
        assert_eq!(range2, "D43:F78");
    }
}
