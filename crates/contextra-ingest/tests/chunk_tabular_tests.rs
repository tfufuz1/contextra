// FILE-CONTEXT
// ZWECK: Akzeptanz- und Property-Tests für Tabular Chunker (WP-ING-13 / P00A / P06).
// INVARIANTEN: Zero-Panic, deterministisch, Spans im kanonischen Text.
// STAND: TS:2026-10-31T00:00:00Z

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use contextra_ingest::chunking::common::token_count;
use contextra_ingest::chunking::config::{ChunkingConfig, TabularMode};
use contextra_ingest::chunking::draft_document;
use contextra_ingest::format::FormatId;
use contextra_ingest::ir::{BlockKind, Boundary, DocumentIr, Locator, Table};
use proptest::prelude::*;

#[test]
fn test_tabular_1_three_column_small_table() {
    let mut ir = DocumentIr::new(FormatId::Xlsx);
    let mut table = Table::default();
    table.caption = Some("Small Sales Table".into());
    table.header_rows = 1;
    table.rows = vec![
        vec!["Produkt".into(), "Anzahl".into(), "Preis".into()],
        vec!["Apfel".into(), "10".into(), "1.50 €".into()],
        vec!["Birne".into(), "5".into(), "2.00 €".into()],
    ];
    table.origin = (0, 0);

    ir.push(
        BlockKind::Boundary(Boundary::Sheet {
            name: "Blatt1".into(),
            index: 0,
        }),
        Locator::default(),
    );
    ir.push(BlockKind::Table(table), Locator::default());

    let canon = ir.canonical();
    let cfg = ChunkingConfig::default();

    let drafts = draft_document(&ir, &canon, &cfg).unwrap();
    assert_eq!(drafts.len(), 1);

    let draft = &drafts[0];
    assert!(draft.text.contains("Sheet: Blatt1 (Zeilen 1–2)"));
    assert!(draft.text.contains("Table: Small Sales Table"));
    assert!(draft.text.contains("| Produkt | Anzahl | Preis |"));
    assert!(draft.text.contains("| Apfel | 10 | 1.50 € |"));

    assert_eq!(draft.locator.sheet.as_deref(), Some("Blatt1"));
    assert_eq!(draft.locator.range.as_deref(), Some("A1:C3"));
    assert_eq!(
        draft.breadcrumb.as_deref(),
        Some("Blatt1 > Small Sales Table")
    );

    assert_eq!(
        draft.extra.get("rows").unwrap(),
        &serde_json::json!({"first": 1, "last": 2})
    );
    assert_eq!(
        draft.extra.get("header").unwrap(),
        &serde_json::json!(["Produkt", "Anzahl", "Preis"])
    );
    assert_eq!(draft.extra.get("synthesized").unwrap(), &true);

    // Span check
    assert!(draft.span.start < draft.span.end);
    assert!(draft.span.start >= canon.block_spans[1].start);
    assert!(draft.span.end <= canon.block_spans[1].end);
}

#[test]
fn test_tabular_2_ten_thousand_rows_row_groups() {
    let mut ir = DocumentIr::new(FormatId::Csv);
    let mut table = Table::default();
    table.header_rows = 1;
    table
        .rows
        .push(vec!["ID".into(), "Name".into(), "Wert".into()]);

    for i in 1..=10_000 {
        table.rows.push(vec![
            i.to_string(),
            format!("Benutzer {}", i),
            format!("{} EUR", i * 10),
        ]);
    }

    ir.push(
        BlockKind::Boundary(Boundary::Sheet {
            name: "Data".into(),
            index: 0,
        }),
        Locator::default(),
    );
    ir.push(BlockKind::Table(table), Locator::default());

    let canon = ir.canonical();
    let mut cfg = ChunkingConfig::default();
    cfg.max_tokens = 256;
    cfg.tabular.mode = TabularMode::RowGroups;

    let hard_limit = (cfg.max_tokens * cfg.hard_limit_pct as usize) / 100;

    let drafts = draft_document(&ir, &canon, &cfg).unwrap();
    assert!(drafts.len() > 10);

    let mut last_seen_row = 0usize;

    for (idx, draft) in drafts.iter().enumerate() {
        assert!(token_count(&draft.text) <= hard_limit);
        assert!(draft.text.contains("| ID | Name | Wert |"));

        let rows_obj = draft.extra.get("rows").unwrap().as_object().unwrap();
        let first = usize::try_from(rows_obj.get("first").unwrap().as_u64().unwrap()).unwrap();
        let last = usize::try_from(rows_obj.get("last").unwrap().as_u64().unwrap()).unwrap();

        // Lückenlos und überschneidungsfrei
        assert_eq!(first, last_seen_row + 1, "Chunk {} first row mismatch", idx);
        assert!(last >= first);
        last_seen_row = last;

        // Span
        assert!(draft.span.start < draft.span.end);
        assert!(draft.span.start >= canon.block_spans[1].start);
        assert!(draft.span.end <= canon.block_spans[1].end);
    }

    assert_eq!(last_seen_row, 10_000);
}

#[test]
fn test_tabular_3_forty_columns_record_rows() {
    let mut ir = DocumentIr::new(FormatId::Xlsx);
    let mut table = Table::default();
    table.header_rows = 1;

    let headers: Vec<String> = (1..=40).map(|c| format!("Kopf_{}", c)).collect();
    table.rows.push(headers);

    for r in 1..=5 {
        let row: Vec<String> = (1..=40).map(|c| format!("W_{}_{}", r, c)).collect();
        table.rows.push(row);
    }

    ir.push(
        BlockKind::Boundary(Boundary::Sheet {
            name: "Wide".into(),
            index: 0,
        }),
        Locator::default(),
    );
    ir.push(BlockKind::Table(table), Locator::default());

    let canon = ir.canonical();
    let mut cfg = ChunkingConfig::default();
    cfg.tabular.mode = TabularMode::Auto; // Wide table => RecordRows

    let drafts = draft_document(&ir, &canon, &cfg).unwrap();
    assert!(!drafts.is_empty());

    let first_text = &drafts[0].text;
    assert!(first_text.contains("Kopf_1: W_1_1"));
    assert!(first_text.contains("; Kopf_2: W_1_2"));
    assert!(!first_text.contains("|---|")); // RecordRows formatting, not GFM table
}

#[test]
fn test_tabular_4_table_without_headers_generic_letters() {
    let mut ir = DocumentIr::new(FormatId::Csv);
    let mut table = Table::default();
    table.header_rows = 0; // Keine Kopfzeilen
    table.rows = vec![
        vec!["Eintrag 1".into(), "Eintrag 2".into()],
        vec!["Eintrag 3".into(), "Eintrag 4".into()],
    ];

    ir.push(
        BlockKind::Boundary(Boundary::Sheet {
            name: "NoHeader".into(),
            index: 0,
        }),
        Locator::default(),
    );
    ir.push(BlockKind::Table(table), Locator::default());

    let canon = ir.canonical();
    let cfg = ChunkingConfig::default();

    let drafts = draft_document(&ir, &canon, &cfg).unwrap();
    assert_eq!(drafts.len(), 1);

    let text = &drafts[0].text;
    assert!(text.contains("| Spalte A | Spalte B |"));
    assert_eq!(
        drafts[0].extra.get("header").unwrap(),
        &serde_json::json!(["Spalte A", "Spalte B"])
    );
}

#[test]
fn test_tabular_5_gigantic_cell_window_split() {
    let mut ir = DocumentIr::new(FormatId::Xlsx);
    let mut table = Table::default();
    table.header_rows = 1;

    let long_text = "A".repeat(20_000);
    table.rows = vec![
        vec!["Titel".into(), "Beschreibung".into()],
        vec!["Projekt X".into(), long_text],
    ];

    ir.push(
        BlockKind::Boundary(Boundary::Sheet {
            name: "BigCell".into(),
            index: 0,
        }),
        Locator::default(),
    );
    ir.push(BlockKind::Table(table), Locator::default());

    let canon = ir.canonical();
    let mut cfg = ChunkingConfig::default();
    cfg.max_tokens = 200;

    let drafts = draft_document(&ir, &canon, &cfg).unwrap();
    assert!(drafts.len() > 1);

    for draft in &drafts {
        assert!(draft.text.contains("Zeilen 1–1"));
        assert!(!draft.text.is_empty());
    }
}

#[test]
fn test_tabular_6_two_sheets_hard_boundary() {
    let mut ir = DocumentIr::new(FormatId::Xlsx);

    let mut table1 = Table::default();
    table1.header_rows = 1;
    table1.rows = vec![vec!["A".into(), "B".into()], vec!["1".into(), "2".into()]];

    let mut table2 = Table::default();
    table2.header_rows = 1;
    table2.rows = vec![vec!["X".into(), "Y".into()], vec!["10".into(), "20".into()]];

    ir.push(
        BlockKind::Boundary(Boundary::Sheet {
            name: "SheetA".into(),
            index: 0,
        }),
        Locator::default(),
    );
    ir.push(BlockKind::Table(table1), Locator::default());

    ir.push(
        BlockKind::Boundary(Boundary::Sheet {
            name: "SheetB".into(),
            index: 1,
        }),
        Locator::default(),
    );
    ir.push(BlockKind::Table(table2), Locator::default());

    let canon = ir.canonical();
    let mut cfg = ChunkingConfig::default();
    cfg.min_tokens = 0; // Kein Zusammenlegen

    let drafts = draft_document(&ir, &canon, &cfg).unwrap();
    assert_eq!(drafts.len(), 2);

    assert_eq!(drafts[0].locator.sheet.as_deref(), Some("SheetA"));
    assert_eq!(drafts[1].locator.sheet.as_deref(), Some("SheetB"));
}

#[test]
fn test_tabular_7_mini_sheets_merging() {
    let mut ir = DocumentIr::new(FormatId::Xlsx);

    let mut table1 = Table::default();
    table1.header_rows = 1;
    table1.rows = vec![
        vec!["K1".into(), "K2".into()],
        vec!["V1".into(), "V2".into()],
    ];

    let mut table2 = Table::default();
    table2.header_rows = 1;
    table2.rows = vec![
        vec!["K3".into(), "K4".into()],
        vec!["V3".into(), "V4".into()],
    ];

    ir.push(
        BlockKind::Boundary(Boundary::Sheet {
            name: "Mini1".into(),
            index: 0,
        }),
        Locator::default(),
    );
    ir.push(BlockKind::Table(table1), Locator::default());

    ir.push(
        BlockKind::Boundary(Boundary::Sheet {
            name: "Mini2".into(),
            index: 1,
        }),
        Locator::default(),
    );
    ir.push(BlockKind::Table(table2), Locator::default());

    let canon = ir.canonical();
    let mut cfg = ChunkingConfig::default();
    cfg.min_tokens = 500; // Auslösen des Zusammenlegens

    let drafts = draft_document(&ir, &canon, &cfg).unwrap();
    assert_eq!(drafts.len(), 1);

    let sheets = drafts[0].extra.get("sheets").unwrap().as_array().unwrap();
    assert_eq!(sheets.len(), 2);
    assert_eq!(sheets[0], "Mini1");
    assert_eq!(sheets[1], "Mini2");
}

#[test]
fn test_tabular_8_skip_empty_rows_and_drop_empty_cols() {
    let mut ir = DocumentIr::new(FormatId::Xlsx);
    let mut table = Table::default();
    table.header_rows = 1;
    table.rows = vec![
        vec!["A".into(), "EmptyCol".into(), "B".into()],
        vec!["1".into(), "".into(), "2".into()],
        vec!["".into(), "   ".into(), "".into()], // leere Zeile
        vec!["3".into(), "".into(), "4".into()],
    ];

    ir.push(
        BlockKind::Boundary(Boundary::Sheet {
            name: "Filtered".into(),
            index: 0,
        }),
        Locator::default(),
    );
    ir.push(BlockKind::Table(table), Locator::default());

    let canon = ir.canonical();
    let mut cfg = ChunkingConfig::default();
    cfg.tabular.skip_empty_rows = true;
    cfg.tabular.drop_empty_columns = true;

    let drafts = draft_document(&ir, &canon, &cfg).unwrap();
    assert_eq!(drafts.len(), 1);

    let draft = &drafts[0];
    assert!(!draft.text.contains("EmptyCol"));
    assert_eq!(
        draft.extra.get("header").unwrap(),
        &serde_json::json!(["A", "B"])
    );
    assert_eq!(
        draft.extra.get("rows").unwrap(),
        &serde_json::json!({"first": 1, "last": 3})
    );
}

#[test]
fn test_tabular_9_origin_5_3_range_d6() {
    let mut ir = DocumentIr::new(FormatId::Xlsx);
    let mut table = Table::default();
    table.header_rows = 1;
    table.origin = (5, 3); // D6
    table.rows = vec![
        vec!["Col1".into(), "Col2".into()],
        vec!["Val1".into(), "Val2".into()],
    ];

    ir.push(
        BlockKind::Boundary(Boundary::Sheet {
            name: "OriginSheet".into(),
            index: 0,
        }),
        Locator::default(),
    );
    ir.push(BlockKind::Table(table), Locator::default());

    let canon = ir.canonical();
    let cfg = ChunkingConfig::default();

    let drafts = draft_document(&ir, &canon, &cfg).unwrap();
    assert_eq!(drafts.len(), 1);

    let range = drafts[0].locator.range.as_deref().unwrap();
    assert!(
        range.starts_with("D6:"),
        "Expected range starting with D6:, got {}",
        range
    );
}

#[test]
fn test_tabular_10_german_umlauts_and_pipe_masking() {
    let mut ir = DocumentIr::new(FormatId::Csv);
    let mut table = Table::default();
    table.header_rows = 1;
    table.rows = vec![
        vec!["Köpfe".into(), "Status | Ausführung".into()],
        vec!["Müller & Söhne".into(), "Aktiv | 100% | OK".into()],
    ];

    ir.push(
        BlockKind::Boundary(Boundary::Sheet {
            name: "Umlaute".into(),
            index: 0,
        }),
        Locator::default(),
    );
    ir.push(BlockKind::Table(table), Locator::default());

    let canon = ir.canonical();
    let cfg = ChunkingConfig::default();

    let drafts = draft_document(&ir, &canon, &cfg).unwrap();
    assert_eq!(drafts.len(), 1);

    let text = &drafts[0].text;
    assert!(text.contains("Müller & Söhne"));
    assert!(text.contains(r"Status \| Ausführung"));
    assert!(text.contains(r"Aktiv \| 100% \| OK"));
}

#[test]
fn test_tabular_searchability_200x8_table() {
    let mut ir = DocumentIr::new(FormatId::Xlsx);
    let mut table = Table::default();
    table.header_rows = 1;

    let headers = vec![
        "Kundennummer".into(),
        "Vorname".into(),
        "Nachname".into(),
        "Stadt".into(),
        "PLZ".into(),
        "Umsatz".into(),
        "Kategorie".into(),
        "Erstellt".into(),
    ];
    table.rows.push(headers.clone());

    for i in 1..=200 {
        table.rows.push(vec![
            format!("K-{:04}", i),
            format!("Vorname_{}", i),
            format!("Nachname_{}", i),
            "Berlin".into(),
            "10115".into(),
            format!("{} EUR", i * 50),
            "Premium".into(),
            "2026-10-31".into(),
        ]);
    }

    ir.push(
        BlockKind::Boundary(Boundary::Sheet {
            name: "Kunden".into(),
            index: 0,
        }),
        Locator::default(),
    );
    ir.push(BlockKind::Table(table), Locator::default());

    let canon = ir.canonical();
    let mut cfg = ChunkingConfig::default();
    cfg.max_tokens = 200; // Erzwingt mehrere Chunks

    let drafts = draft_document(&ir, &canon, &cfg).unwrap();
    assert!(drafts.len() > 1);

    for draft in &drafts {
        for header_name in &headers {
            assert!(
                draft.text.contains(header_name),
                "Chunk missing header context: {}",
                header_name
            );
        }
    }
}

// Proptest Suite
proptest! {
    #[test]
    fn proptest_tabular_row_coverage_and_span_bounds(
        row_count in 1usize..50,
        col_count in 1usize..10,
    ) {
        let mut ir = DocumentIr::new(FormatId::Csv);
        let mut table = Table::default();
        table.header_rows = 1;

        let headers: Vec<String> = (0..col_count).map(|c| format!("Col_{}", c)).collect();
        table.rows.push(headers);

        for r in 1..=row_count {
            let row: Vec<String> = (0..col_count).map(|c| format!("V_{}_{}", r, c)).collect();
            table.rows.push(row);
        }

        ir.push(
            BlockKind::Boundary(Boundary::Sheet {
                name: "PropSheet".into(),
                index: 0,
            }),
            Locator::default(),
        );
        ir.push(BlockKind::Table(table), Locator::default());

        let canon = ir.canonical();
        let cfg = ChunkingConfig::default();

        let drafts1 = draft_document(&ir, &canon, &cfg).unwrap();
        let drafts2 = draft_document(&ir, &canon, &cfg).unwrap();

        // Determinismus
        prop_assert_eq!(&drafts1, &drafts2);

        // Abdeckung aller Datenzeilen
        let mut covered_rows = Vec::new();
        for draft in &drafts1 {
            let rows_obj = draft.extra.get("rows").unwrap().as_object().unwrap();
            let first = usize::try_from(rows_obj.get("first").unwrap().as_u64().unwrap()).unwrap();
            let last = usize::try_from(rows_obj.get("last").unwrap().as_u64().unwrap()).unwrap();

            for r in first..=last {
                covered_rows.push(r);
            }

            // UTF-8 und Spans
            prop_assert!(draft.text.is_ascii() || draft.text.chars().count() > 0);
            prop_assert!(draft.span.start < draft.span.end);
            prop_assert!(draft.span.start >= canon.block_spans[1].start);
            prop_assert!(draft.span.end <= canon.block_spans[1].end);
        }

        let expected_rows: Vec<usize> = (1..=row_count).collect();
        prop_assert_eq!(covered_rows, expected_rows);
    }
}
